//! # `dialogs::insert_image` — putting a picture on the page
//!
//! ## The gap this closes
//!
//! `edit.insert_image` was registered, drawn on Edit ▸ Insert, carried a `★ P3`
//! mark in `shell::commands::reach`'s `SCAFFOLDED` list — and its recorded
//! reason was, verbatim, **"No recorded reason for the missing arm."** One of
//! only three entries in that list of which that was true.
//!
//! `EditSession::add_image` shipped long before this window: an image XObject,
//! an optional `/SMask`, a `q…cm…Do…Q` overlay stream and the page patches, as
//! **one undo entry**, additive, with the original bytes left verbatim.
//!
//! ## ★ Why placement is numeric here, and not a drag on the canvas
//!
//! Every other editor lets you drag a box, and the standing tie-breaker in this
//! project is *"make it work the way other programs do"*. This one asks for a
//! rectangle in millimetres instead, and that is a decision with three reasons
//! rather than a shortcut:
//!
//! 1. **It is the better gesture for this document.** The operator's sheets are
//!    CAD drawings. A picture going into one is a logo in a title block or a
//!    site photograph in a detail box — both of which have a *position on the
//!    sheet* that somebody decided, and neither of which is served by a
//!    freehand drag to "about there".
//! 2. **A placed image cannot be moved afterwards.** It is page **content**,
//!    not an annotation, so it carries no `/Rect` and the move-and-resize
//!    surface `FEATURES.md` plans reaches annotations only. A one-shot
//!    freehand placement with no correction but undo is a worse offer than a
//!    box you can type into.
//! 3. **It can be verified.** A drag needs a new `CanvasTool` variant, a
//!    `DragKind`, and an arm in `canvas::gesture::press_kind` — machinery this
//!    project's own RAG warns is the least safe thing here to change without
//!    driving the binary, and the harness needs the operator's machine. R1 says
//!    a phase is not done until it is driven; shipping a gesture that cannot be
//!    is shipping the thing R1 exists to stop.
//!
//! **A drag-to-place gesture is a second ROUTE to the same action**, not a
//! replacement for this window, and it is the natural next slice: `Action::
//! InsertImage` already carries everything it would produce.
//!
//! ## ★ What this window previews, and the one thing it does not
//!
//! It previews **where the picture lands**, from `NewImage::placed_rect()` —
//! the engine's own function, public for exactly this, whose doc says
//! *"re-deriving the arithmetic in the GUI is how a preview and a result drift
//! apart."* Nothing here computes a rectangle.
//!
//! It previews **the resolution** too, as of 2026-08-19 — filed that morning
//! and shipped the same day. `NewImage::effective_dpi()` and
//! `below_screen_resolution()` are pure, and the half that mattered is that
//! `add_image` now **calls** them rather than repeating the formula, so the
//! preview and the outcome cannot disagree.
//!
//! ★ The four-line version this window nearly computed would have been wrong.
//! Under `ImageFit::Contain` the placed rectangle is the *letterboxed*
//! sub-rectangle, not the box the operator typed, so measuring `rect` reports a
//! resolution low by exactly the letterbox ratio. The pure sibling was not
//! saving four lines; it was saving the letterbox.
//!
//! ## Why the import happens BEFORE the window opens
//!
//! So a file that cannot be placed is refused at the moment it is chosen,
//! naming the file's own problem — *"pdfcer does not place GIF images"*, *"this
//! image uses {feature}, which pdfcer cannot place"* — rather than opening a
//! window full of controls over a picture that was never going to go in.
//!
//! It also means the window can state the picture's real facts: its format, its
//! pixel dimensions **as displayed** (an EXIF-rotated photograph is transposed
//! by the importer), and whether the resolution it reports is one the file
//! declared or one pdfcer assumed.
//!
//! ## conventions: dialogs
//!
//! Corpus: `ui-conventions/dialogs.md`.
//!
//! - G1 is-an-os-window: **GAP, and it is the operator's report of 2026-08-20** —
//!   *"doesn't pop up in its own movable window. It is locked within the
//!   boundaries of the program's window."* Every dialog here is an
//!   `egui::Window`, which is an in-viewport panel. egui can already do the real
//!   thing through `show_viewport_immediate`; the panel was the path of least
//!   resistance and nothing pushed back.
//! - G2 use-the-os-dialog: the file and save pickers are the system's, and
//!   `pdfcer-print` opens the native printer-properties sheet owned by our
//!   window. The dialogs in this directory are pdfcer's own because they carry
//!   choices only pdfcer has — which is the right reason to draw one, and does
//!   not excuse G1.
//! - G3 owned-by-the-app: the native pickers are; an in-viewport panel cannot be
//!   anything else. This becomes a live question the moment G1 is fixed.
//! - G4 enter-accepts-escape-cancels: **PARTIAL** — Escape closes; Enter is not
//!   wired as the affirmative default and no button is drawn as the default, so
//!   an operator who types into the last field and presses Enter gets nothing.
//! - G5 keyboard-reachable: **GAP** — egui's tab order is positional and nothing
//!   here asserts that focus starts in a sensible field or that a modal traps
//!   it.
//! - G6 remembers-position: **GAP** — anchored `CENTER_CENTER` every time, so a
//!   dialog the operator moved comes back to the middle of the window.
//! - G7 destructive-verbs-named: the unsaved-changes dialog names the file and
//!   labels its buttons with verbs rather than Yes/No.
//! - G8 cancel-is-silent: a cancelled picker is a complete, correct,
//!   uninteresting outcome and is never reported as an error.
//! - G9 nothing-blocks-silently: a native picker blocks the UI thread by design,
//!   which is what a modal file dialog is. Long work behind a pdfcer dialog is
//!   not surfaced. **GAP.**

