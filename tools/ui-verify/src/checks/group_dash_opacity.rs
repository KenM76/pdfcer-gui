//! `a_groups_line_style_and_opacity_reach_its_members` — the dimension-groups
//! panel sets the default group's line style to dashed and its opacity to
//! 50 %, and every member that follows the group must resolve to that dash
//! and that opacity, taken from the group tier.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/group_dash_opacity.md`.

use super::dimdrive::{press, press_in_pane, reveal, run};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const INVOKE: &str = "mode.review,measure.manage_groups";
/// The default group's row. Mirrors `panels::dimension_groups::REGION_ROW_PREFIX`.
const ROW: &str = "dimension-groups.row.0";
/// The folded appearance section. Mirrors `REGION_HEADING_PREFIX` + `"appearance"`.
const HEADING: &str = "dimension-groups.heading.appearance";
/// The right dock's frame, which clips the panel as it scrolls; the panel's own
/// rect is its content and moves with it.
const PANE: &str = "dock.right.frame";
/// Mirrors `panels::dimension_groups::style::REGION_DASH_SET`.
const DASH_SET: &str = "dimension-groups.dash.set";
/// Mirrors `style::REGION_DASH` + `.combo`, as `linestyle::chooser_in` publishes it.
const DASH_COMBO: &str = "dimension-groups.dash.combo";
/// Mirrors `style::REGION_DASH` + `.option.` + `LineStyle::Dashed.token()`.
const DASHED: &str = "dimension-groups.dash.option.dashed";
/// Mirrors `style::REGION_OPACITY_SET`.
const OPACITY_SET: &str = "dimension-groups.opacity.set";
/// Mirrors `style::REGION_OPACITY`.
const OPACITY: &str = "dimension-groups.opacity";
const STYLE: &str = "dimension-member-style"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AGroupsLineStyleAndOpacityReachItsMembers;

impl Check for AGroupsLineStyleAndOpacityReachItsMembers {
    fn name(&self) -> &'static str {
        "a_groups_line_style_and_opacity_reach_its_members"
    }

    fn defect(&self) -> &'static str {
        "a ce dimension group's line style and opacity cannot be set from the dimension-groups \
         panel, or are set and never reach the members that follow the group"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, INVOKE, "group-dash-opacity", steps)
    }
}

/// The newest `dimension-member-style` line per member written after `mark`,
/// or the failure sentence when there is none.
fn members_after(
    session: &Session,
    mark: usize,
    gesture: &str,
) -> Result<std::result::Result<Vec<TraceLine>, String>> {
    let trace = session.trace()?;
    let mut newest = std::collections::BTreeMap::new();
    for line in trace.events(STYLE).filter(|e| e.lineno > mark) {
        newest.insert(line.get("dim").unwrap_or_default().to_owned(), line.clone());
    }
    Ok(if newest.is_empty() {
        Err(format!(
            "{gesture} and no `{STYLE}` line followed: the group's style was never set. \
             Trace: {}.",
            session.trace_path().display()
        ))
    } else {
        Ok(newest.into_values().collect())
    })
}

/// The first line in `lines` missing one `key=value` of `want`, as a failure.
fn all_carry(lines: &[TraceLine], want: &[(&str, &str)]) -> Option<String> {
    let bad = lines
        .iter()
        .find(|line| want.iter().any(|(k, v)| line.get(k) != Some(*v)))?;
    let wanted: Vec<String> = want.iter().map(|(k, v)| format!("{k}={v}")).collect();
    Some(format!(
        "a member traced `{}`; {} were wanted.",
        bad.raw,
        wanted.join(", ")
    ))
}

/// The raw lines, for the report.
fn raws(lines: &[TraceLine]) -> String {
    lines
        .iter()
        .map(|l| l.raw.as_str())
        .collect::<Vec<_>>()
        .join("` `")
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    press(ctx, session, pointer, ROW)?;
    reveal(ctx, session, pointer, DASH_SET, HEADING)?;
    press_in_pane(ctx, session, pointer, DASH_SET, PANE)?;
    press(ctx, session, pointer, DASH_COMBO)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, DASHED)?;
    let dashed = match members_after(session, mark, "dashed was chosen")? {
        Ok(lines) => lines,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("after dashed: `{}`", raws(&dashed)));
    if let Some(why) = all_carry(&dashed, &[("dash", "group"), ("solid", "false")]) {
        return Ok(Some(why));
    }

    press_in_pane(ctx, session, pointer, OPACITY_SET, PANE)?;
    press_in_pane(ctx, session, pointer, OPACITY, PANE)?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "A", Some("ctrl"))?;
    pointer.type_text(session, None, "50")?;
    pointer.key(session, None, "Enter", None)?;
    session.settle(40);
    let faded = match members_after(session, mark, "50 % was typed")? {
        Ok(lines) => lines,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("after 50 %: `{}`", raws(&faded)));
    Ok(all_carry(
        &faded,
        &[
            ("dash", "group"),
            ("solid", "false"),
            ("opacity", "0.5"),
            ("opacity_from", "group"),
        ],
    ))
}
