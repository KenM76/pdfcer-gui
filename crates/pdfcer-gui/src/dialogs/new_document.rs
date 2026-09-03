//! # `dialogs::new_document` — the other half of New
//!
//! `RIBBON_IA.md` §5.1's File band specifies one row as
//! **`New (blank / from template)`**, and only the blank half shipped:
//! `file.new` makes an A4 page with no question asked, which is what Acrobat
//! and Inkscape both do. This is the other half — `file.new_from_template`,
//! the command that asks what kind — and Inkscape's split is the shape being
//! followed: `Ctrl+N` makes a document, `Ctrl+Alt+N` chooses what kind.
//!
//! ## ★ Why this could not be built until 2026-08-18
//!
//! `crate::app::blank`'s §3a is the record and is worth reading before
//! touching this file. In short: nothing in `pdfcer-core` wrote a `/MediaBox`,
//! so the only implementation available to a shell was **one checked-in
//! template asset per size** — ten with landscape, more with ANSI, *and a
//! custom size impossible at any count*. That half-capability was refused
//! rather than built, because a surface that answers twelve of thirteen cases
//! forecloses the fix for the thirteenth.
//!
//! `EditSession::set_media_box` and `pdfcer_core::paper` shipped, so the whole
//! thing is one asset, one dialog, every size, both orientations, custom
//! included.
//!
//! ## What it does NOT do, and why each is a decision
//!
//! **It does not resize an existing page.** `set_media_boxes` supports that
//! and it is a genuinely different capability with genuinely different
//! questions — does content move, does `/CropBox` follow, is shrinking below
//! the content a refusal. None of those arise here because `file.new`'s page
//! is empty by construction, which is exactly why the request that unblocked
//! this deliberately asked only for the narrow answer. A page-resize surface
//! belongs in Document ▸ Properties and is not this module's.
//!
//! **It does not remember the last size chosen.** The obvious convenience, and
//! it is left out on the evidence rather than on principle: nobody drafts a
//! sheet in pdfcer, so the *second* use of this dialog is rare enough that a
//! remembered value would more often be stale than helpful — and a New command
//! that silently produced A1 because of something the operator did last
//! Tuesday is worse than one that always starts where its sibling does. If the
//! operator reports otherwise, `crate::app::prefs` is where it would go, beside
//! the opening-view preferences, and this paragraph is the argument to
//! overturn.
//!
//! **It offers no templates**, despite the command's name. `RIBBON_IA.md`'s
//! parenthetical for this row is `(page size)`, which is the IA's own
//! annotation of scope — the same shape as `Export image… (PNG/JPEG/TIFF, DPI
//! picker)`. So the label follows the IA, and
//! [`crate::text::new_document::intro`] states in the window's first line what
//! it actually offers, because that is the cheapest correction available to a
//! session that may propose IA amendments and may not make them.
//!
//! ## Application-scoped, like About and Settings
//!
//! It draws with nothing open and must: an operator with an empty shell is the
//! one most likely to want it, which is the same argument `file.new` and
//! `file.open` are registered with no `enabled_when` for. So it is drawn
//! **before** `crate::dialogs::DialogsState::show`'s document guard, beside
//! About, and closing a document does not close it.
//!
//! ## Rule 4: nothing here is drawn on a page
//!
//! There is no page yet. This window states what it will make and makes it;
//! the sheet it reports is the sheet that lands, and there is no inference to
//! disclose because there is no inference — the operator typed or picked every
//! number in it. The one thing pdfcer decides on their behalf is the **refusal**
//! of an out-of-range custom size, and that is stated in words with its limits
//! named (`crate::text::new_document::custom_refused`).

use egui::Ui;

use crate::app::actions::Action;
use crate::text::new_document as t;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "new-document.body";

/// The size combo, closed.
const REGION_SIZE: &str = "new-document.size";

/// The Create button.
const REGION_CREATE: &str = "new-document.create";

