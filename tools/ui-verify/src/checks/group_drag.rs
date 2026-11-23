//! `a_group_drags_as_one`: two markups grouped by Ctrl+G, the selection
//! cleared, then one press-and-drag on the square, with nothing selected
//! beforehand. Both markups move by the drag in one undo step, judged in a
//! saved copy.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/group_drag.md`.

use super::annot_multiselect::{CIRCLE, FIXTURE, SQUARE};
use super::dimdrive::click_on;
use super::multiselect::{Saves, after, run_saving, save, shift_click, step, tops, window_at};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const GROUPED: &str = "markup-group-applied"; // ui-text-exempt: a trace event name
const APPLIED: &str = "move-mixed-applied"; // ui-text-exempt: a trace event name
/// How far down the drag goes, in page points.
const DROP: f64 = 30.0;

/// See the module documentation.
pub struct AGroupDragsAsOne;

impl Check for AGroupDragsAsOne {
    fn name(&self) -> &'static str {
        "a_group_drags_as_one"
    }

    fn defect(&self) -> &'static str {
        "a press-and-drag on one markup of a group, with nothing selected first, moves only \
         that markup, or nothing, instead of the whole group"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_saving(self, ctx, &FIXTURE, "mode.edit", "group-drag", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &Saves,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &FIXTURE, 0, SQUARE.1)?;
    shift_click(ctx, session, pointer, &FIXTURE, CIRCLE.1)?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "G", Some("ctrl"))?;
    session.settle(40);
    let grouped = step!(after(
        session,
        GROUPED,
        mark,
        "Ctrl+G was pressed on two markups"
    ));
    report.note(format!("`{}`", grouped.raw));
    pointer.key(session, None, "Escape", None)?;
    session.settle(20);

    let from = SQUARE.1;
    let to = (from.0, from.1 - DROP);
    let (wf, wt) = (
        window_at(ctx, session, &FIXTURE, from)?,
        window_at(ctx, session, &FIXTURE, to)?,
    );
    let mark = session.trace()?.mark();
    pointer.drag(session, wf, wt, 8)?;
    session.settle(40);
    let trace = session.trace()?;
    let moved = trace.last_after(APPLIED, mark);
    report.note(format!(
        "after the drag: `{}`",
        moved
            .as_ref()
            .map_or("no move-mixed-applied", |l| l.raw.as_str())
    ));
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let both = [(SQUARE.0, SQUARE.2 - DROP), (CIRCLE.0, CIRCLE.2 - DROP)];
    if let Some(why) = tops(saves.moved.as_path(), &both)? {
        return Ok(Some(format!(
            "a press-and-drag on the square of a two-markup group, nothing selected first: \
             {why} Both were wanted to move by {DROP} pt."
        )));
    }
    Ok(None)
}
