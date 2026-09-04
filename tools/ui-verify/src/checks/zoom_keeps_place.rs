//! `zooming_does_not_throw_away_where_the_operator_panned` — two reports, one
//! question: *does a zoom keep the view where I put it?*
//!
//! # The reports
//!
//! `OPERATOR_REQUESTS.md` O24e and O24f, 2026-08-22:
//!
//! > *"if I am zoomed out to about page size, pan the cells to the center of
//! > the screen, then start to zoom, the page snaps back to near the center
//! > position."*
//!
//! > *"I do lose the view at 2000000% magnification."*
//!
//! The same failure at two scales, and the same shape at both: a zoom
//! **discards the position** instead of magnifying about it.
//!
//! | where | cause |
//! |---|---|
//! | at fit-page zoom | `geometry::zoom_anchor_offset` clamped to `display - viewport`, which is **zero or negative** when the page is no larger than the viewport — so the offset was forced to 0, the centred position |
//! | at ~2,100,000 % | the `f64` tier's anchor was seeded from the **previous frame's** scroll offset, and then never moved on a zoom at all |
//!
//! # ★★ Why this is not the same check as O24c's
//!
//! `panning_at_deep_zoom_stays_where_it_was_put` asks whether a **pan** moves
//! the view and whether the pixels land in the right place. This asks whether a
//! **zoom** preserves it. They failed independently and were fixed
//! independently; one check covering both would have gone red for one reason
//! while the other was still broken, and the second would have been found later
//! and blamed on the first fix.
//!
//! The measurement is the page point under the **viewport centre**, before and
//! after each zoom. Zoom-to-cursor holds the point under the *pointer*, and
//! this check puts the pointer at the centre, so a correct build keeps that
//! page point fixed however deep it goes.
//!
//! ★ Measured in page units rather than screen pixels, deliberately. Screen
//! pixels are what the defect happens in, but page units are what "the same
//! place on the drawing" means — and the tolerance has to shrink with the zoom,
//! or a check at a million percent would quietly accept a metre of drift.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// `VK_CONTROL`, held while the wheel rolls to make it a zoom.
pub(crate) const VK_CONTROL: u16 = 0x11;

/// The canvas viewport, whose centre the pointer sits at.
pub(crate) const CANVAS_REGION: &str = "canvas-viewport";

/// The canvas's own state line.
pub(crate) const CANVAS_EVENT: &str = "canvas";

/// The `f64` position line, which carries the tier.
pub(crate) const POS_EVENT: &str = "canvas-pos";

/// Where to roll the wheel to knock the view off its centred position.
///
/// ★ The pan is what makes this check able to fail. The centred position is
/// exactly where the O24e defect snapped **to**, so a check that zoomed without
/// panning first would have watched the view "stay" where the bug was about to
/// put it anyway — green, and measuring nothing.
const PAN_AT: (f32, f32) = (0.30, 0.30);

/// How many Ctrl+wheel notches per stage.
const STAGE: usize = 8;

/// The most stages to climb before giving up.
///
/// # ★★ A CAP, not a count — the loop climbs until the zoom SATURATES
///
/// The operator, 2026-08-22: *"can you test up to maximum zoom please?"* So the
/// run does not stop at a chosen depth; it keeps rolling until a whole stage
/// fails to increase the zoom, which is the application saying it has reached
/// its ceiling. With the default maximum of 10¹² % that is a long climb — eight
/// notches multiply the zoom by roughly five, so from a page-fit 76 % it takes
/// about fifteen stages.
///
/// ★ The cap exists only so that a build broken in the *other* direction — one
/// that climbs by an epsilon for ever — ends the run instead of wedging the
/// suite. Reaching it is reported as a SKIP, not a pass: a run that never found
/// the ceiling has not tested the ceiling.
///
/// ★★ The saturation test asks the APPLICATION where its ceiling is rather than
/// comparing against a constant. The maximum is an operator setting, so a check
/// that hard-coded 10¹² % would silently stop testing the ceiling the day he
/// changed it — the same silently-inert control this whole request began with.
const MAX_STAGES: usize = 24;

