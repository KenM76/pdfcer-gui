//! `remove_white_margins_crops_to_the_ink` — Pages ▸ Crop… ▸ Remove white
//! margins crops the page to what it draws.
//!
//! Driven off the desktop with the scripted pointer: `pages.crop` rung on
//! `blank-overhang.pdf`, Remove white margins pressed with no paper kept. The
//! measured crop must match the engine CLI's own `set-crop-box
//! --remove-white-margins` on the same file, and the engine must report the
//! page changed. The defect it catches is a button that measures nothing, or
//! a measured crop that never reaches the page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_crop_white.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "blank-overhang.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/blank-overhang.PROVENANCE.py`.";
const OPENED: &str = "page-crop-opened";
const MEASURED: &str = "white-margins-measured";
const APPLIED: &str = "page-crop-applied";
const WHITE: &str = "page-crop.white";
/// `pdfcer set-crop-box --remove-white-margins` on the fixture, margin 0:
/// "content found within [58.50 398.50 501.50 741.50]".
const ORACLE: [f64; 4] = [58.50, 398.50, 501.50, 741.50];
/// One render pixel at 144 dpi.
const TOLERANCE_PT: f64 = 0.5;

/// See the module documentation.
pub struct CropWhiteMargins;

impl Check for CropWhiteMargins {
    fn name(&self) -> &'static str {
        "remove_white_margins_crops_to_the_ink"
    }

    fn defect(&self) -> &'static str {
        "Pages > Crop > Remove white margins crops nothing, or crops to a box other than the \
         page's drawing"
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("page_crop_white.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "pages.crop"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("page_crop_white.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPENED).is_none() {
        return Err(Error::new(format!(
            "`pages.crop` was rung and no `{OPENED}` line followed."
        )));
    }
    press_region(&session, &pointer, WHITE, "page-crop.")?;
    session.settle(120);
    let trace = session.trace()?;
    let Some(measured) = trace.last(MEASURED) else {
        return Ok(Some(format!(
            "★★★ Remove white margins was pressed and no `{MEASURED}` line followed."
        )));
    };
    report.note(measured.raw.clone());
    let got: Vec<f64> = ["llx", "lly", "urx", "ury"]
        .iter()
        .filter_map(|k| measured.get(k).and_then(|v| v.parse().ok()))
        .collect();
    let matches = got.len() == 4
        && got
            .iter()
            .zip(ORACLE)
            .all(|(g, o)| (g - o).abs() <= TOLERANCE_PT);
    if measured.get("cropped") != Some("1") || !matches {
        return Ok(Some(format!(
            "★★★ the page should crop to the engine CLI's {ORACLE:?}: `{}`.",
            measured.raw
        )));
    }
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "★★★ a crop was measured and no `{APPLIED}` line followed, so it never reached the page."
        )));
    };
    report.note(applied.raw.clone());
    let shot = ctx.out("page-crop-white.png");
    pointer.screenshot(&session, &shot)?;
    report.artifact(shot);
    pointer.gone(&session)?;
    if applied.get("n") != Some("1") {
        return Ok(Some(format!(
            "★★★ the engine changed a different number of pages than the one measured: `{}`.",
            applied.raw
        )));
    }
    Ok(None)
}
