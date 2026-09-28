//! `a_dimension_text_slides_alone` — pressing on a selected linear ce
//! dimension's text and dragging slides the text along the dimension line;
//! the line stays where it is, with the button down and after the release.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_text_slide.md`.

use crate::checks::dimension_label_drag::{ink, label_quad, quad_pixels};
use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
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
/// The selected linear ce dimension's text, screen bounds.
const LABEL: &str = "canvas.dimension-label"; // ui-text-exempt: trace region name
/// `dim-preview baked=1 … label=x0,y0,…` — the painter drew the engine's bake.
const PREVIEW_EVENT: &str = "dim-preview";
/// `dimension-label id=… offset=… text_along=…` — the canvas's release.
const RELEASE_EVENT: &str = "dimension-label";
/// `dimension-place …` — the body drag's release, which moves the line too.
const BODY_EVENT: &str = "dimension-place";
/// The funnel's success line for the engine verb.
const APPLIED_EVENT: &str = "place-dimension";
/// The canvas viewport region, the clip for every pixel rectangle.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
/// The three clicks that place the dimension, as fractions of the page box
/// (y up): A, B, and where the dimension line sits. Blank paper on
/// `blank-overhang.pdf`, below its frame.
const PICKS: [(f64, f64); 3] = [(0.10, 0.15), (0.45, 0.15), (0.35, 0.30)];
/// How much the check zooms in first: at fit-page the text is a few pixels
/// tall and a press cannot be aimed at it.
const ZOOM_FACTOR: f32 = 3.0;
const ZOOM_SPINS: usize = 30;
/// The drag, window points: along the line and across it. The across part is
/// the one that must be ignored.
const DRAG: (f32, f32) = (90.0, -40.0);
/// How far the text's centre may move across the line, window points.
const ACROSS_TOLERANCE: f32 = 2.0;
/// The least ink the text adds to its destination box mid-drag.
const INKED: f64 = 0.01;

/// See the module documentation.
pub struct ADimensionTextSlidesAlone;

impl Check for ADimensionTextSlidesAlone {
    fn name(&self) -> &'static str {
        "a_dimension_text_slides_alone"
    }

    fn defect(&self) -> &'static str {
        "dragging a linear ce dimension's text moved the dimension line with it. His ask, \
         verbatim: \"when I click on the dimension text and drag it should live preview so that \
         it is apparent I am just moving the dimension text\""
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

/// The centre of a window-space rectangle.
fn mid(r: LRect) -> Pt {
    Pt::new((r.min.x + r.max.x) / 2.0, (r.min.y + r.max.y) / 2.0)
}

