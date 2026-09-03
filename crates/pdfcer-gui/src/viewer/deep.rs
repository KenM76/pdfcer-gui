//! # `viewer::deep` — where the view is, when the scroll offset can no longer say
//!
//! `OPERATOR_REQUESTS.md` **O24**, step 2. The operator:
//!
//! > *"how do we get to the insanely high limit? … You should be able to have a
//! > new algorithm take over for bigger zooms?"*
//!
//! He is right that something has to take over, and this is it — but it is not
//! about how pixels are made. **It is about where the viewport's position is
//! stored.**
//!
//! ## ★★★ The measurement that makes this necessary
//!
//! Today the position is an `egui::ScrollArea` offset into a content rectangle
//! of `page × zoom`, and those offsets are `f32`. A screen pixel is exactly one
//! unit of that content space, so the spacing between representable offsets
//! **is** the positioning error in pixels:
//!
//! | zoom | content extent | `f32` step | error on screen |
//! |---|---|---|---|
//! | 100 % | 1,584 pt | 0.0001 | none |
//! | 10,000 % | 158,400 pt | 0.02 | none |
//! | **1,000,000 %** | 15,840,000 pt | 1.00 | **1 pixel** |
//! | 10,000,000 % | 158,400,000 pt | 16.00 | **16 pixels** |
//! | 100,000,000 % | 1,584,000,000 pt | 128.00 | **128 pixels** |
//!
//! Computed by taking the actual `f32` successor of each value, on a 1,584 pt
//! sheet. So the scroll offset stops being able to say where the operator is at
//! about a million percent, and by ten million the view moves in sixteen-pixel
//! jumps — it judders, then sticks.
//!
//! ★ **That is the whole justification for this module**, and it is worth
//! having in numbers because the first attempt at deriving it was wrong: an
//! earlier table divided by the zoom twice and concluded the error stayed
//! sub-pixel for ever, which would have made step 2 unnecessary. It is not. The
//! error is one content unit, and one content unit is one screen pixel, at
//! every zoom.
//!
//! ## What replaces it
//!
//! [`DeepAnchor`] — **a page-space point in `f64`, plus where on screen it
//! sits**. The position stops being "how far the scroll area has scrolled" and
//! becomes "this point of the page is under that pixel of the window", which is
//! a statement whose precision does not decay with the zoom: `f64` carries 53
//! bits of mantissa, so a page coordinate stays exact to far beyond any zoom a
//! person will type.
//!
//! ★ It is the same shape the engine reached for the same problem — its own
//! commit says *"the fix is one subtraction moved into `f64`"*, and its region
//! renderer takes a page-space rectangle rather than a device offset. This is
//! that idea carried one layer up, into the shell's own position model.
//!
//! ## What this module deliberately does NOT do
//!
//! It does not render, does not touch `egui`, and does not decide *when* the
//! deep path takes over — [`crate::render::strategy`] owns that question and
//! answers it from the pixmap ceiling. This is four numbers and the arithmetic
//! that keeps them consistent, which is what lets every claim above be a unit
//! test rather than something to be observed in a window.

/// Where the view is, expressed so that precision does not decay with zoom.
///
/// # The invariant, stated first because everything here serves it
///
/// **The page point [`Self::page`] is drawn at the window point
/// [`Self::screen`].** Panning moves `page`; zooming leaves `page` and
/// `screen` alone and changes only the scale applied between them. That is why
/// a zoom about the cursor is expressible without any large intermediate: the
/// anchor is *already* the thing being held still.
///
/// # Why `f64` for the page and `f32` for the screen
///
/// They are different magnitudes doing different jobs. A page coordinate at
/// deep zoom is the value that needs precision — it is what a scroll offset was
/// failing to carry. A **screen** coordinate is bounded by the window, a few
/// thousand at most, where `f32` is exact to a small fraction of a pixel and
/// always will be.
///
/// ★ Mixing them is deliberate rather than sloppy: making the screen point
/// `f64` too would imply the window can be large enough to need it, which is
/// the kind of false suggestion a type makes silently.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeepAnchor {
    /// The page-space point held under [`Self::screen`], in PDF points from the
    /// page's top-left, y-down — canvas space, matching
    /// [`crate::canvas::mapping`].
    pub page: (f64, f64),
    /// Where in the window that point sits, in logical points from the
    /// viewport's top-left.
    pub screen: (f32, f32),
}

