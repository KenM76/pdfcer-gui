//! `a_centre_lines_end_can_be_dragged` — a selected centre line shows a grip
//! at each of its two ends, and dragging one moves that end and commits.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/centre_line_ends.md`.

use super::centre_line::{ENGINE_EVENT, INVOKE, LOWER, TWO_LINES, UPPER};
use super::dimdrive::{click_on, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

// ui-text-exempt: trace region and event names, never displayed.
const GRIP: &str = "canvas.dimension-vertex";
const SELECTED: &str = "annot-select";
const SHELL_EVENT: &str = "dimension-vertex";
const APPLIED: &str = "move-dimension-vertex";
/// On the committed centre line, midway between the two fixture lines.
const ON_CENTRE: (f64, f64) = (200.0, 140.0);
/// The drag, window points: straight up, off the line.
const DRAG_UP: f32 = 40.0;
const STEPS: u32 = 10;

/// See the module documentation.
pub struct ACentreLinesEndCanBeDragged;

impl Check for ACentreLinesEndCanBeDragged {
    fn name(&self) -> &'static str {
        "a_centre_lines_end_can_be_dragged"
    }

    fn defect(&self) -> &'static str {
        "a centre line drawn with Measure ▸ Centre line shows no grips at its ends when \
         selected, or dragging one moves nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &TWO_LINES, INVOKE, "centre-line-ends", drive)
    }
}

/// The published grip `index`, if it was drawn.
fn grip(ctx: &CheckContext, session: &Session, index: usize) -> Result<Option<LRect>> {
    Ok(declared_in(
        &session.trace()?,
        ui_rect_event(ctx)?,
        &format!("{GRIP}.{index}"),
    )
    .map(|(r, _)| r))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    click_on(ctx, session, pointer, &TWO_LINES, 0, LOWER)?;
    click_on(ctx, session, pointer, &TWO_LINES, 0, UPPER)?;
    if session.trace()?.last(ENGINE_EVENT).is_none() {
        return Ok(Some(format!(
            "two clicks on the fixture's lines committed no centre line (`{ENGINE_EVENT}`), so \
             there is nothing to drag. Trace: {path}."
        )));
    }
    pointer.key(session, None, "Escape", None)?;
    session.settle(15);
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &TWO_LINES, 0, ON_CENTRE)?;
    let picked = session
        .trace()?
        .last_after(SELECTED, mark)
        .is_some_and(|l| l.get("kind") == Some("CeDimension"));
    if !picked {
        return Ok(Some(format!(
            "a click on the centre line selected no ce dimension. Trace: {path}."
        )));
    }
    let (Some(a), Some(b)) = (grip(ctx, session, 0)?, grip(ctx, session, 1)?) else {
        return Ok(Some(format!(
            "the selected centre line published no `{GRIP}.0` and `{GRIP}.1`: its ends have \
             no grips. Trace: {path}."
        )));
    };
    if grip(ctx, session, 2)?.is_some() {
        return Ok(Some(format!(
            "the selected centre line published a third grip `{GRIP}.2`; it has two ends. \
             Trace: {path}."
        )));
    }
    report.note(format!("grips at {a:?} and {b:?}"));
    let from = WindowPoint::centre_of(b);
    let to = WindowPoint::centre_of(LRect {
        min: Pt::new(from.x(), from.y() - DRAG_UP),
        max: Pt::new(from.x(), from.y() - DRAG_UP),
    });
    let mark = session.trace()?.mark();
    pointer.drag(session, from, to, STEPS)?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(shell) = trace.last_after(SHELL_EVENT, mark) else {
        return Ok(Some(format!(
            "dragging end 1's grip traced no `{SHELL_EVENT}`: the press did not start an end \
             drag. Trace: {path}."
        )));
    };
    report.note(format!("release: `{}`", shell.raw));
    let moved_up = shell
        .get("dy")
        .and_then(|v| v.parse::<f64>().ok())
        .is_some_and(|dy| dy > 1.0);
    if shell.get("index") != Some("1") || !moved_up {
        return Ok(Some(format!(
            "the release traced `{}`: `index=1` and a positive `dy` (page y is up) were \
             wanted. Trace: {path}.",
            shell.raw
        )));
    }
    Ok(trace.last_after(APPLIED, mark).is_none().then(|| {
        format!(
            "the end drag was raised and the engine applied nothing: no `{APPLIED}` line. \
             Trace: {path}."
        )
    }))
}
