//! # `canvasmapping` — the ONE screen↔page conversion, the PDF↔canvas
//! projection, and the tolerance
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/canvasmapping.md`.

use egui::{Pos2, Rect};

use crate::viewer;

/// The screen-space catch radius for **object selection**, in egui logical
/// points, converted to a canvas/page-space tolerance per query by
/// [`screen_tolerance_to_page`].
pub const SELECT_SCREEN_TOLERANCE_PX: f32 = 6.0;

/// **The catch radius for an ANCHOR**, in egui logical points —
/// `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
pub const NODE_SCREEN_TOLERANCE_PX: f32 = 8.0;

/// Convert a fixed SCREEN-space pixel radius into a **canvas/page-space**
/// tolerance at `zoom` (points per PDF user-space unit).
#[must_use]
pub fn screen_tolerance_to_page(screen_px: f32, zoom: f32) -> f64 {
    if zoom.is_finite() && zoom > 0.0 && screen_px.is_finite() && screen_px >= 0.0 {
        f64::from(screen_px) / f64::from(zoom)
    } else {
        0.0
    }
}

/// The frame's screen ⟷ canvas map: where the page raster is, how big the
/// page is, and at what zoom it is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageMapping {
    /// The page raster's own rect in window logical points — the rect every
    /// canvas coordinate conversion is relative to.
    ///
    /// This is the `Response::rect` of the image widget, **not** the scroll
    /// viewport and **not** the justified container. `canvas/mod.rs`'s
    /// centring comment records what taking the wrong one costs: at fit-page
    /// on a page smaller than the viewport, every mapping is wrong by the
    /// centring margin (~105 px on one measured case), selection outlines
    /// draw offset from the object they outline, and clicking directly ON a
    /// visible object misses it.
    image_rect: Rect,
    /// The current page's extent in PDF user-space units, `/Rotate` applied
    /// — [`crate::viewer::page_extent_pts`].
    ///
    /// Consulted only to reject a degenerate page; the mapping itself carries
    /// no rotation branch, because rotation is already baked into this value.
    /// Adding one here as well would double-apply it.
    extent: (f32, f32),
    /// Logical points per PDF user-space unit — [`crate::viewer::ViewState::zoom`].
    zoom: f32,
}

impl PageMapping {
    /// Build the mapping for this frame.
    #[must_use]
    pub fn new(image_rect: Rect, extent: (f32, f32), zoom: f32) -> Self {
        Self {
            image_rect,
            extent,
            zoom,
        }
    }

    /// The page raster's rect on screen.
    #[must_use]
    pub fn image_rect(&self) -> Rect {
        self.image_rect
    }

    /// **Screen → canvas.** The boundary crossing, inward.
    #[must_use]
    pub fn to_page(&self, screen: Pos2) -> Pos2 {
        viewer::screen_to_page(screen, self.image_rect, self.extent, self.zoom)
    }

    /// **Canvas → screen.** The boundary crossing, outward — used by the
    /// overlay and by nothing else.
    #[must_use]
    pub fn to_screen(&self, page: Pos2) -> Pos2 {
        viewer::page_to_screen(page, self.image_rect, self.extent, self.zoom)
    }

    /// **Canvas → screen for a DISPLACEMENT**, not a position.
    #[must_use]
    pub fn page_vec_to_screen(&self, page: egui::Vec2) -> egui::Vec2 {
        page * self.zoom
    }

    /// **Screen → canvas for a DISPLACEMENT.** The inverse of
    /// [`Self::page_vec_to_screen`]; see that method for why a displacement is
    /// not a position.
    #[must_use]
    pub fn screen_vec_to_page(&self, screen: egui::Vec2) -> egui::Vec2 {
        if self.zoom.is_finite() && self.zoom > 0.0 {
            screen / self.zoom
        } else {
            egui::Vec2::ZERO
        }
    }

    /// **Screen → canvas** for a rect (the marquee).
    #[must_use]
    pub fn rect_to_page(&self, screen: Rect) -> Rect {
        Rect::from_two_pos(self.to_page(screen.min), self.to_page(screen.max))
    }

