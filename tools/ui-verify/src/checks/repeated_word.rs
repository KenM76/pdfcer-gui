//! `ctrl_b_bolds_the_second_copy_of_a_repeated_word` — **a restyle of a word
//! that occurs twice in one piece of text restyles the copy the caret is in**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/repeated_word.md`.

use crate::checks::word_styles::launch_on;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const FIXTURE: &str = "repeated-word.pdf";
/// Inside the second `M10`, which starts at x = 108.
const IN_SECOND: (f64, f64) = (118.0, 703.0);
const FORMAT_TAB: &str = "ribbon.tab.format"; // ui-text-exempt: a trace region name, never displayed
const SPAN_APPLIED: &str = "text-span-style-applied"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "text-style-declined"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
/// The first copy, still in the operator it started in: only a restyle of the
/// second copy leaves it.
const FIRST_KEPT: &str = "(M10 x ";
/// What a restyle of the first copy leaves behind.
const FIRST_TAKEN: &str = "( x M10)";

/// See the module documentation.
pub struct CtrlBBoldsTheSecondCopyOfARepeatedWord;

impl Check for CtrlBBoldsTheSecondCopyOfARepeatedWord {
    fn name(&self) -> &'static str {
        "ctrl_b_bolds_the_second_copy_of_a_repeated_word"
    }

    fn defect(&self) -> &'static str {
        "a restyle of the second copy of a word repeated in one piece of text was refused, or \
         restyled the first copy"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, doc) = launch_on(ctx, report, "repeated-word", FIXTURE)?;
    let source_len = std::fs::metadata(&doc).map_or(0, |m| m.len());
    if let Err(why) = d.caret_at(IN_SECOND)? {
        return Ok(Some(why));
    }
    d.click(FORMAT_TAB)?;
    d.pointer.key(&d.session, None, "B", Some("ctrl"))?;
    d.session.settle(40);
    let trace = d.session.trace()?;
    let declined = trace.events(DECLINED).last().map(|l| l.raw.clone());
    let applied = trace.events(SPAN_APPLIED).last().map(|l| l.raw.clone());
    report.note(format!("applied: {applied:?}; declined: {declined:?}"));
    if declined.is_some() || !applied.as_deref().is_some_and(|l| l.contains("applied=1")) {
        d.pointer.gone(&d.session)?;
        return Ok(Some(format!(
            "Ctrl+B in the second `M10` did not restyle one piece: applied {applied:?}, \
             declined {declined:?}. Trace: {}.",
            d.path()
        )));
    }
    d.pointer.key(&d.session, None, "S", Some("ctrl"))?;
    d.session.settle(40);
    let saved = d
        .session
        .trace()?
        .events(SAVED)
        .last()
        .map(|l| l.raw.clone());
    d.pointer.gone(&d.session)?;
    if !saved.as_deref().is_some_and(|l| l.contains("outcome=ok")) {
        return Ok(Some(format!(
            "Ctrl+S did not save the restyle: {saved:?}. Trace: {}.",
            d.path()
        )));
    }
    let bytes =
        std::fs::read(&doc).map_err(|e| Error::new(format!("reading the saved copy: {e}")))?;
    let start = usize::try_from(source_len).unwrap_or(0).min(bytes.len());
    let tail = &bytes[start..];
    let has = |needle: &str| tail.windows(needle.len()).any(|w| w == needle.as_bytes());
    report.note(format!(
        "saved revision: {} bytes; `{FIRST_KEPT}` {}, `{FIRST_TAKEN}` {}",
        tail.len(),
        has(FIRST_KEPT),
        has(FIRST_TAKEN)
    ));
    if !has(FIRST_KEPT) || has(FIRST_TAKEN) {
        return Ok(Some(format!(
            "the saved revision restyled the wrong copy: a string starting `{FIRST_KEPT}` {}, \
             the remainder `{FIRST_TAKEN}` {}. Trace: {}.",
            has(FIRST_KEPT),
            has(FIRST_TAKEN),
            d.path()
        )));
    }
    Ok(None)
}
