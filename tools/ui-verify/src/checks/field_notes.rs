//! `a_field_drawn_otherwise_says_why_in_the_forms_panel` — the Forms panel
//! shows, beside a field, what pdfcer's drawing of it showed in place of or
//! ignored from its stored state, in the engine's own words.
//!
//! Driven off the desktop on `fixtures/field-display-notes.pdf`, with the
//! Forms panel opened by command. Oracles:
//!
//! - `Amount` (an unreproduced format script) has a `forms-field-note
//!   kind=unformatted` line whose reason carries the engine's sentence,
//!   and its row declares `forms.fill.note.0`;
//! - `Pick` (`/TI 7` over two options) has `kind=ignored-top-index` naming
//!   7, and its row declares `forms.fill.note.1`;
//! - `Plain` has no note line and no note region.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

const OFFSCREEN: &str = "-4200,-4200,1400,1100";
const FIXTURE: &str = "field-display-notes.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/field-display-notes.PROVENANCE.py`.";
const INVOKE: &str = "mode.edit,view.panel_forms";
const NOTE: &str = "forms-field-note";
/// The engine's words for a format script it does not reproduce.
const UNRECOGNISED: &str = "not a recognised AF*_Format call";

/// See the module documentation.
pub struct AFieldDrawnOtherwiseSaysWhyInTheFormsPanel;

impl Check for AFieldDrawnOtherwiseSaysWhyInTheFormsPanel {
    fn name(&self) -> &'static str {
        "a_field_drawn_otherwise_says_why_in_the_forms_panel"
    }

    fn defect(&self) -> &'static str {
        "a field drawn with its value unformatted, or a list box whose stored rows were ignored, \
         looks ordinary in the Forms panel: nothing says the drawing differs from what is stored"
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, &'static str)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let doc = ctx.out("field_notes.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("field_notes.trace.txt"));
    spec.pdf = Some(doc);
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
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(90);
    Ok((session, ui_rect))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, ui_rect) = launch(ctx, report)?;
    let trace = session.trace()?;
    for line in trace.events(NOTE) {
        report.note(line.raw.clone());
    }
    if let Some(f) = expect(&trace, ui_rect, "0", "unformatted", UNRECOGNISED) {
        return Ok(Some(f));
    }
    if let Some(f) = expect(&trace, ui_rect, "1", "ignored-top-index", "7") {
        return Ok(Some(f));
    }
    if trace.events(NOTE).any(|l| l.get("row") == Some("2"))
        || declared(&trace, ui_rect, "forms.fill.note.2").is_some()
    {
        return Ok(Some(
            "★★★ `Plain`, drawn as stored, has a note: the notes are not the engine's.".to_owned(),
        ));
    }
    Ok(None)
}

/// Row `row` has a `kind` note whose reason contains `words`, and declares
/// its note region.
fn expect(trace: &Trace, ui_rect: &str, row: &str, kind: &str, words: &str) -> Option<String> {
    let found = trace.events(NOTE).any(|l| {
        l.get("row") == Some(row)
            && l.get("kind") == Some(kind)
            && l.get("reason").is_some_and(|r| r.contains(words))
    });
    if !found {
        return Some(format!(
            "★★★ no `{NOTE} row={row} kind={kind}` line whose reason carries `{words}`: the \
             engine's note for that field never reached the panel."
        ));
    }
    let region = format!("forms.fill.note.{row}");
    if declared(trace, ui_rect, &region).is_none() {
        return Some(format!(
            "★★★ the note was read but `{region}` was never declared: the panel does not show it."
        ));
    }
    None
}
