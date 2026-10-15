//! `enlarge_scans_doubles_every_pages_picture_as_one_undo_step` — File ▸
//! Enlarge scans, at its defaults (2×, bicubic), gives every page's scanned
//! picture twice the samples on each side, pages with text included, and
//! folds the run into one undo entry. Driven with the scripted pointer in a
//! window placed off the desktop, on a copy of `fixtures/skewed-scans.pdf`.
//!
//! The oracle is the engine's answer in the trace: each `deskew-page` line
//! carries the picture's sizes before and after as `upscale_image` returned
//! them, so a run that enlarged nothing, or by the wrong factor, or skipped
//! the page with text as Straighten does, reads wrong.

use crate::checks::driving::declared;
use crate::checks::straighten_scans::{await_finished, click, launch};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const UI_RECT: &str = "ui-rect";
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.enlarge_scans"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const COMMIT: &str = "deskew.commit"; // ui-text-exempt: a trace region name, never displayed
const PAGE: &str = "deskew-page"; // ui-text-exempt: a trace event name, never displayed
const STARTED: &str = "deskew-started"; // ui-text-exempt: a trace event name, never displayed

/// The fixture's page count.
const PAGES: usize = 4;

/// See the module documentation.
pub struct EnlargeScansDoublesEveryPicture;

impl Check for EnlargeScansDoublesEveryPicture {
    fn name(&self) -> &'static str {
        "enlarge_scans_doubles_every_pages_picture_as_one_undo_step"
    }

    fn defect(&self) -> &'static str {
        "Enlarge scans misses a page, enlarges by the wrong factor, or leaves the run as one \
         undo step per page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch(ctx, &mut report, "enlarge-scans").and_then(|(session, pointer)| {
            let outcome = drive(&mut report, &session, &pointer);
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

/// Open the window, press Enlarge with its defaults, read the run.
fn drive(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click(session, pointer, TAB)?;
    let trace = session.trace()?;
    if declared(&trace, UI_RECT, ITEM).is_none() && declared(&trace, UI_RECT, COLLAPSED).is_some() {
        click(session, pointer, COLLAPSED)?;
        session.settle(5);
    }
    // An absent item is the defect, not a harness error: clicking a region
    // never drawn would end the check SKIPPED on the build that lacks it.
    if declared(&session.trace()?, UI_RECT, ITEM).is_none() {
        return Ok(Some(format!(
            "File ▸ Recognise shows no Enlarge scans item (`{ITEM}` was never drawn)."
        )));
    }
    click(session, pointer, ITEM)?;
    session.settle(20);
    click(session, pointer, COMMIT)?;
    let Some(finished) = await_finished(session)? else {
        return Ok(Some(format!(
            "Enlarge was pressed and the run never finished. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    if let Some(started) = trace.events(STARTED).last() {
        report.note(format!("started: `{}`", started.raw));
    }
    report.note(format!("finished: `{}`", finished.raw));
    let pages: Vec<TraceLine> = trace.events(PAGE).cloned().collect();
    for line in &pages {
        report.note(format!("page: `{}`", line.raw));
    }
    if let Some(wrong) = judge(&pages) {
        return Ok(Some(wrong));
    }
    let n = PAGES.to_string();
    if finished.get("corrected") != Some(n.as_str()) || finished.get("folded") != Some("one") {
        return Ok(Some(format!(
            "the run ended `{}`: {PAGES} pages were enlarged and they should be one undo \
             entry (`corrected={PAGES} folded=one`).",
            finished.raw
        )));
    }
    Ok(None)
}

/// Every page enlarged, each side doubled; the first disagreement, worded.
fn judge(pages: &[TraceLine]) -> Option<String> {
    if pages.len() != PAGES {
        return Some(format!(
            "the run logged {} page(s), not the fixture's {PAGES}.",
            pages.len()
        ));
    }
    for (index, line) in pages.iter().enumerate() {
        if line.get("outcome") != Some("enlarged") {
            return Some(format!(
                "page {} should be `enlarged` and the run logged `{}`.",
                index + 1,
                line.raw
            ));
        }
        let size = |key| {
            line.get(key).and_then(|v| {
                let (w, h) = v.split_once('x')?;
                Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))
            })
        };
        match (size("from"), size("to")) {
            (Some(from), Some(to)) if doubled(from, to) => {}
            _ => {
                return Some(format!(
                    "page {} was not enlarged 2× on each side: `{}`.",
                    index + 1,
                    line.raw
                ));
            }
        }
    }
    None
}

/// `to` is `from` doubled, each side, within the engine's rounding.
fn doubled(from: (u32, u32), to: (u32, u32)) -> bool {
    from.0 > 0 && from.1 > 0 && to.0.abs_diff(2 * from.0) <= 1 && to.1.abs_diff(2 * from.1) <= 1
}

#[cfg(test)]
mod tests {
    use super::doubled;

    #[test]
    fn only_a_doubling_passes() {
        assert!(doubled((100, 50), (200, 100)));
        assert!(!doubled((100, 50), (100, 50)));
        assert!(!doubled((100, 50), (300, 150)));
        assert!(!doubled((0, 0), (0, 0)));
    }
}
