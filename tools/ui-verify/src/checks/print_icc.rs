//! `print_simulates_a_press` — the Print dialog's colour management converts
//! the preview through a chosen ICC profile, the same transform built for the
//! sheets.
//!
//! Driven off the desktop with the scripted pointer: `file.print` is rung on
//! `print-colour.pdf`, a Letter page about 63 % saturated red. The check opens
//! the Colour tab, opens the Simulate press list and picks the first CMYK
//! profile the dialog listed. Oracle: a `print-icc of=preview` line with more
//! than half the raster's pixels changed. A saturated sRGB red lies outside
//! every press gamut, so a proof through any CMYK condition must move it.
//! A machine with no CMYK profile installed SKIPS, naming that.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "print-colour.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/print-colour.PROVENANCE.py`.";
const OPEN: &str = "print-open";
const ICC: &str = "print-icc";
const LISTED: &str = "print-icc-listed";
const LISTS: &str = "print-icc-lists";
const TAB: &str = "print.tab.colour";
const PROOF: &str = "print.icc.proof";

/// See the module documentation.
pub struct PrintSimulatesAPress;

impl Check for PrintSimulatesAPress {
    fn name(&self) -> &'static str {
        "print_simulates_a_press"
    }

    fn defect(&self) -> &'static str {
        "the Print dialog cannot proof a press condition through an ICC profile, or its preview \
         does not show the conversion"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Verdict::Pass) => report.pass(),
            Ok(Verdict::Fail(failure)) => report.fail(failure),
            Ok(Verdict::Skip(why)) => report.skip(why),
            Err(why) => report.from_error(&why),
        }
    }
}

enum Verdict {
    Pass,
    Fail(String),
    Skip(String),
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_icc.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_icc.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Verdict> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    press_region(&session, &pointer, TAB, "print.tab.")?;
    let trace = session.trace()?;
    let cmyk = trace
        .events(LISTED)
        .find(|l| l.get("list") == Some(PROOF) && l.get("space") == Some("cmyk"))
        .map(|l| (l.get_usize("item"), l.raw.clone()));
    if trace.last(LISTS).is_none() {
        return Ok(Verdict::Fail(format!(
            "★★★ the Colour tab was opened and no `{LISTS}` line followed: the \
             colour-management controls never listed the installed profiles."
        )));
    }
    let Some((Some(item), listed)) = cmyk else {
        return Ok(Verdict::Skip(
            "no CMYK profile is installed on this machine to simulate.".to_owned(),
        ));
    };
    report.note(listed);
    press_region(&session, &pointer, PROOF, "print.icc.")?;
    let before = session.trace()?.events(ICC).count();
    press_region(&session, &pointer, &format!("{PROOF}.item.{item}"), PROOF)?;
    let trace = session.trace()?;
    let Some(line) = trace
        .events(ICC)
        .skip(before)
        .find(|l| l.get("of") == Some("preview"))
    else {
        return Ok(Verdict::Fail(format!(
            "★★★ picking proof item {item} traced no `{ICC} of=preview` line: the preview was \
             not converted. Chosen: {}",
            trace
                .last("print-icc-chosen")
                .map_or("no `print-icc-chosen` line", |l| l.raw.as_str())
        )));
    };
    let (changed, pixels) = (
        line.get_usize("changed").unwrap_or(0),
        line.get_usize("pixels").unwrap_or(0),
    );
    report.note(line.raw.clone());
    if changed * 2 <= pixels {
        return Ok(Verdict::Fail(format!(
            "★★★ the proof changed {changed} of {pixels} preview pixels, on a page more than \
             half a red no press can print."
        )));
    }
    pointer.gone(&session)?;
    Ok(Verdict::Pass)
}
