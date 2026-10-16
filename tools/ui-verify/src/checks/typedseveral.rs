//! Typed X over several selected objects, driven on `overlapping-boxes.pdf`:
//! red is clicked and green Shift-clicked, X is typed for the box around
//! both, and both move by the same distance as one Undo.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/typedseveral.md`.

use super::alignmixed::shift_click;
use super::dimdrive::{click_on, press, run_on};
use super::nudgecontent::{BOXES, INVOKE, RED_ONLY};
use super::stroke_style::{into_view, type_into};
use super::typedturn::fail;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const X_FIELD: &str = "properties.geometry.x"; // ui-text-exempt: a trace region name, never displayed
const APPLY: &str = "properties.geometry.apply"; // ui-text-exempt: a trace region name, never displayed
const SEVERAL: &str = "geometry-several"; // ui-text-exempt: a trace event name, never displayed
const UNDO: &str = "undo"; // ui-text-exempt: a trace event name, never displayed
/// Inside the green box only, page space.
const GREEN_ONLY: (f64, f64) = (40.0, 250.0);
/// Red (60..220, 60..180) and green (20..80, 230..270) together.
const BOTH: &str = "bx=20.00 by=60.00 bw=200.00 bh=210.00";
/// The same box 100 points right. Only green moving reads `bx=60.00`; only
/// red moving keeps `bx=20.00`; both moving, but not together, changes `bw`.
const BOTH_MOVED: &str = "bx=120.00 by=60.00 bw=200.00 bh=210.00";

/// See the module documentation.
pub struct TypedXMovesSeveral;

impl Check for TypedXMovesSeveral {
    fn name(&self) -> &'static str {
        "a_typed_x_moves_several_objects_together"
    }

    fn defect(&self) -> &'static str {
        "Properties offers no position for several selected objects, or a typed one moves \
         only some of them, or takes more than one Undo"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &BOXES, INVOKE, "typedseveral", drive)
    }
}

/// The last `geometry-several` line after `mark`, or "nothing".
fn last_several(session: &Session, mark: usize) -> Result<String> {
    Ok(session
        .trace()?
        .last_after(SEVERAL, mark)
        .map_or_else(|| "nothing".to_owned(), |l| l.raw.clone()))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &BOXES, 0, RED_ONLY)?;
    let mark = session.trace()?.mark();
    let selected = shift_click(ctx, session, pointer, BOXES.page, GREEN_ONLY)?;
    report.note(format!("after Shift+click on green: `{selected}`"));
    session.settle(10);
    let before = last_several(session, mark)?;
    report.note(format!("Properties reads: `{before}`"));
    if !before.contains(" n=2 ") || !before.contains(BOTH) {
        return Ok(Some(fail(
            session,
            &format!("Properties must read the two boxes as one, `n=2 {BOTH}`"),
            &before,
        )));
    }
    type_into(ctx, session, pointer, X_FIELD, "120")?;
    into_view(ctx, session, pointer, APPLY)?;
    press(ctx, session, pointer, APPLY)?;
    session.settle(40);
    let mark = session.trace()?.mark();
    session.settle(10);
    let moved = last_several(session, mark)?;
    report.note(format!("after Apply: `{moved}`"));
    if !moved.contains(BOTH_MOVED) {
        return Ok(Some(fail(
            session,
            &format!("Apply must move both boxes 100 points right, `{BOTH_MOVED}`"),
            &moved,
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
    let after = last_several(session, undo.map_or(mark, |l| l.lineno))?;
    report.note(format!("after one Ctrl+Z: `{after}`"));
    Ok((!after.contains(BOTH)).then(|| {
        fail(
            session,
            &format!("one Ctrl+Z must put both back, `{BOTH}`"),
            &after,
        )
    }))
}
