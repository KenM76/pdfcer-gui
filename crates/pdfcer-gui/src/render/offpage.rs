//! # `render::offpage` — proving the engine can rasterize past the page edge
//!
//! ## Why this module is nothing but tests
//!
//! `OPERATOR_REQUESTS.md` **O23**, second half:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/offpage.md`.

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

    /// **A region entirely outside the page rasterizes.**
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

    /// **The pixmap is sized to the REQUESTED region, not to the overlap
    /// with the page.**
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

    /// **A region larger than the page in every direction works too.**
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

    /// **`PageObjects::page_bbox()` includes off-page geometry**, which is
    /// what makes the scrollable extent computable in one call.
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
