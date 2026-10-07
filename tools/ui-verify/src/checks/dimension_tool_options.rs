//! The dimension tools' Tool options, driven: the linear tool's direction, the
//! group a new ce dimension joins, and a new group starting from another
//! group's scale.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_tool_options.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,1000";
const FIXTURE: &str = "dimension-scaled.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/dimension-scaled.PROVENANCE.py`.";
const PAGE: PageGeometry = PageGeometry {
    width_pt: 400.0,
    height_pt: 300.0,
};
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

type Body =
    fn(&CheckContext, &mut CheckReport, &Session, &ScriptedPointer) -> Result<Option<String>>;

fn run(check: &dyn Check, ctx: &CheckContext, invoke: &str, stem: &str, body: Body) -> CheckReport {
    let mut report = CheckReport::new(check.name(), check.defect());
    let driven = launch(ctx, &mut report, invoke, stem).and_then(|(session, pointer)| {
        let outcome = body(ctx, &mut report, &session, &pointer);
        let parked = pointer.gone(&session);
        match outcome? {
            Some(failure) => Ok(Some(failure)),
            None => parked.map(|_| None),
        }
    });
    match driven {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

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

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    invoke: &str,
    stem: &str,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(&format!("{stem}.pdf"));
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", invoke),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

fn ui_rect_event(ctx: &CheckContext) -> Result<&'static str> {
    ctx.profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))
}

/// Click `region` where it was last declared; an error naming it when it is not.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<()> {
    let (rect, vp) =
        declared_in(&session.trace()?, ui_rect_event(ctx)?, region).ok_or_else(|| {
            Error::new(format!(
                "`{region}` was never declared. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

/// Press `opener` (a dock tab or a folded heading) when `region` has not been drawn yet.
fn reveal(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    opener: &str,
) -> Result<()> {
    if declared_in(&session.trace()?, ui_rect_event(ctx)?, region).is_none() {
        press(ctx, session, pointer, opener)?;
    }
    Ok(())
}

/// Click `at` (page points) on page 1, through the current canvas mapping.
fn click_page(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<()> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
    let point = mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?;
    pointer.hover(session, point)?;
    session.settle(10);
    pointer.click(session, point)?;
    session.settle(20);
    Ok(())
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
