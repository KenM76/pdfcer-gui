//! `an_encrypted_file_is_edited_under_the_password_it_opened_with` — a file
//! whose password grants editing is edited and saved still encrypted; a file
//! whose password does not is refused with a sentence and a button that
//! reopens it for its owner password, after which the edit lands.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/encrypted_edit.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

const OFFSCREEN: &str = "-4800,-4200,2800,1350";
/// Opens with `userpw`, which grants every permission (`/P -4`).
const GRANTED: &str = "encrypted-aes-128.pdf";
const GRANTED_METHOD: &str =
    "Copy it again from pdfcer; see `fixtures/encrypted-aes-128.PROVENANCE.md`.";
/// Opens with no password and grants only Print; its owner password is `ownerpw`.
const PRINT_ONLY: &str = "D:/Dev/pdfcer/fixtures/synthetic/encryption/enc-emptyuser-print-only.pdf";
const UI_RECT: &str = "ui-rect";
const PROMPT: &str = "dialog:password"; // ui-text-exempt: a trace region name, never displayed
const FIELD: &str = "password.field"; // ui-text-exempt: a trace region name, never displayed
const OPEN: &str = "password.open"; // ui-text-exempt: a trace region name, never displayed
const RAIL_ROTATE: &str = "rail.rotate.pages.rotate_right"; // ui-text-exempt: a trace region name, never displayed
const REMEDY: &str = "status-group:decline.remedy"; // ui-text-exempt: a trace region name, never displayed
const ROTATED: &str = "rotate-pages"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "rotate-pages-refused"; // ui-text-exempt: a trace event name, never displayed
const CAUSE: &str = "edit-encrypted-refused"; // ui-text-exempt: a trace event name, never displayed
const REOPEN: &str = "unlock-reopen"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
const SECURITY_TAB: &str = "ribbon.tab.security"; // ui-text-exempt: a trace region name, never displayed
const UNLOCK: &str = "ribbon.item.file.unlock"; // ui-text-exempt: a trace region name, never displayed
const SIGN: &str = "ribbon.item.file.sign"; // ui-text-exempt: a trace region name, never displayed
const SIGN_OPENED: &str = "sign-opened"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AnEncryptedFileIsEditedUnderThePasswordItOpenedWith;

impl Check for AnEncryptedFileIsEditedUnderThePasswordItOpenedWith {
    fn name(&self) -> &'static str {
        "an_encrypted_file_is_edited_under_the_password_it_opened_with"
    }

    fn defect(&self) -> &'static str {
        "an encrypted file is refused every edit whatever its password grants, or saves \
         decrypted, or a refusal by its password says nothing of the owner password and offers \
         no way to reopen with it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = phase(ctx, &mut report, "granted", granted).and_then(|failure| {
            failure.map_or_else(
                || phase(ctx, &mut report, "print_only", print_only),
                |f| Ok(Some(f)),
            )
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

type Steps = fn(&mut CheckReport, &Session, &ScriptedPointer, &Path) -> Result<Option<String>>;

