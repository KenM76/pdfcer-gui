//! # `viewer::ceiling` — how far this page can actually be zoomed
//!
//! `OPERATOR_REQUESTS.md` **O24**. Three limits bind at three different
//! depths, and keeping them in one file is what stops a caller reconciling
//! them differently from another:
//!
//! | limit | what it is | where it bites |
//! |---|---|---|
//! | [`max_zoom_for_page`] | the whole-page raster exceeds `MAX_PIXMAP_EDGE` | ~1,000 % on a large sheet |
//! | [`SUB_PIXEL_CONTENT_EXTENT`] | the `f32` scroll offset can no longer hold the point under the cursor | ~86,000 % on a 1,224 pt sheet, ~132,000 % on US Letter |
//! | the operator's own setting | whatever he asked for | wherever he says |
//!
//! ★★ The first is not a limit at all once the region tier can render past
//! it — the raster becomes window-sized and the page's size stops entering
//! the arithmetic. The second is, and is the one that decides what the shell
//! can honestly offer today; `viewer::deep::DeepAnchor` is what raises it,
//! and [`crate::canvas::viewpos`] hands the position over to it at exactly
//! this extent.
//!
//! ## Why this is its own file
//!
//! R2's 1,500-line ceiling forced the split when the positional cap landed,
//! and the seam is real: everything here answers one question — *how far may
//! this page be magnified?* — where the rest of [`super`] answers *where is
//! the view and what is it showing?*

// ★ `max_zoom_for_page` stays in [`super`] with the rest of the raster-side
// arithmetic and its own tests, and is imported rather than moved: it answers
// a question about a PIXMAP, where everything in this file answers one about
// how far the operator may go. Moving it would have dragged its tests across a
// seam they do not belong on.
use super::{MAX_ZOOM, MIN_ZOOM, max_zoom_for_page, zoom_for_raster_scale};
/// The content extent at which the position model hands over from `egui`'s
/// `f32` scroll offset to [`crate::canvas::deep::DeepAnchor`] — `2^20`.
///
/// One unit of content space is one screen pixel, so the spacing between
/// representable `f32` offsets **is** the positioning error. `2^20` puts one
/// step at 0.125 px.
///
/// # ★★★ It gates HOLDING A POINT, not ADDRESSING A PIXEL
///
/// The two requirements part company as the zoom rises, and reading this
/// constant as the second is the mistake that sets it 16× too high:
///
/// | | |
/// |---|---|
/// | the `f32` error, in PAGE POINTS | `page_pt × 2^-23` — **constant**, because the offset grows with the zoom and the division by zoom cancels |
/// | what "holding the point under the cursor" allows | a fraction of the VIEWPORT in page points — `viewport_px / zoom` — which **shrinks** |
///
/// So there is a crossing, it is far below the point at which an offset stops
/// addressing every pixel, and past it the view drifts off the cursor while
/// still addressing every pixel perfectly.
///
/// ★★ Measured through the running binary rather than derived, on
/// `SW41177.pdf` (1,224 pt tall):
/// `zooming_does_not_throw_away_where_the_operator_panned` failed reproducibly
/// at notch 7 of stage 5, between 292,415 % and 357,156 % — a content extent
/// near **3.6 million**, where one `f32` step is 0.43 px and seven wheel
/// notches had accumulated 19 px of drift against a tolerance of 8. `2^20` is
/// 3.4× finer than the point that failed, and hands over at 85,700 % on that
/// sheet and 132,400 % on US Letter.
///
/// ★ Drawing is not what limits this. Driving to the top of the setting on a
/// US Letter page drew at a content extent of 20.5 billion — a 2,048 px step —
/// and stopped at 41 billion. Usability gives out four orders of magnitude
/// earlier, and this is that point.
///
/// ★ `pub` because [`crate::canvas::geometry`] bounds the pasteboard against
/// it as well. The pasteboard grows with the zoom (an overhang measured in
/// points, multiplied by the scale), so without a bound tied to THIS number
/// the scroll content could pass the hand-over point while the strip itself
/// was still comfortably below the tier boundary — the position model would
/// have handed over late, and silently. One constant, both uses.
pub const SUB_PIXEL_CONTENT_EXTENT: f32 = 1_048_576.0;

// ★★★ **A constant that changes what it gates has to be re-derived, not
// re-tuned.** This one was a cap on where an `f32` offset stops addressing
// every pixel before it became a tier hand-over threshold, and the two
// questions have answers 16× apart. `OPERATOR_REQUESTS.md` **O49**.

