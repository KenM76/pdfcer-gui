//! **Which picture this page needs** — the raster-tier decision, and nothing
//! else.
//!
//! # Why this is its own module
//!
//! Split out of `canvas::present` on 2026-09-10, when adding O23's halo tier
//! put that file at 1,503 lines and the R2 gate — *no source file over 1,500
//! lines* — refused it. The gate's own text says what to do about that: *"split
//! the module along its seams — one subject per file — rather than raising the
//! limit"*. This is the seam, and it was a seam before the gate said so.
//!
//! Everything else in `present` is about **drawing and handling** what the
//! frame already has. This is the one block that decides what to **ask the
//! renderer for**, and it answers a question with a single, checkable shape:
//!
//! > *Given where the operator is looking and where this page's ink reaches,
//! > is the whole page the right thing to rasterize?*
//!
//! It has exactly one output — [`OpenDoc::raster_region`] — and
//! `OpenDoc::region_for` feeds that same value into **both** the cache key and
//! the engine request. So setting it here is sufficient to change what is asked
//! for and what counts as stale, and nothing downstream has to be told.
//!
//! # The three tiers, in the order they are tried
//!
//! | tier | when | what is rasterized |
//! |---|---|---|
//! | **region** | the whole-page pixmap would exceed the engine's ceiling, or would blend too much ink | the visible rectangle, whose device size is a multiple of the WINDOW and so constant at every zoom |
//! | **halo** | the page fits, but its ink reaches past the sheet | the union of the crop box and the content box |
//! | **whole** | otherwise | the crop box, as it always was |
//!
//! ★ The order is a safety argument, not a preference: a halo is never smaller
//! than the crop box, so a page that cannot fit a whole-page raster certainly
//! cannot fit a halo. Reaching the second row at all means the whole page fits.
//!
//! # What this module deliberately does NOT do
//!
//! It does not build anything, and it does not touch a texture. In particular
//! it reads the page's content bounds through
//! [`OpenDoc::content_bounds_if_known`] and **never** builds them: a
//! decomposition measured **469 ms** on the operator's own CAD sheet and this
//! runs every frame. The build is `OpenDoc::ensure_content_bounds`, called from
//! `render::settle` after the picture has been asked for — so the halo appears
//! a frame after the page rather than every edit stalling for half a second.
//!
//! # Its trace
//!
//! One line per change, through `canvas::trace::halo`:
//!
//! ```text
//! pdfcer-diag canvas-halo tier=halo known=true box=-160.000,0.000,200.000,200.000
//! ```
//!
//! `known=` is the part that makes a failure readable: `tier=whole known=false`
//! is *"nobody has decomposed this page yet"*, which is normal on the first
//! frame; `tier=whole known=true` is *"the ink was measured and no halo was
//! asked for"*, which on a page with off-page content is a defect.
//! `ui-verify`'s `off_page_visible` reads exactly this distinction.

use crate::app::state::OpenDoc;
use crate::viewer;

use super::trace;

