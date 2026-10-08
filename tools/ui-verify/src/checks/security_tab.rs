//! `the_security_tab_holds_security_and_protect` — the Security tab carries
//! the file's security tools and, in Edit alone, the redaction tools; File no
//! longer carries them. And `the_snapshot_tool_is_on_the_left_rail` — the rail's
//! Navigate group arms the snapshot tool. Both driven off the desktop through
//! the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/security_tab.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, live_names};
use crate::checks::security_notes::await_line;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "four-pages.pdf";
const UI_RECT: &str = "ui-rect";
const TAB: &str = "ribbon.tab.security"; // ui-text-exempt: a trace region name, never displayed
/// A Security-group item and the group's collapsed button, either of which
/// proves the band is drawn.
const SECURITY: [&str; 2] = ["ribbon.item.file.encrypt", "ribbon.group.security.security"];
/// The same for the Protect group.
/// The redaction items; Protect's Remove metadata is drawn in every mode.
const PROTECT: [&str; 2] = ["ribbon.item.edit.redact", "ribbon.item.edit.offpage"];
const RAIL_SNAPSHOT: &str = "rail.navigate.view.tool_snapshot"; // ui-text-exempt: a trace region name, never displayed
const ARMED: &str = "snapshot-tool";

/// See the module documentation.
pub struct TheSecurityTabHoldsSecurityAndProtect;

/// See the module documentation.
pub struct TheSnapshotToolIsOnTheLeftRail;

impl Check for TheSecurityTabHoldsSecurityAndProtect {
    fn name(&self) -> &'static str {
        "the_security_tab_holds_security_and_protect"
    }

    fn defect(&self) -> &'static str {
        "the security and protect tools are not on a Security tab, are still on File, or \
         redaction shows in Read where no content may be edited"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = edit_mode(ctx, &mut report).and_then(|failure| match failure {
            Some(f) => Ok(Some(f)),
            None => read_mode(ctx, &mut report),
        });
        verdict(report, outcome)
    }
}

impl Check for TheSnapshotToolIsOnTheLeftRail {
    fn name(&self) -> &'static str {
        "the_snapshot_tool_is_on_the_left_rail"
    }

    fn defect(&self) -> &'static str {
        "the snapshot tool is reachable only from View ▸ Navigate: the left tool strip has no \
         row for it, or its row does not arm the tool"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = rail(ctx, &mut report);
        verdict(report, outcome)
    }
}

fn verdict(report: CheckReport, outcome: Result<Option<String>>) -> CheckReport {
    match outcome {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

/// Edit mode: File has lost the tools, Security has both groups.
fn edit_mode(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report, "mode.edit", "security_tab_edit")?;
    let before = session.trace()?;
    let early = declared_names(&before, UI_RECT, SECURITY[0]);
    if !early.is_empty() {
        return Ok(Some(format!(
            "`{}` was drawn before the Security tab was pressed: File still carries Encrypt.",
            SECURITY[0]
        )));
    }
    press(&session, &pointer, TAB)?;
    let trace = session.trace()?;
    let live = live_names(&trace, UI_RECT, "ribbon.");
    report.note(format!("Edit, Security tab: {}", list(&live)));
    for (band, names) in [("Security", SECURITY), ("Protect", PROTECT)] {
        if !live.iter().any(|n| names.iter().any(|w| n.starts_with(w))) {
            return Ok(Some(format!(
                "in Edit the Security tab draws no {band} band (neither `{}` nor `{}`). On \
                 screen: {}.",
                names[0],
                names[1],
                list(&live)
            )));
        }
    }
    pointer.gone(&session)?;
    Ok(None)
}

/// Read mode: the Security band, and no redaction anywhere in the run.
fn read_mode(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report, "mode.read", "security_tab_read")?;
    press(&session, &pointer, TAB)?;
    let trace = session.trace()?;
    let live = live_names(&trace, UI_RECT, "ribbon.");
    report.note(format!("Read, Security tab: {}", list(&live)));
    if !live
        .iter()
        .any(|n| SECURITY.iter().any(|w| n.starts_with(w)))
    {
        return Ok(Some(format!(
            "in Read the Security tab draws no Security band. On screen: {}.",
            list(&live)
        )));
    }
    let leaked: Vec<String> = PROTECT
        .iter()
        .flat_map(|w| declared_names(&trace, UI_RECT, w))
        .collect();
    if !leaked.is_empty() {
        return Ok(Some(format!(
            "Read drew redaction, which edits page content: {}.",
            list(&leaked)
        )));
    }
    pointer.gone(&session)?;
    Ok(None)
}

/// The rail row is drawn in Read and arms the snapshot tool.
fn rail(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report, "mode.read", "rail_snapshot")?;
    if declared_in(&session.trace()?, UI_RECT, RAIL_SNAPSHOT).is_none() {
        return Ok(Some(format!(
            "the left rail draws no `{RAIL_SNAPSHOT}`. Navigate rows: {}.",
            list(&declared_names(
                &session.trace()?,
                UI_RECT,
                "rail.navigate."
            ))
        )));
    }
    press(&session, &pointer, RAIL_SNAPSHOT)?;
    let Some(armed) = await_line(&session, ARMED)? else {
        return Ok(Some(format!(
            "the rail's snapshot row was pressed and no `{ARMED}` line followed."
        )));
    };
    report.note(format!("armed: `{}`", armed.raw));
    if armed.get("armed") != Some("true") {
        return Ok(Some(format!(
            "the rail's snapshot row traced `{}`, not an armed tool.",
            armed.raw
        )));
    }
    pointer.gone(&session)?;
    Ok(None)
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
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(crate::checks::driving::repo_fixture(
        FIXTURE,
        "It is a checked-in fixture.",
    )?);
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
    report.note(format!("{stem}: pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

/// Click `name` in whichever viewport declared it.
fn press(session: &Session, pointer: &ScriptedPointer, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region to press. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    Ok(())
}
