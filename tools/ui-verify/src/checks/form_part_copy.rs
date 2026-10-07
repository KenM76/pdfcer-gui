//! `copying_a_part_of_a_placed_drawing_says_why_nothing_was_copied` — a block
//! inside a form XObject is selected and copied; the engine copies page
//! objects only (G145), so the copy must be refused out loud, not as "nothing
//! is selected".
//!
//! Design and rationale: `docs/modules/ui-verify/checks/form_part_copy.md`.

use crate::checks::driving;
use crate::checks::form_node_move::{enter_leaf_at, run_body};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "../../fixtures/form-parts.pdf";
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "clipboard-copy-refused"; // ui-text-exempt: a trace event name, never displayed
const COPIED: &str = "clipboard-copy"; // ui-text-exempt: a trace event name, never displayed
/// Mirrors `app::status::REGION_EDIT_DISCLOSURE`.
const DISCLOSURE: &str = "status-group:edit-disclosure"; // ui-text-exempt: a trace region name, never displayed
/// Inside the polyline's first leg, leaf 1 of `form-parts.pdf`.
const ON_THE_POLYLINE: (f64, f64) = (240.0, 120.0);

/// See the module documentation.
pub struct CopyingAPartOfAPlacedDrawingSaysWhyNothingWasCopied;

impl Check for CopyingAPartOfAPlacedDrawingSaysWhyNothingWasCopied {
    fn name(&self) -> &'static str {
        "copying_a_part_of_a_placed_drawing_says_why_nothing_was_copied"
    }

    fn defect(&self) -> &'static str {
        "a part inside a placed drawing is selected and Copy says nothing is selected, or \
         says nothing at all"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, self.name(), select_and_copy) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn select_and_copy(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    let (ui_rect, _) = enter_leaf_at(ctx, session, pointer, page, ON_THE_POLYLINE)?;
    let trace = session.trace()?;
    report.note(format!(
        "selected: `{}`",
        trace.last(SELECTION).map_or("none", |l| l.raw.as_str())
    ));
    let mark = trace.mark();
    pointer.copy(session, None)?;
    session.settle(30);

    let trace = session.trace()?;
    let refused = trace.last_after(REFUSED, mark);
    report.note(format!(
        "after Copy: `{}`",
        refused.map_or("no refusal", |l| l.raw.as_str())
    ));
    if let Some(copied) = trace.last_after(COPIED, mark) {
        return Ok(Some(format!(
            "Copy of a part of a placed drawing reported a copy: `{}`. The engine has no \
             in-form copy (G145). Trace: {}.",
            copied.raw,
            session.trace_path().display()
        )));
    }
    let Some(refused) = refused.filter(|l| l.get("reason") == Some("inside-form")) else {
        return Ok(Some(format!(
            "Copy of a part of a placed drawing was not refused as inside-form (last refusal: \
             {}). The operator is told nothing is selected while it is. Trace: {}.",
            refused.map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    if refused.get_usize("n").is_none_or(|n| n == 0) {
        return Ok(Some(format!(
            "`{}` counts no part. Trace: {}.",
            refused.raw,
            session.trace_path().display()
        )));
    }
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE).is_none() {
        return Ok(Some(format!(
            "the copy was refused and the status bar drew no `{DISCLOSURE}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok(None)
}
