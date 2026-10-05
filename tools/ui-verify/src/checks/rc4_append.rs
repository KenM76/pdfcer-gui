//! `rc4_edits_wait_for_the_operator_and_say_what_they_cost` — an edit to an
//! RC4-encrypted file is refused with a sentence and a button until the
//! operator allows it; allowed, the edit lands, Ctrl+S saves under the old
//! encryption, and the save says how many changed parts reuse a key stream.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/rc4_append.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

const OFFSCREEN: &str = "-4200,-4200,1200,1350";
const INVOKE: &str = "pages.rotate_right";
const FIXTURE: &str = "encrypted-rc4-128.pdf";
const METHOD: &str = "Copy it again from pdfcer; see `fixtures/encrypted-rc4-128.PROVENANCE.md`.";
const UI_RECT: &str = "ui-rect";
const RAIL_ROTATE: &str = "rail.rotate.pages.rotate_right"; // ui-text-exempt: a trace region name, never displayed
const REMEDY: &str = "status-group:decline.remedy"; // ui-text-exempt: a trace region name, never displayed
const DISCLOSED: &str = "rc4-disclosed"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "rotate-pages-refused"; // ui-text-exempt: a trace event name, never displayed
const ROTATED: &str = "rotate-pages"; // ui-text-exempt: a trace event name, never displayed
const POLICY: &str = "rc4-append"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
const REUSED: &str = "rc4-keystream-reused"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct Rc4EditsWaitForTheOperatorAndSayWhatTheyCost;

impl Check for Rc4EditsWaitForTheOperatorAndSayWhatTheyCost {
    fn name(&self) -> &'static str {
        "rc4_edits_wait_for_the_operator_and_say_what_they_cost"
    }

    fn defect(&self) -> &'static str {
        "an RC4-encrypted file refuses every edit with nothing said and no way to allow it, or \
         allowing it changes nothing, or a save under RC4 does not say it reused a key stream"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer, copy)| {
            let outcome = steps(&mut report, &session, &pointer, &copy);
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
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
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
    // A copy: the check saves in place.
    let copy = ctx.out("rc4_append.pdf");
    if let Some(dir) = copy.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::new(e.to_string()))?;
    }
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &copy)
        .map_err(|e| Error::new(format!("could not copy the fixture: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("rc4_append.trace.txt"));
    spec.pdf = Some(copy.clone());
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("rc4_append.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched as pid {} on a scratch copy",
        session.pid()
    ));
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

fn steps(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    copy: &Path,
) -> Result<Option<String>> {
    if await_new(session, DISCLOSED, 0)?.is_none() {
        return Ok(Some(format!(
            "{FIXTURE} is RC4-encrypted and opening it traced no `{DISCLOSED}`: the operator \
             is not told edits wait for him."
        )));
    }
    let Some(refused) = await_new(session, REFUSED, 0)? else {
        return Ok(Some(format!(
            "`{INVOKE}` on an RC4 file traced no `{REFUSED}`: the edit either landed without \
             being allowed or never ran."
        )));
    };
    report.note(format!("before allowing: `{}`", refused.raw));
    let rotated_before = session.trace()?.events(ROTATED).count();
    let Some((button, vp)) = declared_in(&session.trace()?, UI_RECT, REMEDY) else {
        return Ok(Some(format!(
            "the refusal drew no button beside it (`{REMEDY}`): the operator is told no and \
             not how to say yes."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(button))?;
    session.settle(20);
    let Some(policy) = await_new(session, POLICY, 0)? else {
        return Ok(Some(format!(
            "the button beside the refusal traced no `{POLICY}` line."
        )));
    };
    if policy.get("policy") != Some("preserve") {
        return Ok(Some(format!(
            "the button did not allow edits: `{}`.",
            policy.raw
        )));
    }
    // The rail's button, not `]`: the chord is not offered in Read mode.
    let Some((rotate, rail_vp)) = declared_in(&session.trace()?, UI_RECT, RAIL_ROTATE) else {
        return Ok(Some(format!(
            "no `{RAIL_ROTATE}` button was declared to press."
        )));
    };
    pointer.click_in(session, rail_vp.as_deref(), WindowPoint::centre_of(rotate))?;
    session.settle(20);
    let Some(rotated) = await_new(session, ROTATED, rotated_before)? else {
        return Ok(Some(format!(
            "with edits allowed, the rotate button traced no `{ROTATED}`: allowing changed nothing."
        )));
    };
    report.note(format!("after allowing: `{}`", rotated.raw));
    saved(report, session, pointer, vp.as_deref(), copy)
}

/// Ctrl+S, then the save line, the reuse line and the file's own bytes.
fn saved(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    vp: Option<&str>,
    copy: &Path,
) -> Result<Option<String>> {
    let before = std::fs::metadata(copy).map_or(0, |m| m.len());
    pointer.key(session, vp, "S", Some("ctrl"))?;
    session.settle(40);
    let Some(save) = await_new(session, SAVED, 0)? else {
        return Ok(Some(format!("Ctrl+S traced no `{SAVED}` line.")));
    };
    if save.get("outcome") != Some("ok") {
        return Ok(Some(format!("the save failed: `{}`.", save.raw)));
    }
    let Some(reused) = await_new(session, REUSED, 0)? else {
        return Ok(Some(format!(
            "a save under RC4 traced no `{REUSED}`: the operator is not told what it cost."
        )));
    };
    report.note(format!("saved: `{}`", reused.raw));
    let bytes = std::fs::read(copy).map_err(|e| Error::new(e.to_string()))?;
    let encrypted = bytes.windows(8).any(|w| w == b"/Encrypt");
    report.note(format!("file {before} -> {} bytes", bytes.len()));
    if bytes.len() as u64 <= before || !encrypted {
        return Ok(Some(format!(
            "the saved file is {} bytes from {before}, /Encrypt present: {encrypted}. An \
             appended revision under the old encryption owes both.",
            bytes.len()
        )));
    }
    Ok(None)
}
