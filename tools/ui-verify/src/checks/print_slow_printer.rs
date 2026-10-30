//! `print_dialog_never_waits_on_a_printer` — a default printer that takes
//! seconds to answer must not hold the Print window shut.
//!
//! Driven off the desktop with no input: `PDFCER_DIAG_SLOW_PRINTER` makes every
//! read of the default printer sleep [`DELAY`] seconds, and `file.print` is rung
//! through `PDFCER_DIAG_INVOKE`. Oracles, in order:
//!
//! 1. `printer-probe-start` — the printers began reading at launch.
//! 2. `print-open` within [`EARLY`] frames, long before the delay is up — the
//!    dialog was built without waiting.
//! 3. A `print-plan pending=true` line — the dialog drew while the printer was
//!    still outstanding, and said so.
//! 4. A later `print-plan pending=false sheets=Some(..)` — the answer arrived
//!    off the UI thread and the job was planned from it.
//!
//! Skipped where the spooler lists no printers, since there is then nothing to
//! slow.

use std::time::{Duration, Instant};

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Seconds each read of the default printer is held up.
const DELAY: u64 = 10;
/// Frames the dialog has to appear in; at 25 ms a frame, well under [`DELAY`].
const EARLY: u32 = 60;
/// How long to wait for the planned job once the delayed answers land.
const LATE: Duration = Duration::from_secs(4 * DELAY + 30);
const PROBE: &str = "printer-probe-start";
const OPEN: &str = "print-open";
const PLAN: &str = "print-plan";

/// See the module documentation.
pub struct PrintDialogNeverWaitsOnAPrinter;

impl Check for PrintDialogNeverWaitsOnAPrinter {
    fn name(&self) -> &'static str {
        "print_dialog_never_waits_on_a_printer"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Print freezes the program until a slow or unreachable printer answers"
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
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. `file.print` is gated on `doc.open`."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let session = launch(ctx, report, &exe, pdf, viewport_env)?;
    let started = Instant::now();
    session.settle(EARLY);
    early(&session, report, started)
}

/// Launch with the default printer held [`DELAY`] seconds and Print rung.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    exe: &std::path::Path,
    pdf: std::path::PathBuf,
    viewport_env: &str,
) -> Result<Session> {
    let slow = format!("<default>={DELAY}");
    let mut spec = LaunchSpec::new(exe, ctx.out("print_slow_printer.trace.txt"));
    spec.pdf = Some(pdf);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
        ("PDFCER_DIAG_SLOW_PRINTER", slow.as_str()),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    Ok(session)
}

/// The dialog opened while the printer was still held, and said so.
fn early(session: &Session, report: &mut CheckReport, started: Instant) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(probe) = trace.last(PROBE) else {
        return Ok(Some(format!(
            "★★★ no `{PROBE}` line: the printers were not read in the background at launch. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    if probe.get("printers") == Some("0") {
        return Err(Error::new(
            "the spooler lists no printers on this machine, so there is no printer to slow. \
             Reported as SKIPPED.",
        ));
    }
    let Some(open) = trace.last(OPEN) else {
        return Ok(Some(format!(
            "★★★ `file.print` was rung and no `{OPEN}` line followed within {EARLY} frames \
             ({:.1} s since launch), while the default printer was held {DELAY} s: the dialog \
             waited on the printer. Trace: {}.",
            started.elapsed().as_secs_f32(),
            session.trace_path().display()
        )));
    };
    report.note(open.raw.clone());
    let early_pending = trace.events(PLAN).any(|l| l.get("pending") == Some("true"));
    if !early_pending {
        return Ok(Some(format!(
            "★★★ the dialog opened but no `{PLAN} pending=true` line was traced while the default \
             printer was held {DELAY} s: either it read the printer on the UI thread, or the \
             slow-printer variable did not reach it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    late_plan(session, report)
}

/// Wait for the plan made from the delayed answer.
fn late_plan(session: &Session, report: &mut CheckReport) -> Result<Option<String>> {
    let deadline = Instant::now() + LATE;
    loop {
        let trace = session.trace()?;
        if let Some(done) = trace.events(PLAN).find(|l| {
            l.get("pending") == Some("false")
                && l.get("sheets").is_some_and(|s| s.starts_with("Some"))
        }) {
            report.note(done.raw.clone());
            return Ok(None);
        }
        if Instant::now() >= deadline {
            return Ok(Some(format!(
                "★★★ the default printer's answer was due after about {} s and no `{PLAN} \
                 pending=false sheets=Some(..)` line arrived within {} s: the dialog never \
                 planned the job from the late answer. Trace: {}.",
                2 * DELAY,
                LATE.as_secs(),
                session.trace_path().display()
            )));
        }
        session.settle(40);
    }
}
