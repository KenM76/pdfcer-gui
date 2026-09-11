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
//!
//! ## ★★★ `/Rotate` — O174, and why a y flip alone was never enough
//!
//! The operator, 2026-09-10, on `A-591.pdf`:
//!
//! > *"this pdf causes problems zooming past about 1600% — the view appears to
//! > jump to another location and when I pan back to where something is visible
//! > it appears to be distorted."*
//!
//! That file is a single 792 × 1224 pt page carrying **`/Rotate 270`** (set by
//! an incremental update; the original object says `/Rotate 0`, which is why a
//! naive grep of the file finds both). Every fixture in this repository and
//! every page in the engine's own region tests is `/Rotate 0`, so until his
//! file arrived nothing had ever exercised this path on a turned page.
//!
//! ### The two coordinate systems this module bridges, precisely
//!
//! | space | origin | y | `/Rotate` |
//! |---|---|---|---|
//! | **canvas** — what the canvas lays out, hit-tests and draws in | page's top-left **after** turning | down | already **resolved**; a 270°-turned 792 × 1224 page is 1224 × 792 here |
//! | **PDF user** — what `render_page_region` takes | `CropBox` lower-left | up | **not applied**; the rect is still 792 × 1224, and the engine turns it itself |
//!
//! The original implementation converted between them with one subtraction —
//! `height − y` — which is exactly right for `/Rotate 0` **on a crop box whose
//! origin is (0, 0)**, and silently wrong for every other page. On his sheet
//! the shell asked the engine for a rectangle whose axes were swapped and
//! mirrored relative to the one the operator was looking at:
//!
//! - the engine rasterized **a different part of the drawing** → *"the view
//!   appears to jump to another location"*;
//! - and that raster's width and height were **swapped**, while
//!   [`region_on_screen`] computed a destination rect from the un-swapped
//!   canvas rect, so the texture was stretched into the wrong aspect →
//!   *"it appears to be distorted"*.
//!
//! Both symptoms, from one missing transform, appearing exactly at the zoom
//! where `render::strategy` hands over from the whole-page tier to this one —
//! `MAX_PIXMAP_EDGE / 1224 pt ≈ 13.4×`, i.e. about **1,340 %** on his page at a
//! 100 % display, which is the *"about 1600%"* he reported.
//!
//! ### ★★ Why the existing round-trip test could not see it
//!
//! `a_region_maps_to_screen_and_back_to_itself` asserts that [`page_region`]
//! and [`region_on_screen`] are inverses **of each other**. They were — both
//! made the same wrong assumption, so the round trip closed perfectly while
//! the rectangle handed to the engine pointed somewhere else entirely. An
//! oracle built out of both halves of the system under test agrees with itself
//! by construction.
//!
//! The calibration that *can* see it is
//! [`tests::the_engine_rasterizes_the_rectangle_the_canvas_asked_for`]: it
//! pushes the region this module produces through the **engine's own**
//! `region_base_geometry_of` and asserts the device rectangle that comes back
//! is the canvas rectangle we started from. That is a measurement against the
//! other side of the boundary, and it fails on `/Rotate 90`, `180` and `270`
//! before this fix.

use pdfcer_core::page_tree::{Page, Rect};

/// Everything about a page that the canvas ⟷ PDF-user-space conversions need:
/// its crop box and its `/Rotate`.
///
/// # Why a struct rather than two arguments
///
/// Because they are only meaningful together, and because the pair is what
/// makes a *page* — the same reason `pdfcer_render::RegionGeometry` is a struct
/// rather than five positional values. A caller that passed a crop box and
/// forgot the rotation would get the pre-O174 behaviour back, compiling
/// cleanly, on the exact pages where it is wrong.
///
/// # ★ The crop box is narrowed through `f32` on the way in
///
/// `pdfcer_render::region_base_geometry_of` does this — deliberately, with a
/// comment saying why: the whole-page path truncates the crop box to `f32`
/// before it multiplies, and computing from the `f64` box instead lands on a
/// different pixel for some pages, which broke the engine's own poster-tiling
/// reassembly test. This module's job is to be the **exact inverse** of that
/// function, so it must start from the same numbers. Narrowing here rather
/// than at each use keeps that a property of the type instead of a rule
/// someone has to remember four times.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageFrame {
    /// The crop box, already narrowed through `f32` exactly as the engine
    /// narrows it. **PDF user space**, y-up.
    crop: Rect,
    /// `/Rotate`, verbatim. Matched with the same arms the engine matches —
    /// `90`, `180`, `270`, and everything else treated as upright — rather
    /// than normalised here, because a second normalisation rule is a second
    /// place for the two sides to disagree about what `/Rotate 450` means.
    rotate: u16,
}