/// The highest zoom this page can reach **when the region tier is
/// available** — `OPERATOR_REQUESTS.md` O24.
///
/// # ★★ Why this is a different function rather than a flag on the old one
///
/// [`max_zoom_for_page`] answers a question about a **pixmap**: how far can
/// this page be magnified before its whole-page raster exceeds
/// `MAX_PIXMAP_EDGE`? That question is real and its answer is a genuine
/// ceiling — *for the whole-page tier*.
///
/// It is simply **not the question** once the renderer can be asked for a
/// region. There the pixmap is the size of the window, so the page's own
/// size stops entering the arithmetic at all and the only remaining limit is
/// whatever the operator has said they want. Two different questions with
/// two different answers are two functions; adding a boolean to the first
/// would have produced one function whose name describes only half of what
/// it does.
///
/// # ★ It is dormant, and deliberately so
///
/// Nothing calls this yet. It lands ahead of the canvas change that will,
/// so that the arithmetic can be reviewed and tested while it cannot affect
/// a running build — the same staging the render worker's `region` field
/// took.
///
/// `limit` is the operator's own maximum, which becomes a setting. Clamped
/// to at least [`MIN_ZOOM`] so a nonsensical stored value cannot make the
/// document unzoomable.
#[must_use]
pub fn max_zoom_with_regions(limit: f32) -> f32 {
    if limit.is_finite() && limit >= MIN_ZOOM {
        limit
    } else {
        MIN_ZOOM
    }
}

