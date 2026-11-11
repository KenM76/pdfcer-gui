//! `print_draws_sharp_edges_at_print_resolution` — printed sheets are drawn
//! with hard edges at 300 DPI and finer unless the operator asks for smoothing,
//! and the Edge smoothing choice reaches the sheets and is remembered. Whether
//! hard edges leave no seam is the engine's own test.
//!
//! Driven off the desktop with the scripted pointer, `file.print` rung on
//! `transparency-cmyk.pdf` (a page the vector route declines, so its sheet is
//! a picture) and `PDFCER_DIAG_PRINT_DRY_RUN=1` so no press reaches a device.
//! Five launches on one preferences file, each pressing Print and reading the
//! `print-edges` line the sheet render traces:
//!
//! 1. untouched: `choice=auto`, and `smoothed` is true exactly when `dpi` is
//!    below 300.
//! 2. Smooth picked from the list: `choice=on smoothed=true`. Smooth rather
//!    than Sharp, because at 300 DPI Sharp and Automatic trace alike.
//! 3. relaunched, untouched: `choice=on smoothed=true`, so it was remembered.
//! 4. Smooth, no seams picked: `choice=seamless smoothed=true seamless=true`.
//! 5. Automatic picked back: `choice=auto`, leaving the preference at its
//!    default for the checks that follow.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "transparency-cmyk.pdf";
const METHOD: &str = "Rebuild it from its PROVENANCE file in `fixtures/`.";
const OPEN: &str = "print-open";
const EDGES: &str = "print-edges";
const LIST: &str = "print.edges";
const PRINT: &str = "dialog.buttons.accept";
/// `EDGE_CHOICES` order: Automatic, Smooth, Sharp, Smooth no seams.
const AUTO_ITEM: usize = 0;
const SMOOTH_ITEM: usize = 1;
const SEAMLESS_ITEM: usize = 3;

/// See the module documentation.
pub struct PrintDrawsSharpEdgesAtPrintResolution;

impl Check for PrintDrawsSharpEdgesAtPrintResolution {
    fn name(&self) -> &'static str {
        "print_draws_sharp_edges_at_print_resolution"
    }

    fn defect(&self) -> &'static str {
        "printed sheets keep smoothed edges at print resolution, which leaves faint seams where \
         shapes touch, or the Edge smoothing choice does not reach the sheets or is not remembered"
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
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("print_edges.{round}.trace.txt")));
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
        ctx.out(&format!("print_edges.{round}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// What one Print press traced.
struct Edges {
    choice: String,
    dpi: u32,
    smoothed: bool,
    seamless: bool,
    raw: String,
}

/// One launch: pick `item` from the list when given, press Print, and return
/// the `print-edges` line.
fn round(
    ctx: &CheckContext,
    report: &mut CheckReport,
    n: usize,
    item: Option<usize>,
) -> Result<Edges> {
    let (session, pointer) = launch(ctx, report, n)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    if let Some(item) = item {
        press_region(&session, &pointer, LIST, "print.")?;
        press_region(&session, &pointer, &format!("{LIST}.item.{item}"), LIST)?;
    }
    press_region(&session, &pointer, PRINT, "dialog.")?;
    session.settle(240);
    let trace = session.trace()?;
    let line = trace.last(EDGES).ok_or_else(|| {
        Error::new(format!(
            "Print was pressed and no `{EDGES}` line followed: the sheets were never rendered."
        ))
    })?;
    let edges = Edges {
        choice: line.get("choice").unwrap_or_default().to_owned(),
        dpi: line
            .get_usize("dpi")
            .and_then(|d| u32::try_from(d).ok())
            .unwrap_or(0),
        smoothed: line.get("smoothed") == Some("true"),
        seamless: line.get("seamless") == Some("true"),
        raw: line.raw.clone(),
    };
    report.note(edges.raw.clone());
    pointer.gone(&session)?;
    Ok(edges)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let first = round(ctx, report, 1, None)?;
    if first.choice != "auto" || first.smoothed != (first.dpi < 300) {
        return Ok(Some(format!(
            "★★★ an untouched Print must choose by resolution, smoothing only below 300 DPI: \
             `{}`.",
            first.raw
        )));
    }
    let smooth = round(ctx, report, 2, Some(SMOOTH_ITEM))?;
    if smooth.choice != "on" || !smooth.smoothed {
        return Ok(Some(format!(
            "★★★ Smooth was picked and the sheets were not smoothed: `{}`.",
            smooth.raw
        )));
    }
    let relaunched = round(ctx, report, 3, None)?;
    if relaunched.choice != "on" || !relaunched.smoothed {
        return Ok(Some(format!(
            "★★★ the relaunched dialog did not open on the Smooth it was left on: `{}`.",
            relaunched.raw
        )));
    }
    let seamless = round(ctx, report, 4, Some(SEAMLESS_ITEM))?;
    if seamless.choice != "seamless" || !seamless.smoothed || !seamless.seamless {
        return Ok(Some(format!(
            "★★★ Smooth, no seams was picked and the sheets were not drawn seamless: `{}`.",
            seamless.raw
        )));
    }
    let restored = round(ctx, report, 5, Some(AUTO_ITEM))?;
    if restored.choice != "auto" || restored.smoothed != (restored.dpi < 300) || restored.seamless {
        return Ok(Some(format!(
            "★★★ Automatic was picked back and the sheets did not use it: `{}`.",
            restored.raw
        )));
    }
    Ok(None)
}
