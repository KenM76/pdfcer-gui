//! `document_properties_say_which_page_boxes_were_not_used_as_written` — on a
//! file whose page 2 crop box runs past the sheet and whose page 3 trim box
//! misses it, Document properties says both, naming the box and the page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/page_boxes.md`.

use crate::checks::driving::declared;
use crate::checks::security_notes::{await_line, launch_on};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "page-boxes.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/page-boxes.PROVENANCE.py`.";
const EVENT: &str = "page-boxes"; // ui-text-exempt: a trace event name, never displayed
const UI_RECT: &str = "ui-rect";
const DOCK: &str = "dock.right.frame";
/// `panels::docprops::boxes::REGION`.
const REGION: &str = "docprops.page-boxes"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct DocumentPropertiesSayWhichPageBoxesWereNotUsedAsWritten;

impl Check for DocumentPropertiesSayWhichPageBoxesWereNotUsedAsWritten {
    fn name(&self) -> &'static str {
        "document_properties_say_which_page_boxes_were_not_used_as_written"
    }

    fn defect(&self) -> &'static str {
        "a crop or trim box written past or off the sheet is quietly cut or replaced, so the \
         page shown and printed is not the one the file states, with nothing said"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_on(ctx, &mut report, FIXTURE, METHOD, "page_boxes")
            .and_then(|session| steps(&mut report, &session));
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn steps(report: &mut CheckReport, session: &Session) -> Result<Option<String>> {
    let Some(line) = await_line(session, EVENT)? else {
        return Ok(Some(format!(
            "Document properties was opened on {FIXTURE} and no `{EVENT}` line followed: the \
             page boxes were never read."
        )));
    };
    report.note(format!("boxes: `{}`", line.raw));
    // Page 1 is the control: every box defaulted, so it is named nowhere.
    let owed = [
        ("pages", Some("3")),
        ("crop_clipped", Some("2")),
        ("trim_unusable", Some("3")),
        ("crop_unusable", None),
        ("trim_clipped", None),
        ("bleed_clipped", None),
        ("bleed_unusable", None),
        ("art_clipped", None),
        ("art_unusable", None),
    ];
    if owed.iter().any(|(k, v)| line.get(k) != *v) {
        return Ok(Some(format!(
            "{FIXTURE}'s page 2 crop box runs past the sheet and its page 3 trim box misses it; \
             the panel said `{}`.",
            line.raw
        )));
    }
    let trace = session.trace()?;
    let Some(drawn) = declared(&trace, UI_RECT, REGION) else {
        return Ok(Some(format!(
            "the boxes were read and no visible `{REGION}` region was published: the operator \
             is never told."
        )));
    };
    let Some(dock) = declared(&trace, UI_RECT, DOCK) else {
        return Ok(Some(format!(
            "no `{DOCK}` region to measure the lines against."
        )));
    };
    if drawn.max.x > dock.max.x + 0.5 {
        return Ok(Some(format!(
            "the page-box lines run past the dock's right edge ({:.1} > {:.1}), so their end is \
             cut off.",
            drawn.max.x, dock.max.x
        )));
    }
    Ok(None)
}