impl PageFrame {
    /// The frame of a page as the engine sees it.
    #[must_use]
    pub fn of(page: &Page) -> Self {
        Self::new(page.crop_box, page.rotate)
    }

    /// The crop box **as this type holds it** — already narrowed through
    /// `f32` the way the engine narrows it.
    ///
    /// ★ Exposed so [`super::halo::region`] — and `canvas::present`, which
    /// calls it — union the content box against the *same* numbers
    /// [`Self::canvas_box_of`] maps against. Reading `page.crop_box` there
    /// instead would be a second source for one value, and the two differ by
    /// up to an `f32` ulp — enough for a halo that is exactly the crop box to
    /// come back as a one-ulp overhang and flip a whole document into a bigger
    /// raster for nothing.
    #[must_use]
    pub const fn crop(self) -> Rect {
        self.crop
    }

    /// The frame from a crop box and a rotation given directly, for tests and
    /// for any caller holding those rather than a [`Page`].
    #[must_use]
    pub fn new(crop: Rect, rotate: u16) -> Self {
        Self {
            crop: Rect {
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "narrowing through f32 is the point — see the type's documentation" // ui-text-exempt: clippy lint justification, never displayed
                )]
                llx: f64::from(crop.llx as f32),
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "narrowing through f32 is the point — see the type's documentation" // ui-text-exempt: clippy lint justification, never displayed
                )]
                lly: f64::from(crop.lly as f32),
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "narrowing through f32 is the point — see the type's documentation" // ui-text-exempt: clippy lint justification, never displayed
                )]
                urx: f64::from(crop.urx as f32),
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "narrowing through f32 is the point — see the type's documentation" // ui-text-exempt: clippy lint justification, never displayed
                )]
                ury: f64::from(crop.ury as f32),
            },
            rotate,
        }
    }

    /// An upright page of the given size with its origin at (0, 0) — the shape
    /// every fixture in this repository had before O174, and the only shape the
    /// pre-O174 arithmetic was correct for.
    ///
    /// Kept for tests, which is where a page is described by its size rather
    /// than by a box.
    #[cfg(test)]
    #[must_use]
    pub fn upright(page_pts: (f32, f32)) -> Self {
        Self::new(
            Rect::from_corners(0.0, 0.0, f64::from(page_pts.0), f64::from(page_pts.1)),
            0,
        )
    }

    /// **Canvas space → PDF user space**, at scale 1, in `f64`.
    ///
    /// The exact inverse of the coefficient table in
    /// `pdfcer_render::region_base_geometry_of`, which at `scale = 1` maps user
    /// space onto the same canvas space this shell lays pages out in:
    ///
    /// | `/Rotate` | user → canvas | canvas → user (this function) |
    /// |---|---|---|
    /// | 0 (and any other value) | `cx = x − llx`, `cy = ury − y` | `x = llx + cx`, `y = ury − cy` |
    /// | 90 | `cx = y − lly`, `cy = x − llx` | `x = llx + cy`, `y = lly + cx` |
    /// | 180 | `cx = urx − x`, `cy = y − lly` | `x = urx − cx`, `y = lly + cy` |
    /// | 270 | `cx = ury − y`, `cy = urx − x` | `x = urx − cy`, `y = ury − cx` |
    ///
    /// ★ Note that 90 and 270 **swap the axes**: the canvas's x comes from the
    /// PDF's y. That is the whole of O174 — a conversion that only subtracted
    /// in y could never produce it, however carefully the subtraction was
    /// written.
    #[must_use]
    fn canvas_to_user(self, cx: f64, cy: f64) -> (f64, f64) {
        let Rect { llx, lly, urx, ury } = self.crop;
        match self.rotate {
            90 => (llx + cy, lly + cx),
            180 => (urx - cx, lly + cy),
            270 => (urx - cy, ury - cx),
            _ => (llx + cx, ury - cy),
        }
    }

    /// **PDF user space → canvas space**, at scale 1, in `f64`. The forward
    /// direction of the table in [`Self::canvas_to_user`], and its exact
    /// inverse.
    #[must_use]
    fn user_to_canvas(self, x: f64, y: f64) -> (f64, f64) {
        let Rect { llx, lly, urx, ury } = self.crop;
        match self.rotate {
            90 => (y - lly, x - llx),
            180 => (urx - x, y - lly),
            270 => (ury - y, urx - x),
            _ => (x - llx, ury - y),
        }
    }

    /// The bounding box in **canvas space** of a rectangle given in PDF user
    /// space, as `(x0, y0, x1, y1)` with `x0 ≤ x1` and `y0 ≤ y1`.
    ///
    /// ★ A bounding box of the two mapped corners, not a corner-by-corner
    /// copy: under 90° and 270° the axes swap and under 180° both mirror, so
    /// the mapped "lower-left" is not the canvas's top-left. Every rotation
    /// here is a multiple of 90°, so the box of the two opposite corners is
    /// the exact image of the rectangle — no rotation-of-a-rotated-rect
    /// inflation is possible.
    ///
    /// ★ Visible to the rest of `render` rather than private, because
    /// [`super::halo::reach`] needs the same mapping for the content bounding
    /// box and O174 is exactly the class of defect that a second hand-written
    /// copy reproduces. Deliberately **not** `pub`: PDF-user-space geometry is
    /// this module's subject, and a caller outside `render` that wants canvas
    /// coordinates wants [`region_on_screen`] instead.
    #[must_use]
    pub(in crate::render) fn canvas_box_of(self, region: Rect) -> (f64, f64, f64, f64) {
        let (ax, ay) = self.user_to_canvas(region.llx, region.lly);
        let (bx, by) = self.user_to_canvas(region.urx, region.ury);
        (ax.min(bx), ay.min(by), ax.max(bx), ay.max(by))
    }
}

