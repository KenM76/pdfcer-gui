//! `remove_ocr_text_takes_the_pages_chosen` — File ▸ Remove OCR text asks
//! which pages, as Recognise text does, and takes off only the layers on the
//! pages chosen. Driven with the scripted pointer in a window placed off the
//! desktop, on a copy of `fixtures/ocr-layers.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/remove_ocr_pages.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::security_notes::await_line;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "ocr-layers.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/ocr-layers.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.remove_ocr"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const THIS_PAGE: &str = "remove-ocr.scope.current"; // ui-text-exempt: a trace region name, never displayed
const COMMIT: &str = "remove-ocr.commit"; // ui-text-exempt: a trace region name, never displayed
const NAMED: &str = "remove-ocr-named"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "remove-ocr-layers-applied"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct RemoveOcrTextTakesThePagesChosen;

impl Check for RemoveOcrTextTakesThePagesChosen {
    fn name(&self) -> &'static str {
        "remove_ocr_text_takes_the_pages_chosen"
    }

    fn defect(&self) -> &'static str {
        "Remove OCR text offers no choice of pages, or takes the layers off every page \
         whatever was chosen"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = drive(&mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            let outcome = outcome?;
            parked?;
            Ok(outcome)
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Open the window, choose This page (page 1), remove; then open it again
/// and see page 2's layer still named.
fn drive(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    open_window(session, pointer)?;
    let Some(named) = await_line(session, NAMED)? else {
        return Ok(Some(format!(
            "Remove OCR text opened no window naming the layers it would take: no `{NAMED}` \
             line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("opened: `{}`", named.raw));
    if named.get("layers") != Some("2") || named.get("pages") != Some("2") {
        return Ok(Some(format!(
            "the window opened on All pages and named `{}`, not the fixture's two layers on \
             two pages.",
            named.raw
        )));
    }
    click(session, pointer, THIS_PAGE)?;
    let named = session.trace()?.last(NAMED).map(|l| l.raw.clone());
    report.note(format!("This page: {named:?}"));
    click(session, pointer, COMMIT)?;
    session.settle(30);
    let Some(applied) = session.trace()?.last(APPLIED).map(|l| l.raw.clone()) else {
        return Ok(Some(format!(
            "Remove was pressed and no `{APPLIED}` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("removed: `{applied}`"));
    if !applied.contains("removed=1 pages=1") {
        return Ok(Some(format!(
            "This page was chosen on page 1 and the removal reported `{applied}`, not one \
             layer on one page."
        )));
    }
    let before = session.trace()?.events(NAMED).count();
    open_window(session, pointer)?;
    let trace = session.trace()?;
    let again = trace
        .events(NAMED)
        .skip(before)
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("reopened: {again:?}"));
    if !again
        .as_deref()
        .is_some_and(|l| l.contains("layers=1 pages=1"))
    {
        return Ok(Some(format!(
            "after removing page 1's layer the window reopened naming {again:?}, not page 2's \
             one layer left."
        )));
    }
    Ok(None)
}

fn open_window(session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    click(session, pointer, TAB)?;
    let trace = session.trace()?;
    if declared(&trace, UI_RECT, ITEM).is_none() && declared(&trace, UI_RECT, COLLAPSED).is_some() {
        click(session, pointer, COLLAPSED)?;
    }
    click(session, pointer, ITEM)?;
    session.settle(20);
    Ok(())
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(FIXTURE, METHOD)?;
    let doc = ctx.out("remove-ocr-pages.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("remove-ocr-pages.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "mode.edit"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("remove-ocr-pages.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    Ok((session, pointer))
}

/// Click `name` in whichever viewport declared it.
fn click(session: &Session, pointer: &ScriptedPointer, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        let prefix = name.rsplit_once('.').map_or(name, |(head, _)| head);
        Error::new(format!(
            "no `{name}` region. Declared under `{prefix}`: {}.",
            list(&declared_names(&trace, UI_RECT, prefix))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(15);
    Ok(())
}