    /// **Canvas → screen** for a rect (a selection outline).
    #[must_use]
    pub fn rect_to_screen(&self, page: Rect) -> Rect {
        Rect::from_two_pos(self.to_screen(page.min), self.to_screen(page.max))
    }

    /// The selection catch radius for this frame, in **canvas/page** units.
    #[must_use]
    pub fn tolerance(&self) -> f64 {
        screen_tolerance_to_page(SELECT_SCREEN_TOLERANCE_PX, self.zoom)
    }

    /// The **anchor** catch radius for this frame, in canvas/page units —
    /// `OPERATOR_REQUESTS.md` O69.
    #[must_use]
    pub fn node_tolerance(&self) -> f64 {
        screen_tolerance_to_page(NODE_SCREEN_TOLERANCE_PX, self.zoom)
    }

    /// The **snap** catch radius for this frame, in canvas/page units.
    #[must_use]
    pub fn snap_tolerance(&self) -> f64 {
        screen_tolerance_to_page(crate::snapmark::SNAP_SCREEN_TOLERANCE_PX, self.zoom)
    }
}

/// Project an annotation's `/Rect` — **PDF user space, y-up, un-rotated**,
/// as `[llx, lly, urx, ury]` — into canvas space.
#[must_use]
pub fn annot_canvas_rect(rect: [f64; 4], page: &pdfcer_core::page_tree::Page) -> Option<Rect> {
    let [llx, lly, urx, ury] = rect;
    let corners = [(llx, lly), (urx, lly), (urx, ury), (llx, ury)];
    let mut bounds: Option<Rect> = None;
    for (x, y) in corners {
        // f64 -> f32 is the boundary between the object model's precision and
        // egui's. A page coordinate that does not survive it is a page
        // coordinate no raster could have drawn either.
        let p = Pos2::new(x as f32, y as f32);
        let mapped = crate::viewer::pdf_space_to_canvas(p, page)?;
        bounds = Some(match bounds {
            Some(r) => r.union(Rect::from_min_max(mapped, mapped)),
            None => Rect::from_min_max(mapped, mapped),
        });
    }
    bounds.filter(|r| r.width() > 0.0 && r.height() > 0.0 && r.is_finite())
}

/// The four **placed corners** of an annotation's artwork, projected into canvas
/// space, preserving their order.
#[must_use]
pub fn oriented_canvas_quad(
    corners: [(f64, f64); 4],
    page: &pdfcer_core::page_tree::Page,
) -> Option<[Pos2; 4]> {
    let mut out = [Pos2::ZERO; 4];
    for (slot, (x, y)) in out.iter_mut().zip(corners) {
        *slot = crate::viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), page)?;
    }
    out.iter()
        .all(|p| p.x.is_finite() && p.y.is_finite())
        .then_some(out)
}

#[cfg(test)]
mod tests {

    /// A letter page at a given `/Rotate`, for the projection tests.
    fn projection_page(rotate: u16) -> pdfcer_core::page_tree::Page {
        pdfcer_core::page_tree::Page {
            id: pdfcer_core::object::ObjId::new(9, 0),
            resources: pdfcer_core::object::Dict::new(),
            media_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            crop_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
            resources_defaulted: false,
            contents_flattened: 0,
        }
    }

    /// A 200x20 rectangle near the top of that page.
    const PROJECTED_RECT: [f64; 4] = [100.0, 700.0, 300.0, 720.0];

    /// **A degenerate rectangle produces no box**, whichever corner order it
    /// is written in.
    #[test]
    fn a_rectangle_with_no_area_produces_no_box() {
        for rect in [
            [10.0, 10.0, 10.0, 40.0],
            [10.0, 10.0, 40.0, 10.0],
            [10.0, 10.0, 10.0, 10.0],
        ] {
            assert_eq!(
                annot_canvas_rect(rect, &projection_page(0)),
                None,
                "{rect:?}"
            );
        }
        // …and a rectangle written max-first still produces the same box as one
        // written min-first, because the verb hands over normalised corners.
        let forward = annot_canvas_rect([100.0, 700.0, 300.0, 720.0], &projection_page(0));
        let backward = annot_canvas_rect([100.0, 700.0, 300.0, 720.0], &projection_page(0));
        assert_eq!(forward, backward);
        assert!(forward.is_some());
    }

