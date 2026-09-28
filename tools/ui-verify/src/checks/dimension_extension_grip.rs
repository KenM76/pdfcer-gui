//! `an_extension_line_grip_shortens_its_line` — a selected linear ce
//! dimension shows a grip at the near end of each extension line; dragging it
//! towards the dimension line previews the shortened line with the button
//! down and the release shortens it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_extension_grip.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint, WindowFrame};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::pixels;
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode the dimension is drawn in.
const MODE: &str = "review";
/// The Measure tab.
const TAB: &str = "ribbon.tab.measure";
/// The ribbon item that arms the Linear tool.
const ITEM: &str = "ribbon.item.measure.linear";
/// `measure-tool tool=…` — the canvas reporting what armed.
const ARM_EVENT: &str = "measure-tool";
/// The `Debug` spelling of `CanvasTool::Measure(MeasureKind::Linear)`.
const ARM_VALUE: &str = "Measure(Linear)";
/// `add-dimension …` — the engine accepted the dimension.
const COMMIT_EVENT: &str = "add-dimension";
/// The grip of end A's extension line.
const GRIP: &str = "canvas.dimension-extension.0";
/// `dim-preview baked=1 …` / `baked=0 refused=…` — the painter's account.
const PREVIEW_EVENT: &str = "dim-preview";
/// `dimension-extension-gap id=… end=… gap=…` — the canvas's release.
const RELEASE_EVENT: &str = "dimension-extension-gap";
/// The funnel's success line for the engine verb.
const APPLIED_EVENT: &str = "set-dimension-extension-gap";
/// The three clicks that place the dimension, as fractions of the page box
/// (y up): A, B, and where the dimension line sits. Blank paper on
/// `blank-overhang.pdf`, below its frame.
const PICKS: [(f64, f64); 3] = [(0.10, 0.15), (0.45, 0.15), (0.35, 0.30)];
/// How much the check zooms in before reading pixels: at fit-page an
/// extension line is a fraction of a pixel wide and reads as paper.
const ZOOM_FACTOR: f32 = 3.0;
const ZOOM_SPINS: usize = 30;
/// How far along the extension line the grip is dragged, as a fraction of
/// the distance from A to the dimension line.
const DRAG_FRACTION: f32 = 0.5;
/// Logical points kept clear of each end of the read stretch, so neither the
/// grip handle nor the line's new start falls inside it.
const END_CLEAR: f32 = 6.0;
/// Half the width of the read stretch, logical points.
const HALF_WIDTH: f32 = 2.0;
/// The ink fraction above which the stretch counts as carrying a line.
const LINED: f64 = 0.05;
/// The ink fraction below which it counts as blank paper.
const BLANK: f64 = 0.01;

/// See the module documentation.
pub struct AnExtensionLineGripShortensItsLine;

impl Check for AnExtensionLineGripShortensItsLine {
    fn name(&self) -> &'static str {
        "an_extension_line_grip_shortens_its_line"
    }

    fn defect(&self) -> &'static str {
        "a linear ce dimension's extension lines could not be lengthened or shortened on the \
         canvas. His ask, verbatim: \"when we click on a dimension, the end points on the \
         connection side of the dimension can have their lengths adjusted\""
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

/// The ink fraction of a logical-space box, and a one-line account of it.
fn ink(img: &Image, frame: &WindowFrame, r: LRect) -> (f64, String) {
    let report = pixels::ink_run_into(img, frame.logical_to_capture_pixels(r));
    #[allow(clippy::cast_precision_loss)]
    let fraction = report.ink as f64 / report.sampled.max(1) as f64;
    (fraction, report.summary())
}

