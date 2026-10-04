//! `scaling_a_sheet_scales_the_drawing` — choosing "Scale the drawing to fit"
//! in the Sheet size window must land the new sheet AND scale what is drawn on
//! it by the factor the window quoted, through `EditSession::scale_pages`
//! rather than the box-only resize.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_scale.md`.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::page_size::{
    A6_PT, APPLY, FIXTURE, OFFSCREEN, TOLERANCE_PT, a6_or_stale_index, click_dialog_region,
    open_and_census, pick_a6_portrait,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::trace::Trace;

/// The "Scale the drawing to fit" radio.
const FIT: &str = "page-size.drawing.fit";

/// `page-size-opened … extent=llx,lly,urx,ury` — the survey's drawn extent of
/// the operand sheet, read from the open document each time the window opens.
const OPENED_EVENT: &str = "page-size-opened";

/// `page-size-scaled mode=… scale_min=… scale_max=…` — the pre-commit quote.
const QUOTE_EVENT: &str = "page-size-scaled";

/// `page-size-commit … drawing=fit` — the commit button acted.
const COMMIT_EVENT: &str = "page-size-commit";

/// `page-scale-applied n=… mode=… scale_max=…` — the scale verb ran.
const APPLIED_EVENT: &str = "page-scale-applied";

/// `page-size-applied …` — the box-only verb ran; must be absent.
const BOX_ONLY_EVENT: &str = "page-size-applied";

/// `page-size-sheet index=… w=… h=…` — the page tree after the edit.
const SHEET_EVENT: &str = "page-size-sheet";

/// How close a scale factor must be to the expected one.
const FACTOR_TOLERANCE: f64 = 0.001;

/// How close the scaled drawn extent must be, in points.
const EXTENT_TOLERANCE_PT: f64 = 0.5;

/// See the module documentation.
pub struct ScalingASheetScalesTheDrawing;

impl Check for ScalingASheetScalesTheDrawing {
    fn name(&self) -> &'static str {
        "scaling_a_sheet_scales_the_drawing"
    }

    fn defect(&self) -> &'static str {
        "choosing to scale the drawing onto a new sheet size changes only the paper, scales by \
         a different factor than the window quoted, or does nothing"
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

/// The last `page-size-opened` line's drawn extent, as (width, height).
fn extent(trace: &Trace) -> Option<(f64, f64)> {
    let parts: Vec<f64> = trace
        .last(OPENED_EVENT)?
        .get("extent")?
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    match parts[..] {
        [llx, lly, urx, ury] => Some((urx - llx, ury - lly)),
        _ => None,
    }
}