use std::sync::Arc;

use egui::Ui;
use pdfcer_core::edit::{ImageFit, NewImage};
use pdfcer_core::image_import::ImportedImage;
use pdfcer_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::images as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:insert-image"; // ui-text-exempt: trace region name, never displayed
/// The region the width field publishes, so a driven check has a numeric
/// control it can move.
pub const REGION_WIDTH: &str = "insert-image.width"; // ui-text-exempt: trace region name, never displayed
/// The region the Insert button publishes.
/// The **Place it on the page…** button's region — `OPERATOR_REQUESTS.md` O66.
///
/// ★ Published on every frame the dialog draws, which is every frame it is NOT
/// hiding for a placement — so its absence from a trace means the window has
/// stepped aside, which is exactly the state a driven check needs to observe.
pub const REGION_PLACE: &str = "insert-image.place"; // ui-text-exempt: trace region name, never displayed
pub const REGION_INSERT: &str = "insert-image.insert"; // ui-text-exempt: trace region name, never displayed

/// Points per millimetre.
///
/// A PDF user-space unit is 1/72 inch by definition (§8.3.2.3) and an inch is
/// 25.4 mm. Spelled here rather than imported for the reason
/// `panels::docprops` gives for its own copy: a two-term definition
/// restated is cheaper to read than an import that sends the reader to another
/// module for a number they already know.
const PTS_PER_MM: f64 = 72.0 / 25.4;

/// The smallest box that can be placed, in millimetres.
///
/// One millimetre. Below that the picture is not a picture on any sheet this
/// application is for, and a zero-area box is refused separately with its own
/// sentence — `no_area` — because *"give it a size"* and *"that is too small"*
/// are different instructions.
const MIN_MM: f64 = 1.0;

