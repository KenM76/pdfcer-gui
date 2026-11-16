//! `a_dimension_text_drag_moves_the_dimension` — pressing on a selected linear
//! ce dimension's text and dragging moves the dimension as a press on its
//! line does, along and across, previewed with the button down and committed
//! where the pointer left it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_text_drag.md`.

use super::dimdrive::{run, ui_rect_event};
use super::dimension_text_select::select_by_line;
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The region the canvas publishes over a selected linear ce dimension's text.
const LABEL: &str = "canvas.dimension-label"; // ui-text-exempt: trace region name
/// `dim-preview baked=1 …` — the painter drew the engine's bake.
const PREVIEW: &str = "dim-preview"; // ui-text-exempt: a trace event name, never displayed
/// `dimension-place id=… offset=… text_along=…` — the canvas's release.
const RELEASE: &str = "dimension-place"; // ui-text-exempt: a trace event name, never displayed
/// The edit funnel's line for the applied placement.
const APPLIED: &str = "place-dimension"; // ui-text-exempt: a trace event name, never displayed
/// The drag, in window points: along the horizontal dimension, then across.
const DRAG: (f32, f32) = (50.0, 36.0);
/// Frames the drag is spread over, so the preview has frames to draw in.
const STEPS: u32 = 12;

/// The check. See the module header.
pub struct ADimensionTextDragMovesTheDimension;

impl Check for ADimensionTextDragMovesTheDimension {
    fn name(&self) -> &'static str {
        "a_dimension_text_drag_moves_the_dimension"
    }

    fn defect(&self) -> &'static str {
        "dragging a linear ce dimension's text could only slide it along the line, or showed \
         nothing while it moved. His ask, verbatim: \"When I drag the text of a dimension I \
         should be able to perform the same movement as when I select the dimension line. Also \
         needs a live preview.\""
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, "mode.review", "dim-text-drag", drive)
    }
}

/// The published text region, or an error naming the trace.
fn label(ctx: &CheckContext, session: &Session) -> Result<LRect> {
    declared_in(&session.trace()?, ui_rect_event(ctx)?, LABEL)
        .map(|(r, _)| r)
        .ok_or_else(|| {
            Error::new(format!(
                "the selected ce dimension published no `{LABEL}` region. Trace: {}.",
                session.trace_path().display()
            ))
        })
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    select_by_line(ctx, session, pointer)?;
    let was = WindowPoint::centre_of(label(ctx, session)?);
    let to = WindowPoint::centre_of(LRect {
        min: Pt::new(was.x() + DRAG.0, was.y() + DRAG.1),
        max: Pt::new(was.x() + DRAG.0, was.y() + DRAG.1),
    });
    let trace_path = session.trace_path().display().to_string();
    let mark = session.trace()?.mark();
    pointer.drag(session, was, to, STEPS)?;
    session.settle(40);
    let trace = session.trace()?;
    let baked = trace
        .events(PREVIEW)
        .filter(|l| l.lineno > mark && l.get("baked") == Some("1"))
        .count();
    if baked == 0 {
        return Ok(Some(format!(
            "NO LIVE PREVIEW WHILE THE TEXT WAS DRAGGED: no `{PREVIEW} baked=1` line during \
             the drag. Trace: {trace_path}."
        )));
    }
    let Some(release) = trace.last_after(RELEASE, mark) else {
        return Ok(Some(format!(
            "THE RELEASE DID NOT PLACE THE DIMENSION: no `{RELEASE}` line. Trace: {trace_path}."
        )));
    };
    if trace.last_after(APPLIED, mark).is_none() {
        return Ok(Some(format!(
            "the release asked for `{}` and the engine did not apply it: no `{APPLIED}` line. \
             Trace: {trace_path}.",
            release.raw
        )));
    }
    let now = WindowPoint::centre_of(label(ctx, session)?);
    let (along, across) = (now.x() - was.x(), now.y() - was.y());
    report.note(format!(
        "{baked} baked preview frames; `{}`; the text moved {along:.1} along and {across:.1} \
         across on a drag of {DRAG:?}",
        release.raw
    ));
    // The across part is what a slide along the line cannot produce.
    Ok(
        (across.abs() < 0.5 * DRAG.1.abs() || along.abs() < 0.5 * DRAG.0.abs()).then(|| {
            format!(
                "THE TEXT DID NOT FOLLOW THE DRAG BOTH WAYS. It moved {along:.1} along and \
                 {across:.1} across on a drag of {DRAG:?}; a press on the text is to move the \
                 dimension as a press on its line does (`gesture::meaning`, \
                 `DimensionPress::Label`). Trace: {trace_path}."
            )
        }),
    )
}
