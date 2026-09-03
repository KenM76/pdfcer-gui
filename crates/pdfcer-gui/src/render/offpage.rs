//! # `render::offpage` — proving the engine can rasterize past the page edge
//!
//! ## Why this module is nothing but tests
//!
//! `OPERATOR_REQUESTS.md` **O23**, second half:
//!
//! > *"also objects should still be reachable even if they are off the page."*
//!
//! Answering that against `pdfcer-core` and `pdfcer-render` source on 2026-08-21
//! produced an unusually clean result — **no engine change is required**. The
//! decomposer applies no page-box culling of any kind, so an object painted at
//! `(-5000, -5000)` is already in `PageObjects::objects` with a truthful
//! negative bounding box; `hit_test_point_all`'s only predicate on the query
//! point is `is_finite`, so it will already select one. The single place
//! off-page content disappears is the **raster**, and only because the
//! whole-page entry point sizes its pixmap to the `/CropBox`.
//!
//! `pdfcer_render::render_page_region` takes an arbitrary page-space rectangle
//! and — this is the part the feature depends on — **never clamps or intersects
//! it with the crop box**.
//!
//! ## ★★ The caveat that produced this file
//!
//! That last claim is true *by construction*: there is no code in the region
//! path that could reject an off-page rectangle. It is also, in the engine's own
//! test suite, **entirely unproven**. Its region tests cover sub-rectangles,
//! quadrant tiling and a stroke-mitre band — every one of them **inside** the
//! page.
//!
//! Correct-by-construction and covered-by-a-test are different things, and this
//! project has spent a day on the difference. A shell feature built on an
//! unexercised engine path is a feature whose first failure will look like a
//! shell defect.
//!
//! So: before any of O23's second half is built, these run.
//!
//! ## What this is NOT
//!
//! Not a test of `pdfcer-render`. That crate is
//! [read-only to this project](../../../../PROJECT_PLAN.md) and its coverage is
//! its own business. This is **pdfcer-gui asserting the properties it is about to
//! rely on**, in this repository, so that if a future engine bump changes them
//! the failure lands here — on the consumer, with a message naming the feature
//! that cared — rather than as a mysterious blank canvas.
//!
//! That is the same posture `app::settings`' funnel test takes toward
//! `pdfcer-core`'s option structs: assert the contract you depend on, where you
//! depend on it.

#[cfg(test)]
mod tests {
    use pdfcer_core::page_tree::Rect;

    use crate::app::state::{FOUR_PAGES, open_fixture};

    /// The scale every case renders at. Small on purpose: these assert
    /// *geometry*, not fidelity, and a big pixmap only makes them slow.
    const SCALE: f32 = 0.5;

    /// Render `region` of page 0 of the four-page fixture, or say why not.
    fn region_of(doc: &crate::app::state::OpenDoc, region: Rect) -> (u32, u32) {
        let page = doc.pages.first().expect("the fixture has a page");
        let options = pdfcer_render::RenderOptions::default();
        let view = doc.session.view();
        let out = pdfcer_render::render_page_region(&view, page, SCALE, region, &options)
            .expect("an off-page region must rasterize rather than refuse");
        (out.pixmap.width(), out.pixmap.height())
    }

    /// ★★★ **A region entirely outside the page rasterizes.**
    ///
    /// The property O23's second half stands on. If this ever fails, the engine
    /// has started clamping the region against the crop box and *"objects off
    /// the page are reachable"* is no longer buildable in this shell without an
    /// engine change — which is exactly the finding that would otherwise be
    /// made by an operator staring at a blank rectangle.
    #[test]
    fn a_region_entirely_off_the_page_still_rasterizes() {
        let doc = open_fixture(FOUR_PAGES);
        let page = doc.pages.first().expect("a page").clone();
        let crop = page.crop_box;

        // A square well to the LEFT of and BELOW the page — no overlap at all.
        let side = 100.0_f64;
        let region = Rect::from_corners(
            crop.llx - 400.0,
            crop.lly - 400.0,
            crop.llx - 400.0 + side,
            crop.lly - 400.0 + side,
        );

        let (w, h) = region_of(&doc, region);
        assert!(
            w > 0 && h > 0,
            "an off-page region produced an empty pixmap: {w}x{h}"
        );
    }

