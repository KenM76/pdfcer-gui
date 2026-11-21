//! `objects_panel_says_where_a_placed_drawing_is_drawn` — on a page placing a
//! title block that three pages share and a detail only this page holds, the
//! Objects panel says how widely each is drawn.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/placed_drawings.md`.

use crate::checks::driving::declared;
use crate::checks::security_notes::{await_line, launch_invoking};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "shared-title-block.pdf";
const METHOD: &str = "Rebuild it with `python -I fixtures/shared-title-block.PROVENANCE.py`.";
const INVOKE: &str = "view.panel_objects";
const EVENT: &str = "objects-placed"; // ui-text-exempt: a trace event name, never displayed
const UI_RECT: &str = "ui-rect";
/// `panels::objects::placed::REGION`.
const REGION: &str = "panel.objects.placed"; // ui-text-exempt: a trace region name, never displayed
/// Form 4 is placed on all three pages and twice on the second; form 5 once.
const OWED: [(&str, &str, &str); 2] = [("4", "4", "3"), ("5", "1", "1")];

/// See the module documentation.
pub struct ObjectsPanelSaysWhereAPlacedDrawingIsDrawn;

impl Check for ObjectsPanelSaysWhereAPlacedDrawingIsDrawn {
    fn name(&self) -> &'static str {
        "objects_panel_says_where_a_placed_drawing_is_drawn"
    }

    fn defect(&self) -> &'static str {
        "a title block shared by every sheet is listed like any other object, so an operator \
         edits inside it believing the change stays on this page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_invoking(
            ctx,
            &mut report,
            (FIXTURE, METHOD),
            "placed_drawings",
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
    if await_line(session, EVENT)?.is_none() {
        return Ok(Some(format!(
            "the Objects panel was opened on {FIXTURE} and no `{EVENT}` line followed: the \
             panel never said where its placed drawings are drawn."
        )));
    }
    session.settle(10);
    let trace = session.trace()?;
    for (form, places, pages) in OWED {
        let Some(line) = trace
            .events(EVENT)
            .filter(|l| l.get("page") == Some("0") && l.get("form") == Some(form))
            .last()
        else {
            return Ok(Some(format!(
                "page 0 places form {form} and the panel traced no `{EVENT}` line for it."
            )));
        };
        report.note(format!("form {form}: `{}`", line.raw));
        let said = [
            line.get("places"),
            line.get("pages"),
            line.get("lower_bound"),
        ];
        if said != [Some(places), Some(pages), Some("0")] {
            return Ok(Some(format!(
                "form {form} is drawn in {places} place(s) on {pages} page(s), counted whole; \
                 the panel said `{}`.",
                line.raw
            )));
        }
    }
    if declared(&trace, UI_RECT, REGION).is_none() {
        return Ok(Some(format!(
            "the counts were right and no visible `{REGION}` region was published: the operator \
             is never told."
        )));
    }
    Ok(None)
}
