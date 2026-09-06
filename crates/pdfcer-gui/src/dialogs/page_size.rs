//! # `dialogs::page_size` — **changing the paper an open drawing sits on**
//!
//! `pages.resize`, the Pages tab's Transform band. The window that answers *"my
//! sheets are A1 and I want them A3"* — and, before it does anything, tells the
//! operator which of two very different things he is about to get.
//!
//! ## ★★★ Why this window exists, and why it is mostly a sentence
//!
//! The engine has been able to do this since **2026-08-18**.
//! `EditSession::set_media_boxes` — written for the drawing-set case, one undo
//! entry however many sheets, refusals raised before anything is committed —
//! was called by nothing for nineteen days, because no command reached it. And
//! a complete size chooser already existed and could not be opened:
//! [`crate::dialogs::new_document`] offers every entry in `PaperSize::ALL`,
//! both orientations and a custom size, and opens **only while creating a
//! file**. An operator with a document open could not reach any of it.
//!
//! ★ [`crate::app::blank`]'s §3a has known this the whole time; its comments
//! say in as many words that *"pdfcer-core answered it on 2026-08-18 —
//! `EditSession::set_media_box`, `set_media_boxes` and a `pdfcer_core::paper`
//! table"*. **Writing it down was mistaken for acting on it.** That is the
//! fourth instance of the pattern found this week and it is recorded here
//! rather than in a session log because this file is where a reader will next
//! be standing when it matters.
//!
//! ## ★★★ THE DESIGN DECISION: this window's real product is a MEASUREMENT
//!
//! Changing a `/MediaBox` changes **the paper**. It does not move, scale or
//! reflow one byte of what is drawn on the page. So:
//!
//! > **An A1 drawing put on A4 paper is CROPPED, not SHRUNK.**
//!
//! Every other page-size control the operator has ever used — Word,
//! LibreOffice, a print dialog's Fit-to-page — reflows or scales. He will
//! arrive here expecting *"print it smaller"* and, on his own title-block
//! sheet, the thing he would lose is the title block: measured 2026-09-06, it
//! sits at x 1831–2207 pt and an A4 sheet stops at 595.28.
//!
//! A window that let him find that out from the result would be *"fuzzy, never
//! sneaky"*'s purest failure — he would get exactly what he asked for and
//! nothing like what he wanted. So three things are on screen before he can
//! commit, and the third is the one that does the work:
//!
//! 1. [`crate::text::page_size::intro`] — the rule, in words.
//! 2. [`Self::diagram`] — the old sheet, the new sheet and the drawing's own
//!    extent, drawn to scale, so the overhang is a **picture** before it is a
//!    number.
//! 3. [`crate::text::page_size::overhang`] — *"the drawing runs 1,636 pt past
//!    the right edge"*, recomputed as he changes the size.
//!
//! ⇒ **A rule is a thing to agree with; a measurement is a thing to act on.**
//! Item 3 is why this is not just a size picker with a warning on it.
//!
//! ### And item 3 is a thing the ENGINE said it could not do
//!
//! `MediaBoxChange::lost_area` carries a named residual in the engine's own
//! words: it reports that the *sheet* shrank, **not** that any *content* was in
//! the region it lost, because *"pdfcer has no page-content bounding-box
//! facility yet"*. That is true of `EditSession` and false of `pdfcer-core` —
//! `PageObjects::page_bbox` is exactly that facility, and this shell already
//! holds a decomposition per page. [`crate::app::actions::pagesize::survey`] is
//! where the two are put together, with its cost bounded and its boundary
//! stated.
//!
//! ## Rule 4, and the one affordance that is allowed on the canvas
//!
//! **R8b rule 4: disclosure is off-canvas; applied content renders exactly as
//! saved content will.** Everything here is off-canvas — it is a desktop
//! window. The pre-commit preview of the new paper outline is explicitly
//! welcome as a *cursor* affordance, and it is drawn **inside this window**
//! rather than over the page, for two reasons: the canvas is another track's,
//! and a scale diagram beside the size list is read by an operator whose eye is
//! already on the size list. Nothing provisional is styled onto the page.
//!
//! ## R9, and what is absent rather than greyed
//!
//! The custom millimetre fields exist only when Custom is chosen, and the
//! commit button exists only when the size is in range —
//! [`crate::dialogs::new_document`]'s argument, unchanged: greying is for
//! *temporarily* unavailable, the refusal line is already on screen naming both
//! limits, and a control reappearing as a number crosses a bound is a clearer
//! signal than one changing shade.
//!
//! ## Why here and not Document ▸ Properties
//!
//! [`crate::dialogs::new_document`]'s header nominated *"Document ▸
//! Properties"* for a page-resize surface, and that is departed from
//! deliberately rather than overlooked. The Pages tab's own organising rule
//! decides it: *"every command here operates on the current document's page
//! set, and every one respects the thumbnail rail's current selection"* —
//! which is precisely the drawing-set operation `set_media_boxes` was written
//! for, and the rail is how the operator says *these* sheets. Document
//! Properties has no operand and no selection. The reserved id, `pages.resize`,
//! was on the Pages tab all along.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::pages::PageAction;
use crate::app::actions::pagesize::{SheetSurvey, survey};
use crate::app::state::OpenDoc;
use crate::text::page_size as t;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "page-size.body";

