//! # `dialogs::import_text` — a text file becomes pages
//!
//! `file.import_text`, the import half of the operator's ask:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/import_text.md`.

use egui::Ui;
use pdfcer_gui_base::entry;

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
/// The margin box, in points.
const REGION_MARGIN: &str = "import-text.margin"; // ui-text-exempt: diagnostic region name

/// The faces this window offers.
const FACES: &[pdfcer_core::fontdata::Std14] = &[
    pdfcer_core::fontdata::Std14::Helvetica,
    pdfcer_core::fontdata::Std14::HelveticaBold,
    pdfcer_core::fontdata::Std14::TimesRoman,
    pdfcer_core::fontdata::Std14::TimesBold,
    pdfcer_core::fontdata::Std14::Courier,
];

/// The narrowest and widest font size the spinner will reach, in points.
const SIZE_RANGE: std::ops::RangeInclusive<f64> = 6.0..=72.0;

/// The margin spinner's range, in points.
const MARGIN_RANGE: std::ops::RangeInclusive<f64> = 0.0..=150.0;

/// Where the pages land, as the four radios offer it.
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
    /// An index rather than the `PaperSize` itself, because `PaperSize` is
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
    /// Deferred by one statement, which is `insert_pages`' and the print
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

    /// The action a `.txt` dropped on the window raises: the pages this
    /// window makes with its controls untouched, after `current_page`.
    #[must_use]
    pub fn dropped(path: std::path::PathBuf, current_page: usize) -> Action {
        let window = Self::open(path, current_page);
        Action::File(crate::app::actions::importtext::FileAction::ImportText {
            template: Box::new(window.template()),
            position: window.insert_position(),
            path: window.path,
        })
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
                // It carries the CHOICES and not just the press. A build
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
            let (widget, refusal) = entry::drag_value(
                ui,
                &mut self.margin,
                entry::Kind::Length(entry::LengthUnit::Point),
            );
            let response = refusal.show(ui.add(widget.speed(1.0).range(MARGIN_RANGE)));
            crate::diag::ui_rect_visible(REGION_MARGIN, response.rect, ui.clip_rect());
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
            let (widget, refusal) = entry::drag_value(
                ui,
                &mut self.size,
                entry::Kind::Length(entry::LengthUnit::Point),
            );
            refusal.show(ui.add(widget.speed(0.25).range(SIZE_RANGE)));
        });
        // Beside the chooser it qualifies, not at the foot of the window —
        // `panels::properties`' standing rule, for its reason: a caveat below
        // everything arrives after the operator has already decided.
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
            // Cancel closes the host, which is the same act as the window's
            // own close button — so there is one way to abandon this and not
            // two that could come to differ.
            if ui.button(t::cancel()).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

/// The sheet at `index`, clamped.
fn sheet_of(index: usize) -> pdfcer_core::paper::PaperSize {
    let all = pdfcer_core::paper::PaperSize::ALL;
    all[index.min(all.len() - 1)]
}

/// The index of the sheet this window opens on.
fn default_sheet() -> usize {
    pdfcer_core::paper::PaperSize::ALL
        .iter()
        .position(|s| s.id() == "a4")
        .unwrap_or(0)
}

/// The label for the face at `index` in [`FACES`], clamped.
fn face_name(index: usize) -> &'static str {
    t::face_name(FACES[index.min(FACES.len() - 1)])
}

#[cfg(test)]
mod tests;
