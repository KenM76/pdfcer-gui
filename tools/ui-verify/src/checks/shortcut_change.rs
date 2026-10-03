//! `a_changed_shortcut_takes_effect_and_persists`: Settings ▸ Keyboard
//! shortcuts gives Find the key Ctrl+K; after Save, Ctrl+K opens Find and
//! Ctrl+F no longer does, and a fresh launch reads the change back from the
//! preferences file (`OPERATOR_REQUESTS.md` O274). Driven off the desktop
//! through the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/shortcut_change.md`.

use std::path::Path;

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const OPEN_SETTINGS: &str = "file.settings";
const HEADING: &str = "settings.heading.shortcuts";
const FILTER: &str = "settings.keys.filter";
const CHANGE: &str = "settings.keys.change.edit.find";
const REASSIGN: &str = "settings.keys.reassign";
const SAVE: &str = "dialog:settings.save";
const COMMAND: &str = "edit.find";
/// Typed into the filter; the Find row must survive it.
const NEEDLE: &str = "Find"; // ui-text-exempt: typed input, not prose
const SET: &str = "shortcut-set"; // ui-text-exempt: a trace event name, never displayed
const CLASH: &str = "shortcut-clash"; // ui-text-exempt: a trace event name, never displayed
const DISPATCHED: &str = "chord-command"; // ui-text-exempt: a trace event name, never displayed
const PREFS_SAVED: &str = "prefs-saved"; // ui-text-exempt: a trace event name, never displayed
const PREFS_FILE: &str = "preferences.txt";
/// The line Save must write.
const WRITTEN: &str = "shortcut.edit.find = Ctrl+K"; // ui-text-exempt: a preferences line

/// See the module documentation.
pub struct AChangedShortcutTakesEffectAndPersists;

impl Check for AChangedShortcutTakesEffectAndPersists {
    fn name(&self) -> &'static str {
        "a_changed_shortcut_takes_effect_and_persists"
    }

    fn defect(&self) -> &'static str {
        "a keyboard shortcut changed in Settings ▸ Keyboard shortcuts does not reach the keys, \
         leaves the old key working, or is gone after a restart"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    exe: &Path,
    invoke: Option<&str>,
    stem: &str,
) -> Result<(Session, ScriptedPointer)> {
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    if let Some(invoke) = invoke {
        spec.env
            .push(("PDFCER_DIAG_INVOKE".to_owned(), invoke.to_owned()));
    }
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(40);
    Ok((session, pointer))
}

/// Click a declared region in whichever viewport declared it; `None` when it
/// is not declared, else the viewport.
fn click(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<Option<Option<String>>> {
    let Some((rect, viewport)) = declared_in(&session.trace()?, ui_rect, name) else {
        return Ok(None);
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    Ok(Some(viewport))
}

/// How many times the main window dispatched `edit.find` from a chord.
fn finds(session: &Session) -> Result<usize> {
    Ok(session
        .trace()?
        .events(DISPATCHED)
        .filter(|l| l.get("id") == Some(COMMAND))
        .count())
}

/// Press `key` with Ctrl in the main window; whether it dispatched Find.
fn press(session: &Session, pointer: &ScriptedPointer, key: &str) -> Result<bool> {
    let before = finds(session)?;
    pointer.key(session, None, key, Some("ctrl"))?;
    session.settle(20);
    Ok(finds(session)? > before)
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, "")
        .map_err(|e| Error::new(format!("could not clear the preferences: {e}")))?;

    let (session, pointer) = launch(ctx, report, &exe, Some(OPEN_SETTINGS), "shortcuts")?;
    if click(&session, &pointer, ui_rect, HEADING)?.is_none() {
        return Ok(Some(format!(
            "the Settings window lists no Keyboard shortcuts page: no `{HEADING}`. Pages: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "settings.heading."
            ))
        )));
    }
    let Some(vp) = click(&session, &pointer, ui_rect, FILTER)? else {
        return Ok(Some(format!(
            "the Keyboard shortcuts page declares no `{FILTER}` box."
        )));
    };
    pointer.type_text(&session, vp.as_deref(), NEEDLE)?;
    session.settle(20);
    if click(&session, &pointer, ui_rect, CHANGE)?.is_none() {
        return Ok(Some(format!(
            "filtering for {NEEDLE:?} left no `{CHANGE}` button. Rows: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "settings.keys.change."
            ))
        )));
    }
    pointer.key(&session, vp.as_deref(), "K", Some("ctrl"))?;
    session.settle(20);
    let trace = session.trace()?;
    if trace.events(CLASH).next().is_some() {
        report.note("Ctrl+K was held by another command; pressing Reassign");
        if click(&session, &pointer, ui_rect, REASSIGN)?.is_none() {
            return Ok(Some(format!(
                "a clash was traced and no `{REASSIGN}` is offered."
            )));
        }
    }
    let set = session
        .trace()?
        .events(SET)
        .any(|l| l.get("command") == Some(COMMAND) && l.get("chords") == Some("Ctrl+K"));
    if !set {
        return Ok(Some(format!(
            "Ctrl+K was pressed while Find's Change waited and no `{SET} command={COMMAND} \
             chords=Ctrl+K` followed. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let saves = session.trace()?.events(PREFS_SAVED).count();
    if click(&session, &pointer, ui_rect, SAVE)?.is_none() {
        return Ok(Some(format!("the Settings window declares no `{SAVE}`.")));
    }
    session.settle(20);
    if session.trace()?.events(PREFS_SAVED).count() == saves {
        return Ok(Some(format!(
            "Save was pressed and no `{PREFS_SAVED}` line followed."
        )));
    }
    let new_key = press(&session, &pointer, "K")?;
    let old_key = press(&session, &pointer, "F")?;
    pointer.gone(&session)?;
    drop(session);
    if !new_key || old_key {
        return Ok(Some(format!(
            "after Save, Ctrl+K dispatched Find: {new_key}; Ctrl+F dispatched Find: {old_key}. \
             The live keys are not the ones the page set."
        )));
    }
    report.note("after Save, Ctrl+K opens Find and Ctrl+F does not");

    let text = std::fs::read_to_string(userdata.join(PREFS_FILE))
        .map_err(|e| Error::new(format!("cannot read the preferences file: {e}")))?;
    if !text.lines().any(|l| l.trim() == WRITTEN) {
        return Ok(Some(format!(
            "Save wrote no `{WRITTEN}` line to {}.",
            userdata.join(PREFS_FILE).display()
        )));
    }
    let (session, pointer) = launch(ctx, report, &exe, None, "shortcuts-relaunch")?;
    let new_key = press(&session, &pointer, "K")?;
    let old_key = press(&session, &pointer, "F")?;
    pointer.gone(&session)?;
    if !new_key || old_key {
        return Ok(Some(format!(
            "after a restart, Ctrl+K dispatched Find: {new_key}; Ctrl+F dispatched Find: \
             {old_key}. The saved key was not read back."
        )));
    }
    report.note("a fresh launch reads the line back: Ctrl+K opens Find, Ctrl+F does not");
    Ok(None)
}
