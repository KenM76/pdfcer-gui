//! # `dialogs::print::colour` — print in colour, grey or black only
//!
//! The choice is applied to the rendered sheets with
//! `pdfcer_print::output_color::apply_output_color`, after rendering and
//! before spooling, so the document is never touched. The preview raster goes
//! through the same call, so the preview shows what the paper will get.
//! Per dialog, not remembered.

use pdfcer_print::output_color::{OutputColor, apply_output_color};

use super::PrintDialog;
use crate::text::print::colour as t;

/// The trace word for `mode`, also the suffix of its region.
pub(super) const fn word(mode: OutputColor) -> &'static str {
    match mode {
        // ui-text-exempt: diagnostic token, never displayed in the UI
        OutputColor::Color => "colour",
        // ui-text-exempt: diagnostic token, never displayed in the UI
        OutputColor::Grayscale => "grey",
        // ui-text-exempt: diagnostic token, never displayed in the UI
        OutputColor::BlackOnly => "black",
    }
}

/// The three choices. A change drops the preview raster so it is redrawn
/// in the new colour.
pub(super) fn radios(ui: &mut egui::Ui, dialog: &mut PrintDialog) {
    ui.label(t::heading());
    for (mode, label) in [
        (OutputColor::Color, t::colour()),
        (OutputColor::Grayscale, t::grey()),
        (OutputColor::BlackOnly, t::black()),
    ] {
        let radio = ui.radio(dialog.output_color == mode, label);
        crate::diag::ui_rect_visible(
            &format!(
                "{}{}",
                pdfcer_gui_base::printregions::REGION_COLOUR_PREFIX,
                word(mode)
            ),
            radio.rect,
            ui.clip_rect(),
        );
        if radio.clicked() && dialog.output_color != mode {
            dialog.output_color = mode;
            dialog.preview_texture = None;
        }
    }
    if dialog.output_color == OutputColor::BlackOnly {
        ui.label(
            egui::RichText::new(t::black_note())
                .small()
                .color(egui_shell::theme::Theme::of(ui.ctx()).palette.text_muted),
        );
    }
}

/// Recolour one raster (premultiplied RGBA8) for `mode`, tracing `what`
/// (`preview` or `sheet`) and how many pixels changed.
pub(super) fn recolour(rgba: &mut [u8], mode: OutputColor, what: &str) {
    if mode == OutputColor::Color {
        return;
    }
    let changed = apply_output_color(rgba, mode);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-colour of={what} mode={} changed={changed} pixels={}",
            word(mode),
            rgba.len() / 4
        )
    });
}