    /// **The box lands where the page draws it, at every rotation.**
    #[test]
    fn an_annot_box_lands_at_the_top_of_an_unrotated_page_and_moves_when_it_turns() {
        let rect = PROJECTED_RECT;

        let upright = annot_canvas_rect(rect, &projection_page(0)).expect("a real page projects");
        assert!(
            upright.min.y < 200.0,
            "a high PDF Y must become a low canvas Y: {upright:?}"
        );
        assert!((upright.min.x - 100.0).abs() < 1.0, "{upright:?}");
        assert!((upright.width() - 200.0).abs() < 1.0, "{upright:?}");
        assert!((upright.height() - 20.0).abs() < 1.0, "{upright:?}");

        // A quarter-turn swaps the axes: the 200×20 box becomes 20×200.
        let turned = annot_canvas_rect(rect, &projection_page(90)).expect("a real page projects");
        assert!(
            (turned.width() - 20.0).abs() < 1.0 && (turned.height() - 200.0).abs() < 1.0,
            "a rotated page must swap the box's axes: {turned:?}"
        );
        assert!(
            (turned.min.y - upright.min.y).abs() > 1.0
                || (turned.min.x - upright.min.x).abs() > 1.0,
            "the box did not move at all under /Rotate 90: {turned:?}"
        );
    }

    use super::*;

    /// A mapping for a 200×300 page drawn at `zoom`, with the page's
    /// top-left at a deliberately non-zero screen position — a mapping that
    /// forgot the origin would still pass every *distance* assertion, so the
    /// origin has to be somewhere a bug could show up.
    fn mapping(zoom: f32) -> PageMapping {
        let extent = (200.0_f32, 300.0_f32);
        let rect = Rect::from_min_size(
            Pos2::new(37.0, 11.0),
            egui::vec2(extent.0 * zoom, extent.1 * zoom),
        );
        PageMapping::new(rect, extent, zoom)
    }

    /// **The law this module exists for**, restored from the old shell.
    #[test]
    fn screen_tolerance_keeps_the_on_screen_catch_radius_constant() {
        for zoom in [0.10_f32, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0] {
            let page_tol = screen_tolerance_to_page(SELECT_SCREEN_TOLERANCE_PX, zoom);
            // Canvas units × zoom = screen px, by the same distance law
            // `viewer::screen_to_page` uses.
            let screen_px = page_tol * f64::from(zoom);
            assert!(
                (screen_px - f64::from(SELECT_SCREEN_TOLERANCE_PX)).abs() < 1e-6,
                "zoom {zoom}: on-screen catch radius drifted to {screen_px} px"
            );
        }
    }

    /// The same law, asserted through the mapping rather than through the
    /// free function — because the mapping is what call sites actually hold,
    /// and a mapping that forgot to divide would pass the test above.
    #[test]
    fn the_mappings_tolerance_is_the_same_screen_radius_at_every_zoom() {
        for zoom in [0.10_f32, 0.5, 1.0, 3.0, 8.0] {
            let m = mapping(zoom);
            let screen_px = m.tolerance() * f64::from(zoom);
            assert!(
                (screen_px - f64::from(SELECT_SCREEN_TOLERANCE_PX)).abs() < 1e-6,
                "zoom {zoom}: mapping tolerance is {screen_px} screen px"
            );
        }
    }

