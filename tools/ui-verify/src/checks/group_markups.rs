//! Group and Ungroup on markups, driven: two markups are joined by Ctrl+G, a
//! click on either then selects both, and Ctrl+Shift+G releases them so a
//! click selects one again.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/group_markups.md`.

use super::annot_multiselect::{CIRCLE, FIXTURE, SQUARE};
use super::dimdrive::{click_on, run_on};
use super::multiselect::{after, shift_click, step};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const GROUPED: &str = "markup-group-applied"; // ui-text-exempt: a trace event name
const UNGROUPED: &str = "markup-ungroup-applied"; // ui-text-exempt: a trace event name
const TOGETHER: &str = "annot-group-select"; // ui-text-exempt: a trace event name
const ALONE: &str = "annot-select"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct GroupedMarkupsSelectTogether;

impl Check for GroupedMarkupsSelectTogether {
    fn name(&self) -> &'static str {
        "grouped_markups_select_together"
    }

    fn defect(&self) -> &'static str {
        "two selected markups cannot be grouped, or a click on one of a group selects only \
         that one, or Ungroup leaves them selecting together"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.edit", "group-markups", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
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
    if grouped.get("members") != Some("1") {
        return Ok(Some(format!(
            "Ctrl+G on two markups traced `{}`: one member under one primary was wanted.",
            grouped.raw
        )));
    }
    // A plain click on the member, then on the primary: each selects both.
    for (name, at) in [("the square", SQUARE.1), ("the circle", CIRCLE.1)] {
        let mark = session.trace()?.mark();
        click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
        session.settle(10);
        let together = step!(after(
            session,
            TOGETHER,
            mark,
            &format!("{name}, one of a group, was clicked")
        ));
        report.note(format!("`{}`", together.raw));
        if together.get("mates") != Some("1") || together.get("selected") != Some("2") {
            return Ok(Some(format!(
                "a click on {name} traced `{}`: both markups of the group selected was wanted.",
                together.raw
            )));
        }
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "G", Some("ctrl+shift"))?;
    session.settle(40);
    let released = step!(after(session, UNGROUPED, mark, "Ctrl+Shift+G was pressed"));
    report.note(format!("`{}`", released.raw));
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, SQUARE.1)?;
    session.settle(10);
    let alone = step!(after(
        session,
        ALONE,
        mark,
        "the square was clicked after Ungroup"
    ));
    let still = session.trace()?.last_after(TOGETHER, mark).is_some();
    report.note(format!("ungrouped: `{}` together={still}", alone.raw));
    if still {
        return Ok(Some(format!(
            "after Ungroup a click on the square still selected its old group. Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok(None)
}
