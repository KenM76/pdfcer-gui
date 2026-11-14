//! # `dialogs::print::colour` — print in colour, grey or black only
//!
//! The choice is applied to the rendered sheets with
//! `pdfcer_print::output_color::apply_output_color`, after rendering and
//! before spooling, so the document is never touched. The preview raster goes
//! through the same call, so the preview shows what the paper will get.
//! Separations are the fourth choice: plain pages print as one grey sheet
//! per ink through `pdfcer_render::plates` (see `plates`). Per dialog, not
//! remembered.

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

/// The trace word and region suffix for separations.
pub(super) const SEPARATIONS: &str = "separations"; // ui-text-exempt: diagnostic token

/// The four choices: the three output colours, then separations, which print
/// in colour as far as `output_color` is concerned. A change of colour drops
/// the preview raster so it is redrawn; separations leave it, as the preview
/// stays in colour.
pub(super) fn radios(ui: &mut egui::Ui, dialog: &mut PrintDialog) {
    ui.label(t::heading());
    for (mode, label) in [
        (OutputColor::Color, t::colour()),
        (OutputColor::Grayscale, t::grey()),
        (OutputColor::BlackOnly, t::black()),
    ] {
        let on = dialog.output_color == mode && !dialog.separations;
        let radio = ui.radio(on, label);
        region(word(mode), radio.rect, ui);
        if radio.clicked() && !on {
            dialog.separations = false;
            if dialog.output_color != mode {
                dialog.output_color = mode;
                dialog.preview_texture = None;
            }
        }
    }
    let radio = ui.radio(dialog.separations, t::separations());
    region(SEPARATIONS, radio.rect, ui);
    if radio.clicked() && !dialog.separations {
        dialog.separations = true;
        if dialog.output_color != OutputColor::Color {
            dialog.output_color = OutputColor::Color;
            dialog.preview_texture = None;
        }
    }
    let note = if dialog.separations {
        Some(t::separations_note())
    } else {
        (dialog.output_color == OutputColor::BlackOnly).then(t::black_note)
    };
    if let Some(note) = note {
        ui.label(
            egui::RichText::new(note)
                .small()
                .color(egui_shell::theme::Theme::of(ui.ctx()).palette.text_muted),
        );
    }
}

fn region(word: &str, rect: egui::Rect, ui: &egui::Ui) {
    crate::diag::ui_rect_visible(
        &format!(
            "{}{word}",
            pdfcer_gui_base::printregions::REGION_COLOUR_PREFIX
        ),
        rect,
        ui.clip_rect(),
    );
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