// Where the two tier boundaries sit, for the guard at the end of the climb: the
// region raster tier engages at `MAX_PIXMAP_EDGE / page_height` ≈ 2,070 % on a
// Letter sheet, and the `f64` position tier at
// `SUB_PIXEL_CONTENT_EXTENT / page_height` ≈ 2,118,000 %. Both are far below
// the ceiling, so a saturating climb crosses them on the way — but the guard
// checks rather than assumes.

/// How far the anchored page point may drift **per wheel notch**, as a
/// fraction of the page width currently visible.
///
/// # ★★ Per notch, not per stage, and the difference is not pedantry
///
/// The first version read the position once per stage of eight notches and
/// judged it against a one-notch tolerance. It failed by 3.17 pt against
/// 2.57 pt — a real number and a meaningless one, because eight anchored zoom
/// steps each carry their own `f32` rounding and the accumulated slop was being
/// measured against the budget for one.
///
/// The tempting fix is to multiply the tolerance by the notch count. That is
/// loosening a threshold to fit an observation, which is exactly how a check
/// stops being able to see the defect it was written for. Reading after
/// **every** notch keeps the tolerance tight and localises a failure to the
/// notch that caused it.
///
/// ★ A fraction rather than an absolute, because the tolerance must shrink with
/// the zoom. Two percent of what is on screen is generous against a defect that
/// discards the position outright — O24e moved the view by the whole pan, and
/// O24f by the whole zoom ratio.
pub(crate) const DRIFT_FRACTION: f64 = 0.02;

/// The smallest drift this check will ever call a failure, in page points.
///
/// # ★★ A floor on the TOLERANCE, which is not the same as loosening it
///
/// [`DRIFT_FRACTION`] is a fraction of what is on screen, so it shrinks with
/// the zoom — which is right, and which at the top of the climb takes it below
/// what any instrument here can resolve. A tolerance finer than the measurement
/// is not a strict check; it is a coin toss that reports whichever way the last
/// bit fell.
///
/// This is deliberately far below anything an operator could see: a ten
/// thousandth of a point is about a fortieth of the width of a banana cell's
/// label stroke on the fixture. Every defect this check exists for moves the
/// view by hundreds of points or by the whole pan. Nothing real hides under it.
///
/// ★ It is a **floor on the tolerance**, applied only where the proportional
/// tolerance would be smaller — not a widening of it at the zooms where the
/// proportional one is meaningful. Those are different changes and only one of
/// them is honest.
pub(crate) const RESOLUTION_FLOOR: f64 = 1e-4;

/// See the module documentation.
pub struct ZoomingDoesNotThrowAwayWhereTheOperatorPanned;

impl Check for ZoomingDoesNotThrowAwayWhereTheOperatorPanned {
    fn name(&self) -> &'static str {
        "zooming_does_not_throw_away_where_the_operator_panned"
    }

    fn defect(&self) -> &'static str {
        "zooming after panning snaps the view back to the centre of the page, or loses it \
         entirely past about two million percent — the zoom discards the position instead of \
         magnifying about it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// One reading: the page point under the viewport centre, the zoom, the span.
///
/// ★ `pub(crate)` because [`super::zoom_out_keeps_place`] measures the same
/// quantity on the way back down and must measure it with the **same
/// instrument**. Two spellings of "where is the view" would drift, and the one
/// that drifted would be the one whose check went green.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Held {
    /// The page point under the viewport centre, in PDF user-space points.
    pub(crate) page: (f64, f64),
    /// Logical points per user-space unit.
    pub(crate) zoom: f64,
    /// How wide the viewport is in page units — the scale the drift tolerance
    /// is taken against.
    pub(crate) span: f64,
    /// Whether the reading was taken while the `f64` anchor owned the
    /// position. Carried on the reading rather than looked up separately so
    /// that [`judge`] cannot be handed a tier from a different frame than the
    /// measurement it is judging.
    pub(crate) deep: bool,
}