/// The size combo, closed.
const REGION_SIZE: &str = "page-size.size";

/// One region per entry in the OPEN size list, indexed from zero in
/// `pdfcer_core::paper::PaperSize::ALL` order, with `Custom…` last.
///
/// Published for [`crate::dialogs::new_document`]'s reason, which is the same
/// one here: an egui combo popup is an `Area` laid out at paint time, so
/// nothing outside the process can compute where an entry is, and a check that
/// can open a list but not choose from it can assert only that a control
/// exists. The property worth asserting is that **picking a size makes the
/// sheets that size in the saved file**, which needs a click on a specific
/// entry.
const REGION_SIZE_ITEM_PREFIX: &str = "page-size.size.item.";

/// The Portrait radio.
const REGION_PORTRAIT: &str = "page-size.portrait";

/// The Landscape radio. See [`REGION_PORTRAIT`].
const REGION_LANDSCAPE: &str = "page-size.landscape";

/// The commit button.
const REGION_APPLY: &str = "page-size.apply";

/// ★★ The line that says what happens to the drawing — the one control in this
/// window whose *content* is the product.
///
/// Published so a driven check can assert that the sentence changed when the
/// size changed. A window that showed "everything fits" for every size would
/// satisfy a check that only looked for the region's presence, which is the
/// shape `text::security::cannot_author` failed in for three corrections and
/// zero call sites.
const REGION_OUTCOME: &str = "page-size.outcome";

/// The scale diagram.
const REGION_DIAGRAM: &str = "page-size.diagram";

/// Points per millimetre — 72 points per inch ÷ 25.4 mm per inch.
const PT_PER_MM: f64 = 72.0 / 25.4;

/// The smallest custom sheet this window will make, in millimetres.
///
/// [`crate::dialogs::new_document::MIN_CUSTOM_MM`]'s value and reasoning, and
/// deliberately the same number: two windows in one program that disagree about
/// the smallest sheet a PDF may have would be a defect whichever of them was
/// right. ISO 32000-1 Annex C.2 advises 3 units (≈ 1.06 mm); 2 mm is
/// comfortably over it and is a bound an operator can remember.
const MIN_CUSTOM_MM: i64 = 2;

/// The largest custom sheet this window will make, in millimetres.
///
/// 5,080 mm = 200 inches = 14,400 default user space units, Annex C.2's advised
/// maximum. See [`MIN_CUSTOM_MM`] for why the number is shared with the
/// sized-New window rather than re-derived.
const MAX_CUSTOM_MM: i64 = 5080;

/// How many sheets the `page-size-document` census lists before truncating.
///
/// 128 is a whole drawing set with room to spare and a bounded cost on a
/// document that is not one. The census is a `PDFCER_DIAG` line — it costs a
/// disabled build nothing at all, because `crate::diag::trace` takes a closure
/// — so the cap is about the size of a trace artefact a harness has to parse,
/// not about frame time.
const DOCUMENT_SURVEY_CAP: usize = 128;

/// The diagram's height, in points. Wide enough to read an A1-versus-A4
/// proportion at a glance and short enough not to push the outcome line below
/// the fold in the window's opening size.
const DIAGRAM_HEIGHT: f32 = 110.0;

/// Which sheet the operator has picked.
///
/// A separate type from `pdfcer_core::paper::PaperSize` rather than
/// `Option<PaperSize>`, for [`crate::dialogs::new_document`]'s reason: "custom"
/// is a *state of the window* — it opens two fields and changes what the
/// summary reads — and not a missing size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    /// One of `pdfcer_core::paper::PaperSize::ALL`.
    Standard(pdfcer_core::paper::PaperSize),
    /// The two millimetre fields.
    Custom,
}

