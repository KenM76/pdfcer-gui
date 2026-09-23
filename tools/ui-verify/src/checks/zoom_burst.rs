//! `zooming_at_wheel_speed_never_blanks_the_detail_under_the_cursor` — O231.
//!
//! Ctrl+wheel in bursts of [`BURST`] notches [`GAP`] apart — a hand spinning
//! the wheel, not the one-notch-and-wait of
//! `zooming_click_by_click_keeps_the_detail_under_the_cursor` — with the pointer
//! held over `--doc-point`, all the way in to [`CEILING`](super::zoom_notch_walk)
//! and back out. After each burst the patch under the pointer is photographed
//! back to back for [`WATCH`], then once more when rendering has settled.
//!
//! # What fails it
//!
//! - **A white-out:** any photograph taken while the program catches up shows
//!   the patch uniformly paper-coloured, and the settled photograph of the same burst shows it
//!   inked. Between the last notch and the new raster the canvas owes the
//!   previous raster, magnified; a blank patch is a raster lost or painted
//!   somewhere else.
//! - **Drift:** the page point under the pointer moves by more than
//!   [`DRIFT_PX`] screen points at the settled zoom across one burst.
//! - **An order for the wrong place:** as the notch walk, over the whole trace.
//!
//! A run in which no burst settled inked has measured nothing and is an error,
//! not a pass: aim `--doc-point` at a stroke.

use std::time::{Duration, Instant};

use super::zoom_notch_walk::{
    DRIFT_PX, PATCH, Rig, blank, page_under, region_misses_view, rig, zoom_now,
};
use crate::checks::raster_wall::panic_was_converted;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::DocPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};

const VK_CONTROL: u16 = 0x11;

/// Notches per burst.
const BURST: usize = 5;

/// Between notches within a burst: a brisk spin of a detented wheel.
const GAP: Duration = Duration::from_millis(30);

/// How long after a burst to keep photographing.
const WATCH: Duration = Duration::from_millis(700);

/// Zoom ratio to climb to.
const CEILING: f32 = 2.0e3;

/// Runaway backstop, bursts per direction.
const MAX_BURSTS: usize = 40;

const SETTLED_FRAMES: u32 = 120;

pub struct ZoomingAtWheelSpeedNeverBlanksTheDetailUnderTheCursor;

impl Check for ZoomingAtWheelSpeedNeverBlanksTheDetailUnderTheCursor {
    fn name(&self) -> &'static str {
        "zooming_at_wheel_speed_never_blanks_the_detail_under_the_cursor"
    }

    fn defect(&self) -> &'static str {
        "spinning the wheel to zoom, the area under the cursor goes white at some zoom and \
         the view seems to jump (O231)"
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

/// One burst's record.
struct Burst {
    zoom: f32,
    /// How far the page moved under the pointer across the burst, in logical
    /// points at the new zoom.
    drift: Option<f64>,
    /// Milliseconds after the last notch of each uniform in-flight photograph.
    blank_at_ms: Vec<u128>,
    settled_inked: bool,
}

/// Whether the patch centred on `centre` is blank — see [`blank`].
fn patch_blank(image: &crate::image::Image, rig: &Rig, centre: (f64, f64)) -> bool {
    #[allow(clippy::cast_possible_truncation)]
    let (x, y) = (centre.0 as f32, centre.1 as f32);
    let patch = LRect::new(Pt::new(x - PATCH, y - PATCH), Pt::new(x + PATCH, y + PATCH));
    blank(image, &rig.frame, patch)
}

/// Where `target` is on screen now, in window logical points, from the last
/// `canvas-pos` line — `None` when it is not on the canvas with a patch's
/// margin to spare.
fn target_now(rig: &Rig, target: DocPoint) -> Result<Option<(f64, f64)>> {
    let trace = rig.session.trace()?;
    let Some(z) = trace
        .events("canvas")
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .map(f64::from)
    else {
        return Ok(None);
    };
    let Some(line) = trace.events("canvas-pos").last() else {
        return Ok(None);
    };
    let pair = |k: &str| -> Option<(f64, f64)> {
        let (x, y) = line.get(k)?.split_once(',')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    };
    let (Some(at), Some(ext)) = (pair("at"), pair("ext")) else {
        return Ok(None);
    };
    let c = rig.canvas;
    let w = (
        f64::from(c.min.x) + target.x * z - at.0,
        f64::from(c.min.y) + (ext.1 - target.y) * z - at.1,
    );
    let m = f64::from(PATCH);
    let inside = w.0 > f64::from(c.min.x) + m
        && w.0 < f64::from(c.max.x) - m
        && w.1 > f64::from(c.min.y) + m
        && w.1 < f64::from(c.max.y) - m;
    Ok(inside.then_some(w))
}

/// Where the program last saw the pointer, window logical points — from its
/// own `canvas-pointer` line, so the drift is measured at the pixel the
/// pointer really sits on and not at the harness's unrounded aim, whose
/// sub-pixel miss the zoom magnifies.
fn pointer_now(rig: &Rig) -> Result<Option<(f64, f64)>> {
    let trace = rig.session.trace()?;
    Ok(trace.events("canvas-pointer").last().and_then(|l| {
        let (x, y) = l
            .get("screen")?
            .trim_matches(|c| c == '(' || c == ')')
            .split_once(',')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    }))
}

