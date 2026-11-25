//! `a_group_made_like_another_copies_its_set_up` — Manage ce-dimension groups
//! on `fixtures/dimension-scaled.pdf`: *New group like this one* on the
//! default group's right-click menu, and then *Add group like ‹name›* beside
//! *Add group*, each make one group, in one undo entry, with the template's
//! scale, number format, standard and style.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_like.md`.

use super::dimdrive::{press, reveal, run, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review,measure.manage_groups"; // ui-text-exempt: ribbon command ids
/// Mirrors `panels::dimension_groups`'s regions.
const ROW: &str = "dimension-groups.row."; // ui-text-exempt: a trace region name
const MENU_LIKE: &str = "dimension-groups.row-menu.like"; // ui-text-exempt: a trace region name
const BUTTON_LIKE: &str = "dimension-groups.add-like"; // ui-text-exempt: a trace region name
const ADD_HEADING: &str = "dimension-groups.heading.add"; // ui-text-exempt: a trace region name
const LIKE: &str = "dimension-group-like"; // ui-text-exempt: a trace event name
/// The default group, which carries the fixture's 5 mm/pt calibration.
const TEMPLATE: usize = 0;
/// The four properties the engine copies.
const SAME: [&str; 4] = ["same_scale", "same_format", "same_standard", "same_style"];

/// See the module documentation.
pub struct AGroupMadeLikeAnotherCopiesItsSetUp;

impl Check for AGroupMadeLikeAnotherCopiesItsSetUp {
    fn name(&self) -> &'static str {
        "a_group_made_like_another_copies_its_set_up"
    }

    fn defect(&self) -> &'static str {
        "a ce-dimension group made like another starts with the default scale or numbers, \
         takes more than one undo step, or one of its two routes is missing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, INVOKE, "dimension-like", body)
    }
}

/// The `LIKE` line after `mark`, judged: one new group, copied from the
/// template, the `depth`-th entry on the undo stack, and drawn as a row.
fn judge(
    ctx: &CheckContext,
    session: &Session,
    mark: usize,
    route: &str,
    depth: usize,
) -> Result<std::result::Result<usize, String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(LIKE, mark) else {
        return Ok(Err(format!(
            "{route} was pressed and no `{LIKE}` followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let copied = SAME.iter().all(|k| line.get(k) == Some("1"));
    if line.get_usize("template") != Some(TEMPLATE)
        || !copied
        || line.get_usize("undo_depth") != Some(depth)
    {
        return Ok(Err(format!(
            "{route} traced `{}`: template {TEMPLATE}, all four of {} = 1 and undo depth \
             {depth} were wanted.",
            line.raw,
            SAME.join(", ")
        )));
    }
    let Some(group) = line.get_usize("group") else {
        return Ok(Err(format!("{route}'s `{}` names no group.", line.raw)));
    };
    session.settle(20);
    let drawn = declared_in(
        &session.trace()?,
        ui_rect_event(ctx)?,
        &format!("{ROW}{group}"),
    );
    if drawn.is_none() {
        return Ok(Err(format!(
            "{route} made group {group} and the panel draws no row `{ROW}{group}`."
        )));
    }
    Ok(Ok(group))
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let template_row = format!("{ROW}{TEMPLATE}");
    let Some((row, _)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, &template_row) else {
        return Ok(Some(format!(
            "the default group's row `{template_row}` was never drawn. Trace: {path}."
        )));
    };

    // Route 1: the row's right-click menu.
    pointer.right_click(session, WindowPoint::centre_of(row))?;
    session.settle(10);
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, MENU_LIKE)?;
    let first = match judge(ctx, session, mark, "New group like this one", 1)? {
        Ok(group) => group,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("menu route made group {first}"));

    // Route 2: select the template row, then the button in the Add fold.
    press(ctx, session, pointer, &template_row)?;
    reveal(ctx, session, pointer, BUTTON_LIKE, ADD_HEADING)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, BUTTON_LIKE)?;
    let second = match judge(ctx, session, mark, "Add group like", 2)? {
        Ok(group) => group,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("button route made group {second}"));
    if second == first {
        return Ok(Some(format!(
            "both routes named group {first}: the second made nothing new."
        )));
    }
    Ok(None)
}
