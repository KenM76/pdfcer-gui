//! # `panels::redact::appearance` — what a redaction looks like once applied
//!
//! The operator's **one** choice of fill colour and overlay caption, held for
//! the whole panel and applied to every mark authored from it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/redact/appearance.md`.

use crate::text::redact as t;
use pdfcer_core::vartext::Quadding;

/// The region the fill swatch publishes, so a check can find and drive it.
pub const REGION_FILL: &str = "redact.appearance.fill"; // ui-text-exempt: trace region name, never displayed
/// The region the overlay-text field publishes.
pub const REGION_OVERLAY: &str = "redact.appearance.overlay"; // ui-text-exempt: trace region name, never displayed

pub use pdfcer_gui_base::redactlook::{Appearance, Fill, MAX_OVERLAY_CHARS};

/// **Draw the appearance controls**, editing the panel's own state in place.
pub fn show(ui: &mut egui::Ui, state: &mut crate::panels::PanelsState) {
    egui::CollapsingHeader::new(egui::RichText::new(t::appearance_heading()))
        .default_open(false)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(t::appearance_intro()).small().weak());
            ui.add_space(6.0);
            controls(ui, &mut state.redact_mut().appearance);
        });
}

/// The controls themselves, over a borrowed [`Appearance`].
///
fn controls(ui: &mut egui::Ui, appearance: &mut Appearance) {
    // ---- the fill --------------------------------------------------------
    ui.horizontal(|ui| {
        ui.label(t::fill_label());
        for option in [Fill::Black, Fill::White, Fill::Transparent] {
            // A `selectable_value` rather than a radio: three named
            // alternatives on one row, which is what the settings window's
            // `option` helper would draw vertically and there is no room for
            // that here.
            ui.selectable_value(&mut appearance.fill, option, t::fill_option_label(option));
        }
        // The custom colour is a SWATCH, not a fourth segment, because the
        // only useful preview of a colour is the colour. Seeded from whatever
        // is currently chosen so that switching from Black to a custom colour
        // starts somewhere sensible rather than at an arbitrary hue.
        let mut rgb = match appearance.fill {
            Fill::Custom(r, g, b) => [r as f32, g as f32, b as f32],
            Fill::White => [1.0, 1.0, 1.0],
            _ => [0.0, 0.0, 0.0],
        };
        let swatch = ui
            .color_edit_button_rgb(&mut rgb)
            .on_hover_text(t::fill_option_label(Fill::Custom(0.0, 0.0, 0.0)));
        crate::diag::ui_rect(REGION_FILL, swatch.rect);
        if swatch.changed() {
            appearance.fill = Fill::Custom(f64::from(rgb[0]), f64::from(rgb[1]), f64::from(rgb[2]));
        }
    });
    if appearance.fill == Fill::Transparent {
        // Said at the control and not in a tooltip: a tooltip is not read
        // before a choice is made, and this is the choice an operator can
        // misread as "do not redact".
        ui.label(egui::RichText::new(t::fill_transparent_note()).small());
    }

    ui.add_space(8.0);

    // ---- the caption -----------------------------------------------------
    ui.label(t::overlay_label());
    let field = ui.add(
        // escape-disposition: keeps-draft — typed into the redaction appearance
        // this panel holds, which outlives the field and the panel both.
        egui::TextEdit::singleline(&mut appearance.overlay_text)
            .hint_text(t::overlay_hint())
            .char_limit(MAX_OVERLAY_CHARS),
    );
    crate::diag::ui_rect(REGION_OVERLAY, field.rect);

    // Everything below is about a caption, so none of it is drawn when there
    // is not one. A justification control for text that does not exist is the
    // "control governing a control governing nothing" this module's header
    // names.
    if !appearance.has_overlay() {
        return;
    }

    // The legibility warning, and it is a DISCLOSURE rather than advice —
    // `.small()` without `.weak()`, the same weight the settings window
    // reserves for something pdfcer owes the operator rather than something it
    // is explaining. The engine cannot colour this text and told us so; an
    // operator who applies a black caption onto a black box has lost nothing
    // recoverable, but they have lost the caption.
    if appearance.caption_is_illegible() {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(t::overlay_illegible_warning())
                .small()
                .color(egui_shell::theme::Theme::of(ui.ctx()).palette.danger),
        );
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(t::quadding_label());
        for option in [Quadding::Left, Quadding::Center, Quadding::Right] {
            ui.selectable_value(
                &mut appearance.quadding,
                option,
                t::quadding_option_label(option),
            );
        }
    });
    ui.add_space(4.0);
    ui.label(egui::RichText::new(t::overlay_bound()).small().weak());
}
