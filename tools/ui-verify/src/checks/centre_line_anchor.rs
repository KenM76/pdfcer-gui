//! `a_dimension_follows_the_centre_line_it_was_placed_on` — a linear ce
//! dimension picked from a centre line is anchored to it, and moving the
//! centre line moves the dimension and says so on the status line.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/centre_line_anchor.md`.

use super::centre_line::{ENGINE_EVENT, INVOKE, LOWER, TWO_LINES, UPPER};
use super::dimdrive::{Fixture, click_on, press_on_tab, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

// ui-text-exempt: trace region and event names, never displayed.
const TAB: &str = "ribbon.tab.measure";
const LINEAR: &str = "ribbon.item.measure.linear";
const ADDED: &str = "dimension-added";
const PICK_EVENT: &str = "measure-pick";
const PROMOTED: &str = "outcome=Promoted";
const SELECTED: &str = "annot-select";
const GRIP: &str = "canvas.dimension-vertex.1";
const APPLIED: &str = "move-dimension-vertex";
/// The engine's disclosure for a dependent that followed, as the funnel
/// traces it.
const FOLLOWED: &str = "moved with the geometry it is anchored to";
/// The centre line's midpoint: halfway along the fixture's lines, y midway.
const MIDPOINT: (f64, f64) = (200.0, 140.0);
/// The lower line's left end, a stated node.
const LOWER_END: (f64, f64) = (100.0, 100.0);
/// Where the linear ce dimension is placed, below and left of both picks.
const PLACE: (f64, f64) = (130.0, 60.0);
/// On the centre line, right of the dimension's pick.
const ON_CENTRE: (f64, f64) = (260.0, 140.0);
const DRAG_UP: f32 = 40.0;
const STEPS: u32 = 10;

/// See the module documentation.
pub struct ADimensionFollowsTheCentreLine;

impl Check for ADimensionFollowsTheCentreLine {
    fn name(&self) -> &'static str {
        "a_dimension_follows_the_centre_line_it_was_placed_on"
    }

    fn defect(&self) -> &'static str {
        "a linear ce dimension picked from a centre line is not bound to it: the commit \
         carries no anchor, or moving the centre line leaves the dimension behind and the \
         status line says nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &TWO_LINES, INVOKE, "centre-line-anchor", drive)
    }
}

/// Click `at` on page 1 of `fixture`, and once more if the first click only
/// promoted a derived snap.
pub(super) fn pick(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    fixture: &Fixture,
    at: (f64, f64),
) -> Result<()> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, fixture, 0, at)?;
    let promoted = session
        .trace()?
        .last_after(PICK_EVENT, mark)
        .is_some_and(|l| l.raw.contains(PROMOTED));
    if promoted {
        click_on(ctx, session, pointer, fixture, 0, at)?;
    }
    Ok(())
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
            "two clicks on the fixture's lines committed no centre line. Trace: {path}."
        )));
    }
    press_on_tab(ctx, session, pointer, LINEAR, TAB)?;
    let mark = session.trace()?.mark();
    pick(ctx, session, pointer, &TWO_LINES, MIDPOINT)?;
    pick(ctx, session, pointer, &TWO_LINES, LOWER_END)?;
    pick(ctx, session, pointer, &TWO_LINES, PLACE)?;
    let trace = session.trace()?;
    let Some(added) = trace.last_after(ADDED, mark) else {
        return Ok(Some(format!(
            "three linear picks from the centre line committed no ce dimension (`{ADDED}`). \
             Trace: {path}."
        )));
    };
    report.note(format!("commit: `{}`", added.raw));
    let anchored = added
        .get("anchors")
        .is_some_and(|a| a.split(',').any(|t| t.starts_with("0:0:line-")));
    if !anchored {
        return Ok(Some(format!(
            "the linear ce dimension picked from the centre line committed as `{}`: its \
             first point (index 0) should carry an anchor to the centre line (dim 0). \
             Trace: {path}.",
            added.raw
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
    let grip = declared_in(&session.trace()?, ui_rect_event(ctx)?, GRIP).map(|(r, _)| r);
    let (true, Some(grip)) = (picked, grip) else {
        return Ok(Some(format!(
            "a click on the centre line did not select it with its end grips drawn. \
             Trace: {path}."
        )));
    };
    let from = WindowPoint::centre_of(grip);
    let to = WindowPoint::centre_of(LRect {
        min: Pt::new(from.x(), from.y() - DRAG_UP),
        max: Pt::new(from.x(), from.y() - DRAG_UP),
    });
    let mark = session.trace()?.mark();
    pointer.drag(session, from, to, STEPS)?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(applied) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "dragging the centre line's end applied nothing (`{APPLIED}`). Trace: {path}."
        )));
    };
    report.note(format!("move: `{}`", applied.raw));
    Ok((!applied.raw.contains(FOLLOWED)).then(|| {
        format!(
            "the centre line moved and the edit disclosed no ce dimension following it \
             (`{FOLLOWED}`): the anchored dimension stayed behind, or moved silently. \
             Trace: {path}."
        )
    }))
}
