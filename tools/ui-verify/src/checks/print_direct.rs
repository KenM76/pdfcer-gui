//! `print_sends_the_pdf_itself` — with *Send the PDF itself* on, a plain job
//! goes to a printer that reads PDF as the PDF, not through the driver.
//!
//! Runs [`netprint_scripted`]'s local IPP printer and points the dialog at it
//! with `PDFCER_DIAG_PRINT_DIRECT_URI`, so no Windows port is resolved. Opens
//! Print (dry run, off the desktop, scripted pointer) on `labelled-pages.pdf`,
//! turns the switch on, waits for the printer's answer and presses Print.
//! Oracle: a `print-direct route=direct` line, no `print-spool-done` line, and
//! a Print-Job the printer received whose body carries `%PDF-`.
//! Ignores `--pdf`; the fixture is the method.

use std::sync::{Arc, Mutex};

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::netprint_scripted::{Jobs, operation, serve};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "labelled-pages.pdf";
const METHOD: &str = "A four-page PDF with no annotations, so the default scope prints it exactly as a printer reading the PDF would. Rebuild it from its PROVENANCE file.";
const OPEN: &str = "print-open";
const FOUND: &str = "print-direct-found"; // ui-text-exempt: trace event name
const ROUTE: &str = "print-direct"; // ui-text-exempt: trace event name
const SPOOLED: &str = "print-spool-done";
const SWITCH: &str = "print.direct";
/// On the Pages tab above the switch; the wheel goes over it.
const ANCHOR: &str = "print.as-image";
const UI_RECT: &str = "ui-rect";
const PRINT: &str = "dialog.buttons.accept";
const URI_ENV: &str = "PDFCER_DIAG_PRINT_DIRECT_URI"; // ui-text-exempt: an environment variable name
/// IPP Print-Job (RFC 8011 §5.2.1).
const PRINT_JOB: u16 = 0x0002;

pub struct PrintSendsThePdfItself;

impl Check for PrintSendsThePdfItself {
    fn name(&self) -> &'static str {
        "print_sends_the_pdf_itself"
    }

    fn defect(&self) -> &'static str {
        "Send the PDF itself is on, the printer reads PDF, and the job still goes through the driver"
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
    uri: &str,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_direct.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
        ("PDFCER_DIAG_PRINT_DRY_RUN", "1"),
        (URI_ENV, uri),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_direct.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let jobs: Jobs = Arc::new(Mutex::new(Vec::new()));
    let port = serve(Arc::clone(&jobs))?;
    let uri = format!("ipp://127.0.0.1:{port}/ipp/print");
    let (session, pointer) = launch(ctx, report, &uri)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    for _ in 0..8 {
        let trace = session.trace()?;
        if declared_in(&trace, UI_RECT, SWITCH).is_some() {
            break;
        }
        let Some((over, vp)) = declared_in(&trace, UI_RECT, ANCHOR) else {
            break;
        };
        pointer.wheel_in(&session, vp.as_deref(), WindowPoint::centre_of(over), -3.0)?;
        session.settle(10);
    }
    press_region(&session, &pointer, SWITCH, "print.")?;
    session.settle(90);
    let found = session.trace()?.last(FOUND).map(|l| l.raw.clone());
    press_region(&session, &pointer, PRINT, "dialog.")?;
    session.settle(120);
    let trace = session.trace()?;
    let route = trace.last(ROUTE).map(|l| l.raw.clone());
    let spooled = trace.last(SPOOLED).map(|l| l.raw.clone());
    pointer.gone(&session)?;
    let received: Vec<Vec<u8>> = jobs.lock().map(|j| j.clone()).unwrap_or_default();
    if let Some(found) = &found {
        report.note(found.clone());
    }
    let Some(route) = route else {
        return Ok(Some(format!(
            "★★★ Print was pressed with the switch on and no `{ROUTE} route=` line was traced: \
             the direct route was never consulted. Printer answer: {found:?}."
        )));
    };
    report.note(route.clone());
    if !route.contains("route=direct ") {
        return Ok(Some(format!(
            "★★★ the printer at {uri} reads PDF and the job went to the driver: `{route}`."
        )));
    }
    if let Some(spooled) = spooled {
        return Ok(Some(format!(
            "★★★ the job was sent as the PDF and ALSO spooled through the driver: `{spooled}`."
        )));
    }
    let Some(job) = received.iter().find(|b| operation(b) == Some(PRINT_JOB)) else {
        return Ok(Some(format!(
            "★★★ the dialog traced a direct send and the printer received no Print-Job \
             ({} requests).",
            received.len()
        )));
    };
    if !job.windows(5).any(|w| w == b"%PDF-") {
        return Ok(Some(
            "★★★ the Print-Job the printer received carries no PDF.".to_owned(),
        ));
    }
    Ok(None)
}
