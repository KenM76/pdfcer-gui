//! `print_sizes_a_user_unit_page_physically` — the Print window measures a
//! page at its PHYSICAL size, `/MediaBox` times `/UserUnit` (ISO 32000-1
//! §14.11.2), so an oversized drawing is not planned at a fraction of itself.
//!
//! Opens Print (dry run, off the desktop, scripted pointer) on
//! `fixtures/user-unit.pdf` — a 306 x 396 box with `/UserUnit 2`, physically
//! Letter — chooses Match the pages, and reads the last `print-plan` line.
//! `largest=` is the page the paper decision measured, from the same
//! `plan::page_sizes` every plan, preview and render reads: `612x792` is the
//! physical size, `306x396` the box read as points. Ignores `--pdf`; the
//! fixture is the method.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::print_auto_paper::size;
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "user-unit.pdf";
const METHOD: &str = "A page whose /UserUnit makes its box half its physical size; any other file \
                      cannot tell the two measures apart. Rebuild it from its PROVENANCE file.";
const OPEN: &str = "print-open";
const PLAN: &str = "print-plan";
const PAPER: &str = "print.paper";
const PAPER_AUTO: &str = "print.paper.auto";

/// The fixture's physical size, and its box read as points.
const PHYSICAL: (f64, f64) = (612.0, 792.0);
const BOX: (f64, f64) = (306.0, 396.0);
/// The trace prints whole points.
const TOLERANCE_PT: f64 = 1.0;

pub struct PrintSizesAUserUnitPagePhysically;

impl Check for PrintSizesAUserUnitPagePhysically {
    fn name(&self) -> &'static str {
        "print_sizes_a_user_unit_page_physically"
    }

    fn defect(&self) -> &'static str {
        "a page with /UserUnit is planned at its box size in points, so it prints at a fraction of its real size"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_user_unit.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_user_unit.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Within the tolerance, either way round.
fn near(a: (f64, f64), b: (f64, f64)) -> bool {
    let close = |x: f64, y: f64| (x - y).abs() <= TOLERANCE_PT;
    (close(a.0, b.0) && close(a.1, b.1)) || (close(a.0, b.1) && close(a.1, b.0))
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    press_region(&session, &pointer, PAPER, "print.")?;
    press_region(&session, &pointer, PAPER_AUTO, "print.paper.")?;
    session.settle(30);
    let trace = session.trace()?;
    let line = trace
        .last(PLAN)
        .ok_or_else(|| Error::new(format!("no `{PLAN}` line.")))?;
    let (pick, largest) = (
        line.get("pick").unwrap_or("<absent>").to_owned(),
        line.get("largest").unwrap_or("<absent>").to_owned(),
    );
    pointer.gone(&session)?;
    report.note(format!("pick={pick} largest={largest}"));
    if pick != "auto" {
        return Err(Error::new(format!(
            "the press on `{PAPER_AUTO}` left `pick={pick}`, so Match the pages was never chosen \
             and nothing was measured."
        )));
    }
    let Some(measured) = size(&largest) else {
        return Err(Error::new(format!(
            "Match the pages was chosen and `largest={largest}` carries no page measurement."
        )));
    };
    if near(measured, PHYSICAL) {
        return Ok(None);
    }
    if near(measured, BOX) {
        return Ok(Some(format!(
            "the page measured {largest}: its box read as points, ignoring `/UserUnit 2`, so it \
             would print at half its size on each side. `plan::page_sizes` must multiply by \
             `Page::user_unit`."
        )));
    }
    Ok(Some(format!(
        "the page measured {largest}, neither its physical {}x{} nor its box {}x{}.",
        PHYSICAL.0, PHYSICAL.1, BOX.0, BOX.1
    )))
}