impl DeepAnchor {
    /// The anchor that puts the page's top-left corner at the viewport's
    /// top-left — the state a document opens in.
    #[must_use]
    pub const fn origin() -> Self {
        Self {
            page: (0.0, 0.0),
            screen: (0.0, 0.0),
        }
    }

    /// Where a page point lands in the window at `zoom`.
    ///
    /// The forward half of the pair. Every large magnitude is subtracted
    /// **inside `f64`** before the result is narrowed, which is the whole
    /// technique: `point - self.page` is small even when both are billions, so
    /// the product with `zoom` is small, and nothing large ever reaches `f32`.
    #[must_use]
    pub fn to_screen(&self, point: (f64, f64), zoom: f64) -> (f32, f32) {
        let dx = (point.0 - self.page.0) * zoom;
        let dy = (point.1 - self.page.1) * zoom;
        (
            f64::from(self.screen.0) as f32 + dx as f32,
            f64::from(self.screen.1) as f32 + dy as f32,
        )
    }

    /// Which page point is under a window point at `zoom`.
    ///
    /// The exact inverse of [`Self::to_screen`], and the two are tested as
    /// inverses rather than each against a hand-computed number — a pair that
    /// round-trips is the property the canvas actually depends on.
    #[must_use]
    pub fn to_page(&self, screen: (f32, f32), zoom: f64) -> (f64, f64) {
        if !zoom.is_finite() || zoom <= 0.0 {
            return self.page;
        }
        let dx = f64::from(screen.0 - self.screen.0) / zoom;
        let dy = f64::from(screen.1 - self.screen.1) / zoom;
        (self.page.0 + dx, self.page.1 + dy)
    }

    /// Pan by a window-space delta: the content follows the hand.
    ///
    /// ★ The sign is the same convention [`crate::canvas::geometry::pan_offset`]
    /// uses and for the same reason — dragging right moves the page right,
    /// which means the page point under the cursor moves *left* in page space.
    /// Getting this backwards produces a canvas that works and feels wrong,
    /// which is harder to notice than one that is broken.
    #[must_use]
    pub fn panned(&self, delta: (f32, f32), zoom: f64) -> Self {
        if !zoom.is_finite() || zoom <= 0.0 {
            return *self;
        }
        Self {
            page: (
                self.page.0 - f64::from(delta.0) / zoom,
                self.page.1 - f64::from(delta.1) / zoom,
            ),
            screen: self.screen,
        }
    }

    /// Zoom about a window point, holding whatever page point is under it.
    ///
    /// ★★ **This is the operation the whole module exists for.** In the scroll
    /// -offset model, zooming about the cursor means solving for a new offset —
    /// which is where the large magnitudes and their lost precision came from.
    /// Here it is a re-statement: read which page point is under the cursor,
    /// then declare that *that* point is now anchored there. No large number is
    /// formed, so nothing is lost, at any zoom.
    #[must_use]
    pub fn zoomed_about(&self, at: (f32, f32), from_zoom: f64) -> Self {
        Self {
            page: self.to_page(at, from_zoom),
            screen: at,
        }
    }

