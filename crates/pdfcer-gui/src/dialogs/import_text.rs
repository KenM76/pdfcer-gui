//! # `dialogs::import_text` — a text file becomes pages
//!
//! `file.import_text`, registered and wired 2026-09-07, closing the half of the
//! operator's 2026-09-04 ask that could not be built at the time:
//!
//! > *"also the engine can export PDFs as text. we should have export/import
//! > for that."*
//!
//! ## ★★★ Why this could not exist until 2026-09-06
//!
//! **`pdfcer-core` could not create a page.** It could copy one, insert one
//! from another document, delete one — and there was no verb anywhere in the
//! crate that made a sheet out of nothing. `blank_document` is that primitive
//! and `place_text` is the paginating placer built on it, both shipped as
//! `Pass 252.0` in answer to this shell's own request, and the engine's reply
//! names the gap in those words.
//!
//! ⇒ Until then `dialogs::export_text`'s header said, correctly, *"there is no
//! import and this window says nothing about one"*, and no control was drawn
//! that would decline when pressed. That was R9 working. This window is what
//! replaces it, and the export window's paragraph is corrected rather than
//! deleted.
//!
//! ## ★★ This window is a CHOOSER, and that inverts its sibling's job
//!
//! `export_text` names **losses** — layout, fonts, position — because exporting
//! throws things away. Importing **invents**: a sheet size, margins, a
//! typeface, a size, a leading, an alignment, none of which is in the text file.
//! So every control here is a decision somebody has to make, the defaults are
//! answers to *"what would he have picked?"*, and only one sentence warns about
//! anything. `text::import_text`'s header carries that argument in full.
//!
//! ## ★ The defaults, and each one is a choice rather than an inheritance
//!
//! | control | default | why |
//! |---|---|---|
//! | sheet | **A4** | `PageTemplate::new()` is US Letter; this operator's world is metric and every other sheet chooser in this program opens on a metric size |
//! | margin | 72 pt | one inch, the engine's `DEFAULT_MARGIN_PT`, and the number a reader expects on a page of prose |
//! | font | Helvetica | the engine's own default, and the face a plain text file most often wants |
//! | size | 11 pt | the engine's is 12; 11 fits a 66-character line on A4 at one-inch margins, which is the classic measure for readable prose |
//! | position | **after the current page** | `insert_pages`' default, and for its stated reason: it is what *"insert here"* means to somebody who navigated to a sheet first |
//!
//! ★★ **The sheet list is `pdfcer_core::paper::PaperSize::ALL`**, rendered by
//! `text::new_document::size_name` — the same list and the same labels the New
//! Document and Page Size windows use. Three surfaces, one answer to *"what is
//! A1?"*, and an operator who learned the list in one meets it unchanged in the
//! others. A fourth private table here would be a fourth chance to disagree
//! about a sheet's dimensions.
//!
//! ## ★★★ What this window does NOT do, and the reason is the engine's
//!
//! **It does not read the file.** Not to count its lines, not to preview its
//! pagination, not to check its characters against the chosen face. Every one of
//! those means running the import to draw a window that offers to run the
//! import — and `place_text` **plans before it writes**, refusing with nothing
//! created, so the answers arrive as real numbers a moment later instead of
//! provisional ones now.
//!
//! ⇒ The file is read once, in the apply arm, and the report is the receipt.

use egui::Ui;

use crate::app::actions::Action;
use crate::text::import_text as t;
use crate::text::new_document as sheets;

/// The dialog body's published region, for `ui-verify`.
const REGION_BODY: &str = "import-text.body"; // ui-text-exempt: diagnostic region name
/// The commit button's region.
const REGION_IMPORT: &str = "import-text.import"; // ui-text-exempt: diagnostic region name
/// The sheet chooser's region.
const REGION_SHEET: &str = "import-text.sheet"; // ui-text-exempt: diagnostic region name
/// The font chooser's region.
const REGION_FACE: &str = "import-text.face"; // ui-text-exempt: diagnostic region name