/// The box round the part of a line from `a` to `b` (window space) that
/// excludes [`END_CLEAR`] at each end. `None` when the line is too short.
fn stretch(a: Pt, b: Pt) -> Option<LRect> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.hypot(dy);
    if len <= 2.0 * END_CLEAR + 2.0 {
        return None;
    }
    let (ux, uy) = (dx / len, dy / len);
    let p = Pt::new(a.x + ux * END_CLEAR, a.y + uy * END_CLEAR);
    let q = Pt::new(b.x - ux * END_CLEAR, b.y - uy * END_CLEAR);
    Some(LRect::new(
        Pt::new(p.x.min(q.x) - HALF_WIDTH, p.y.min(q.y) - HALF_WIDTH),
        Pt::new(p.x.max(q.x) + HALF_WIDTH, p.y.max(q.y) + HALF_WIDTH),
    ))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a dimension and drags its grip. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("dimension_extension_grip.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Review, Measure, Linear ----------------------------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, TAB) else {
        return Ok(Some(format!(
            "the `{MODE}` mode declares no `{TAB}` region."
        )));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)? else {
        return Ok(Some(format!(
            "the Measure tab declares no `{ITEM}`. Items declared: {}.",
            list(&crate::checks::driving::declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.measure."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    let trace = session.trace()?;
    if !trace
        .events(ARM_EVENT)
        .any(|l| l.get("tool") == Some(ARM_VALUE))
    {
        return Ok(Some(format!(
            "the Linear item was clicked and no `{ARM_EVENT} tool={ARM_VALUE}` followed."
        )));
    }

    // --- 2: place the dimension ---------------------------------------------
    if trace
        .last("canvas")
        .and_then(|l| l.get("page"))
        .and_then(|v| v.parse::<usize>().ok())
        != Some(0)
    {
        return Err(Error::new(
            "the canvas is not showing page 1, so the page geometry does not describe what is \
             on screen.",
        ));
    }
    let frame = session.frame()?;
    let window = |mapping: &CanvasMapping, fx: f64, fy: f64| -> Result<Pt> {
        let w = mapping.doc_to_window(DocPoint::new(0, fx * page.width_pt, fy * page.height_pt))?;
        Ok(Pt::new(w.x(), w.y()))
    };
    // A window-space point as a screen point.
    let screen = |p: Pt| frame.declared_center(LRect::new(p, p));
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    for &(fx, fy) in &PICKS {
        driver.click_at(screen(window(&mapping, fx, fy)?))?;
        session.settle(14);
    }
    session.settle(20);
    if session.trace()?.events(COMMIT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "the three picks placed no dimension, so there is no grip to drag. That is \
             `measure_linear`'s subject — run it first. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 3: zoom in on end A's extension line, then select ------------------
    let centre = screen(window(
        &mapping,
        PICKS[0].0,
        f64::midpoint(PICKS[0].1, PICKS[2].1),
    )?);
    let start = crate::checks::scale_aim::current_zoom(&session)?;
    for _ in 0..ZOOM_SPINS {
        if crate::checks::scale_aim::current_zoom(&session)? >= start * ZOOM_FACTOR {
            break;
        }
        driver.scroll_at_held(centre, &[vk::CONTROL], 1, 1)?;
        session.settle(10);
    }
    session.settle(40);
    report.note(format!(
        "zoomed from {start:.3} to {:.3}",
        crate::checks::scale_aim::current_zoom(&session)?
    ));
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        driver.press(vk::V)?;
        session.settle(12);
    }
    let on_line = screen(window(&mapping, 0.20, PICKS[2].1)?);
    driver.click_at(on_line)?;
    session.settle(18);
    let trace = session.trace()?;
    let Some(grip) = declared(&trace, ui_rect, GRIP) else {
        return Ok(Some(format!(
            "★ NO EXTENSION-LINE GRIP. The dimension was clicked on its line and no `{GRIP}` \
             region was published. Either the click did not select it, or `canvas::dimext::grips` \
             offered nothing. Regions declared under `canvas.dimension`: {}.",
            list(&crate::checks::driving::declared_names(
                &trace,
                ui_rect,
                "canvas.dimension"
            ))
        )));
    };
    let grip_at = Pt::new(
        (grip.min.x + grip.max.x) / 2.0,
        (grip.min.y + grip.max.y) / 2.0,
    );
    // The foot of end A's extension line on the dimension line.
    let foot = window(&mapping, PICKS[0].0, PICKS[2].1)?;
    let to_at = Pt::new(
        grip_at.x + (foot.x - grip_at.x) * DRAG_FRACTION,
        grip_at.y + (foot.y - grip_at.y) * DRAG_FRACTION,
    );
    let Some(cut) = stretch(grip_at, to_at) else {
        return Err(Error::new(
            "the drag is too short on screen to read the stretch it removes. SKIPPED — the \
             window is too small for this check's geometry.",
        ));
    };
    let before_shot = ctx.out("dimension_extension_grip_before.png");
    let before = crate::capture::window_to_png(&session, &before_shot)?;
    report.artifact(before_shot);
    let (was, was_account) = ink(&before, &frame, cut);
    if was < LINED {
        return Ok(Some(format!(
            "the stretch {cut:?} the drag should remove carries no line before the drag \
             ({was_account}), so its absence afterwards would prove nothing. The grip is at \
             {grip_at:?} and the extension line should run from it to {foot:?}."
        )));
    }

    // --- 4: drag the grip along its line, button down -----------------------
    let from: ScreenPoint = screen(grip_at);
    let to: ScreenPoint = screen(to_at);
    let during_shot = ctx.out("dimension_extension_grip_during.png");
    let (during, grip_during) = driver.drag_observed(from, to, || {
        let image = crate::capture::window_to_png(&session, &during_shot)?;
        Ok((image, declared(&session.trace()?, ui_rect, GRIP)))
    })?;
    report.artifact(during_shot);
    // With the button down the committed line must already be gone from the
    // stretch: the preview draws over a render that omits the committed
    // dimension.
    let (mid, mid_account) = ink(&during, &frame, cut);
    if mid >= LINED {
        return Ok(Some(format!(
            "★ THE SHORTENING DID NOT SHOW DURING THE DRAG. With the button down the stretch \
             {cut:?} still carries {mid_account} (before: {was_account}), so the committed line \
             shows through the preview. `canvas::dimpreview::underlay` covers it."
        )));
    }
    report.note(format!(
        "mid-drag the removed stretch is clear ({was_account} → {mid_account})"
    ));
    let Some(moved) = grip_during else {
        return Ok(Some(format!(
            "★ THE GRIP VANISHED MID-DRAG: no `{GRIP}` region was declared with the button down."
        )));
    };
    let travelled = Pt::new(
        (moved.min.x + moved.max.x) / 2.0 - grip_at.x,
        (moved.min.y + moved.max.y) / 2.0 - grip_at.y,
    );
    let asked = Pt::new(to_at.x - grip_at.x, to_at.y - grip_at.y);
    let along = (travelled.x * asked.x + travelled.y * asked.y) / asked.x.hypot(asked.y);
    if along < 0.5 * asked.x.hypot(asked.y) {
        return Ok(Some(format!(
            "★ THE GRIP DID NOT FOLLOW THE DRAG. With the button down it sat {along:.1} px \
             along a {:.1} px drag, so a shortening drag shows the operator nothing until \
             release. `canvas::dimext::grips_drawn` draws the live grip.",
            asked.x.hypot(asked.y)
        )));
    }
    report.note(format!(
        "the grip followed the pointer {along:.1} px of a {:.1} px drag",
        asked.x.hypot(asked.y)
    ));
    let trace = session.trace()?;
    if let Some(refused) = trace
        .events(PREVIEW_EVENT)
        .find(|l| l.get("baked") == Some("0"))
    {
        return Ok(Some(format!(
            "★ THE ENGINE REFUSED TO BAKE THE CHANGED GAP: `{}`.",
            refused.raw
        )));
    }
    let Some(baked) = trace
        .events(PREVIEW_EVENT)
        .filter(|l| l.get("baked") == Some("1"))
        .last()
    else {
        return Ok(Some(format!(
            "★ NO LIVE PREVIEW WHILE THE GRIP WAS DRAGGED: no `{PREVIEW_EVENT} baked=1` line. \
             The press did not reach `canvas::dimext::drag`, or `dimpreview::paint` declined. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the drag was previewed: `{}`", baked.raw));

    // --- 5: release — the stretch is gone -----------------------------------
    session.settle(30);
    let trace = session.trace()?;
    let Some(release) = trace.last(RELEASE_EVENT) else {
        return Ok(Some(format!(
            "★ THE RELEASE ASKED FOR NOTHING: no `{RELEASE_EVENT}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if trace.last(APPLIED_EVENT).is_none() {
        return Ok(Some(format!(
            "the release asked for `{}` and the engine did not apply it: no `{APPLIED_EVENT}` \
             line. A `{APPLIED_EVENT}-refused` line carries its reason: {:?}.",
            release.raw,
            trace
                .last(&format!("{APPLIED_EVENT}-refused"))
                .map(|l| l.raw.clone())
        )));
    }
    // Deselected first, so no handle or outline is read as the line.
    driver.press(vk::ESCAPE)?;
    session.settle(18);
    let after_shot = ctx.out("dimension_extension_grip_after.png");
    let after = crate::capture::window_to_png(&session, &after_shot)?;
    report.artifact(after_shot);
    let (now, now_account) = ink(&after, &frame, cut);
    if now >= BLANK {
        return Ok(Some(format!(
            "★ THE EXTENSION LINE STILL COVERS THE STRETCH THE DRAG REMOVED. `{}` was applied, \
             and {cut:?} went from {was_account} to {now_account}.",
            release.raw
        )));
    }
    report.note(format!(
        "`{}` shortened end A's extension line: {cut:?} went from {was_account} to \
         {now_account}",
        release.raw
    ));
    Ok(None)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn the_stretch_keeps_clear_of_both_ends() {
        let r = stretch(Pt::new(10.0, 10.0), Pt::new(10.0, 60.0)).unwrap();
        assert!((r.min.y - (10.0 + END_CLEAR - HALF_WIDTH)).abs() < 1e-4);
        assert!((r.max.y - (60.0 - END_CLEAR + HALF_WIDTH)).abs() < 1e-4);
        assert!(stretch(Pt::new(0.0, 0.0), Pt::new(0.0, 5.0)).is_none());
    }
}
