//! The dimension tools' Tool options, driven: the linear tool's direction, the
//! group a new ce dimension joins, and a new group starting from another
//! group's scale.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_tool_options.md`.

use super::dimdrive::{click_page, press, reveal, run};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const PROPERTIES_TAB: &str = "dock.tab.file.properties";
const ADD_HEADING: &str = "dimension-groups.heading.add";
/// Mirrors `panels::properties::tooldim`'s regions.
const DIM_COMBO: &str = "properties.tool.dim.group.combo";
const DIM_NEW: &str = "properties.tool.dim.group.option.new";
const DIM_NAME: &str = "properties.tool.dim.new_name";
const DIM_CREATE: &str = "properties.tool.dim.create";
const HORIZONTAL: &str = "properties.tool.dim.direction.horizontal";
/// Mirrors `panels::dimension_groups`'s regions.
const NEW_NAME: &str = "dimension-groups.new_name";
const NEW_UNIT: &str = "dimension-groups.new_unit.combo";
const NEW_UNIT_FEET: &str = "dimension-groups.new_unit.option.ft";
const NEW_SCALE: &str = "dimension-groups.new_scale.combo";
const SCALE_FROM_DEFAULT: &str = "dimension-groups.new_scale.option.0";
const ADD: &str = "dimension-groups.add";
const DRAW_INTO: &str = "dimension-groups.draw_into.";
const ADDED: &str = "dimension-added"; // ui-text-exempt: a trace event name, never displayed
const SHOWN: &str = "dimension-member-shown"; // ui-text-exempt: a trace event name, never displayed
const AUTHORING: &str = "dimension-authoring-group"; // ui-text-exempt: a trace event name, never displayed
const COPIED: &str = "dimension-group-scale-copied"; // ui-text-exempt: a trace event name, never displayed
/// The three picks, in page points. Δx 200, Δy 150: aligned 250 pt.
const A: (f64, f64) = (60.0, 60.0);
const B: (f64, f64) = (260.0, 210.0);
const PLACE: (f64, f64) = (160.0, 30.0);
/// The fixture's default group reads 5 mm per point.
const MM_PER_PT: f64 = 5.0;
const MM_PER_FOOT: f64 = 304.8;

/// The linear tool set to Horizontal measures the run, not the diagonal.
pub struct AHorizontalDimensionMeasuresOnlyTheRun;
/// A group made in Tool options takes the next ce dimension and the scale.
pub struct ADimensionJoinsTheGroupMadeInToolOptions;
/// A group added with "Same as Default" in feet reads the default's lengths.
pub struct ANewGroupCopiesAGroupsScaleInItsOwnUnit;

impl Check for AHorizontalDimensionMeasuresOnlyTheRun {
    fn name(&self) -> &'static str {
        "a_horizontal_dimension_measures_only_the_run"
    }

    fn defect(&self) -> &'static str {
        "the linear tool has no Horizontal or Vertical choice, or ignores it: a ce dimension \
         between two diagonal points always measures the diagonal"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(
            self,
            ctx,
            "mode.review,measure.linear",
            "dim-horizontal",
            horizontal,
        )
    }
}

impl Check for ADimensionJoinsTheGroupMadeInToolOptions {
    fn name(&self) -> &'static str {
        "a_dimension_joins_the_group_made_in_tool_options"
    }

    fn defect(&self) -> &'static str {
        "adding a ce dimension offers no way to pick or make its group: it always joins the \
         default group, and a new group starts with no scale"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(
            self,
            ctx,
            "mode.review,measure.linear",
            "dim-tool-group",
            tool_group,
        )
    }
}

impl Check for ANewGroupCopiesAGroupsScaleInItsOwnUnit {
    fn name(&self) -> &'static str {
        "a_new_group_copies_a_groups_scale_in_its_own_unit"
    }

    fn defect(&self) -> &'static str {
        "a new ce dimension group cannot start from an existing group's scale, or copies the \
         number and so reads millimetres as feet"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(
            self,
            ctx,
            "mode.review,measure.linear,measure.manage_groups",
            "dim-copy-scale",
            copy_scale,
        )
    }
}

