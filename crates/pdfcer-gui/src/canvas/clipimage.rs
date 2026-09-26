//! # `canvas::clipimage` — **the copied selection, as a picture other programs
//! # can paste**
//!
//! ## What this closes
//!
//! The operator (`OPERATOR_REQUESTS.md` **O71**):
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/clipimage.md`.

use pdfcer_core::vector::ObjectClip;

/// How many pixels the longer edge of the picture aims for.
const TARGET_EDGE_PX: f32 = 1600.0;

/// The hard ceiling on either edge, whatever the target implies.
const MAX_EDGE_PX: f32 = 4096.0;

/// The smallest scale worth rendering at.
const MIN_SCALE: f32 = 1.0;

/// **Render a copied selection and put it on the operating system's
/// clipboard**, alongside `text`.
pub fn publish(clip: &ObjectClip, text: &str) -> Option<(u32, u32)> {
    let pdf = clip.to_pdf();
    let doc = pdfcer_core::document::Document::from_bytes(pdf.bytes).ok()?;
    let pages = pdfcer_core::page_tree::pages(&doc).ok()?;
    let page = pages.first()?;

    let (w_pt, h_pt) = pdf.size;
    let scale = scale_for(w_pt, h_pt)?;

    // `render_page`, the three-argument form, rather than the one this shell
    // uses for the canvas. That one takes a `DocumentView` and `RenderOptions`
    // because it renders an EDITING SESSION with the operator's annotation and
    // layer choices applied. This renders a freshly parsed standalone document
    // with no session, no annotations and no layers — the clip's own PDF — so
    // there is nothing for those parameters to say.
    let rendered = pdfcer_render::render_page(&doc, page, scale).ok()?;
    let pixmap = rendered.pixmap;
    let (width, height) = (pixmap.width(), pixmap.height());
    let rgba = on_white(pixmap.data());

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("clipboard-image w={width} h={height} scale={scale:.2}")
    });
    native_window::clipboard::set_image_and_text(&rgba, width, height, text)
        .then_some((width, height))
}

/// The render scale for a clip of this size, in points.
fn scale_for(w_pt: f64, h_pt: f64) -> Option<f32> {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a page dimension is bounded by the format's own limits and is exact in f32 at these magnitudes" // ui-text-exempt: a lint justification, never displayed
    )]
    let (w, h) = (w_pt as f32, h_pt as f32);
    if !(w.is_finite() && h.is_finite()) || w < 2.0 || h < 2.0 {
        return None;
    }
    let long = w.max(h);
    let scale = (TARGET_EDGE_PX / long).max(MIN_SCALE);
    // Clamped against BOTH edges, not the long one. A tall thin selection
    // scaled to hit the target on its long edge is fine; a wide one scaled by
    // the same factor could still exceed the ceiling on the other axis, and
    // the ceiling is about total pixels rather than about shape.
    Some(
        scale
            .min(MAX_EDGE_PX / w)
            .min(MAX_EDGE_PX / h)
            .max(MIN_SCALE),
    )
}

/// Composite premultiplied RGBA over white, returning straight RGBA.
fn on_white(premultiplied: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(premultiplied.len());
    for px in premultiplied.chunks_exact(4) {
        let a = u32::from(px[3]);
        let unfilled = 255 - a;
        for channel in &px[..3] {
            #[allow(
                clippy::cast_possible_truncation,
                reason = "the sum is at most 255 by construction: a premultiplied channel is at most its alpha" // ui-text-exempt: a lint justification, never displayed
            )]
            let value = (u32::from(*channel) + unfilled).min(255) as u8;
            out.push(value);
        }
        // Opaque: the picture has been flattened onto paper.
        out.push(255);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A small selection is scaled UP, and a huge one is capped.**
    #[test]
    fn the_scale_fills_the_target_and_stops_at_the_ceiling() {
        // 100 pt wide → 16× fills 1,600 px.
        let small = scale_for(100.0, 50.0).expect("a real clip");
        assert!((small - 16.0).abs() < 0.01, "{small}");

        // 3,000 pt wide → the target implies 0.53×, which is below the floor,
        // so it renders at 1:1 rather than smaller than the selection itself.
        let big = scale_for(3000.0, 1000.0).expect("a real clip");
        assert!((big - 1.0).abs() < 0.01, "{big}");

        // 200 × 4,000 pt → the target implies 0.4 on the long edge, the floor
        // lifts it to 1.0, and 1.0 × 4,000 is under the 4,096 ceiling.
        let tall = scale_for(200.0, 4000.0).expect("a real clip");
        assert!((tall - 1.0).abs() < 0.01, "{tall}");
    }

    /// A degenerate clip is refused rather than rendered.
    #[test]
    fn a_degenerate_clip_produces_no_picture() {
        assert!(scale_for(1.0, 1.0).is_none());
        assert!(scale_for(0.0, 0.0).is_none());
        assert!(scale_for(f64::NAN, 10.0).is_none());
    }

    /// **Premultiplied over white, and the half-transparent case is the
    /// one that matters.**
    #[test]
    fn half_transparent_red_becomes_pale_red_not_dark_red() {
        let out = on_white(&[128, 0, 0, 128]);
        assert_eq!(out, vec![255, 127, 127, 255]);
    }

    /// An opaque pixel passes through unchanged, and a transparent one becomes
    /// white.
    #[test]
    fn the_two_ends_are_exact() {
        assert_eq!(on_white(&[10, 20, 30, 255]), vec![10, 20, 30, 255]);
        assert_eq!(on_white(&[0, 0, 0, 0]), vec![255, 255, 255, 255]);
    }
}
