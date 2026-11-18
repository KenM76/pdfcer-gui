//! `use_these_pages_makes_the_form_the_document` — Form preview ▸ Use these
//! pages writes a dynamic XFA form's computed pages over its stand-in.
//!
//! Opens a copy of `fixtures/dynamic-xfa.pdf` (one stand-in page, one computed
//! page), waits for the preview that opens with it, presses Use these pages,
//! and requires the engine's outcome on the trace — one page written over one
//! replaced — with the status-row disclosure attached and the window closed.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "dynamic-xfa.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/dynamic-xfa.PROVENANCE.py`.";
const DRAWN: &str = "form-preview";
const DIALOG: &str = "dialog:form-preview";
const USE: &str = "form-preview.use";
const APPLIED: &str = "form-pages-applied";
/// The funnel's line for the edit, carrying its disclosures.
const FUNNEL: &str = "form-pages";

/// See the module documentation.
pub struct UseThesePagesMakesTheFormTheDocument;

impl Check for UseThesePagesMakesTheFormTheDocument {
    fn name(&self) -> &'static str {
        "use_these_pages_makes_the_form_the_document"
    }

    fn defect(&self) -> &'static str {
        "Form preview offers no way to make the form's pages the document's, or pressing Use \
         these pages leaves the stand-in page in place, writes it silently, or leaves the \
         window open"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let doc = ctx.out("form_pages.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("form_pages.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("form_pages.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(90);
    press_use(ctx, report, ui_rect, &session, &pointer)
}

/// Press Use these pages in the preview that opened with the form, and judge
/// the engine's outcome, the funnel's disclosure and the window's closing.
fn press_use(
    ctx: &CheckContext,
    report: &mut CheckReport,
    ui_rect: &str,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    if trace.last(DRAWN).is_none() {
        return Err(Error::new(format!(
            "the preview that opens with a dynamic form traced no `{DRAWN}` line, so there is \
             nothing to use; `a_dynamic_form_opens_on_its_own_pages` owns that defect. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some((rect, vp)) = declared_in(&trace, ui_rect, USE) else {
        return Ok(Some(format!(
            "★★★ the preview drew the form and declared no `{USE}` region: there is no Use \
             these pages button. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let before = ctx.out("form_pages.before.png");
    pointer.screenshot(session, &before)?;
    report.artifact(before);
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(60);
    let after = ctx.out("form_pages.after.png");
    pointer.screenshot(session, &after)?;
    report.artifact(after);

    let trace = session.trace()?;
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "★★★ pressing Use these pages traced no `{APPLIED}` line: the engine wrote no page. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(applied.raw.clone());
    if applied.get("written") != Some("1") || applied.get("replaced") != Some("1") {
        return Ok(Some(format!(
            "★★★ `{}` must carry written=1 replaced=1: the fixture's one computed page replaces \
             its one stand-in.",
            applied.raw
        )));
    }
    let Some(funnel) = trace
        .events(FUNNEL)
        .find(|l| l.raw.contains("disclosures=") && l.raw.contains("epoch="))
    else {
        return Ok(Some(format!(
            "★★★ no `{FUNNEL}` edit line followed `{APPLIED}`: the edit did not pass the funnel, \
             so it is not on the undo stack."
        )));
    };
    if funnel.raw.contains("disclosures=none") {
        return Ok(Some(format!(
            "★★★ `{}` disclosed nothing: every written page is inferred, and the status row \
             must say so.",
            funnel.raw
        )));
    }
    report.note(funnel.raw.clone());
    if declared(&trace, ui_rect, DIALOG).is_some() {
        return Ok(Some(
            "★★★ the preview window was still drawn after Use these pages: it shows the stand-in \
             document's form over pages that are now the form's own."
                .to_owned(),
        ));
    }
    Ok(None)
}
