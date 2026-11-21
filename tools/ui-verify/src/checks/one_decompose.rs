//! `an_edit_reads_the_page_from_the_session` — after an arrow key moves an
//! object, the page model the canvas picks from is the edit session's own,
//! not a second decomposition of the same page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/one_decompose.md`.

use crate::checks::word_styles::{Driven, launch_invoking};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIXTURE: &str = "clear-image-over-ink.pdf";
const INVOKE: &str = "mode.edit";
const SET: &str = "selection-set"; // ui-text-exempt: a trace event name, never displayed
const MOVED: &str = "canvas-move"; // ui-text-exempt: a trace event name, never displayed
const BUILT: &str = "page-objects-built"; // ui-text-exempt: a trace event name, never displayed
/// The black square, seen through the image's clear half.
const SQUARE: (f64, f64) = (260.0, 150.0);

/// See the module documentation.
pub struct AnEditReadsThePageFromTheSession;

impl Check for AnEditReadsThePageFromTheSession {
    fn name(&self) -> &'static str {
        "an_edit_reads_the_page_from_the_session"
    }

    fn defect(&self) -> &'static str {
        "every edit decomposes the page twice, once in the engine and once in the window, so a \
         dense page holds the next input for twice as long as it needs to"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_invoking(ctx, &mut report, "one-decompose", FIXTURE, INVOKE)
            .and_then(|(d, _)| steps(&mut report, &d));
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn steps(report: &mut CheckReport, d: &Driven) -> Result<Option<String>> {
    let selections = d.session.trace()?.events(SET).count();
    d.click_page(SQUARE)?;
    if d.session.trace()?.events(SET).count() == selections {
        return Ok(Some(format!(
            "a click at {SQUARE:?} selected nothing. Trace: {}.",
            d.path()
        )));
    }
    let (moves, builds) = {
        let trace = d.session.trace()?;
        (trace.events(MOVED).count(), trace.events(BUILT).count())
    };
    d.pointer.key(&d.session, None, "ArrowRight", None)?;
    for _ in 0..30 {
        d.session.settle(10);
        let trace = d.session.trace()?;
        if trace.events(MOVED).count() > moves && trace.events(BUILT).count() > builds {
            break;
        }
    }
    let trace = d.session.trace()?;
    if trace.events(MOVED).count() == moves {
        return Ok(Some(format!(
            "ArrowRight with the square selected moved nothing: no `{MOVED}`. Trace: {}.",
            d.path()
        )));
    }
    let rebuilt: Vec<_> = trace.events(BUILT).skip(builds).collect();
    let Some(first) = rebuilt.first() else {
        return Ok(Some(format!(
            "the page model was never rebuilt after the move: no `{BUILT}`. Trace: {}.",
            d.path()
        )));
    };
    report.note(format!("after the move: `{}`", first.raw));
    if let Some(own) = rebuilt.iter().find(|l| l.get("source") != Some("session")) {
        return Ok(Some(format!(
            "the window decomposed the edited page itself instead of reading the session's \
             model: `{}`.",
            own.raw
        )));
    }
    Ok(None)
}