/// Launch on a scratch copy of the phase's fixture, run `steps`, park.
fn phase(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    steps: Steps,
) -> Result<Option<String>> {
    let source = if stem == "granted" {
        repo_fixture(GRANTED, GRANTED_METHOD)?
    } else {
        let path = PathBuf::from(PRINT_ONLY);
        if !path.is_file() {
            return Err(Error::new(format!(
                "{PRINT_ONLY} is missing; it is pdfcer-core's own synthetic corpus, read here."
            )));
        }
        path
    };
    let (session, pointer, copy) = launch(ctx, report, &source, stem)?;
    let outcome = steps(report, &session, &pointer, &copy);
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(format!("{stem}: {failure}"))),
        None => parked.map(|_| None),
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    source: &Path,
    stem: &str,
) -> Result<(Session, ScriptedPointer, PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let copy = ctx.out(&format!("encrypted_edit.{stem}.pdf"));
    if let Some(dir) = copy.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::new(e.to_string()))?;
    }
    std::fs::copy(source, &copy)
        .map_err(|e| Error::new(format!("could not copy the fixture: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("encrypted_edit.{stem}.trace.txt")));
    spec.pdf = Some(copy.clone());
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
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("encrypted_edit.{stem}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("{stem}: pid {} on a scratch copy", session.pid()));
    session.settle(60);
    Ok((session, pointer, copy))
}

/// The `name` line after the first `after` of them, waiting up to 30 settles.
fn await_new(session: &Session, name: &str, after: usize) -> Result<Option<TraceLine>> {
    for _ in 0..30 {
        if let Some(line) = session.trace()?.events(name).nth(after) {
            return Ok(Some(line.clone()));
        }
        session.settle(10);
    }
    Ok(None)
}

/// Click a declared region, waiting up to 30 settles for it.
fn press(session: &Session, pointer: &ScriptedPointer, region: &str) -> Result<bool> {
    for _ in 0..30 {
        if let Some((rect, vp)) = declared_in(&session.trace()?, UI_RECT, region) {
            pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
            session.settle(20);
            return Ok(true);
        }
        session.settle(10);
    }
    Ok(false)
}

/// Answer the password prompt with `password`.
fn answer(session: &Session, pointer: &ScriptedPointer, password: &str) -> Result<Option<String>> {
    if !press(session, pointer, FIELD)? {
        return Ok(Some(format!(
            "no `{PROMPT}` prompt with a `{FIELD}` was declared."
        )));
    }
    let vp = declared_in(&session.trace()?, UI_RECT, FIELD).and_then(|(_, vp)| vp);
    pointer.type_text(session, vp.as_deref(), password)?;
    session.settle(10);
    if !press(session, pointer, OPEN)? {
        return Ok(Some(format!("the prompt declared no `{OPEN}` button.")));
    }
    session.settle(40);
    Ok(None)
}

/// `userpw` grants editing: the rotation lands, and the save keeps `/Encrypt`.
fn granted(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    copy: &Path,
) -> Result<Option<String>> {
    if let Some(why) = answer(session, pointer, "userpw")? {
        return Ok(Some(why));
    }
    if !press(session, pointer, RAIL_ROTATE)? {
        return Ok(Some(format!(
            "no `{RAIL_ROTATE}` button was declared to press."
        )));
    }
    let Some(rotated) = await_new(session, ROTATED, 0)? else {
        let refused = session.trace()?.events(CAUSE).last().map(|l| l.raw.clone());
        return Ok(Some(format!(
            "a rotation of a file whose password grants editing traced no `{ROTATED}`; \
             refusal: {refused:?}."
        )));
    };
    report.note(format!("granted: `{}`", rotated.raw));
    let before = std::fs::metadata(copy).map_or(0, |m| m.len());
    let vp = declared_in(&session.trace()?, UI_RECT, RAIL_ROTATE).and_then(|(_, vp)| vp);
    pointer.key(session, vp.as_deref(), "S", Some("ctrl"))?;
    session.settle(40);
    let Some(save) = await_new(session, SAVED, 0)? else {
        return Ok(Some(format!("Ctrl+S traced no `{SAVED}` line.")));
    };
    if save.get("outcome") != Some("ok") {
        return Ok(Some(format!("the save failed: `{}`.", save.raw)));
    }
    let bytes = std::fs::read(copy).map_err(|e| Error::new(e.to_string()))?;
    let encrypted = bytes.windows(8).any(|w| w == b"/Encrypt");
    report.note(format!("granted: file {before} -> {} bytes", bytes.len()));
    if bytes.len() as u64 <= before || !encrypted {
        return Ok(Some(format!(
            "the saved file is {} bytes from {before}, /Encrypt present: {encrypted}. An \
             appended revision under the file's own encryption owes both.",
            bytes.len()
        )));
    }
    Ok(None)
}

/// No password grants only Print: refused with the password cause and its
/// button; the button reopens for `ownerpw`, and the rotation then lands.
fn print_only(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    _copy: &Path,
) -> Result<Option<String>> {
    if let Some(why) = file_tab(report, session, pointer)? {
        return Ok(Some(why));
    }
    if !press(session, pointer, RAIL_ROTATE)? {
        return Ok(Some(format!(
            "no `{RAIL_ROTATE}` button was declared to press."
        )));
    }
    if await_new(session, REFUSED, 0)?.is_none() {
        return Ok(Some(format!(
            "a rotation of a print-only file traced no `{REFUSED}`."
        )));
    }
    let Some(cause) = await_new(session, CAUSE, 0)? else {
        return Ok(Some(format!(
            "the refusal traced no `{CAUSE}`: the funnel did not recognise the encryption \
             refusal by its variant."
        )));
    };
    report.note(format!("print_only: `{}`", cause.raw));
    if cause.get("cause") != Some("password") {
        return Ok(Some(format!(
            "the refusal named the wrong cause: `{}`; the opening password is the cause.",
            cause.raw
        )));
    }
    if !press(session, pointer, REMEDY)? {
        return Ok(Some(format!(
            "the refusal drew no button beside it (`{REMEDY}`): the operator is told no and \
             not how to reopen."
        )));
    }
    let Some(reopen) = await_new(session, REOPEN, 0)? else {
        return Ok(Some(format!(
            "the button beside the refusal traced no `{REOPEN}`."
        )));
    };
    report.note(format!("print_only: `{}`", reopen.raw));
    if let Some(why) = answer(session, pointer, "ownerpw")? {
        return Ok(Some(why));
    }
    let before = session.trace()?.events(ROTATED).count();
    if !press(session, pointer, RAIL_ROTATE)? {
        return Ok(Some(format!(
            "after reopening, no `{RAIL_ROTATE}` button was declared to press."
        )));
    }
    let Some(rotated) = await_new(session, ROTATED, before)? else {
        return Ok(Some(format!(
            "reopened with the owner password, the rotation traced no new `{ROTATED}`."
        )));
    };
    report.note(format!("print_only after reopening: `{}`", rotated.raw));
    Ok(None)
}

/// The Security tab offers Unlock, and Sign's window names the password as the
/// cause of its refusal; the window is then closed.
fn file_tab(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if !press(session, pointer, SECURITY_TAB)? {
        return Ok(Some(format!("no `{SECURITY_TAB}` was declared to press.")));
    }
    if declared_in(&session.trace()?, UI_RECT, UNLOCK).is_none() {
        return Ok(Some(format!(
            "the Security tab drew no `{UNLOCK}` on a document its password withholds edits from."
        )));
    }
    if !press(session, pointer, SIGN)? {
        return Ok(Some(format!("the Security tab drew no `{SIGN}`.")));
    }
    let Some(opened) = await_new(session, SIGN_OPENED, 0)? else {
        return Ok(Some(format!("Sign traced no `{SIGN_OPENED}`.")));
    };
    report.note(format!("print_only: `{}`", opened.raw));
    if opened.get("refusal") != Some("encrypted-password") {
        return Ok(Some(format!(
            "Sign's refusal was `{}`; the opening password is the cause, so it must be \
             `encrypted-password`.",
            opened.raw
        )));
    }
    let vp = declared_in(&session.trace()?, UI_RECT, SECURITY_TAB).and_then(|(_, vp)| vp);
    pointer.key(session, vp.as_deref(), "Escape", None)?;
    session.settle(20);
    Ok(None)
}
