//! `print_header_footer_reaches_the_printed_copy` — a footer typed on the
//! Print dialog's Comments tab is stamped on a copy, and the preview renders
//! that copy.
//!
//! Driven off the desktop with the scripted pointer: `file.print` is rung on
//! `print-impose.pdf`, five Letter pages. The check opens the Comments tab,
//! clicks the footer box, types `Page {page} of {pages}` and presses Enter,
//! which leaves the box and applies the line. Oracles:
//!
//! - `print-running built pages=5 lines=1 first="Page 1 of 5"`: the engine
//!   stamped every page and expanded both tokens. A GUI that drew its own
//!   text, or stamped nothing, writes no such line.
//! - a `print-preview-render` after it with `stamp=` not 0: the preview
//!   re-rendered from the copy rather than keeping the document's raster.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "print-impose.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/print-impose.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const OPEN: &str = "print-open";
const BUILT: &str = "print-running";
const RENDER: &str = "print-preview-render";
const TAB: &str = "print.tab.comments";
const FOOTER: &str = "print.running.footer";
const TYPED: &str = "Page {page} of {pages}";
const EXPECTED: &str = "first=\"Page 1 of 5\"";

/// See the module documentation.
pub struct PrintHeaderFooterReachesThePrintedCopy;

impl Check for PrintHeaderFooterReachesThePrintedCopy {
    fn name(&self) -> &'static str {
        "print_header_footer_reaches_the_printed_copy"
    }

    fn defect(&self) -> &'static str {
        "a header or footer typed in the Print dialog is not stamped on the printed copy, or \
         the preview does not show it"
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_running.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_running.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Click the centre of region `name`; return the viewport it is declared in.
fn click(session: &Session, pointer: &ScriptedPointer, name: &str) -> Result<Option<String>> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region. Declared under `print.`: {}.",
            list(&declared_names(&trace, UI_RECT, "print."))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(viewport)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    click(&session, &pointer, TAB)?;
    let viewport = click(&session, &pointer, FOOTER)?;
    pointer.type_text(&session, viewport.as_deref(), TYPED)?;
    pointer.key(&session, viewport.as_deref(), "Enter", None)?;
    session.settle(90);
    let trace = session.trace()?;
    let Some(built) = trace.last(BUILT).map(|l| l.raw.clone()) else {
        return Ok(Some(format!(
            "★★★ typed `{TYPED}` in the footer and pressed Enter; no `{BUILT}` line: \
             nothing was stamped."
        )));
    };
    report.note(built.clone());
    if !built.contains("built pages=5 lines=1") || !built.contains(EXPECTED) {
        return Ok(Some(format!(
            "★★★ the stamped copy is not five pages carrying `Page 1 of 5` first: `{built}`."
        )));
    }
    let rendered = trace
        .last(RENDER)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(rendered.clone());
    if rendered.is_empty() || rendered.contains("stamp=0") {
        return Ok(Some(format!(
            "★★★ the copy was stamped but the preview last rendered the document itself: \
             `{rendered}`."
        )));
    }
    pointer.gone(&session)?;
    Ok(None)
}