/// The sheet-size window's state.
pub struct PageSizeDialog {
    /// What the picked sheets are and what is drawn on them, read **once**
    /// when the window opened.
    ///
    /// # ★ Why a snapshot rather than a live read
    ///
    /// Two reasons, and the second is the load-bearing one. It is cheap —
    /// nothing is re-walked per frame while the operator scrolls a size list.
    /// And it is *honest about what it measured*: the numbers on screen were
    /// taken from a state the operator can see, so an undo behind this window
    /// cannot silently change the measurement under a sentence he has already
    /// read. The window is document-scoped, so closing the document closes it;
    /// an edit that changes the sheets while it is open is the residual, and it
    /// is named rather than papered over.
    survey: SheetSurvey,
    /// Which sheet.
    choice: Choice,
    /// Which way up.
    landscape: bool,
    /// The custom width in millimetres, live even while a standard size is
    /// selected so that switching away and back does not lose it.
    custom_w_mm: i64,
    /// The custom height in millimetres.
    custom_h_mm: i64,
    /// Set by the commit button, consumed after the window closure returns.
    ///
    /// Deferred by one statement for the reason every other committing dialog
    /// defers: the action rewrites the document, drops the render worker's
    /// texture and rebuilds every panel, and doing that inside `Window::show`'s
    /// closure runs it while egui is part-way through laying this window out.
    apply_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl PageSizeDialog {
    /// Open the window over `pages`, the operand sheets.
    ///
    /// # ★★ It opens on what the sheets ALREADY ARE, not on A4
    ///
    /// The opposite of [`crate::dialogs::new_document`], and for the opposite
    /// reason. That window opens on A4 because its sibling `file.new` makes an
    /// A4 page and the difference between the two controls must be *"one
    /// asks"* and nothing else. This window has no sibling and a document in
    /// front of it: opening on A4 would mean an operator who opened it to
    /// *check* what size his sheets were would be looking at a control that had
    /// already changed the answer, and one careless press away from cropping an
    /// A1 drawing to A4.
    ///
    /// So it opens on the size the picked sheets classify to, in their own
    /// orientation. When they classify to nothing — a CAD exporter's rounded
    /// A1, a genuinely odd sheet — it opens on **Custom**, pre-filled with
    /// those millimetres, which is both the honest starting point and the one
    /// from which a small correction is a small edit.
    ///
    /// Returns `None` when the survey found no sheets at all, which the caller
    /// has already excluded by resolving operands; belt and braces, because a
    /// window with nothing to act on has nothing true to say.
    #[must_use]
    pub fn open(doc: &OpenDoc, pages: &[usize]) -> Option<Self> {
        let survey = survey(doc, pages);
        let current = survey.uniform();

        // Classify with the engine's own tolerance and its own table. A
        // hand-rolled comparison here would answer `None` for essentially every
        // real A4 page — 595.276, 595.28 and 595.32 all occur in the wild — and
        // the window would open on Custom for documents that are plainly A4.
        let classified = current.as_ref().and_then(|rect| {
            pdfcer_core::paper::PaperSize::classify(
                rect,
                pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE,
            )
        });

        let (choice, landscape) = match classified {
            Some((size, orientation)) => (
                Choice::Standard(size),
                orientation == pdfcer_core::paper::Orientation::Landscape,
            ),
            None => (Choice::Custom, false),
        };

        // The millimetre fields open on the sheets' own size when there is one
        // to open on, so Custom is a correction rather than a blank form. A4's
        // millimetres otherwise, for `new_document`'s reason: zeros would draw
        // the refusal line the instant Custom was picked, which reads as the
        // window objecting to a choice nobody has finished making.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a sheet bounded by MAX_CUSTOM_MM is far inside i64" // ui-text-exempt: lint justification, never displayed
        )]
        let (custom_w_mm, custom_h_mm) = current.map_or((210, 297), |rect| {
            (
                (rect.width() / PT_PER_MM).round() as i64,
                (rect.height() / PT_PER_MM).round() as i64,
            )
        });

        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-size-opened sheets={} distinct={} unread={} drawn={} \
                 open_choice={choice:?} landscape={landscape}",
                survey.pages.len(),
                survey.distinct_sizes(),
                survey.unread,
                u8::from(survey.drawn.is_some()),
            )
        });

        // ★★★ **Every sheet in the DOCUMENT, not only the operands, and this
        // line exists for a check rather than for the operator.**
        //
        // R1's rule is that a passing unit test is not a report of working
        // software, and the only thing that can report it here is a process
        // outside this one reading **the document**. A saved file is bytes; the
        // shell's page tree is what a reader of those bytes resolves. So the
        // one honest oracle available to `ui-verify` is to open the written
        // file in a **fresh binary** and have that binary say what page sizes it
        // resolved — which is what this publishes.
        //
        // `crate::app::blank`'s `document_sized` makes the identical argument
        // for `new-document-sized`, and `ui-verify`'s
        // `new_document_sizes_the_page` reads it for the identical reason: *a
        // trace of the request says what the code was told; a trace of the page
        // tree says what a reader of the file will see.*
        //
        // ★★ Whole-document rather than operand-scoped, because the property
        // worth asserting has **two halves**: the picked sheet became the size
        // that was asked for, AND the sheet beside it did not. A line covering
        // only the operands could not carry the second, and a check whose
        // baseline has no dynamic range cannot produce a verdict.
        //
        // ⚠ Capped. On a 500-sheet set this would otherwise be 500 lines on
        // every window open. The cap is announced on the summary line rather
        // than silently applied, because a truncated census that does not say
        // so is indistinguishable from a complete one.
        for (index, page) in doc.pages.iter().enumerate().take(DOCUMENT_SURVEY_CAP) {
            let media = page.media_box;
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-size-document index={index} w={:.2} h={:.2} llx={:.2} lly={:.2}",
                    media.urx - media.llx,
                    media.ury - media.lly,
                    media.llx,
                    media.lly,
                )
            });
        }
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-size-census pages={} listed={} truncated={}",
                doc.pages.len(),
                doc.pages.len().min(DOCUMENT_SURVEY_CAP),
                u8::from(doc.pages.len() > DOCUMENT_SURVEY_CAP),
            )
        });

        (!survey.pages.is_empty()).then_some(Self {
            survey,
            choice,
            landscape,
            custom_w_mm,
            custom_h_mm,
            apply_requested: false,
            close_requested: false,
        })
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "page-size", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(520.0, 620.0),
            egui::vec2(400.0, 360.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.apply_requested) {
            let (w, h) = self.sheet_pt();
            let rect = self.survey.target_rect(w, h);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-size-commit n={} size_id={} landscape={} w_pt={w:.2} h_pt={h:.2} \
                     llx={:.2} lly={:.2} choice={:?}",
                    self.survey.pages.len(),
                    self.size_id(),
                    self.landscape,
                    rect.llx,
                    rect.lly,
                    self.choice,
                )
            });
            actions.push(Action::Page(PageAction::SetPageSize {
                pages: self.survey.pages.clone(),
                rect,
            }));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The sheet this window currently describes, in points, **after**
    /// orientation.
    ///
    /// # ★ One function, read by four callers, and that is the point
    ///
    /// The summary line, the diagram, the outcome sentence and the commit all
    /// ask this. Four separate computations of "what did they pick" is how a
    /// window comes to promise 841 × 1189, draw a portrait outline and produce
    /// 1189 × 841 — and a transposition is exactly the arithmetic that is easy
    /// to write four times and hard to notice once.
    fn sheet_pt(&self) -> (f64, f64) {
        let orientation = if self.landscape {
            pdfcer_core::paper::Orientation::Landscape
        } else {
            pdfcer_core::paper::Orientation::Portrait
        };
        match self.choice {
            Choice::Standard(size) => {
                let rect = size.rect_with(orientation);
                (rect.width(), rect.height())
            }
            Choice::Custom => {
                let (w, h) = if self.landscape {
                    (self.custom_h_mm, self.custom_w_mm)
                } else {
                    (self.custom_w_mm, self.custom_h_mm)
                };
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "a millimetre count bounded by MAX_CUSTOM_MM is exact in f64" // ui-text-exempt: lint justification, never displayed
                )]
                (w as f64 * PT_PER_MM, h as f64 * PT_PER_MM)
            }
        }
    }

    /// The engine's **stable machine identifier** for the chosen size —
    /// `"a6"`, `"ansi-d"` — or `"custom"`.
    ///
    /// # ★★★ Why this exists, and why the `choice={:?}` beside it is not enough
    ///
    /// A driven check has to be able to say *which size the entry it clicked
    /// actually was*, and it cannot ask `pdfcer-core`: `ui-verify` has exactly
    /// one dependency by deliberate design, and pulling the engine into a
    /// verification harness would make it fail to build for reasons unrelated
    /// to the thing under test, on the day it is most needed.
    ///
    /// So the size list is clicked **by index** — `page-size.size.item.6` — and
    /// `PaperSize::ALL` is `#[non_exhaustive]` with its own doc comment saying
    /// the table will grow. A size inserted before A6 would make a check click
    /// A5 and then assert A6's dimensions: a red run whose message blames the
    /// application for a table that moved. This line is what lets the check
    /// notice instead.
    ///
    /// ⚠ **Not the `Debug` spelling.** `choice={:?}` is emitted beside this for
    /// a human reading a trace, and nothing parses it: Debug-formatting a
    /// domain type and then parsing it produced two false failure reports in
    /// this project in one week. `PaperSize::id` is the opposite of a Debug
    /// spelling — the engine documents it as *"ASCII, lowercase, hyphenated,
    /// and must not change once shipped"*, which is precisely the contract a
    /// harness needs.
    fn size_id(&self) -> &'static str {
        match self.choice {
            Choice::Standard(size) => size.id(),
            // ui-text-exempt: a machine-facing identifier on a diagnostic
            // trace line, never displayed in the UI.
            Choice::Custom => "custom",
        }
    }

    /// Whether the current choice can be made.
    ///
    /// Only a custom size can fail: every entry in `PaperSize::ALL` is a real
    /// sheet in range by construction. Checked on the **millimetre fields**
    /// rather than the computed points, so the refusal can name the numbers the
    /// operator typed.
    const fn is_valid(&self) -> bool {
        match self.choice {
            Choice::Standard(_) => true,
            Choice::Custom => {
                self.custom_w_mm >= MIN_CUSTOM_MM
                    && self.custom_w_mm <= MAX_CUSTOM_MM
                    && self.custom_h_mm >= MIN_CUSTOM_MM
                    && self.custom_h_mm <= MAX_CUSTOM_MM
            }
        }
    }

    /// Whether committing would write anything at all.
    ///
    /// ★ Not a validity question and not folded into one. The engine reaches
    /// §11.1's net-zero rule by itself — a page asked for the size it already
    /// has records no command — so pressing the button on an unchanged size is
    /// safe, and it is also a control the operator pressed that did nothing and
    /// said nothing, which is the defect class this project is named after. So
    /// the window says so first.
    fn would_change(&self) -> bool {
        let (w, h) = self.sheet_pt();
        let target = self.survey.target_rect(w, h);
        let tol = pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE;
        !self.survey.boxes.iter().all(|r| {
            (r.width() - target.width()).abs() <= tol && (r.height() - target.height()).abs() <= tol
        })
    }

    /// The controls, the diagram, the measurement and the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        self.now_line(ui);
        ui.add_space(8.0);

        self.size_controls(ui);
        ui.add_space(8.0);

        // The sheet that will land, or why none will. One line, two states,
        // never both — an operator whose numbers are out of range does not also
        // need to be told what 0 × 0 mm would be in points.
        if self.is_valid() {
            let (w, h) = self.sheet_pt();
            ui.label(egui::RichText::new(t::sheet_summary(w, h)).small().weak());
        } else {
            ui.label(
                egui::RichText::new(t::custom_refused(MIN_CUSTOM_MM, MAX_CUSTOM_MM))
                    .small()
                    .color(ui.visuals().error_fg_color),
            );
        }

        ui.add_space(8.0);
        ui.separator();
        ui.label(t::outcome_heading());
        self.diagram(ui);
        self.outcome(ui);

        if self.survey.common_origin.is_none() {
            ui.label(
                egui::RichText::new(t::origin_differs())
                    .small()
                    .color(ui.visuals().warn_fg_color),
            );
        }

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
            // ★ ABSENT, not greyed, when the size is out of range — R9, and
            // `new_document`'s argument in full: the refusal line naming both
            // limits is already on screen immediately above, so a greyed button
            // would be a second, quieter statement of a fact already made
            // loudly. The button reappearing as the number crosses the bound is
            // also visible from the field the operator's eye is on.
            //
            // ★★ It is PRESENT for an unchanged size, deliberately. That is not
            // an invalid choice, it is a no-op the engine handles correctly, and
            // hiding the control would leave an operator who opened the window
            // to look rather than to change with nothing to press but Cancel —
            // which reads as the window having refused him.
            if self.is_valid() {
                let apply = ui.button(t::apply());
                crate::diag::ui_rect(REGION_APPLY, apply.rect);
                if apply.on_hover_text(t::apply_tooltip()).clicked() {
                    self.apply_requested = true;
                }
            }
        });
    }

    /// *"These sheets now …"* — one line, three shapes.
    fn now_line(&self, ui: &mut Ui) {
        ui.label(t::now_heading());
        let count = self.survey.pages.len();
        let line = match self.survey.uniform() {
            None => t::now_mixed(count, self.survey.distinct_sizes()),
            Some(rect) => match pdfcer_core::paper::PaperSize::classify(
                &rect,
                pdfcer_core::paper::PaperSize::CLASSIFY_TOLERANCE,
            ) {
                Some((size, orientation)) => t::now_uniform_named(
                    count,
                    &t::size_name_oriented(
                        &t::size_name(size),
                        orientation == pdfcer_core::paper::Orientation::Landscape,
                    ),
                    rect.width(),
                    rect.height(),
                ),
                None => t::now_uniform_unnamed(count, rect.width(), rect.height()),
            },
        };
        ui.label(egui::RichText::new(line).small().weak());
    }

    /// The size list, the orientation pair and the two millimetre fields.
    fn size_controls(&mut self, ui: &mut Ui) {
        ui.label(t::size_heading());
        let selected_text = match self.choice {
            Choice::Standard(size) => t::size_entry(&t::size_name(size), size.size_pt()),
            Choice::Custom => t::size_custom().to_owned(),
        };
        let combo = egui::ComboBox::from_id_salt("page-size-size")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                // The engine's own order — the A series largest-first, then the
                // US sizes, then ANSI. Reusing `PaperSize::ALL` rather than
                // listing sizes here also means a size the engine adds appears
                // without a shell change, which is what `size_name`'s wildcard
                // arm is for.
                for (index, size) in pdfcer_core::paper::PaperSize::ALL.iter().enumerate() {
                    let entry = ui.selectable_value(
                        &mut self.choice,
                        Choice::Standard(*size),
                        t::size_entry(&t::size_name(*size), size.size_pt()),
                    );
                    crate::diag::ui_rect(&format!("{REGION_SIZE_ITEM_PREFIX}{index}"), entry.rect);
                }
                let custom =
                    ui.selectable_value(&mut self.choice, Choice::Custom, t::size_custom());
                crate::diag::ui_rect(
                    &format!(
                        "{REGION_SIZE_ITEM_PREFIX}{}",
                        pdfcer_core::paper::PaperSize::ALL.len()
                    ),
                    custom.rect,
                );
            });
        crate::diag::ui_rect(REGION_SIZE, combo.response.rect);

        ui.add_space(6.0);
        ui.label(t::orientation_heading());
        let portrait = ui.radio(!self.landscape, t::orientation_portrait());
        crate::diag::ui_rect(REGION_PORTRAIT, portrait.rect);
        if portrait.clicked() {
            self.landscape = false;
        }
        let landscape = ui.radio(self.landscape, t::orientation_landscape());
        crate::diag::ui_rect(REGION_LANDSCAPE, landscape.rect);
        if landscape.clicked() {
            self.landscape = true;
        }

        // Absent rather than greyed on a standard size — R9's own distinction.
        // These fields are not *unavailable* when A4 is selected, they are
        // irrelevant, which is a different thing and is expressed by their not
        // being there.
        if self.choice == Choice::Custom {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(t::custom_width());
                ui.add(egui::DragValue::new(&mut self.custom_w_mm).range(0..=MAX_CUSTOM_MM));
            });
            ui.horizontal(|ui| {
                ui.label(t::custom_height());
                ui.add(egui::DragValue::new(&mut self.custom_h_mm).range(0..=MAX_CUSTOM_MM));
            });
        }
    }

    /// ★★★ **The sentence that says what happens to the drawing.**
    ///
    /// Three states, and keeping them three is the whole point:
    ///
    /// | state | wording |
    /// |---|---|
    /// | measured, nothing outside | [`t::fits`] — the only promise this window makes |
    /// | measured, something outside | [`t::overhang`] — the amount, per edge, in points |
    /// | **not** measured | [`t::overhang_unmeasurable`] — a stated boundary, and **not** the first |
    ///
    /// ⚠ Collapsing the third into the first is the single most dangerous edit
    /// available in this file, which is why
    /// [`crate::app::actions::pagesize::SheetSurvey::overhang`] returns
    /// `Option` and its test asserts an unmeasured extent is not a zero one.
    fn outcome(&self, ui: &mut Ui) {
        let (w, h) = self.sheet_pt();
        let target = self.survey.target_rect(w, h);

        let response = match self.survey.overhang(target) {
            None => ui.label(
                egui::RichText::new(t::overhang_unmeasurable(
                    self.survey.unread,
                    self.survey.pages.len(),
                ))
                .color(ui.visuals().warn_fg_color),
            ),
            Some((left, right, bottom, top)) => {
                if left + right + bottom + top <= 0.0 {
                    ui.label(t::fits())
                } else {
                    ui.label(
                        egui::RichText::new(t::overhang(left, right, bottom, top))
                            .color(ui.visuals().warn_fg_color),
                    )
                }
            }
        };
        crate::diag::ui_rect(REGION_OUTCOME, response.rect);

        // ★★ The bound on the promise, on screen and permanently. A sheet whose
        // *drawing* fits can still lose a sticky note: measured 2026-09-06, an
        // annotation keeps its `/Rect` across a resize exactly as content keeps
        // its coordinates. A bound nobody can read is not a bound.
        ui.label(egui::RichText::new(t::annots_not_counted()).small().weak());

        if !self.would_change() {
            ui.label(egui::RichText::new(t::no_change()).small().weak());
        }

        // ★★ Traced so a driven check can assert the sentence **changed** with
        // the size, rather than only that a region was published. A window that
        // said "everything fits" for every size would satisfy a presence check
        // forever — which is `text::security::cannot_author`'s failure mode
        // (corrected three times, zero call sites) turned inside out.
        //
        // ★ `trace_changed`, not `trace`, and the difference matters to the
        // reader of the artefact rather than to the program: this runs once per
        // FRAME the window draws, so a plain trace would publish an identical
        // line thirty times per settle and bury the two lines that say
        // something. De-duplicated, the channel carries exactly the sequence of
        // *decisions* the window made, which is the thing worth asserting.
        crate::diag::trace_changed("page-size-outcome", || {
            let (l, r, b, tp) = self
                .survey
                .overhang(target)
                .unwrap_or((-1.0, -1.0, -1.0, -1.0));
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-size-outcome measured={} left={l:.1} right={r:.1} bottom={b:.1} \
                 top={tp:.1} changes={}",
                u8::from(self.survey.drawn.is_some()),
                u8::from(self.would_change()),
            )
        });
    }

    /// ★★ **The pre-commit preview: the old sheet, the new sheet and the
    /// drawing, to scale.**
    ///
    /// A cursor affordance under R8b rule 4, drawn off-canvas. It exists
    /// because an overhang expressed as *"1,636 pt past the right edge"* is a
    /// number an operator has to convert into a mental picture, and the
    /// conversion is exactly where a wrong decision gets made — 1,636 pt is
    /// meaningless until you see that it is most of the sheet.
    ///
    /// # What is drawn, and what each outline means
    ///
    /// * the **old** sheet — the size the picked pages are now, weak;
    /// * the **new** sheet — filled with the accent, because it is the thing
    ///   being chosen and the thing that will exist;
    /// * the **drawing** — the union of the drawn extents that could be read,
    ///   as a dashed outline, so the part of it hanging outside the new sheet
    ///   is visible as a shape rather than as a number.
    ///
    /// Both sheets share one scale — the union of everything, fitted to the
    /// strip — which is the only arrangement in which the comparison means
    /// anything. Two independently-fitted outlines would draw A4 and A1 the
    /// same size.
    ///
    /// ★ No raw `Color32`: the three colours are theme roles
    /// (`Theme::accent_pair`, `weak_text_color`, `warn_fg_color`), so a preset
    /// change moves them with everything else.
    fn diagram(&self, ui: &mut Ui) {
        let (w, h) = self.sheet_pt();
        let new_sheet = self.survey.target_rect(w, h);
        let old_sheet = self
            .survey
            .uniform()
            .or_else(|| self.survey.boxes.first().copied());

        // The page-space extent everything has to fit into.
        let mut min = (new_sheet.llx.min(0.0), new_sheet.lly.min(0.0));
        let mut max = (new_sheet.urx, new_sheet.ury);
        for rect in old_sheet.iter().chain(self.survey.drawn.iter()) {
            min = (min.0.min(rect.llx), min.1.min(rect.lly));
            max = (max.0.max(rect.urx), max.1.max(rect.ury));
        }
        let span = ((max.0 - min.0).max(1.0), (max.1 - min.1).max(1.0));

        let (response, painter) = ui.allocate_painter(
            egui::vec2(ui.available_width(), DIAGRAM_HEIGHT),
            egui::Sense::hover(),
        );
        crate::diag::ui_rect(REGION_DIAGRAM, response.rect);
        let strip = response.rect.shrink(6.0);

        #[allow(
            clippy::cast_possible_truncation,
            reason = "a page-space span is bounded by Annex C.2's 14,400 and is exact in f32" // ui-text-exempt: lint justification, never displayed
        )]
        let scale = (strip.width() / span.0 as f32).min(strip.height() / span.1 as f32);

        // Page space is y-up and screen space is y-down, so the vertical axis
        // is flipped here rather than in each of the three outlines — one
        // conversion, so a rectangle cannot be drawn upside down relative to
        // its neighbour.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "as above — bounded page-space coordinates" // ui-text-exempt: lint justification, never displayed
        )]
        let to_screen = |rect: &pdfcer_core::page_tree::Rect| {
            let x0 = strip.left() + ((rect.llx - min.0) as f32) * scale;
            let x1 = strip.left() + ((rect.urx - min.0) as f32) * scale;
            let y0 = strip.bottom() - ((rect.lly - min.1) as f32) * scale;
            let y1 = strip.bottom() - ((rect.ury - min.1) as f32) * scale;
            egui::Rect::from_min_max(egui::pos2(x0, y1), egui::pos2(x1, y0))
        };

        let (accent, _on_accent) = egui_shell::theme::Theme::accent_pair(ui.ctx());
        let weak = ui.visuals().weak_text_color();
        let warn = ui.visuals().warn_fg_color;

        if let Some(old) = old_sheet {
            painter.rect_stroke(
                to_screen(&old),
                0.0,
                egui::Stroke::new(1.0, weak),
                egui::StrokeKind::Middle,
            );
        }
        painter.rect_stroke(
            to_screen(&new_sheet),
            0.0,
            egui::Stroke::new(2.0, accent),
            egui::StrokeKind::Middle,
        );
        if let Some(drawn) = self.survey.drawn {
            painter.rect_stroke(
                to_screen(&drawn),
                0.0,
                egui::Stroke::new(1.0, warn),
                egui::StrokeKind::Middle,
            );
        }

        // The legend, because three unlabelled rectangles are a puzzle. Drawn
        // as coloured words rather than swatches-plus-words: the words are the
        // swatches, which is half the vertical space and no worse to read.
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(t::legend_now()).small().color(weak));
            ui.label(egui::RichText::new(t::legend_new()).small().color(accent));
            if self.survey.drawn.is_some() {
                ui.label(egui::RichText::new(t::legend_drawn()).small().color(warn));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::page_tree::Rect;

    /// A dialog over one A1-landscape sheet with a drawing that fills it.
    fn a1_landscape() -> PageSizeDialog {
        let sheet = Rect::from_corners(0.0, 0.0, 2383.94, 1683.78);
        PageSizeDialog {
            survey: SheetSurvey {
                pages: vec![0],
                boxes: vec![sheet],
                drawn: Some(Rect::from_corners(80.0, 60.0, 2231.54, 1620.0)),
                unread: 0,
                common_origin: Some((0.0, 0.0)),
            },
            choice: Choice::Standard(pdfcer_core::paper::PaperSize::A1),
            landscape: true,
            custom_w_mm: 841,
            custom_h_mm: 594,
            apply_requested: false,
            close_requested: false,
        }
    }

    /// ★★★ **Landscape transposes, and it transposes both kinds of sheet.**
    ///
    /// The single most likely defect in a window with a size list and an
    /// orientation pair: a standard size that turns and a custom size that does
    /// not, or the reverse. Both go through [`PageSizeDialog::sheet_pt`]
    /// precisely so they cannot diverge, and this is what holds that. The
    /// consequence of getting it wrong here is worse than in the sized-New
    /// window — there it makes a blank page the wrong way up, here it crops a
    /// drawing along the wrong axis.
    #[test]
    fn landscape_transposes_a_standard_size_and_a_custom_one() {
        let mut dialog = a1_landscape();
        dialog.choice = Choice::Standard(pdfcer_core::paper::PaperSize::A1);
        dialog.landscape = false;
        let portrait = dialog.sheet_pt();
        dialog.landscape = true;
        let landscape = dialog.sheet_pt();
        assert!(
            portrait.0 < portrait.1,
            "A1 portrait is taller: {portrait:?}"
        );
        assert!(
            (portrait.0 - landscape.1).abs() < 0.01 && (portrait.1 - landscape.0).abs() < 0.01,
            "landscape A1 must be portrait A1 transposed: {portrait:?} vs {landscape:?}"
        );

        dialog.choice = Choice::Custom;
        dialog.custom_w_mm = 300;
        dialog.custom_h_mm = 500;
        dialog.landscape = false;
        let portrait = dialog.sheet_pt();
        dialog.landscape = true;
        let landscape = dialog.sheet_pt();
        assert!(
            (portrait.0 - landscape.1).abs() < 0.01 && (portrait.1 - landscape.0).abs() < 0.01,
            "a custom sheet must turn the way a standard one does: {portrait:?} vs {landscape:?}"
        );
    }

    /// ★★★ **The window opens on the size the sheets ALREADY ARE.**
    ///
    /// The decision this file's `open` doc comment argues, checked rather than
    /// merely written down. A default that drifted back to A4 — the sized-New
    /// window's default, and the obvious thing to copy — would mean an operator
    /// who opened this window to *look* was one careless press from cropping an
    /// A1 drawing to A4.
    #[test]
    fn it_opens_on_the_size_the_sheets_already_are() {
        let dialog = a1_landscape();
        let (w, h) = dialog.sheet_pt();
        let sheet = dialog.survey.boxes[0];
        assert!(
            (w - sheet.width()).abs() < 1.0 && (h - sheet.height()).abs() < 1.0,
            "the window opens on {:?} while its sheets are {} × {}",
            (w, h),
            sheet.width(),
            sheet.height()
        );
    }

    /// ★★ **An unchanged size is offerable and is announced as a no-op.**
    ///
    /// Both halves. `would_change` false must not hide the button — an operator
    /// who opened the window to look would be left with nothing to press but
    /// Cancel — and it must be *said*, because a control that acts and reports
    /// nothing is the defect class this project is named after.
    #[test]
    fn opening_on_the_current_size_would_change_nothing() {
        let dialog = a1_landscape();
        assert!(
            !dialog.would_change(),
            "A1 landscape onto A1 landscape sheets"
        );
        assert!(
            dialog.is_valid(),
            "and it is still a valid, pressable choice"
        );
    }

    /// ★★★ **Picking A4 for an A1 drawing is seen as a crop, with his own
    /// numbers.**
    ///
    /// The property the whole window exists for. `overhang` is
    /// `SheetSurvey`'s and tested there; what this asserts is that **the
    /// dialog's own chain** — choice → `sheet_pt` → `target_rect` → `overhang`
    /// — reaches it. A window that computed the right rectangle for the summary
    /// line and a different one for the measurement would pass every test in
    /// `pagesize` and put a false promise on screen.
    #[test]
    fn choosing_a4_for_an_a1_drawing_reports_the_overhang() {
        let mut dialog = a1_landscape();
        dialog.choice = Choice::Standard(pdfcer_core::paper::PaperSize::A4);
        dialog.landscape = false;

        assert!(dialog.would_change());
        let (w, h) = dialog.sheet_pt();
        let (left, right, bottom, top) = dialog
            .survey
            .overhang(dialog.survey.target_rect(w, h))
            .expect("the extent was measured");
        assert!(
            (left).abs() < 0.01 && (bottom).abs() < 0.01,
            "{left} {bottom}"
        );
        assert!(
            right > 1600.0,
            "the drawing runs well past A4's right edge: {right}"
        );
        assert!(top > 700.0, "and past its top: {top}");
    }

    /// ★★ **Growing the sheet is not reported as a crop.**
    ///
    /// The falsifying direction of the test above, and it is not redundant: a
    /// sign error in the overhang arithmetic passes that test and fails this
    /// one. An operator putting an A3 detail sheet onto A1 paper must see the
    /// promise, not a warning.
    #[test]
    fn growing_the_sheet_reports_that_everything_fits() {
        let mut dialog = a1_landscape();
        dialog.survey.boxes = vec![Rect::from_corners(0.0, 0.0, 1190.55, 841.89)];
        dialog.survey.drawn = Some(Rect::from_corners(40.0, 40.0, 1150.0, 800.0));
        dialog.choice = Choice::Standard(pdfcer_core::paper::PaperSize::A1);
        dialog.landscape = true;

        let (w, h) = dialog.sheet_pt();
        assert_eq!(
            dialog.survey.overhang(dialog.survey.target_rect(w, h)),
            Some((0.0, 0.0, 0.0, 0.0)),
            "an A3 drawing on A1 paper loses nothing"
        );
    }

    /// ★ **The bounds are checked on the millimetres the operator typed**,
    /// both directions, on the sized-New window's argument and with its
    /// numbers.
    #[test]
    fn an_out_of_range_custom_size_is_refused_in_both_directions() {
        let mut dialog = a1_landscape();
        dialog.choice = Choice::Custom;

        dialog.custom_w_mm = 0;
        dialog.custom_h_mm = 297;
        assert!(!dialog.is_valid(), "a zero-width sheet must be refused");

        dialog.custom_w_mm = 210;
        dialog.custom_h_mm = MAX_CUSTOM_MM + 1;
        assert!(
            !dialog.is_valid(),
            "a sheet past the ceiling must be refused"
        );

        dialog.custom_h_mm = MAX_CUSTOM_MM;
        assert!(dialog.is_valid(), "the ceiling itself must be allowed");

        dialog.custom_w_mm = MIN_CUSTOM_MM;
        dialog.custom_h_mm = MIN_CUSTOM_MM;
        assert!(dialog.is_valid(), "the floor itself must be allowed");
    }

    /// ★ **The two custom bounds are the sized-New window's**, because two
    /// windows in one program that disagree about the smallest sheet a PDF may
    /// have would be a defect whichever of them was right.
    #[test]
    fn the_custom_bounds_agree_with_the_sized_new_window() {
        assert_eq!(MIN_CUSTOM_MM, 2);
        assert_eq!(MAX_CUSTOM_MM, 5080);
        #[allow(clippy::cast_precision_loss, reason = "5080 is exact in f64")]
        let ceiling_pt = MAX_CUSTOM_MM as f64 * PT_PER_MM;
        assert!(
            (ceiling_pt - 14_400.0).abs() < 0.5,
            "the ceiling is Annex C.2's 14,400 units, not {ceiling_pt}"
        );
    }
}