/// The Insert-image window's live state.
///
/// Existence is the "open" state, as everywhere in [`super`].
pub struct InsertImageDialog {
    /// The imported picture.
    ///
    /// ★ `Arc`, and it is the only field here that is not a number. An
    /// `ImportedImage` owns the decoded or re-encoded stream bytes — megabytes
    /// for a scan — and the `Action` this window raises has to carry it out of
    /// the frame to the apply phase. Cloning it there would double the peak,
    /// and borrowing it would tie an `Action` to a widget's lifetime, which is
    /// the coupling the funnel exists to remove.
    image: Arc<ImportedImage>,
    /// The chosen file's name, for the window's first row.
    ///
    /// The **name**, not the path: the window is about the picture, and a full
    /// path in a narrow row wraps to three lines and buries the size beneath
    /// it.
    name: String,
    /// The page it will go on, frozen at open.
    ///
    /// ★ Frozen for [`super::insert_pages`]' reason, one control smaller: an
    /// operator who opens this window on page 7 and then pages away must not
    /// find the picture landing on page 9. The window says which page, so the
    /// choice is checkable.
    page_index: usize,
    /// The page's own extent in points, for the on-the-sheet check.
    page_size_pt: (f64, f64),
    /// The box, in millimetres from the page's bottom-left.
    /// ★★★ The operator's offer to point at the page instead of typing —
    /// `OPERATOR_REQUESTS.md` O66. See [`crate::dialogs::placing`]; it holds
    /// one boolean and derives everything else.
    place: crate::dialogs::placing::PlaceHandoff,
    x_mm: f64,
    /// See [`Self::x_mm`].
    y_mm: f64,
    /// See [`Self::x_mm`].
    width_mm: f64,
    /// See [`Self::x_mm`].
    height_mm: f64,
    /// What happens when the box's shape differs from the picture's.
    fit: ImageFit,
    /// Set by Insert, consumed after the window's closure returns.
    ///
    /// The same one-statement deferral every dialog here uses: `add_image`
    /// writes an XObject and patches the page, which is a document edit, and
    /// the action funnel's invariant is that no code path runs from a widget to
    /// a document.
    insert_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl InsertImageDialog {
    /// Open the window for an already-imported picture.
    ///
    /// # ★ The box is seeded at the picture's NATURAL size, centred
    ///
    /// Natural size is what the file asks for — its pixels at the resolution it
    /// declares, or one pixel per point when it declares none — so an operator
    /// who presses Insert immediately gets the placement the picture was made
    /// for. Seeding at some fraction of the page would be pdfcer choosing a
    /// scale nobody asked for, and the operator would have no way to tell that
    /// from the picture's own size.
    ///
    /// It is **clamped to the sheet** on both axes, because a 300-pixel-wide
    /// logo at 72 dpi is bigger than an A4 page and a window that opened
    /// refusing its own default would be a window that looks broken. The clamp
    /// preserves the aspect ratio, so a clamped default is still the picture's
    /// shape.
    #[must_use]
    pub fn open(image: Arc<ImportedImage>, name: String, doc: &OpenDoc) -> Self {
        let page_index = doc.view.page_index;
        let (pw, ph) = doc.pages.get(page_index).map_or((0.0, 0.0), |page| {
            let (w, h) = crate::viewer::page_extent_pts(page);
            (f64::from(w), f64::from(h))
        });
        let (nw, nh) = image.natural_size_pt();
        // Fit inside the sheet with a small margin, preserving shape. `min` of
        // the two axis ratios and never above 1.0: a picture smaller than the
        // page keeps its natural size exactly.
        let usable = 0.9;
        let scale = if nw > 0.0 && nh > 0.0 && pw > 0.0 && ph > 0.0 {
            ((pw * usable / nw).min(ph * usable / nh)).min(1.0)
        } else {
            1.0
        };
        let (w, h) = (nw * scale, nh * scale);
        Self {
            image,
            name,
            page_index,
            page_size_pt: (pw, ph),
            place: crate::dialogs::placing::PlaceHandoff::default(),
            x_mm: ((pw - w) / 2.0).max(0.0) / PTS_PER_MM,
            y_mm: ((ph - h) / 2.0).max(0.0) / PTS_PER_MM,
            width_mm: (w / PTS_PER_MM).max(MIN_MM),
            height_mm: (h / PTS_PER_MM).max(MIN_MM),
            fit: ImageFit::Contain,
            insert_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        // ★★★ **Step aside while the operator points** — `OPERATOR_REQUESTS.md`
        // O66. The FIRST line, before the window is built, so nothing of this
        // dialog exists on a frame where a placement is pending for it.
        //
        // ★ It returns `true` — *still open* — while drawing nothing. The
        // dialog is not closed: its typed numbers, its chosen fit and its
        // position are all exactly where they were, and they come back with it.
        // That is the difference between stepping aside and being dismissed,
        // and it is why his numbers survive the trip.
        //
        // `hidden` is DERIVED from `canvas::placing`'s pending record, so
        // whatever clears that record brings this window back — Escape, a mode
        // change, another tool, the document closing. There is no flag here to
        // forget to clear, which is the bug the Set-scale round trip has.
        if self
            .place
            .hidden(ctx, crate::canvas::placing::PlaceKind::Image)
        {
            return true;
        }
        // ★ ITS OWN OS WINDOW as of 2026-08-21, and the screen anchor is
        // retired rather than moved: an OS window is anchored to the DESKTOP,
        // which is what the standing rule was reaching for. Size is an opening
        // bid — see [`crate::dialogs::host::Host::fit`].
        let (frame, ()) = crate::dialogs::host::Host::new(
            "insert-image", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(480.0, 620.0),
            egui::vec2(380.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.insert_requested) {
            let rect = self.rect_pt();
            // ★ The PREVIEWED resolution is traced, and it is here for a driven
            // check rather than for a human reading a log.
            //
            // `add_image`'s own disclosure carries the same number, from the
            // same producer, and a harness comparing the two proves this window
            // is still ASKING the engine rather than computing its own. The
            // engine held up its half by deleting its copy of the formula so
            // there is one derivation left; the shell can only hold up its half
            // by making the equality observable.
            //
            // The failure it catches is specific and quiet: a re-derivation
            // here would be low by exactly the letterbox ratio under `Contain`,
            // which is the default, and both numbers would look perfectly
            // reasonable.
            let (dpi_x, _) = self.spec(rect).effective_dpi();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "insert-image-requested page={} llx={:.2} lly={:.2} urx={:.2} ury={:.2} \
                     fit={:?} dpi={dpi_x:.0}",
                    self.page_index, rect.llx, rect.lly, rect.urx, rect.ury, self.fit
                )
            });
            actions.push(Action::InsertImage {
                page: self.page_index,
                rect,
                fit: self.fit,
                image: Arc::clone(&self.image),
            });
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The box, in PDF points, as the engine wants it.
    ///
    /// One function, read by the validity check, the landing preview and the
    /// action. Three separate conversions is how a window comes to promise one
    /// rectangle and produce another — the same argument
    /// `dialogs::new_document::sheet_pt` makes for its own single derivation.
    /// Drain the operator's request to point at the page — O66.
    pub fn take_place_request(&mut self) -> bool {
        self.place.take_request()
    }

    /// **Write a placed rectangle back into the four millimetre fields** — O66.
    ///
    /// ★★ Through the INVERSE of [`rect_pt`], not through a second conversion.
    /// That function's own doc comment exists to keep one arithmetic for
    /// millimetres and points; a placement that converted separately would be
    /// the second, and the two would drift by a rounding rule nobody chose.
    ///
    /// ★ A **degenerate** rect — what a click produces — writes the corner and
    /// leaves the size alone. The dialog already has a width and a height, typed
    /// or defaulted from the picture's own aspect, and a click is a statement
    /// about *where*, not about *how big*. Overwriting the size with zero would
    /// throw away the one thing the operator did not ask to change.
    pub fn place(&mut self, rect: Rect) {
        self.x_mm = rect.llx / PTS_PER_MM;
        self.y_mm = rect.lly / PTS_PER_MM;
        let (w, h) = (rect.urx - rect.llx, rect.ury - rect.lly);
        if w > 0.0 && h > 0.0 {
            self.width_mm = (w / PTS_PER_MM).max(MIN_MM);
            self.height_mm = (h / PTS_PER_MM).max(MIN_MM);
        }
    }

    fn rect_pt(&self) -> Rect {
        rect_pt(self.x_mm, self.y_mm, self.width_mm, self.height_mm)
    }

    /// The placement spec, built the one way.
    ///
    /// ★ The builder rather than a struct literal — `NewImage` is
    /// `#[non_exhaustive]`, so a downstream crate cannot construct it
    /// field-by-field, and the constructor is what keeps a field added upstream
    /// from silently defaulting here.
    ///
    /// **One function, three readers**: the landing preview, the resolution
    /// preview, and the trace the harness cross-checks against the outcome. The
    /// apply arm builds it the same way, which is what makes `placed_rect()`
    /// here and the rectangle written there the same answer rather than two —
    /// and it is the same argument [`rect_pt`] makes about the millimetre
    /// conversion, one layer up.
    fn spec<'a>(&'a self, rect: Rect) -> NewImage<'a> {
        let spec = NewImage::new(self.page_index, rect, &self.image);
        match self.fit {
            ImageFit::Stretch => spec.stretching(),
            // `Contain` is the constructor's default, and the wildcard is
            // forced by `#[non_exhaustive]` rather than chosen. A third fit
            // mode would land here as Contain, which is the safe direction: it
            // never distorts a picture nobody asked to distort.
            _ => spec,
        }
    }