/// The faces this window offers.
///
/// # ★★ Five of the fourteen, and the narrowing is the design
///
/// `Std14` has fourteen members; twelve are text faces and two (`Symbol`,
/// `ZapfDingbats`) are pictorial. A chooser listing all fourteen would offer
/// **Dingbats** for a page of prose, which is not a choice anybody is making,
/// and would offer four Helvetica variants where the bold and oblique ones are
/// *emphasis* rather than a body face — and this import sets one face for the
/// whole document, so emphasis has nothing to contrast with.
///
/// ⇒ What is left is the actual decision: **serif, sans, or monospace**, plus
/// the two bolds an operator might want for a short notice. `Courier` earns its
/// place because a text file is very often a listing, a schedule or a register
/// where columns aligned with spaces only survive in a monospaced face — which
/// is the one case where the *font* changes whether the import is readable.
const FACES: &[pdfcer_core::fontdata::Std14] = &[
    pdfcer_core::fontdata::Std14::Helvetica,
    pdfcer_core::fontdata::Std14::HelveticaBold,
    pdfcer_core::fontdata::Std14::TimesRoman,
    pdfcer_core::fontdata::Std14::TimesBold,
    pdfcer_core::fontdata::Std14::Courier,
];

/// The narrowest and widest font size the spinner will reach, in points.
///
/// ★ Six is below the smallest an operator would set for body text and is where
/// a `PageTooShort` refusal stops being plausible; seventy-two is one inch, past
/// which a single line no longer fits an A4 measure and the import becomes one
/// word per page. Both ends are far outside anything reasonable **on purpose**:
/// this is a guard against a scrub running away, not a judgement about
/// typography.
const SIZE_RANGE: std::ops::RangeInclusive<f64> = 6.0..=72.0;

/// The margin spinner's range, in points.
///
/// ★ Zero is legal and is what somebody importing a listing to be re-cropped
/// wants; the ceiling is a quarter of A4's short edge, past which the column is
/// narrower than the margins around it and `PageTooShort` becomes likely. The
/// engine refuses that case by name and this range makes reaching it a
/// deliberate act rather than an accident of a scrub.
const MARGIN_RANGE: std::ops::RangeInclusive<f64> = 0.0..=150.0;

/// Where the pages land, as the four radios offer it.
///
/// ★★ A local enum **only** for the radio state, converted to
/// `pdfcer_core::pageops::InsertPosition` at the point of use — copied
/// deliberately from `dialogs::insert_pages`, whose own note gives the reason:
/// two of the four need the current page index, which the radio does not carry
/// and the dialog does.
///
/// ⇒ It is a *copy* rather than a shared type, and that is worth defending:
/// the four variants are a fact about **this program's vocabulary for
/// inserting**, and both dialogs converting the same four into the same engine
/// enum at the same place is the property that matters. A shared enum would add
/// a module for four unit variants and would not make the two windows any more
/// alike than reading them side by side already does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Where {
    /// Before the page the operator is looking at.
    BeforeCurrent,
    /// After it. **The default** — `insert_pages`' reasoning, unchanged.
    AfterCurrent,
    /// Before every existing page.
    Start,
    /// After every existing page.
    End,
}

/// The import window's state.
pub struct ImportTextDialog {
    /// The text file the picker chose.
    path: std::path::PathBuf,
    /// Its name, for the title bar. Held rather than derived per frame.
    name: String,
    /// The page the operator was looking at when they asked.
    ///
    /// Frozen at open, for `insert_pages`' stated reason: the dialog is modal
    /// in spirit and the page behind it does not move, so a position that
    /// re-read the view every frame would mean *"after page 7"* silently
    /// becoming *"after page 9"*.
    current_page: usize,
    /// Which sheet, as an index into `PaperSize::ALL`.
    ///
    /// ★ An index rather than the `PaperSize` itself, because `PaperSize` is
    /// `#[non_exhaustive]` and the engine has said the table will grow — an
    /// index survives that, and `text::new_document` already stores it this way
    /// for the same reason.
    sheet: usize,
    /// The margin, points. One number for all four — see `t::margin_label`.
    margin: f64,
    /// Which face, as an index into [`FACES`].
    face: usize,
    /// Font size, points.
    size: f64,
    /// Which of the four positions.
    position: Where,
    /// Set by the commit button, consumed after the window closure returns.
    ///
    /// ★ Deferred by one statement, which is `insert_pages`' and the print
    /// dialog's rule: the action replaces most of the document's derived state,
    /// and doing that inside `Window::show`'s closure runs it while egui is
    /// part-way through laying this window out.
    import_requested: bool,
}

