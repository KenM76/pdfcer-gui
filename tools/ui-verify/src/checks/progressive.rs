//! `progressive` — **the page is never blank while a sharper one is on its way.**
//!
//! Driven through the scripted pointer on a window placed off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/progressive.md`.

use crate::checks::scripteddrive;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The canvas viewport's published region.
const CANVAS_REGION: &str = "canvas-viewport";
/// The canvas trace line, which carries the zoom and the rect.
const CANVAS_EVENT: &str = "canvas";
/// Notches per zoom batch, and how many batches to climb.
const ZOOM_BATCH: usize = 6;
/// How many batches to climb before giving up on reaching the region tier.
const MAX_ZOOM_BATCHES: usize = 30;
/// Wheel notches for the pan. Negative is one direction; the sign does not
/// matter, only that it is far enough to leave the overscan.
const PAN_NOTCHES: i32 = -40;
/// Notches to zoom back out by in phase C.
const ZOOM_OUT_NOTCHES: usize = 24;
/// The application's own report of how much of the view it drew.
const COVERAGE_EVENT: &str = "canvas-coverage";
/// The least of the view that may be left undrawn at any moment.
const MIN_COVERED: f64 = 0.5;
/// How far to zoom in before the gestures.
const CLIMB_TO: f64 = 20.0;
/// How many frames to allow the render to finish before the reference capture.
const SETTLE_TO_FINISH: u32 = 90;
/// The fewest ink pixels a view must carry for this check to mean anything.
const MIN_INK: usize = 2_000;

/// Which gesture a phase performs.
#[derive(Clone, Copy)]
enum Gesture {
    /// Wheel, no modifier.
    Pan,
    /// Ctrl+wheel, downward.
    ZoomOut,
}

/// The page is never blank while a sharper one is on its way.
pub struct ProgressiveRenderNeverGoesBlank;

impl Check for ProgressiveRenderNeverGoesBlank {
    fn name(&self) -> &'static str {
        "progressive"
    }

    fn defect(&self) -> &'static str {
        "panning or zooming out at high magnification blanks the page until the new raster \
         arrives — the operator loses sight of the drawing for as long as it takes to render, \
         on exactly the gesture they use to find their way around it"
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