/// **The zoom ceiling in force**, given the operator's configured maximum.
///
/// ★★ The ONE place the two tiers are reconciled, so the two call sites that
/// need a ceiling — `app::actions::apply` and `canvas::zoom` — cannot answer
/// the question differently. Their own comments already note that each derives
/// this per action rather than caching it; deriving it *differently* is the
/// failure that would follow.
///
/// The rule is one sentence: **the whole-page raster limit binds only while the
/// operator has not asked to go past it.** Below their maximum the pixmap
/// ceiling is real and is what stops them; above it, the region tier takes over
/// and the page's size stops entering the arithmetic at all.
///
/// `limit_percent` is [`crate::app::prefs::Prefs::max_zoom_percent`]. Passing
/// the shipped default reproduces the old behaviour exactly, which is what
/// keeps a fresh install unchanged.
///
/// # ★★★ `learned_raster_scale` — the THIRD ceiling, and the only one that may
/// override the operator — O186
///
/// `None` on every page of every document until a render has actually been
/// refused for a raster limit, and on that page it is
/// [`crate::render::ceiling::RasterCeiling::for_page`]'s answer: **a raster
/// scale, in device pixels per PDF point**, not a zoom. Converted here by
/// [`zoom_for_raster_scale`], which is the exact inverse of
/// [`crate::viewer::raster_scale`] because both run through one
/// [`crate::viewer::raster_density`] — the display density **and** the
/// operator's render quality. Dividing by the density alone is O218: on
/// Sharper it returns a ceiling half again too high, so the clamp that exists
/// to rescue him hands the engine another pixmap it refuses.
///
/// It is applied as a hard `min` *after* everything above, and that ordering is
/// the whole point. The two derived ceilings are predictions about what the
/// renderer will accept; this one is a **measurement of what it refused**. The
/// operator's `max_zoom_percent` deliberately has no upper bound — he asked for
/// a trillion percent and got it — and above the region tier the page's size
/// stops entering the arithmetic, so *nothing else in this function can stop a
/// page that physically cannot be rasterized further*. That is O186: he met the
/// wall, and the wall was reported to him as an error painted across his
/// drawing.
///
/// ★ So this clause, uniquely, binds below a number the operator typed. That is
/// not the shell overruling him — it is the shell declining to re-offer a zoom
/// it has already watched fail. His own ruling: *"zoom should stop at the limit
/// and not end up showing an error"*. The reason it stopped is disclosed on the
/// bottom bar by `crate::app::status::rasterstop`, which is the other half of
/// the same sentence and is why a silent clamp here is honest rather than
/// mysterious.
///
/// ★★ Floored at [`MIN_ZOOM`], so a learned ceiling can never make a document
/// unzoomable. `max_zoom_with_regions` already makes that guarantee for the
/// operator's setting and the same guarantee is owed here, for the stronger
/// reason that this value was not chosen by anyone.
#[must_use]
pub fn zoom_ceiling(
    page_pts: (f32, f32),
    pixels_per_point: f32,
    quality: crate::app::prefs::RenderQuality,
    limit_percent: f32,
    learned_raster_scale: Option<f32>,
) -> f32 {
    let limit = max_zoom_with_regions(limit_percent / 100.0);
    let whole_page = max_zoom_for_page(page_pts, pixels_per_point, quality);

    // ★★★ THE DEFAULT MUST CHANGE NOTHING, and a plain `max` breaks that.
    //
    // `max_zoom_for_page` can fall BELOW `MAX_ZOOM` on a large page at a high
    // display scale — an A1 sheet at 1.5x tops out at 690 %, not 800 % —
    // because the pixmap ceiling bites first. A plain `limit.max(whole_page)`
    // would then raise that page's ceiling to the operator's default of 800 %
    // and rasterize a pixmap the engine refuses.
    //
    // Caught by `the_default_setting_reproduces_the_old_ceiling_exactly`, which
    // is exactly why that test walks three page sizes and three display scales
    // rather than one of each.
    //
    // So the region tier is only allowed to lift the ceiling when the operator
    // has asked for MORE THAN THE SHIPPED DEFAULT. Below that, nothing about
    // the old behaviour is touched — which is the property that makes this
    // feature safe to land.
    // ★★★ The lift is bounded BELOW by `MAX_ZOOM`, not by the default.
    //
    // The first version compared against `DEFAULT_MAX_ZOOM_PERCENT`, which
    // worked while the default was 800 % and became a no-op the moment the
    // operator raised the default to the maximum — the ceiling would then have
    // been the whole-page limit always, and the setting inert everywhere. A
    // guard phrased in terms of a value that can move is a guard that stops
    // guarding when it moves.
    //
    // `MAX_ZOOM` is the right bound because it is what the SHELL offered before
    // any of this: below it, `max_zoom_for_page`'s pixmap ceiling is a real
    // constraint and must keep binding — an A1 sheet at a 1.5x display tops out
    // at 690 %, and lifting that would ask the engine for a raster it refuses.
    // Above it, the region tier can render and the page's size stops mattering.
    // ★★★ THE POSITIONAL CAP IS GONE, because tier 3 replaced it — O24.
    //
    //

    // ★★★ AND NOTHING CAPS IT ANY MORE. Two ceilings stood here today and both
    // are gone, each removed by finding where the precision was ACTUALLY lost:
    //
    // The sub-pixel cap went when `viewer::deep::DeepAnchor` took the position
    // off the `f32` scroll offset. The strip-extent cap went when the drawn
    // rect stopped being derived from the page's own screen rect — a value with
    // a magnitude around 10^12 px at deep zoom, where `f32`'s spacing is 131,072
    // px and the thing being drawn is 1,400 px across.
    //
    // ★ Both fixes are the same move: **do not form the large number**. Neither
    // needed a wider type anywhere it would cost anything, which is why the
    // answer to "keep the 32-bit strip or build a 64-bit one?" turned out to be
    // neither — carrying a huge intermediate more precisely is worse than not
    // computing it, and it would have left two layout paths to keep in step.
    //
    // Measured after both: a page DRAWN at 10^12 % — the figure the operator
    // named — on US Letter, with no failed rasters.

    let derived = limit.max(whole_page.min(MAX_ZOOM));

    // ★ O186. `None` must change this expression into the identity, which is
    // what `map_or(derived, ...)` says and what
    // `a_page_that_has_refused_nothing_keeps_its_derived_ceiling` pins — the
    // property that makes this safe to add to a shipped ceiling.
    learned_raster_scale.map_or(derived, |scale| {
        // A non-finite or non-positive scale is not a ceiling. `RasterCeiling`
        // refuses to learn one, so this cannot arrive from it; the guard is
        // here because this function is `pub` and a caller that computed a
        // scale some other way must not be able to pin the zoom to zero.
        if !scale.is_finite() || scale <= 0.0 {
            return derived;
        }
        let learned = zoom_for_raster_scale(scale, pixels_per_point, quality);
        derived.min(learned).max(MIN_ZOOM)
    })
}

/// Whether this page at this zoom needs the `f64` position model — O24 tier 3.
///
/// True exactly where an `f32` scroll offset stops placing the view to within a
/// screen pixel, which is [`SUB_PIXEL_CONTENT_EXTENT`]. Below it the scroll
/// area is authoritative and nothing about the canvas changes; above it
/// [`super::deep::DeepAnchor`] is.
///
#[must_use]
pub fn deep_position_needed(page_pts: (f32, f32), zoom: f32) -> bool {
    let longest = page_pts.0.max(page_pts.1);
    longest.is_finite()
        && zoom.is_finite()
        && longest > 0.0
        && zoom > 0.0
        && longest * zoom > SUB_PIXEL_CONTENT_EXTENT
}

