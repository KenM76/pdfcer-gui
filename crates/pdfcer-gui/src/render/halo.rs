//! # `render::halo` — rasterizing the ground OUTSIDE the sheet
//!
//! ## The report, and which half of it this is
//!
//!
//! > *"also objects should still be reachable even if they are off the page."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/halo.md`.

use pdfcer_core::page_tree::Rect;

use super::region::PageFrame;

/// How far content may hang outside the crop box before it is treated as
/// deliberately off-page, in **pdf** points.
pub const OVERHANG_TOLERANCE_PTS: f64 = 1.0;

/// **The box to rasterize so that every object on this page is painted**, or
/// [`None`] if this page does not need one.
#[must_use]
pub fn region(crop: Rect, content: Option<Rect>, raster_scale: f32) -> Option<Rect> {
    let content = content?;
    if !finite(crop) || !finite(content) {
        return None;
    }
    // A degenerate crop box cannot be reasoned about and the whole-page path
    // already refuses it safely — answering here would send a nonsense
    // rectangle to the renderer, which is `strategy::for_page`'s rule too.
    if crop.urx <= crop.llx || crop.ury <= crop.lly {
        return None;
    }
    // `Bounds::EMPTY` is `min = +∞, max = −∞`, so an un-drawn page arrives as
    // an inverted box rather than a zero-sized one. Caught by `finite` above;
    // this catches the degenerate-but-finite case a single point would produce.
    if content.urx < content.llx || content.ury < content.lly {
        return None;
    }

    let union = Rect::from_corners(
        crop.llx.min(content.llx),
        crop.lly.min(content.lly),
        crop.urx.max(content.urx),
        crop.ury.max(content.ury),
    );
    let overhang = (crop.llx - union.llx)
        .max(crop.lly - union.lly)
        .max(union.urx - crop.urx)
        .max(union.ury - crop.ury);
    if overhang <= OVERHANG_TOLERANCE_PTS {
        return None;
    }

    // The same ceiling `strategy::for_page` applies to the whole-page tier,
    // asked of the bigger box. Rotation is irrelevant to it — a quarter turn
    // swaps the two edges and does not change which is longest — which is why
    // this function needs no `PageFrame` and the one below does.
    //
    // Through `strategy::region_raster_fits` and not spelled here. The
    // order that reaches the worker is guarded by that same predicate, one
    // frame later and at a scale this call could not see — and a halo box
    // validated here at one scale was rasterized at the next one up, refused,
    // and turned into the document's zoom ceiling. Two spellings of one wall
    // is how that happened; one spelling is why it cannot happen again.
    //
    // ⚠ The scale guard stays HERE and is not delegated, because the two
    // functions answer bad input in opposite directions on purpose:
    // `region_raster_fits` answers `true` so that a caller never withholds an
    // order over a number it could not reason about, while this function
    // answers `None` so that a nonsense rectangle is never sent to the
    // renderer. Both are right for their own caller; only spelling the second
    // one keeps it that way. The union's own edges need no such guard — it
    // contains the crop box, which was checked finite and non-degenerate
    // above.
    if !raster_scale.is_finite() || raster_scale <= 0.0 {
        return None;
    }
    if !super::strategy::region_raster_fits(union, raster_scale) {
        return None;
    }
    Some(union)
}

/// Whether every corner of `r` is a finite number.
#[must_use]
fn finite(r: Rect) -> bool {
    r.llx.is_finite() && r.lly.is_finite() && r.urx.is_finite() && r.ury.is_finite()
}

