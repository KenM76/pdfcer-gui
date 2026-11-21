//! The Export image window's separation-plates choice, offered under EMF.

use egui::Ui;

/// The checkbox. ui-verify clicks it by this name.
pub const REGION_PLATES: &str = "export-image.plates"; // ui-text-exempt: trace region name, never displayed

/// Draw the choice into `ui`.
pub(super) fn show(ui: &mut Ui, plates: &mut bool) {
    let before = *plates;
    let response = ui.checkbox(plates, crate::text::export_plates::checkbox());
    crate::diag::ui_rect_visible(REGION_PLATES, response.rect, ui.clip_rect());
    ui.weak(crate::text::export_plates::hint());
    if *plates != before {
        let on = u8::from(*plates);
        crate::diag::trace(|| format!("export-image-plates-chosen plates={on}")); // ui-text-exempt: diagnostic trace
    }
}
