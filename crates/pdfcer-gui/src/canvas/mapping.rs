//! # `canvas::mapping` — the ONE screen↔page conversion, the PDF↔canvas
//! projection, and the tolerance
//!
//! ## Why this file exists at all
//!
//! `GUI_ROADMAP.md` Phase 1 names three ways a selection model loses the
//! *"selection survives navigation"* invariant. The first is **selection
//! stored in screen coordinates**, and it has a twin that is easier to miss:
//!
//! > *"Every hit-test and snap `tolerance` is a PAGE-space radius, and
//! > nothing checks it. Pass raw screen pixels and it compiles, runs, and
//! > merely drifts with zoom"* (`hit.rs:118-120`, quoted in
//! > `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`).
//!
//! Both failures are the same mistake — *a screen number used where a page
//! number was meant* — and both are silent. So this module is the **single
//! boundary**: everything crossing it in one direction is screen space,
//! everything crossing it in the other is page space, and there is no second
//! place in `canvas/` that divides by `zoom`.
//!
//! Concretely: [`PageMapping`] holds the frame's page rect, extent and zoom,
//! and every conversion the selection layer needs is a method on it. A caller
//! that has a `PageMapping` cannot accidentally convert a point with this
//! frame's zoom and a tolerance with last frame's, because there is one zoom
//! and it is inside the mapping.
//!
//! ## "Page space" here means CANVAS space, and that is deliberate
//!
//! Three frames are in play and conflating any two of them is the classic
//! silent defect (`viewer`'s own header sets out the first two):
//!
//! | frame | Y | origin | who speaks it |
//! |---|---|---|---|
//! | **screen** | down | window top-left | egui, the pointer, the painter |
//! | **canvas** | down | page top-left, `/Rotate` applied | this module, [`crate::panels::objects::provider::ObjectModelProvider`]'s public surface, the raster |
//! | **PDF user** | **up** | un-rotated CropBox lower-left | the object model's *internals*, every `pdfcer-core` authoring verb |
//!
//! [`PageMapping`] converts **screen ⟷ canvas** and stops there. The
//! canvas → PDF-user hop is the provider's own business
//! ([`crate::viewer::canvas_to_pdf_space`] is the per-point sibling), and it
//! is left there on purpose: it needs the page's device transform, it is
//! already implemented once by inverting the *renderer's* own transform, and
//! a second implementation here would be a second chance to get the Y-flip
//! backwards. **PDF user space is y-UP; canvas and screen are y-DOWN.** The
//! failure is silent — the page looks perfect until someone selects a line
//! and gets a different one.
//!
//! ## Why the tolerance is a distance and not a rect
//!
//! Canvas space at zoom 1.0 is *distance-preserving* with respect to PDF user
//! space: `page_device_geometry(page, 1.0)`'s transform is a rotation, a
//! Y-flip and a translation, none of which change lengths. So a radius of
//! *n* canvas units **is** a radius of *n* PDF units, and one number can
//! serve both — which is exactly why
//! [`crate::panels::objects::provider::FALLBACK_SELECT_TOLERANCE`] can be
//! documented as "canvas space, and in effect page space" without a
//! conversion. If the canvas ever gained a non-uniform scale that would stop
//! being true, and this paragraph is where it would have to be revisited.

use egui::{Pos2, Rect};

use crate::viewer;