/// **How far the canvas may look past the sheet**, in screen coordinates.
#[must_use]
pub fn reach(
    place: egui::Rect,
    extent: (f32, f32),
    frame: PageFrame,
    content: Option<Rect>,
) -> egui::Rect {
    let Some(content) = content else {
        return place;
    };
    if !finite(content) || content.urx < content.llx || content.ury < content.lly {
        return place;
    }
    let (ex, ey) = (f64::from(extent.0), f64::from(extent.1));
    if !(ex > 0.0 && ey > 0.0 && place.width() > 0.0 && place.height() > 0.0) {
        return place;
    }
    // Screen pixels per canvas point, both axes. From the placement, for the
    // reason named in the `extent` parameter's docs.
    let sx = f64::from(place.width()) / ex;
    let sy = f64::from(place.height()) / ey;

    // Into CANVAS space — y-down from the page's top-left, `/Rotate`
    // resolved. `canvas_box_of` takes the bounding box of the two mapped
    // corners rather than mapping them corner-for-corner, because 90° and 270°
    // swap the axes; that is O174, and doing it by hand here would be the
    // second place it could be got wrong.
    let (cx0, cy0, cx1, cy1) = frame.canvas_box_of(content);

    // The union with the sheet itself, so a page whose content is entirely
    // inside it comes back as `place` to the pixel.
    let left = f64::from(place.min.x) + cx0.min(0.0) * sx;
    let top = f64::from(place.min.y) + cy0.min(0.0) * sy;
    let right = f64::from(place.min.x) + cx1.max(ex) * sx;
    let bottom = f64::from(place.min.y) + cy1.max(ey) * sy;
    if !(left.is_finite() && top.is_finite() && right.is_finite() && bottom.is_finite()) {
        return place;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "an egui::Rect is f32; this is the same boundary region_on_screen documents" // ui-text-exempt: clippy lint justification, never displayed
    )]
    egui::Rect::from_min_max(
        egui::pos2(left as f32, top as f32),
        egui::pos2(right as f32, bottom as f32),
    )
}

/// **How far the drawn content reaches past the sheet, per axis, in canvas
/// points** — the number O23's *scroll* half (part A) was missing.
#[must_use]
pub fn overhang(extent: (f32, f32), frame: PageFrame, content: Option<Rect>) -> (f32, f32) {
    const NONE: (f32, f32) = (0.0, 0.0);
    let Some(content) = content else {
        return NONE;
    };
    if !finite(content) || content.urx < content.llx || content.ury < content.lly {
        return NONE;
    }
    let (ex, ey) = (f64::from(extent.0), f64::from(extent.1));
    if !(ex > 0.0 && ey > 0.0) {
        return NONE;
    }
    let (cx0, cy0, cx1, cy1) = frame.canvas_box_of(content);
    // The larger of the two sides on each axis. Symmetric on purpose: the
    // pasteboard this feeds is the same width on both sides of the strip, so
    // a drawing that hangs off only the left still gets the room on the right.
    // That room is blank paper the operator already had a viewport of, and
    // making it asymmetric would mean two different strip margins per axis —
    // a second offset space, which is the shape of the defect O23 spent three
    // attempts on.
    let ox = (-cx0).max(cx1 - ex).max(0.0);
    let oy = (-cy0).max(cy1 - ey).max(0.0);
    if !(ox.is_finite() && oy.is_finite()) {
        return NONE;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "canvas points are f32 everywhere in the canvas; this is the same boundary `reach` documents" // ui-text-exempt: clippy lint justification, never displayed
    )]
    (ox as f32, oy as f32)
}

#[cfg(test)]
mod tests {
    use super::{OVERHANG_TOLERANCE_PTS, overhang, reach, region};
    use crate::render::region::PageFrame;
    use pdfcer_core::page_tree::Rect;

    /// A 200 × 200 sheet at the origin — the shape of
    /// `tools/ui-verify/fixtures/off-page-object.pdf`, so the numbers in these
    /// tests and in the driven check are the same numbers.
    const CROP: Rect = Rect {
        llx: 0.0,
        lly: 0.0,
        urx: 200.0,
        ury: 200.0,
    };

    fn r(llx: f64, lly: f64, urx: f64, ury: f64) -> Rect {
        Rect::from_corners(llx, lly, urx, ury)
    }

    /// The ordinary page: everything drawn is on the sheet, so there is no halo
    /// tier and nothing about today's rendering changes.
    #[test]
    fn a_page_whose_content_is_on_the_sheet_has_no_halo() {
        assert_eq!(region(CROP, Some(r(10.0, 10.0, 190.0, 190.0)), 1.0), None);
    }

    /// A border stroke's half-width is not a feature request. See
    /// [`OVERHANG_TOLERANCE_PTS`].
    #[test]
    fn a_hairline_overhang_is_not_a_halo() {
        let hair = OVERHANG_TOLERANCE_PTS / 2.0;
        assert_eq!(
            region(CROP, Some(r(-hair, -hair, 200.0 + hair, 200.0 + hair)), 1.0),
            None
        );
    }

