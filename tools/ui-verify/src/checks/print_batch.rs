//! `batch_print_prints_each_file` — Tools ▸ Batch ▸ Batch print… opens the
//! Print window on the files picked, and one Print press spools each of them
//! in order with that window's settings.
//!
//! Driven off the desktop with the scripted pointer and nothing open. The
//! picker seam hands over `print-impose.pdf` (five pages), a file that is not
//! a PDF, and `four-pages-unrotated.pdf`, in that order.
//! `PDFCER_DIAG_PRINT_DRY_RUN=1` makes every spool a dry run: the engine does
//! everything but start the job, so the press reaches no paper. Oracles:
//!
//! - `print-batch loaded=2 skipped=1`: the bad file is left out, not fatal.
//! - after the press, two `print-batch-file … outcome=Ok(..)` lines naming the
//!   two PDFs in the picked order, and `print-batch-done files=2`.
//! - every `print-spool-done` says `printed=false`, and there are two: one job
//!   per file, and none of them reached a device.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIRST: &str = "print-impose.pdf";
const SECOND: &str = "four-pages-unrotated.pdf";
const BAD: &str = "not-a-pdf.pdf";
const METHOD: &str = "Rebuild it from its PROVENANCE file in `fixtures/`.";
const UI_RECT: &str = "ui-rect";
const LOADED: &str = "print-batch";
const FILE: &str = "print-batch-file";
const DONE: &str = "print-batch-done";
const SPOOLED: &str = "print-spool-done";
const PRINT: &str = "dialog.buttons.accept";

/// See the module documentation.
pub struct BatchPrintPrintsEachFile;

impl Check for BatchPrintPrintsEachFile {
    fn name(&self) -> &'static str {
        "batch_print_prints_each_file"
    }

    fn defect(&self) -> &'static str {
        "Batch print does not send each picked file, in order, as its own job"
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

fn sources(ctx: &CheckContext) -> Result<String> {
    let mut picked = Vec::new();
    for name in [FIRST, BAD, SECOND] {
        let to = ctx.out(name);
        if name == BAD {
            std::fs::write(&to, b"this is not a PDF\n")
        } else {
            std::fs::copy(repo_fixture(name, METHOD)?, &to).map(|_| ())
        }
        .map_err(|e| Error::new(format!("writing {name}: {e}")))?;
        picked.push(to.display().to_string());
    }
    Ok(picked.join(";"))
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let picked = sources(ctx)?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_batch.trace.txt"));
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "tools.batch_print"),
        ("PDFCER_DIAG_BATCH_PRINT_SOURCES", picked.as_str()),
        ("PDFCER_DIAG_PRINT_DRY_RUN", "1"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_batch.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(90);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let trace = session.trace()?;
    let Some(loaded) = trace.last(LOADED).map(|l| l.raw.clone()) else {
        return Ok(Some(format!(
            "★★★ `tools.batch_print` was rung with three files picked; no `{LOADED}` line."
        )));
    };
    report.note(loaded.clone());
    if !loaded.contains("loaded=2 skipped=1") {
        return Ok(Some(format!(
            "★★★ expected the two PDFs opened and the text file left out: `{loaded}`."
        )));
    }
    let (rect, viewport) = declared_in(&trace, UI_RECT, PRINT).ok_or_else(|| {
        Error::new(format!(
            "no `{PRINT}` region: the window has no job to print. Declared under `dialog.`: {}.",
            list(&declared_names(&trace, UI_RECT, "dialog."))
        ))
    })?;
    pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(240);
    let trace = session.trace()?;
    let files: Vec<String> = trace.events(FILE).map(|l| l.raw.clone()).collect();
    let spools: Vec<String> = trace.events(SPOOLED).map(|l| l.raw.clone()).collect();
    for line in files.iter().chain(&spools) {
        report.note(line.clone());
    }
    if let Some(real) = spools.iter().find(|l| !l.contains("printed=false")) {
        return Ok(Some(format!("★★★ a spool started a real job: `{real}`.")));
    }
    let in_order = files.len() == 2
        && files[0].contains(FIRST)
        && files[1].contains(SECOND)
        && files.iter().all(|l| l.contains("outcome=Ok("));
    if !in_order || spools.len() != 2 {
        return Ok(Some(format!(
            "★★★ expected {FIRST} then {SECOND} each sent as one job; got {} file line(s) \
             and {} spool(s): {}.",
            files.len(),
            spools.len(),
            list(&files)
        )));
    }
    match trace.last(DONE).map(|l| l.raw.clone()) {
        Some(done) if done.contains("files=2") => {
            report.note(done);
        }
        other => {
            return Ok(Some(format!(
                "★★★ both files spooled but the batch did not finish: `{other:?}`."
            )));
        }
    }
    pointer.gone(&session)?;
    Ok(None)
}