    /// Whether the current box can be placed, and what is wrong if not.
    ///
    /// ★ **Refused rather than clamped**, and the refusal names the problem.
    /// A box silently moved back onto the sheet is a placement the operator did
    /// not make, and they would discover it by looking at the drawing rather
    /// than at this window — which is `Tolerance::validate`'s rule applied one
    /// feature along: *"a corrected value the operator never saw is exactly the
    /// sneaky case."*
    fn refusal(&self) -> Option<&'static str> {
        refusal(
            self.x_mm,
            self.y_mm,
            self.width_mm,
            self.height_mm,
            self.page_size_pt,
        )
    }

    /// The whole window body.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        // --- what the picture is -----------------------------------------
        ui.horizontal(|ui| {
            ui.label(t::source_label());
            ui.label(&self.name);
        });
        let (px_w, px_h) = self.image.display_size_px();
        ui.horizontal(|ui| {
            ui.label(t::source_size_label());
            ui.label(t::source_size(self.image.format, px_w, px_h));
        });
        let (nw, nh) = self.image.natural_size_pt();
        ui.weak(t::natural_size(
            nw / PTS_PER_MM,
            nh / PTS_PER_MM,
            self.image.dpi,
        ));
        ui.add_space(8.0);

        // --- where it goes ------------------------------------------------
        // No `.strong()` — R84 / DEFECTS.md D11.
        ui.label(t::placement_heading());
        ui.label(t::placement_page(self.page_index.saturating_add(1)));
        ui.weak(t::placement_page_hint());
        ui.add_space(4.0);

        let max_mm = {
            let (pw, ph) = self.page_size_pt;
            (pw.max(ph) / PTS_PER_MM).max(MIN_MM)
        };
        ui.horizontal(|ui| {
            ui.label(t::placement_x());
            ui.add(spinner(&mut self.x_mm, -max_mm..=max_mm));
            ui.label(t::placement_y());
            ui.add(spinner(&mut self.y_mm, -max_mm..=max_mm));
        });
        ui.horizontal(|ui| {
            ui.label(t::placement_width());
            let response = ui.add(spinner(&mut self.width_mm, MIN_MM..=max_mm));
            crate::diag::ui_rect(REGION_WIDTH, response.rect);
            ui.label(t::placement_height());
            ui.add(spinner(&mut self.height_mm, MIN_MM..=max_mm));
        });
        ui.add_space(6.0);
        // ★★★ **The second route to the same four numbers** —
        // `OPERATOR_REQUESTS.md` O66: *"anything we are inserting like this
        // should have an option in its dialogue box to place it with the mouse
        // instead of by positional co-ordinates."*
        //
        // Beside the spinners rather than beneath the whole form, because it is
        // an alternative to THEM specifically — not to the fit, not to the
        // picture, not to Insert. An operator reading the four fields and
        // wondering how to know what to type finds the answer on the next line.
        //
        // ★ Neither route is the real one. This fills the fields in and the
        // operator can correct them afterwards, which the note under the button
        // says out loud.
        self.place.button(ui, REGION_PLACE);
        ui.add_space(8.0);

        // --- how it fits ---------------------------------------------------
        ui.label(t::fit_heading());
        for option in [ImageFit::Contain, ImageFit::Stretch] {
            ui.radio_value(&mut self.fit, option, t::fit_name(option));
        }
        ui.weak(t::fit_hint(self.fit));

        // ★ The landing, from the ENGINE's own arithmetic. Shown only when it
        // differs from the box, which under `Stretch` is never — a line
        // restating the two numbers above it would be noise.
        let spec = self.spec(self.rect_pt());
        let placed = spec.placed_rect();
        let asked = self.rect_pt();
        let differs = (placed.urx - placed.llx - (asked.urx - asked.llx)).abs() > 0.5
            || (placed.ury - placed.lly - (asked.ury - asked.lly)).abs() > 0.5;
        if differs {
            ui.weak(t::placed_note(
                (placed.urx - placed.llx) / PTS_PER_MM,
                (placed.ury - placed.lly) / PTS_PER_MM,
            ));
        }
        // ★ The resolution, previewed — the number that decides whether the
        // sheet plots, shown beside the spinners that set it rather than after
        // the commit that fixes it.
        //
        // From `effective_dpi()` and `below_screen_resolution()`, which are the
        // SAME calls `add_image` makes to build its own disclosure. Nothing
        // here computes a resolution: the naive four-liner would have measured
        // `rect` rather than the placed rectangle and reported a figure low by
        // exactly the letterbox ratio under `Contain`.
        //
        // Not `weak` when it is soft. This is the one line in the window that
        // changes what an operator should do, and a quiet grey sentence is the
        // one they skip.
        let dpi = t::dpi_preview(spec.effective_dpi(), spec.below_screen_resolution());
        if spec.below_screen_resolution() {
            ui.label(dpi);
        } else {
            ui.weak(dpi);
        }
        ui.add_space(8.0);

        // --- commit --------------------------------------------------------
        ui.separator();
        ui.horizontal(|ui| {
            match self.refusal() {
                // The commit control is ABSENT while the box cannot be placed,
                // not greyed — the standing rule, and the same choice the
                // Insert-from-file dialog makes for an unparseable range. The
                // sentence beside it says what to change.
                Some(reason) => {
                    ui.colored_label(ui.visuals().error_fg_color, reason);
                }
                None => {
                    let response = ui.button(t::insert_button());
                    crate::diag::ui_rect(REGION_INSERT, response.rect);
                    if response.clicked() {
                        self.insert_requested = true;
                    }
                }
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// The box, in PDF points, from millimetres.
///
/// # ★ Free rather than a method, and `#[non_exhaustive]` is what forced it
///
/// `ImportedImage` is `#[non_exhaustive]`, so **this crate cannot construct
/// one** — which means it cannot construct an [`InsertImageDialog`] either, and
/// a method on it could not be tested without a real decoded picture on disk.
///
/// The constraint pushed toward the better shape, which is the part worth
/// recording. These two functions are the whole of this window's arithmetic,
/// they are pure, and they are the same shape
/// `crate::text::measure::two_line_reading` was pushed into for the same reason
/// one feature along. A rule stated as a function is a rule that can be
/// asserted; a rule stated as a method on an unconstructible type is a rule
/// nobody checks.
///
/// It is also the ONE conversion in this window. The validity check, the
/// landing preview and the action all read it, because three separate
/// conversions is how a window comes to promise one rectangle and produce
/// another.
#[must_use]
fn rect_pt(x_mm: f64, y_mm: f64, width_mm: f64, height_mm: f64) -> Rect {
    Rect {
        llx: x_mm * PTS_PER_MM,
        lly: y_mm * PTS_PER_MM,
        urx: (x_mm + width_mm) * PTS_PER_MM,
        ury: (y_mm + height_mm) * PTS_PER_MM,
    }
}

/// Whether a box can be placed on a sheet of `page_size_pt`, and what is wrong
/// if not.
///
/// ★ **Refused rather than clamped**, and the refusal names the problem. A box
/// silently moved back onto the sheet is a placement the operator did not make,
/// and they would discover it by looking at the drawing rather than at this
/// window — `Tolerance::validate`'s rule applied one feature along: *"a
/// corrected value the operator never saw is exactly the sneaky case."*
///
/// ★ **An overhang is NOT refused.** Bleeding a picture past the crop box is a
/// real thing to do deliberately, and refusing it would make this window
/// stricter than the format — the class of helpfulness that makes an operator
/// fight their tool. Only a box **wholly** off the sheet is declined, because
/// that one cannot be anything but a mistake.
#[must_use]
fn refusal(
    x_mm: f64,
    y_mm: f64,
    width_mm: f64,
    height_mm: f64,
    page_size_pt: (f64, f64),
) -> Option<&'static str> {
    if width_mm < MIN_MM || height_mm < MIN_MM {
        return Some(t::no_area());
    }
    let rect = rect_pt(x_mm, y_mm, width_mm, height_mm);
    let (pw, ph) = page_size_pt;
    if rect.urx <= 0.0 || rect.ury <= 0.0 || rect.llx >= pw || rect.lly >= ph {
        return Some(t::off_the_page());
    }
    None
}

/// A millimetre spinner.
///
/// One tenth of a millimetre per drag step: a logo in a title block is
/// positioned to the millimetre and a photograph is not positioned at all, so
/// finer would be motion nobody uses and coarser would make the common case
/// need typing.
fn spinner(value: &mut f64, range: std::ops::RangeInclusive<f64>) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .speed(0.1)
        .range(range)
        .suffix(t::millimetres())
}

/// Open the window for `status`, or decline.
///
/// Applies the two guards every dialog in [`super`] applies at the one place it
/// is built. The no-document guard is real here rather than ceremonial: the
/// window's box is seeded from a page's extent, and a window over an empty
/// canvas would open on a zero-sized sheet and refuse its own default.
#[must_use]
pub fn open_for(
    status: &Status,
    image: Arc<ImportedImage>,
    name: String,
) -> Option<InsertImageDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => {
            Some(InsertImageDialog::open(image, name, doc))
        }
        _ => None,
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "exact conversion is the property under test"
)] // ui-text-exempt: lint justification, never displayed
mod tests {
    use super::*;

