//! `escape_finishes_a_markup_run` — with Markup ▸ Polygon armed, three corners
//! and Escape author the polygon: `markup-finish via=escape`, `markup-commit
//! kind=Polygon … vertices=3`, the engine's `add-markup`, and
//! `canvas-escape outcome=FinishedMarkupVertexRun`; the pen stays armed.
//!
//! `escape_finishes_a_measure_run` — with Measure ▸ Length armed, three picks
//! and Escape author the ce dimension: `measure-finish via=escape
//! kind=PathLength`, the engine's `add-dimension`, and `canvas-escape
//! outcome=FinishedMeasureRun`.
//!
//! An Escape that discards the run traces `AbandonedMarkupVertexRun` or
//! `AbandonedMeasurePick` and reaches no engine verb, so neither check can pass
//! on it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/escape_finishes_a_run.md`.

use super::dimdrive::{click_page, run};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

// ui-text-exempt: command ids, trace event names and values, never displayed.
const MARKUP_INVOKE: &str = "mode.review,markup.polygon";
const MEASURE_INVOKE: &str = "mode.review,measure.length";
const ESCAPE: &str = "canvas-escape";
const FINISH_MARKUP: &str = "markup-finish";
const FINISH_MEASURE: &str = "measure-finish";
const COMMIT_MARKUP: &str = "markup-commit";
const ENGINE_MARKUP: &str = "add-markup";
const ENGINE_DIMENSION: &str = "add-dimension";
const TOOL_EVENT: &str = "canvas-tool";

/// Three corners in page points, clear of the fixture's ce dimension.
const CORNERS: [(f64, f64); 3] = [(70.0, 50.0), (190.0, 60.0), (130.0, 110.0)];

/// See the module documentation.
pub struct EscapeFinishesAMarkupRun;
/// See the module documentation.
pub struct EscapeFinishesAMeasureRun;

impl Check for EscapeFinishesAMarkupRun {
    fn name(&self) -> &'static str {
        "escape_finishes_a_markup_run"
    }

    fn defect(&self) -> &'static str {
        "Escape during a polygon, polyline or cloud throws the corners away instead of \
         finishing the shape with the corners placed so far"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, MARKUP_INVOKE, "escape-finishes-markup", markup)
    }
}

impl Check for EscapeFinishesAMeasureRun {
    fn name(&self) -> &'static str {
        "escape_finishes_a_measure_run"
    }

    fn defect(&self) -> &'static str {
        "Escape during a Length, Perimeter, Area or Circular measure throws the picks away \
         instead of committing the ce dimension they make"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(
            self,
            ctx,
            MEASURE_INVOKE,
            "escape-finishes-measure",
            measure,
        )
    }
}

/// Click the three corners, press Escape, and return the trace mark taken
/// before the press with the press's `canvas-escape` outcome.
fn corners_then_escape(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<(usize, Option<String>)> {
    for at in CORNERS {
        click_page(ctx, session, pointer, at)?;
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Escape", None)?;
    session.settle(40);
    let outcome = session
        .trace()?
        .last_after(ESCAPE, mark)
        .and_then(|l| l.get("outcome").map(str::to_owned));
    Ok((mark, outcome))
}

/// Whether an `event` line after `mark` satisfies `ok`.
fn seen(
    session: &Session,
    event: &str,
    mark: usize,
    ok: impl Fn(&crate::trace::TraceLine) -> bool,
) -> Result<bool> {
    Ok(session
        .trace()?
        .events(event)
        .any(|l| l.lineno > mark && ok(l)))
}

fn markup(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let (mark, outcome) = corners_then_escape(ctx, session, pointer)?;
    report.note(format!(
        "Escape after three corners traced outcome={outcome:?}"
    ));
    if outcome.as_deref() != Some("FinishedMarkupVertexRun") {
        return Ok(Some(format!(
            "Escape after three polygon corners traced `{ESCAPE} outcome={outcome:?}`, not \
             `FinishedMarkupVertexRun`: rung 3a of `canvas::keys` did not finish the run \
             (`markup::vertex::escape`). Trace: {path}."
        )));
    }
    let via = seen(session, FINISH_MARKUP, mark, |l| {
        l.get("via") == Some("escape")
    })?;
    let commit = seen(session, COMMIT_MARKUP, mark, |l| {
        l.get("kind") == Some("Polygon") && l.get_usize("vertices") == Some(3)
    })?;
    let engine = seen(session, ENGINE_MARKUP, mark, |_| true)?;
    report.note(format!(
        "via=escape {via}, commit vertices=3 {commit}, engine {engine}"
    ));
    if !(via && commit && engine) {
        return Ok(Some(format!(
            "Escape claimed the run finished, and of `{FINISH_MARKUP} via=escape`, \
             `{COMMIT_MARKUP} kind=Polygon vertices=3` and the engine's `{ENGINE_MARKUP}` the \
             trace holds: {via}, {commit}, {engine}. The polygon did not reach the document \
             with its three corners. Trace: {path}."
        )));
    }
    let disarmed = seen(session, TOOL_EVENT, mark, |_| true)?;
    Ok(disarmed.then(|| {
        format!(
            "the Escape that finished the polygon also changed the armed tool (`{TOOL_EVENT} \
             armed=…`): one press, two effects. Trace: {path}."
        )
    }))
}

fn measure(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let (mark, outcome) = corners_then_escape(ctx, session, pointer)?;
    report.note(format!(
        "Escape after three picks traced outcome={outcome:?}"
    ));
    if outcome.as_deref() != Some("FinishedMeasureRun") {
        return Ok(Some(format!(
            "Escape after three Length picks traced `{ESCAPE} outcome={outcome:?}`, not \
             `FinishedMeasureRun`: rung 3a of `canvas::keys` did not commit the run \
             (`measure::escape`). Trace: {path}."
        )));
    }
    let via = seen(session, FINISH_MEASURE, mark, |l| {
        l.get("via") == Some("escape") && l.get("kind") == Some("PathLength")
    })?;
    let engine = seen(session, ENGINE_DIMENSION, mark, |_| true)?;
    report.note(format!("via=escape {via}, engine {engine}"));
    Ok((!(via && engine)).then(|| {
        format!(
            "Escape claimed the run committed, and of `{FINISH_MEASURE} via=escape \
             kind=PathLength` and the engine's `{ENGINE_DIMENSION}` the trace holds: {via}, \
             {engine}. The ce dimension did not reach the document. Trace: {path}."
        )
    }))
}
