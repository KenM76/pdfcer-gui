//! # `app::status::ocrlayer` — what the OCR text layer is showing
//!
//! One line, drawn only while `view.ocr_overlay` is on.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/ocrlayer.md`.

use crate::app::state::OpenDoc;

/// Put the line on the bar, if there is one to put.
pub(super) fn show(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(raw) = doc.view.ocr_overlay else {
        return;
    };
    let percent = (crate::canvas::ocrlayer::painted_fraction(raw) * 100.0).round() as u32;
    let blocks = doc.page_text().map_or(0, |text| {
        text.runs
            .iter()
            .filter(|run| crate::canvas::ocrlayer::is_ocr_run(run))
            .count()
    });
    ui.label(crate::text::status::ocr_layer_line(percent, blocks));
    ui.separator();
}
