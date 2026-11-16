//! `print_composes_sheets_for_the_pdf` — with *Send the PDF itself* on, a
//! two-per-sheet job goes to a printer that reads PDF as sheets the engine
//! composed, not as the document's own pages and not through the driver.
//!
//! Runs [`netprint_scripted`]'s local IPP printer and points the dialog at it
//! with `PDFCER_DIAG_PRINT_DIRECT_URI`. Opens Print (dry run, off the desktop,
//! scripted pointer) on `labelled-pages.pdf`, chooses Multiple (two pages per
//! sheet by default), turns the switch on, waits for the printer's answer and
//! presses Print.
//!
//! Oracle: a `print-direct-compose` line with `shape=nup sheets=2`; a
//! `print-direct route=direct` line with `composed=1 pages=2` (the document
//! sent as it is would say `composed=0 pages=4`); no `print-spool-done` line;
//! and a Print-Job the printer received whose body carries `%PDF-`, asks
//! `print-scaling` `none`, and names no `orientation-requested`.
//! Ignores `--pdf`; the fixture is the method.

use std::sync::{Arc, Mutex};

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, repo_fixture};
use crate::checks::netprint_scripted::{Jobs, operation, serve};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "labelled-pages.pdf";
const METHOD: &str = "A four-page PDF of one page size and no /Rotate, so two per sheet composes two sheets. Rebuild it from its PROVENANCE file.";
const OPEN: &str = "print-open";
const COMPOSE: &str = "print-direct-compose"; // ui-text-exempt: trace event name
const NOTE: &str = "print-direct-note"; // ui-text-exempt: trace event name
const ROUTE: &str = "print-direct"; // ui-text-exempt: trace event name
const SPOOLED: &str = "print-spool-done";
const MULTIPLE: &str = "print.impose.multiple";
const SWITCH: &str = "print.direct";
/// On the Pages tab above the switch; the wheel goes over it.
const ANCHOR: &str = "print.as-image";
const UI_RECT: &str = "ui-rect";
const PRINT: &str = "dialog.buttons.accept";
const URI_ENV: &str = "PDFCER_DIAG_PRINT_DIRECT_URI"; // ui-text-exempt: an environment variable name
/// IPP Print-Job (RFC 8011 §5.2.1).
const PRINT_JOB: u16 = 0x0002;
/// `print-scaling` = `none`, as RFC 8010 §3.1.4 encodes an attribute: the
/// name, then the value's two-byte length and the value.
const SCALING_NONE: &[u8] = b"print-scaling\x00\x04none";
const ORIENTATION: &[u8] = b"orientation-requested";

pub struct PrintComposesSheetsForThePdf;

impl Check for PrintComposesSheetsForThePdf {
    fn name(&self) -> &'static str {
        "print_composes_sheets_for_the_pdf"
    }

    fn defect(&self) -> &'static str {
        "Send the PDF itself is on, the job is two pages per sheet, and the printer is handed the document's own pages or the driver prints it"
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_direct_compose.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_direct_compose.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Choose Multiple, bring the switch into view, turn it on, press Print.
fn print_two_up(session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, MULTIPLE).ok_or_else(|| {
        Error::new(format!(
            "no `{MULTIPLE}` region. Declared under `print.impose.`: {}.",
            list(&declared_names(&trace, UI_RECT, "print.impose."))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(60);
    for _ in 0..8 {
        let trace = session.trace()?;
        if declared_in(&trace, UI_RECT, SWITCH).is_some() {
            break;
        }
        let Some((over, vp)) = declared_in(&trace, UI_RECT, ANCHOR) else {
            break;
        };
        pointer.wheel_in(session, vp.as_deref(), WindowPoint::centre_of(over), -3.0)?;
        session.settle(10);
    }
    press_region(session, pointer, SWITCH, "print.")?;
    session.settle(90);
    press_region(session, pointer, PRINT, "dialog.")?;
    session.settle(120);
    Ok(())
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
    print_two_up(&session, &pointer)?;
    let trace = session.trace()?;
    let composed = trace.last(COMPOSE).map(|l| l.raw.clone());
    let route = trace.last(ROUTE).map(|l| l.raw.clone());
    let spooled = trace.last(SPOOLED).map(|l| l.raw.clone());
    let notes: Vec<String> = trace.events(NOTE).map(|l| l.raw.clone()).collect();
    pointer.gone(&session)?;
    for n in notes {
        report.note(n);
    }
    let received: Vec<Vec<u8>> = jobs.lock().map(|j| j.clone()).unwrap_or_default();
    Ok(judge(composed, route, spooled, &received, report))
}

/// The verdict over what the dialog traced and the printer received.
fn judge(
    composed: Option<String>,
    route: Option<String>,
    spooled: Option<String>,
    received: &[Vec<u8>],
    report: &mut CheckReport,
) -> Option<String> {
    let Some(route) = route else {
        return Some(format!(
            "★★★ Print was pressed with the switch on and no `{ROUTE} route=` line was traced: \
             the direct route was never consulted."
        ));
    };
    report.note(route.clone());
    if !route.contains("route=direct ") {
        return Some(format!(
            "★★★ the printer reads PDF and the two-per-sheet job went to the driver: `{route}`."
        ));
    }
    let Some(composed) = composed else {
        return Some(format!(
            "★★★ the two-per-sheet job was sent as the PDF and no `{COMPOSE}` line was traced: \
             the engine never laid its sheets out. Route: `{route}`."
        ));
    };
    report.note(composed.clone());
    let fields = |line: &str, want: &[&str]| -> Vec<String> {
        want.iter()
            .filter(|f| !line.split_whitespace().any(|w| w == **f))
            .map(|f| (*f).to_owned())
            .collect()
    };
    let missing = fields(&composed, &["shape=nup", "sheets=2"]);
    if !missing.is_empty() {
        return Some(format!(
            "★★★ four pages two per sheet should compose two n-up sheets; the compose line lacks {}: `{composed}`.",
            list(&missing)
        ));
    }
    let missing = fields(&route, &["composed=1", "pages=2"]);
    if !missing.is_empty() {
        return Some(format!(
            "★★★ the composed sheets were not what was sent; the route line lacks {}: `{route}`.",
            list(&missing)
        ));
    }
    if let Some(spooled) = spooled {
        return Some(format!(
            "★★★ the job was sent as the PDF and ALSO spooled through the driver: `{spooled}`."
        ));
    }
    let Some(job) = received.iter().find(|b| operation(b) == Some(PRINT_JOB)) else {
        return Some(format!(
            "★★★ the dialog traced a direct send and the printer received no Print-Job \
             ({} requests).",
            received.len()
        ));
    };
    let has = |needle: &[u8]| job.windows(needle.len()).any(|w| w == needle);
    if !has(b"%PDF-") {
        return Some("★★★ the Print-Job the printer received carries no PDF.".to_owned());
    }
    if !has(SCALING_NONE) {
        return Some(
            "★★★ composed sheets were sent without `print-scaling none`: the printer may scale \
             sheets that are already final."
                .to_owned(),
        );
    }
    if has(ORIENTATION) {
        return Some(
            "★★★ composed sheets were sent with `orientation-requested`: the printer may turn \
             sheets whose orientation is already in the PDF."
                .to_owned(),
        );
    }
    None
}
