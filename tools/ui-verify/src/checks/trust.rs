//! `the_trust_switches_and_web_sites_are_kept` — Security ▸ Trust's two
//! switches flip and are written to the preferences at once, and Web sites…
//! forgets a remembered site on OK. Driven with the scripted pointer in a
//! window placed off the desktop; the trace and the preferences file are the
//! oracles.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/trust.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
use crate::checks::scripteddrive::{click, userdata};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "four-pages.pdf";
const METHOD: &str = "It is checked in.";
const UI_RECT: &str = "ui-rect";
const TAB: &str = "ribbon.tab.security"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.security.trust.collapsed"; // ui-text-exempt: a trace region name, never displayed
const ASK_FILES: &str = "file.trust_ask_files";
const PLAY_3D: &str = "file.trust_play_3d";
const WEB_SITES: &str = "file.trust_web_sites";
const FORGET_FIRST: &str = "trust_sites.forget.0";
const OK: &str = "dialog.buttons.accept";
const SWITCH: &str = "trust-switch"; // ui-text-exempt: a trace event name, never displayed
const SET: &str = "trust-sites-set"; // ui-text-exempt: a trace event name, never displayed
/// Both switches on, one remembered site: every step below changes all three.
const SEEDED: &str =
    "web_links = ask\nweb_site = allow a.example\nopen_other_files = ask\nplay_3d = true\n";

/// See the module documentation.
pub struct TheTrustSwitchesAndWebSitesAreKept;

impl Check for TheTrustSwitchesAndWebSitesAreKept {
    fn name(&self) -> &'static str {
        "the_trust_switches_and_web_sites_are_kept"
    }

    fn defect(&self) -> &'static str {
        "a Security ▸ Trust switch does not flip, or is not written to the preferences, or Web \
         sites… does not forget a site on OK"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn run(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let dir = userdata(ctx)?;
    crate::sandbox::write_prefs(&dir, SEEDED)
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;
    let (session, pointer) = launch(ctx, report)?;
    let outcome = drive(report, &session, &pointer);
    let parked = pointer.gone(&session);
    drop(session);
    let outcome = outcome?;
    parked?;
    if outcome.is_some() {
        return Ok(outcome);
    }
    let prefs = std::fs::read_to_string(dir.join(crate::sandbox::PREFS_FILE)).unwrap_or_default();
    Ok(judge_prefs(&prefs))
}

/// The first preferences line that does not say what the drive set.
fn judge_prefs(prefs: &str) -> Option<String> {
    let has = |line: &str| prefs.lines().any(|l| l.trim() == line);
    for want in ["open_other_files = allow", "play_3d = false"] {
        if !has(want) {
            return Some(format!(
                "the preferences do not say `{want}` after the switch was turned off:\n{prefs}"
            ));
        }
    }
    if prefs
        .lines()
        .any(|l| l.trim_start().starts_with("web_site"))
    {
        return Some(format!(
            "Web sites… forgot `a.example` and OK kept it in the preferences:\n{prefs}"
        ));
    }
    None
}

fn drive(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    for id in [ASK_FILES, PLAY_3D] {
        let mark = session.trace()?.mark();
        press_item(session, pointer, id)?;
        let trace = session.trace()?;
        let Some(line) = trace.last_after(SWITCH, mark) else {
            return Ok(Some(format!(
                "pressing `{id}` traced no `{SWITCH}` line. Trace: {}.",
                session.trace_path().display()
            )));
        };
        report.note(format!("switch: `{}`", line.raw));
        if line.get("id") != Some(id) || line.get("on") != Some("false") {
            return Ok(Some(format!(
                "expected `{SWITCH} id={id} on=false`, the trace says `{}`.",
                line.raw
            )));
        }
    }
    let mark = session.trace()?.mark();
    press_item(session, pointer, WEB_SITES)?;
    session.settle(20);
    click(session, pointer, FORGET_FIRST)?;
    click(session, pointer, OK)?;
    let trace = session.trace()?;
    let Some(line) = trace.last_after(SET, mark) else {
        return Ok(Some(format!(
            "OK in Web sites… traced no `{SET}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("sites: `{}`", line.raw));
    if line.get("sites") != Some("0") || line.get("mode") != Some("ask") {
        return Ok(Some(format!(
            "forgetting the one remembered site should leave `mode=ask sites=0`, the trace says `{}`.",
            line.raw
        )));
    }
    Ok(None)
}

/// Press a Security ▸ Trust item, opening the collapsed group first when the
/// window is too narrow to show it.
fn press_item(session: &Session, pointer: &ScriptedPointer, id: &str) -> Result<()> {
    click(session, pointer, TAB)?;
    let item = format!("ribbon.item.{id}");
    let trace = session.trace()?;
    if declared(&trace, UI_RECT, &item).is_none() && declared(&trace, UI_RECT, COLLAPSED).is_some()
    {
        click(session, pointer, COLLAPSED)?;
    }
    click(session, pointer, &item)
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out("trust.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("trust.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("trust.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    Ok((session, pointer))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the drive should leave passes; each thing it changes, left
    /// unchanged, fails.
    #[test]
    fn the_preferences_judge_names_each_unchanged_line() {
        let done = "web_links = ask\nopen_other_files = allow\nplay_3d = false\n";
        assert_eq!(judge_prefs(done), None);
        assert!(judge_prefs(SEEDED).is_some_and(|f| f.contains("open_other_files = allow")));
        let kept = "web_site = allow a.example\nopen_other_files = allow\nplay_3d = false\n";
        assert!(judge_prefs(kept).is_some_and(|f| f.contains("forgot")));
    }
}