/// # Tests — the three ceilings, and the ladder that has to be able to reach them
///
/// ★★ **Moved here from [`super`] on 2026-09-12, and the move is the point.**
/// Every test below asks one question — *how far may this page be magnified?* —
/// which is the question this file exists to answer, and they were sitting in
/// `viewer/mod.rs` only because they predate the split that created this file.
/// `viewer/mod.rs` had reached 1,498 lines of R2's 1,500, so O186's fourth
/// parameter could not have been tested at all without finding the seam first.
/// R2's own wording: *"when a file approaches the limit, that is the signal to
/// find the seam, not to raise the limit."* The seam was already named in this
/// file's header.
///
/// ★ What deliberately did NOT move: the tests of `super::max_zoom_for_page`,
/// `super::raster_scale` and `ViewState`'s clamping. Those answer questions
/// about a *pixmap* and about *where the view is*, and the note above this
/// file's `use super::...` line already records why that function stays in
/// [`super`] "with its own tests". Dragging them across would have made this
/// file the home of two subjects instead of one.
#[cfg(test)]
#[allow(clippy::float_cmp, reason = "ladder rungs are exact f32 literals")] // ui-text-exempt: clippy lint justification, never displayed
mod tests {
    use super::*;
    // ★ `ViewState` and `ZOOM_LADDER` are the two things these tests reach back
    // into [`super`] for, and both are reached for the same reason: a ceiling is
    // only worth anything if the control the operator actually presses can climb
    // to it. See `the_zoom_ladder_can_climb_to_a_configured_maximum` below.
    use crate::viewer::{ViewState, ZOOM_LADDER, raster_scale};
    // Every test below that is not ABOUT the render quality passes `Normal`,
    // whose multiplier is 1.0 — so each of their assertions is the same number
    // it was before the quality factor entered the arithmetic, and a failure
    // here is a failure of the thing the test names.
    use crate::app::prefs::RenderQuality;

    /// ★★★ **The ladder can actually REACH a configured maximum**, stepping.
    ///
    /// `zoom_ceiling` answering a big number is necessary and not sufficient:
    /// the `+` button walks `ZOOM_LADDER`, which ends at 8.0. If stepping
    /// stopped there the setting would be honoured by every code path except
    /// the one the operator actually uses, which is the same silently-inert
    /// control in a subtler place.
    ///
    /// ★ This is the gap `OPERATOR_REQUESTS.md` O24 predicted in its own
    /// words — *"the buttons stop working exactly where the setting starts
    /// mattering"* — asserted rather than left to be discovered.
    #[test]
    fn the_zoom_ladder_can_climb_to_a_configured_maximum() {
        let ceiling = zoom_ceiling(
            (1584.0, 1224.0),
            1.0,
            RenderQuality::Normal,
            500_000.0,
            None,
        );
        let mut zoom = 1.0_f32;
        for _ in 0..200 {
            let mut view = ViewState {
                zoom,
                ..ViewState::default()
            };
            view.zoom_in(ceiling);
            if (view.zoom - zoom).abs() < f32::EPSILON {
                break;
            }
            zoom = view.zoom;
        }
        assert!(
            zoom > 100.0,
            "stepping stalled at {zoom}x against a ceiling of {ceiling}x — the ladder \
             cannot reach the configured maximum, so the setting is inert for the +/- \
             buttons even though `zoom_ceiling` honours it"
        );
    }

    /// ★★★ **THE SETTING IS NOT DECORATIVE** — the whole risk of O24.
    ///
    /// `OPERATOR_REQUESTS.md` O24 warned in as many words that shipping the
    /// setting without the mechanism would produce *"a control that is drawn,
    /// accepted, persisted, and quietly overruled downstream"* — the operator
    /// types 100,000 % and the zoom stops near a thousand with nothing said.
    ///
    /// This is that failure, stated as an assertion. `zoom_ceiling` must
    /// answer the operator's configured maximum wherever it is higher than
    /// the whole-page raster limit, on a page large enough that the raster
    /// limit really does bind.
    #[test]
    fn a_configured_maximum_is_honoured_past_the_whole_page_raster_limit() {
        let a1 = (1584.0_f32, 1224.0);
        let whole_page = max_zoom_for_page(a1, 1.0, RenderQuality::Normal);
        assert!(
            whole_page < 20.0,
            "the premise: an A1 sheet's whole-page ceiling is around 1,000% ({whole_page})"
        );

        // ★ Below the positional cap the configured maximum is honoured
        // exactly. `10_000%` and `100_000%` are both well inside it on an A1
        // sheet, whose cap is around 1,050,000%.
        for percent in [10_000.0_f32, 100_000.0] {
            let ceiling = zoom_ceiling(a1, 1.0, RenderQuality::Normal, percent, None);
            assert!(
                (ceiling - percent / 100.0).abs() / (percent / 100.0) < 1e-6,
                "{percent}% was overruled: ceiling {ceiling}, wanted {}",
                percent / 100.0
            );
        }

        // ★★ …and a TRILLION percent is honoured too, since tier 3 wired the
        // `f64` position model. The cap that stood here until then is gone; the
        // same constant now decides when `DeepAnchor` takes over instead of
        // when to refuse.
        // ★★ …and above it the STRIP EXTENT is what binds now, not the raster
        // and not the scroll offset. Asking for a trillion percent yields the
        // deepest zoom the page is confirmed to actually draw at.
        let deep = zoom_ceiling(a1, 1.0, RenderQuality::Normal, 1e12, None);
        assert!(
            (deep - 1e10).abs() / 1e10 < 1e-6,
            "a trillion percent must be honoured in full now that nothing caps it: {deep}x"
        );
        assert!(
            deep_position_needed(a1, deep),
            "…and at that zoom the f64 anchor must be the one positioning the view"
        );
        assert!(
            deep > 1_000_000.0,
            "the cap must still be past 100,000,000%: {deep}x"
        );
    }