    /// ★★ **The pixmap is sized to the REQUESTED region, not to the overlap
    /// with the page.**
    ///
    /// The distinction that matters for a canvas. A build that quietly
    /// intersected the region with the crop box would still return `Ok` and a
    /// non-empty pixmap for a region that merely *touches* the page — and the
    /// canvas would then draw a raster smaller than the rectangle it asked
    /// for, which presents as content sliding rather than as an error.
    ///
    /// Asserted with a tolerance of one pixel per axis: `region_device_geometry`
    /// floors the origin and ceils the extent, so an exact equality would be
    /// pinning rounding rather than behaviour.
    #[test]
    fn the_pixmap_matches_the_region_asked_for_not_its_overlap_with_the_page() {
        let doc = open_fixture(FOUR_PAGES);
        let page = doc.pages.first().expect("a page").clone();
        let crop = page.crop_box;

        // Straddling the page's left edge: half on, half off.
        let width = 200.0_f64;
        let height = 150.0_f64;
        let region = Rect::from_corners(
            crop.llx - width / 2.0,
            crop.lly + 10.0,
            crop.llx + width / 2.0,
            crop.lly + 10.0 + height,
        );

        let (w, h) = region_of(&doc, region);
        let want_w = (width * f64::from(SCALE)).round() as i64;
        let want_h = (height * f64::from(SCALE)).round() as i64;

        assert!(
            (i64::from(w) - want_w).abs() <= 1,
            "width {w} is not the region's {want_w} — the region was clipped to the page"
        );
        assert!(
            (i64::from(h) - want_h).abs() <= 1,
            "height {h} is not the region's {want_h} — the region was clipped to the page"
        );
    }

    /// ★ **A region larger than the page in every direction works too.**
    ///
    /// The shape a pasteboard actually asks for: the page plus a margin all
    /// round, in one raster. Separate from the two above because it is the case
    /// where the region *contains* the crop box rather than missing or
    /// straddling it, and a clamp would be invisible in the other two if it
    /// only triggered on containment.
    #[test]
    fn a_region_containing_the_whole_page_and_a_margin_works() {
        let doc = open_fixture(FOUR_PAGES);
        let page = doc.pages.first().expect("a page").clone();
        let crop = page.crop_box;

        let margin = 120.0_f64;
        let region = Rect::from_corners(
            crop.llx - margin,
            crop.lly - margin,
            crop.urx + margin,
            crop.ury + margin,
        );

        let (w, h) = region_of(&doc, region);
        let page_w = ((crop.urx - crop.llx) * f64::from(SCALE)).round() as u32;
        assert!(
            w > page_w,
            "a region wider than the page produced a pixmap no wider than it: {w} vs {page_w}"
        );
        assert!(h > 0);
    }

    /// ★★ **`PageObjects::page_bbox()` includes off-page geometry**, which is
    /// what makes the scrollable extent computable in one call.
    ///
    /// Asserted here rather than taken from the engine's documentation because
    /// O23's plan uses it as the source of *"what must I be able to scroll to in
    /// order to reach everything"*. If the decomposer ever started culling
    /// against the page box, this union would silently shrink to the page and
    /// the shell would stop being able to reach the very objects the feature
    /// exists for — with nothing failing.
    ///
    /// The fixture has no off-page content, so this asserts the weaker, stable
    /// property: the union is non-empty and is not *narrower* than the ink it
    /// contains. It is the guard rail, not the demonstration — a fixture with
    /// deliberate off-page geometry is worth adding when the feature is built.
    #[test]
    fn the_content_union_is_available_and_non_empty() {
        let doc = open_fixture(FOUR_PAGES);
        let provider = doc.page_objects().expect("page 0 decomposes");
        let union = provider.page_objects().page_bbox();
        assert!(
            union.max.x > union.min.x && union.max.y > union.min.y,
            "the content union is empty on a page with three objects: {union:?}"
        );
    }
}
