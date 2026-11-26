//! `a_groups_circle_line_type_reaches_its_members` — the dimension-groups
//! panel sets the default group's circle line type to Phantom, and every
//! member that follows the group must resolve its circle to Phantom from the
//! group tier while its own lines stay solid. See
//! `docs/modules/ui-verify/checks/group_circle_dash.md`.

use super::dimdrive::{press, press_in_pane, reveal, run};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review,measure.manage_groups";
const ROW: &str = "dimension-groups.row.0"; // ui-text-exempt: trace region name
const HEADING: &str = "dimension-groups.heading.appearance"; // ui-text-exempt: trace region name
const PANE: &str = "dock.right.frame"; // ui-text-exempt: trace region name
/// Mirrors `panels::dimension_groups::parts`' circle dash row.
const SET: &str = "dimension-groups.circle.dash.set"; // ui-text-exempt: trace region name
const COMBO: &str = "dimension-groups.circle.dash.combo"; // ui-text-exempt: trace region name
const PHANTOM: &str = "dimension-groups.circle.dash.option.phantom"; // ui-text-exempt: trace region name
const STYLE: &str = "dimension-member-style"; // ui-text-exempt: a trace event name

pub struct AGroupsCircleLineTypeReachesItsMembers;

impl Check for AGroupsCircleLineTypeReachesItsMembers {
    fn name(&self) -> &'static str {
        "a_groups_circle_line_type_reaches_its_members"
    }

    fn defect(&self) -> &'static str {
        "a ce dimension group's circle line type cannot be set from the dimension-groups \
         panel, or is set and does not reach the members, or changes their own lines too"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, INVOKE, "group-circle-dash", steps)
    }
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    press(ctx, session, pointer, ROW)?;
    reveal(ctx, session, pointer, SET, HEADING)?;
    press_in_pane(ctx, session, pointer, SET, PANE)?;
    press(ctx, session, pointer, COMBO)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, PHANTOM)?;
    let trace = session.trace()?;
    let mut newest = std::collections::BTreeMap::new();
    for line in trace.events(STYLE).filter(|e| e.lineno > mark) {
        newest.insert(line.get("dim").unwrap_or_default().to_owned(), line.clone());
    }
    let path = session.trace_path().display();
    if newest.is_empty() {
        return Ok(Some(format!(
            "Phantom was chosen for the group's circle and no `{STYLE}` line followed. \
             Trace: {path}."
        )));
    }
    let want = [
        ("circle_dash", "group"),
        ("circle_line_type", "phantom"),
        ("circle_solid", "false"),
        ("solid", "true"),
    ];
    for line in newest.values() {
        report.note(format!("member: `{}`", line.raw));
        if want.iter().any(|(k, v)| line.get(k) != Some(*v)) {
            return Ok(Some(format!(
                "a member traced `{}`; its circle Phantom from the group and its own lines \
                 solid were wanted. Trace: {path}.",
                line.raw
            )));
        }
    }
    Ok(None)
}