fn burst(
    ctx: &CheckContext,
    rig: &Rig,
    target: DocPoint,
    direction: i32,
    n: usize,
    report: &mut CheckReport,
) -> Result<Burst> {
    let dir = if direction > 0 { "in" } else { "out" };
    // Re-aim so the target, not the harness's first estimate of it, is under
    // the pointer: at deep zoom a patch is a few page points across.
    let aim = target_now(rig, target)?.unwrap_or(rig.aim);
    #[allow(clippy::cast_possible_truncation)]
    let at = rig.frame.offset_from(
        rig.at,
        (aim.0 - rig.aim.0) as f32,
        (aim.1 - rig.aim.1) as f32,
    );
    rig.driver.move_to(at)?;
    rig.session.settle(4);
    let pointer = pointer_now(rig)?.unwrap_or(aim);
    let before = page_under(&rig.session, rig.canvas, pointer)?;

    rig.driver
        .scroll_burst(at, &[VK_CONTROL], direction, BURST, GAP)?;
    let t0 = Instant::now();
    // In memory and without raising, so samples come as fast as the screen
    // grab allows; written out only when they matter.
    let mut samples = Vec::new();
    while t0.elapsed() < WATCH {
        let ms = t0.elapsed().as_millis();
        samples.push((
            ms,
            crate::capture::frame_capture(&rig.session, &rig.frame, false)?,
        ));
    }
    rig.session.settle(SETTLED_FRAMES);
    let zoom = zoom_now(&rig.session)?;
    let path = ctx.out(&format!(
        "burst-{dir}-{n:02}-settled-{:.0}pct.png",
        zoom * 100.0
    ));
    let settled = crate::capture::window_to_png(&rig.session, &path)?;
    report.artifact(path);
    let settled_inked = !patch_blank(&settled, rig, aim);
    let mut blank_at_ms = Vec::new();
    for (ms, image) in &samples {
        if patch_blank(image, rig, aim) {
            blank_at_ms.push(*ms);
            let path = ctx.out(&format!("burst-{dir}-{n:02}-blank-{ms:04}ms.png"));
            image.save_png(&path)?;
            report.artifact(path);
        }
    }
    let under = page_under(&rig.session, rig.canvas, pointer)?
        .filter(|(x, y)| x.is_finite() && y.is_finite());
    let drift = match (before, under) {
        (Some(b), Some(u)) => Some((u.0 - b.0).hypot(u.1 - b.1) * f64::from(zoom)),
        _ => None,
    };
    report.note(format!(
        "{dir} {n:02}: zoom {:.1}% under pointer {} moved {} settled {} — {} samples, blank at {:?} ms",
        zoom * 100.0,
        under.map_or_else(|| "?".to_owned(), |(x, y)| format!("({x:.4}, {y:.4})")),
        drift.map_or_else(|| "?".to_owned(), |d| format!("{d:.1} pt")),
        if settled_inked { "ink" } else { "BLANK" },
        samples.len(),
        blank_at_ms,
    ));
    Ok(Burst {
        zoom,
        drift,
        blank_at_ms,
        settled_inked,
    })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let rig = rig(ctx, report, "burst.trace.txt")?;
    let floor = zoom_now(&rig.session)?;
    let target = ctx
        .target
        .ok_or_else(|| Error::new("no --doc-point to hold under the pointer."))?;

    let mut walk: Vec<(&str, Burst)> = Vec::new();
    for n in 0..MAX_BURSTS {
        let b = burst(ctx, &rig, target, 1, n, report)?;
        let top = b.zoom;
        walk.push(("in", b));
        if top >= CEILING {
            break;
        }
    }
    for n in 0..MAX_BURSTS {
        let b = burst(ctx, &rig, target, -1, n, report)?;
        let bottom = b.zoom;
        walk.push(("out", b));
        if bottom <= floor * 1.01 {
            break;
        }
    }

    if let Some(bad) = panic_was_converted(&rig.session)? {
        return Ok(Some(bad));
    }

    let mut failures = Vec::new();
    for (dir, b) in &walk {
        if let Some(d) = b.drift
            && d > DRIFT_PX
        {
            failures.push(format!(
                "{dir} burst to {:.1}%: the page moved {d:.1} pt under the pointer",
                b.zoom * 100.0
            ));
        }
        if b.settled_inked && !b.blank_at_ms.is_empty() {
            failures.push(format!(
                "{dir} burst to {:.1}%: the patch under the pointer went BLANK {:?} ms after \
                 the last notch, and settled inked",
                b.zoom * 100.0,
                b.blank_at_ms
            ));
        }
    }
    if let Some(bad) = region_misses_view(&rig.session.trace()?, rig.canvas) {
        failures.push(bad);
    }
    let measured = walk.iter().filter(|(_, b)| b.settled_inked).count();
    report.note(format!(
        "{} bursts, {measured} settled inked and so able to show a white-out",
        walk.len()
    ));
    if failures.is_empty() && measured == 0 {
        return Err(Error::new(
            "no burst settled with ink under the pointer, so no white-out could have been \
             seen. Aim --doc-point at a stroke.",
        ));
    }
    Ok((!failures.is_empty()).then(|| failures.join("\n")))
}