/// Read the page point currently under the centre of the canvas.
///
/// # ★★★ From the `f64` position line, because the `f32` one runs out
///
/// The first version derived this from the `canvas` line's `rect=` and `zoom=`:
/// `(centre − rect.min) / zoom`. Correct, and it stops working partway up the
/// climb. At 41,000,000 % a Letter page's rect holds a magnitude near 2.5 × 10⁸,
/// where an `f32`'s representable spacing is 32 — so the page point it yields
/// resolves to about 8 × 10⁻⁵ pt, while the drift tolerance at that zoom is
/// 3 × 10⁻⁵. **The measurement became coarser than the thing being measured**,
/// and the check failed with "moved 0.0000 pt, where 0.0000 is the tolerance"
/// against a build that was holding the point perfectly.
///
/// ★ That is the harness's floor, not the application's, and the tempting fix —
/// widening the tolerance — would have hidden a real defect at every zoom below
/// it. The `canvas-pos` line already carries the same quantity in `f64`
/// (`canvas::trace::position`, added for O24b for exactly this reason), so the
/// fix is to read the instrument that can still see.
///
/// `at=` is how far the view has been panned from the acting page's corner, in
/// screen pixels. The page point under a window point `p` is therefore
/// `(at + (p − viewport.min)) / zoom`, and the second term is small at every
/// depth — so no large intermediate is formed here either.
///
/// Falls back to the `f32` derivation when no position line has been emitted,
/// which is the case for a build older than that trace field. ★ The fallback is
/// SILENT by design at shallow zooms, where the two agree to many decimals, and
/// is why [`RESOLUTION_FLOOR`] exists as a second guard.
pub(crate) fn held(session: &Session, canvas: crate::geom::LRect) -> Result<Option<Held>> {
    let trace = session.trace()?;
    let Some(line) = trace.events(CANVAS_EVENT).last() else {
        return Ok(None);
    };
    let Some(zoom) = line.get_f32("zoom") else {
        return Ok(None);
    };
    if zoom <= 0.0 {
        return Ok(None);
    }
    let z = f64::from(zoom);
    let cx = f64::from(canvas.min.x + canvas.max.x) / 2.0;
    let cy = f64::from(canvas.min.y + canvas.max.y) / 2.0;

    // The f64 pan position, if this build publishes one.
    let at = trace.events(POS_EVENT).last().and_then(|l| {
        let (x, y) = l.get("at")?.split_once(',')?;
        Some((x.parse::<f64>().ok()?, y.parse::<f64>().ok()?))
    });

    let page = if let Some((ax, ay)) = at {
        (
            (ax + (cx - f64::from(canvas.min.x))) / z,
            (ay + (cy - f64::from(canvas.min.y))) / z,
        )
    } else {
        let Some(rect) = line.get_rect("rect") else {
            return Ok(None);
        };
        (
            (cx - f64::from(rect.min.x)) / z,
            (cy - f64::from(rect.min.y)) / z,
        )
    };

    Ok(Some(Held {
        page,
        zoom: z,
        span: f64::from(canvas.max.x - canvas.min.x) / z,
        deep: trace
            .events(POS_EVENT)
            .last()
            .and_then(|l| l.get("tier"))
            .is_some_and(|t| t == "deep"),
    }))
}

