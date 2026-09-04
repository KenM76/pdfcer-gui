//! `zooming_past_the_pixmap_ceiling_still_renders` — the operator's own
//! failure, reproduced and then asserted away.
//!
//! # The report
//!
//! `OPERATOR_REQUESTS.md` O24, 2026-08-22:
//!
//! > *"I got a requested raster size 14580x18868 is empty or exceeds
//! > MAX_PIXMAP_EDGE when I got to 2382% zoom."*
//!
//! A US Letter page at 2382 % is 18,868 device pixels tall against a 16,384
//! cap. The whole-page raster cannot be made, and until the region tier was
//! wired into the canvas nothing else was tried — so the page simply stopped
//! rendering and said so.
//!
//! # ★★ Why this needs driving rather than a unit test
//!
//! Every part of the mechanism already had unit tests before he hit this:
//! `strategy::for_page` knew the ceiling, `render::region` converted the
//! rectangles, the worker could call `render_page_region`, and the cache keyed
//! on it. **All of them passed, and the feature did not exist**, because
//! nothing called the strategy — `strategy::for_page` appeared in the shell
//! only inside comments.
//!
//! That is the defect this project keeps finding and the reason R1 exists: a
//! mechanism that is complete and unreachable looks identical, from a test
//! suite, to one that works. The only thing that can tell them apart is a
//! zoom driven past the ceiling on a running application.
//!
//! # What it asserts
//!
//! * no render reports `outcome=failed`, which is what the raster-size refusal
//!   produces, **and**
//! * a raster actually arrives afterwards.
//!
//! ★ Both, because either alone is satisfiable by a build that renders nothing
//! at all: a canvas that never asks cannot fail, and a canvas that fails
//! silently still drew something earlier.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The zoom group on the status bar — `− ⟨percent⟩ +`.
const ZOOM_REGION: &str = "status-group:zoom";

/// The canvas's own state line, read for the zoom it reports.
const CANVAS_EVENT: &str = "canvas";

/// The worker's completion line.
const RENDER_EVENT: &str = "render-async-done";

/// How many times to press `+`.
///
/// Enough to SATURATE, deliberately. The ladder runs to 800 % and then doubles,
/// so this walks all the way to the ceiling and keeps pressing — which means
/// the check exercises the whole range rather than a point in the middle of it,
/// and would catch a build that renders at 25,000 % and fails at 2,000,000 %.
///
/// ★ Saturating also makes the check independent of where the ceiling happens
/// to be. It moved twice on 2026-08-22 — first when the region tier removed the
/// raster limit, then when the positional cap replaced it — and a press count
/// tuned to reach a particular zoom would have needed retuning both times.
const PRESSES: usize = 60;

/// The zoom this check has to exceed to be testing anything, as a multiplier.
///
/// A Letter page's whole-page raster fails at about 26×. Reported as SKIPPED
/// below this rather than passed: a run that never left the whole-page tier has
/// not exercised the region tier at all.
const MUST_EXCEED: f32 = 26.0;

/// See the module documentation.
pub struct ZoomingPastThePixmapCeilingStillRenders;

impl Check for ZoomingPastThePixmapCeilingStillRenders {
    fn name(&self) -> &'static str {
        "zooming_past_the_pixmap_ceiling_still_renders"
    }

    fn defect(&self) -> &'static str {
        "zooming past about 1000% stops the page rendering — \"requested raster size … exceeds \
         MAX_PIXMAP_EDGE\" — because the whole page is rasterized however deep the zoom goes"
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
        .ok_or_else(|| Error::new("no --pdf. There is nothing to zoom into."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses the zoom-in button repeatedly. \
             Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("deep-zoom.trace.txt"));
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
    let zoom_group = driving::declared(&trace, ui_rect, ZOOM_REGION).ok_or_else(|| {
        Error::new(format!(
            "the status bar never declared `{ZOOM_REGION}`, so the zoom-in button cannot be aimed \
             at. Either no document is open or the region was renamed."
        ))
    })?;
    let frame = session.frame()?;

    // ★ The `+` is the RIGHTMOST control of the group — the layout is
    // right-to-left and zoom-in is added first. Aiming at the middle would hit
    // the readout, which now opens the maximum-zoom popup instead of zooming.
    let plus = frame.declared_at(zoom_group, 0.93, 0.5);

    for _ in 0..PRESSES {
        driver.click_at(plus)?;
        session.settle(10);
    }
    // The raster is debounced, and a deep-zoom region render re-interprets the
    // whole content stream — so the wait is generous. ★ At 90 frames the run
    // ended with the top rung's raster still in flight, which reads in the
    // trace as `drawn=0` and is indistinguishable from a page that cannot be
    // drawn at all. A settle that is too short does not fail the check; it
    // makes its evidence ambiguous, which is worse.
    session.settle(240);

    let trace = session.trace()?;
    let zoom = trace
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0);
    if zoom < MUST_EXCEED {
        return Err(Error::new(format!(
            "after {PRESSES} presses the zoom is {zoom}x, below the {MUST_EXCEED}x where a Letter \
             page's whole-page raster gives out — so the region tier was never exercised. Either \
             the maximum-zoom setting is capping this run or the button was missed. SKIPPED \
             rather than passed."
        )));
    }
    report.note(format!("zoomed to {:.0}%", zoom * 100.0));

    // --- the assertion ------------------------------------------------------
    let failures = trace
        .events(RENDER_EVENT)
        .filter(|l| l.get("outcome").is_some_and(|o| o == "failed"))
        .count();
    if failures > 0 {
        return Ok(Some(format!(
            "★★ {failures} RENDER(S) FAILED at {:.0}% zoom. This is the operator's own report — \
             \"requested raster size 14580x18868 is empty or exceeds MAX_PIXMAP_EDGE\" — which \
             means the whole page is still being rasterized past the ceiling. Check that \
             `canvas::show` sets `doc.raster_region` from `render::strategy::for_page`: every \
             piece of the region tier can be present and correct while nothing calls it, which \
             is exactly how this shipped.",
            zoom * 100.0
        )));
    }

    let rasters = trace
        .events(RENDER_EVENT)
        .filter(|l| l.get("outcome").is_some_and(|o| o == "done"))
        .count();
    if rasters == 0 {
        return Ok(Some(format!(
            "no render completed at {:.0}% zoom. Nothing failed either, which means the canvas \
             stopped ASKING — a page that is never requested cannot report a raster-size refusal, \
             so this would pass a failure count and still show the operator a blank sheet.",
            zoom * 100.0
        )));
    }
    report.note(format!("{rasters} raster(s) completed, none failed"));

    Ok(None)
}
