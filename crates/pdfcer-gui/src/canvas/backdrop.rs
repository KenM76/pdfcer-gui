//! # `canvas::backdrop` — the low-resolution page under the sharp one
//!
//! **Operator request, 2026-08-26:**
//!
//! > *"the screen should never be blank while waiting to render when zooming
//! > out — there should be at least a low resolution zoom of the newly panned
//! > or zoomed out area instead of just remaining blank while the higher
//! > definition render occurs."*
//!
//! Two things live here and they are two halves of one subject: the picture
//! that stops the page going blank, and the number that makes its absence
//! falsifiable.
//!
//! ## ★★★ The defect, measured before anything was built
//!
//! Above the pixmap ceiling a raster is a picture of the **visible region**
//! rather than of the page, and `canvas::mod` places it at *its own* region's
//! rect — which is right, and is what stops the page lurching during a pan.
//! What it cannot do is cover ground the region never included. Zoom out from
//! deep zoom and the held picture shrinks to a speck of a big sheet; pan far
//! enough and it leaves the window altogether.
//!
//! Driven on a real CAD sheet: zooming out from 3590 % held **`covered=0.000`
//! for about twenty frames**. The operator is looking at blank paper.
//!
//! ## ★★ Why a screenshot is the wrong oracle here, against this project's rule
//!
//! `D:/dev/rag/egui/` records that layout and clipping defects have exactly one
//! oracle and it is a rendered screenshot. **This is not a layout defect, it is
//! a timing one**, and the interval is shorter than a window capture takes.
//! Three camera-based checks were built and driven before this module existed,
//! and every one of them was *unable to fail*:
//!
//! 1. *"is the canvas near-uniform?"* — passed, because the uncovered area is
//!    drawn as the page's own white and a technical drawing is ~90 % white
//!    anyway.
//! 2. *"count ink during, compare with ink once settled"* — passed, with
//!    **identical** counts both sides: the raster landed before the shutter.
//! 3. The same again with the stale-texture path deliberately sabotaged so the
//!    page had to blank. **Still passed.**
//!
//! What the application knows and a camera does not is whether the pixels it
//! drew are a picture of the whole visible area or of a fraction of it. That is
//! one exact ratio, and [`publish_coverage`] publishes it.
//!
//! ## The fix costs no extra rasterisation
//!
//! [`crate::app::state::OpenDoc::base_texture`] is not a new render. It is the
//! whole-page texture the shell already made, kept rather than dropped when a
//! sharper one replaces it, and only while it is under
//! [`crate::render::raster::BASE_MAX_PIXELS`]. A document opens at a fit zoom,
//! so the first raster of every page qualifies; the huge whole-page rasters
//! near the region tier never do.

use egui::Ui;

use crate::app::state::OpenDoc;

/// Paint the backdrop for `page`, and say whether anything was painted.
///
/// ## ★★ It is drawn at the page's WHOLE rect, and that is the difference
///
/// The live texture above the pixmap ceiling is a picture of one region and
/// must be placed at that region's rect or the operator sees the right pixels
/// in the wrong place. The backdrop is a picture of the **whole page**, so it
/// goes at the whole page's rect and has no gaps to leave. That asymmetry is
/// the entire mechanism.
///
/// ## ★ Order is the cheapest part of this
///
/// It is painted first and the sharp texture is painted on top, so wherever the
/// sharp one reaches, it wins. No blending, no masking, no seam arithmetic —
/// the painter's algorithm does all of it, and the operator sees full detail
/// exactly where it exists and a soft stand-in everywhere else.
///
/// ## Two filters, each excluding a specific wrong picture
///
/// * **the same page** — a backdrop captured a frame before a page change is a
///   picture of the neighbour, which is the hazard `OpenDoc::region_for`'s own
///   page check exists for one level up;
/// * **the same edit epoch** — an out-of-date backdrop would show an object the
///   operator has just deleted, in the part of the page the sharp raster does
///   not cover. Rule 4's line is that pdfcer may be **fuzzy** and may not be
///   **sneaky**, and stale content is the second. The cost of that strictness
///   is that an edit made while zoomed deep leaves no backdrop until the
///   operator zooms out far enough to make a small whole-page raster again — a
///   narrower gap than the one this closes, and an honest one.
pub(super) fn paint(ui: &Ui, doc: &OpenDoc, page: usize, current: usize, rect: egui::Rect) -> bool {
    // ★ Only for the CURRENT page. A strip neighbour has its own texture and
    // its own "no texture yet" state, which `render::strip::draw_page_state`
    // already answers honestly; giving it this page's pixels would be drawing
    // one page on another.
    if page != current {
        return false;
    }
    let Some(base) = doc
        .base_texture
        .as_ref()
        .filter(|t| t.key.page() == page)
        // ★ Per-page (O74). The rule this enforces is unchanged and is rule
        // 4's — "a backdrop from before an edit would show content the document
        // no longer has" — but the question is now asked of the page the
        // backdrop is a picture of, rather than of the whole document.
        .filter(|_| doc.base_texture_epoch == doc.page_epochs.get(doc.view.page_index))
    else {
        return false;
    };
    egui::Image::from_texture(&base.texture).paint_at(ui, rect);
    true
}

/// Publish how much of the visible page actually has a picture on it.
///
/// ★★★ **The backdrop counts**, and counting it is what makes this number mean
/// *"is the operator looking at the drawing?"* rather than *"is the sharp
/// raster ready?"*. Those are different questions and only the first is the
/// defect that was reported.
///
/// ★ Traced on change rather than per frame, so what reaches the channel is the
/// sequence of **distinct states** the canvas passed through. A blank held for
/// twenty frames appears once, which is what makes a minimum over the series
/// meaningful and what keeps this off the `canvas-pointer` list of traces that
/// buried a capture in identical lines.
pub(super) fn publish_coverage(
    ui: &Ui,
    doc: &OpenDoc,
    rect: egui::Rect,
    paint_rect: egui::Rect,
    backdrop: bool,
    textured: bool,
    is_current: bool,
) {
    if !is_current {
        return;
    }
    let want = rect.intersect(ui.clip_rect());
    let area = |r: egui::Rect| f64::from(r.width().max(0.0) * r.height().max(0.0));
    let got = if backdrop {
        want
    } else {
        paint_rect.intersect(want)
    };
    let covered = if area(want) > 0.0 {
        area(got) / area(want)
    } else {
        1.0
    };
    crate::diag::trace_on_change("canvas-coverage", || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "covered={covered:.3} textured={} backdrop={} zoom={:.3}",
            u8::from(textured),
            u8::from(backdrop),
            doc.view.zoom
        )
    });
}
