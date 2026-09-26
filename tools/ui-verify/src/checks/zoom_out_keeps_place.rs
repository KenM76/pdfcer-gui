//! `zooming_back_out_keeps_the_view` — the **descent**, which nothing had ever
//! driven.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/zoom_out_keeps_place.md`.

use crate::checks::driving;
use crate::checks::zoom_keeps_place::{
    CANVAS_REGION, DRIFT_FRACTION, RESOLUTION_FLOOR, VK_CONTROL, held, settled, tier,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// Where to roll the wheel to knock the view off its centred position. The
/// sibling's constant, and for the sibling's reason.
const PAN_AT: (f32, f32) = (0.30, 0.30);

/// The most notches to spend climbing to the deep tier before giving up.
const MAX_CLIMB: usize = 140;

/// **The sheet this check is calibrated against**, pinned rather than taken from
/// `--pdf`.
const FIXTURE: &str = "fixtures/a1-titleblock.pdf";

/// How far past the threshold to climb before turning round.
const PAST_THRESHOLD: usize = 6;

/// The most notches to spend descending.
const DESCENT_MARGIN: usize = 16;

/// How many consecutive `tier=scroll` readings end the descent.
const SETTLE_NOTCHES: usize = 8;

/// See the module documentation.
pub struct ZoomingBackOutKeepsTheView;

impl Check for ZoomingBackOutKeepsTheView {
    fn name(&self) -> &'static str {
        "zooming_back_out_keeps_the_view"
    }

    fn defect(&self) -> &'static str {
        "zooming back out from past the f32 scroll offset's addressing limit throws the page off \
         screen into a corner — the f64 anchor's position is never handed back to the f32 scroll \
         offset, so the first shallow frame solves its zoom against an offset that was forced to \
         zero while deep"
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

#[allow(
    clippy::too_many_lines,
    reason = "one linear driven sequence, narrated"
)] // ui-text-exempt: clippy lint justification, never displayed
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    //
    // It used to read `ctx.pdf` and SKIP when that was absent, which made the
    // check unrunnable on its own: `--check zooming_back_out_keeps_the_view`
    // reported *"no --pdf. There is nothing to zoom."* and only `sweep-full.sh`
    // — which hands the chunked checks one shared fixture — ever actually drove
    // it. That mattered the moment the `!reached_deep` branch below became a
    // FAIL: a failure nobody can reproduce with a one-line command is a failure
    // nobody reproduces.
    //
    // ⚠ And the page SIZE is load-bearing here in a way it is not for most
    // checks. The hand-over threshold bounds `longest_page_pt × zoom`, so the
    // number of notches the climb takes depends on the sheet. [`FIXTURE`] is the
    // same A1 sheet the sweep passes, which is what the measured notch counts in
    // this file are quoted against.
    let pdf = crate::fixture::workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "{FIXTURE} is missing from the repository, so there is no sheet whose hand-over \
             threshold this climb is calibrated against. SKIPPED."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was IGNORED; this check pins {FIXTURE} because the hand-over it must cross and \
             come back down through is a bound on `longest_page_pt * zoom`, so the page size \
             decides the notch count"
        ));
    }
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("zoom-out-keeps-place.trace.txt"));
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

    // --- 1. knock the view off centre ---------------------------------------
    report.note(
        "panning off the centred position first: the centred position is what a \
         position-discarding zoom lands ON, so a run that started there could not tell holding \
         from losing"
            .to_owned(),
    );
    driver.scroll_at(frame.declared_at(canvas, PAN_AT.0, PAN_AT.1), -4)?;
    session.settle(20);

    // --- 2. climb until the f64 tier owns the position ----------------------
    let mut climbed = 0usize;
    let mut reached_deep = false;
    while climbed < MAX_CLIMB {
        driver.scroll_at_held(centre, &[VK_CONTROL], 1, 1)?;
        session.settle(6);
        climbed += 1;
        if tier(&session)? == "deep" {
            reached_deep = true;
            break;
        }
    }
    //
    // It was written as `Err` (which this harness reports as SKIPPED) on the
    // defensible ground that a run which never crossed the hand-over has
    // measured nothing about coming back down through it, and a check that
    // measured nothing should not claim a pass. That is still true.
    //
    //
    // ⚠ The message still explains the two benign causes, because naming them
    // is what makes a failure diagnosable. It no longer offers them as a reason
    // to look away.
    if !reached_deep {
        let zoom = held(&session, canvas)?.map_or(0.0, |h| h.zoom * 100.0);
        return Ok(Some(format!(
            "after {climbed} Ctrl+wheel notches the position tier was still `scroll`, at \
             {zoom:.0}%. The run never reached the f64 tier, so it cannot have come back down \
             through the hand-over — which is the entire subject of this check. Either the wheel \
             lost its Ctrl and panned instead, or the deep threshold is higher than this climb \
             reaches. ★ This used to be SKIPPED; it is a FAIL because the crossing has been \
             measured on this fixture, so failing to cross it is a regression rather than a run \
             that fell short."
        )));
    }
    let at_threshold = held(&session, canvas)?.map_or(0.0, |h| h.zoom * 100.0);
    report.note(format!(
        "reached tier `deep` after {climbed} notches, at {at_threshold:.0}%"
    ));

    driver.scroll_at_held(centre, &[VK_CONTROL], 1, PAST_THRESHOLD)?;
    session.settle(20);

    // `settled`, not `held` — see its documentation. egui smooths a
    // Ctrl+wheel notch across about a dozen frames, and the turn-round point is
    // the reading every drift below is measured against: taken mid-animation it
    // biases the whole descent.
    let Some(mut prev) = settled(&session, canvas)? else {
        return Err(Error::new(
            "the canvas never published a rect and a zoom, so there is no page point to follow. \
             SKIPPED.",
        ));
    };
    report.note(format!(
        "turning round at {:.0}% on tier `{}`; the page point under the viewport centre is \
         ({:.6}, {:.6})",
        prev.zoom * 100.0,
        tier(&session)?,
        prev.page.0,
        prev.page.1
    ));

    // --- 3. descend, measuring after every single notch ----------------------
    let budget = climbed + PAST_THRESHOLD + DESCENT_MARGIN;
    let mut tiers: Vec<String> = vec![tier(&session)?];
    let mut descended = 0usize;
    let mut shallow_run = 0usize;
    let mut worst = 0.0_f64;
    // How many notches were measured with a `deep` reading on one side and a
    // `scroll` reading on the other.
    let mut crossings = 0usize;
    let mut notch = 0usize;
    while notch < budget && shallow_run < SETTLE_NOTCHES {
        driver.scroll_at_held(centre, &[VK_CONTROL], -1, 1)?;
        notch += 1;

        // Wait for the notch to LAND before reading anything about it —
        // both the position and the tier. See [`super::zoom_keeps_place::settled`].
        let Some(after) = settled(&session, canvas)? else {
            return Err(Error::new(
                "the canvas stopped publishing a rect and a zoom. SKIPPED.",
            ));
        };

        let now = tier(&session)?;
        if tiers.last().map(String::as_str) != Some(now.as_str()) {
            tiers.push(now.clone());
        }
        if now == "scroll" {
            shallow_run += 1;
        } else {
            shallow_run = 0;
        }
        if after.zoom < prev.zoom {
            descended += 1;
        }
        let drift = (after.page.0 - prev.page.0)
            .abs()
            .max((after.page.1 - prev.page.1).abs());
        // EVERY notch is asserted, at the same tolerance, at every zoom.
        //
        //
        // `crossings` counts the notches that spanned the tier boundary, so a
        // run that never measured the hand-over is a SKIP rather than a pass.
        //
        // The tolerance is taken against the SMALLER span of the two readings
        // — the one further in — so a notch is judged by the tighter of the
        // two scales it spans, never by the looser.
        let crossed = prev.deep != after.deep;
        if crossed {
            crossings += 1;
        }
        let allowed = (prev.span.min(after.span) * DRIFT_FRACTION).max(RESOLUTION_FLOOR);
        worst = worst.max(if allowed > 0.0 { drift / allowed } else { 0.0 });

        if drift > allowed {
            return Ok(Some(format!(
                "★★★ ZOOMING BACK OUT THREW THE VIEW AWAY. Notch {notch} of the descent, between \
                 {:.0}% and {:.0}% (tier `{now}`, having come from `{}`): the page point under \
                 the viewport centre moved from ({:.6}, {:.6}) to ({:.6}, {:.6}) — {drift:.6} pt \
                 against a tolerance of {allowed:.6}. The pointer was ON the centre for every \
                 notch, so zoom-to-cursor should have held that point. \
                 The page is {:.1} page-widths from where it belongs. \
                 If `{now}` is `scroll` and the previous reading was `deep`, this is the \
                 DOWNWARD hand-over: `CanvasFrame::offset` is recorded from the scroll offset, \
                 which is forced to zero for the whole time the f64 anchor owns the position — \
                 so the first shallow frame's `zoom_anchor_offset` solves against an offset that \
                 describes the centred position rather than where the operator actually is.",
                prev.zoom * 100.0,
                after.zoom * 100.0,
                tiers.iter().rev().nth(1).map_or("?", String::as_str),
                prev.page.0,
                prev.page.1,
                after.page.0,
                after.page.1,
                drift / after.span.max(f64::MIN_POSITIVE)
            )));
        }
        prev = after;
    }

    // --- 4. the guards that stop a run that proved nothing from passing ------
    if descended * 4 < notch * 3 {
        return Err(Error::new(format!(
            "only {descended} of {notch} wheel notches zoomed OUT, ending at {:.0}%. Fewer than \
             three quarters descended, which is not a descent — it is a wheel that was not \
             zooming, or a ladder refusing to step down. SKIPPED rather than passed.",
            prev.zoom * 100.0
        )));
    }
    if shallow_run < SETTLE_NOTCHES {
        return Err(Error::new(format!(
            "after {notch} descending notches the position tier had still not been `scroll` for \
             {SETTLE_NOTCHES} notches in a row (last tier `{}`, at {:.0}%). The run never came \
             back through the hand-over, so it has not tested it. SKIPPED rather than passed.",
            tiers.last().map_or("?", String::as_str),
            prev.zoom * 100.0
        )));
    }
    if !tiers.iter().any(|t| t == "deep") || !tiers.iter().any(|t| t == "scroll") {
        return Err(Error::new(format!(
            "the descent saw only tier(s) `{}` — it never crossed the boundary between the f64 \
             anchor and the f32 scroll offset, which is the whole subject of this check. SKIPPED \
             rather than passed.",
            tiers.join(" to ")
        )));
    }

    // `tiers` records what the TRACE reported; `crossings` records what
    // this check actually JUDGED, and only the second is evidence. A run that
    // saw both tiers but never measured a single notch spanning them has not
    // tested the hand-over, which is the whole subject — the same distinction
    // between "it happened" and "it was measured" that the sibling's tier
    // guard draws, one level stricter.
    if crossings == 0 {
        return Err(Error::new(format!(
            "the trace reported tiers `{}`, but no single notch was measured with a `deep` \
             reading on one side and a `scroll` reading on the other — so the hand-over itself \
             was never judged. SKIPPED rather than passed.",
            tiers.join(" to ")
        )));
    }
    report.note(format!(
        "descended {descended} of {notch} notches back to {:.0}%, crossing {} — {crossings} \
         notch(es) spanned the hand-over and were judged; the worst asserted drift was {:.0}% of \
         its tolerance",
        prev.zoom * 100.0,
        tiers.join(" to "),
        worst * 100.0
    ));
    Ok(None)
}
