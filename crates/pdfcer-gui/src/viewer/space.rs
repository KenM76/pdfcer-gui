//! # `viewer::space` — the coordinate spaces a canvas gesture crosses
//!
//! Two bridges, four functions, and one rule: **every conversion between the
//! screen, the page raster and an authoring API goes through here.** A second
//! formula anywhere else is a second thing to keep in sync with the renderer.
//!
//! ## The three spaces, named because they are easy to conflate
//!
//! - **Screen space** — egui points in the window, what a pointer event
//!   carries.
//! - **Canvas space** — page-device points at zoom 1.0: Y-**down**, origin
//!   top-left, `/Rotate` already resolved into a possibly-swapped
//!   width/height. This is the space [`super::page_extent_pts`] measures and
//!   the space the on-screen raster is drawn in. [`screen_to_page`] and
//!   [`page_to_screen`] cross between screen and canvas space, and they carry
//!   **no rotation logic** — rotation is already baked into the `extent` they
//!   are handed.
//! - **PDF user space** — Y-**up**, origin at the *un-rotated*
//!   MediaBox/CropBox lower-left: exactly what an annotation `/Rect`, a
//!   content-stream operand or the object model expresses.
//!   [`canvas_to_pdf_space`] and [`pdf_space_to_canvas`] cross between canvas
//!   and user space.
//!
//! ## ★★ Why the second bridge inverts the renderer instead of deriving
//!
//! The canvas⟷user pair reuses — and inverts — the **same** device transform
//! `pdfcer_render::page_device_geometry` computes to rasterize the page, so
//! the interaction geometry and the render agree by construction. Two
//! hand-derived rotation formulas would agree on the day they were written
//! and drift the first time either side learned about a new `/Rotate` case.
//!
//! ## The failure contract, and why it differs between the two bridges
//!
//! The screen⟷canvas pair answers [`egui::Pos2::ZERO`] for a degenerate page
//! or zoom, matching `fit_scale` and `clamp_zoom`: there is no sensible canvas
//! coordinate for a page with no area, and a painter asking for one every
//! frame must not get a NaN. The canvas⟷user pair answers `None`, because its
//! callers are **authoring** — a commit declined is right where a commit of
//! garbage geometry is not.

use egui::{Pos2, Rect};
use pdfcer_core::page_tree::Page;
use pdfcer_render::tiny_skia::{Point, Transform};

/// Map a screen point to **canvas space**.
///
/// `image_rect` is the canvas Response's own `.rect` for this frame
/// (the rect the page raster occupies on screen); `extent` is
/// [`super::page_extent_pts`] for the current page (the rotated device
/// width/height); `zoom` is [`super::ViewState::zoom`]. The page raster is drawn
/// at `image_rect.min` scaled by `zoom`, so undoing that — subtract the
/// origin, divide by the zoom — is the whole of the arithmetic.
///
/// **No rotation branch lives here on purpose.** Rotation-correctness comes
/// entirely from `extent` already carrying the rotated width/height (see
/// [`super::page_extent_pts`]); adding a rotation-aware branch here as well would
/// double-apply it. The `extent` argument is consulted only to reject a
/// degenerate page (per the contract below) — the mapping itself is a pure
/// affine undo of the draw.
///
/// Returns [`Pos2::ZERO`] for a degenerate page or zoom (zero/negative/
/// non-finite `extent` or `zoom`), mirroring [`super::fit_scale`]/[`super::clamp_zoom`]'s
/// "fail to a finite, harmless value, never a NaN/panic" discipline: there
/// is no sensible canvas coordinate for a page with no area.
#[must_use]
pub fn screen_to_page(pos: Pos2, image_rect: Rect, extent: (f32, f32), zoom: f32) -> Pos2 {
    if !geometry_inputs_ok(extent, zoom) {
        return Pos2::ZERO;
    }
    Pos2::new(
        (pos.x - image_rect.min.x) / zoom,
        (pos.y - image_rect.min.y) / zoom,
    )
}

/// The exact inverse of [`screen_to_page`]: **canvas space** → screen.
///
/// Needed every frame by any live-preview overlay (a stored canvas-space
/// geometry must be projected back to the screen to be drawn) and, from
/// stage S4, to draw a hit-tested object's selection outline. Same
/// degenerate-input contract as [`screen_to_page`].
#[must_use]
#[allow(
    dead_code,
    reason = "the inverse half of a bridge whose forward half IS live (screen_to_page, used by the canvas pointer trace); its first drawing consumer is S4's selection outline. Kept because a bridge with one direction implemented is exactly how the two ends drift apart." // ui-text-exempt: clippy lint justification, never displayed
)]
pub fn page_to_screen(page_pt: Pos2, image_rect: Rect, extent: (f32, f32), zoom: f32) -> Pos2 {
    if !geometry_inputs_ok(extent, zoom) {
        return Pos2::ZERO;
    }
    Pos2::new(
        page_pt.x * zoom + image_rect.min.x,
        page_pt.y * zoom + image_rect.min.y,
    )
}