    /// The feature. An object off the left edge grows the box to the left
    /// and leaves the other three sides alone — a halo that grew symmetrically
    /// would cost four times the pixels to show the same object.
    #[test]
    fn an_object_off_the_left_edge_grows_the_box_leftwards_only() {
        let halo = region(CROP, Some(r(-160.0, 100.0, 100.0, 140.0)), 1.0)
            .expect("content 160 pt off the sheet is a halo");
        assert!((halo.llx - -160.0).abs() < 1e-9, "left edge: {halo:?}");
        assert!((halo.lly - 0.0).abs() < 1e-9, "bottom edge: {halo:?}");
        assert!((halo.urx - 200.0).abs() < 1e-9, "right edge: {halo:?}");
        assert!((halo.ury - 200.0).abs() < 1e-9, "top edge: {halo:?}");
    }

    /// A page nobody has decomposed yet is not a page with no off-page content.
    /// Case 1 of [`region`]'s three — the caller must not read it as a promise.
    #[test]
    fn an_unmeasured_page_has_no_halo() {
        assert_eq!(region(CROP, None, 1.0), None);
    }

    /// `Bounds::EMPTY` is `min = +∞, max = −∞`. A page that drew nothing must
    /// not produce an infinite raster request.
    #[test]
    fn an_empty_content_box_has_no_halo() {
        let empty = Rect {
            llx: f64::INFINITY,
            lly: f64::INFINITY,
            urx: f64::NEG_INFINITY,
            ury: f64::NEG_INFINITY,
        };
        assert_eq!(region(CROP, Some(empty), 1.0), None);
    }

    /// The ceiling. The same object at a zoom whose raster would not fit
    /// gives up the halo tier rather than asking for a pixmap the engine
    /// refuses — and `canvas::present` then falls through to the
    /// visible-region tier, which is what [`reach`] is for.
    #[test]
    fn a_halo_that_would_not_fit_is_declined() {
        let content = Some(r(-5000.0, 0.0, 200.0, 200.0));
        // 5,200 pt wide. At 1× that is 5,200 px and fits.
        assert!(region(CROP, content, 1.0).is_some());
        // At 8× it is 41,600 px against a 16,383 ceiling.
        assert_eq!(region(CROP, content, 8.0), None);
    }

    /// A degenerate scale or crop box is never answered, for the reason
    /// `strategy::for_page` gives: a nonsense rectangle must not reach the
    /// renderer.
    #[test]
    fn nonsense_input_is_declined() {
        let content = Some(r(-500.0, 0.0, 200.0, 200.0));
        assert_eq!(region(CROP, content, 0.0), None);
        assert_eq!(region(CROP, content, f32::NAN), None);
        assert_eq!(region(r(0.0, 0.0, 0.0, 0.0), content, 1.0), None);
    }