/// One region per entry in the OPEN size list, indexed from zero in
/// `pdfcer_core::paper::PaperSize::ALL` order, with `Custom…` last.
///
/// # Why the entries are published
///
/// The same argument the print dialog's paper list makes: an egui combo popup
/// is an `Area` laid out at paint time, so nothing outside the process can
/// compute where an entry is — and a check that can open a list but not choose
/// from it can assert only that a control exists. "The control exists" is
/// exactly what was true of the print dialog's tray checkbox for four months
/// while it did nothing.
///
/// Here the property worth asserting is that **picking a size produces a page
/// of that size**, end to end through `set_media_box`, a full rewrite and a
/// re-parse. That needs a click on a specific entry.
const REGION_SIZE_ITEM_PREFIX: &str = "new-document.size.item.";

/// The Portrait radio.
///
/// Published because the transposition is the most likely defect in this
/// window and the one a unit test cannot see end to end: `sheet_pt` is pinned
/// in tests, and what is *not* pinned there is that the radio the operator
/// clicks is the one that reaches it.
const REGION_PORTRAIT: &str = "new-document.portrait";

/// The Landscape radio. See [`REGION_PORTRAIT`].
const REGION_LANDSCAPE: &str = "new-document.landscape";

/// Points per millimetre — 72 points per inch ÷ 25.4 mm per inch.
///
/// Stated once. The engine's `paper` module has the same constant for the same
/// reason, and the two are not shared deliberately: a four-character constant
/// hoisted into a public API to give two unrelated callers a common dependency
/// is not a saving, and the *value* cannot drift because it is a definition.
const PT_PER_MM: f64 = 72.0 / 25.4;

/// The smallest custom sheet this dialog will make, in millimetres.
///
/// ISO 32000-1 Annex C.2 advises a minimum of **3 units** (≈ 1.06 mm), so 1 mm
/// would be marginally under it and 2 mm is comfortably over. Rounded up
/// rather than to the letter of the advice because a sub-millimetre page is
/// not a thing anybody wants and a bound an operator can remember is worth
/// more than a bound derived to two decimal places.
const MIN_CUSTOM_MM: i64 = 2;

/// The largest custom sheet this dialog will make, in millimetres.
///
/// **5,080 mm = 200 inches = 14,400 default user space units**, which is
/// ISO 32000-1 Annex C.2's advised maximum. See
/// [`crate::text::new_document::custom_refused`] for the full sourcing,
/// including the fact that ISO 32000-2 drops the number entirely and this is
/// therefore 1.7-era portability advice pdfcer is choosing to honour.
const MAX_CUSTOM_MM: i64 = 5080;

/// Which sheet the operator has picked.
///
/// A separate type from `pdfcer_core::paper::PaperSize` rather than
/// `Option<PaperSize>`, because "custom" is a *state of the dialog* — it opens
/// two fields and changes what the summary line reads — and not a missing
/// size. `Option` would have made the two fields' relevance depend on a `None`
/// that also means "nothing chosen yet", which is a state this dialog never
/// has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    /// One of `pdfcer_core::paper::PaperSize::ALL`.
    Standard(pdfcer_core::paper::PaperSize),
    /// The two millimetre fields.
    Custom,
}

