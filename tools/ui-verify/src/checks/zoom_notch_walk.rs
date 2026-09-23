//! `zooming_click_by_click_keeps_the_detail_under_the_cursor` — O231.
//!
//! Ctrl+wheel one notch at a time with the pointer held still over a detail,
//! all the way in and all the way back out, recording at every notch which page
//! point is under the pointer and photographing the patch around it
//! twice: shortly after the notch, and once rendering has settled.
//!
//! # Why the existing zoom checks cannot see O231
//!
//! `the_page_still_renders_at_every_decade_of_zoom` photographs at decades and
//! **re-aims the pointer** at the target before each photograph, so a view
//! that jumped is silently followed; and it asks whether the whole canvas is
//! blank, not the part under the pointer. `zooming_does_not_throw_away_where_
//! the_operator_panned` measures the position eight notches at a time and
//! never looks at pixels. A transition between two samples is invisible to
//! both — this walks every notch.
//!
//! # What fails it
//!
//! - **Drift:** the page point under the pointer, computed in `f64` from the
//!   `canvas-pos` line, moves by more than [`DRIFT_PX`] screen points across
//!   one notch. Zoom-to-cursor holds that point fixed by definition. Measured
//!   notch to notch rather than against `--doc-point`, because the harness's
//!   initial aim carries its own error and a constant page offset times a
//!   growing zoom reads as a runaway it is not.
//! - **An order for the wrong place:** on any `scroll`-tier frame, the region
//!   the shell wants rendered (`canvas-pos want=`) does not contain what the
//!   viewport shows at that frame's zoom and pan. A region ordered from the
//!   previous frame's offset against a new zoom names a place `offset / zoom`
//!   points away; if the debounce lets it through, the raster that arrives is
//!   painted at its own region and the view under the pointer is blank.
//! - **A white-out that recovers on the way out:** the settled patch under the
//!   pointer is uniformly paper at a zoom where, walking back out, the settled patch at
//!   the same zoom was not. The same view at the same zoom must paint the same
//!   thing whichever direction it was reached from; a patch that is blank only
//!   on the way in is the canvas losing a raster, not the drawing being empty.
//!
//! Every capture is kept as an artifact, named by direction, notch and zoom.

use crate::checks::driving;
use crate::checks::raster_wall::panic_was_converted;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

const VK_CONTROL: u16 = 0x11;
const CANVAS_REGION: &str = "canvas-viewport";
const CANVAS_EVENT: &str = "canvas";
const POS_EVENT: &str = "canvas-pos";

/// Largest tolerated move, in logical points at the new zoom, of the page point
/// under the pointer across one zoom step. Screen units, because the defect is
/// seen on screen.
///
/// Above the floor both walks measure on SW41177 with the pointer still: up to
/// about 5 points per step, from the scroll area rounding the drawn page to
/// whole pixels on every animation frame plus the harness reading the
/// position from the trace at both ends. Carrying the anchor fraction across
/// frames did not move that figure. A region ordered for the wrong place, the
/// defect this guards, is off by whole screens.
pub(super) const DRIFT_PX: f64 = 12.0;

/// Half the side of the photographed patch around the pointer, logical points.
pub(super) const PATCH: f32 = 48.0;

/// Stop climbing at this zoom (a ratio, 1.0 = 100 %). Deep enough to cross the
/// whole-page raster ceiling, the region tier and into the `f64` tier, which
/// begins near 650 on an A1 sheet.
const CEILING: f32 = 2.0e3;

/// The most notches in either direction, a runaway backstop.
const MAX_NOTCHES: usize = 160;

/// Frames to wait before the early photograph, and before the settled one.
const EARLY_FRAMES: u32 = 3;
const SETTLED_FRAMES: u32 = 120;

pub struct ZoomingClickByClickKeepsTheDetailUnderTheCursor;

impl Check for ZoomingClickByClickKeepsTheDetailUnderTheCursor {
    fn name(&self) -> &'static str {
        "zooming_click_by_click_keeps_the_detail_under_the_cursor"
    }

    fn defect(&self) -> &'static str {
        "at some zoom step the area under the cursor goes white and the view seems to jump, \
         while zooming back out lands on the detail again (O231)"
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

/// One notch's record.
struct Step {
    zoom: f32,
    under: Option<(f64, f64)>,
    early_uniform: bool,
    settled_uniform: bool,
}

pub(super) fn zoom_now(session: &Session) -> Result<f32> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0))
}

/// The page point, in PDF points with y up, under window point `aim`, from the
/// `f64` pan line.
pub(super) fn page_under(
    session: &Session,
    canvas: LRect,
    aim: (f64, f64),
) -> Result<Option<(f64, f64)>> {
    let trace = session.trace()?;
    let Some(zoom) = trace
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
    else {
        return Ok(None);
    };
    let Some(line) = trace.events(POS_EVENT).last() else {
        return Ok(None);
    };
    let pair = |k: &str| -> Option<(f64, f64)> {
        let (x, y) = line.get(k)?.split_once(',')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    };
    let (Some(at), Some(ext)) = (pair("at"), pair("ext")) else {
        return Ok(None);
    };
    let z = f64::from(zoom);
    Ok(Some((
        (at.0 + aim.0 - f64::from(canvas.min.x)) / z,
        ext.1 - (at.1 + aim.1 - f64::from(canvas.min.y)) / z,
    )))
}