    /// ★★★ **The default reaches the maximum** — the operator's instruction of
    /// 2026-08-22, *"Also set the default to be able to hit the maximum zoom."*
    ///
    ///
    /// ★ The property is kept, not dropped: **what must not change is the
    /// PANNING**, which is what he actually cares about. That is asserted by
    /// `every_zoom_the_shell_offers_today_still_rasterizes_the_whole_page` in
    /// `render::strategy`, which walks the whole ladder — the ceiling is
    /// permission, and the strategy is behaviour.
    #[test]
    fn the_default_reaches_the_maximum_on_every_page_and_display_scale() {
        for page in [(1584.0_f32, 1224.0), (612.0, 792.0), (306.0, 396.0)] {
            for ppp in [1.0_f32, 1.5, 2.0] {
                // The render quality is an axis here because the default takes the
                // region tier, where the pixmap the quality scales is the WINDOW
                // and not the page — so the shipped default must reach the same
                // maximum at all three settings, and a quality factor leaking into
                // the region tier would show up as one of these rows failing.
                for quality in [
                    RenderQuality::Faster,
                    RenderQuality::Normal,
                    RenderQuality::Sharper,
                ] {
                    let ceiling = zoom_ceiling(
                        page,
                        ppp,
                        quality,
                        crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT,
                        None,
                    );
                    // ★ The default asks for the maximum and now GETS it, on every
                    // page and display scale — which is only honest because tier 3
                    // positions the view past the point an `f32` offset could.
                    // ★ The default asks for the maximum and gets the deepest the
                    // strip can still place a page at — which is what the shell can
                    // actually deliver, on every page size.
                    let wanted = crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT / 100.0;
                    assert!(
                        (ceiling - wanted).abs() / wanted < 1e-6,
                        "page {page:?} at {ppp}x: ceiling {ceiling} should be {wanted}"
                    );
                    assert!(
                        ceiling > 1_000_000.0,
                        "every page must reach past 100,000,000%: {page:?} got {ceiling}x"
                    );
                }
            }
        }
    }

