//! `crop_window_sets_the_trim_box` — Pages ▸ Crop… with Trim chosen writes a
//! trim box, not a crop.
//!
//! Driven off the desktop with the scripted pointer: `pages.crop` rung on
//! `four-pages-unrotated.pdf`, the Trim radio pressed, 10 typed into the left
//! margin, the apply button pressed. Oracles:
//!
//! - `page-crop-box box=trim` after the radio: the choice reseeded the margins.
//!   The fixture has no trim box, so they come from its crop box, which is the
//!   sheet: all four zero.
//! - the picture and the Remove button are declared, and Remove white margins
//!   is not: it crops, so it belongs to the crop box alone.
//! - `page-crop-commit ... box=trim n=1 set=1`, then the engine's
//!   `page-crop-applied ... box=TrimBox explicit=1`: one page's own `/TrimBox`
//!   was written. A window that ignored the choice commits `box=crop`, and the
//!   engine reports `CropBox`.

use crate::checks::driving::{SHELL_DIAG_ENV, list, live_names, repo_fixture};
use crate::checks::page_crop_even::type_left;
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "four-pages-unrotated.pdf";
const METHOD: &str = "Rebuild it from its PROVENANCE file in `fixtures/`.";
const UI_RECT: &str = "ui-rect";
const OPENED: &str = "page-crop-opened";
const CHOSEN: &str = "page-crop-box";
const COMMIT: &str = "page-crop-commit";
const APPLIED: &str = "page-crop-applied";
const TRIM: &str = "page-crop.box.trim";
const APPLY: &str = "page-crop.apply";
const PRESENT: [&str; 2] = ["page-crop.picture", "page-crop.remove"];
const ABSENT: &str = "page-crop.white";

/// See the module documentation.
pub struct CropWindowSetsTheTrimBox;

impl Check for CropWindowSetsTheTrimBox {
    fn name(&self) -> &'static str {
        "crop_window_sets_the_trim_box"
    }

    fn defect(&self) -> &'static str {
        "Pages > Crop with Trim chosen crops the page, or writes no trim box"
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("page_crop_boxes.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("page_crop_boxes.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Choose Trim; the defect it found, if any.
fn choose_trim(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let before = session.trace()?;
    // The control: the absence measured below is of a region this trace can see.
    if !live_names(&before, UI_RECT, "page-crop.")
        .iter()
        .any(|n| n == ABSENT)
    {
        return Err(Error::new(format!(
            "`{ABSENT}` is not on screen with Crop chosen, so its absence under Trim \
             would prove nothing. On screen: {}.",
            list(&live_names(&before, UI_RECT, "page-crop."))
        )));
    }
    let mark = before.mark();
    press_region(session, pointer, TRIM, "page-crop.box.")?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(chosen) = trace.last_after(CHOSEN, mark) else {
        return Ok(Some(format!(
            "★★★ Trim was pressed and no `{CHOSEN}` line followed."
        )));
    };
    report.note(chosen.raw.clone());
    if chosen.get("box") != Some("trim") || chosen.get("margins_mm") != Some("0,0,0,0") {
        return Ok(Some(format!(
            "★★★ choosing Trim on a page with no trim box should seed it from the sheet, \
             box=trim margins_mm=0,0,0,0: `{}`.",
            chosen.raw
        )));
    }
    let live = live_names(&trace, UI_RECT, "page-crop.");
    for name in PRESENT {
        if !live.iter().any(|n| n == name) {
            return Ok(Some(format!(
                "★★★ with Trim chosen the window draws no `{name}`. On screen: {}.",
                list(&live)
            )));
        }
    }
    if live.iter().any(|n| n == ABSENT) {
        return Ok(Some(format!(
            "★★★ with Trim chosen the window still offers `{ABSENT}`, which crops."
        )));
    }
    Ok(None)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPENED).is_none() {
        return Err(Error::new(format!(
            "`pages.crop` was rung and no `{OPENED}` line followed."
        )));
    }
    if let Some(failure) = choose_trim(&session, &pointer, report)? {
        return Ok(Some(failure));
    }
    type_left(&session, &pointer)?;
    press_region(&session, &pointer, APPLY, "page-crop.")?;
    session.settle(120);
    let trace = session.trace()?;
    let Some(commit) = trace.last(COMMIT) else {
        return Ok(Some(format!(
            "the apply button was pressed and no `{COMMIT}` line followed."
        )));
    };
    report.note(commit.raw.clone());
    let got = (commit.get("box"), commit.get("n"), commit.get("set"));
    if got != (Some("trim"), Some("1"), Some("1")) {
        return Ok(Some(format!(
            "★★★ Trim with a 10 mm left margin should commit box=trim n=1 set=1: `{}`.",
            commit.raw
        )));
    }
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "the commit was raised and no `{APPLIED}` line followed."
        )));
    };
    report.note(applied.raw.clone());
    pointer.gone(&session)?;
    if applied.get("box") != Some("TrimBox") || applied.get("explicit") != Some("1") {
        return Ok(Some(format!(
            "★★★ the engine did not write one page's own /TrimBox: `{}`.",
            applied.raw
        )));
    }
    Ok(None)
}
