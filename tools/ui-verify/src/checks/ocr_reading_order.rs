//! `recognised_text_reads_column_by_column` — on `fixtures/ocr-two-columns.pdf`,
//! a scan of two columns of two paragraphs, File ▸ Recognise text… with
//! `ocrs` writes the layer in lines and blocks laid out column by column, so
//! Export to Text *as drawn* of the saved file reads the four paragraphs in
//! order. Paragraph grouping is not asserted: on OCR word boxes the engine
//! makes every line its own block (G131); the block count is noted. Run with the scripted pointer in a window
//! placed off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_reading_order.md`.

use crate::checks::ocr_export_scripted::{Format, exported_text};
use crate::checks::ocr_scripted::{SAVED, STRUCTURE, recognise};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const FIXTURE: &str = "ocr-two-columns.pdf";
const ENGINE: &str = "ocrs";
/// The word each paragraph opens with, in reading order: two left, two right.
const OPENERS: [&str; 4] = ["Apples", "Bridges", "Candles", "Dolphins"];
/// The export window's *as drawn* order, which writes the content stream's order.
const AS_DRAWN: &str = "export-text.order.as-drawn"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct RecognisedTextReadsColumnByColumn;

impl Check for RecognisedTextReadsColumnByColumn {
    fn name(&self) -> &'static str {
        "recognised_text_reads_column_by_column"
    }

    fn defect(&self) -> &'static str {
        "the OCR layer is written in the order the engine found its words, so a two-column \
         scan's recognised text interleaves the columns"
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
        .join(FIXTURE);
    let copy = ctx.out("ocr-two-columns.pdf");
    std::fs::copy(&fixture, &copy).map_err(|e| Error::new(format!("copying the fixture: {e}")))?;

    let (trace, path) = recognise(ctx, report, &copy, ENGINE, "columns", true)?;
    let Some(structure) = trace.last(STRUCTURE) else {
        return Ok(Some(format!(
            "no `{STRUCTURE}` line: no layer was applied, or its report was not read. Trace: \
             {path}."
        )));
    };
    report.note(format!("structure: `{}`", structure.raw));
    let lines = structure.get_usize("lines").unwrap_or(0);
    let blocks = structure.get_usize("blocks").unwrap_or(0);
    if blocks == 0 || lines < blocks {
        return Ok(Some(format!(
            "★ `{}`: the layer must be written as lines inside blocks. Trace: {path}.",
            structure.raw
        )));
    }
    if blocks == lines {
        report.note("every line is its own block: paragraphs are not grouped (G131)");
    }
    let saved = trace.last(SAVED).map(|l| l.raw.clone());
    if !saved.as_deref().is_some_and(|l| l.contains("outcome=ok")) {
        return Ok(Some(format!("Ctrl+S traced {saved:?}. Trace: {path}.")));
    }

    let (text, _) =
        match exported_text(ctx, report, &copy, "columns", Format::Text, Some(AS_DRAWN))? {
            Ok(done) => done,
            Err(failure) => return Ok(Some(failure)),
        };
    let lower = text.to_lowercase();
    let at: Vec<Option<usize>> = OPENERS
        .iter()
        .map(|w| lower.find(&w.to_lowercase()))
        .collect();
    report.note(format!("openers at {at:?} in {} bytes", text.len()));
    if at.iter().any(Option::is_none) {
        return Ok(Some(format!(
            "the exported text lacks an opener of {OPENERS:?} (positions {at:?}); recognition \
             misread the fixture, so the order cannot be judged."
        )));
    }
    if !at.windows(2).all(|pair| pair[0] < pair[1]) {
        return Ok(Some(format!(
            "★ the layer as drawn reads the paragraphs out of order: {OPENERS:?} at {at:?}. A \
             writer that lays out line by line across the page interleaves the two columns."
        )));
    }
    Ok(None)
}