/// The window-space centre of a page-space quad.
fn quad_centre(mapping: &CanvasMapping, quad: [(f64, f64); 4]) -> Result<Pt> {
    let (mut x, mut y) = (0.0_f32, 0.0_f32);
    for (px, py) in quad {
        let w = mapping.doc_to_window(DocPoint::new(0, px, py))?;
        x += w.x() / 4.0;
        y += w.y() / 4.0;
    }
    Ok(Pt::new(x, y))
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
            "input is disabled (--no-input). This check draws a dimension and drags its text. \
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("dimension_text_slide.trace.txt"));
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
            "the three picks placed no dimension, so there is no text to drag. That is \
             `measure_linear`'s subject — run it first. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 3: zoom in on the text, then select the dimension by its line ----
    let centre = screen(window(
        &mapping,
        f64::midpoint(PICKS[0].0, PICKS[1].0),
        PICKS[2].1,
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
    // Re-read: arming Select can change the ribbon's height and so move the
    // canvas, which leaves a mapping read before it a few pixels stale.
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let Some(label) = declared(&trace, ui_rect, LABEL) else {
        return Ok(Some(format!(
            "★ NO TEXT REGION. The dimension was clicked on its line and no `{LABEL}` region was \
             published, so the press has no text to land on. Regions declared under \
             `canvas.dimension`: {}.",
            list(&crate::checks::driving::declared_names(
                &trace,
                ui_rect,
                "canvas.dimension"
            ))
        )));
    };
    let Some(viewport) = declared(&trace, ui_rect, VIEWPORT_REGION) else {
        return Err(Error::new(format!(
            "no `{VIEWPORT_REGION}` region was declared."
        )));
    };
    let frame = session.frame()?;
    let canvas_px = frame.logical_to_capture_pixels(viewport);
    let was_at = mid(label);
    let to_at = Pt::new(was_at.x + DRAG.0, was_at.y + DRAG.1);
    let before_shot = ctx.out("dimension_text_slide_before.png");
    let before = crate::capture::window_to_png(&session, &before_shot)?;
    report.artifact(before_shot);

    // --- 4: drag the text, button down ---------------------------------------
    let from: ScreenPoint = screen(was_at);
    let to: ScreenPoint = screen(to_at);
    let during_shot = ctx.out("dimension_text_slide_during.png");
    let during = driver.drag_observed(from, to, || {
        crate::capture::window_to_png(&session, &during_shot)
    })?;
    report.artifact(during_shot);
    let trace = session.trace()?;
    if let Some(refused) = trace
        .events(PREVIEW_EVENT)
        .find(|l| l.get("baked") == Some("0"))
    {
        return Ok(Some(format!(
            "★ THE ENGINE REFUSED TO BAKE THE SLID TEXT: `{}`.",
            refused.raw
        )));
    }
    let Some(baked) = trace
        .events(PREVIEW_EVENT)
        .filter(|l| l.get("baked") == Some("1"))
        .last()
    else {
        return Ok(Some(format!(
            "★ NO LIVE PREVIEW WHILE THE TEXT WAS DRAGGED: no `{PREVIEW_EVENT} baked=1` line. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let Some(quad) = baked.get("label").and_then(label_quad) else {
        return Err(Error::new(format!(
            "the `{PREVIEW_EVENT}` line carries no readable `label=` field: `{}`",
            baked.raw
        )));
    };
    let mid_at = quad_centre(&mapping, quad)?;
    let (along, across) = (mid_at.x - was_at.x, mid_at.y - was_at.y);
    if across.abs() > ACROSS_TOLERANCE {
        return Ok(Some(format!(
            "★ THE LINE MOVED WITH THE TEXT. Mid-drag the previewed text sits {across:.1} px \
             across the line from where it was ({was_at:?} → {mid_at:?}); a slide keeps it on \
             the line. `canvas::dimlabel::slid` projects the drag onto the line."
        )));
    }
    if along < 0.5 * DRAG.0 {
        return Ok(Some(format!(
            "★ THE TEXT DID NOT FOLLOW THE DRAG. Mid-drag the previewed text moved {along:.1} px \
             along a {:.1} px drag. `{}`",
            DRAG.0, baked.raw
        )));
    }
    report.note(format!(
        "mid-drag the text slid {along:.1} px along the line and {across:.1} px across it: `{}`",
        baked.raw
    ));
    if let Some(dest) = quad_pixels(&mapping, &frame, quad, canvas_px)? {
        let (was, was_account) = ink(&before, dest);
        let (now, now_account) = ink(&during, dest);
        if now < was + INKED {
            return Ok(Some(format!(
                "★ THE PREVIEW IS NOT ON THE GLASS. The previewed text box {dest:?} went from \
                 {was_account} to {now_account} with the button down."
            )));
        }
        report.note(format!(
            "the previewed text is drawn: {dest:?} went from {was_account} to {now_account}"
        ));
    }

    // --- 5: release — the text moved, the line did not ----------------------
    session.settle(30);
    let trace = session.trace()?;
    if let Some(body) = trace.last(BODY_EVENT) {
        return Ok(Some(format!(
            "★ THE PRESS ON THE TEXT MOVED THE WHOLE DIMENSION: the release was `{}`, the body \
             drag's, not `{RELEASE_EVENT}`.",
            body.raw
        )));
    }
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
    let Some(after) = declared(&trace, ui_rect, LABEL) else {
        return Ok(Some(format!(
            "after `{}` the dimension is no longer selected: no `{LABEL}` region.",
            release.raw
        )));
    };
    let now_at = mid(after);
    if (now_at.y - was_at.y).abs() > ACROSS_TOLERANCE
        || (now_at.x - mid_at.x).abs() > ACROSS_TOLERANCE
    {
        return Ok(Some(format!(
            "★ THE RELEASE PUT THE TEXT SOMEWHERE ELSE. Before {was_at:?}, previewed at \
             {mid_at:?}, committed at {now_at:?}. `{}`",
            release.raw
        )));
    }
    report.note(format!(
        "`{}` committed the text at {now_at:?}, where the preview showed it, on the same line",
        release.raw
    ));
    Ok(None)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn a_centre_is_the_midpoint() {
        let c = mid(LRect::new(Pt::new(0.0, 10.0), Pt::new(20.0, 30.0)));
        assert!((c.x - 10.0).abs() < 1e-6 && (c.y - 20.0).abs() < 1e-6);
    }
}
