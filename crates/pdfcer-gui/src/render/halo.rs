//! # `render::halo` — rasterizing the ground OUTSIDE the sheet
//!
//! ## The report, and which half of it this is
//!
//! `OPERATOR_REQUESTS.md` **O23**, second clause, 2026-08-21:
//!
//! > *"also objects should still be reachable even if they are off the page."*
//!
//! and, three weeks later, when the reach half had shipped and this one had
//! not:
//!
//! > *"how do I view and edit objects that are off of the page? we added this
//! > feature but I didn't see how to enable it."*
//!
//! O23 broke into three pieces and they landed in this order:
//!
//! | part | what it gave him | where it lives |
//! |---|---|---|
//! | A — **scroll** | slack on every side of the strip, so the grey can be reached | [`crate::canvas::geometry::content_extent`] |
//! | B1 — **reach** | a press in that grey becomes a gesture, so an off-page object can be selected and dragged | [`crate::canvas::pasteboard`] |
//! | B2 — **see** | the off-page object is actually painted | **this module** |
//!
//! Until B2 the operator could select an object he could not see and watch its
//! properties change — honest, and useless. B1's own header says so verbatim so
//! that nobody reading it mistakes reach for sight.
//!
//! ## ★★★ Why the object was invisible, in one sentence
//!
//! `pdfcer_render::render_page` sizes its pixmap to the page's `/CropBox`.
//! Nothing culls the *content* — [`crate::render::offpage`] proves that against
//! the pinned engine — the raster simply has no pixels out there to put it in.
//!
//! So the whole of B2 is: **ask for a bigger box**. `render_page_region` takes
//! an arbitrary page-space rectangle and never intersects it with the crop box,
//! which `render::offpage`'s three tests assert directly because the engine's
//! own suite has never exercised a region outside the page.
//!
//! ## ★★ The box, and the two things it is NOT
//!
//! [`region`] returns the **crop box unioned with the drawn content's bounding
//! box** — in PDF user space, which is the space `render_page_region` and
//! [`crate::render::region::PageFrame`] both already speak, so `/Rotate` needs
//! no special case here at all.
//!
//! It is **not the visible rectangle**. That is the *other* region tier —
//! O24's, which engages above the pixmap ceiling and re-rasterizes whenever a
//! pan crosses a quantisation grid line. A halo raster is a picture of the
//! whole page plus its overhang, exactly as a whole-page raster is a picture of
//! the whole page, so **panning stays free**: the texture is cached under the
//! same key machinery, the operator scrolls out into the grey, and the picture
//! is already there.
//!
//! It is **not overscanned or quantised**.
//! [`crate::render::strategy::region_for`] grows and snaps the visible rect
//! because the visible rect moves continuously; this box moves only when the
//! document is edited, and growing it would make the raster bigger than it has
//! to be for no cache benefit at all.
//!
//! ## ★★ The ceiling, and why exceeding it returns `None` rather than a clamp
//!
//! A halo box is at least as large as the crop box and can be far larger — an
//! object dragged 5,000 pt off a 200 pt page makes it 26 times the sheet. Past
//! `pdfcer_render::MAX_PIXMAP_EDGE` the engine refuses outright, and the shell
//! must not ask.
//!
//! The answer is [`None`], which means *"there is no halo tier for this page at
//! this zoom"* and lets `canvas::present` fall through to O24's visible-region
//! tier — which covers whatever of the grey is actually **on screen**, because
//! [`reach`] stopped that tier clipping its visible rect to the sheet. So the
//! two tiers compose: below the ceiling one cached raster covers everything;
//! above it, the viewport-sized raster covers what is being looked at. There is
//! no zoom at which off-page content disappears, which matters because the zoom
//! an operator uses to *edit* an off-page object is exactly the deep one.
//!
//! A clamp would have been the wrong answer for the reason
//! [`crate::render::region::region_on_screen`]'s header already gives about a
//! different rectangle: shrinking the box without telling the destination is
//! how the right pixels end up in the wrong place.
//!
//! ## ★ The tolerance, and the case it deliberately drops
//!
//! Content bounding boxes poke a hair outside the crop box all the time — half
//! a stroke width on a border line is enough, and a CAD title block draws one
//! on every sheet. Flipping every such page into a bigger raster would cost
//! every operator memory and time to show nothing. So an overhang under
//! [`OVERHANG_TOLERANCE_PTS`] is not a halo.
//!
//! The case that drops is an object deliberately placed less than a point
//! outside the sheet. It stays invisible, exactly as it is today, and it is
//! **already unreachable by eye** at any zoom where a point is less than a
//! pixel. Stated here rather than left to be discovered.
//!
//! ## Rule 15
//!
//! Every number in this module is a **pdf dimension** — a coordinate in the
//! CAD-exported page's own user space. None of it is a **ce dimension**;
//! nothing here authors anything.

use pdfcer_core::page_tree::Rect;

