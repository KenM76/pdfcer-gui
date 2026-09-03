//! # `render::region` — turning "what is on screen" into "what to rasterize"
//!
//! `OPERATOR_REQUESTS.md` **O24**, and the failure that forced it into the
//! canvas on 2026-08-22:
//!
//! > *"I got a requested raster size 14580x18868 is empty or exceeds
//! > MAX_PIXMAP_EDGE when I got to 2382% zoom."*
//!
//! A US Letter page at 2382 % is 18,868 device pixels tall against a 16,384
//! cap. The whole-page raster cannot be made, and the answer he proposed is the
//! right one: *"reducing the raster sized area to around the cursor zoomed area
//! to what will fit"*.
//!
//! ## The two conversions, and why they are here rather than in the canvas
//!
//! | | |
//! |---|---|
//! | [`page_region`] | the visible part of a page, in the **PDF's** coordinates, ready for `render_page_region` |
//! | [`region_on_screen`] | where that rectangle's raster belongs on screen |
//!
//! They are exact inverses of each other and are the only place this shell
//! crosses between screen space and PDF space for a *raster*. Keeping them
//! together is what lets the round trip be a unit test — and a round trip is
//! precisely the property that matters, because getting one of the two slightly
//! wrong produces a page that is drawn *almost* in the right place, which reads
//! as a rendering bug rather than as a coordinate one.
//!
//! ## ★★ The y flip, which is the part that goes wrong
//!
//! Canvas space is y-**down** from the page's top-left; PDF user space is
//! y-**up** from its bottom-left. `render_page_region` documents its rectangle
//! as *"page space, pre-scale — the same coordinate system as `Page::crop_box`,
//! y-up"*, so the flip happens here, once, in both directions.
//!
//! ★ A flip that is applied twice is the identity, and a flip that is missed
//! shows the operator the *opposite end* of the page from the one they are
//! pointing at — which at 2382 % is a uniform field of whatever happens to be
//! there, and looks exactly like a blank raster.

use pdfcer_core::page_tree::Rect;

/// The visible part of a page in **PDF user space**, y-up, ready for
/// `pdfcer_render::render_page_region`.
///
/// * `visible_canvas` — what the operator can see of this page, in canvas
///   points from the page's top-left, y-down.
/// * `page_pts` — the page's own size in points, as
///   [`crate::viewer::page_extent_pts`] reports it.
///
/// The rectangle is quantised by [`super::strategy::region_for`] first, so a
/// small pan asks for the rectangle already rasterized — see that function for
/// why that is the difference between panning smoothly and waiting for a redraw
/// on every pixel of movement.
#[must_use]
/// ★★ `visible_canvas` is `f64`: at deep zoom it holds a rectangle a few times
/// 10⁻⁸ pt wide at an absolute position near 540, and `f32` cannot carry both
/// magnitudes at once. See [`super::strategy::region_for`].
pub fn page_region(visible_canvas: (f64, f64, f64, f64), page_pts: (f32, f32)) -> Rect {
    let (x0, y0, x1, y1) = super::strategy::region_for(visible_canvas);
    let height = f64::from(page_pts.1);
    // y-down from the top becomes y-up from the bottom, so the two y values
    // also swap ends: the canvas's smaller y is the PDF's larger one.
    Rect::from_corners(x0, height - y1, x1, height - y0)
}

/// Where a region's raster belongs on screen, given where the whole page would
/// have been drawn.
///
/// `page_screen` is the rect the page occupies on screen — what the whole-page
/// texture would have filled. The returned rect is the sub-rectangle of it that
/// `region` covers, and it is routinely **larger than the screen and partly
/// negative**, because the region carries overscan beyond the viewport. That is
/// correct and must not be clamped: the texture covers that area, and clamping
/// the destination without cropping the source would stretch the image.
#[must_use]
pub fn region_on_screen(region: Rect, page_pts: (f32, f32), page_screen: egui::Rect) -> egui::Rect {
    let (w, h) = (f64::from(page_pts.0), f64::from(page_pts.1));
    if w <= 0.0 || h <= 0.0 {
        return page_screen;
    }
    let sx = f64::from(page_screen.width()) / w;
    let sy = f64::from(page_screen.height()) / h;
    // The flip again, in the other direction: the PDF's upper y is the
    // canvas's smaller one.
    let left = f64::from(page_screen.min.x) + region.llx * sx;
    let right = f64::from(page_screen.min.x) + region.urx * sx;
    let top = f64::from(page_screen.min.y) + (h - region.ury) * sy;
    let bottom = f64::from(page_screen.min.y) + (h - region.lly) * sy;
    egui::Rect::from_min_max(
        egui::pos2(left as f32, top as f32),
        egui::pos2(right as f32, bottom as f32),
    )
}