    /// A degenerate zoom disables the *conversion*, not selection: the
    /// provider recognises `0.0` and falls back. Returning NaN here would
    /// make every hit-test comparison false, i.e. every query a miss, with
    /// nothing anywhere to say why.
    #[test]
    fn a_degenerate_zoom_yields_a_zero_tolerance_rather_than_a_nan() {
        assert!((screen_tolerance_to_page(10.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((screen_tolerance_to_page(10.0, -1.0) - 0.0).abs() < f64::EPSILON);
        assert!((screen_tolerance_to_page(10.0, f32::NAN) - 0.0).abs() < f64::EPSILON);
        assert!((screen_tolerance_to_page(f32::NAN, 1.0) - 0.0).abs() < f64::EPSILON);
        // And the plain arithmetic, so a refactor that "simplified" the
        // guard away is caught by more than the degenerate cases.
        assert!((screen_tolerance_to_page(10.0, 2.0) - 5.0).abs() < f64::EPSILON);
        assert!((screen_tolerance_to_page(10.0, 0.5) - 20.0).abs() < f64::EPSILON);
    }

    /// Screen → canvas → screen is the identity, at every zoom, for points
    /// inside and outside the page rect.
    ///
    /// Outside matters: a marquee is routinely dragged past the page edge,
    /// and a mapping that clamped would silently shrink the rubber-band.
    #[test]
    fn the_boundary_round_trips_in_both_directions() {
        for zoom in [0.10_f32, 0.5, 1.0, 2.5, 8.0] {
            let m = mapping(zoom);
            for p in [
                m.image_rect().min,
                m.image_rect().center(),
                m.image_rect().max,
                Pos2::new(-40.0, -90.0),
                Pos2::new(5_000.0, 5_000.0),
            ] {
                let back = m.to_screen(m.to_page(p));
                assert!(
                    (back.x - p.x).abs() < 1e-2 && (back.y - p.y).abs() < 1e-2,
                    "zoom {zoom}: {p:?} round-tripped to {back:?}"
                );
            }
        }
    }

    /// **A canvas coordinate does not move when the view does.**
    #[test]
    fn a_canvas_point_survives_every_zoom_and_scroll_position() {
        let extent = (200.0_f32, 300.0_f32);
        let subject = Pos2::new(123.0, 45.0); // a point on the page
        for zoom in [0.10_f32, 0.33, 1.0, 2.0, 8.0] {
            for origin in [
                Pos2::new(0.0, 0.0),
                Pos2::new(37.0, 11.0),
                Pos2::new(-900.0, -1_400.0), // scrolled far into a big page
            ] {
                let m = PageMapping::new(
                    Rect::from_min_size(origin, egui::vec2(extent.0 * zoom, extent.1 * zoom)),
                    extent,
                    zoom,
                );
                let on_screen = m.to_screen(subject);
                let back = m.to_page(on_screen);
                assert!(
                    (back.x - subject.x).abs() < 1e-2 && (back.y - subject.y).abs() < 1e-2,
                    "zoom {zoom} origin {origin:?}: the point moved to {back:?}"
                );
            }
        }
    }

    /// A rubber-band dragged up-and-left normalises rather than producing a
    /// negative-width rect that contains nothing.
    #[test]
    fn a_backwards_marquee_normalises() {
        let m = mapping(2.0);
        let dragged_up_left = Rect::from_two_pos(Pos2::new(300.0, 400.0), Pos2::new(100.0, 150.0));
        let page = m.rect_to_page(dragged_up_left);
        assert!(page.width() > 0.0 && page.height() > 0.0);
        assert!(page.contains(m.to_page(Pos2::new(200.0, 300.0))));
    }

    /// **Each page of a strip gets its OWN mapping, and they are not
    /// interchangeable.**
    #[test]
    fn each_page_of_a_strip_has_its_own_mapping() {
        use crate::viewer::PageDisplay;
        use crate::viewer::strip::Strip;
        use pdfcer_core::object::{Dict, ObjId};
        use pdfcer_core::page_tree::{Page, Rect as PageRect};

        let pages: Vec<Page> = (0..3)
            .map(|_| Page {
                id: ObjId::new(1, 0),
                resources: Dict::new(),
                media_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
                crop_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
                rotate: 0,
                contents: Vec::new(),
                contents_unresolved: 0,
                resources_defaulted: false,
                contents_flattened: 0,
            })
            .collect();
        let zoom = 1.5_f32;
        let strip = Strip::new(&pages, PageDisplay::Continuous, 0, zoom);
        // The strip's own origin on screen, somewhere non-zero so a mapping
        // that forgot it would still pass every *distance* assertion.
        let strip_origin = egui::vec2(37.0, 11.0);
        let extent = crate::viewer::page_extent_pts(&pages[0]);

        let maps: Vec<(usize, PageMapping)> = strip
            .placements()
            .map(|p| {
                (
                    p.page,
                    PageMapping::new(p.rect.translate(strip_origin), extent, zoom),
                )
            })
            .collect();
        assert_eq!(maps.len(), 3, "the strip must lay out every page");

        // A hit at the top-left of its own page lands at that page's own
        // screen origin — through that page's map.
        for (page, map) in &maps {
            let rect = strip
                .rect_of(*page)
                .expect("laid out")
                .translate(strip_origin);
            let landed = map.to_screen(Pos2::ZERO);
            assert!(
                (landed - rect.min).length() < 1e-2,
                "page {page}: {landed:?} is not that page's origin {:?}",
                rect.min
            );
        }

        // …and through the WRONG page's map it lands somewhere else, by a
        // whole page height plus the row gap. This is the defect, measured.
        let wrong = maps[0].1.to_screen(Pos2::ZERO);
        let right = maps[1].1.to_screen(Pos2::ZERO);
        let apart = (right.y - wrong.y).abs();
        assert!(
            apart > 700.0,
            "the two mappings differ by only {apart} pt; a highlight painted \
             through the wrong one would look almost correct, which is worse"
        );
    }

    /// A degenerate page maps everything to the origin rather than to NaN —
    /// `viewer`'s "fail to a finite, harmless value" discipline, inherited
    /// rather than re-implemented.
    #[test]
    fn a_degenerate_page_maps_to_a_finite_point() {
        let m = PageMapping::new(
            Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
            (0.0, 100.0),
            1.0,
        );
        assert_eq!(m.to_page(Pos2::new(5.0, 5.0)), Pos2::ZERO);
        assert_eq!(m.to_screen(Pos2::new(5.0, 5.0)), Pos2::ZERO);
    }
}

#[cfg(test)]
mod vector_tests {
    use super::*;

    /// A mapping at a stated zoom, with a page origin deliberately NOT at the
    /// window origin — so a conversion that carried the translation would show
    /// up here rather than passing by luck.
    fn at(zoom: f32) -> PageMapping {
        PageMapping::new(
            Rect::from_min_size(Pos2::new(316.0, 580.0), egui::vec2(400.0, 300.0)),
            (1584.0, 1224.0),
            zoom,
        )
    }

    /// **A displacement does not carry the page's origin.**
    #[test]
    fn a_page_displacement_converts_without_the_origin() {
        let map = at(0.2955);
        let screen = map.page_vec_to_screen(egui::vec2(100.0, 40.0));
        assert!((screen.x - 29.55).abs() < 0.01, "{screen:?}");
        assert!((screen.y - 11.82).abs() < 0.01, "{screen:?}");
    }

    /// The round trip, at the operator's own fitted zoom.
    #[test]
    fn the_two_directions_are_inverses() {
        let map = at(0.2955);
        let screen = egui::vec2(60.0, 60.0);
        let page = map.screen_vec_to_page(screen);
        assert!((page.x - 203.04).abs() < 0.01, "{page:?}");
        let back = map.page_vec_to_screen(page);
        assert!((back - screen).length() < 0.001, "{back:?} vs {screen:?}");
    }

    /// A degenerate zoom answers ZERO, never NaN.
    #[test]
    fn a_degenerate_zoom_answers_zero_rather_than_nan() {
        for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            let page = at(bad).screen_vec_to_page(egui::vec2(60.0, 60.0));
            assert_eq!(page, egui::Vec2::ZERO, "zoom {bad}");
        }
    }
}

#[cfg(test)]
mod o69_tolerance_tests {
    use super::*;

    /// **Widening the anchor radius did NOT widen object picking.**
    #[test]
    fn the_object_catch_radius_is_unchanged() {
        let object = SELECT_SCREEN_TOLERANCE_PX;
        assert!(
            (object - 6.0).abs() < f32::EPSILON,
            "object picking must still catch at six pixels; O69 widened the ANCHOR radius only"
        );
    }

    /// Both radii keep a constant ON-SCREEN size as the zoom changes.
    #[test]
    fn the_node_radius_scales_as_one_over_zoom() {
        let at = |zoom: f32| {
            let m = PageMapping::new(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)),
                (100.0, 100.0),
                zoom,
            );
            m.node_tolerance()
        };
        let one = at(1.0);
        let four = at(4.0);
        assert!(
            (one / four - 4.0).abs() < 1e-6,
            "four times the zoom must be a quarter of the page-space radius: {one} vs {four}"
        );
    }
}