    /// ★★★ **The `f32` page-local scroll offset that reproduces this anchor's
    /// placement** — the hand-over back out of the deep tier.
    ///
    /// # Why this is the last function this module needed
    ///
    /// [`crate::canvas::mod`]'s deep branch seeds an anchor from the scroll
    /// offset on the way **in**. Nothing converted an anchor back into a
    /// scroll offset on the way **out**, so a zoom-out across the threshold
    /// dropped the position on the floor and the `f32` machinery resumed from
    /// the zero the deep tier forces. `OPERATOR_REQUESTS.md` O26e/O26f, reported as
    /// *"zoom out … repositions the page so that it is off screen in the far
    /// bottom left corner"*.
    ///
    /// # The arithmetic, and where the precision goes
    ///
    /// A page-local offset is defined by
    /// [`crate::canvas::geometry::anchor_screen_pos`]:
    ///
    /// ```text
    ///     screen = margin(display, viewport) + frac × display − offset
    /// ```
    ///
    /// This anchor states `screen` and the page point directly, and
    /// `frac × display` is `page × zoom` — so
    ///
    /// ```text
    ///     offset = margin(display, viewport) + page × zoom − screen
    /// ```
    ///
    /// ★★ `page × zoom` is formed **in `f64`** and narrowed once, at the end.
    /// That is the entire reason this lives here rather than being spelled out
    /// at the call site in `f32`: near the threshold the product is about
    /// 1.4 × 10⁷, where an `f32`'s representable step is a whole screen pixel,
    /// and the `f32` route subtracts two such numbers to get one of a few
    /// hundred. Measured on the descent through 1,185,799 %: the `f32` route
    /// left the view fifty pixels out, this one is inside the one pixel the
    /// destination offset can represent at all.
    ///
    /// ★ The result is genuinely `f32` and that is not a compromise — it is
    /// being handed to an `egui` `ScrollArea`, which stores an `f32`. Below
    /// the threshold that is enough by definition; the threshold is the point
    /// at which it stops being enough, which is why the tier exists.
    ///
    /// `display` is the **current page's** drawn size and `viewport` the
    /// scroll viewport's, the same two measurements every solve in
    /// `canvas::geometry` is handed. A non-finite or non-positive `zoom`
    /// yields `None`: there is no placement to describe, and a `NaN` offset
    /// blanks the canvas.
    #[must_use]
    pub fn page_local_offset(
        &self,
        display: (f32, f32),
        viewport: (f32, f32),
        zoom: f64,
    ) -> Option<(f32, f32)> {
        if !zoom.is_finite() || zoom <= 0.0 {
            return None;
        }
        let axis = |page: f64, screen: f32, display: f32, viewport: f32| -> Option<f32> {
            // The centring margin, spelled the way `canvas::geometry::margin`
            // spells it. Zero at any zoom this function is reached at — the
            // page is millions of pixels across — but carried anyway, so the
            // identity holds for the unit tests that exercise it at ordinary
            // sizes rather than only where it is exercised in anger.
            let margin = f64::from((display.max(viewport) - display) / 2.0);
            let out = margin + page * zoom - f64::from(screen);
            let out = out as f32;
            out.is_finite().then_some(out)
        };
        Some((
            axis(self.page.0, self.screen.0, display.0, viewport.0)?,
            axis(self.page.1, self.screen.1, display.1, viewport.1)?,
        ))
    }