/// Where a region's raster belongs on screen at **deep zoom**, computed from
/// the `f64` anchor rather than from the page's own screen rect.
///
/// # ★★★ Why the other one stops working, and it is not the strip
///
/// [`region_on_screen`] derives its answer from `page_screen` — where the
/// WHOLE page would be drawn. At four billion percent that rect has a
/// magnitude around 10^12 screen pixels, where an `f32`'s spacing is **131,072
/// pixels**. The rect being drawn is about 1,400 pixels across.
///
/// So the precision is lost in an **intermediate a thousand times larger than
/// the result**. The page's full extent is computed, quantised to something
/// coarser than the whole window, and the region's position is then derived
/// from it — inheriting an error that never had to exist.
///
/// ★ That is why the answer is neither a 32-bit strip nor a 64-bit one. Making
/// the strip `f64` would carry the huge number more precisely; **not forming
/// it** is better, costs nothing, and leaves one code path instead of two.
/// Every large magnitude is subtracted inside `f64` before anything narrows —
/// the same technique the engine's own deep-zoom commit describes as *"one
/// subtraction moved into f64"*, and the same one [`DeepAnchor`] itself uses.
///
/// [`DeepAnchor`]: crate::viewer::deep::DeepAnchor
#[must_use]
pub fn region_on_screen_deep(
    region: Rect,
    page_pts: (f32, f32),
    anchor: crate::viewer::deep::DeepAnchor,
    zoom: f64,
    viewport_origin: egui::Pos2,
) -> egui::Rect {
    let height = f64::from(page_pts.1);
    // PDF y-up back to canvas y-down, then anchor-relative, then scaled. The
    // subtraction happens BEFORE the multiply, so nothing large is ever formed.
    let to_screen = |cx: f64, cy: f64| {
        (
            f64::from(viewport_origin.x) + f64::from(anchor.screen.0) + (cx - anchor.page.0) * zoom,
            f64::from(viewport_origin.y) + f64::from(anchor.screen.1) + (cy - anchor.page.1) * zoom,
        )
    };
    let (x0, y0) = to_screen(region.llx, height - region.ury);
    let (x1, y1) = to_screen(region.urx, height - region.lly);
    egui::Rect::from_min_max(
        egui::pos2(x0 as f32, y0 as f32),
        egui::pos2(x1 as f32, y1 as f32),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// US Letter, which is the page the operator's failure was on.
    const LETTER: (f32, f32) = (612.0, 792.0);

    /// ★★★ **The two conversions are inverses**, which is the property the
    /// canvas actually depends on.
    ///
    /// Asserted as a round trip rather than against hand-computed numbers: a
    /// pair that agrees with each other draws the raster where it belongs, and
    /// two numbers that each look plausible in isolation can still disagree.
    #[test]
    fn a_region_maps_to_screen_and_back_to_itself() {
        let page_screen = egui::Rect::from_min_size(
            egui::pos2(100.0, 50.0),
            egui::vec2(612.0 * 4.0, 792.0 * 4.0),
        );
        let visible = (200.0, 300.0, 260.0, 345.0);
        let region = page_region(visible, LETTER);
        let on_screen = region_on_screen(region, LETTER, page_screen);

        // Recover the canvas-space rect from the screen rect and compare it to
        // the quantised region the conversion actually used.
        let sx = f64::from(page_screen.width()) / f64::from(LETTER.0);
        let sy = f64::from(page_screen.height()) / f64::from(LETTER.1);
        let back = (
            ((f64::from(on_screen.min.x) - f64::from(page_screen.min.x)) / sx) as f32,
            ((f64::from(on_screen.min.y) - f64::from(page_screen.min.y)) / sy) as f32,
            ((f64::from(on_screen.max.x) - f64::from(page_screen.min.x)) / sx) as f32,
            ((f64::from(on_screen.max.y) - f64::from(page_screen.min.y)) / sy) as f32,
        );
        let wanted = super::super::strategy::region_for(visible);
        for (got, want) in [
            (back.0, wanted.0),
            (back.1, wanted.1),
            (back.2, wanted.2),
            (back.3, wanted.3),
        ] {
            assert!(
                (f64::from(got) - want).abs() < 0.01,
                "round trip lost the rect: {back:?} vs {wanted:?}"
            );
        }
    }

    /// ★★ **The y flip happens, and in the right direction.**
    ///
    /// Looking at the TOP of the page must ask for the page's HIGH y in PDF
    /// space. A missed flip shows the opposite end of the sheet, which at deep
    /// zoom is a uniform field and reads as a blank raster rather than as a
    /// coordinate error.
    #[test]
    fn looking_at_the_top_of_the_page_asks_for_the_pdf_top() {
        // A window near the page's top edge, in canvas space (y-down).
        let region = page_region((0.0, 0.0, 60.0, 45.0), LETTER);
        assert!(
            region.ury > f64::from(LETTER.1) * 0.9,
            "the top of the page is the PDF's high y: {region:?}"
        );
        // …and near the bottom asks for low y.
        let low = page_region((0.0, 747.0, 60.0, 792.0), LETTER);
        assert!(
            low.lly < f64::from(LETTER.1) * 0.2,
            "the bottom of the page is the PDF's low y: {low:?}"
        );
    }

    /// ★ **The screen rect may extend past the page's own**, because the region
    /// carries overscan. Clamping it would stretch the texture, so this pins
    /// that it is left alone.
    #[test]
    fn the_screen_rect_may_reach_outside_the_page() {
        let page_screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(612.0, 792.0));
        // A window at the very top-left: the overscan reaches off the page.
        let region = page_region((0.0, 0.0, 60.0, 45.0), LETTER);
        let on_screen = region_on_screen(region, LETTER, page_screen);
        assert!(
            on_screen.min.x < page_screen.min.x || on_screen.min.y < page_screen.min.y,
            "overscan should reach outside the page: {on_screen:?}"
        );
    }

    /// ★★★ **The deep placement stays exact where the shallow one cannot.**
    ///
    /// At four billion percent the page's own screen rect has a magnitude of
    /// ~10^12 px, where `f32`'s spacing is 131,072 px — coarser than the whole
    /// window. The anchor-based path never forms that number, so the rect it
    /// returns is correct to a fraction of a pixel.
    ///
    /// Asserted by placing the anchor ON the region's own corner: the answer
    /// must then be the viewport origin exactly, at any zoom.
    #[test]
    fn the_deep_placement_is_exact_at_zooms_where_f32_is_not() {
        let region = page_region((300.0, 400.0, 300.1, 400.1), LETTER);
        let origin = egui::pos2(0.0, 0.0);
        for zoom in [1.0e6_f64, 1.0e8, 4.3e9, 1.0e11] {
            // The anchor holds the region's top-left corner at the origin.
            let anchor = crate::viewer::deep::DeepAnchor {
                page: (region.llx, f64::from(LETTER.1) - region.ury),
                screen: (0.0, 0.0),
            };
            let r = region_on_screen_deep(region, LETTER, anchor, zoom, origin);
            assert!(
                r.min.x.abs() < 0.5 && r.min.y.abs() < 0.5,
                "at {zoom}x the anchored corner drifted to {:?}",
                r.min
            );
            assert!(
                r.width() > 0.0 && r.height() > 0.0,
                "at {zoom}x the rect collapsed: {r:?}"
            );
        }
    }

    /// A degenerate page yields the page's own rect rather than a division by
    /// zero — the canvas then draws as it always did.
    #[test]
    fn a_degenerate_page_falls_back_to_the_page_rect() {
        let page_screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        let region = Rect::from_corners(0.0, 0.0, 10.0, 10.0);
        assert_eq!(
            region_on_screen(region, (0.0, 0.0), page_screen),
            page_screen
        );
    }
}
