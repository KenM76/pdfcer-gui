//! `a_click_inside_an_empty_box_reaches_the_markup_beneath` — on
//! `markup-stack.pdf` a large unfilled square sits on top of a small unfilled
//! circle. A click on the circle's ring selects the circle, a click on the
//! square's edge selects the square, and a click on empty paper inside the
//! square selects nothing.
//!
//! `a_markup_steps_forward_past_the_markup_it_overlaps` — the circle has a
//! closed popup between it and the square in `/Annots`. Bring forward once on
//! the circle and the click where the circle's ring crosses the square's edge,
//! which picked the square, picks the circle.
//!
//! Oracle: the `subtype` of the `annot-select` each click traces. Before
//! markups were picked by their ink, the square's `/Rect` claimed all three.

use super::dimdrive::{Fixture, click_on, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit";
const FIXTURE: Fixture = Fixture {
    file: "markup-stack.pdf",
    method: "Rebuild it with `python fixtures/markup-stack.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const SELECT: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
/// The right of the circle's ring, page space: inside the square's `/Rect`.
const ON_THE_CIRCLE: (f64, f64) = (79.0, 200.0);
/// The square's left edge, away from the circle.
const ON_THE_SQUARE: (f64, f64) = (61.0, 300.0);
/// Where the circle's ring crosses the square's edge.
const ON_BOTH: (f64, f64) = (60.5, 219.0);
const APPLIED: &str = "arrange-annotation-applied"; // ui-text-exempt: a trace event name, never displayed
/// Empty paper inside the square, away from both rings.
const INSIDE_THE_SQUARE: (f64, f64) = (120.0, 300.0);

/// See the module documentation.
pub struct AClickInsideAnEmptyBoxReachesTheMarkupBeneath;
/// See the module documentation.
pub struct AMarkupStepsForwardPastTheMarkupItOverlaps;

impl Check for AClickInsideAnEmptyBoxReachesTheMarkupBeneath {
    fn name(&self) -> &'static str {
        "a_click_inside_an_empty_box_reaches_the_markup_beneath"
    }

    fn defect(&self) -> &'static str {
        "an unfilled markup on top claims every click inside its bounding box, so the markup \
         beneath it cannot be selected"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "markup-click-through", steps)
    }
}

impl Check for AMarkupStepsForwardPastTheMarkupItOverlaps {
    fn name(&self) -> &'static str {
        "a_markup_steps_forward_past_the_markup_it_overlaps"
    }

    fn defect(&self) -> &'static str {
        "bring forward steps a markup past a hidden popup, so nothing on the page changes"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "markup-step-forward", forward)
    }
}

fn forward(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let trace = session.trace_path().display().to_string();
    let before = picked(ctx, session, pointer, ON_BOTH)?;
    report.note(format!("where they cross, before: {before:?}"));
    if before.as_deref() != Some("Square") {
        return Ok(Some(format!(
            "where the ring crosses the edge picked {before:?} before any step, not the \
             square on top: the point is not contested, so the step proves nothing. Trace: \
             {trace}."
        )));
    }
    if picked(ctx, session, pointer, ON_THE_CIRCLE)?.as_deref() != Some("Circle") {
        return Ok(Some(format!(
            "the circle could not be selected. Trace: {trace}."
        )));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "CloseBracket", Some("ctrl"))?;
    session.settle(30);
    if session.trace()?.last_after(APPLIED, mark).is_none() {
        return Ok(Some(format!(
            "Ctrl+] on the selected circle traced no `{APPLIED}`. Trace: {trace}."
        )));
    }
    let after = picked(ctx, session, pointer, ON_BOTH)?;
    report.note(format!(
        "where they cross, after one step forward: {after:?}"
    ));
    Ok((after.as_deref() != Some("Circle")).then(|| {
        format!(
            "one Bring forward on the circle left {after:?} on top where it crosses the \
             square: the step passed the popup, not the square. Trace: {trace}."
        )
    }))
}

/// The subtype a click at `at` selects, or `None` when it selects nothing.
fn picked(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
    Ok(session
        .trace()?
        .last_after(SELECT, mark)
        .and_then(|l| l.get("subtype").map(str::to_owned)))
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let edge = picked(ctx, session, pointer, ON_THE_SQUARE)?;
    report.note(format!("the square's edge picked {edge:?}"));
    let ring = picked(ctx, session, pointer, ON_THE_CIRCLE)?;
    report.note(format!("the circle's ring picked {ring:?}"));
    let empty = picked(ctx, session, pointer, INSIDE_THE_SQUARE)?;
    report.note(format!("empty paper inside the square picked {empty:?}"));
    let trace = session.trace_path().display();
    if edge.as_deref() != Some("Square") {
        return Ok(Some(format!(
            "a click on the square's edge picked {edge:?}, not the square: the control \
             click failed, so the other two prove nothing. Trace: {trace}."
        )));
    }
    if ring.as_deref() != Some("Circle") {
        return Ok(Some(format!(
            "a click on the circle's ring, inside the square's box, picked {ring:?}: the \
             square on top still claims its whole box. Trace: {trace}."
        )));
    }
    Ok(empty.map(|s| {
        format!(
            "a click on empty paper inside the square picked {s}: an unfilled box still \
             claims its interior. Trace: {trace}."
        )
    }))
}
