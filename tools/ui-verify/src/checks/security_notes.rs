//! `security_notes_name_the_cover_and_the_actions` — a §7.6.7 wrapper is
//! named on the status row the moment it opens, and Document properties ▸
//! Security notes lists every action the file would run elsewhere, the one
//! hidden behind another included.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/security_notes.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

const OFFSCREEN: &str = "-4200,-4200,1200,1350";
const INVOKE: &str = "file.document_properties";
const FIXTURE: &str = "security-notes.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/security-notes.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const REGION: &str = "docprops.security-notes";
const DOCK: &str = "dock.right.frame";
const WRAPPER: &str = "wrapper-disclosed"; // ui-text-exempt: a trace event name, never displayed
const NOTES: &str = "security-notes"; // ui-text-exempt: a trace event name, never displayed
/// Each carrier the fixture puts an action on, and the least count owed.
const OWED: [&str; 7] = [
    "page", "outline", "annot", "js", "chained", "network", "launch",
];

/// See the module documentation.
pub struct SecurityNotesNameTheCoverAndTheActions;

impl Check for SecurityNotesNameTheCoverAndTheActions {
    fn name(&self) -> &'static str {
        "security_notes_name_the_cover_and_the_actions"
    }

    fn defect(&self) -> &'static str {
        "a wrapper's cover page opens as if it were the drawing with nothing said, or Security \
         notes misses an action on a page trigger, a bookmark, a link or one hidden behind \
         another action, or the section is never drawn"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch(ctx, &mut report).and_then(|session| steps(&mut report, &session));
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<Session> {
    launch_on(ctx, report, FIXTURE, METHOD, "security_notes")
}

/// Launch on `fixtures/<fixture>` with Document properties open, off the
/// desktop. `method` says how to rebuild a missing fixture; `stem` names the
/// trace.
pub(crate) fn launch_on(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    method: &str,
    stem: &str,
) -> Result<Session> {
    launch_invoking(ctx, report, (fixture, method), stem, INVOKE)
}

/// [`launch_on`] with `invoke` run on opening instead of Document properties.
pub(crate) fn launch_invoking(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (fixture, method): (&str, &str),
    stem: &str,
    invoke: &str,
) -> Result<Session> {
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
    spec.pdf = Some(repo_fixture(fixture, method)?);
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
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok(session)
}

/// The newest `name` line, waiting up to 30 settles.
pub(crate) fn await_line(session: &Session, name: &str) -> Result<Option<TraceLine>> {
    for _ in 0..30 {
        if let Some(line) = session.trace()?.events(name).last() {
            return Ok(Some(line.clone()));
        }
        session.settle(10);
    }
    Ok(None)
}

fn steps(report: &mut CheckReport, session: &Session) -> Result<Option<String>> {
    let Some(cover) = await_line(session, WRAPPER)? else {
        return Ok(Some(format!(
            "{FIXTURE} is a §7.6.7 wrapper and opening it traced no `{WRAPPER}` line: the cover \
             page opened as if it were the drawing, with nothing said."
        )));
    };
    report.note(format!("on open: `{}`", cover.raw));
    if cover.get("payloads") != Some("1") || cover.get("named") != Some("1") {
        return Ok(Some(format!(
            "the fixture names one payload, drawing.pdf; the shell said `{}`.",
            cover.raw
        )));
    }
    let Some(notes) = await_line(session, NOTES)? else {
        return Ok(Some(format!(
            "`{INVOKE}` was invoked and no `{NOTES}` line followed: Document properties never \
             drew Security notes."
        )));
    };
    report.note(format!("census: `{}`", notes.raw));
    let missing: Vec<&str> = OWED
        .into_iter()
        .filter(|k| {
            notes
                .get(k)
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0)
                < 1
        })
        .collect();
    if notes.get("wrapper") != Some("1") || notes.get("truncated") != Some("false") {
        return Ok(Some(format!(
            "Security notes owed wrapper=1 from a finished walk; it traced `{}`.",
            notes.raw
        )));
    }
    if !missing.is_empty() {
        return Ok(Some(format!(
            "the fixture carries an action on every carrier; Security notes counted none on {}. \
             `{}`.",
            missing.join(", "),
            notes.raw
        )));
    }
    let trace = session.trace()?;
    let Some(drawn) = declared(&trace, UI_RECT, REGION) else {
        return Ok(Some(format!(
            "the census was taken but no visible `{REGION}` region was published: the section \
             was measured and never drawn where the operator can see it."
        )));
    };
    let Some(dock) = declared(&trace, UI_RECT, DOCK) else {
        return Ok(Some(format!(
            "no `{DOCK}` region to measure the section against."
        )));
    };
    report.note(format!(
        "section right edge {:.1}, dock right edge {:.1}",
        drawn.max.x, dock.max.x
    ));
    if drawn.max.x > dock.max.x + 0.5 {
        return Ok(Some(format!(
            "Security notes runs past the dock's right edge ({:.1} > {:.1}): a line does not \
             wrap, so its end is cut off.",
            drawn.max.x, dock.max.x
        )));
    }
    Ok(None)
}