fn number(line: &crate::trace::TraceLine, key: &str) -> Option<f64> {
    line.get(key).and_then(|v| v.parse().ok())
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let pdf = crate::checks::driving::repo_fixture(
        FIXTURE.trim_start_matches("fixtures/"),
        "This check compares a known sheet size and drawn extent before and after the scale.",
    )?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("page_scale.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), "mode.edit".to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    spec.place = false;
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("page_scale.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} on {} as pid {}",
        exe.display(),
        pdf.display(),
        session.pid()
    ));
    session.settle(30);
    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the application published no `{}`, so it did not reach a first frame.",
            ctx.profile.vocab.start_event
        )));
    }
    Ok((session, pointer))
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let driver = &pointer;

    // -- A: baseline sheet and drawn extent ---------------------------------
    let before = open_and_census(&session, driver, ui_rect, report, "the fixture")?;
    let sheet = before
        .get(&0)
        .copied()
        .ok_or_else(|| Error::new("the census resolved no page 0."))?;
    let drawn = extent(&session.trace()?).ok_or_else(|| {
        Error::new(format!(
            "the window published no measurable `extent=` for page 0 of {FIXTURE}, so there is \
             no drawing whose scaling could be judged."
        ))
    })?;
    let expected = (A6_PT.0 / sheet.w).min(A6_PT.1 / sheet.h);
    report.note(format!(
        "page 0 is {:.2} x {:.2} with a drawn extent of {:.2} x {:.2}; fitting it onto A6 \
         portrait should scale it by {expected:.4}",
        sheet.w, sheet.h, drawn.0, drawn.1
    ));

    // -- B: choose A6 portrait and Fit; the window quotes the factor ---------
    pick_a6_portrait(&session, driver, ui_rect)?;
    click_dialog_region(&session, driver, ui_rect, FIT)?;
    session.settle(10);
    let trace = session.trace()?;
    let Some(quote) = trace.last(QUOTE_EVENT) else {
        return Ok(Some(format!(
            "`{FIT}` was clicked and the window published no `{QUOTE_EVENT}`: the radio is drawn \
             and choosing it did not switch the window to a scale."
        )));
    };
    let quoted = number(quote, "scale_min").unwrap_or(f64::NAN);
    if quote.get("mode") != Some("Fit") || (quoted - expected).abs() > FACTOR_TOLERANCE {
        return Ok(Some(format!(
            "the window quoted `mode={} scale_min={quoted:.4}` before the commit; fitting a \
             {:.2} x {:.2} sheet onto A6 portrait is {expected:.4}.",
            quote.get("mode").unwrap_or("?"),
            sheet.w,
            sheet.h
        )));
    }

    // -- C: commit; the scale verb ran and the box-only verb did not ---------
    let mark = trace.mark();
    click_dialog_region(&session, driver, ui_rect, APPLY)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(commit) = trace.last_after(COMMIT_EVENT, mark) else {
        return Ok(Some(format!(
            "Apply was clicked and the window published no `{COMMIT_EVENT}`."
        )));
    };
    a6_or_stale_index(commit.get("size_id").unwrap_or("?"))?;
    if commit.get("drawing") != Some("fit") {
        return Ok(Some(format!(
            "the commit was traced with `drawing={}` after Fit was chosen.",
            commit.get("drawing").unwrap_or("?")
        )));
    }
    if trace.last_after(BOX_ONLY_EVENT, mark).is_some() {
        return Ok(Some(format!(
            "Fit was chosen and the commit ran the box-only resize (`{BOX_ONLY_EVENT}`): the \
             paper changed and the drawing was left where it was."
        )));
    }
    let Some(applied) = trace.last_after(APPLIED_EVENT, mark) else {
        return Ok(Some(format!(
            "the commit published no `{APPLIED_EVENT}`, so `scale_pages` did not run."
        )));
    };
    let factor = number(applied, "scale_max").unwrap_or(f64::NAN);
    if applied.get("n") != Some("1") || (factor - quoted).abs() > FACTOR_TOLERANCE {
        return Ok(Some(format!(
            "the engine scaled n={} sheets by {factor:.4}; the window quoted {quoted:.4} for one.",
            applied.get("n").unwrap_or("?")
        )));
    }
    let landed = trace
        .last_after(SHEET_EVENT, mark)
        .filter(|l| l.get("index") == Some("0"))
        .map(|l| (number(l, "w"), number(l, "h")));
    if !matches!(landed, Some((Some(w), Some(h)))
        if (w - A6_PT.0).abs() < TOLERANCE_PT && (h - A6_PT.1).abs() < TOLERANCE_PT)
    {
        return Ok(Some(format!(
            "after the scale page 0's sheet reads {landed:?}; A6 portrait is {:.2} x {:.2}.",
            A6_PT.0, A6_PT.1
        )));
    }

    // -- D: reopen; the drawing itself is now scaled --------------------------
    let after = open_and_census(&session, driver, ui_rect, report, "the scaled document")?;
    let scaled = extent(&session.trace()?).ok_or_else(|| {
        Error::new("the reopened window published no measurable `extent=` for page 0.")
    })?;
    let want = (drawn.0 * factor, drawn.1 * factor);
    if (scaled.0 - want.0).abs() > EXTENT_TOLERANCE_PT
        || (scaled.1 - want.1).abs() > EXTENT_TOLERANCE_PT
    {
        return Ok(Some(format!(
            "the sheet is now {:?} and the drawn extent reads {:.2} x {:.2}; scaled by \
             {factor:.4} it should be {:.2} x {:.2}. {}",
            after.get(&0),
            scaled.0,
            scaled.1,
            want.0,
            want.1,
            if (scaled.0 - drawn.0).abs() < EXTENT_TOLERANCE_PT {
                "It is unchanged: only the paper moved."
            } else {
                "It changed by some other factor."
            }
        )));
    }
    if after.get(&1) != before.get(&1) {
        return Ok(Some(format!(
            "page 1 was not picked and went from {:?} to {:?}: the scale reached a sheet beside the operand.",
            before.get(&1),
            after.get(&1)
        )));
    }
    report.note(format!(
        "VERDICT: page 0 is A6 and its drawing reads {:.2} x {:.2}, the {:.2} x {:.2} it was \
         scaled by {factor:.4}, the factor the window quoted before the commit",
        scaled.0, scaled.1, drawn.0, drawn.1
    ));
    Ok(None)
}
