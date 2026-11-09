//! `print_sends_vector_unless_as_image` — the Print dialog sends plain pages
//! as vector graphics, and its Print as image switch sends them as pictures
//! and is remembered.
//!
//! Driven off the desktop with the scripted pointer, `file.print` rung on
//! `four-pages-unrotated.pdf` (four plain pages, no transparency), and
//! `PDFCER_DIAG_PRINT_DRY_RUN=1` so no press reaches a device. Two launches
//! on one preferences file:
//!
//! 1. click the switch, press Print: `print-spool-done` says `vector=0
//!    fell_back=0`. It is the only one of the two orders that can tell the
//!    switch from no route at all.
//! 2. relaunch: the switch opens ticked, so one click unticks it, and Print
//!    gives `vector=4 fell_back=0`. An unremembered switch opens unticked, the
//!    click ticks it, and the spool says `vector=0`. The second launch also
//!    leaves the preference off for the checks that follow.
//!
//! 3. `transparency-cmyk.pdf`, switch untouched: its one page has
//!    transparency, so the engine declines the vector route and the spool
//!    says `vector=0 fell_back=1` with the reason in `why=`.
//!
//! Every spool must say `printed=false`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "four-pages-unrotated.pdf";
const DECLINED: &str = "transparency-cmyk.pdf";
const METHOD: &str = "Rebuild it from its PROVENANCE file in `fixtures/`.";
const OPEN: &str = "print-open";
const SPOOLED: &str = "print-spool-done";
const SWITCH: &str = "print.as-image";
const PRINT: &str = "dialog.buttons.accept";

/// See the module documentation.
pub struct PrintSendsVectorUnlessAsImage;

impl Check for PrintSendsVectorUnlessAsImage {
    fn name(&self) -> &'static str {
        "print_sends_vector_unless_as_image"
    }

    fn defect(&self) -> &'static str {
        "the Print dialog does not send plain pages as vector graphics, or its Print as image \
         switch does not send pictures, or is not remembered"
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
    round: usize,
    fixture: &str,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(fixture);
    std::fs::copy(repo_fixture(fixture, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {fixture}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("print_route.{round}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
        ("PDFCER_DIAG_PRINT_DRY_RUN", "1"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("print_route.{round}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// One launch on `fixture`: click the switch when `click`, press Print, and
/// return the spool line.
fn round(
    ctx: &CheckContext,
    report: &mut CheckReport,
    n: usize,
    fixture: &str,
    click: bool,
) -> Result<String> {
    let (session, pointer) = launch(ctx, report, n, fixture)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    if click {
        press_region(&session, &pointer, SWITCH, "print.")?;
    }
    press_region(&session, &pointer, PRINT, "dialog.")?;
    session.settle(240);
    let spool = session
        .trace()?
        .last(SPOOLED)
        .map(|l| l.raw.clone())
        .ok_or_else(|| {
            Error::new(format!(
                "Print was pressed and no `{SPOOLED}` line followed."
            ))
        })?;
    report.note(spool.clone());
    pointer.gone(&session)?;
    Ok(spool)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ticked = round(ctx, report, 1, FIXTURE, true)?;
    if !ticked.contains("printed=false") {
        return Ok(Some(format!("★★★ a spool started a real job: `{ticked}`.")));
    }
    if !ticked.contains(" vector=0 fell_back=0 ") {
        return Ok(Some(format!(
            "★★★ Print as image was ticked and pages still went as vector graphics: `{ticked}`."
        )));
    }
    let unticked = round(ctx, report, 2, FIXTURE, true)?;
    if !unticked.contains("printed=false") {
        return Ok(Some(format!(
            "★★★ a spool started a real job: `{unticked}`."
        )));
    }
    if !unticked.contains(" vector=4 fell_back=0 ") {
        return Ok(Some(format!(
            "★★★ the relaunched dialog, its remembered switch clicked off, did not send the four \
             plain pages as vector graphics: `{unticked}`. `vector=0` here means the switch was \
             not remembered, so the click ticked it."
        )));
    }
    let declined = round(ctx, report, 3, DECLINED, false)?;
    if !declined.contains("printed=false") {
        return Ok(Some(format!(
            "★★★ a spool started a real job: `{declined}`."
        )));
    }
    if !declined.contains(" vector=0 fell_back=1 ") || declined.ends_with("why=") {
        return Ok(Some(format!(
            "★★★ {DECLINED}'s one page has transparency the vector route cannot carry; it must \
             go as an image with its reason named: `{declined}`."
        )));
    }
    Ok(None)
}
