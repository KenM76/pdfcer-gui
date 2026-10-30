//! `document_properties_say_which_pdfx_level_the_file_claims` — on a file
//! claiming PDF/X-4 in its XMP packet and naming no output intent, Document
//! properties shows the level and says the intent is missing. A file with only
//! a `/GTS_PDFX` output intent is the control: an intent is not a claim, so
//! no PDF/X row is drawn.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/pdfx_claim.md`.

use crate::checks::driving::declared;
use crate::checks::security_notes::{await_line, launch_on};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const CLAIMS: &str = "print-shop-pdfx4.pdf";
const CLAIMS_METHOD: &str = "Rebuild it with `python fixtures/print-shop-pdfx4.PROVENANCE.py`.";
const INTENT_ONLY: &str = "print-shop-pdfx.pdf";
const INTENT_METHOD: &str = "Rebuild it with `python fixtures/print-shop-pdfx.PROVENANCE.py`.";
const EVENT: &str = "pdfx-claim"; // ui-text-exempt: a trace event name, never displayed
const UI_RECT: &str = "ui-rect";
/// `panels::docprops::pdfx::REGION`.
const REGION: &str = "docprops.pdfx"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct DocumentPropertiesSayWhichPdfxLevelTheFileClaims;

impl Check for DocumentPropertiesSayWhichPdfxLevelTheFileClaims {
    fn name(&self) -> &'static str {
        "document_properties_say_which_pdfx_level_the_file_claims"
    }

    fn defect(&self) -> &'static str {
        "a file made for a press claims a PDF/X level and Document properties does not say so, \
         or reads an output intent alone as a claim"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match steps(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn steps(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let session = launch_on(ctx, report, CLAIMS, CLAIMS_METHOD, "pdfx_claim")?;
    let Some(line) = await_line(&session, EVENT)? else {
        return Ok(Some(format!(
            "Document properties was opened on {CLAIMS} and no `{EVENT}` line followed."
        )));
    };
    report.note(format!("claims: `{}`", line.raw));
    let owed = [
        ("level", "PDF/X-4"),
        ("recognised", "1"),
        ("agree", "1"),
        ("intents", "0"),
    ];
    if owed.iter().any(|(k, v)| line.get(k) != Some(*v)) {
        return Ok(Some(format!(
            "{CLAIMS} claims PDF/X-4 in XMP with no output intent; the panel read `{}`.",
            line.raw
        )));
    }
    if declared(&session.trace()?, UI_RECT, REGION).is_none() {
        return Ok(Some(format!(
            "the claim was read and no visible `{REGION}` region was published."
        )));
    }
    drop(session);

    let control = launch_on(
        ctx,
        report,
        INTENT_ONLY,
        INTENT_METHOD,
        "pdfx_claim_control",
    )?;
    let Some(line) = await_line(&control, EVENT)? else {
        return Ok(Some(format!(
            "Document properties was opened on {INTENT_ONLY} and no `{EVENT}` line followed."
        )));
    };
    report.note(format!("control: `{}`", line.raw));
    if !line.raw.ends_with("pdfx-claim none") {
        return Ok(Some(format!(
            "{INTENT_ONLY} has a PDF/X output intent and states no level; the panel read a \
             claim: `{}`.",
            line.raw
        )));
    }
    if declared(&control.trace()?, UI_RECT, REGION).is_some() {
        return Ok(Some(format!(
            "{INTENT_ONLY} claims nothing and a `{REGION}` row was drawn."
        )));
    }
    Ok(None)
}
