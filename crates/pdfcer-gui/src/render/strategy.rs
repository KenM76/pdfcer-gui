//! # `render::strategy` — whole page, or just the window?
//!
//! `OPERATOR_REQUESTS.md` **O24**. One decision, made in one place, from
//! numbers rather than from a mode flag: **at this zoom, on this page, do we
//! rasterize the whole sheet or only what is on screen?**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/strategy.md`.

/// How much extra to rasterize around the viewport, as a fraction of the
/// viewport on **each** side.
pub const OVERSCAN: f64 = 0.5;

/// What to hand the renderer for one page at one zoom.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Strategy {
    /// Rasterize the whole page. Today's path, and the one that makes panning
    /// free — see the module header.
    WholePage,
    /// Rasterize only the visible rectangle, plus [`OVERSCAN`] on each side.
    ///
    /// Carries the scale the caller should use; the rectangle itself is the
    /// canvas's to compute, because only it knows where the operator is
    /// looking. This module decides *whether*, not *where*.
    Region,
}

/// Decide the strategy for one page at one raster scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// This page has never been observed compositing in a subtractive space, so
    /// the ink ceiling is irrelevant to it and only `MAX_PIXMAP_EDGE` applies.
    ///
    /// **The overwhelming majority of pages, and every page of a CAD drawing.**
    Additive,
    /// This page HAS been observed compositing in ink. The value is the
    /// operator's ceiling from `Settings::max_cmyk_buffer_bytes`, passed
    /// verbatim — `None` means the engine's own default, which every one of its
    /// four public helpers already understands, so there is nothing to resolve
    /// here and no second place for a default to be decided.
    Subtractive(Option<usize>),
}

/// **Can a whole-page raster of this page be ALLOCATED at all at this scale?**
#[must_use]
pub fn whole_page_raster_fits(page_pts: (f32, f32), raster_scale: f32) -> bool {
    let longest = page_pts.0.max(page_pts.1);
    if !longest.is_finite() || longest <= 0.0 || !raster_scale.is_finite() || raster_scale <= 0.0 {
        return true;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "MAX_PIXMAP_EDGE is 16384; f32 is exact to 2^24" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let ceiling = (pdfcer_render::MAX_PIXMAP_EDGE - 1) as f32;
    longest * raster_scale <= ceiling
}

/// **Can a raster of this REGION be allocated at this scale?**
#[must_use]
pub fn region_raster_fits(region: pdfcer_core::page_tree::Rect, raster_scale: f32) -> bool {
    let longest = (region.urx - region.llx).max(region.ury - region.lly);
    if !longest.is_finite() || longest <= 0.0 || !raster_scale.is_finite() || raster_scale <= 0.0 {
        return true;
    }
    longest * f64::from(raster_scale) <= f64::from(pdfcer_render::MAX_PIXMAP_EDGE - 1)
}

#[must_use]
pub fn for_page(page_pts: (f32, f32), raster_scale: f32, ink: Ink) -> Strategy {
    let longest = page_pts.0.max(page_pts.1);
    if !longest.is_finite() || longest <= 0.0 || !raster_scale.is_finite() || raster_scale <= 0.0 {
        // A degenerate page or scale cannot be reasoned about, and the
        // whole-page path already refuses it safely. Never answer `Region` on
        // bad input: that would send a nonsense rectangle to the renderer.
        // Also the guard the ink arithmetic below relies on — it casts a
        // product to `u32`.
        return Strategy::WholePage;
    }
    // The hard ceiling, through [`whole_page_raster_fits`] rather than
    // restated here. One definition of `MAX_PIXMAP_EDGE`'s arithmetic, so the
    // tier the canvas picks and the order the strip declines to place cannot
    // come to disagree about where the wall is — which is the class of defect
    // that produced O186's raster error in the first place.
    if !whole_page_raster_fits(page_pts, raster_scale) {
        return Strategy::Region;
    }

    // The ink ceiling, second, and only for a page that has been seen asking
    // for it. See [`Ink`].
    let Ink::Subtractive(max_bytes) = ink else {
        return Strategy::WholePage;
    };
    // The raster the whole-page tier would actually ask for. Rounded UP, the
    // same direction the renderer rounds when it allocates: a raster half a
    // pixel over the ceiling is over it.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "both edges are below MAX_PIXMAP_EDGE by the guard above, and both are positive" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let (w, h) = (
        (page_pts.0 * raster_scale).ceil() as u32,
        (page_pts.1 * raster_scale).ceil() as u32,
    );
    // `will_composite_in_cmyk`, NOT a pixel count computed here. The
    // engine's request reply is explicit about why: *"the predicate exists so
    // the 20-B/px arithmetic stays on this side of the crate boundary; a copy
    // of a measured limit is a copy that rots the next time the buffer's
    // element type changes."* This shell's own request refused to hardcode
    // 13,421,772 for the same reason, and that refusal is what produced the
    // predicate.
    if pdfcer_render::will_composite_in_cmyk(w, h, max_bytes) {
        Strategy::WholePage
    } else {
        Strategy::Region
    }
}