/// Draw A → B, placed at PLACE; the new ce dimension's `dimension-added` line
/// and the text its `dimension-member-shown` line reports.
fn draw(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<(String, String, String)> {
    let mark = session.trace()?.mark();
    for at in [A, B, PLACE] {
        click_page(ctx, session, pointer, at)?;
    }
    session.settle(30);
    let trace = session.trace()?;
    let added = trace.last_after(ADDED, mark).ok_or_else(|| {
        Error::new(format!(
            "three clicks on the page committed no ce dimension (no `{ADDED}`). Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    let dim = added.get("dim").unwrap_or_default().to_owned();
    let text = trace
        .events(SHOWN)
        .filter(|l| l.lineno > mark && l.get("dim") == Some(dim.as_str()))
        .last()
        .and_then(|l| l.get("text"))
        .unwrap_or_default()
        .to_owned();
    Ok((
        added.raw.clone(),
        added.get("group").unwrap_or_default().to_owned(),
        text,
    ))
}

/// The leading number of a shown measurement, `"1000 mm"` → 1000.
fn leading_number(text: &str) -> Option<f64> {
    text.split_whitespace().next()?.parse().ok()
}

fn reads(text: &str, unit: &str, want: f64, tolerance: f64) -> bool {
    text.ends_with(unit) && leading_number(text).is_some_and(|v| (v - want).abs() <= tolerance)
}

fn horizontal(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    reveal(ctx, session, pointer, HORIZONTAL, PROPERTIES_TAB)?;
    press(ctx, session, pointer, HORIZONTAL)?;
    let (added, _, text) = draw(ctx, session, pointer)?;
    report.note(format!("`{added}` reads `{text}`"));
    let run = (B.0 - A.0) * MM_PER_PT;
    let diagonal = (B.0 - A.0).hypot(B.1 - A.1) * MM_PER_PT;
    if !added.contains("constraint=horizontal") {
        return Ok(Some(format!(
            "Horizontal was chosen and the ce dimension committed as `{added}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok((!reads(&text, " mm", run, 1.0)).then(|| {
        format!(
            "a Horizontal ce dimension between points {run:.0} mm apart across and {diagonal:.0} mm \
             apart diagonally reads `{text}`; it must read the run, {run:.0} mm. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

fn tool_group(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    reveal(ctx, session, pointer, DIM_COMBO, PROPERTIES_TAB)?;
    press(ctx, session, pointer, DIM_COMBO)?;
    press(ctx, session, pointer, DIM_NEW)?;
    press(ctx, session, pointer, DIM_NAME)?;
    pointer.type_text(session, None, "Detail")?;
    session.settle(15);
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, DIM_CREATE)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(made) = trace
        .events(AUTHORING)
        .filter(|l| l.lineno > mark && l.get("via") == Some("created"))
        .last()
    else {
        return Ok(Some(format!(
            "Create in Tool options made no group the authoring group (no `{AUTHORING} via=created`). \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let id = made.get("id").unwrap_or_default().to_owned();
    report.note(format!("created group {id}"));
    let (added, group, text) = draw(ctx, session, pointer)?;
    report.note(format!("`{added}` reads `{text}`"));
    if group != id || id == "0" {
        return Ok(Some(format!(
            "the ce dimension joined group {group}, not the group {id} just made and picked. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    let aligned = (B.0 - A.0).hypot(B.1 - A.1) * MM_PER_PT;
    Ok((!reads(&text, " mm", aligned, 1.0)).then(|| {
        format!(
            "the new group's ce dimension reads `{text}`; it started from the default group's \
             scale, so 250 pt reads {aligned:.0} mm. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

fn copy_scale(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    reveal(ctx, session, pointer, NEW_NAME, ADD_HEADING)?;
    press(ctx, session, pointer, NEW_NAME)?;
    pointer.type_text(session, None, "Feet")?;
    session.settle(15);
    press(ctx, session, pointer, NEW_UNIT)?;
    press(ctx, session, pointer, NEW_UNIT_FEET)?;
    press(ctx, session, pointer, NEW_SCALE)?;
    press(ctx, session, pointer, SCALE_FROM_DEFAULT)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ADD)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(copied) = trace.last_after(COPIED, mark) else {
        return Ok(Some(format!(
            "Add with \"Same as Default\" copied no scale (no `{COPIED}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", copied.raw));
    let id = copied.get("group").unwrap_or_default().to_owned();
    if copied.get("folded") != Some("1") {
        return Ok(Some(format!(
            "the add and its scale are two undo steps: `{}`. Trace: {}.",
            copied.raw,
            session.trace_path().display()
        )));
    }
    press(ctx, session, pointer, &format!("{DRAW_INTO}{id}"))?;
    let (added, group, text) = draw(ctx, session, pointer)?;
    report.note(format!("`{added}` reads `{text}`"));
    let feet = (B.0 - A.0).hypot(B.1 - A.1) * MM_PER_PT / MM_PER_FOOT;
    if group != id {
        return Ok(Some(format!(
            "the ce dimension joined group {group}, not the drawn-into group {id}. Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok((!reads(&text, " ft", feet, 0.01)).then(|| {
        format!(
            "the feet group copied from Default reads `{text}`; 250 pt at 5 mm per point is \
             {feet:.2} ft. Trace: {}.",
            session.trace_path().display()
        )
    }))
}