fn patch_uniform(
    ctx: &CheckContext,
    session: &Session,
    frame: &crate::coords::WindowFrame,
    patch: LRect,
    report: &mut CheckReport,
    label: &str,
) -> Result<bool> {
    let path = ctx.out(&format!("notch-walk-{label}.png"));
    let image = crate::capture::window_to_png(session, &path)?;
    report.artifact(path);
    Ok(blank(&image, frame, patch))
}

/// Luminance above which a uniform patch is paper, not ink.
const PAPER: f64 = 0.5;

/// Whether `patch` is uniformly paper-coloured in `image`.
///
/// Uniform alone is not blank: from about 30,000% a hairline is wider than
/// the patch, and a patch wholly inside it is uniformly black.
pub(super) fn blank(
    image: &crate::image::Image,
    frame: &crate::coords::WindowFrame,
    patch: LRect,
) -> bool {
    let region = frame.logical_to_capture_pixels(patch);
    crate::pixels::region_not_uniform(image, region).is_uniform()
        && crate::pixels::mean_luminance(image, region).is_some_and(|l| l > PAPER)
}

/// The first `scroll`-tier frame whose wanted region does not contain the
/// viewport, as a failure message.
///
/// The viewport in page points is rebuilt from `canvas-pos at=` (the pan in
/// logical points from the page's top-left), the zoom of the `canvas` line
/// before it, and the canvas size. The page's y axis points up, so the top of
/// the view is `ext.y - at.y / zoom`. A tolerance of 1e-3 of the view's width
/// absorbs the region's snapping and the trace's rounding. Off-page content
/// widens the order past the sheet, never narrows it, so clipping the view to
/// the sheet cannot hide a miss.
pub(super) fn region_misses_view(trace: &crate::trace::Trace, canvas: LRect) -> Option<String> {
    let (w, h) = (
        f64::from(canvas.max.x - canvas.min.x),
        f64::from(canvas.max.y - canvas.min.y),
    );
    let mut zoom = None;
    let mut checked = 0usize;
    for line in &trace.lines {
        if line.event == CANVAS_EVENT {
            zoom = line.get_f32("zoom").map(f64::from);
            continue;
        }
        if line.event != POS_EVENT || line.get("tier") != Some("scroll") {
            continue;
        }
        let quad = |k: &str| -> Option<Vec<f64>> {
            let v: Vec<f64> = line
                .get(k)?
                .split(',')
                .map(str::parse)
                .collect::<std::result::Result<_, _>>()
                .ok()?;
            Some(v)
        };
        let (Some(z), Some(at), Some(ext), Some(want)) =
            (zoom, quad("at"), quad("ext"), quad("want"))
        else {
            continue;
        };
        if want.len() != 4 || at.len() != 2 || ext.len() != 2 || z <= 0.0 {
            continue;
        }
        checked += 1;
        // Clipped to the sheet: the region tier orders only what of the page
        // is visible, so a view hanging past an edge owes nothing out there.
        let view = [
            (at[0] / z).max(0.0),
            (ext[1] - (at[1] + h) / z).max(0.0),
            ((at[0] + w) / z).min(ext[0]),
            (ext[1] - at[1] / z).min(ext[1]),
        ];
        let slack = (view[2] - view[0]) * 1e-3;
        let covers = want[0] <= view[0] + slack
            && want[1] <= view[1] + slack
            && want[2] >= view[2] - slack
            && want[3] >= view[3] - slack;
        if !covers {
            return Some(format!(
                "trace line {} at {:.0}%: the region ordered ({:.3},{:.3})-({:.3},{:.3}) does not \
                 contain the view ({:.3},{:.3})-({:.3},{:.3}) ({checked} region frames checked)",
                line.lineno,
                z * 100.0,
                want[0],
                want[1],
                want[2],
                want[3],
                view[0],
                view[1],
                view[2],
                view[3],
            ));
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn notch(
    ctx: &CheckContext,
    session: &Session,
    driver: &Driver,
    frame: &crate::coords::WindowFrame,
    at: crate::coords::ScreenPoint,
    aim: (f64, f64),
    canvas: LRect,
    patch: LRect,
    direction: i32,
    n: usize,
    report: &mut CheckReport,
) -> Result<Step> {
    driver.scroll_at_held(at, &[VK_CONTROL], direction, 1)?;
    session.settle(EARLY_FRAMES);
    let zoom = zoom_now(session)?;
    let dir = if direction > 0 { "in" } else { "out" };
    let label = format!("{dir}-{n:03}-{:.0}pct", zoom * 100.0);
    let early_uniform = patch_uniform(
        ctx,
        session,
        frame,
        patch,
        report,
        &format!("{label}-early"),
    )?;
    session.settle(SETTLED_FRAMES);
    let zoom = zoom_now(session)?;
    let settled_uniform = patch_uniform(
        ctx,
        session,
        frame,
        patch,
        report,
        &format!("{label}-settled"),
    )?;
    let under = page_under(session, canvas, aim)?.filter(|(x, y)| x.is_finite() && y.is_finite());
    report.note(format!(
        "{dir} {n:03}: zoom {:.1}% under pointer {} early {} settled {}",
        zoom * 100.0,
        under.map_or_else(|| "?".to_owned(), |(x, y)| format!("({x:.4}, {y:.4})")),
        if early_uniform { "BLANK" } else { "ink" },
        if settled_uniform { "BLANK" } else { "ink" },
    ));
    Ok(Step {
        zoom,
        under,
        early_uniform,
        settled_uniform,
    })
}

/// A launched session with the pointer held over `--doc-point`, shared by the
/// zoom checks that photograph the patch under the pointer.
pub(super) struct Rig {
    pub session: Session,
    pub driver: Driver,
    pub frame: crate::coords::WindowFrame,
    pub canvas: LRect,
    /// The pointer, in window logical points.
    pub aim: (f64, f64),
    pub at: crate::coords::ScreenPoint,
    /// [`PATCH`] either side of the pointer.
    pub patch: LRect,
}

/// Launch on `--pdf`, find the canvas, and park the pointer over `--doc-point`.
///
/// # Errors
/// Missing binary, PDF, target or page size; input disabled (a SKIP); launch
/// or trace failures.
pub(super) fn rig(ctx: &CheckContext, report: &mut CheckReport, trace: &str) -> Result<Rig> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| Error::new("no --pdf."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new("no --doc-point. This check needs a detail to hold under the pointer.")
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check zooms the canvas. SKIPPED, not passed.",
        ));
    }
    let ui_rect = vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf)
            .ok_or_else(|| Error::new("cannot read a page size. Pass --page-size WxH."))?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out(trace));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    session.expect_thread_panic();
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let wp = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let aim = (f64::from(wp.x()), f64::from(wp.y()));
    let at = frame.to_screen(wp);
    driver.move_to(at)?;
    let patch = LRect::new(
        Pt::new(wp.x() - PATCH, wp.y() - PATCH),
        Pt::new(wp.x() + PATCH, wp.y() + PATCH),
    );
    report.note(format!(
        "pointer held at window ({:.1}, {:.1}) over page {} ({}, {})",
        aim.0, aim.1, target.page, target.x, target.y
    ));
    Ok(Rig {
        session,
        driver,
        frame,
        canvas,
        aim,
        at,
        patch,
    })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let Rig {
        session,
        driver,
        frame,
        canvas,
        aim,
        at,
        patch,
    } = rig(ctx, report, "notch-walk.trace.txt")?;

    let mut inward = Vec::new();
    for n in 0..MAX_NOTCHES {
        let step = notch(
            ctx, &session, &driver, &frame, at, aim, canvas, patch, 1, n, report,
        )?;
        let top = step.zoom;
        inward.push(step);
        if top >= CEILING {
            break;
        }
    }
    let mut outward = Vec::new();
    let floor = inward.first().map_or(1.0, |s| s.zoom);
    for n in 0..MAX_NOTCHES {
        let step = notch(
            ctx, &session, &driver, &frame, at, aim, canvas, patch, -1, n, report,
        )?;
        let bottom = step.zoom;
        outward.push(step);
        if bottom <= floor {
            break;
        }
    }

    if let Some(bad) = panic_was_converted(&session)? {
        return Ok(Some(bad));
    }

    let mut failures = Vec::new();
    for (dir, steps) in [("in", &inward), ("out", &outward)] {
        for (n, pair) in steps.windows(2).enumerate() {
            let (Some(was), Some(now)) = (pair[0].under, pair[1].under) else {
                continue;
            };
            // Page points moved under the pointer, scaled to screen points.
            let d = (now.0 - was.0).hypot(now.1 - was.1) * f64::from(pair[1].zoom);
            if d > DRIFT_PX {
                failures.push(format!(
                    "{dir} notch {} at {:.1}%: the page moved {d:.1} pt under the pointer",
                    n + 1,
                    pair[1].zoom * 100.0
                ));
            }
        }
    }
    for (n, s) in inward.iter().enumerate() {
        if !s.settled_uniform {
            continue;
        }
        let back = outward
            .iter()
            .find(|o| (o.zoom / s.zoom - 1.0).abs() < 0.02 && !o.settled_uniform);
        if let Some(o) = back {
            failures.push(format!(
                "in notch {n} at {:.1}%: the patch under the pointer settled BLANK, and walking \
                 back out it had ink at {:.1}% — the same view painted differently by direction",
                s.zoom * 100.0,
                o.zoom * 100.0
            ));
        }
    }
    if let Some(bad) = region_misses_view(&session.trace()?, canvas) {
        failures.push(bad);
    }
    let early_blank = inward
        .iter()
        .chain(outward.iter())
        .filter(|s| s.early_uniform && !s.settled_uniform)
        .count();
    report.note(format!(
        "{} notches in, {} out; {early_blank} notch(es) blank early and inked once settled",
        inward.len(),
        outward.len()
    ));
    if failures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(failures.join("\n")))
    }
}
