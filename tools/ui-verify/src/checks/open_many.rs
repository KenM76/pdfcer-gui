//! `several_documents_open_from_one_open` — one press of Open, answered with
//! two files selected together, opens both, each in a tab of its own.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/open_many.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_names, list, repo_fixture};
use crate::checks::security_notes::await_line;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The seam answering one Open with a multi-selection.
const OPEN_PATHS_ENV: &str = "PDFCER_DIAG_OPEN_PATHS"; // ui-text-exempt: an environment variable name
const FIRST: &str = "four-pages.pdf";
const SELECTED: [(&str, &str); 2] = [
    (
        "esign-three-pages.pdf",
        "Rebuild it with `python fixtures/esign-three-pages.PROVENANCE.py`.",
    ),
    (
        "labelled-pages.pdf",
        "Rebuild it with `python fixtures/labelled-pages.PROVENANCE.py`.",
    ),
];
const PICKED: &str = "open-picked";
const UI_RECT: &str = "ui-rect";
const TAB: &str = "doc-tab.";

/// See the module documentation.
pub struct SeveralDocumentsOpenFromOneOpen;

impl Check for SeveralDocumentsOpenFromOneOpen {
    fn name(&self) -> &'static str {
        "several_documents_open_from_one_open"
    }

    fn defect(&self) -> &'static str {
        "the Open dialog takes one file, so selecting several with Shift or Ctrl opens only the \
         first (or none), and the rest of the selection is dropped without a word"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch(ctx, &mut report).and_then(|s| steps(&mut report, &s));
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<Session> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut selected = Vec::new();
    for (name, method) in SELECTED {
        selected.push(repo_fixture(name, method)?.to_string_lossy().into_owned());
    }
    let mut spec = LaunchSpec::new(&exe, ctx.out("open_many.trace.txt"));
    spec.pdf = Some(repo_fixture(FIRST, "It is a checked-in fixture.")?);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.open"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.env
        .push((OPEN_PATHS_ENV.to_owned(), selected.join(";")));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok(session)
}

fn steps(report: &mut CheckReport, session: &Session) -> Result<Option<String>> {
    let Some(picked) = await_line(session, PICKED)? else {
        return Ok(Some(format!(
            "`file.open` was invoked and no `{PICKED}` line followed: Open never asked."
        )));
    };
    report.note(format!("picker: `{}`", picked.raw));
    // Both selected files must arrive, each named in its own `open ok`.
    for _ in 0..30 {
        if opened(session)?.len() >= 3 {
            break;
        }
        session.settle(10);
    }
    let opened = opened(session)?;
    let missing: Vec<&str> = SELECTED
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| !opened.iter().any(|raw| raw.contains(name)))
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!(
            "two files were selected in one Open and {} never opened. `open ok` lines: {}. \
             Picker: `{}`.",
            missing.join(", "),
            list(&opened),
            picked.raw
        )));
    }
    let tabs = declared_names(&session.trace()?, UI_RECT, TAB);
    if tabs.len() < 3 {
        return Ok(Some(format!(
            "all three documents opened and the strip draws {} tab(s): {}. Each selected file \
             owes a tab of its own.",
            tabs.len(),
            list(&tabs)
        )));
    }
    report.note(format!("tabs: {}", list(&tabs)));
    Ok(None)
}

/// Every `open ok` line so far, raw.
fn opened(session: &Session) -> Result<Vec<String>> {
    Ok(session
        .trace()?
        .events("open")
        .filter(|line| line.raw.contains("open ok"))
        .map(|line| line.raw.clone())
        .collect())
}
