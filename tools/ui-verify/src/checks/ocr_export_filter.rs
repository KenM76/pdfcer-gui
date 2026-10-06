//! `recognised_text_choice_filters_word` and `recognised_text_choice_filters_text`
//! — on `fixtures/ocr-layers.pdf`, whose two pages each carry visible text
//! and an OCR layer pdfcer wrote (and page 2 a look-alike another tool
//! wrote), the export window offers the recognised-text choice, and *Leave
//! the recognised text out* and *Only the recognised text* each write the
//! side they name. Run with the scripted pointer in a window placed off the
//! desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_export_filter.md`.

use crate::checks::ocr_export_scripted::{Format, exported_text};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const FIXTURE: &str = "ocr-layers.pdf";
/// Text drawn visibly on the pages, plus the look-alike layer's word, which
/// is not pdfcer's and so counts as ordinary page text.
const PAGE_TEXT: [&str; 3] = ["Visible page one", "Visible page two", "not ours"];
/// The words only pdfcer's two OCR layers spell.
const LAYER_TEXT: [&str; 2] = ["recognised one", "recognised two"];

/// See the module documentation.
pub struct RecognisedChoiceFilters {
    pub format: Format,
}

impl Check for RecognisedChoiceFilters {
    fn name(&self) -> &'static str {
        match self.format {
            Format::Word => "recognised_text_choice_filters_word",
            Format::Text => "recognised_text_choice_filters_text",
        }
    }

    fn defect(&self) -> &'static str {
        "the export window's recognised-text choice does not decide what is written: the \
         exported file keeps the OCR layer's words when told to leave them out, or the page's \
         own text when told to keep only the recognised text"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, self.format) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport, format: Format) -> Result<Option<String>> {
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    for (choice, kept, dropped) in [
        ("without", &PAGE_TEXT[..], &LAYER_TEXT[..]),
        ("only", &LAYER_TEXT[..], &PAGE_TEXT[..]),
    ] {
        let region = format!("{}.{choice}", format.recognised());
        let stem = format!("recognised-{choice}-{}", format.extension());
        let (text, trace) = match exported_text(ctx, report, &pdf, &stem, format, Some(&region))? {
            Ok(done) => done,
            Err(failure) => return Ok(Some(failure)),
        };
        let pages = trace
            .last(format.opened())
            .and_then(|l| l.get("ocr_layer_pages").map(str::to_owned));
        if pages.as_deref() != Some("2") {
            return Ok(Some(format!(
                "the window counted `ocr_layer_pages={pages:?}` on a fixture with pdfcer layers \
                 on 2 pages."
            )));
        }
        let missing: Vec<&str> = kept.iter().copied().filter(|w| !text.contains(w)).collect();
        let leaked: Vec<&str> = dropped
            .iter()
            .copied()
            .filter(|w| text.contains(w))
            .collect();
        report.note(format!(
            "★ `{choice}`: missing {missing:?}, present though excluded {leaked:?}"
        ));
        if !missing.is_empty() || !leaked.is_empty() {
            return Ok(Some(format!(
                "★ with `{region}` chosen the export lacks {missing:?} and holds {leaked:?}: the \
                 choice did not decide what was written."
            )));
        }
    }
    Ok(None)
}
