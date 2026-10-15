//! `straighten_scans_honours_its_least_confidence` — the Straighten scans
//! window's least-confidence slider decides which measured tilts are applied,
//! and is remembered. Driven with the scripted pointer off the desktop on a
//! copy of `fixtures/skewed-scans.pdf`: the slider is clicked at its right
//! end (100 %), so the two tilted pages, measured below certainty, must be
//! left alone as unsure; the run's start line must name 100, and the
//! sandbox's preferences file must hold it afterwards.

use crate::checks::driving::declared_in;
use crate::checks::straighten_scans::{await_finished, click, launch, open_dialog};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const UI_RECT: &str = "ui-rect";
const SLIDER: &str = "deskew.min-confidence"; // ui-text-exempt: a trace region name, never displayed
const COMMIT: &str = "deskew.commit"; // ui-text-exempt: a trace region name, never displayed
const STARTED: &str = "deskew-started"; // ui-text-exempt: a trace event name, never displayed
const PAGE: &str = "deskew-page"; // ui-text-exempt: a trace event name, never displayed
const KEY: &str = "deskew_min_confidence";

/// See the module documentation.
pub struct StraightenScansHonoursLeastConfidence;

impl Check for StraightenScansHonoursLeastConfidence {
    fn name(&self) -> &'static str {
        "straighten_scans_honours_its_least_confidence"
    }

    fn defect(&self) -> &'static str {
        "Straighten scans ignores the least confidence set in its window, turns a page \
         measured below it, or forgets the setting"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome =
            launch(ctx, &mut report, "straighten-confidence").and_then(|(session, pointer)| {
                let outcome = drive(ctx, &mut report, &session, &pointer);
                let parked = pointer.gone(&session);
                let outcome = outcome?;
                parked?;
                Ok(outcome)
            });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    open_dialog(session, pointer)?;
    let trace = session.trace()?;
    let (rail, viewport) = declared_in(&trace, UI_RECT, SLIDER)
        .ok_or_else(|| Error::new(format!("the window declared no `{SLIDER}` region.")))?;
    let right_end = LRect {
        min: Pt {
            x: rail.max.x - 2.0,
            y: rail.min.y,
        },
        max: Pt {
            x: rail.max.x - 1.0,
            y: rail.max.y,
        },
    };
    pointer.click_in(
        session,
        viewport.as_deref(),
        WindowPoint::centre_of(right_end),
    )?;
    session.settle(15);
    click(session, pointer, COMMIT)?;
    let Some(finished) = await_finished(session)? else {
        return Ok(Some(
            "Straighten was pressed and the run never finished.".to_owned(),
        ));
    };
    report.note(format!("finished: `{}`", finished.raw));
    let trace = session.trace()?;
    let started = trace.events(STARTED).last().cloned();
    report.note(format!("started: `{:?}`", started.as_ref().map(|l| &l.raw)));
    if started.as_ref().and_then(|l| l.get("min-confidence")) != Some("100") {
        return Ok(Some(format!(
            "the slider was clicked at its right end and the run started with `{:?}`, \
             not `min-confidence=100`.",
            started.map(|l| l.raw)
        )));
    }
    let pages: Vec<TraceLine> = trace.events(PAGE).cloned().collect();
    for line in &pages {
        report.note(format!("page: `{}`", line.raw));
    }
    if let Some(wrong) = judge(&pages) {
        return Ok(Some(wrong));
    }
    let saved = remembered(ctx)?;
    report.note(format!("preferences file: {KEY} = {saved:?}"));
    if saved.as_deref() != Some("100") {
        return Ok(Some(format!(
            "after the run the preferences file holds {KEY} = {saved:?}, not 100."
        )));
    }
    Ok(None)
}

/// Pages 1 and 2 are tilted and measured below 100 %, so both are `unsure`
/// and none is `straightened`.
fn judge(pages: &[TraceLine]) -> Option<String> {
    let outcome = |i: usize| pages.get(i).and_then(|l| l.get("outcome"));
    if outcome(0) != Some("unsure") || outcome(1) != Some("unsure") {
        return Some(format!(
            "at 100 % the two tilted pages should be left alone as `unsure`; the run logged \
             {:?}.",
            pages.iter().map(|l| l.raw.as_str()).collect::<Vec<_>>()
        ));
    }
    pages
        .iter()
        .find(|l| l.get("outcome") == Some("straightened"))
        .map(|l| format!("a page was turned at 100 %: `{}`.", l.raw))
}

/// The key's value in the sandbox's preferences file.
fn remembered(ctx: &CheckContext) -> Result<Option<String>> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary resolved"))?;
    let file = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata")
        .join(crate::sandbox::PREFS_FILE);
    let text = std::fs::read_to_string(&file)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", file.display())))?;
    Ok(text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == KEY).then(|| v.trim().to_owned())
        }))
}