    /// [`reach`] leaves an ordinary page exactly where it was — the property
    /// that lets the call site have no branch.
    #[test]
    fn reach_returns_the_sheet_when_nothing_hangs_over_it() {
        let place = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(400.0, 400.0));
        let frame = PageFrame::new(CROP, 0);
        assert_eq!(
            reach(
                place,
                (200.0, 200.0),
                frame,
                Some(r(10.0, 10.0, 190.0, 190.0))
            ),
            place
        );
        assert_eq!(reach(place, (200.0, 200.0), frame, None), place);
    }

    /// An object off the LEFT of an upright page reaches LEFT on screen.
    ///
    /// 400 screen px for 200 pt is 2 px/pt, so 160 pt of overhang is 320 px.
    #[test]
    fn reach_grows_left_for_content_off_the_left_edge() {
        let place = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(400.0, 400.0));
        let out = reach(
            place,
            (200.0, 200.0),
            PageFrame::new(CROP, 0),
            Some(r(-160.0, 100.0, 100.0, 140.0)),
        );
        assert!((out.min.x - (100.0 - 320.0)).abs() < 0.01, "{out:?}");
        assert_eq!(out.min.y, place.min.y);
        assert_eq!(out.max.x, place.max.x);
        assert_eq!(out.max.y, place.max.y);
    }

    /// O174's case, which a y-only conversion cannot produce: on a
    /// `/Rotate 90` page the PDF's −x becomes the canvas's −y, so the same
    /// object reaches **up** the screen rather than left.
    #[test]
    fn reach_follows_the_rotation() {
        let place = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(400.0, 400.0));
        let out = reach(
            place,
            (200.0, 200.0),
            PageFrame::new(CROP, 90),
            Some(r(-160.0, 100.0, 100.0, 140.0)),
        );
        assert_eq!(out.min.x, place.min.x, "x must not move: {out:?}");
        assert!((out.min.y - (50.0 - 320.0)).abs() < 0.01, "{out:?}");
    }

    // ---- `overhang` --------------------------------------------------
    //
    // The same four questions `reach` is asked, one step earlier and in
    // canvas POINTS rather than screen pixels, because the pasteboard has to
    // be sized before the page has a placement. See `overhang`'s own header
    // for why `reach` cannot answer them at that moment.

    /// The ordinary page: nothing hangs over, so the pasteboard term is zero
    /// and `canvas::geometry` produces byte-for-byte what it did before O23's
    /// second half existed.
    #[test]
    fn a_page_whose_content_is_on_the_sheet_has_no_overhang() {
        assert_eq!(
            overhang(
                (200.0, 200.0),
                PageFrame::new(CROP, 0),
                Some(r(10.0, 10.0, 190.0, 190.0))
            ),
            (0.0, 0.0)
        );
    }

    /// An unmeasured page is not a page with nothing off it — the same
    /// distinction `region` draws in its case 1. `content_bounds_if_known`
    /// PEEKS, so `None` is the answer on every page the operator has not
    /// decomposed, which is most of them most of the time.
    #[test]
    fn an_unmeasured_page_has_no_overhang() {
        assert_eq!(
            overhang((200.0, 200.0), PageFrame::new(CROP, 0), None),
            (0.0, 0.0)
        );
    }

    /// The feature, in points. An object 160 pt off the left edge of a
    /// 200 pt sheet gives 160 pt of x overhang and no y overhang, so the
    /// pasteboard grows on the axis the object is actually on.
    #[test]
    fn an_object_off_the_left_edge_overhangs_on_x_only() {
        let (ox, oy) = overhang(
            (200.0, 200.0),
            PageFrame::new(CROP, 0),
            Some(r(-160.0, 100.0, 100.0, 140.0)),
        );
        assert!((ox - 160.0).abs() < 0.01, "x overhang {ox}");
        assert!(oy.abs() < 0.01, "y overhang {oy}");
    }

    /// O174's case again, and the reason this goes through
    /// `PageFrame::canvas_box_of` rather than subtracting the crop box
    /// directly: on a `/Rotate 90` page the PDF's −x is the canvas's −y, so
    /// the SAME object overhangs the other axis. A hand-rolled
    /// `crop.llx - content.llx` would have grown the pasteboard sideways on a
    /// landscape sheet and left the object as unreachable as before.
    #[test]
    fn overhang_follows_the_rotation() {
        let (ox, oy) = overhang(
            (200.0, 200.0),
            PageFrame::new(CROP, 90),
            Some(r(-160.0, 100.0, 100.0, 140.0)),
        );
        assert!(ox.abs() < 0.01, "x overhang {ox}");
        assert!((oy - 160.0).abs() < 0.01, "y overhang {oy}");
    }

    /// A degenerate extent cannot produce a NaN pasteboard. `canvas::present`
    /// asks this question before layout, so a page whose size is not yet known
    /// is an ordinary case, not an error.
    #[test]
    fn a_degenerate_extent_has_no_overhang() {
        assert_eq!(
            overhang(
                (0.0, 0.0),
                PageFrame::new(CROP, 0),
                Some(r(-160.0, 100.0, 100.0, 140.0))
            ),
            (0.0, 0.0)
        );
    }

    /// `Bounds::EMPTY` is `min = +∞, max = −∞`, and an infinite overhang would
    /// make the scroll content infinite. Declined, like `region` declines it.
    #[test]
    fn an_empty_content_box_has_no_overhang() {
        let empty = Rect {
            llx: f64::INFINITY,
            lly: f64::INFINITY,
            urx: f64::NEG_INFINITY,
            ury: f64::NEG_INFINITY,
        };
        assert_eq!(
            overhang((200.0, 200.0), PageFrame::new(CROP, 0), Some(empty)),
            (0.0, 0.0)
        );
    }
}
