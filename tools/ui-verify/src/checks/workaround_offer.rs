//! # `a_refused_edit_offers_its_workaround`
//!
//! Types into a line of `quote-operator.pdf` drawn with the `'` operator. The
//! engine refuses that edit and names a workaround; the check asserts that the
//! Properties panel offers it, that nothing was applied before the press, and
//! that pressing *Make the edit this way* commits the same edit through
//! `rewrite-quote-operator`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/workaround_offer.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, declared, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::Click;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "quote-operator.pdf";
const INVOKE: &str = "mode.edit,edit.text";
const PROPERTIES_PANEL: &str = "file.properties";
/// Inside `Quoted line`, whose baseline is y=680 at 12 pt.
const QUOTED: (f64, f64) = (100.0, 684.0);
const OFFER: &str = "workaround-offer"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "edit-text-workaround"; // ui-text-exempt: a trace event name, never displayed
const LEFT_EDGE: &str = "edit-text-left-edge"; // ui-text-exempt: a trace event name, never displayed
const APPLY_REGION: &str = "properties.workaround.apply"; // ui-text-exempt: a region name, never displayed

pub struct ARefusedEditOffersItsWorkaround;

impl Check for ARefusedEditOffersItsWorkaround {
    fn name(&self) -> &'static str {
        "a_refused_edit_offers_its_workaround"
    }

    fn defect(&self) -> &'static str {
        "an edit the engine refuses but could make another way is simply refused: no offer is \
         shown, or pressing it does not make the edit the way it named"
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

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer, std::path::PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(FIXTURE, "Run fixtures/quote-operator.PROVENANCE.py.")?;
    let doc = ctx.out("workaround-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("workaround.trace.txt"));
    spec.pdf = Some(doc.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("workaround.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    driving::raise_dock_tab(&session, &pointer, ui_rect, PROPERTIES_PANEL)?;
    session.settle(14);
    let path = session.trace_path().display().to_string();
    if session.trace()?.events(OFFER).next().is_some() {
        return Ok(Some(format!(
            "★ `{OFFER}` was traced before any edit was made, so the offer is not tied to a \
             refusal. Trace: {path}."
        )));
    }
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, QUOTED.0, QUOTED.1))?;
    pointer.click(&session, at)?;
    session.settle(20);
    pointer.key(&session, None, "End", None)?;
    pointer.type_text(&session, None, "s")?;
    session.settle(20);
    pointer.key(&session, None, "Escape", None)?;
    session.settle(30);
    let trace = session.trace()?;
    if let Some(failure) = judge_refusal(&trace, &path) {
        return Ok(Some(failure));
    }
    let Some(apply) = declared(&trace, ui_rect, APPLY_REGION) else {
        return Ok(Some(format!(
            "★★ the offer was traced and no `{APPLY_REGION}` region was declared, so its button \
             is not on screen. Regions beginning `properties.`: {}. Trace: {path}.",
            list(&declared_names(&trace, ui_rect, "properties."))
        )));
    };
    report.note("the refused edit raised the rewrite-quote-operator offer");
    let edges_before = trace.events(LEFT_EDGE).count();
    pointer.click_rect(&session, apply)?;
    session.settle(30);
    let trace = session.trace()?;
    let applied = trace.events(APPLIED).last().map(|l| l.raw.clone());
    let committed = trace
        .events(LEFT_EDGE)
        .skip(edges_before)
        .last()
        .map(|l| l.raw.clone());
    pointer.gone(&session)?;
    match (&applied, &committed) {
        (Some(a), Some(c))
            if a.contains("used=rewrite-quote-operator")
                && a.contains("exact=1")
                && c.contains("committed=yes") =>
        {
            report.note(format!("pressed: `{a}`"));
            Ok(None)
        }
        _ => Ok(Some(format!(
            "★★★ pressing the offer did not commit the edit through rewrite-quote-operator. \
             Workaround line: {applied:?}; commit: {committed:?}. Trace: {path}."
        ))),
    }
}

/// The refused commit must change nothing and raise the named offer.
fn judge_refusal(trace: &crate::trace::Trace, path: &str) -> Option<String> {
    if let Some(early) = trace.events(APPLIED).next() {
        return Some(format!(
            "★ a workaround was applied before the operator pressed anything: `{}`. Trace: \
             {path}.",
            early.raw
        ));
    }
    let refused = trace
        .events(LEFT_EDGE)
        .last()
        .is_some_and(|l| l.raw.contains("committed=no"));
    let offer = trace.events(OFFER).last().map(|l| l.raw.clone());
    match offer {
        Some(o) if refused && o.contains("workaround=rewrite-quote-operator") => None,
        o => Some(format!(
            "★ typing into the `'` line and pressing Escape did not leave a refused edit with \
             the rewrite-quote-operator offer. Refused: {refused}; offer: {o:?}. Trace: {path}."
        )),
    }
}