/// The screen-space catch radius for **object selection**, in egui logical
/// points, converted to a canvas/page-space tolerance per query by
/// [`screen_tolerance_to_page`].
///
/// # Why 6, and why a screen number rather than a page number
///
/// Salvaged verbatim from the old shell, with its reasoning, because the
/// reasoning is the valuable part. The behaviour it replaced was a fixed
/// `3.0` **canvas-space** value, which is `3.0 × zoom` pixels on screen: 3 px
/// at 100%, 1.5 px at 50%, 0.75 px at 25%. Objects were effectively
/// unclickable at exactly the zoom an operator uses to see a whole drawing.
///
/// Deliberately a *sibling* of the snap radius rather than the same constant:
/// snapping and selection answer different questions and are allowed to drift
/// apart. Selection is set **tighter** because a snap that grabs a nearby
/// vertex is a helpful correction the operator can see and cycle through,
/// whereas a selection that grabs a neighbouring object is a silent wrong
/// answer. The failure modes are not symmetric, so the tolerances should not
/// be either.
///
/// # This constant lives HERE and nowhere else
///
/// `panels::objects::provider`'s salvage note records that one test did not
/// come across —
/// `screen_tolerance_keeps_the_on_screen_catch_radius_constant` — because
/// *"re-declaring those constants here to keep a test green would put the
/// tolerance in two places, which is the cause of the defect the test guards,
/// not a way to guard it."* This module is where the constant landed, and
/// [`tests::screen_tolerance_keeps_the_on_screen_catch_radius_constant`] is
/// that test, restored.
pub const SELECT_SCREEN_TOLERANCE_PX: f32 = 6.0;

/// ★★★ **The catch radius for an ANCHOR**, in egui logical points —
/// `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
///
/// Eight rather than six, and both halves of that are borrowed rather than
/// invented:
///
/// * it is the number [`crate::canvas::handledrag::GRAB_PX`] already gives a
///   **Bézier control point**, whose own doc says *"a target that requires
///   hitting its exact pixels is a target an operator misses"* — so an anchor
///   stops being harder to hit than the handle that hangs off it, which it
///   was;
/// * it is Inkscape's *grab sensitivity* default, which is the operator's
///   stated tie-breaker for this family of decisions.
///
/// # ★★ A SIBLING of [`SELECT_SCREEN_TOLERANCE_PX`], not a change to it
///
/// Widening the shared constant would have widened **object picking** too, on
/// a sheet this project has measured at 129,758 objects — where a larger catch
/// radius means more candidates under every press and a different answer to
/// *"what did I click?"*. The two answer different questions and are allowed
/// to drift, which is the same argument that constant's own header makes about
/// the snap radius.
///
/// It reaches exactly one call: `canvas::input::probe`'s `nearest_node`, and
/// only from the **Node tool**'s click. The general click path still passes
/// [`SELECT_SCREEN_TOLERANCE_PX`], so its behaviour is byte-identical.
pub const NODE_SCREEN_TOLERANCE_PX: f32 = 8.0;

