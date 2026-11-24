//! The print preview's sheet number: a box showing the sheet on screen, in
//! which a typed number goes to that sheet.
//!
//! The typed text lives in egui's memory under the box's id while it has
//! focus; the box shows the current sheet otherwise. A number goes to its
//! sheet when the box loses focus (Enter, Tab or a click elsewhere); Escape,
//! or a number outside `1..=sheets`, leaves the sheet as it was. The box is as
//! wide as the largest sheet number, measured in the theme's font, so the
//! arrow after it stays put as the number changes.

use egui::Ui;
use pdfcer_gui_base::text::print as t;

use super::super::PrintDialog;

/// Trace region of the sheet-number box.
pub(super) const REGION: &str = "print.preview.sheet-number";

/// Draw `Sheet [n] of total`, going to a typed sheet.
pub(super) fn number(ui: &mut Ui, dialog: &mut PrintDialog, shown: usize, sheets: usize) {
    ui.label(t::preview_sheet());
    let id = ui.id().with("print-preview-sheet-number");
    let typed: Option<String> = ui.data(|d| d.get_temp(id));
    let mut text = typed.unwrap_or_else(|| (shown + 1).to_string());
    let font = egui::TextStyle::Body.resolve(ui.style());
    let digit = ui.ctx().fonts_mut(|f| f.glyph_width(&font, '0'));
    let digits = sheets.max(1).to_string().len().max(2);
    // escape-disposition: not-content (a navigation box; Escape keeps the sheet)
    let edit = egui::TextEdit::singleline(&mut text)
        .id(id)
        .desired_width(digit * digits as f32)
        .horizontal_align(egui::Align::Center)
        .char_limit(digits);
    let response = ui.add(edit).on_hover_text(t::preview_sheet_tooltip());
    crate::diag::ui_rect_visible(REGION, response.rect, ui.clip_rect());
    if response.has_focus() {
        ui.data_mut(|d| d.insert_temp(id, text));
    } else if response.lost_focus() {
        ui.data_mut(|d| d.remove::<String>(id));
        let escaped = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let target = text
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|n| (1..=sheets).contains(n));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "print-preview-sheet-typed text={} went={}",
                text.trim(),
                target.filter(|_| !escaped).map_or(0, |n| n)
            )
        });
        if let Some(n) = target
            && !escaped
            && n - 1 != shown
        {
            dialog.preview_page = n - 1;
            super::reset_view(dialog);
        }
    }
    ui.label(t::preview_sheet_of(sheets));
}