impl ImportTextDialog {
    /// Open it for `path`.
    #[must_use]
    pub fn open(path: std::path::PathBuf, current_page: usize) -> Self {
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        Self {
            path,
            name,
            current_page,
            sheet: default_sheet(),
            margin: 72.0,
            face: 0,
            size: 11.0,
            position: Where::AfterCurrent,
            import_requested: false,
        }
    }

    /// The engine's position for the selected radio.
    const fn insert_position(&self) -> pdfcer_core::pageops::InsertPosition {
        use pdfcer_core::pageops::InsertPosition;
        match self.position {
            Where::BeforeCurrent => InsertPosition::Before(self.current_page),
            Where::AfterCurrent => InsertPosition::After(self.current_page),
            Where::Start => InsertPosition::Start,
            Where::End => InsertPosition::End,
        }
    }

    /// The template the chosen controls describe.
    ///
    /// ★★ Built from `PageTemplate::new()` and then overridden, rather than
    /// constructed field by field. `PageTemplate` is the engine's type and it
    /// gains fields — `leading`, `alignment`, `color` and `unmappable` are all
    /// left exactly as the engine set them, which is the whole point: this
    /// window chooses the four things it draws controls for and takes the
    /// engine's answer for everything else, so a field added tomorrow arrives
    /// with the engine's default rather than with a zero this shell invented.
    fn template(&self) -> pdfcer_core::text_edit::PageTemplate {
        let mut template = pdfcer_core::text_edit::PageTemplate::new();
        let (w, h) = sheet_of(self.sheet).size_pt();
        template.media_box = pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, w, h);
        template.margin_left = self.margin;
        template.margin_right = self.margin;
        template.margin_top = self.margin;
        template.margin_bottom = self.margin;
        template.face = FACES[self.face.min(FACES.len() - 1)];
        template.size = self.size;
        template
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "import-text", // ui-text-exempt: a viewport key, never displayed.
            t::window_title_for(&self.name).as_str(),
            egui::vec2(460.0, 540.0),
            egui::vec2(380.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });

        if std::mem::take(&mut self.import_requested) {
            let template = self.template();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★★ It carries the CHOICES and not just the press. A build
                // whose chooser wrote the wrong sheet, or whose margin never
                // left its default, raises an identical action from an
                // identical click — and the pages it makes are wrong in a way
                // that looks like a rendering problem three steps later.
                format!(
                    "import-text-requested sheet={} margin={:.0} face={:?} size={:.1} at={:?}",
                    sheet_of(self.sheet).id(),
                    self.margin,
                    template.face,
                    self.size,
                    self.position,
                )
            });
            actions.push(Action::File(
                crate::app::actions::importtext::FileAction::ImportText {
                    path: self.path.clone(),
                    template: Box::new(template),
                    position: self.insert_position(),
                },
            ));
            return false;
        }

        !frame.closed
    }

    /// The window's controls.
    fn body(&mut self, ui: &mut Ui) {
        // No `.strong()` — R84 / DEFECTS.md D11.
        ui.label(egui::RichText::new(t::standing_note()).small().weak());
        ui.separator();

        ui.label(t::sheet_heading());
        ui.horizontal(|ui| {
            ui.label(t::size_label());
            let response = egui::ComboBox::from_id_salt("import-text-sheet") // ui-text-exempt: internal widget id
                .selected_text(sheets::size_name(sheet_of(self.sheet)))
                .show_ui(ui, |ui| {
                    for (index, size) in pdfcer_core::paper::PaperSize::ALL.iter().enumerate() {
                        ui.selectable_value(&mut self.sheet, index, sheets::size_name(*size));
                    }
                })
                .response;
            crate::diag::ui_rect_visible(REGION_SHEET, response.rect, ui.clip_rect());
        });
        ui.horizontal(|ui| {
            ui.label(t::margin_label());
            ui.add(
                egui::DragValue::new(&mut self.margin)
                    .speed(1.0)
                    .range(MARGIN_RANGE)
                    .suffix(t::points_suffix()),
            );
        });

        ui.separator();
        ui.label(t::type_heading());
        ui.horizontal(|ui| {
            ui.label(t::face_label());
            let response = egui::ComboBox::from_id_salt("import-text-face") // ui-text-exempt: internal widget id
                .selected_text(face_name(self.face))
                .show_ui(ui, |ui| {
                    for index in 0..FACES.len() {
                        ui.selectable_value(&mut self.face, index, face_name(index));
                    }
                })
                .response;
            crate::diag::ui_rect_visible(REGION_FACE, response.rect, ui.clip_rect());
        });
        ui.horizontal(|ui| {
            ui.label(t::size_pt_label());
            ui.add(
                egui::DragValue::new(&mut self.size)
                    .speed(0.25)
                    .range(SIZE_RANGE)
                    .suffix(t::points_suffix()),
            );
        });
        // ★ Under the chooser it qualifies, not at the foot of the window —
        // `REVIEW_TRIAGE.md`'s rule and the properties panel's placement: a
        // caveat below everything arrives after the operator has decided.
        ui.label(egui::RichText::new(t::face_note()).small().weak());

        ui.separator();
        ui.label(t::where_heading());
        ui.radio_value(
            &mut self.position,
            Where::BeforeCurrent,
            t::before_current(),
        );
        ui.radio_value(&mut self.position, Where::AfterCurrent, t::after_current());
        ui.radio_value(&mut self.position, Where::Start, t::at_start());
        ui.radio_value(&mut self.position, Where::End, t::at_end());

        ui.separator();
        ui.horizontal(|ui| {
            let response = ui.button(t::import());
            crate::diag::ui_rect_visible(REGION_IMPORT, response.rect, ui.clip_rect());
            if response.clicked() {
                self.import_requested = true;
            }
            // ★ Cancel closes the host, which is the same act as the window's
            // own close button — so there is one way to abandon this and not
            // two that could come to differ.
            if ui.button(t::cancel()).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

/// The sheet at `index`, clamped.
///
/// ★ Clamped rather than indexed, because `PaperSize::ALL` can shrink between
/// builds as well as grow — the engine says the table moves — and a stored
/// index from a longer list must not panic a window open.
fn sheet_of(index: usize) -> pdfcer_core::paper::PaperSize {
    let all = pdfcer_core::paper::PaperSize::ALL;
    all[index.min(all.len() - 1)]
}

/// The index of the sheet this window opens on.
///
/// ★★ **A4, found by id rather than by position.** `PaperSize::ALL`'s order is
/// the engine's business and it has said the table will grow; a hard-coded
/// index would silently open on a different sheet the day one is inserted
/// before A4 — and a window that opens on the wrong paper is a defect an
/// operator only notices after importing.
///
/// Falls back to the first entry, which cannot be wrong in a way that matters:
/// the chooser is right there and the sheet is the first thing in the window.
fn default_sheet() -> usize {
    pdfcer_core::paper::PaperSize::ALL
        .iter()
        .position(|s| s.id() == "a4")
        .unwrap_or(0)
}

/// The label for the face at `index` in [`FACES`], clamped.
///
/// ★ A one-line adapter over `text::import_text::face_name`, and the clamp is
/// the whole reason it exists: this window stores an INDEX, and a stored index
/// from a longer list must not panic a window open. The words themselves live
/// in the catalogue, where `check-ui-strings` can see them — it reported all
/// four of them the first time this file tried to keep them locally.
fn face_name(index: usize) -> &'static str {
    t::face_name(FACES[index.min(FACES.len() - 1)])
}

#[cfg(test)]
mod tests;
