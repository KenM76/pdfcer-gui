//! `objects_panel_says_where_the_list_and_page_disagree` — on a page holding
//! one fully transparent path, one gradient and one spot-coloured path, the
//! Objects panel says each in a line under its summary.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/object_disagreements.md`.

use crate::checks::driving::declared;
use crate::checks::security_notes::{await_line, launch_invoking};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "object-disagreements.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/object-disagreements.PROVENANCE.py`.";
const INVOKE: &str = "view.panel_objects";
const EVENT: &str = "objects-disagree"; // ui-text-exempt: a trace event name, never displayed
const UI_RECT: &str = "ui-rect";
/// `panels::objects::disagree::REGION`.
const REGION: &str = "panel.objects.disagree"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct ObjectsPanelSaysWhereTheListAndPageDisagree;

impl Check for ObjectsPanelSaysWhereTheListAndPageDisagree {
    fn name(&self) -> &'static str {
        "objects_panel_says_where_the_list_and_page_disagree"
    }

    fn defect(&self) -> &'static str {
        "a gradient the operator can see cannot be found in the object list, an invisible path \
         gets selected from empty paper, or a spot colour is named as something else, with \
         nothing anywhere saying why"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_invoking(
            ctx,
            &mut report,
            (FIXTURE, METHOD),
            "object_disagreements",
            INVOKE,
        )
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
            "the Objects panel was opened on {FIXTURE} and no `{EVENT}` line followed: the \
             panel never asked where its list and the page disagree."
        )));
    };
    report.note(format!("panel: `{}`", line.raw));
    // One of each, on page 0; the black control square adds to none.
    let owed = [
        ("page", "0"),
        ("invisible", "1"),
        ("shadings", "1"),
        ("undecoded", "1"),
    ];
    if owed.iter().any(|(k, v)| line.get(k) != Some(v)) {
        return Ok(Some(format!(
            "{FIXTURE} holds one transparent path, one gradient and one spot-coloured path; the \
             panel said `{}`.",
            line.raw
        )));
    }
    if declared(&session.trace()?, UI_RECT, REGION).is_none() {
        return Ok(Some(format!(
            "the counts were right and no visible `{REGION}` region was published: the operator \
             is never told."
        )));
    }
    Ok(None)
}