/// Which position tier the canvas last reported.
pub(crate) fn tier(session: &Session) -> Result<String> {
    let trace = session.trace()?;
    Ok(trace
        .events(POS_EVENT)
        .last()
        .and_then(|l| l.get("tier"))
        .unwrap_or("?")
        .to_owned())
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. There is nothing to zoom."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check pans and zooms the canvas. Reported as \
             SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("zoom-keeps-place.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;
    let centre = frame.declared_at(canvas, 0.5, 0.5);

    // --- knock the view off centre, at the starting (page-fit) zoom ----------
    //
    // ★ The middle button is the operator's own gesture; this harness has no
    // middle drag, and a primary drag pans only under the hand tool. The wheel
    // is unconditional and moves the view off the centred position, which is
    // all this check needs in order to be capable of failing.
    driver.scroll_at(frame.declared_at(canvas, PAN_AT.0, PAN_AT.1), -4)?;
    session.settle(20);

    let Some(mut prev) = held(&session, canvas)? else {
        return Err(Error::new(
            "the canvas never published a rect and a zoom, so there is no page point to follow. \
             SKIPPED.",
        ));
    };
    report.note(format!(
        "panned off-centre; the page point under the centre is ({:.2}, {:.2}) at {:.0}%",
        prev.page.0,
        prev.page.1,
        prev.zoom * 100.0
    ));

    let mut tiers: Vec<String> = Vec::new();
    let mut worst = 0.0_f64;
    // How many notches were measured with a `deep` reading on one side and a
    // `scroll` reading on the other -- the UPWARD hand-over, judged.
    let mut crossings = 0usize;
    let mut climbed = 0usize;
    let mut saturated = false;
    let mut stage = 0usize;
    while stage < MAX_STAGES && !saturated {
        let stage_from = prev.zoom;
        // ★★ ONE NOTCH AT A TIME. See `DRIFT_FRACTION` — reading once per
        // stage compared eight steps of accumulated rounding against the
        // budget for one, and failed a correct build by 3.17 pt against 2.57.
        for notch in 0..STAGE {
            driver.scroll_at_held(centre, &[VK_CONTROL], 1, 1)?;
            session.settle(6);

            let now = tier(&session)?;
            if !tiers.contains(&now) {
                tiers.push(now.clone());
            }

            let Some(after) = held(&session, canvas)? else {
                return Err(Error::new(
                    "the canvas stopped publishing a rect and a zoom. SKIPPED.",
                ));
            };
            let drift = (after.page.0 - prev.page.0)
                .abs()
                .max((after.page.1 - prev.page.1).abs());
            // ★ `crossings` counts the notches that spanned the tier
            // boundary — the UPWARD hand-over, which is what O24f broke. It
            // is counted rather than merely observed because `tiers` records
            // what the TRACE reported and only a notch measured with a
            // reading on each side is evidence that the crossing was judged.
            let crossed = prev.deep != after.deep;
            if crossed {
                crossings += 1;
            }
            if after.zoom > prev.zoom {
                climbed += 1;
            }
            let allowed = (prev.span.min(after.span) * DRIFT_FRACTION).max(RESOLUTION_FLOOR);
            worst = worst.max(if allowed > 0.0 { drift / allowed } else { 0.0 });

            if drift > allowed {
                return Ok(Some(format!(
                    "★★ THE ZOOM THREW THE VIEW AWAY. Notch {notch} of stage {stage}, between \
                     {:.0}% and {:.0}% (tier `{now}`): the page point under the viewport centre \
                     moved from ({:.4}, {:.4}) to ({:.4}, {:.4}) — {drift:.4} pt, where \
                     {allowed:.4} is the tolerance. The pointer was ON the centre, so \
                     zoom-to-cursor should have held that point. On tier `scroll` this is O24e: \
                     `geometry::zoom_anchor_offset` clamping to `display - viewport`, which is \
                     zero or negative whenever the page is no larger than the viewport, forcing \
                     the offset to the centred position. On tier `deep` it is O24f: the anchor \
                     seeded from the PREVIOUS frame's scroll offset, or never moved on a zoom \
                     at all because nothing called `DeepAnchor::zoomed_about`.",
                    prev.zoom * 100.0,
                    after.zoom * 100.0,
                    prev.page.0,
                    prev.page.1,
                    after.page.0,
                    after.page.1
                )));
            }
            prev = after;
        }
        // The ceiling, recognised by the zoom not moving through a whole stage
        // of eight notches. See `MAX_STAGES`.
        saturated = prev.zoom <= stage_from * (1.0 + f64::EPSILON);
        stage += 1;
        report.note(format!(
            "stage {stage}: {:.0}% to {:.0}% on tier `{}`, worst per-notch drift so far is \
             {:.0}% of the tolerance",
            stage_from * 100.0,
            prev.zoom * 100.0,
            tiers.last().map_or("?", String::as_str),
            worst * 100.0
        ));
    }

    // ★ A run that never climbed has said nothing. Checked once at the end
    // rather than per notch: a single notch that lands on the rung the zoom is
    // already at is not a fault, a whole run that never moves is.
    if !saturated {
        return Err(Error::new(format!(
            "after {MAX_STAGES} stages the zoom was still climbing, at {:.0}%. The run never \
             reached the ceiling, so it has not tested it. Either the maximum-zoom setting is \
             higher than this climb can reach, or something is increasing the zoom by an \
             epsilon per notch. SKIPPED rather than passed.",
            prev.zoom * 100.0
        )));
    }
    report.note(format!(
        "saturated at {:.0}% after {stage} stage(s)",
        prev.zoom * 100.0
    ));

    // ★★ MOST notches must have advanced, not all of them.
    //
    // This guard catches a run where the wheel was not zooming at all — a lost
    // Ctrl turns Ctrl+wheel into an ordinary pan, and the climb would then be a
    // pan reporting nothing. It is NOT a claim that every notch advances, and
    // phrasing it that way is what made it fire on a perfect run: the ceiling
    // is reached partway through a stage, so the tail of that stage and the
    // whole of the next legitimately stand still. The measured climb to 10¹² %
    // advanced on 117 of 128 notches.
    //
    // ★ Three quarters, with room to spare: a build whose wheel is panning
    // instead of zooming advances on ZERO notches, so the two cases are nowhere
    // near each other and the exact fraction is not load-bearing.
    if climbed * 4 < stage * STAGE * 3 {
        return Err(Error::new(format!(
            "only {climbed} of {} wheel notches zoomed in, ending at {:.0}%. Fewer than three \
             quarters advanced, which is not a climb that saturated near the top — it is a \
             wheel that was not zooming. Either the Ctrl was lost and it panned instead, or the \
             ladder is refusing to step. SKIPPED rather than passed.",
            stage * STAGE,
            prev.zoom * 100.0
        )));
    }

    // ★★★ REFUSE TO PASS A RUN THAT NEVER LEFT ONE TIER.
    //
    // Half of what this check is for is the HAND-OVER between the `f32` scroll
    // offset and the `f64` anchor, and a run that stayed on one side of it has
    // not tested that at all. The same guard as `deep_pan`'s
    // `REGION_TIER_REQUIRED`, for the same reason: on 2026-08-22 a check
    // reported PASS twice against a binary with the defect deliberately put
    // back in, because it never reached the tier it was named after.
    if tiers.len() < 2 {
        return Err(Error::new(format!(
            "every reading was on tier `{}` — the run never crossed the boundary between the f32 \
             scroll offset and the f64 anchor, which is half of what this check is for. That \
             boundary is at SUB_PIXEL_CONTENT_EXTENT / page_height, about 2,118,000 % on a US \
             Letter sheet. Raise STAGES or STAGE until it is crossed. SKIPPED rather than passed.",
            tiers.first().map_or("?", String::as_str)
        )));
    }
    report.note(format!(
        "crossed tiers: {} -- {crossings} notch(es) spanned the hand-over and were judged",
        tiers.join(" to ")
    ));

    Ok(None)
}