/// Whether the geometry inputs describe a real, finite page at a real
/// zoom — the shared degenerate-input guard for the screen⟷canvas bridge.
#[must_use]
fn geometry_inputs_ok(extent: (f32, f32), zoom: f32) -> bool {
    zoom.is_finite() && zoom > 0.0 && extent.0.is_finite() && extent.0 > 0.0 && extent.1 > 0.0
}

/// Convert a **canvas-space** point into genuine **PDF user space** — the
/// frame every `pdfcer-core` authoring API consumes.
///
/// Implemented by inverting the SAME transform
/// [`pdfcer_render::page_device_geometry`] computes to rasterize this page
/// at scale 1.0 (its third tuple element, a
/// [`pdfcer_render::tiny_skia::Transform`]). Canvas space *is* that
/// transform's output space at scale 1.0, so its inverse is exactly the
/// canvas→user map, rotation and Y-flip included, with no second formula to
/// keep in sync (the geometry analogue of "reuse the renderer's own walk so
/// they agree by construction").
///
/// Returns `None` only for a genuinely non-invertible page transform (a
/// degenerate page). Callers decline the commit rather than author garbage
/// geometry.
#[must_use]
pub fn canvas_to_pdf_space(point: Pos2, page: &Page) -> Option<Pos2> {
    let (_, _, ctm) = pdfcer_render::page_device_geometry(page, 1.0);
    let inverse = ctm.invert()?;
    Some(apply_transform(&inverse, point))
}

/// The exact inverse of [`canvas_to_pdf_space`]: **PDF user space** →
/// **canvas space**.
///
/// Needed by any consumer that receives geometry already in PDF space — the
/// primary case being the object-model provider handing back a hit-tested
/// object's bounds in PDF space, which the selection overlay must project to
/// the screen via `page_to_screen(pdf_space_to_canvas(bounds, page), ..)`.
/// Returns `None` under the same non-invertible-page condition as
/// [`canvas_to_pdf_space`], so the two bridges decline together.
#[must_use]
#[allow(
    dead_code,
    reason = "built and tested at S0; first live consumer is S4's selection-outline projection" // ui-text-exempt: clippy lint justification, never displayed
)]
pub fn pdf_space_to_canvas(point: Pos2, page: &Page) -> Option<Pos2> {
    let (_, _, ctm) = pdfcer_render::page_device_geometry(page, 1.0);
    // Guard on invertibility so the two directions accept/decline the same
    // pages; the forward map itself does not need the inverse, but a page
    // whose transform cannot round-trip has no well-defined canvas point.
    ctm.invert()?;
    Some(apply_transform(&ctm, point))
}