/// The New-document dialog's state.
pub struct NewDocumentDialog {
    /// Which sheet.
    choice: Choice,
    /// Which way up. Applied to a standard size by transposing it, and to a
    /// custom size by transposing the two fields — one rule, so the summary
    /// line cannot disagree with the document.
    landscape: bool,
    /// The custom width in millimetres, live even while a standard size is
    /// selected so that switching away and back does not lose it. Same
    /// reasoning as the print dialog's `range_text`.
    custom_w_mm: i64,
    /// The custom height in millimetres.
    custom_h_mm: i64,
    /// Set by Create, consumed after the window closure returns.
    ///
    /// Deferred by one statement for the same reason the print dialog's commit
    /// is: the action replaces the open document, which drops the render
    /// worker's texture and rebuilds every panel, and doing that inside
    /// `Window::show`'s closure runs it while egui is part-way through laying
    /// this window out.
    create_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl NewDocumentDialog {
    /// Open the dialog, on A4 portrait.
    ///
    /// ★ **A4 portrait and not something cleverer.** It is what `file.new`
    /// makes, and the two commands sit next to each other in the same ribbon
    /// group: an operator who opens this window to check what it offers should
    /// see the state the plain command would have produced, so the difference
    /// between the two controls is *"one asks"* and nothing else.
    ///
    /// `crate::app::blank`'s §3 is where A4 is argued — two of the three
    /// reference applications ship it, and the operator's own corpus is
    /// A-series.
    #[must_use]
    pub fn open() -> Self {
        Self {
            choice: Choice::Standard(pdfcer_core::paper::PaperSize::A4),
            landscape: false,
            // A4 in millimetres, so the custom fields open on a size rather
            // than on zeros. Zeros would render the refusal line the instant
            // an operator picked Custom, which reads as the dialog objecting
            // to a choice they have not finished making.
            custom_w_mm: 210,
            custom_h_mm: 297,
            create_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        // ★ ITS OWN OS WINDOW as of 2026-08-21. The screen-anchor note is
        // retired rather than moved: the operator's standing objection is to
        // surfaces whose position is derived from the page, and a desktop
        // window is as far from page-derived as a position gets. Size is an
        // opening bid; see [`crate::dialogs::host::Host::fit`].
        let (frame, ()) = crate::dialogs::host::Host::new(
            "new-document", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 520.0),
            egui::vec2(380.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.create_requested) {
            let (width_pt, height_pt) = self.sheet_pt();
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "new-document-create choice={:?} landscape={} w_pt={width_pt:.2} \
                     h_pt={height_pt:.2}",
                    self.choice, self.landscape,
                )
            });
            actions.push(Action::NewSized {
                width_pt,
                height_pt,
            });
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The sheet this dialog currently describes, in points, **after**
    /// orientation.
    ///
    /// # ★ One function, read by three callers, and that is the point
    ///
    /// The summary line, the validity check and the action all ask this. Three
    /// separate computations of "what did they pick" is how a window comes to
    /// promise 841 × 1189 and produce 1189 × 841 — and the transposition is
    /// exactly the kind of arithmetic that is easy to write twice and hard to
    /// notice once.
    ///
    /// A standard size comes from `PaperSize::rect_with`, which is the
    /// engine's own table applying its own orientation rule. Nothing here
    /// re-derives a sheet size: `594.0 * 72/25.4` is a number the engine
    /// computes and a hand-rounded `1683.78` is a number that is *not* A1 and
    /// will not compare equal to a CAD exporter's.
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

    /// Whether the current choice can be made.
    ///
    /// Only a custom size can fail: every entry in `PaperSize::ALL` is a real
    /// sheet in range by construction. The check is on the **millimetre
    /// fields** rather than on the computed points, so the message can name the
    /// numbers the operator typed.
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

    /// The controls, the summary and the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        ui.label(t::size_heading());
        let selected_text = match self.choice {
            Choice::Standard(size) => t::size_entry(&t::size_name(size), size.size_pt()),
            Choice::Custom => t::size_custom().to_owned(),
        };
        let combo = egui::ComboBox::from_id_salt("new-document-size")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                // The engine's own order: the A series largest-first, then the
                // US sizes, then ANSI. Largest-first is deliberate on their
                // side and right on ours — this operator's sheets are A1 and
                // A3, and burying them under A4 would make the common case the
                // hard one. Reusing `PaperSize::ALL` rather than listing the
                // sizes here also means a size the engine adds appears without
                // a shell change, which is what `size_name`'s wildcard is for.
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

        ui.add_space(8.0);
        ui.label(t::orientation_heading());
        // A pair of radios rather than a doubled 32-entry list, which is
        // `pdfcer_core::paper`'s own argument for keeping `Orientation` a
        // separate type: *"orientation is orthogonal to size, every size
        // supports both, and a front end almost always wants the two as
        // separate controls."*
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

        // The two millimetre fields, present ONLY on Custom.
        //
        // Absent rather than greyed: R9's own distinction. Greying is for
        // *temporarily* unavailable, and these fields are not unavailable when
        // A4 is selected — they are irrelevant, which is a different thing and
        // is expressed by their not being there.
        if self.choice == Choice::Custom {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(t::custom_width());
                ui.add(egui::DragValue::new(&mut self.custom_w_mm).range(0..=MAX_CUSTOM_MM));
            });
            ui.horizontal(|ui| {
                ui.label(t::custom_height());
                ui.add(egui::DragValue::new(&mut self.custom_h_mm).range(0..=MAX_CUSTOM_MM));
            });
        }

        ui.add_space(8.0);
        // ★ The summary reports the sheet that will land, or says why none
        // will. One line, two states, never both — an operator whose numbers
        // are out of range does not also need to be told what 0 × 0 mm would
        // be in points.
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

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
            // ★ ABSENT, not greyed, when the size is out of range — and this is
            // the one place in this dialog where that rule is arguable, so the
            // argument is written down.
            //
            // R9 reserves greying for *temporarily* unavailable, and this IS
            // temporary: two keystrokes make it act. The case for greying is
            // therefore real. What decides it the other way is that the
            // refusal line is already on screen, immediately above, naming both
            // limits — so a greyed button would be a second, quieter statement
            // of a fact already made loudly, which is the print dialog's
            // reasoning for its own absent commit button.
            //
            // The button reappearing as the number crosses the bound is also a
            // clearer signal than a button changing shade, because it is
            // visible from the field the operator's eye is already on.
            if self.is_valid() {
                let create = ui.button(t::create());
                crate::diag::ui_rect(REGION_CREATE, create.rect);
                if create.on_hover_text(t::create_tooltip()).clicked() {
                    self.create_requested = true;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Landscape transposes, and it transposes both kinds of sheet.
    ///
    /// The single most likely defect in this window: a standard size that
    /// turns and a custom size that does not, or the reverse. Both go through
    /// [`NewDocumentDialog::sheet_pt`] precisely so they cannot diverge, and
    /// this is what holds that.
    #[test]
    fn landscape_transposes_a_standard_size_and_a_custom_one() {
        let mut dialog = NewDocumentDialog::open();
        dialog.choice = Choice::Standard(pdfcer_core::paper::PaperSize::A1);
        let portrait = dialog.sheet_pt();
        dialog.landscape = true;
        let landscape = dialog.sheet_pt();
        assert!(
            portrait.0 < portrait.1,
            "A1 portrait must be taller than wide: {portrait:?}"
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
            "a custom sheet must turn the same way a standard one does: {portrait:?} vs {landscape:?}"
        );
    }

    /// ★ A standard size comes from the ENGINE's table, to the last decimal.
    ///
    /// Not a tautology test. The failure it exists for is a shell that
    /// hand-rounds A1 to `1683.78 × 2383.94` — numbers that look right, are
    /// wrong in the fourth significant figure, and will not compare equal to
    /// the `/MediaBox` a CAD exporter writes. The engine converts from the
    /// defining millimetres for exactly that reason and this pins that the
    /// dialog does not re-derive it.
    #[test]
    fn a_standard_sheet_is_the_engines_own_number() {
        let mut dialog = NewDocumentDialog::open();
        dialog.choice = Choice::Standard(pdfcer_core::paper::PaperSize::A1);
        let (w, h) = dialog.sheet_pt();
        let expected = pdfcer_core::paper::PaperSize::A1.size_pt();
        assert!(
            (w - expected.0).abs() < f64::EPSILON && (h - expected.1).abs() < f64::EPSILON,
            "A1 must be exactly the engine's {expected:?}, not {:?}",
            (w, h)
        );
    }

    /// ★ The bounds are checked on the millimetres the operator typed.
    ///
    /// Both directions, because a check written as `> 0` would let a 12-metre
    /// sheet through and one written as `< MAX` would let a zero through, and
    /// each is a single missing clause.
    #[test]
    fn an_out_of_range_custom_size_is_refused_in_both_directions() {
        let mut dialog = NewDocumentDialog::open();
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

    /// ★ The dialog opens on exactly what `file.new` makes.
    ///
    /// The two commands sit beside each other in one ribbon group, and the
    /// difference between them must be "one asks" and nothing else. A default
    /// that drifted to A3 here would make the sibling controls quietly
    /// disagree about what a new document is.
    #[test]
    fn it_opens_on_the_size_the_plain_new_command_makes() {
        let dialog = NewDocumentDialog::open();
        let (w, h) = dialog.sheet_pt();
        let (doc, pages) = crate::app::blank::document().expect("the template parses");
        drop(doc);
        let media = pages[0].media_box;
        assert!(
            (w - (media.urx - media.llx)).abs() < 0.1 && (h - (media.ury - media.lly)).abs() < 0.1,
            "this dialog opens on {:?} while `file.new` makes {} x {}",
            (w, h),
            media.urx - media.llx,
            media.ury - media.lly,
        );
    }
}