    /// The millimetre conversion is the definition, not a rounded copy.
    ///
    /// ★ A hand-rounded `2.8346` would be wrong in the sixth decimal, and a
    /// picture placed at 210 mm would land 0.0004 mm off A4's edge — invisible,
    /// permanent, and different from every other number in this application.
    /// `dialogs::new_document` makes the same point about `594.0 * 72/25.4`.
    #[test]
    fn a_millimetre_is_the_definition() {
        assert_eq!(PTS_PER_MM, 72.0 / 25.4);
        // A4's width, to the precision a placement needs.
        assert!((210.0 * PTS_PER_MM - 595.2755905511812).abs() < 1e-9);
    }

    /// A4 in points, the sheet every case below is measured against.
    const A4: (f64, f64) = (595.276, 841.89);

    /// ★ A box wholly off the sheet is refused; one that overhangs is not.
    ///
    /// The second half is the decision worth pinning. Bleeding a picture past
    /// the crop box is a real thing to do deliberately, and refusing it would
    /// make this window stricter than the format — the class of helpfulness
    /// that makes an operator fight their tool.
    #[test]
    fn off_the_sheet_is_refused_and_an_overhang_is_not() {
        assert!(refusal(50.0, 50.0, 40.0, 20.0, A4).is_none(), "ordinary");
        assert!(
            refusal(-20.0, 50.0, 40.0, 20.0, A4).is_none(),
            "an overhang past the left edge is a deliberate bleed, not an error"
        );
        assert!(
            refusal(500.0, 50.0, 40.0, 20.0, A4).is_some(),
            "wholly right of an A4 sheet"
        );
        assert!(
            refusal(50.0, -400.0, 40.0, 20.0, A4).is_some(),
            "wholly below the sheet"
        );
    }

    /// A box with no area is refused with its OWN sentence.
    ///
    /// Different from off-the-page because the instruction is different — *give
    /// it a size* rather than *move it back* — and one message covering both
    /// would tell half the operators the wrong thing to do.
    #[test]
    fn a_sizeless_box_gets_its_own_refusal() {
        let none = refusal(50.0, 50.0, 0.0, 20.0, A4);
        assert_eq!(none, Some(t::no_area()));
        assert_ne!(none, Some(t::off_the_page()));
    }
}
