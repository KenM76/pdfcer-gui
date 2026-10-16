//! A typed turn, driven on `overlapping-boxes.pdf`: the red box is selected,
//! 90 is typed into Properties' Rotate by and applied, and the box turns a
//! quarter anticlockwise about its own centre, which Properties re-reads as
//! the swapped box.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/typedturn.md`.

use super::dimdrive::{click_on, press, run_on};
use super::links::drew_after;
use super::nudgecontent::{BOXES, INVOKE, RED_ONLY, last_geometry};
use super::stroke_style::{into_view, type_into};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const TURN_FIELD: &str = "properties.geometry.turn"; // ui-text-exempt: a trace region name, never displayed
const APPLY: &str = "properties.geometry.apply"; // ui-text-exempt: a trace region name, never displayed
const TURNED: &str = "transform-objects"; // ui-text-exempt: a trace event name, never displayed
/// The red box, 160 by 120 about (140,120), as Properties shows it.
const RED_BOX: &str = "x=60.00 y=60.00 w=160.00 h=120.00";
/// The same box a quarter turn later: 120 by 160 about the same centre.
const RED_TURNED: &str = "x=80.00 y=40.00 w=120.00 h=160.00";

/// See the module documentation.
pub struct ATypedTurnRotatesAnObjectAboutItsCentre;

impl Check for ATypedTurnRotatesAnObjectAboutItsCentre {
    fn name(&self) -> &'static str {
        "a_typed_turn_rotates_an_object_about_its_centre"
    }

    fn defect(&self) -> &'static str {
        "Properties offers no Rotate by for a page object, or a typed turn goes the wrong way, \
         about the wrong point, or nowhere"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &BOXES, INVOKE, "typedturn", drive)
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
    if !drew_after(session, TURN_FIELD, mark)? {
        return Ok(Some(fail(
            session,
            "Properties must offer Rotate by for a page object",
            &format!("no `{TURN_FIELD}` region"),
        )));
    }
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, TURN_FIELD, "90")?;
    into_view(ctx, session, pointer, APPLY)?;
    press(ctx, session, pointer, APPLY)?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(turned) = trace
        .last_after(TURNED, mark)
        .map(|l| (l.lineno, l.raw.clone()))
    else {
        return Ok(Some(fail(
            session,
            &format!("Apply must reach the engine as `{TURNED}`"),
            "nothing",
        )));
    };
    report.note(format!("applied: `{}`", turned.1));
    // After the commit, so the draft that was typed into cannot answer.
    let after = last_geometry(session, turned.0)?;
    report.note(format!("red afterwards: `{after}`"));
    Ok(
        (!after.contains(RED_TURNED) || !after.contains("turn=0.00")).then(|| {
            fail(
                session,
                &format!("Properties must re-read `{RED_TURNED}` with Rotate by back at 0"),
                &after,
            )
        }),
    )
}

pub(super) fn fail(session: &Session, want: &str, got: &str) -> String {
    format!(
        "{want}; it read `{got}`. Trace: {}.",
        session.trace_path().display()
    )
}
