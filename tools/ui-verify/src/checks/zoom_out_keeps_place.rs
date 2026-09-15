//! `zooming_back_out_keeps_the_view` — the **descent**, which nothing had ever
//! driven.
//!
//! # The report
//!
//!
//! > *"Zoom out has a small bug where it sometimes seems to reposition the page
//! > so that it is off screen in the far bottom left corner. This happened when
//! > I zoomed back from around 2 million% but seems to happen at other
//! > junctions too."*
//!
//! **2 million % is the same number as O24f's**, and it was not a number he
//! picked either time: on the day of both reports
//! `SUB_PIXEL_CONTENT_EXTENT / page_height` was 16,777,216 / 792 ≈
//! **2,118,000 %** on a US Letter sheet, which is where the position hands over
//! between the `f32` scroll offset and the `f64` [`DeepAnchor`]. O24f fixed
//! that hand-over **going up**. This check exists because nothing in the suite
//! had ever come back **down** through it.
//!
//!
//! [`DeepAnchor`]: pdfcer-gui `viewer::deep::DeepAnchor`
//!
//! # ★★★ Why the sibling check could not see this
//!
//! [`super::zoom_keeps_place`] climbs. It climbs all the way to the ceiling,
//! one notch at a time, with a tolerance tight enough to catch a fraction of a
//! point — and then the run ends, at 10¹² %, having never once rolled the wheel
//! the other way. Its own header calls the hand-over *"half of what this check
//! is for"*, and it guards against a run that never crossed the boundary. Both
//! statements are true of the **upward** crossing only.
//!
//! A hand-over is two functions, not one. Seeding the `f64` anchor from the
//! scroll offset on the way in and reconstructing the scroll offset from the
//! `f64` anchor on the way out are separate pieces of code, and only the first
//! one was ever written. **A check that only ever travels in one direction
//! tests one of them.**
//!
//! # What is measured
//!
//! The same quantity as the sibling, with the same instrument (its [`held`] is
//! `pub(crate)` for exactly this reason — two spellings of *"where is the
//! view"* would drift, and the one that drifted would be the one whose check
//! went green): **the page point under the centre of the canvas**, read from
//! the `f64` `canvas-pos` line, before and after every single wheel notch.
//!
//! The pointer sits on that centre for every notch, so zoom-to-cursor should
//! hold that page point still to within rounding, at every zoom, in both
//! directions. A descent that throws the view away moves it by a large
//! fraction of the page — the defect is not subtle once it is looked for.
//!
//! # The shape of the run
//!

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
///
/// The threshold is about 132,000 % on a Letter sheet — `2^20` px of content
/// extent over 792 pt; see the module header for why the older 2,118,000 % no
/// longer applies — and a page-fit start is about 76 %, so the climb is a
/// factor of ~1,700. A wheel notch multiplies the zoom by roughly 1.22, so ~38
/// notches reach it. The cap is generous: **reaching it is now a FAIL**, and the
/// only build that reaches it is one whose wheel is not zooming — which the
/// descent guard would report anyway.
///
/// ⚠ The cap is **not** re-tuned downward just because the threshold fell.
/// Spending a hundred notches this run would cost a second; arriving exactly at
/// the boundary and reporting a failure would cost a reader an investigation.
///
/// ★ On [`FIXTURE`] the numbers are **measured, not derived**, and the first
/// two written here were neither.
///
/// That sheet is 2383.9 pt on its long side, not Letter's 792 — and the long
/// side is what the bound is taken against, because the threshold is
/// `SUB_PIXEL_CONTENT_EXTENT / longest_page_pt`. A paragraph written on
/// 2026-09-13 said *"1683.8 pt tall"*, quoted a threshold of **623** and a
/// crossing at **33** notches; every one of those is wrong. 1683.8 is the
/// SHORT side, and using it inflated the threshold by the aspect ratio.
///
/// The driven run reports the real figures on its own progress lines:
/// threshold **~440**, crossed at notch **39**, at a zoom of **46,479 %**,
/// turning round at 154,316 %. Those are what [`MAX_CLIMB`]'s headroom is
/// measured against.
///
/// ⚠ The Letter arithmetic above is kept anyway, because it is where the cap
/// came from — but it is arithmetic about a different sheet, and a reader who
/// takes it for this check's notch count will be out by six notches and two
/// orders of magnitude of zoom.
const MAX_CLIMB: usize = 140;

/// **The sheet this check is calibrated against**, pinned rather than taken from
/// `--pdf`.
///
/// The same A1 fixture `sweep-full.sh` hands the chunked checks, chosen for that
/// reason: it is the sheet on which this check has actually been observed
/// crossing the hand-over and coming back down, so every notch count in this
/// file is quoted against it. See the pinning block in `drive` for why `--pdf`
/// is ignored rather than preferred.
const FIXTURE: &str = "fixtures/a1-titleblock.pdf";

/// How far past the threshold to climb before turning round.
///
/// ★ Not zero, deliberately. A descent that begins with the zoom balanced
/// exactly on the boundary could cross back on its first notch, and a check
/// whose first measurement *is* the hand-over cannot distinguish "the
/// hand-over is broken" from "the climb ended somewhere unlucky". Starting a
/// few notches inside gives the deep tier a chance to be measured holding
/// still before the interesting notch arrives.
const PAST_THRESHOLD: usize = 6;

/// The most notches to spend descending.
///
/// The climb records how many notches it took; the descent is allowed that
/// many plus this margin, so the run is symmetric by construction rather than
/// by a hard-coded depth that would stop matching the day the ladder changes.
const DESCENT_MARGIN: usize = 16;

/// How many consecutive `tier=scroll` readings end the descent.
///
/// ★ Consecutive, not "the first one". The whole subject of this check is the
/// frames immediately after the hand-over — a descent that stopped the instant
/// the tier flipped would stop one notch before the defect had a chance to
/// show, which is the same mistake as measuring once per stage.
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

    // ★ `settled`, not `held` — see its documentation. egui smooths a
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

        // ★ Wait for the notch to LAND before reading anything about it —
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
        // ★★★ EVERY notch is asserted, at the same tolerance, at every zoom.
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

    // ★★★ `tiers` records what the TRACE reported; `crossings` records what
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