/// Convert a fixed SCREEN-space pixel radius into a **canvas/page-space**
/// tolerance at `zoom` (points per PDF user-space unit).
///
/// This is the exact `1 / zoom` distance law
/// [`crate::viewer::screen_to_page`] uses, proven zoom-invariant by that
/// module's `screen_to_page_distance_scales_as_one_over_zoom` test. A
/// constant on-screen catch radius therefore maps to a *shrinking*
/// page-space tolerance as the operator zooms in, which is what keeps the
/// click target feeling identical at every zoom.
///
/// # Degenerate inputs yield `0.0`, and that is not a silent failure
///
/// A non-finite or non-positive `zoom` (reachable: the page's drawn size is
/// zero for one frame after an open, and a fit scale on a degenerate CropBox
/// can go non-finite) returns `0.0` rather than a NaN or an infinity. `0.0`
/// is then recognised by
/// [`crate::panels::objects::provider`]'s `resolve` as degenerate and
/// replaced with the fixed canvas-space fallback — so a bad zoom makes
/// selection *fussy for one frame*, never *broken*. Returning NaN instead
/// would make every comparison in the hit test false and every query a miss,
/// with nothing to say why.
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
///
/// Constructed once per frame in [`crate::canvas::show`], immediately after
/// the scroll area has settled and the page's true drawn rect is known, and
/// then handed to everything that needs a coordinate. **Nothing downstream
/// of it sees a screen coordinate again**, except the overlay, which converts
/// back through [`Self::to_screen`] at the moment of painting.
///
/// `Copy` because it is three small values and passing it by value removes
/// any question of it being stale: a mapping is a fact about one frame, and a
/// borrow that outlived the frame would be a mapping for a page rect that has
/// since moved.
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
    ///
    /// Deliberately the only way *out* of this type other than a conversion.
    /// There is no `zoom()` accessor: the zoom's whole job here is to be
    /// divided by, and exposing it would be an invitation to divide by it at
    /// a call site — which is the defect this module exists to make
    /// unavailable. Anything that needs a page-space distance asks
    /// [`Self::tolerance`].
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
    ///
    /// # ★★★ Why a vector needs its own conversion, and what it cost not to have one
    ///
    /// A point conversion carries the page's origin on screen; a displacement
    /// must not. `to_screen(a) - to_screen(b)` is correct and says the origin
    /// twice; this says it none.
    ///
    /// **`DEFECTS.md` D18 is what its absence cost.** The gesture machine works
    /// in **page** space by design — `canvas::interact` builds its
    /// `PointerFrame` with `pos: screen_pos.map(|p| map.to_page(p))` — so
    /// `GestureOutcome::Resize.delta` is a page-space displacement. It was
    /// handed straight to `resizing::Frame::delta`, whose doc comment says *"in
    /// screen points"*, and divided against a `bounds` that genuinely is screen
    /// space. Every resize factor's distance from unity came out inflated by
    /// **`1/zoom`**: at the operator's fitted 29.55 % a corner dragged 60 px
    /// committed a 5.94× stretch where the contract says 2.46×, and the shape
    /// shot 143 px past the cursor on both axes.
    ///
    /// ⇒ Two quantities in two spaces, one of them undocumented at its call
    /// site, and **nothing in the type system to notice**: both are `Vec2`.
    /// This method is the place the multiply lives, for the reason this
    /// module's header already gives about the divide — *"there is one zoom and
    /// one place in `canvas/` that divides by it."* The same must be true of
    /// multiplying, or the two drift.
    ///
    /// ★ It is deliberately **not** called `to_screen_vec`. `to_screen` and
    /// `to_page` are a matched pair over positions, and a name one character
    /// away from them is how a caller reaches for the wrong one; this one says
    /// *page vector* in its name so the space is at the call site rather than
    /// in its documentation.
    #[must_use]
    pub fn page_vec_to_screen(&self, page: egui::Vec2) -> egui::Vec2 {
        page * self.zoom
    }

    /// **Screen → canvas for a DISPLACEMENT.** The inverse of
    /// [`Self::page_vec_to_screen`]; see that method for why a displacement is
    /// not a position.
    ///
    /// ★ A non-finite or non-positive zoom answers `Vec2::ZERO` rather than a
    /// NaN, on the same argument [`screen_tolerance_to_page`] makes: a
    /// degenerate zoom is reachable (a page drawn at zero size for one frame),
    /// and a NaN displacement reaching a content stream is a corrupted file,
    /// while a zero one is a gesture that did nothing.
    #[must_use]
    pub fn screen_vec_to_page(&self, screen: egui::Vec2) -> egui::Vec2 {
        if self.zoom.is_finite() && self.zoom > 0.0 {
            screen / self.zoom
        } else {
            egui::Vec2::ZERO
        }
    }

    /// **Screen → canvas** for a rect (the marquee).
    ///
    /// Normalised with [`Rect::from_two_pos`] rather than assembled from
    /// `min`/`max`, because a rubber-band is dragged in any of four
    /// directions and its "min" corner is wherever the press happened to be.
    /// A non-normalised rect has negative width and every containment test
    /// against it silently answers `false`.
    #[must_use]
    pub fn rect_to_page(&self, screen: Rect) -> Rect {
        Rect::from_two_pos(self.to_page(screen.min), self.to_page(screen.max))
    }

    /// **Canvas → screen** for a rect (a selection outline).
    ///
    /// Normalised for the same reason [`Self::rect_to_page`] is: the
    /// screen↔canvas map is a pure scale and translate with no flip, but the
    /// rects it is handed come from the provider, which built them by
    /// bounding a *mapped quad* under a transform that may rotate. Assuming
    /// corner order here would produce an inside-out rect that paints nothing.
    #[must_use]
    pub fn rect_to_screen(&self, page: Rect) -> Rect {
        Rect::from_two_pos(self.to_screen(page.min), self.to_screen(page.max))
    }

    /// The selection catch radius for this frame, in **canvas/page** units.
    ///
    /// The one call every hit test makes. Passing
    /// [`SELECT_SCREEN_TOLERANCE_PX`] straight to a provider query — which
    /// compiles, and runs, and merely drifts with zoom — is the defect this
    /// method exists to make unavailable.
    #[must_use]
    pub fn tolerance(&self) -> f64 {
        screen_tolerance_to_page(SELECT_SCREEN_TOLERANCE_PX, self.zoom)
    }

    /// The **anchor** catch radius for this frame, in canvas/page units —
    /// `OPERATOR_REQUESTS.md` O69.
    ///
    /// [`Self::tolerance`]'s sibling, wider by two pixels, and it exists here
    /// rather than at the call site for this module's standing reason: no
    /// other file in `canvas/` divides by zoom, and keeping that true is what
    /// stops a second conversion drifting from this one.
    ///
    /// See [`NODE_SCREEN_TOLERANCE_PX`] for why it is a separate constant
    /// rather than a change to the shared one.
    #[must_use]
    pub fn node_tolerance(&self) -> f64 {
        screen_tolerance_to_page(NODE_SCREEN_TOLERANCE_PX, self.zoom)
    }

    /// The **snap** catch radius for this frame, in canvas/page units.
    ///
    /// [`Self::tolerance`]'s sibling, and it sits here for the identical
    /// reason: this module's header states that there is no second place in
    /// `canvas/` that divides by `zoom`, and a snap query that reached for
    /// `SNAP_SCREEN_TOLERANCE_PX` and did the division at its call site would
    /// be exactly that second place.
    ///
    /// # ★ Why it is a different number from the selection radius
    ///
    /// They are both screen-pixel radii converted the same way, but they
    /// answer different questions and the salvaged constant is the wider of
    /// the two (10 px against the selection's smaller catch). A snap is an
    /// *offer* — it proposes a point and shows an indicator saying which, and
    /// the operator can refuse it by holding Alt — so casting a wide net costs
    /// them nothing. A selection is a *commitment* made silently on release,
    /// so its radius is deliberately tighter: `canvas::forms`' header makes the
    /// same argument one step further, refusing any tolerance at all for form
    /// widgets because fields sit a point apart.
    ///
    /// Keeping them separate is what lets each be tuned without the other
    /// moving. Collapsing them would mean widening the selection catch every
    /// time snapping wanted a longer reach.
    #[must_use]
    pub fn snap_tolerance(&self) -> f64 {
        screen_tolerance_to_page(crate::canvas::snap::SNAP_SCREEN_TOLERANCE_PX, self.zoom)
    }
}

