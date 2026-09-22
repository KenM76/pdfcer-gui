//! # `app::ocrband` — the View ▸ Display blend slider
//!
//! One control, drawn for one custom-item kind
//! ([`crate::shell::manifest::OCR_BLEND`]), reporting a new position for
//! `ViewState::ocr_overlay`.
//!
//! ## Why it reports rather than writes
//!
//! The ribbon's custom-item closure holds the open document through a shared
//! borrow — it has to, because every other control in the band reads it to
//! draw itself. So this returns the new position and
//! [`crate::app::PdfcerApp::ribbon_band`] applies it after the band is drawn,
//! in the shape `file.recent`'s parked choice already uses.
//!
//! ## Why there is no command behind it
//!
//! It edits what is on screen: no document, no undo entry, nothing that
//! reaches the file. That is [`crate::shell::manifest::COLOUR_SWATCH`]'s case
//! exactly, and that constant's own note carries the ruling. R8 is met by the
//! item's `shown_when`, which is the *toggle's* selected-condition — see
//! [`crate::shell::manifest::OCR_BLEND`].
//!
//! ## It commits continuously, and that is the point
//!
//! No draft, no commit-on-release, unlike the Format band's number fields.
//! Those commit a document edit and an intermediate value would be an undo
//! entry nobody asked for; this one changes a blend the operator is *looking
//! at* while dragging, and a slider that only took effect on release would be
//! a slider he could not aim. Nothing is re-rasterized — the veil is a tint on
//! the cached texture and the text is vector — so the cost of a frame mid-drag
//! is a repaint, not a render.

use crate::text::ocr as t;

/// How wide the slider is drawn, in points.
///
/// Wide enough that a percent is aimable — the travel is about one point per
/// percent — and narrow enough to sit in a band beside five icon-only
/// switches without pushing the group into the overflow menu on a laptop.
const TRAVEL_PT: f32 = 110.0;

/// Draw the blend control, if `kind` is its kind.
///
/// Returns the position the operator has just put it at, in `0.0..=1.0`, or
/// `None` on every frame the slider was not moved and for every other kind.
///
/// `at` is `ViewState::ocr_overlay` — `None` with nothing open and `None` with
/// the layer off. The item's condition cannot be set in either state, so those
/// are the defensive arm rather than a reachable one, and they draw nothing
/// rather than a slider over a mode that is not running.
///
/// ★ It takes the **number**, not the document, and that is deliberate rather
/// than minimal. A renderer handed an `OpenDoc` can decline for two unrelated
/// reasons — wrong kind, or nothing to show — and a test with no document
/// cannot tell which one answered, so the kind guard could be deleted with
/// every test still green. Taking the value makes the two independent and the
/// guard falsifiable.
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
    ///
    /// The return value is the wrong oracle for these two tests and would make
    /// both of them unfalsifiable: `draw` answers `None` on every frame the
    /// operator did not move the slider, which is every frame in a headless
    /// context. What differs between *drew a slider* and *declined* is whether
    /// anything was allocated, so that is what is measured.
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
    ///
    /// The failure this guards is `manifest::COLOUR_SWATCH`'s, from the other
    /// side. That note records a kind the manifest wrote and no renderer
    /// matched, which left a caption over an empty band for a release; a
    /// renderer that answered *every* kind is the same defect inverted — a
    /// blend slider standing where the Font group asked for a face chooser.
    ///
    /// ★ It is given `Some(0.5)` on purpose. Passing `None` would let the
    /// guard be deleted with this test still green, because the blend would
    /// then be missing too and either arm could be the one answering.
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
    ///
    /// The alternative is a slider sitting at zero over a mode that is not
    /// running, which reads as *the layer is on and showing nothing* rather
    /// than as *the layer is off*.
    #[test]
    fn its_own_kind_with_no_blend_draws_nothing() {
        assert_eq!(laid_out(crate::shell::manifest::OCR_BLEND, None), 0.0);
    }
}
