//! `a_click_on_a_clear_pixel_picks_the_ink_beneath` — a click where an
//! image's soft mask is fully clear selects the black square showing through
//! it, and a click on the image's opaque half selects the image.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/clear_pixels.md`.

use crate::checks::word_styles::{Driven, launch_invoking};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIXTURE: &str = "clear-image-over-ink.pdf";
const INVOKE: &str = "mode.edit";
const SET: &str = "selection-set"; // ui-text-exempt: a trace event name, never displayed
/// Inside the image's clear right half, over the square (object 0).
const CLEAR: (f64, f64) = (260.0, 150.0);
/// Inside the image's opaque left half (object 1), over nothing else.
const OPAQUE: (f64, f64) = (150.0, 150.0);

/// See the module documentation.
pub struct AClickOnAClearPixelPicksTheInkBeneath;

impl Check for AClickOnAClearPixelPicksTheInkBeneath {
    fn name(&self) -> &'static str {
        "a_click_on_a_clear_pixel_picks_the_ink_beneath"
    }

    fn defect(&self) -> &'static str {
        "a click on ink seen through the transparent part of an image selects the image, so the \
         line or shape beneath cannot be picked"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_invoking(ctx, &mut report, "clear-pixels", FIXTURE, INVOKE)
            .and_then(|(d, _)| steps(&mut report, &d));
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Click at `at` and return the newest `selection-set` line's `object=`.
fn pick(report: &mut CheckReport, d: &Driven, at: (f64, f64)) -> Result<Option<String>> {
    let before = d.session.trace()?.events(SET).count();
    d.click_page(at)?;
    let trace = d.session.trace()?;
    if trace.events(SET).count() == before {
        return Ok(None);
    }
    let line = trace.events(SET).last().map(|l| l.raw.clone());
    report.note(format!(
        "click at {at:?}: `{}`",
        line.as_deref().unwrap_or("")
    ));
    Ok(trace
        .events(SET)
        .last()
        .and_then(|l| l.get("object"))
        .map(str::to_owned))
}

fn steps(report: &mut CheckReport, d: &Driven) -> Result<Option<String>> {
    let clear = pick(report, d, CLEAR)?;
    if clear.as_deref() != Some("0") {
        return Ok(Some(format!(
            "a click at {CLEAR:?}, where the image is fully clear over the black square, \
             selected object {clear:?}, not the square (object 0). Trace: {}.",
            d.path()
        )));
    }
    let opaque = pick(report, d, OPAQUE)?;
    if opaque.as_deref() != Some("1") {
        return Ok(Some(format!(
            "a click at {OPAQUE:?}, on the image's opaque half, selected object {opaque:?}, not \
             the image (object 1): clear-pixel testing made the image unclickable. Trace: {}.",
            d.path()
        )));
    }
    Ok(None)
}
