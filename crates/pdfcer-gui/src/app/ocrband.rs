//! # `app::ocrband` — the View ▸ Display blend slider
//!
//! One control, drawn for one custom-item kind
//! ([`crate::shell::manifest::OCR_BLEND`]), reporting a new position for
//! `ViewState::ocr_overlay`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/ocrband.md`.

use crate::text::ocr as t;

/// How wide the slider is drawn, in points.
const TRAVEL_PT: f32 = 110.0;

/// Draw the blend control, if `kind` is its kind.
pub(super) fn draw(ui: &mut egui::Ui, kind: &str, at: Option<f32>) -> Option<f32> {
    if kind != crate::shell::manifest::OCR_BLEND {
        return None;
    }
    let overlay = at?;
    // The number the PAINTER used, so the readout and the page agree even on
    // a value that arrived corrupt — `canvas::ocrlayer::painted_fraction`'s
    // own note carries the argument.
    let mut percent = (crate::canvas::ocrlayer::painted_fraction(overlay) * 100.0).round();
    let response = ui.add_sized(
        egui::Vec2::new(TRAVEL_PT, ui.spacing().interact_size.y),
        egui::Slider::new(&mut percent, 0.0..=100.0)
            .text(t::layer_blend_label())
            .suffix(t::layer_blend_suffix())
            .fixed_decimals(0),
    );
    crate::diag::ui_rect(
        &egui_shell::ribbon::report::band_item(crate::shell::manifest::OCR_BLEND),
        response.rect,
    );
    let response = response.on_hover_text(t::layer_blend_tooltip());
    if !response.changed() {
        return None;
    }
    let moved = percent / 100.0;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("ocr-blend set={moved:.3}")
    });
    Some(moved)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How wide the laid-out region is after `draw` is asked for one kind.
    fn laid_out(kind: &'static str, at: Option<f32>) -> f32 {
        let ctx = egui::Context::default();
        let mut width = f32::NAN;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let before = ui.min_rect().width();
            let _ = draw(ui, kind, at);
            width = ui.min_rect().width() - before;
        });
        width
    }

    /// The kind guard is load-bearing: a foreign kind draws nothing even when
    /// there is a perfectly good blend to draw.
    #[test]
    fn a_foreign_kind_draws_nothing_even_with_a_blend_to_draw() {
        for kind in [
            crate::shell::manifest::FONT_FACE,
            crate::shell::manifest::MARKUP_OPACITY,
            crate::shell::manifest::COLOUR_SWATCH,
            crate::shell::manifest::RECENT_FILES,
        ] {
            assert_eq!(
                laid_out(kind, Some(0.5)),
                0.0,
                "{kind} is not this renderer's kind"
            );
        }
    }

    /// …and its own kind, with a blend, draws the slider — which is the
    /// control arm without which the test above proves only that this module
    /// draws nothing at all.
    #[test]
    fn its_own_kind_with_a_blend_draws_the_slider() {
        assert!(
            laid_out(crate::shell::manifest::OCR_BLEND, Some(0.5)) >= TRAVEL_PT,
            "the slider asks for at least its own travel"
        );
    }

    /// With the layer off there is no blend, and the control does not invent
    /// one.
    #[test]
    fn its_own_kind_with_no_blend_draws_nothing() {
        assert_eq!(laid_out(crate::shell::manifest::OCR_BLEND, None), 0.0);
    }
}