/// The visible part of a page in **PDF user space**, y-up, ready for
/// `pdfcer_render::render_page_region`.
///
/// * `visible_canvas` — what the operator can see of this page, in canvas
///   points from the page's top-left, y-down, **with `/Rotate` already
///   resolved** (which is what canvas space means — see
///   [`crate::viewer::page_extent_pts`]).
/// * `frame` — the page's crop box and rotation, which together say how canvas
///   space and PDF user space are related for *this* page.
///
/// The rectangle is quantised by [`super::strategy::region_for`] first, so a
/// small pan asks for the rectangle already rasterized — see that function for
/// why that is the difference between panning smoothly and waiting for a redraw
/// on every pixel of movement.
///
/// ★★ The quantisation happens in **canvas** space, before the conversion, and
/// that ordering is load-bearing: the grid the pan snaps to has to be the grid
/// the *window* moves on, and the window moves in canvas space. Snapping after
/// the rotation would put the grid on the page's un-turned axes, so a pan due
/// east on his sheet would cross grid lines belonging to north.
#[must_use]
/// ★★ `visible_canvas` is `f64`: at deep zoom it holds a rectangle a few times
/// 10⁻⁸ pt wide at an absolute position near 540, and `f32` cannot carry both
/// magnitudes at once. See [`super::strategy::region_for`].
pub fn page_region(visible_canvas: (f64, f64, f64, f64), frame: PageFrame) -> Rect {
    let (x0, y0, x1, y1) = super::strategy::region_for(visible_canvas);
    let (ax, ay) = frame.canvas_to_user(x0, y0);
    let (bx, by) = frame.canvas_to_user(x1, y1);
    // `from_corners` normalises, which is what turns the two mapped corners
    // into the rectangle's image under a quarter turn.
    Rect::from_corners(ax, ay, bx, by)
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
///
/// `page_pts` is the page's **canvas** extent as
/// [`crate::viewer::page_extent_pts`] reports it — rotation already applied,
/// and rounded the way the engine rounds its pixmap. It is what `page_screen`
/// was laid out from, so the two scales are derived from it and not from
/// `frame`'s crop box; a page whose extent rounded is then still placed exactly
/// on itself.
#[must_use]
pub fn region_on_screen(
    region: Rect,
    page_pts: (f32, f32),
    frame: PageFrame,
    page_screen: egui::Rect,
) -> egui::Rect {
    let (w, h) = (f64::from(page_pts.0), f64::from(page_pts.1));
    if w <= 0.0 || h <= 0.0 {
        return page_screen;
    }
    let sx = f64::from(page_screen.width()) / w;
    let sy = f64::from(page_screen.height()) / h;
    // Back to canvas space — the full inverse this time, not a lone y flip.
    let (cx0, cy0, cx1, cy1) = frame.canvas_box_of(region);
    let left = f64::from(page_screen.min.x) + cx0 * sx;
    let right = f64::from(page_screen.min.x) + cx1 * sx;
    let top = f64::from(page_screen.min.y) + cy0 * sy;
    let bottom = f64::from(page_screen.min.y) + cy1 * sy;
    #[allow(
        clippy::cast_possible_truncation,
        reason = "an egui::Rect is f32; the narrowing is the boundary, and region_on_screen_deep exists for the zooms where it is not good enough" // ui-text-exempt: clippy lint justification, never displayed
    )]
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
    frame: PageFrame,
    anchor: crate::viewer::deep::DeepAnchor,
    zoom: f64,
    viewport_origin: egui::Pos2,
) -> egui::Rect {
    // PDF user space back to canvas space, then anchor-relative, then scaled.
    // The subtraction happens BEFORE the multiply, so nothing large is ever
    // formed.
    let to_screen = |cx: f64, cy: f64| {
        (
            f64::from(viewport_origin.x) + f64::from(anchor.screen.0) + (cx - anchor.page.0) * zoom,
            f64::from(viewport_origin.y) + f64::from(anchor.screen.1) + (cy - anchor.page.1) * zoom,
        )
    };
    let (cx0, cy0, cx1, cy1) = frame.canvas_box_of(region);
    let (x0, y0) = to_screen(cx0, cy0);
    let (x1, y1) = to_screen(cx1, cy1);
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the narrowing is the boundary to egui; the arithmetic above is what this function exists to keep in f64" // ui-text-exempt: clippy lint justification, never displayed
    )]
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

    /// An upright Letter page whose crop box starts at the origin — the shape
    /// every one of these tests assumed implicitly before O174.
    fn letter() -> PageFrame {
        PageFrame::upright(LETTER)
    }

    /// The operator's own page: 792 × 1224 pt turned a quarter turn, so its
    /// **canvas** extent is 1224 × 792.
    const A591_CROP: (f64, f64) = (792.0, 1224.0);
    const A591_CANVAS: (f32, f32) = (1224.0, 792.0);

    fn a591() -> PageFrame {
        PageFrame::new(Rect::from_corners(0.0, 0.0, A591_CROP.0, A591_CROP.1), 270)
    }

    /// Every rotation the engine distinguishes, plus one it does not, and a
    /// crop box that does **not** start at the origin — because the pre-O174
    /// arithmetic was wrong about that too and nothing would have caught it.
    fn frames() -> Vec<(&'static str, PageFrame, (f32, f32))> {
        let letter_box = Rect::from_corners(0.0, 0.0, 612.0, 792.0);
        // A crop box offset from the MediaBox origin, as a trimmed press sheet
        // carries.
        let offset_box = Rect::from_corners(20.0, 35.0, 632.0, 827.0);
        vec![
            ("upright", PageFrame::new(letter_box, 0), (612.0, 792.0)),
            ("90", PageFrame::new(letter_box, 90), (792.0, 612.0)),
            ("180", PageFrame::new(letter_box, 180), (612.0, 792.0)),
            ("270", PageFrame::new(letter_box, 270), (792.0, 612.0)),
            (
                "offset crop, upright",
                PageFrame::new(offset_box, 0),
                (612.0, 792.0),
            ),
            (
                "offset crop, 270",
                PageFrame::new(offset_box, 270),
                (792.0, 612.0),
            ),
            ("A-591 (the report)", a591(), A591_CANVAS),
        ]
    }

    // =======================================================================
    // ★★★ O174 — the calibration against the OTHER SIDE of the boundary
    // =======================================================================

    /// ★★★ **The engine rasterizes the rectangle the canvas asked for** — on
    /// every rotation, and on a crop box that does not start at the origin.
    ///
    /// # Why this test and not another round trip
    ///
    /// `a_region_maps_to_screen_and_back_to_itself` below asserts that
    /// [`page_region`] and [`region_on_screen`] are inverses of each other, and
    /// it passed for the whole life of the region tier while his page was being
    /// rasterized in the wrong place. Both halves shared one wrong assumption,
    /// so they agreed perfectly. **An oracle assembled from two pieces of the
    /// system under test measures their agreement, not their correctness.**
    ///
    /// The independent oracle is `pdfcer_render::region_base_geometry_of`: the
    /// engine's own user-space → device-space mapping, the very function
    /// `render_page_region` uses to decide which pixels to make. Push a canvas
    /// rectangle out through [`page_region`] and back in through that, and the
    /// device rectangle that comes out must be the canvas rectangle we started
    /// from. Anything else means the operator is being shown a different part
    /// of his drawing from the one he is pointing at.
    ///
    /// ★ Asserted at `scale = 1`, where device space **is** canvas space. The
    /// engine's `x0`/`y0` are the region's left and top edges in page-device
    /// space, and `width`/`height` its size there, so the comparison needs no
    /// arithmetic of its own — which is the point, since arithmetic in a test
    /// is one more place to make the same mistake twice.
    #[test]
    fn the_engine_rasterizes_the_rectangle_the_canvas_asked_for() {
        // A window somewhere in the middle of the sheet, in canvas points.
        let visible = (300.0_f64, 220.0, 380.0, 275.0);
        for (name, frame, _extent) in frames() {
            let region = page_region(visible, frame);
            let geometry =
                pdfcer_render::region_base_geometry_of(frame.crop, frame.rotate, 1.0, region)
                    .unwrap_or_else(|| panic!("{name}: the engine refused the region {region:?}"));

            // What `region_for` actually asked for, in canvas space — the
            // quantised rect, not the raw one, because that is what was
            // converted.
            let (qx0, qy0, qx1, qy1) = super::super::strategy::region_for(visible);

            // ★ Half a pixel. The engine floors and ceils its device corners
            // to whole pixels, so an exact conversion still lands within one;
            // the defect this test exists for is off by hundreds.
            let tol = 1.0_f64;
            assert!(
                (f64::from(geometry.x0) - qx0).abs() <= tol
                    && (f64::from(geometry.y0) - qy0).abs() <= tol,
                "{name}: the engine will rasterize from device ({}, {}) but the canvas is looking \
                 at ({qx0:.3}, {qy0:.3}) — the operator would be shown a different part of the \
                 page. region={region:?}",
                geometry.x0,
                geometry.y0
            );
            assert!(
                (f64::from(geometry.width) - (qx1 - qx0)).abs() <= tol + 1.0
                    && (f64::from(geometry.height) - (qy1 - qy0)).abs() <= tol + 1.0,
                "{name}: the engine will make a {}×{} raster for a window that is {:.3}×{:.3} — \
                 the texture is drawn into a rect of the SECOND shape, so the picture comes out \
                 stretched. region={region:?}",
                geometry.width,
                geometry.height,
                qx1 - qx0,
                qy1 - qy0
            );
        }
    }

    /// ★★ **The two conversions are inverses on every rotation**, which is the
    /// property `canvas::present` depends on once the engine agrees about
    /// *which* rectangle is being drawn.
    ///
    /// Kept alongside the calibration above rather than replaced by it: they
    /// answer different questions, and the pair of them is what says both that
    /// the right pixels are made and that they are put in the right place.
    #[test]
    fn the_region_and_its_screen_rect_are_inverses_on_every_rotation() {
        let visible = (300.0_f64, 220.0, 380.0, 275.0);
        for (name, frame, extent) in frames() {
            let page_screen = egui::Rect::from_min_size(
                egui::pos2(100.0, 50.0),
                egui::vec2(extent.0 * 4.0, extent.1 * 4.0),
            );
            let region = page_region(visible, frame);
            let on_screen = region_on_screen(region, extent, frame, page_screen);
            let sx = f64::from(page_screen.width()) / f64::from(extent.0);
            let sy = f64::from(page_screen.height()) / f64::from(extent.1);
            let back = (
                (f64::from(on_screen.min.x) - f64::from(page_screen.min.x)) / sx,
                (f64::from(on_screen.min.y) - f64::from(page_screen.min.y)) / sy,
                (f64::from(on_screen.max.x) - f64::from(page_screen.min.x)) / sx,
                (f64::from(on_screen.max.y) - f64::from(page_screen.min.y)) / sy,
            );
            let wanted = super::super::strategy::region_for(visible);
            for (got, want) in [
                (back.0, wanted.0),
                (back.1, wanted.1),
                (back.2, wanted.2),
                (back.3, wanted.3),
            ] {
                assert!(
                    (got - want).abs() < 0.01,
                    "{name}: round trip lost the rect: {back:?} vs {wanted:?}"
                );
            }
        }
    }

    /// ★★ **A turned page's region is inside the page it belongs to.**
    ///
    /// The cheapest statement of O174 and the one that needs no engine call: a
    /// window in the middle of the canvas must map to a rectangle inside the
    /// **crop box**, which on his sheet is 792 wide and 1224 tall. The pre-fix
    /// arithmetic produced `llx` up to 1224 on a page only 792 wide — a
    /// rectangle off the side of the sheet, which is why he saw blank paper.
    #[test]
    fn a_turned_pages_region_lands_inside_its_crop_box() {
        let frame = a591();
        // The middle of the canvas: x in 0..1224, y in 0..792.
        let region = page_region((600.0, 380.0, 660.0, 425.0), frame);
        assert!(
            region.llx >= 0.0
                && region.urx <= A591_CROP.0
                && region.lly >= 0.0
                && region.ury <= A591_CROP.1,
            "the region {region:?} is outside the 792 × 1224 crop box of the operator's own page \
             — that is O174: the engine rasterizes somewhere else, and the canvas shows blank \
             paper"
        );
    }

    /// ★ **The rotation is not a no-op** — a guard against a future
    /// "simplification" that deletes the axis swap because two of the four
    /// arms look alike.
    #[test]
    fn a_quarter_turn_swaps_the_axes() {
        let upright = page_region(
            (600.0, 380.0, 660.0, 425.0),
            PageFrame::upright((1224.0, 792.0)),
        );
        let turned = page_region((600.0, 380.0, 660.0, 425.0), a591());
        assert!(
            (upright.llx - turned.llx).abs() > 100.0,
            "a 270° page must not produce the upright page's rectangle: {upright:?} vs {turned:?}"
        );
        // Width and height swap: 60 × 45 upright becomes 45 × 60 turned.
        assert!(
            (upright.width() - turned.height()).abs() < 0.01
                && (upright.height() - turned.width()).abs() < 0.01,
            "the turned rectangle should be the upright one with its axes swapped: {upright:?} vs \
             {turned:?}"
        );
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
        let region = page_region((0.0, 0.0, 60.0, 45.0), letter());
        assert!(
            region.ury > f64::from(LETTER.1) * 0.9,
            "the top of the page is the PDF's high y: {region:?}"
        );
        // …and near the bottom asks for low y.
        let low = page_region((0.0, 747.0, 60.0, 792.0), letter());
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
        let region = page_region((0.0, 0.0, 60.0, 45.0), letter());
        let on_screen = region_on_screen(region, LETTER, letter(), page_screen);
        assert!(
            on_screen.min.x < page_screen.min.x || on_screen.min.y < page_screen.min.y,
            "overscan should reach outside the page: {on_screen:?}"
        );
    }

    // =======================================================================
    // ★★★ Where the SHARP picture ends and the blurry one begins
    // =======================================================================

    /// ★★★ **The sharp raster covers the window, on every side, at every phase
    /// of the snap grid** — the operator's report of 2026-09-04.
    ///
    /// > *"the canvas does a fading around the edges on stuff shown at the
    /// > edges of the view. I don't want this. it should render true."*
    ///
    /// ## What this asserts, and why it is the composed chain rather than one
    /// function
    ///
    /// [`super::strategy::region_for`] has its own test of this property in
    /// page points, and it is the tighter one. This is the same claim made
    /// **where the operator makes it — in screen pixels, about the rectangle
    /// the texture is actually painted at** — and it therefore has to go
    /// through every conversion `canvas::present` goes through:
    ///
    /// | step | what it produces |
    /// |---|---|
    /// | the viewport, in the page's own points | what `present` derives from `visible_rect ∩ place` |
    /// | [`page_region`] (which calls `region_for`) | the PDF-space rect that will be rasterized |
    /// | [`region_on_screen`] | `paint_rect` — where that raster lands |
    ///
    /// ★★ The y flip lives in the middle of that chain and is the reason this
    /// test is worth writing separately. `region_for` snaps in canvas space,
    /// y-**down**; `page_region` then flips to PDF space, y-**up**; and
    /// `region_on_screen` flips back. A margin that is generous on the snapped
    /// low side and starved on the high side comes out of that pair of flips
    /// attached to a *different screen edge* than the page-space test names, and
    /// only a test that composes all three can say which edge of the operator's
    /// window is the starved one.
    ///
    /// ★ Since O174 the chain also crosses a rotation, so this runs on **every**
    /// frame rather than on an upright Letter page alone: a starved edge that
    /// depended on the axis swap would otherwise be invisible here.
    ///
    /// ## What a failure looks like on his screen
    ///
    /// `paint_rect` is where `canvas::present` draws the sharp texture;
    /// `canvas::backdrop` has already painted the low-resolution whole-page
    /// texture underneath, across the page's *whole* rect. So every screen pixel
    /// inside the window but outside `paint_rect` is showing **the blurry
    /// stand-in instead of the page**. A margin of zero on a side means that
    /// band opens along that edge of the window on the first pixel of a pan and
    /// stays open for the ~1.6 s a region raster takes on a CAD sheet.
    #[test]
    fn the_sharp_raster_covers_the_window_on_every_side() {
        for (name, frame, extent) in frames() {
            // A region-tier zoom: the page is ~26 windows across, so the window
            // is entirely inside it and `visible ∩ place` is the whole window —
            // the ordinary case at the zooms this tier engages at.
            let zoom = 24.0_f32;
            let window = egui::vec2(1400.0, 900.0);
            let page_screen = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(extent.0, extent.1) * zoom,
            );
            // The viewport in page points, which is what `present` hands on.
            let (vw, vh) = (f64::from(window.x / zoom), f64::from(window.y / zoom));

            let mut worst = f64::INFINITY;
            let mut worst_side = "";
            let mut worst_at = (0.0_f64, 0.0_f64);
            // One full grid step in each axis — the phase is the whole
            // variable, and a single position samples one phase and proves
            // nothing.
            for i in 0..120 {
                let t = f64::from(i) / 120.0;
                let (x0, y0) = (137.0 + t * vw * 0.5, 249.0 + t * vh * 0.5);
                let region = page_region((x0, y0, x0 + vw, y0 + vh), frame);
                let paint = region_on_screen(region, extent, frame, page_screen);
                // The window on screen, from the same canvas-space rect.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "canvas points at this magnitude are exact in f32" // ui-text-exempt: clippy lint justification, never displayed
                )]
                let view = egui::Rect::from_min_size(
                    egui::pos2(
                        page_screen.min.x + (x0 as f32) * zoom,
                        page_screen.min.y + (y0 as f32) * zoom,
                    ),
                    window,
                );
                for (side, gap, extent_px) in [
                    ("left", view.min.x - paint.min.x, window.x),
                    ("right", paint.max.x - view.max.x, window.x),
                    ("top", view.min.y - paint.min.y, window.y),
                    ("bottom", paint.max.y - view.max.y, window.y),
                ] {
                    let fraction = f64::from(gap / extent_px);
                    if fraction < worst {
                        worst = fraction;
                        worst_side = side;
                        worst_at = (x0, y0);
                    }
                }
            }
            assert!(
                // ★ `0.24`, a little under the quarter the snap can guarantee —
                // this chain crosses `f32` twice (the page's screen rect and the
                // returned `egui::Rect`) at a magnitude of ~19,000 px, so a few
                // ULPs of slack is honest rather than lax. What the threshold
                // excludes is a side with no margin at all, which is the reported
                // defect and which measured `0.0000` here before the fix.
                worst >= 0.24,
                "{name}: the sharp raster reaches only {:.4} of a window past the {worst_side} \
                 edge at its worst grid phase (view origin {:.2},{:.2} pt) — every pixel between \
                 there and the edge of the window is `canvas::backdrop`'s low-resolution \
                 stand-in, which is the operator's fade",
                worst,
                worst_at.0,
                worst_at.1
            );
        }
    }

    /// ★★★ **The deep placement stays exact where the shallow one cannot.**
    ///
    /// At four billion percent the page's own screen rect has a magnitude of
    /// ~10^12 px, where `f32`'s spacing is 131,072 px — coarser than the whole
    /// window. The anchor-based path never forms that number, so the rect it
    /// returns is correct to a fraction of a pixel.
    ///
    /// Asserted by placing the anchor ON the region's own canvas-space corner:
    /// the answer must then be the viewport origin exactly, at any zoom. ★ The
    /// anchor is seeded through [`PageFrame::canvas_box_of`] rather than by
    /// hand, because after O174 "the region's corner in canvas space" is a
    /// rotation away from its `llx`/`ury` and a hand-written seed would only be
    /// right for the upright case — which is the whole class of mistake this
    /// module was just corrected for.
    #[test]
    fn the_deep_placement_is_exact_at_zooms_where_f32_is_not() {
        let origin = egui::pos2(0.0, 0.0);
        for (name, frame, _extent) in frames() {
            let region = page_region((300.0, 400.0, 300.1, 400.1), frame);
            let (cx0, cy0, _, _) = frame.canvas_box_of(region);
            for zoom in [1.0e6_f64, 1.0e8, 4.3e9, 1.0e11] {
                let anchor = crate::viewer::deep::DeepAnchor {
                    page: (cx0, cy0),
                    screen: (0.0, 0.0),
                };
                let r = region_on_screen_deep(region, frame, anchor, zoom, origin);
                assert!(
                    r.min.x.abs() < 0.5 && r.min.y.abs() < 0.5,
                    "{name}: at {zoom}x the anchored corner drifted to {:?}",
                    r.min
                );
                assert!(
                    r.width() > 0.0 && r.height() > 0.0,
                    "{name}: at {zoom}x the rect collapsed: {r:?}"
                );
            }
        }
    }

    /// A degenerate page yields the page's own rect rather than a division by
    /// zero — the canvas then draws as it always did.
    #[test]
    fn a_degenerate_page_falls_back_to_the_page_rect() {
        let page_screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        let region = Rect::from_corners(0.0, 0.0, 10.0, 10.0);
        assert_eq!(
            region_on_screen(region, (0.0, 0.0), letter(), page_screen),
            page_screen
        );
    }
}