/// Decide this frame's raster tier for the page being acted on, and record it
/// on `doc`.
///
/// ★ Set for the **current page only**. A region is expressed in one page's own
/// coordinate space, and `OpenDoc::region_for` refuses it for any other page
/// rather than rasterizing the wrong part of a neighbour.
///
/// # Arguments
///
/// * `layout` — where every page in this view sits, in strip space.
/// * `current` — the page being acted on. Passed rather than re-read from
///   `doc.view.page_index` so this function and its caller cannot disagree
///   about which page the frame is about.
/// * `raster_scale` — device pixels per point, as the draw loop will key on.
///   Both ceiling questions below are asked at this scale.
/// * `deep` — whether the view is at tier 3, where the scroll offset is no
///   longer the position and the visible rectangle must come from the anchor.
/// * `visible_rect` / `avail` — what of the strip is on screen, and the size of
///   the viewport, in strip space.
pub(super) fn decide(
    doc: &mut OpenDoc,
    layout: &viewer::strip::Strip,
    current: usize,
    raster_scale: f32,
    deep: bool,
    visible_rect: egui::Rect,
    avail: egui::Vec2,
) {
    // ★★★ O24's REGION TIER, decided here because only the canvas knows
    // where the operator is looking.
    //
    // The operator, 2026-08-22, at 2382 % on a US Letter page:
    //
    // > *"I got a requested raster size 14580x18868 is empty or exceeds
    // > MAX_PIXMAP_EDGE"*
    //
    // 18,868 device pixels against a 16,384 cap. Above that ceiling the
    // whole-page raster cannot be made at all, so the request becomes the
    // visible rectangle instead — whose device size is a multiple of the
    // WINDOW and therefore constant at every zoom.
    //
    // ★ Set for the page being acted on only. A region is in one page's
    // own coordinate space, and `OpenDoc::region_for` refuses it for any
    // other page rather than rasterizing the wrong part of a neighbour.
    doc.raster_region = None;
    if let Some(place) = layout.rect_of(current) {
        let extent = viewer::page_extent_pts(&doc.pages[current]);
        let frame = crate::render::region::PageFrame::of(&doc.pages[current]);
        // ★★★ O23's "see" half. Where the ink on this page actually
        // reaches — which on a page with an object dragged off the sheet
        // is bigger than the sheet.
        //
        // `_if_known` and not a build: the decomposition costs 469 ms on
        // the operator's own drawing and this runs every frame. It is
        // built from `render::settle`, after the picture has landed, so
        // the halo appears a frame after the page rather than the edit
        // stalling for half a second. See
        // `OpenDoc::content_bounds_if_known` for the whole argument.
        let content = doc.content_bounds_if_known();
        // ★ The THIRD argument, added 2026-08-26: whether this page is
        // blended in ink, and at what ceiling. It ends the whole-page tier
        // at the colour ceiling as well as the pixmap one — but only for a
        // page that has been observed asking for ink, which on a CAD sheet
        // is never. `render::strategy::Ink` carries the whole argument,
        // including the 263 % measurement that made the unconditional
        // version unacceptable.
        if crate::render::strategy::for_page(extent, raster_scale, doc.ink_at(current))
            == crate::render::strategy::Strategy::Region
            && place.width() > 0.0
            && place.height() > 0.0
        {
            // What is visible OF THIS PAGE, in strip space, then in the
            // page's own points. The two scales are derived from the
            // placement rather than from the zoom, so a page whose
            // placement has been rounded still maps exactly onto itself.
            // ★★ At tier 3 the visible rect comes from the ANCHOR, for the
            // same reason the placement does: `place` has a magnitude of
            // ~10^12 at deep zoom, and `seen.min.x - place.min.x` subtracts
            // two huge `f32`s to get a small one — losing exactly the
            // precision the answer needs. `DeepAnchor::visible_rect` does
            // that subtraction in `f64`.
            let visible_canvas = if deep {
                let anchor = doc
                    .deep_anchor
                    .unwrap_or_else(viewer::deep::DeepAnchor::origin);
                // ★★★ HANDED ON IN `f64` — O24i. This used to cast to
                // `f32` here, and that one line is what stopped detail
                // improving past about 10⁷ %: the rect is a few times
                // 10⁻⁸ pt wide at an absolute position near 540, and no
                // `f32` holds both magnitudes. See
                // `render::strategy::region_for`.
                Some(anchor.visible_rect((avail.x, avail.y), f64::from(doc.view.zoom)))
            } else {
                // ★★★ `halo::reach(place, ..)` and NOT `place`. Above the
                // pixmap ceiling the request is the visible rectangle, and
                // intersecting it with the SHEET is what kept an off-page
                // object out of the raster at every deep zoom even after
                // the halo tier covered every shallow one. `reach` returns
                // `place` unchanged on an ordinary page, so this line is a
                // no-op for every document that has no off-page content.
                let seen = visible_rect
                    .intersect(crate::render::halo::reach(place, extent, frame, content));
                if seen.width() > 0.0 && seen.height() > 0.0 {
                    let sx = extent.0 / place.width();
                    let sy = extent.1 / place.height();
                    // ★ Widened to `f64`, losslessly. Below the deep
                    // threshold the `f32` arithmetic was never the
                    // problem — the rect is a fair fraction of the page
                    // there — but `page_region` takes one type, and a
                    // second entry point that narrowed would be the seam
                    // the defect crawled back through.
                    Some((
                        f64::from((seen.min.x - place.min.x) * sx),
                        f64::from((seen.min.y - place.min.y) * sy),
                        f64::from((seen.max.x - place.min.x) * sx),
                        f64::from((seen.max.y - place.min.y) * sy),
                    ))
                } else {
                    None
                }
            };
            if let Some(visible_canvas) = visible_canvas {
                doc.raster_region = Some((
                    current,
                    crate::render::region::page_region(visible_canvas, frame),
                ));
            }
            trace::halo("region", content.is_some(), doc.region_for(current));
        } else {
            // ★★★ THE HALO TIER — O23's "see" half, and the reason an
            // object placed off the sheet is drawn at all.
            //
            // `render_page` sizes its pixmap to the `/CropBox`. Nothing
            // culls the content — `render::offpage` proves that against
            // the pinned engine — there are simply no pixels out there to
            // put it in. So when this page's ink reaches past the sheet,
            // ask for the bigger box instead.
            //
            // ★ SECOND, after the region tier, and the order is the
            // safety argument: a halo is never smaller than the crop box,
            // so if the whole page does not fit, the halo certainly does
            // not. Reaching this branch at all means the whole-page
            // raster fits; `halo::region` then asks the same ceiling
            // question of the bigger box and answers `None` if the answer
            // changed — in which case this page stays on the whole-page
            // tier and the off-page object is simply not visible until the
            // operator zooms out, which is the honest failure and the one
            // a clamp would have hidden.
            let halo = crate::render::halo::region(frame.crop(), content, raster_scale);
            if let Some(halo) = halo {
                doc.raster_region = Some((current, halo));
            }
            trace::halo(
                if halo.is_some() { "halo" } else { "whole" },
                content.is_some(),
                halo,
            );
        }
    }
}
