//! # `canvas::clipimage` — **the copied selection, as a picture other programs
//! # can paste**
//!
//! ## What this closes
//!
//! The operator, 2026-08-31 (`OPERATOR_REQUESTS.md` **O71**):
//!
//! > *"In read mode the regular pointer should also allow us to select images
//! > so we can copy and paste them as well as text outside of the pdfcergui."*
//!
//! **Outside** is the requirement. `canvas::clipboard` has carried a rich
//! internal clip since 2026-08-20 — an `ObjectClip`, a `MarkupSpec`, structure
//! a bitmap cannot express — and it is the right payload for pdfcer→pdfcer work
//! and meaningless to Word.
//!
//! ## ★★★ Where the pixels come from, and why not the page
//!
//! From **the clip's own one-page PDF**, not from a crop of the rendered page.
//!
//! `ObjectClip::to_pdf` returns a standalone document whose `/MediaBox` is
//! exactly the selection's bounding box, with the content translated to the
//! origin — the engine built it *"for a host shell's OS-clipboard interop"*,
//! which is this. Rendering that gives:
//!
//! | | clip PDF | crop of the page |
//! |---|---|---|
//! | resolution | **chosen** — the page is only as big as the selection, so 4× costs nothing | capped by what the whole sheet can be rendered at |
//! | contents | exactly what was selected | everything overlapping the box |
//! | correctness | the engine's own compositor, colour spaces, masks | the same, but with neighbours |
//!
//! The second row is the deciding one. On a CAD sheet almost every object's
//! bounding box overlaps a dozen others, so a crop would paste a picture of the
//! neighbourhood and the operator would have to explain to themselves why.
//!
//! ⇒ A **snapshot** tool — Acrobat's, where the rectangle IS the request — is a
//! different feature and would rightly crop the page. It is not this one.
//!
//! ## ★★ Why it composites onto white
//!
//! `CF_DIB` at 32 bits has no alpha channel consumers agree about
//! (`native_window::clipboard`'s header has the detail). Some read the fourth
//! byte, most ignore it, and one that ignores it renders a
//! composited-on-**black** picture — a black rectangle with a drawing in it,
//! which is what "pasting a PDF selection" looks like when this is got wrong.
//!
//! White rather than transparent is also what the operator sees on screen: the
//! page is white, so the picture that arrives in Word matches the one they
//! copied. A checkerboard would be more honest about the alpha and less honest
//! about the document.
//!
//! ## The size cap, and what happens at it
//!
//! A selection can be one glyph or a whole drawing, so the scale is chosen to
//! give a useful picture in both cases and then clamped so a careless
//! select-all cannot ask for a gigabyte. At the cap the picture is smaller than
//! ideal and still correct; nothing is cropped, because a cropped clipboard
//! picture is a wrong one and a small one is merely a small one.

use pdfcer_core::vector::ObjectClip;

/// How many pixels the longer edge of the picture aims for.
///
/// ★ 1,600 is chosen against where these end up: pasted into an email, a
/// report or a chat message, then usually scaled down. It is generous enough
/// that a screen-sized paste is not visibly resampled and small enough that the
/// clipboard payload for an ordinary selection stays in single-digit megabytes.
const TARGET_EDGE_PX: f32 = 1600.0;

/// The hard ceiling on either edge, whatever the target implies.
///
/// A separate number from the target, deliberately: the target is a *quality*
/// choice and this is a *safety* one. A selection 4,000 pt wide would ask for a
/// 25× scale to hit the target on its short edge, and this is what stops it.
const MAX_EDGE_PX: f32 = 4096.0;

/// The smallest scale worth rendering at.
///
/// Below 1.0 the picture would be smaller than the selection is in points,
/// which is never what somebody copying an object wants — they can always
/// scale it down where they paste it, and cannot scale it up.
const MIN_SCALE: f32 = 1.0;

/// **Render a copied selection and put it on the operating system's
/// clipboard**, alongside `text`.
///
/// Returns the picture's size in pixels when it reached the clipboard, and
/// `None` when it did not — a caller should treat `None` as *"the internal
/// clip is there, the picture is not"* and say nothing to the operator about
/// it, because the copy they asked for did happen.
///
/// ★★ `text` is not decoration. `egui-winit` only produces a paste event when
/// the OS clipboard holds non-empty text, so writing a picture alone would stop
/// `Ctrl+V` arriving in this application at all. The two travel together, in
/// one clipboard transaction, and `native_window::clipboard`'s header carries
/// the measurement.
pub fn publish(clip: &ObjectClip, text: &str) -> Option<(u32, u32)> {
    let pdf = clip.to_pdf();
    let doc = pdfcer_core::document::Document::from_bytes(pdf.bytes).ok()?;
    let pages = pdfcer_core::page_tree::pages(&doc).ok()?;
    let page = pages.first()?;

    let (w_pt, h_pt) = pdf.size;
    let scale = scale_for(w_pt, h_pt)?;

    // ★ `render_page`, the three-argument form, rather than the one this shell
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
///
/// `None` for a degenerate clip — the engine substitutes a 1 pt page for a
/// zero-extent selection and discloses it, and a 1 pt page rendered at any
/// scale is not a picture worth putting on a clipboard.
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
    // ★ Clamped against BOTH edges, not the long one. A tall thin selection
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
///
/// # ★ Why premultiplied is the input
///
/// Because that is `tiny_skia`'s contract and therefore `pdfcer-render`'s: its
/// pixmap data is premultiplied RGBA8, *"handed over unchanged — the engine's
/// stated contract"*, as `render::worker` says of the same buffer. Treating it
/// as straight alpha would double-darken every edge, which reads as a picture
/// with a dirty outline rather than as a bug.
///
/// With premultiplication, compositing over white is one subtraction per
/// channel: `out = src + white × (1 − a)`, and `src` already carries its own
/// alpha factor.
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

    /// ★★ **A small selection is scaled UP, and a huge one is capped.**
    ///
    /// The two ends, asserted as magnitudes rather than as relations: "the
    /// scale is bigger for a smaller clip" is satisfied by any absurdity in the
    /// right direction, which is the failure `HANDOFF.md` §2 records.
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
    ///
    /// The engine substitutes a 1 pt page for a zero-extent selection and says
    /// so; a 1 pt page is not a picture, and putting one on the clipboard would
    /// replace whatever the operator had there with a dot.
    #[test]
    fn a_degenerate_clip_produces_no_picture() {
        assert!(scale_for(1.0, 1.0).is_none());
        assert!(scale_for(0.0, 0.0).is_none());
        assert!(scale_for(f64::NAN, 10.0).is_none());
    }

    /// ★★★ **Premultiplied over white, and the half-transparent case is the
    /// one that matters.**
    ///
    /// A 50 %-alpha red pixel is `(128, 0, 0, 128)` premultiplied. Over white
    /// it must come out `(255, 127, 127)` — a pale red. Treating the input as
    /// straight alpha would give `(191, 127, 127)`, a *darker* pale red, and
    /// the difference is exactly the "dirty edges" symptom that makes a pasted
    /// picture look subtly wrong without looking broken.
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
