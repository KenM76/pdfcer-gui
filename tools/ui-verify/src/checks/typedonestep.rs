//! A typed move and turn applied together are one Undo, driven on
//! `overlapping-boxes.pdf`: the red box is selected, X and Rotate by are typed
//! and applied, and one Ctrl+Z puts the box back where it was.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/typedonestep.md`.

use super::dimdrive::{click_on, press, run_on};
use super::nudgecontent::{BOXES, INVOKE, RED_ONLY, last_geometry};
use super::stroke_style::{into_view, type_into};
use super::typedturn::fail;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const X_FIELD: &str = "properties.geometry.x"; // ui-text-exempt: a trace region name, never displayed
const TURN_FIELD: &str = "properties.geometry.turn"; // ui-text-exempt: a trace region name, never displayed
const APPLY: &str = "properties.geometry.apply"; // ui-text-exempt: a trace region name, never displayed
const FOLDED: &str = "one-undo-step"; // ui-text-exempt: a trace event name, never displayed
const UNDO: &str = "undo"; // ui-text-exempt: a trace event name, never displayed
/// The red box as Properties shows it before any edit.
const RED_BOX: &str = "x=60.00 y=60.00 w=160.00 h=120.00";

/// See the module documentation.
pub struct ATypedEditIsOneUndo;

impl Check for ATypedEditIsOneUndo {
    fn name(&self) -> &'static str {
        "a_typed_move_and_turn_are_one_undo"
    }

    fn defect(&self) -> &'static str {
        "Applying a typed position and turn together leaves two Undo steps, so one Ctrl+Z \
         leaves the object half-edited"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &BOXES, INVOKE, "typedonestep", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &BOXES, 0, RED_ONLY)?;
    session.settle(10);
    let before = last_geometry(session, mark)?;
    report.note(format!("red selected: `{before}`"));
    if !before.contains(RED_BOX) {
        return Ok(Some(fail(
            session,
            &format!("the click must select red at `{RED_BOX}`"),
            &before,
        )));
    }
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, X_FIELD, "100")?;
    type_into(ctx, session, pointer, TURN_FIELD, "90")?;
    into_view(ctx, session, pointer, APPLY)?;
    press(ctx, session, pointer, APPLY)?;
    session.settle(40);
    let folded = session
        .trace()?
        .last_after(FOLDED, mark)
        .map_or_else(|| "nothing".to_owned(), |l| l.raw.clone());
    report.note(format!("applied: `{folded}`"));
    if !folded.contains(" pushed=2 folded=true ") {
        return Ok(Some(fail(
            session,
            &format!("Apply must push the move and the turn and fold them (`{FOLDED}`)"),
            &folded,
        )));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    let trace = session.trace()?;
    let undo = trace.last_after(UNDO, mark);
    report.note(format!(
        "Ctrl+Z: `{}`",
        undo.map_or("nothing", |l| l.raw.as_str())
    ));
    let after = last_geometry(session, undo.map_or(mark, |l| l.lineno))?;
    report.note(format!("red afterwards: `{after}`"));
    Ok((!after.contains(RED_BOX)).then(|| {
        fail(
            session,
            &format!("one Ctrl+Z must put red back at `{RED_BOX}`"),
            &after,
        )
    }))
}
