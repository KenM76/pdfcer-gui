//! # `dialogs::compact_streams` — the Brotli switch in Save a compacted copy
//!
//! Contract: [`brotli_switch`] draws the switch that sets
//! `recompress::Choices::brotli`, greyed while recompressing is off. What
//! Brotli means for readers is the plan's own note, shown with the other
//! notes once the copy is built, never retyped here.

use egui::Ui;

use crate::text::compact_streams as t;

/// The Brotli switch.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BROTLI: &str = "compact.brotli";

/// Draw the switch; whether it changed.
pub fn brotli_switch(ui: &mut Ui, recompress: bool, brotli: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_space(ui.spacing().indent);
        ui.add_enabled_ui(recompress, |ui| {
            let switch = ui
                .checkbox(brotli, t::brotli_toggle())
                .on_hover_text(t::brotli_tooltip())
                .on_disabled_hover_text(t::brotli_needs_recompress());
            crate::diag::ui_rect_visible(REGION_BROTLI, switch.rect, ui.clip_rect());
            changed = switch.changed();
        });
    });
    changed
}