/// Apply a `tiny_skia` [`Transform`] to a single egui [`Pos2`].
///
/// One place the `Pos2` ⟷ `tiny_skia::Point` marshalling lives, so the two
/// bridge directions cannot marshal inconsistently.
#[must_use]
fn apply_transform(transform: &Transform, point: Pos2) -> Pos2 {
    let mut mapped = [Point::from_xy(point.x, point.y)];
    transform.map_points(&mut mapped);
    Pos2::new(mapped[0].x, mapped[0].y)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The round-trips are driven at the ends of the range the UI actually
    // offers rather than at invented scales, so a change to either bound
    // re-tests the bridges at the new one.
    use super::super::{MAX_ZOOM, MIN_ZOOM, page_extent_pts};
    use pdfcer_core::object::{Dict, ObjId};
    use pdfcer_core::page_tree::Rect as PageRect;

    /// A minimal page fixture: a `w`×`h` MediaBox/CropBox at the origin
    /// with the given clockwise `/Rotate`. Enough for the geometry
    /// functions, which read only `crop_box` and `rotate`.
    fn test_page(w: f64, h: f64, rotate: u16) -> Page {
        Page {
            id: ObjId::new(1, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, w, h),
            crop_box: PageRect::from_corners(0.0, 0.0, w, h),
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
            resources_defaulted: false,
            contents_flattened: 0,
        }
    }

    /// Two `Pos2` are equal within a few `f32` ULPs of accumulated error.
    fn near(a: Pos2, b: Pos2) -> bool {
        (a.x - b.x).abs() <= 1e-3 && (a.y - b.y).abs() <= 1e-3
    }

    #[test]
    fn screen_page_round_trips_at_every_rotation() {
        // Property 1: page_to_screen ∘ screen_to_page == identity, for the
        // extent `page_extent_pts` actually returns at each of the four
        // legal rotations. The four angles test that NOTHING
        // rotation-specific leaks into these functions — they are agnostic
        // to rotation, because `extent` already carries it.
        for &rotate in &[0u16, 90, 180, 270] {
            let page = test_page(200.0, 300.0, rotate);
            let extent = page_extent_pts(&page);
            for &zoom in &[MIN_ZOOM, 0.5, 1.0, 2.5, MAX_ZOOM] {
                let display = egui::vec2(extent.0 * zoom, extent.1 * zoom);
                let rect = Rect::from_min_size(Pos2::new(37.0, 11.0), display);
                for &p in &[
                    Pos2::new(37.0, 11.0),
                    Pos2::new(100.0, 250.0),
                    rect.center(),
                    rect.max,
                ] {
                    let round =
                        page_to_screen(screen_to_page(p, rect, extent, zoom), rect, extent, zoom);
                    // Round-trip within a few ULPs at rotate={0,90,180,270},
                    // zoom across the ladder extremes, for several points.
                    assert!(near(round, p));
                }
            }
        }
    }

    #[test]
    fn screen_to_page_distance_scales_as_one_over_zoom() {
        // Property 2: a fixed SCREEN distance maps to a page-space distance
        // of screen_distance / zoom — the invariance any screen-space snap
        // tolerance relies on.
        let extent = (200.0, 300.0);
        for &zoom in &[MIN_ZOOM, 0.5, 1.0, 3.0, MAX_ZOOM] {
            let rect = Rect::from_min_size(
                Pos2::new(5.0, 9.0),
                egui::vec2(extent.0 * zoom, extent.1 * zoom),
            );
            let a = screen_to_page(Pos2::new(50.0, 50.0), rect, extent, zoom);
            let b = screen_to_page(Pos2::new(90.0, 50.0), rect, extent, zoom);
            let page_dx = (b.x - a.x).abs();
            // A 40px screen span maps to a 40/zoom page span, for every zoom.
            assert!((page_dx - 40.0 / zoom).abs() <= 1e-3);
        }
    }

    #[test]
    fn screen_page_reject_degenerate_inputs_without_panicking() {
        // Property 4: zero/negative/non-finite geometry falls back to a
        // finite, harmless value rather than a NaN or a panic.
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 100.0));
        assert_eq!(
            screen_to_page(Pos2::new(5.0, 5.0), rect, (0.0, 100.0), 1.0),
            Pos2::ZERO
        );
        assert_eq!(
            screen_to_page(Pos2::new(5.0, 5.0), rect, (100.0, 100.0), 0.0),
            Pos2::ZERO
        );
        assert_eq!(
            page_to_screen(Pos2::new(5.0, 5.0), rect, (100.0, -1.0), 1.0),
            Pos2::ZERO
        );
        assert_eq!(
            page_to_screen(Pos2::new(5.0, 5.0), rect, (100.0, 100.0), f32::NAN),
            Pos2::ZERO
        );
    }

    #[test]
    fn canvas_pdf_bridge_round_trips_at_every_rotation() {
        // pdf_space_to_canvas ∘ canvas_to_pdf_space is the identity at each
        // rotation.
        for &rotate in &[0u16, 90, 180, 270] {
            let page = test_page(200.0, 300.0, rotate);
            for &p in &[
                Pos2::new(0.0, 0.0),
                Pos2::new(50.0, 80.0),
                Pos2::new(120.0, 240.0),
            ] {
                let user = canvas_to_pdf_space(p, &page).unwrap();
                let back = pdf_space_to_canvas(user, &page).unwrap();
                assert!(near(back, p), "rotate={rotate} p={p:?} back={back:?}"); // ui-text-exempt: test failure message, never displayed
            }
        }
    }

    #[test]
    fn pdf_space_to_canvas_agrees_with_the_renderer_by_construction() {
        // The forward map must equal `page_device_geometry`'s own
        // (already pixel-tested) transform — this is what proves "agrees
        // with the renderer by construction", not merely self-consistent.
        for &rotate in &[0u16, 90, 180, 270] {
            let page = test_page(200.0, 300.0, rotate);
            let (_, _, ctm) = pdfcer_render::page_device_geometry(&page, 1.0);
            for &p in &[
                Pos2::new(0.0, 0.0),
                Pos2::new(200.0, 0.0),
                Pos2::new(0.0, 300.0),
            ] {
                let via_bridge = pdf_space_to_canvas(p, &page).unwrap();
                let via_render = apply_transform(&ctm, p);
                assert!(near(via_bridge, via_render), "rotate={rotate} p={p:?}"); // ui-text-exempt: test failure message, never displayed
            }
        }
    }

    #[test]
    fn pdf_space_bridge_places_the_lower_left_corner_at_the_bottom() {
        // A concrete orientation check, un-rotated: PDF user-space (Y-up)
        // origin (0,0) is the page's lower-left, which in canvas space
        // (Y-down) is the BOTTOM-left — i.e. y == page height.
        let page = test_page(200.0, 300.0, 0);
        let ll = pdf_space_to_canvas(Pos2::new(0.0, 0.0), &page).unwrap();
        assert!(near(ll, Pos2::new(0.0, 300.0)));
        let ul = pdf_space_to_canvas(Pos2::new(0.0, 300.0), &page).unwrap();
        assert!(near(ul, Pos2::new(0.0, 0.0)));
    }
}