/// The live zoom.
fn zoom_now(trace: &Trace) -> f64 {
    trace
        .last(CANVAS_EVENT)
        .and_then(|l| l.get("zoom"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

/// Every coverage ratio the application has published, in order.
fn coverage_samples(trace: &Trace) -> Vec<f64> {
    trace
        .events(COVERAGE_EVENT)
        .filter_map(|l| l.get("covered"))
        .filter_map(|v| v.parse().ok())
        .collect()
}

/// The window's own frame, and return the image.
fn capture(
    session: &Session,
    pointer: &ScriptedPointer,
    ctx: &CheckContext,
    name: &str,
) -> Result<crate::image::Image> {
    scripteddrive::shot(session, pointer, &ctx.out(name))
        .map_err(|e| Error::new(format!("the window's frame could not be taken: {e}")))
}

/// `times` Ctrl+wheel lines of `dy` at `at`.
fn zoom_wheel(
    session: &Session,
    pointer: &ScriptedPointer,
    at: crate::coords::WindowPoint,
    dy: f32,
    times: usize,
) -> Result<()> {
    for _ in 0..times {
        pointer.wheel(session, at, dy, true)?;
    }
    Ok(())
}

/// How many sampled canvas pixels carry **ink** — anything appreciably darker
/// than paper.
fn ink(img: &crate::image::Image, region: crate::geom::PixRect) -> usize {
    img.pixels_in(region)
        // 200 is well below page white (measured 248) and well above any line
        // colour on a technical drawing. The exact value is not delicate: the
        // comparison below is between two captures of the SAME view, so a
        // threshold that mis-classifies a pixel mis-classifies it in both.
        .filter(|p| u32::from(p.r) + u32::from(p.g) + u32::from(p.b) < 200 * 3)
        .count()
}

#[allow(
    clippy::too_many_lines,
    reason = "one driven sequence; the ORDER is the subject" // ui-text-exempt: lint justification
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Pass a DENSE drawing — the check measures what is on screen before a \
             raster arrives, and on a sparse page the raster arrives too quickly for the \
             interval under test to exist.",
        )
    })?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("progressive.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = scripteddrive::offscreen(ctx, &mut spec, "progressive")?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(
            "the trace has no start line, so the diagnostic switch did not reach the process.",
        ));
    }
    let canvas = crate::checks::driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;

    // **The wheel is aimed at CONTENT, not at the canvas centre**, and the
    // first run of this check is why. Ctrl+wheel is zoom-about-the-pointer, so
    // whatever is under the cursor is what stays on screen; aiming at the
    // middle of the canvas zoomed a dense CAD sheet to 3590 % over a patch of
    // empty paper, and the check correctly refused to proceed — 99.98 % one
    // colour before any gesture had been made.
    //
    // That refusal was the right behaviour and it is also the trap: a check
    // that measures blankness cannot tell a blanked canvas from one that was
    // legitimately empty, so it has to establish content FIRST or it is
    // measuring nothing. Hence `--doc-point`, and hence this being a hard
    // requirement rather than a default.
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point with CONTENT on \
             it — a title block, a run of text, a dense corner. There is deliberately no \
             default: zooming at the canvas centre lands on empty paper on most drawings, \
             and a check that measures blankness cannot tell empty paper from a blanked \
             canvas.",
        )
    })?;
    let page = match ctx.page_size {
        Some((w, h)) => crate::coords::PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    let mapping =
        crate::coords::CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let centre = mapping.doc_to_window(crate::coords::DocPoint::new(
        target.page,
        target.x,
        target.y,
    ))?;
    report.note(format!(
        "aiming the wheel at PDF ({}, {}) on page {}",
        target.x, target.y, target.page
    ));

    // --- A: climb until region rendering is in force -----------------------
    //
    // The target is expressed as a zoom rather than as a strategy, because
    // the strategy is not traced. What matters for this check is only that the
    // rasters have become slow and partial, which is true well before the
    // formal region tier.
    let mut batches = 0;
    while batches < MAX_ZOOM_BATCHES {
        let z = zoom_now(&session.trace()?);
        if z >= CLIMB_TO {
            break;
        }
        zoom_wheel(&session, &pointer, centre, 1.0, ZOOM_BATCH)?;
        session.settle(4);
        batches += 1;
    }
    session.settle(60);
    let trace = session.trace()?;
    let zoom = zoom_now(&trace);
    report.note(format!("climbed to zoom {:.0}%", zoom * 100.0));
    if zoom < CLIMB_TO * 0.5 {
        return Err(Error::new(format!(
            "only reached {:.0}%, which is not deep enough for a raster to be slow. Either the \
             wheel is not reaching the canvas or this build caps the zoom lower.",
            zoom * 100.0
        )));
    }
    let px = frame.logical_to_capture_pixels(canvas);
    let settled_ink = ink(
        &capture(&session, &pointer, ctx, "progressive-before.png")?,
        px,
    );
    report.note(format!(
        "{settled_ink} ink pixel(s) on the canvas before the gesture"
    ));
    if settled_ink < MIN_INK {
        return Err(Error::new(format!(
            "only {settled_ink} ink pixel(s) on the canvas before any gesture — the view is over \
             blank paper, so nothing below would mean anything. Aim --doc-point at content."
        )));
    }

    // --- B and C: the two gestures he named --------------------------------
    for (label, gesture) in [("pan", Gesture::Pan), ("zoom out", Gesture::ZoomOut)] {
        let before_frames = coverage_samples(&session.trace()?);
        match gesture {
            Gesture::Pan => {
                pointer.wheel(&session, centre, PAN_NOTCHES as f32, false)?;
            }
            Gesture::ZoomOut => {
                zoom_wheel(&session, &pointer, centre, -1.0, ZOOM_OUT_NOTCHES)?;
            }
        }
        // Let it finish. There is no shutter to beat — the trace already holds
        // every frame, so waiting costs only time and buys a complete record.
        session.settle(SETTLE_TO_FINISH);
        let after = session.trace()?;
        let samples: Vec<f64> = coverage_samples(&after)
            .into_iter()
            .skip(before_frames.len())
            .collect();
        if samples.is_empty() {
            return Err(Error::new(format!(
                "the {label} produced no `{COVERAGE_EVENT}` line, so either the view did not move \
                 or this build does not publish its coverage. Nothing to judge."
            )));
        }
        let worst = samples.iter().copied().fold(f64::INFINITY, f64::min);
        let blanked = samples.iter().filter(|c| **c < MIN_COVERED).count();
        report.note(format!(
            "{label}: {} coverage sample(s), worst {:.1}% of the view drawn, {blanked} below \
             {:.0}%",
            samples.len(),
            worst * 100.0,
            MIN_COVERED * 100.0
        ));

        if worst < MIN_COVERED {
            return Ok(Some(format!(
                "★ after the {label} at {:.0}%, the page was drawn over only {:.1}% of the view \
                 at its worst, and stayed below {:.0}% for {blanked} frame(s). The operator is \
                 looking at blank paper while the new area renders — their words: \"the screen \
                 should never be blank while waiting to render when zooming out\". What is wanted \
                 is a low-resolution stand-in covering the whole page until the sharp raster \
                 lands.",
                zoom * 100.0,
                worst * 100.0,
                MIN_COVERED * 100.0
            )));
        }
        report.note(format!("{label}: the page stayed covered throughout"));
    }
    pointer.gone(&session)?;
    Ok(None)
}