use super::region::PageFrame;

/// How far content may hang outside the crop box before it is treated as
/// deliberately off-page, in **pdf** points.
///
/// See the module header. One point is about a stroke's half-width on a heavy
/// border and about 1/72 inch — far below anything an operator could see, and
/// far below anything he would have placed on purpose.
pub const OVERHANG_TOLERANCE_PTS: f64 = 1.0;

/// **The box to rasterize so that every object on this page is painted**, or
/// [`None`] if this page does not need one.
///
/// * `crop` — the page's crop box, in PDF user space.
/// * `content` — the drawn content's bounding box in the same space, as
///   `PageObjects::page_bbox` reports it. [`None`] when the page has not been
///   decomposed, which is the ordinary state of a strip neighbour.
/// * `raster_scale` — the device scale the whole-page tier would render at.
///
/// Returns [`None`] in three distinct situations, and they are three different
/// facts about the page rather than three spellings of "no":
///
/// 1. **nothing is known** — `content` is `None` or empty. Not "there is no
///    off-page content"; *"nobody has looked"*. The caller must not read this
///    as a guarantee.
/// 2. **nothing hangs over** — the union is the crop box, to within
///    [`OVERHANG_TOLERANCE_PTS`]. The overwhelming majority of pages, and the
///    reason this whole module costs a normal document nothing.
/// 3. **it would not fit** — see the module header; O24's visible-region tier
///    takes over.
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

    // ★ The same ceiling `strategy::for_page` applies to the whole-page tier,
    // asked of the bigger box. Rotation is irrelevant to it — a quarter turn
    // swaps the two edges and does not change which is longest — which is why
    // this function needs no `PageFrame` and the one below does.
    let longest = (union.urx - union.llx).max(union.ury - union.lly);
    if !longest.is_finite() || longest <= 0.0 || !raster_scale.is_finite() || raster_scale <= 0.0 {
        return None;
    }
    let ceiling = f64::from(pdfcer_render::MAX_PIXMAP_EDGE - 1);
    if longest * f64::from(raster_scale) > ceiling {
        return None;
    }
    Some(union)
}

/// Whether every corner of `r` is a finite number.
///
/// Its own function because `Bounds::EMPTY` is built from infinities on
/// purpose — see [`region`]'s case 1 — so "is this box real?" is a question
/// this module asks twice and must answer the same way both times.
#[must_use]
fn finite(r: Rect) -> bool {
    r.llx.is_finite() && r.lly.is_finite() && r.urx.is_finite() && r.ury.is_finite()
}

/// **How far the canvas may look past the sheet**, in screen coordinates.
///
/// O24's visible-region tier asks *"what part of this page can be seen?"* and
/// answered it, until this function existed, with `visible.intersect(page)` —
/// which is correct as a statement about the **sheet** and wrong as a statement
/// about the **page's content**. It is what made an off-page object invisible
/// at every zoom above the pixmap ceiling even after [`region`] had covered
/// every zoom below it.
///
/// * `place` — where the sheet is on screen.
/// * `extent` — the page's canvas extent in points, as
///   [`crate::viewer::page_extent_pts`] reports it (rotation already resolved).
///   `place` was laid out from this, so the two scales come from the pair and
///   not from the crop box — a page whose extent rounded is still placed
///   exactly on itself, which is
///   [`crate::render::region::region_on_screen`]'s rule and must stay one rule.
/// * `frame` — the page's crop box and `/Rotate`.
/// * `content` — the drawn content's bounding box in PDF user space, or
///   [`None`] when nobody has looked.
///
/// Returns `place` unchanged whenever there is nothing to add, so the caller
/// has no branch and the ordinary page keeps exactly today's behaviour.
///
/// ## ★ Why this takes the content box and not [`region`]'s answer
///
/// [`region`] returns [`None`] above the pixmap ceiling — which is precisely
/// when this function matters. Feeding it here would switch the visible-region
/// tier's reach off at the one zoom it is the only tier left.
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

    // ★★ Into CANVAS space — y-down from the page's top-left, `/Rotate`
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

#[cfg(test)]
mod tests {
    use super::{OVERHANG_TOLERANCE_PTS, reach, region};
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

    /// ★ A border stroke's half-width is not a feature request. See
    /// [`OVERHANG_TOLERANCE_PTS`].
    #[test]
    fn a_hairline_overhang_is_not_a_halo() {
        let hair = OVERHANG_TOLERANCE_PTS / 2.0;
        assert_eq!(
            region(CROP, Some(r(-hair, -hair, 200.0 + hair, 200.0 + hair)), 1.0),
            None
        );
    }

    /// ★★★ The feature. An object off the left edge grows the box to the left
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

    /// ★★ The ceiling. The same object at a zoom whose raster would not fit
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

    /// ★★★ An object off the LEFT of an upright page reaches LEFT on screen.
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

    /// ★★ O174's case, which a y-only conversion cannot produce: on a
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
}
