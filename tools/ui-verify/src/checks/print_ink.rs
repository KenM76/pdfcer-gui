//! `print_composites_an_oversized_sheet_in_ink` — a printed sheet whose ink
//! raster is past the engine's buffer ceiling is composited in ink, in row
//! bands, and never falls back to sRGB, which prints overprint and blend
//! modes in the wrong colours.
//!
//! Driven off the desktop with the scripted pointer, `file.print` rung on
//! `transparency-cmyk-a1.pdf` (an A1 page with a subtractive transparency
//! group) and `PDFCER_DIAG_PRINT_DRY_RUN=1`, so no press reaches a device. The
//! printer's largest sheet is picked (`print-features largest_form`), because
//! a Letter sheet at 300 DPI fits the buffer in one pass and an A3 one does
//! not; a hand-picked sheet is not remembered. Print is pressed and the last
//! `print-ink` line the sheet render traces is read:
//!
//! | `engaged` | `refused` | `bands` | verdict |
//! |---|---|---|---|
//! | false | any | any | SKIP: the sheet never composited in ink |
//! | true | > 0 | any | FAIL: the sheet fell back to sRGB |
//! | true | 0 | 0 | SKIP: the sheet fit in one pass at that resolution |
//! | true | 0 | > 1 | PASS |

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "transparency-cmyk-a1.pdf";
const METHOD: &str = "Rebuild it from its PROVENANCE file in `fixtures/`.";
const OPEN: &str = "print-open";
const EDGES: &str = "print-edges";
const INK: &str = "print-ink";
const FEATURES: &str = "print-features";
const PAPER: &str = "print.paper";
const PLAN: &str = "print-plan";
const UI_RECT: &str = "ui-rect";
const PRINT: &str = "dialog.buttons.accept";

/// See the module documentation.
pub struct PrintCompositesAnOversizedSheetInInk;

impl Check for PrintCompositesAnOversizedSheetInInk {
    fn name(&self) -> &'static str {
        "print_composites_an_oversized_sheet_in_ink"
    }

    fn defect(&self) -> &'static str {
        "a printed sheet too large for the ink buffer falls back to sRGB compositing, so \
         overprint and blend modes print in the wrong colours"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Verdict::Pass) => report.pass(),
            Ok(Verdict::Fail(why)) => report.fail(why),
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_ink.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_ink.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Pick paper list item `item`, scrolling the list to its end first because it
/// is longer than the dialog, and confirm the planned sheet changed. A
/// hand-picked sheet is not remembered, so this leaves no preference behind.
fn pick_sheet(session: &Session, pointer: &ScriptedPointer, item: usize) -> Result<()> {
    let sheet = |s: &Session| -> Result<String> {
        Ok(s.trace()?
            .last(PLAN)
            .and_then(|l| l.get("sheet").map(str::to_owned))
            .unwrap_or_default())
    };
    let before = sheet(session)?;
    press_region(session, pointer, PAPER, "print.")?;
    let first = format!("{PAPER}.item.1");
    let (rect, viewport) = declared_in(&session.trace()?, UI_RECT, &first)
        .ok_or_else(|| Error::new(format!("the paper list opened without `{first}`.")))?;
    pointer.wheel_in(
        session,
        viewport.as_deref(),
        WindowPoint::centre_of(rect),
        -40.0,
    )?;
    session.settle(30);
    press_region(session, pointer, &format!("{PAPER}.item.{item}"), PAPER)?;
    session.settle(30);
    let after = sheet(session)?;
    if after == before {
        return Err(Error::new(format!(
            "paper item {item} was pressed and the planned sheet stayed `{before}`: the pick \
             never landed."
        )));
    }
    Ok(())
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Verdict> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    let largest = session
        .trace()?
        .last(FEATURES)
        .and_then(|l| l.get_usize("largest_form"))
        .unwrap_or(0);
    if largest == 0 {
        return Ok(Verdict::Skip(format!(
            "the printer listed no paper (`{FEATURES}` has no `largest_form`), so no sheet \
             could be made large enough."
        )));
    }
    pick_sheet(&session, &pointer, largest)?;
    press_region(&session, &pointer, PRINT, "dialog.")?;
    session.settle(600);
    let trace = session.trace()?;
    let dpi = trace
        .last(EDGES)
        .map(|l| l.raw.clone())
        .ok_or_else(|| Error::new(format!("Print was pressed and no `{EDGES}` line followed.")))?;
    report.note(dpi.clone());
    let Some(ink) = trace.last(INK) else {
        return Err(Error::new(format!(
            "`{dpi}` was traced and no `{INK}` line followed: no sheet was rendered."
        )));
    };
    report.note(ink.raw.clone());
    pointer.gone(&session)?;
    let refused = ink.get_usize("refused").unwrap_or(0);
    let bands = ink.get_usize("bands").unwrap_or(0);
    Ok(if ink.get("engaged") != Some("true") {
        Verdict::Skip(format!(
            "the sheet never composited in ink (`{}`): a fact about the fixture.",
            ink.raw
        ))
    } else if refused > 0 {
        Verdict::Fail(format!(
            "★★★ the sheet fell back to sRGB compositing: `{}` after `{dpi}`.",
            ink.raw
        ))
    } else if bands == 0 {
        Verdict::Skip(format!(
            "the sheet fit the ink buffer in one pass (`{}` after `{dpi}`), so there was \
             no banding to measure.",
            ink.raw
        ))
    } else {
        Verdict::Pass
    })
}