    /// ★★ **…and a LOW setting still lets the pixmap ceiling bind.**
    ///
    /// The half that survives from the test this replaced, and it is the one
    /// that stops the change being dangerous: below `MAX_ZOOM` the whole-page
    /// raster limit is a real constraint — an A1 sheet at 1.5x tops out at
    /// 690 %, not 800 % — and asking past it would demand a raster the engine
    /// refuses.
    #[test]
    fn a_low_setting_does_not_lift_the_whole_page_raster_limit() {
        let a1 = (1584.0_f32, 1224.0);
        let whole_page = max_zoom_for_page(a1, 1.5, RenderQuality::Normal);
        assert!(
            whole_page < MAX_ZOOM,
            "the premise: {whole_page} < {MAX_ZOOM}"
        );

        // A setting BELOW the pixmap ceiling must not raise it…
        let ceiling = zoom_ceiling(a1, 1.5, RenderQuality::Normal, 300.0, None);
        assert!(
            (ceiling - whole_page).abs() < 1e-4,
            "a 300% setting should leave the {whole_page}x pixmap ceiling alone, got {ceiling}"
        );
    }
    /// ★★★ **The position model changes hands exactly where an `f32` offset
    /// stops being able to place the view** — O24 tier 3.
    ///
    /// One unit of content space is one screen pixel, so `2^24` content points
    /// is the last extent at which the offset is exact. Below it the scroll
    /// area is authoritative and the canvas is unchanged; above it
    /// `viewer::deep::DeepAnchor` is.
    ///
    /// ★ Asserted on both sides of the threshold, because a predicate that
    /// answered `true` everywhere would put the whole shell on the deep path —
    /// and that path is the one that has never carried ordinary use.
    #[test]
    fn the_deep_position_model_takes_over_only_past_the_sub_pixel_extent() {
        let letter = (612.0_f32, 792.0);
        let threshold = SUB_PIXEL_CONTENT_EXTENT / letter.1;

        assert!(
            !deep_position_needed(letter, threshold * 0.9),
            "below the extent the scroll offset must stay authoritative"
        );
        assert!(
            deep_position_needed(letter, threshold * 1.1),
            "above it the f64 anchor must take over"
        );

        // Every zoom the shell has ever offered stays on the ordinary path.
        for zoom in ZOOM_LADDER {
            assert!(
                !deep_position_needed(letter, *zoom),
                "zoom {zoom} left the ordinary position model"
            );
        }

        // Degenerate input never claims to need the deep path.
        for bad in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
            assert!(!deep_position_needed(letter, bad));
            assert!(!deep_position_needed((bad, bad), 1.0));
        }
    }

    /// ★★★ **The page's size stops mattering once regions are available** —
    /// O24.
    ///
    /// This is the whole point of the region tier stated as an assertion. In
    /// the whole-page tier an A0 sheet hits its ceiling far sooner than a
    /// business card, because the ceiling is a pixmap size and the page is in
    /// it. With regions the pixmap is the window, so both pages reach the same
    /// limit — the operator's.
    #[test]
    fn with_regions_the_page_size_no_longer_caps_the_zoom() {
        let huge = (3370.0_f32, 2384.0); // A0
        let tiny = (180.0_f32, 252.0); // a business card

        // Whole-page tier: the two pages have very different ceilings.
        assert!(
            max_zoom_for_page(tiny, 1.0, RenderQuality::Normal)
                > max_zoom_for_page(huge, 1.0, RenderQuality::Normal),
            "the whole-page ceiling must depend on the page's size"
        );

        // Region tier: neither page enters the arithmetic.
        let limit = 10_000.0_f32;
        assert!((max_zoom_with_regions(limit) - limit).abs() < f32::EPSILON);
    }

    /// A stored limit that is nonsense must not make the document unzoomable.
    #[test]
    fn a_broken_limit_falls_back_to_the_floor_rather_than_to_zero() {
        for bad in [f32::NAN, f32::NEG_INFINITY, -5.0, 0.0, MIN_ZOOM / 2.0] {
            assert!(
                (max_zoom_with_regions(bad) - MIN_ZOOM).abs() < f32::EPSILON,
                "{bad} should fall back to MIN_ZOOM"
            );
        }
    }

    /// ★ **Infinity is not a limit**, and is refused rather than passed
    /// through — an infinite ceiling would propagate into a scroll extent and
    /// blank the canvas, which is the failure `geometry`'s guards exist for.
    #[test]
    fn an_infinite_limit_is_refused() {
        assert!((max_zoom_with_regions(f32::INFINITY) - MIN_ZOOM).abs() < f32::EPSILON);
    }

    // ---- the learned ceiling — O186 ------------------------------------

    /// ★★★ **A page that has refused nothing keeps exactly the ceiling it had**
    /// — O186's safety property, and the only reason a third clause was safe to
    /// add to a shipped expression.
    ///
    /// `learned_raster_scale` is `None` on every page of every document until a
    /// render is actually refused for a raster limit, which for almost every
    /// file the operator opens is never. `None` must therefore be the exact
    /// identity, not approximately so.
    ///
    /// Walked across three page sizes, three display densities and three
    /// configured maxima — 27 combinations — rather than asserted at one point,
    /// because the expression it guards is a `max` of a `min` and a regression
    /// that broke only the middle rung would sail past a single-point check.
    /// That is not hypothetical:
    /// `the_default_reaches_the_maximum_on_every_page_and_display_scale` above
    /// walks a grid for exactly this reason and its own comment records that one
    /// case would have missed the defect it was written for.
    ///
    /// ★ The comparison is against the derived parts **recomputed here**, not
    /// against another call to [`zoom_ceiling`]. There is no three-argument form
    /// any more, so a test that compared the function to itself would pass under
    /// every possible change to it — including deleting the whole body.
    #[test]
    fn a_page_that_has_refused_nothing_keeps_its_derived_ceiling() {
        for page in [(1584.0_f32, 1224.0), (612.0, 792.0), (306.0, 396.0)] {
            for ppp in [1.0_f32, 1.5, 2.0] {
                for quality in [
                    RenderQuality::Faster,
                    RenderQuality::Normal,
                    RenderQuality::Sharper,
                ] {
                    for percent in [300.0_f32, crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT, 1e12] {
                        let derived = max_zoom_with_regions(percent / 100.0)
                            .max(max_zoom_for_page(page, ppp, quality).min(MAX_ZOOM));
                        let actual = zoom_ceiling(page, ppp, quality, percent, None);
                        assert!(
                            (actual - derived).abs() <= derived.abs() * 1e-6,
                            "None must change nothing: {page:?} at {ppp}x {quality:?}, \
                             {percent}% gave {actual}, wanted {derived}"
                        );
                    }
                }
            }
        }
    }

    /// ★★★ **A learned ceiling binds even above the number the operator typed**,
    /// which no other clause in [`zoom_ceiling`] does.
    ///
    /// This is the clause that answers O186. `max_zoom_percent` is deliberately
    /// unbounded — he asked for a trillion percent and
    /// `a_configured_maximum_is_honoured_past_the_whole_page_raster_limit` above
    /// asserts that he gets it — and above the region tier the page's own size
    /// stops entering the arithmetic. So without this clause *nothing* in this
    /// function can stop a page whose rasterizer has already been measured
    /// giving out, and what he saw instead was an error sentence painted across
    /// his drawing.
    ///
    /// The numbers are a real measurement, not a round one: an E-size sheet
    /// refused at raster scale 284,964, where a business card got to 8,053,069 —
    /// a 28× spread on the same build, which is the whole reason this ceiling
    /// has to be learned per page rather than derived once.
    #[test]
    fn a_learned_ceiling_overrules_even_a_trillion_percent() {
        let e_size = (2448.0_f32, 1584.0);
        // `RasterCeiling::BACKOFF` applied to the measured refusal, spelled out
        // rather than imported: this test is about whether `zoom_ceiling`
        // honours what it is handed, and reaching across to the other module for
        // its constant would couple the two without testing either better.
        let learned = 284_964.0_f32 * 0.75;
        let uncapped = zoom_ceiling(e_size, 1.0, RenderQuality::Normal, 1e12, None);
        let capped = zoom_ceiling(e_size, 1.0, RenderQuality::Normal, 1e12, Some(learned));
        assert!(
            uncapped > learned,
            "the premise: uncapped the shell offers {uncapped}x, well past {learned}x"
        );
        assert!(
            (capped - learned).abs() <= learned * 1e-6,
            "a learned ceiling must bind: got {capped}x, wanted {learned}x"
        );
    }

    /// ★★ **The learned value is a raster SCALE, so the zoom it permits moves
    /// with BOTH halves of [`crate::viewer::raster_density`]** — the display
    /// density and the operator's render quality.
    ///
    /// Pinned as its own test because this is the one part of O186 that fails
    /// *silently* rather than visibly if it is got backwards. A ceiling stored
    /// as a zoom would be too high on a high-DPI monitor and too low on a
    /// standard one, so the operator would meet the same wall again on exactly
    /// one of his two screens while the shell looked correct on the other — and
    /// a window dragged between them would change the answer with no event
    /// anywhere to explain it.
    ///
    /// ★★★ **The quality rows are O218 and they are the ones that were false.**
    /// The density row alone passed for a year while the conversion divided by
    /// the density and dropped the quality multiplier, because the test that
    /// measured the conversion only ever varied the half that was right. A
    /// test's coverage of a product is the product of the axes it varies, and
    /// this one varied one of two.
    #[test]
    fn the_learned_ceiling_is_a_raster_scale_and_not_a_zoom() {
        let page = (612.0_f32, 792.0);
        let learned = 50_000.0_f32;
        let at_100 = zoom_ceiling(page, 1.0, RenderQuality::Normal, 1e12, Some(learned));
        let at_200 = zoom_ceiling(page, 2.0, RenderQuality::Normal, 1e12, Some(learned));
        assert!(
            (at_100 - learned).abs() <= learned * 1e-6,
            "at 100% density the scale IS the zoom, got {at_100}"
        );
        assert!(
            (at_200 - learned / 2.0).abs() <= learned * 1e-6,
            "at 200% density the same scale is half the zoom, got {at_200}"
        );

        for quality in [
            RenderQuality::Faster,
            RenderQuality::Normal,
            RenderQuality::Sharper,
        ] {
            let got = zoom_ceiling(page, 1.0, quality, 1e12, Some(learned));
            let want = learned / quality.multiplier();
            assert!(
                (got - want).abs() <= want * 1e-6,
                "{quality:?} rasterizes at {}x, so the same scale permits {want}x, got {got}",
                quality.multiplier()
            );
        }
    }

    /// ★★★ **A ceiling this function reports must be one the ENGINE would
    /// accept** — O218, and the assertion that fails on the old arithmetic.
    ///
    /// Every other test here compares one derivation against another, which
    /// cannot catch a factor missing from both. This one closes the loop the
    /// only way it can be closed without a renderer: take the zoom this
    /// function offers under a learned ceiling, put it back through
    /// [`crate::viewer::raster_scale`] — the function the canvas actually uses
    /// to order a raster — and check that it does not re-order the scale that
    /// was refused.
    ///
    /// # ★★ Why the LEARNED clause, and why the derived ones cannot be asserted
    /// this way
    ///
    /// A ceiling from the derived clauses is not a claim about a whole-page
    /// raster at all. Above [`max_zoom_for_page`] the canvas switches to
    /// [`crate::render::strategy::Strategy::Region`], whose pixmap is the
    /// *window* — so a derived ceiling well past the page's own limit is
    /// correct, and an assertion that a derived ceiling fits `MAX_PIXMAP_EDGE`
    /// would be testing a rule the shell does not have. The learned clause is
    /// different in kind: it is a **measurement of what the engine refused**,
    /// so re-ordering at or above it is by definition another refusal.
    ///
    /// The operator's own build runs `render_quality = sharper`, so the Sharper
    /// rows are not hypothetical. The old conversion divided the refused scale
    /// by the display density alone, which returns a zoom that rasterizes at
    /// exactly `scale × 1.5` — the clamp that exists to rescue him handed the
    /// engine a *larger* raster than the one it had just refused, and he was
    /// shown *"this zoom is further in than pdfcer can rasterize"* again.
    #[test]
    fn a_learned_ceiling_is_not_re_ordered_at_the_zoom_it_permits() {
        let page = (2448.0_f32, 1584.0);
        // A real refusal, with `RasterCeiling::BACKOFF` already applied — the
        // E-size measurement `a_learned_ceiling_overrules_even_a_trillion_percent`
        // uses, so both tests speak about the same wall.
        let refused = 284_964.0_f32;
        let learned = refused * 0.75;
        for ppp in [1.0_f32, 1.5, 2.0] {
            for quality in [
                RenderQuality::Faster,
                RenderQuality::Normal,
                RenderQuality::Sharper,
            ] {
                let ceiling = zoom_ceiling(page, ppp, quality, 1e12, Some(learned));
                let ordered = raster_scale(ceiling, ppp, quality);
                assert!(
                    ordered <= learned * (1.0 + 1e-6),
                    "{ppp}x {quality:?}: the ceiling {ceiling}x orders raster scale \
                     {ordered}, past the {learned} the engine accepted"
                );
                // …and not uselessly far below it either, or the clamp would
                // answer a refusal by throwing away magnification the page can
                // still draw at. The two halves together say the conversion is
                // the inverse, not merely conservative.
                assert!(
                    ordered >= learned * (1.0 - 1e-6),
                    "{ppp}x {quality:?}: the ceiling {ceiling}x orders only {ordered}, \
                     short of the {learned} the page reached"
                );
            }
        }
    }

    /// ★★ **A learned ceiling can never make a document unzoomable, and can
    /// never RAISE a ceiling.**
    ///
    /// Three clauses in one test because each is a bound on the same `min`:
    ///
    /// * floored at [`MIN_ZOOM`], so even an absurdly small learned scale leaves
    ///   the document navigable — the failure mode that would otherwise present
    ///   to the operator as a file that refuses to be magnified at all, with no
    ///   sentence anywhere that could explain it;
    /// * a learned value ABOVE the derived ceiling changes nothing, because this
    ///   clause is a narrowing and not a replacement;
    /// * a degenerate value changes nothing. [`crate::render::ceiling`] refuses
    ///   to learn one, so it cannot arrive from there — the guard exists because
    ///   this function is `pub`, and this test is what stops the guard being
    ///   deleted as unreachable by someone who checked only the one caller.
    #[test]
    fn a_learned_ceiling_is_bounded_below_and_never_raises_anything() {
        let page = (612.0_f32, 792.0);

        let floored = zoom_ceiling(
            page,
            1.0,
            RenderQuality::Normal,
            800.0,
            Some(f32::MIN_POSITIVE),
        );
        assert!(
            (floored - MIN_ZOOM).abs() < 1e-6,
            "an absurd learned scale must floor at MIN_ZOOM, got {floored}"
        );

        let derived = zoom_ceiling(page, 1.0, RenderQuality::Normal, 800.0, None);
        let above = zoom_ceiling(
            page,
            1.0,
            RenderQuality::Normal,
            800.0,
            Some(derived * 100.0),
        );
        assert!(
            (above - derived).abs() <= derived * 1e-6,
            "a learned ceiling above the derived one changes nothing: {above} vs {derived}"
        );

        for bad in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
            let got = zoom_ceiling(page, 1.0, RenderQuality::Normal, 800.0, Some(bad));
            assert!(
                (got - derived).abs() <= derived * 1e-6,
                "a learned scale of {bad} must change nothing, got {got}"
            );
        }
    }
}