/// The rectangle to rasterize for [`Strategy::Region`]: the visible rect grown
/// by [`OVERSCAN`] on each side.
///
/// Takes and returns page-space points. Growing here rather than at the call
/// site is what keeps the overscan one number in one place — a second caller
/// that grew it differently would produce a cache that never hits, because two
/// requests for the same view would ask for different rectangles.
#[must_use]
/// `f64`, since 2026-08-22 — see [`region_for`] for what `f32` cost here.
pub fn overscanned(visible: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (x0, y0, x1, y1) = visible;
    let w = (x1 - x0).abs();
    let h = (y1 - y0).abs();
    let dx = w * OVERSCAN;
    let dy = h * OVERSCAN;
    (x0 - dx, y0 - dy, x1 + dx, y1 + dy)
}

/// The page-space rectangle to rasterize for [`Strategy::Region`], **quantised**
/// so that small pans reuse the same raster.
///
/// `visible` is what the operator can see, in page points. The returned rect is
/// grown by [`OVERSCAN`] on each side and then snapped to a grid, and both
/// halves matter:
///
/// | | |
/// |---|---|
/// | **the overscan** | gives margin, so a pan inside it needs no new raster |
/// | **the quantisation** | makes the SAME view produce the SAME rect, so the cache hits |
///
/// Without the snap the rect would change on every pixel of movement, every
/// request would be a cache miss, and the operator would wait for a redraw
/// continuously — which is precisely the *"wait for detail to render after
/// panning"* he refused. The snap turns that into at most one redraw per half
/// viewport of travel.
///
/// The grid step is half the visible extent rather than a constant: a
/// constant in page points would be a different distance on screen at every
/// zoom, so the redraw cadence would vary with magnification for no reason the
/// operator could see.
///
/// # The snap moves the WINDOW, and it must move it about its CENTRE
///
///
/// > *"the canvas does a fading around the edges on stuff shown at the edges of
/// > the view. I don't want this. it should render true."*
///
/// He is describing [`crate::canvas::backdrop`]'s low-resolution whole-page
/// texture showing along the edge of the window — a second, blurry rendering of
/// content the sharp region raster ought to have covered. It is not a taste
/// question and it is not a slow raster: rule 4's *"applied content renders
/// exactly as saved content will render"* is violated the moment the same
/// content is on screen at two fidelities at once.
///
///
/// The origin was floored onto the grid and the window was then laid out
/// *forwards* from there:
///
/// ```text
/// snapped_x = (x0 / step_x).floor() * step_x      // ≤ x0, by up to step_x
/// window    = (snapped_x, snapped_x + w)          // …so it sits LEFT of the view
/// ```
///
/// `.floor()` only ever moves the window **towards the origin**, by anything up
/// to a whole grid step — and the grid step is half a viewport. The overscan is
/// then added symmetrically to a window that is already off-centre, so the
/// margin the operator actually gets is off-centre too:
///
/// | side | margin, in viewports |
/// |---|---|
/// | left / top | `0.5` … `1.0` |
/// | **right / bottom** | **`0.0` … `0.5`** |
///
/// Measured over a full grid step in 2,000 increments, before the fix:
/// **left `0.5000`, top `0.5002`, right `0.0002`, bottom `0.0000`.** At the
/// worst phase the sharp raster stops *exactly at the bottom edge of the
/// window* while a full half-screen of it is spent off the left, where nothing
/// can ever see it.
///
/// ## Why that reads as a permanent fade rather than an occasional one
///
/// The raster in hand lags the view: `canvas::present` computes the wanted
/// region from `last_scroll_offset`, which is the *previous* frame's, and the
/// current page's texture slot is deliberately served **without a staleness
/// check** (O24c) so that a pan shows the last good picture instead of blank
/// paper. On the benchmark CAD sheet a region raster takes ~1.6 s to land
/// (`BENCHMARK.md`: 691 ms of fixed cost plus ~0.19 µs per pixel), so for that
/// whole window what is drawn is a picture of the *previous* region.
///
/// ⇒ With a right-hand margin of ~0, **any pan at all — one pixel — puts the
/// view's leading edge outside the held raster**, and it stays outside for a
/// second and a half. The operator pans constantly. That is why he experiences
/// a fade that is always there rather than a redraw that occasionally lags.
///
/// ## The fix, and what it costs
///
/// Snap the window's **centre** to the same grid instead of its origin, and
/// round rather than floor:
///
/// ```text
/// centre_x  = (( x0 + w/2 ) / step_x).round() * step_x   // within step_x/2 of the view's centre
/// window    = (centre_x - w/2, centre_x + w/2)
/// ```
///
/// **This costs nothing.** The window is the same size, the returned rect
/// is still exactly `(1 + 2 × OVERSCAN)` viewports across, the grid step is
/// unchanged, and the cache-hit cadence is unchanged — `round(c / step)` steps
/// exactly as often as `floor(x0 / step)` did, once per half viewport of
/// travel. The only thing that changes is *which* half viewport of already-paid-
/// for raster the operator gets, and the answer becomes "a quarter of it on
/// every side" instead of "all of it on two sides and none on the other two".
/// Measured after the fix, same sweep: **left `0.2500`, right `0.2502`, top
/// `0.2502`, bottom `0.2500`.**
///
/// ## What it does NOT fix, stated rather than glossed
///
/// A quarter viewport is a guarantee about where the raster *reaches*, not
/// about how fast the operator moves. A pan of more than a quarter of the
/// window inside one raster's ~1.6 s still arrives beyond the held picture and
/// still shows the backdrop at the leading edge. Closing *that* means buying
/// more margin, and [`OVERSCAN`]'s table prices it: `0.75` costs about
/// **+0.5 s** per raster and `1.0` about **+1.2 s**, against a raster that
/// already takes ~1.6 s. That is an operator decision about a trade he has
/// already ruled on once (*"I don't want the affect that other readers have
/// where you always have to wait for detail"*), so the constant is left where
/// he set it and this function stops wasting what it buys.
///
/// # `f64`, and this is the arithmetic that forces it
///
/// `OPERATOR_REQUESTS.md` **O24i**. The snap divides a page coordinate by the
/// grid step:
///
/// ```text
/// snapped_x = (x0 / step_x).floor() * step_x
/// ```
///
/// (The division is unchanged by the centring above; only its numerator moved
/// from the window's origin to the window's centre, which has the same
/// magnitude and therefore the same requirement.)
///
/// At a trillion percent the visible extent is about 5 × 10⁻⁸ pt, so `step_x`
/// is 2 × 10⁻⁸ — while `x0` is an ordinary page coordinate near 540. Their
/// quotient is **2 × 10¹⁰**, and an `f32`'s last exactly representable integer
/// is 2²⁴ ≈ 1.7 × 10⁷. The `.floor()` is then applied to a number that has
/// already lost its integer part, and the snapped origin comes back quantised
/// to tens of `f32` ULPs.
///
/// Measured before it was fixed: from about 10⁷ % the region stopped
/// shrinking and floored at 2.4414 × 10⁻³ × 3.0213 × 10⁻³ pt — **fifty
/// thousand times** the 4.8 × 10⁻⁸ × 6.2 × 10⁻⁸ the viewport actually showed.
/// The raster was still produced and `drawn=1` was still traced, so every
/// existing check passed; what the operator saw was a fraction of one texel
/// stretched across the window, which reads as blank paper.
///
/// The magnitudes here are the reason. This function mixes an **absolute
/// page position** with a **relative extent**, and at deep zoom those differ by
/// ten orders of magnitude — which is exactly the shape `f32` cannot hold. The
/// rest of the region path was already `f64` (`page_region` returns a `f64`
/// rect, `RenderKey` stores `f64` bits); this was the one narrowing left, and
/// it was narrowing the value the whole tier exists to compute.
#[must_use]
pub fn region_for(visible: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (x0, y0, x1, y1) = visible;
    let w = (x1 - x0).abs();
    let h = (y1 - y0).abs();
    if !(w.is_finite() && h.is_finite()) || w <= 0.0 || h <= 0.0 {
        return visible;
    }
    // Snap the window to a half-viewport grid, then grow from there. Snapping
    // after growing would move the margin around instead of the window.
    //
    // It is the window's CENTRE that lands on the grid, not its origin.
    // `.floor()` on the origin only ever moves the window one way — towards the
    // page origin, by up to a whole step — so the overscan added around it is
    // spent off the left and top and the right and bottom are left with as
    // little as nothing. `.round()` on the centre is bounded by half a step in
    // EITHER direction, which is what makes the margin symmetric. See this
    // function's header for the measurement and for the operator's report.
    let step_x = w * 0.5;
    let step_y = h * 0.5;
    // `step` and half the window are the same number today, and they are
    // still written as two ideas: the grid step is the *cadence* (how far the
    // operator may travel before a redraw) and the half-window is the
    // *geometry* (where the window's centre sits relative to its origin). Only
    // the first would move if the redraw cadence were ever retuned, and a
    // reader who saw one name would have to work out which of the two it meant.
    let snapped_x = ((x0 + w * 0.5) / step_x).round() * step_x - w * 0.5;
    let snapped_y = ((y0 + h * 0.5) / step_y).round() * step_y - h * 0.5;
    overscanned((snapped_x, snapped_y, snapped_x + w, snapped_y + h))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The A1 sheet this project's benchmark and fixtures are built from.
    const A1_LONG_PT: f32 = 1584.0;

    /// **The zoom the operator uses today does not change tiers.**
    #[test]
    fn every_zoom_the_shell_offers_today_still_rasterizes_the_whole_page() {
        for zoom in crate::viewer::ZOOM_LADDER {
            assert_eq!(
                for_page((A1_LONG_PT, 1100.0), *zoom, Ink::Additive),
                Strategy::WholePage,
                "zoom {zoom} left the whole-page tier on an A1 sheet"
            );
        }
    }

    /// …and the switch happens where the raster would actually fail, rather
    /// than at a number somebody chose.
    #[test]
    fn the_switch_is_the_pixmap_ceiling() {
        #[allow(clippy::cast_precision_loss, reason = "16384 is exact in f32")]
        let ceiling = (pdfcer_render::MAX_PIXMAP_EDGE - 1) as f32;
        let exact = ceiling / A1_LONG_PT;

        assert_eq!(
            for_page((A1_LONG_PT, 1100.0), exact, Ink::Additive),
            Strategy::WholePage
        );
        assert_eq!(
            for_page((A1_LONG_PT, 1100.0), exact * 1.01, Ink::Additive),
            Strategy::Region,
            "just past the ceiling the region tier must take over"
        );
    }

    /// **A smaller page keeps free panning for longer**, which is the reason
    /// the switch is a pixmap size and not a zoom percentage.
    #[test]
    fn a_smaller_page_survives_to_a_higher_zoom() {
        let big = (A1_LONG_PT, 1100.0);
        let small = (306.0_f32, 396.0); // a quarter-letter slip
        let zoom = 20.0_f32; // 2,000 %

        assert_eq!(for_page(big, zoom, Ink::Additive), Strategy::Region);
        assert_eq!(
            for_page(small, zoom, Ink::Additive),
            Strategy::WholePage,
            "a small page should not be pushed into the region tier by a zoom it can afford"
        );
    }

    /// The display scale is already inside `raster_scale`, so a 150 % display
    /// reaches the ceiling at two-thirds the zoom. Asserted because it is the
    /// kind of thing that is correct by accident and then broken by a
    /// refactor that "tidies up" the units.
    #[test]
    fn the_display_scale_moves_the_switch() {
        let page = (A1_LONG_PT, 1100.0);
        #[allow(clippy::cast_precision_loss, reason = "16384 is exact in f32")]
        let ceiling = (pdfcer_render::MAX_PIXMAP_EDGE - 1) as f32;
        let zoom_at_1x = ceiling / A1_LONG_PT;

        // The same zoom on a 1.5x display is 1.5x the raster scale.
        assert_eq!(
            for_page(page, zoom_at_1x, Ink::Additive),
            Strategy::WholePage
        );
        assert_eq!(
            for_page(page, zoom_at_1x * 1.5, Ink::Additive),
            Strategy::Region
        );
    }

    /// [`whole_page_raster_fits`] is the pixmap ceiling and **the pixmap
    /// ceiling only** — it agrees with [`for_page`] about the wall and has no
    /// opinion about anything else.
    #[test]
    fn the_hard_ceiling_is_the_same_wall_for_page_finds() {
        #[allow(clippy::cast_precision_loss, reason = "16384 is exact in f32")]
        let ceiling = (pdfcer_render::MAX_PIXMAP_EDGE - 1) as f32;
        let exact = ceiling / A1_LONG_PT;

        assert!(
            whole_page_raster_fits((A1_LONG_PT, 1100.0), exact),
            "the scale that exactly reaches the ceiling must still fit"
        );
        assert!(
            !whole_page_raster_fits((A1_LONG_PT, 1100.0), exact * 1.01),
            "past the ceiling the whole-sheet raster cannot be allocated"
        );
        // And the two agree, which is the property that keeps the canvas's tier
        // choice and the strip's order gate from drifting apart.
        for scale in [exact * 0.5, exact, exact * 1.01, exact * 100.0] {
            assert_eq!(
                whole_page_raster_fits((A1_LONG_PT, 1100.0), scale),
                for_page((A1_LONG_PT, 1100.0), scale, Ink::Additive) == Strategy::WholePage,
                "the two disagreed about the wall at scale {scale}"
            );
        }
    }

    /// **An ink page that `for_page` sends to the region tier still FITS.**
    #[test]
    fn an_ink_page_pushed_to_the_region_tier_still_fits_whole() {
        // A scale where the ink ceiling bites and the pixmap one does not: a
        // tiny byte budget forces `will_composite_in_cmyk` to refuse, while the
        // page is only a few thousand pixels across.
        let page = (612.0_f32, 792.0);
        let scale = 4.0_f32;
        assert_eq!(
            for_page(page, scale, Ink::Subtractive(Some(1))),
            Strategy::Region,
            "a 1-byte ink budget must push this page off the whole-page tier"
        );
        assert!(
            whole_page_raster_fits(page, scale),
            "...and its whole-sheet raster must still be allocatable, which is \
             why the strip's gate asks this and not for_page"
        );
    }

    /// Degenerate input never answers `Region`, because a nonsense rectangle
    /// would then be handed to the renderer.
    #[test]
    fn degenerate_input_falls_back_to_the_whole_page() {
        for bad in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
            assert_eq!(
                for_page((A1_LONG_PT, 1100.0), bad, Ink::Additive),
                Strategy::WholePage
            );
            assert_eq!(
                for_page((bad, bad), 1.0, Ink::Additive),
                Strategy::WholePage
            );
            // And the order gate says "orderable" on the same input, for the
            // same reason: the whole-page path refuses a degenerate page with
            // its own sentence, and answering "it does not fit" here would
            // instead make a strip page silently claim it was zoomed past when
            // its real problem is a broken box.
            assert!(whole_page_raster_fits((A1_LONG_PT, 1100.0), bad));
            assert!(whole_page_raster_fits((bad, bad), 1.0));
        }
    }

    /// **Panning does not redraw continuously** — the property the
    /// operator's constraint turns on.
    ///
    /// He refused *"the affect that other readers have where you always have to
    /// wait for detail to render after panning"*. A region that changed on every
    /// pixel would do exactly that, because every request would miss the cache.
    /// [`region_for`]'s header states the bound it offers instead: **at most one
    /// redraw per half viewport of travel**, so a pan of one whole viewport may
    /// ask for at most three distinct rasters — the one it started in, and one
    /// for each grid line it crosses.
    ///
    ///
    ///
    /// ```text
    /// let base = region_for((1000.0, 1000.0, 1800.0, 1600.0));
    /// for (dx, dy) in [(80.0, 0.0), …] {          // a tenth of a viewport
    ///     assert_eq!(base, region_for(base moved by dx, dy));
    /// }
    /// ```
    ///
    /// — one base position, four small pans, asserting the rect never changes.
    /// **No snapping implementation can satisfy that**, because a small pan that
    /// happens to cross a grid line must change the rect; that is what a grid
    /// *is*. It passed only because `1000, 1000` happened to sit mid-cell under
    /// the old grid.
    ///
    /// Measured rather than argued, because it is the difference between a
    /// test that was over-specified and a change that broke something. Sweeping
    /// the base across a full grid step in 400 increments and applying the same
    /// four pans, **the old implementation fails its own assertion in 320 of the
    /// 1,600 cases** — and the new one fails it in 320 as well. Identical. The
    /// assertion was never describing behaviour that distinguished them; it was
    /// describing where `1000, 1000` fell.
    ///
    /// ⇒ So it is replaced by the bound the header actually promises, asserted
    /// over every phase. Both implementations satisfy it (measured: at most 3
    /// for each), which is the point — the cache-hit cadence is exactly what it
    /// was, and the centring changed only *which* side the margin lands on.
    #[test]
    fn panning_a_whole_viewport_asks_for_at_most_three_rasters() {
        let (w, h) = (800.0_f64, 600.0_f64);
        // Every phase of the snap grid, because the phase is the whole
        // variable — see the header above on what testing one costs.
        for phase in 0..200 {
            let base = 1000.0 + f64::from(phase) * (w * 0.5) / 200.0;
            let mut seen: Vec<(f64, f64, f64, f64)> = Vec::new();
            // One viewport of travel, sampled every hundredth of it — fine
            // enough that a rect appearing and vanishing between samples would
            // have to live for under 8 points at this size.
            for i in 0..=100 {
                let x0 = base + f64::from(i) * w / 100.0;
                let r = region_for((x0, 1000.0, x0 + w, 1000.0 + h));
                if !seen.contains(&r) {
                    seen.push(r);
                }
            }
            assert!(
                seen.len() <= 3,
                "panning one viewport from phase {phase} asked for {} distinct rasters — the \
                 grid step is half a viewport, so three is the most a full viewport of travel \
                 can cross, and more than that is the continuous redraw the operator refused",
                seen.len()
            );
        }
    }

    /// …and most small pans reuse the raster outright.
    #[test]
    fn most_small_pans_reuse_the_raster() {
        let (w, h) = (800.0_f64, 600.0_f64);
        let pans = [(80.0, 0.0), (0.0, 60.0), (40.0, 30.0), (-40.0, -30.0)];
        let mut reused = 0_usize;
        let mut total = 0_usize;
        for phase in 0..400 {
            let t = f64::from(phase) / 400.0;
            let (x0, y0) = (1000.0 + t * w * 0.5, 1000.0 + t * h * 0.5);
            let base = region_for((x0, y0, x0 + w, y0 + h));
            for (dx, dy) in pans {
                let moved = region_for((x0 + dx, y0 + dy, x0 + w + dx, y0 + h + dy));
                total += 1;
                if base == moved {
                    reused += 1;
                }
            }
        }
        let fraction = reused as f64 / total as f64;
        assert!(
            fraction >= 0.75,
            "only {fraction:.3} of view positions reuse the raster across a tenth-of-a-viewport \
             pan ({reused}/{total}) — the snap has stopped quantising and the operator is \
             waiting for a redraw on every pan, which is what he refused"
        );
    }

    /// …and a large pan does ask for a new one, or the operator would be looking
    /// at a raster that no longer covers the window.
    #[test]
    fn a_pan_past_the_margin_asks_for_a_new_rectangle() {
        let base = region_for((1000.0, 1000.0, 1800.0, 1600.0));
        let far = region_for((2000.0, 1000.0, 2800.0, 1600.0));
        assert_ne!(base, far);
    }

    /// **The raster is bounded by the WINDOW, not by the zoom** — which is
    /// the whole reason the region tier exists and the answer to the operator's
    /// `MAX_PIXMAP_EDGE` failure at 2382 %.
    #[test]
    fn the_region_raster_stays_window_sized_at_any_zoom() {
        let viewport_px = 1400.0_f32;
        for zoom in [1.0_f32, 23.82, 1_000.0, 100_000.0] {
            // What the operator can see, in page points, at this zoom.
            let visible_pt = viewport_px / zoom;
            let r = region_for((0.0, 0.0, f64::from(visible_pt), f64::from(visible_pt)));
            let device = (r.2 - r.0) * f64::from(zoom);
            assert!(
                device <= f64::from(pdfcer_render::MAX_PIXMAP_EDGE),
                "at {zoom}x the region would be {device} px, past the {} cap",
                pdfcer_render::MAX_PIXMAP_EDGE
            );
            // …and it is the same size at every zoom, being 2x the window.
            assert!(
                (device - f64::from(viewport_px) * 2.0).abs() < 1.0,
                "{device} at {zoom}x"
            );
        }
    }

    /// The overscan grows the rect on every side, by the documented fraction.
    #[test]
    fn the_overscan_grows_every_side_by_half_a_viewport() {
        let (x0, y0, x1, y1) = overscanned((100.0, 200.0, 300.0, 400.0));
        // 200 wide, 200 tall; half of each is 100.
        assert!((x0 - 0.0).abs() < 0.001);
        assert!((y0 - 100.0).abs() < 0.001);
        assert!((x1 - 400.0).abs() < 0.001);
        assert!((y1 - 500.0).abs() < 0.001);
    }

    /// **The overscanned rect is a pure function of the visible rect**, so
    /// two requests for the same view ask for the same rectangle.
    #[test]
    fn the_same_view_always_asks_for_the_same_rectangle() {
        let view = (12.5, 33.25, 812.5, 633.25);
        assert_eq!(overscanned(view), overscanned(view));
    }

    // =======================================================================
    // The MARGIN — what is sharp beyond the edge of the view
    // =======================================================================

    /// The smallest gap, on any of the four sides, between the view and the
    /// edge of the region that will be rasterized — expressed as a **fraction
    /// of the viewport**, which is the unit [`OVERSCAN`]'s own table is
    /// written in.
    fn margin_fraction(visible: (f64, f64, f64, f64)) -> f64 {
        let (x0, y0, x1, y1) = visible;
        let (w, h) = (x1 - x0, y1 - y0);
        let r = region_for(visible);
        // Four gaps, each normalised by the viewport extent it is measured
        // along. A negative value would mean the region does not even reach
        // the view; zero means it stops exactly at the edge.
        [
            (x0 - r.0) / w,
            (r.2 - x1) / w,
            (y0 - r.1) / h,
            (r.3 - y1) / h,
        ]
        .into_iter()
        .fold(f64::INFINITY, f64::min)
    }

    /// **[`OVERSCAN`]'s promise holds in every direction, not just two of
    /// them.**
    #[test]
    fn the_overscan_reaches_a_quarter_screen_in_every_direction() {
        // An ordinary page coordinate and an ordinary viewport, in points at
        // some region-tier zoom. The numbers are deliberately not round: a
        // grid-aligned view is the one phase that cannot fail.
        let (w, h) = (48.3_f64, 37.1_f64);
        let (step_x, step_y) = (w * 0.5, h * 0.5);
        let mut worst = f64::INFINITY;
        let mut worst_at = (0.0_f64, 0.0_f64);
        for i in 0..200 {
            let t = f64::from(i) / 200.0;
            let x0 = 217.4 + t * step_x;
            let y0 = 361.9 + t * step_y;
            let m = margin_fraction((x0, y0, x0 + w, y0 + h));
            if m < worst {
                worst = m;
                worst_at = (x0, y0);
            }
        }
        assert!(
            // A quarter, less a hair for `f64`. The snap quantises the
            // window to a half-viewport grid, so half a screen on all four
            // sides at once is not achievable at every phase by ANY
            // implementation that keeps the raster at 2× the window: the
            // quarter is the arithmetic ceiling, not this implementation's
            // limit. The threshold pins the guarantee rather than the
            // arithmetic, and what it excludes is the case the operator
            // reported — a side with essentially no margin at all.
            worst >= 0.245,
            "the sharp raster reaches only {:.4} of a viewport past the view at its worst phase \
             (view origin {:.3},{:.3}) — OVERSCAN's table promises half a screen in ANY \
             direction, and a side with no margin is the low-resolution backdrop showing along \
             the edge of the window",
            worst,
            worst_at.0,
            worst_at.1
        );
    }

    /// O24i — **the region must keep shrinking all the way to the
    /// ceiling.**
    #[test]
    fn the_region_stays_proportional_to_the_view_at_every_depth() {
        // A page coordinate far from the origin, which is the whole
        // difficulty: near zero even `f32` would cope.
        let at = 540.158_756_f64;
        for zoom in [1.0e3_f64, 1.0e5, 1.0e7, 1.0e9, 1.0e10, 1.0e12] {
            let w = 484.0 / zoom;
            let h = 619.0 / zoom;
            let r = region_for((at, at, at + w, at + h));
            let got_w = r.2 - r.0;
            let want_w = w * (1.0 + 2.0 * OVERSCAN);
            assert!(
                // A part in a thousand, and the slack is `f64`'s own.
                //
                // The extent is computed as `(x0 + w) - x0` at an absolute
                // position near 540, where an `f64` ULP is 1.1e-13. At a
                // trillion percent `w` is 1e-9 pt — about 8,800 ULPs — so the
                // subtraction returns a relative error near 1e-4 and no
                // implementation can do better while the position is absolute.
                //
                // Which is also the real ceiling of this design, worth
                // stating: 8,800 representable steps across a 484-pixel
                // viewport is 18 per pixel, so the arithmetic is still
                // comfortable at the maximum zoom the shell offers. The tier
                // below it ran out at 2^24; this one has room left.
                (got_w / want_w - 1.0).abs() < 1e-3,
                "at zoom {zoom:e} the region is {got_w:e} pt wide, {:.1}x the {want_w:e} the \
                 overscanned view needs",
                got_w / want_w
            );
        }
    }

    /// …and the snapped origin must stay WITHIN one grid step of the view.
    #[test]
    fn the_snapped_origin_stays_next_to_the_view_at_every_depth() {
        let at = 540.158_756_f64;
        for zoom in [1.0e3_f64, 1.0e5, 1.0e7, 1.0e9, 1.0e10, 1.0e12] {
            let w = 484.0 / zoom;
            let h = 619.0 / zoom;
            let r = region_for((at, at, at + w, at + h));
            // The snap floors to a half-view grid and the overscan then grows
            // by half a view, so the origin can legitimately sit one and a
            // half views below the view's own. Anything beyond that is the
            // origin having been corrupted rather than quantised.
            let slack = w * 1.5 + h * 1.5;
            assert!(
                (at - r.0).abs() <= slack && (at - r.1).abs() <= slack,
                "at zoom {zoom:e} the region starts at ({:e}, {:e}), {:e} pt from the view at \
                 {at} — the snap has lost the coordinate rather than quantised it",
                r.0,
                r.1,
                (at - r.0).abs().max((at - r.1).abs())
            );
        }
    }

    // =======================================================================
    // The ink ceiling — the operator's "colours change with zoom"
    // =======================================================================

    /// A4 in points, which is what every figure the engine published about this
    /// ceiling is stated against.
    const A4: (f32, f32) = (595.0, 842.0);

    /// **An additive page is not touched by any of this.**
    #[test]
    fn an_additive_page_ignores_the_colour_ceiling_entirely() {
        assert_eq!(
            for_page(A4, 12.0, Ink::Additive),
            Strategy::WholePage,
            "a page that is not blended in ink must reach the pixmap ceiling and nothing else"
        );
    }

    /// **A page blended in ink switches to the region tier at the colour
    /// ceiling**, which is far below the pixmap one.
    #[test]
    fn a_page_blended_in_ink_leaves_the_whole_page_tier_at_the_colour_ceiling() {
        // Walk up in fine steps and find where the answer changes. Walking
        // rather than probing two chosen points: two samples either side of a
        // transition look exactly like no transition at all if the transition
        // is not where it was assumed to be.
        let mut switch = None;
        let mut scale = 1.0_f32;
        while scale < 20.0 {
            if for_page(A4, scale, Ink::Subtractive(None)) == Strategy::Region {
                switch = Some(scale);
                break;
            }
            scale += 0.01;
        }
        let switch = switch.expect(
            "a subtractive A4 page must leave the whole-page tier somewhere below 2000 % zoom",
        );

        // The engine's published figure for real A4 at the default ceiling is
        // about 518 %. This asserts the neighbourhood rather than the digits:
        // the exact value is the engine's to move, and pinning it here would be
        // the copy this design refuses to make.
        assert!(
            (4.8..5.4).contains(&switch),
            "a subtractive A4 page left the whole-page tier at {:.0} %, and the engine's default \
             ceiling is about 518 %",
            switch * 100.0
        );

        assert_eq!(
            for_page(A4, switch - 0.02, Ink::Subtractive(None)),
            Strategy::WholePage,
            "the step below the switch must still be whole-page, or the search found a cliff \
             that is not the one this test is about"
        );
        assert_eq!(
            for_page(A4, switch * 3.0, Ink::Additive),
            Strategy::WholePage,
            "the same raster is comfortably inside the PIXMAP ceiling, which is what makes the \
             band this repair closes exist at all"
        );
    }

    /// **Raising the operator's ceiling moves the switch up**, which is the
    /// entire point of the setting existing.
    #[test]
    fn a_larger_ceiling_keeps_the_whole_page_tier_for_longer() {
        let switch_for = |max_bytes: Option<usize>| {
            let mut scale = 1.0_f32;
            while scale < 40.0 {
                if for_page(A4, scale, Ink::Subtractive(max_bytes)) == Strategy::Region {
                    return scale;
                }
                scale += 0.01;
            }
            f32::INFINITY
        };
        let default = switch_for(None);
        let half_gig = switch_for(Some(512 * 1024 * 1024));
        let one_gig = switch_for(Some(1024 * 1024 * 1024));
        let two_gig = switch_for(Some(2048 * 1024 * 1024));

        assert!(
            default < half_gig && half_gig < one_gig && one_gig < two_gig,
            "raising the ceiling must postpone the switch: default {default:.2}, 512 MiB \
             {half_gig:.2}, 1 GiB {one_gig:.2}, 2 GiB {two_gig:.2}"
        );
        // And the ordering is not merely strict — it is worth something. Double
        // the memory buys √2 the linear scale, because a raster is
        // two-dimensional; a build that moved the switch by a rounding error
        // would satisfy the ordering above and would be useless.
        assert!(
            one_gig > default * 1.5,
            "four times the default ceiling should buy about twice the linear scale, not \
             {:.2}x",
            one_gig / default
        );
    }

    /// A ceiling so small that no useful raster fits still answers, and answers
    /// the safe way.
    #[test]
    fn an_absurdly_small_ceiling_falls_to_the_region_tier_rather_than_failing() {
        assert_eq!(
            for_page(A4, 1.0, Ink::Subtractive(Some(0))),
            Strategy::Region
        );
    }

    /// Degenerate input is still refused before the ink question is reached.
    #[test]
    fn degenerate_input_is_refused_before_the_ink_ceiling_is_consulted() {
        for bad in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
            assert_eq!(
                for_page(A4, bad, Ink::Subtractive(None)),
                Strategy::WholePage,
                "a scale of {bad} reached the ink predicate"
            );
            assert_eq!(
                for_page((bad, bad), 1.0, Ink::Subtractive(None)),
                Strategy::WholePage,
                "a page of {bad} pt reached the ink predicate"
            );
        }
    }
}
