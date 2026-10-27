//! `editable_ocr_takes_the_words_out_of_the_scan` — File ▸ Recognise text…
//! with *Editable text and images* writes visible text and rewrites the scan,
//! the saved copy keeps it, and a second run on that page is refused with the
//! engine's `EditableLayerPresent`. Driven with the scripted pointer off the
//! desktop on a copy of `fixtures/synthetic-image-only.pdf`, with `ocrs`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_editable.md`.

use crate::checks::ocr_scripted::{SAVED, recognise_choosing};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const ENGINE: &str = "ocrs";
const EDITABLE: &str = "ocr-output-editable"; // ui-text-exempt: a trace region name, never displayed
const SKIP: &str = "ocr-skip"; // ui-text-exempt: a trace region name, never displayed
const STARTED: &str = "ocr-started"; // ui-text-exempt: a trace event name, never displayed
const OUTPUT: &str = "ocr-layer-output"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "ocr-layer-refused"; // ui-text-exempt: a trace event name, never displayed
/// The engine's `EditableLayerPresent` wording, kept verbatim on the trace.
const EDITABLE_PRESENT: &str = "carries editable OCR text"; // ui-text-exempt: engine error words

/// See the module documentation.
pub struct EditableOcrTakesTheWordsOutOfTheScan;

impl Check for EditableOcrTakesTheWordsOutOfTheScan {
    fn name(&self) -> &'static str {
        "editable_ocr_takes_the_words_out_of_the_scan"
    }

    fn defect(&self) -> &'static str {
        "Recognise text's Editable text and images writes invisible text over the untouched \
         scan, or a second run on that page is not refused"
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
    let fixture = crate::fixture::workspace_root()
        .join("fixtures")
        .join("synthetic-image-only.pdf");
    let copy = ctx.out("ocr-editable.pdf");
    std::fs::copy(&fixture, &copy).map_err(|e| Error::new(format!("copying the fixture: {e}")))?;
    let (trace, path) =
        match recognise_choosing(ctx, report, &copy, ENGINE, "editable", true, &[EDITABLE]) {
            Ok(run) => run,
            // The dialog drew everything up to Run but not the choice: the
            // feature is absent, which is the defect, not a harness problem.
            Err(e) if e.message().contains(&format!("no `{EDITABLE}` region")) => {
                return Ok(Some(format!(
                    "★★ Recognise text offers no `{EDITABLE}` choice."
                )));
            }
            Err(e) => return Err(e),
        };
    let Some(started) = trace.last(STARTED) else {
        return Ok(Some(format!("Run traced no `{STARTED}`. Trace: {path}.")));
    };
    if started.get("output") != Some("editable") {
        return Ok(Some(format!(
            "★ the choice was clicked and the run asked for output={:?}: `{}`. Trace: {path}.",
            started.get("output"),
            started.raw
        )));
    }
    let Some(output) = trace.last(OUTPUT) else {
        return Ok(Some(format!(
            "★ no `{OUTPUT}` line: the recognition was never written. Refused: {:?}. \
             Trace: {path}.",
            trace.last(REFUSED).map(|l| l.raw.clone())
        )));
    };
    report.note(format!("written: `{}`", output.raw));
    if let Some(why) = judge_output(
        output.get_usize("editable"),
        output.get_usize("visible"),
        output.get_usize("images-rewritten"),
    ) {
        return Ok(Some(format!("{why} `{}`. Trace: {path}.", output.raw)));
    }
    let saved = trace.last(SAVED).map(|l| l.raw.clone());
    if !saved.as_deref().is_some_and(|l| l.contains("outcome=ok")) {
        return Ok(Some(format!("Ctrl+S traced {saved:?}. Trace: {path}.")));
    }
    // The saved copy, reopened; the page now draws text, so the skip box is
    // cleared to reach the engine's own refusal.
    let (again, path) =
        recognise_choosing(ctx, report, &copy, ENGINE, "editable-again", false, &[SKIP])?;
    let Some(refused) = again.last(REFUSED) else {
        return Ok(Some(format!(
            "★★ a second run on the saved editable page was not refused: {:?}. Trace: {path}.",
            again.last(OUTPUT).map(|l| l.raw.clone())
        )));
    };
    report.note(format!("again: `{}`", refused.raw));
    Ok((!refused.raw.contains(EDITABLE_PRESENT)).then(|| {
        format!(
            "★★ the second run was refused for another reason: `{}`. Trace: {path}.",
            refused.raw
        )
    }))
}

/// One editable page, words made visible, at least one image rewritten.
fn judge_output(
    editable: Option<usize>,
    visible: Option<usize>,
    rewritten: Option<usize>,
) -> Option<&'static str> {
    if editable.unwrap_or(0) == 0 {
        return Some("★★ the layer was written as searchable, not editable.");
    }
    if visible.unwrap_or(0) == 0 {
        return Some("★★ no word was made visible.");
    }
    if rewritten.unwrap_or(0) == 0 {
        return Some("★★ the scan was not rewritten: the words are still in the picture.");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::judge_output;

    #[test]
    fn only_an_editable_page_with_visible_words_and_a_new_scan_passes() {
        assert_eq!(judge_output(Some(1), Some(5), Some(1)), None);
        assert!(judge_output(Some(0), Some(5), Some(1)).is_some());
        assert!(judge_output(Some(1), Some(0), Some(1)).is_some());
        assert!(judge_output(Some(1), Some(5), Some(0)).is_some());
        assert!(judge_output(None, None, None).is_some());
    }
}