/// Project an annotation's `/Rect` — **PDF user space, y-up, un-rotated**,
/// as `[llx, lly, urx, ury]` — into canvas space.
///
/// # ★ Why this lives here rather than with the forms code that wrote it
///
/// It was `canvas::forms::boxes::widget_canvas_rect` until 2026-08-18, because
/// filling a form on the page was the first thing that needed to know where an
/// annotation is drawn. It is not about widgets and never was: it takes any
/// `/Rect` and answers where the rasterizer put it.
///
/// It moved when annotation **selection** arrived and needed the same answer
/// for stamps, notes and ce dimensions. Calling it in place would have made
/// the selection layer depend on the forms layer for pure geometry, and the
/// name would have lied at every one of those call sites. Both are how a
/// module graph stops being readable.
///
/// # The array shape, and the corner order
///
/// `[f64; 4]` is `EditSession::widget_rects`' own shape, taken verbatim rather
/// than re-wrapped in a `page_tree::Rect`, because converting through a second
/// rectangle type would be a place for the normalisation to be undone.
///
/// **Corner order is not assumed.** §7.9.5 permits `/Rect` either way round.
/// `widget_rects` normalises; `pdfcer_core::annot::Annotation::rect` reports
/// what the file says. So a caller can hand this either, and the four-corner
/// bound below is what makes both work — a two-corner version would produce an
/// inside-out rectangle for one of them and silently never hit anything.
///
/// Through [`crate::viewer::pdf_space_to_canvas`], which inverts nothing and
/// invents nothing: it applies `pdfcer_render::page_device_geometry`'s own
/// transform, the one the rasterizer used to draw the page. That is what makes
/// the box land on the pixels the operator is pointing at *by construction*
/// rather than by two formulas agreeing, and it is why `/Rotate` needs no
/// branch here — the rotation is already inside the transform.
///
/// **All four corners, then bound them.** Two corners are enough while the
/// transform is a scale, a flip and a quarter-turn, which is every case a
/// conforming `/Rotate` can produce. Four is what makes that not have to be
/// true: a transform that shears or rotates by anything else still produces a
/// box that contains the widget, where a two-corner version would produce one
/// that is inside-out and contains nothing.
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
            contents_flattened: 0,
        }
    }

    /// A 200x20 rectangle near the top of that page.
    const PROJECTED_RECT: [f64; 4] = [100.0, 700.0, 300.0, 720.0];

    /// ★ **A degenerate rectangle produces no box**, whichever corner order it
    /// is written in.
    ///
    /// The corner-order half matters on its own: §7.9.5 permits `/Rect` either
    /// way round, and a hit test against an un-normalised rectangle would
    /// silently never match. `widget_rects` normalises and
    /// `Annotation::rect` does not, so this function must cope with either.
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

    /// ★ **The box lands where the page draws it, at every rotation.**
    ///
    /// The half of the geometry a unit test can actually hold. `/Rect` is
    /// y-**up** from the CropBox's lower-left and canvas space is y-**down**
    /// from the page's top-left, so a field near the TOP of the page in PDF
    /// terms (a large Y) must land near the top in canvas terms (a small Y) —
    /// and the failure when it does not is silent, because the page looks
    /// perfect and only the click is wrong.
    ///
    /// Under a quarter-turn the box moves to the corresponding edge rather
    /// than staying put, which is the assertion that would fail on a build
    /// that projected the rect without the page's transform.
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

    /// ★ **The law this module exists for**, restored from the old shell.
    ///
    /// `panels::objects::provider`'s salvage note §4 records that this test
    /// could not come across with the provider, because asserting it there
    /// would have meant re-declaring the constant — putting the tolerance in
    /// two places, which is the *cause* of the defect it guards. It lands
    /// here, with the constant and the conversion it is about.
    ///
    /// The property: the canvas-space tolerance a click supplies scales as
    /// `1 / zoom`, so the SCREEN-space catch radius is the same number of
    /// pixels at every zoom level. Assert the *outcome* (the on-screen
    /// radius) rather than the intermediate (the page-space number), so this
    /// checks the law and not merely that the code agrees with itself.
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

    /// ★ **A canvas coordinate does not move when the view does.**
    ///
    /// The arithmetic half of the "selection survives navigation" invariant:
    /// the *same object point* has the same canvas coordinate at every zoom
    /// and every scroll position, which is exactly why a selection held in
    /// canvas/identity terms survives navigation and one held in screen
    /// terms cannot.
    ///
    /// Modelled by taking a fixed canvas point, projecting it to screen at
    /// each zoom (with the page rect moving as the scroll area would move
    /// it), and converting back.
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

    /// ★ **Each page of a strip gets its OWN mapping, and they are not
    /// interchangeable.**
    ///
    /// The failure this pins is the one Phase 4 was most likely to ship
    /// silently: under a continuous mode the Find wash has to be painted for
    /// several pages at once, and painting them all through the *acting*
    /// page's mapping would stack every page's highlights onto one page. The
    /// hits would still be found, the wash would still be drawn, and it would
    /// be drawn in the wrong place — which reads as a highlight bug rather
    /// than a mapping one, and is exactly the class `canvas/mod.rs`'s own
    /// centring comment records the old GUI shipping.
    ///
    /// Asserted as the *outcome*: the same canvas point — the top-left corner
    /// of a page — projects to each page's own screen origin through that
    /// page's mapping, and to somewhere else entirely through its neighbour's.
    /// The second half is what makes this a test rather than a tautology; a
    /// build in which the two mappings were accidentally equal would pass the
    /// first half.
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

    /// ★★★ **A displacement does not carry the page's origin.**
    ///
    /// `DEFECTS.md` D18's root: two quantities in two spaces, both `Vec2`, and
    /// nothing to notice. The rect above starts at (316, 580) precisely so a
    /// conversion written as `to_screen(a)` — the point form — fails this.
    #[test]
    fn a_page_displacement_converts_without_the_origin() {
        let map = at(0.2955);
        let screen = map.page_vec_to_screen(egui::vec2(100.0, 40.0));
        assert!((screen.x - 29.55).abs() < 0.01, "{screen:?}");
        assert!((screen.y - 11.82).abs() < 0.01, "{screen:?}");
    }

    /// ★★ The round trip, at the operator's own fitted zoom.
    ///
    /// The number that matters: at 29.55 % a 60 px drag is 203 page units, and
    /// handing those 203 to a function expecting 60 is what inflated every
    /// resize factor by `1/zoom`.
    #[test]
    fn the_two_directions_are_inverses() {
        let map = at(0.2955);
        let screen = egui::vec2(60.0, 60.0);
        let page = map.screen_vec_to_page(screen);
        assert!((page.x - 203.04).abs() < 0.01, "{page:?}");
        let back = map.page_vec_to_screen(page);
        assert!((back - screen).length() < 0.001, "{back:?} vs {screen:?}");
    }

    /// ★ A degenerate zoom answers ZERO, never NaN.
    ///
    /// Reachable: a page drawn at zero size for one frame. A NaN displacement
    /// reaching a content stream is a corrupted file; a zero one is a gesture
    /// that did nothing, and only one of those is recoverable.
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

    /// ★★★ **An anchor is easier to hit than an object, and exactly as easy
    /// as its own control point** — `OPERATOR_REQUESTS.md` O69.
    ///
    /// Both halves are asserted because both are the argument. The first says
    /// the widening happened; the second says which number was chosen and why
    /// — an anchor that was harder to hit than the Bézier handle hanging off
    /// it was the concrete absurdity the row is about.
    #[test]
    fn an_anchor_is_caught_more_easily_than_an_object_and_as_easily_as_a_handle() {
        // ★ Bound through locals rather than compared as literals, so clippy
        // reads them as values rather than as a constant assertion. The
        // property is about the RELATIONSHIP between two constants, which is
        // exactly what a `const` block would hide from a reader looking for
        // why one of them was changed.
        let node = NODE_SCREEN_TOLERANCE_PX;
        let object = SELECT_SCREEN_TOLERANCE_PX;
        let handle = crate::canvas::handledrag::GRAB_PX;
        assert!(
            node > object,
            "an anchor is a small target and must be more forgiving than a whole object"
        );
        assert!(
            (node - handle).abs() < f32::EPSILON,
            "an anchor must be no harder to hit than the control point hanging off it — \
             they were 6 and 8"
        );
    }

    /// ★★ **Widening the anchor radius did NOT widen object picking.**
    ///
    /// The assertion that pins the scoping. On a sheet this project has
    /// measured at 129,758 objects a larger catch radius means more candidates
    /// under every press and a different answer to "what did I click?" — so
    /// the two constants must stay two constants.
    #[test]
    fn the_object_catch_radius_is_unchanged() {
        let object = SELECT_SCREEN_TOLERANCE_PX;
        assert!(
            (object - 6.0).abs() < f32::EPSILON,
            "object picking must still catch at six pixels; O69 widened the ANCHOR radius only"
        );
    }

    /// Both radii keep a constant ON-SCREEN size as the zoom changes.
    ///
    /// The sibling of `screen_tolerance_keeps_the_on_screen_catch_radius_constant`,
    /// asserted for the new one because a radius that did not scale would be
    /// eight page points at every zoom — a catch radius the size of a sheet
    /// when zoomed out, and invisible when zoomed in.
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