    /// The page-space rectangle visible in a viewport of `size` at `zoom` —
    /// what [`crate::render::strategy`]'s region tier asks the renderer for.
    ///
    /// Returned as `(x0, y0, x1, y1)` in canvas space. Degenerate input yields
    /// a degenerate rect rather than a panic; the caller's own guards refuse it.
    #[must_use]
    pub fn visible_rect(&self, size: (f32, f32), zoom: f64) -> (f64, f64, f64, f64) {
        let top_left = self.to_page((0.0, 0.0), zoom);
        let bottom_right = self.to_page(size, zoom);
        (top_left.0, top_left.1, bottom_right.0, bottom_right.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The zooms this module exists for, as multipliers.
    const DEEP: [f64; 4] = [1_000.0, 100_000.0, 1_000_000.0, 10_000_000.0];

    /// ★★★ **A page point round-trips through the screen at every depth.**
    ///
    /// The property the canvas depends on, and the one the `f32` scroll offset
    /// loses: at 10,000,000 % the offset's representable step is sixteen screen
    /// pixels, so a position could not survive this round trip at all.
    ///
    /// The tolerance is in **page points**, scaled by the zoom — a tenth of a
    /// screen pixel at whatever magnification is under test. A fixed page-space
    /// tolerance would get easier as the zoom rises, which is backwards.
    #[test]
    fn a_page_point_survives_the_round_trip_at_every_depth() {
        let anchor = DeepAnchor {
            page: (812.5, 431.25),
            screen: (400.0, 300.0),
        };
        for zoom in DEEP {
            for probe in [(812.5, 431.25), (812.6, 431.3), (800.0, 420.0)] {
                let screen = anchor.to_screen(probe, zoom);
                let back = anchor.to_page(screen, zoom);
                let tol = 0.1 / zoom; // a tenth of a screen pixel
                assert!(
                    (back.0 - probe.0).abs() < tol && (back.1 - probe.1).abs() < tol,
                    "zoom {zoom}: {probe:?} -> {screen:?} -> {back:?}"
                );
            }
        }
    }

    /// ★★ **Zooming about a point holds that point still** — the operation the
    /// module exists for, asserted as the invariant rather than as an offset.
    #[test]
    fn zooming_about_a_point_leaves_it_under_the_cursor() {
        let anchor = DeepAnchor {
            page: (100.0, 200.0),
            screen: (0.0, 0.0),
        };
        let cursor = (640.0_f32, 480.0);

        for &from in &DEEP {
            let under_before = anchor.to_page(cursor, from);
            let zoomed = anchor.zoomed_about(cursor, from);

            // At ANY new zoom the same page point is still under the cursor,
            // because the anchor now names it directly.
            for &to in &DEEP {
                let landed = zoomed.to_screen(under_before, to);
                assert!(
                    (landed.0 - cursor.0).abs() < 0.01 && (landed.1 - cursor.1).abs() < 0.01,
                    "from {from} to {to}: {landed:?} should be {cursor:?}"
                );
            }
        }
    }

    /// ★ **A pan of one screen pixel moves the view by one screen pixel**, at
    /// depths where the `f32` offset moves by sixteen or by nothing.
    ///
    /// This is the defect step 2 removes, stated positively.
    #[test]
    fn a_one_pixel_pan_moves_exactly_one_pixel_at_ten_million_percent() {
        let zoom = 100_000.0_f64; // 10,000,000 %
        let anchor = DeepAnchor {
            page: (5_000.0, 4_000.0),
            screen: (0.0, 0.0),
        };
        let probe = anchor.to_page((100.0, 100.0), zoom);

        let panned = anchor.panned((1.0, 0.0), zoom);
        let moved = panned.to_screen(probe, zoom);

        // The probe was at x=100; after panning right by one pixel it is at 101.
        assert!(
            (moved.0 - 101.0).abs() < 0.01,
            "a one-pixel pan should move the content one pixel: {moved:?}"
        );
        assert!((moved.1 - 100.0).abs() < 0.01, "y must not drift");
    }

    /// The pan sign follows the hand — dragging right moves the page right.
    #[test]
    fn panning_right_moves_the_page_right() {
        let anchor = DeepAnchor::origin();
        let zoom = 2.0;
        let probe = (10.0, 10.0);
        let before = anchor.to_screen(probe, zoom);
        let after = anchor.panned((50.0, 0.0), zoom).to_screen(probe, zoom);
        assert!(
            after.0 > before.0,
            "dragging right must move the content right: {before:?} -> {after:?}"
        );
    }

    /// The visible rect is the window mapped back into the page, and it shrinks
    /// as the zoom rises — which is what makes the region raster stay small.
    #[test]
    fn the_visible_rect_shrinks_as_the_zoom_rises() {
        let anchor = DeepAnchor::origin();
        let size = (800.0_f32, 600.0);

        let shallow = anchor.visible_rect(size, 1.0);
        let deep = anchor.visible_rect(size, 1_000.0);

        let w = |r: (f64, f64, f64, f64)| r.2 - r.0;
        assert!((w(shallow) - 800.0).abs() < 0.01);
        assert!(
            w(deep) < w(shallow) / 100.0,
            "at 100,000% the window covers a thousandth of the page: {deep:?}"
        );
    }

    /// Degenerate zoom is refused rather than propagated — an infinity or a
    /// zero here would reach a scroll extent and blank the canvas, which is the
    /// failure `canvas::geometry`'s guards exist for.
    #[test]
    fn a_degenerate_zoom_leaves_the_anchor_alone() {
        let anchor = DeepAnchor {
            page: (7.0, 9.0),
            screen: (1.0, 2.0),
        };
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(anchor.to_page((10.0, 10.0), bad), anchor.page);
            assert_eq!(anchor.panned((5.0, 5.0), bad), anchor);
        }
    }
}
