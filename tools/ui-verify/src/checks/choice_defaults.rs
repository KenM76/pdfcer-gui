//! `a_multi_select_lists_defaults_can_be_chosen` — the Properties panel's
//! default checkboxes on a multi-select list record more than one default
//! choice, read back from the document.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/choice_defaults.md`.

use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "all-field-kinds.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/all-field-kinds.PROVENANCE.py`.";
const FIELD: &str = "ListMulti";
/// Options are Mon, Tue, Wed, Thu, Fri; entry `i` is option `i`.
const TUE: &str = "properties.choice_opts.default.1";
const THU: &str = "properties.choice_opts.default.3";
const READ: &str = "choice-defaults-read";

/// See the module documentation.
pub struct AMultiSelectListsDefaultsCanBeChosen;

impl Check for AMultiSelectListsDefaultsCanBeChosen {
    fn name(&self) -> &'static str {
        "a_multi_select_lists_defaults_can_be_chosen"
    }

    fn defect(&self) -> &'static str {
        "a multi-select list can be given only one default choice, or the second pick replaces \
         the first instead of joining it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = properties_pane::launch_on_field(
            ctx,
            &mut report,
            (FIXTURE, METHOD),
            FIELD,
            "choice_defaults",
        )
        .and_then(|(session, pointer)| {
            let outcome = steps(ctx, &mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let read = (READ, FIELD);
    // The fixture's list has a value (`/V`) and no default (`/DV`).
    if let Some(failure) = properties_pane::reads(
        session,
        report,
        read,
        "before any press",
        &[("defaults", "-")],
    )? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, TUE, 30)?;
    if let Some(failure) =
        properties_pane::reads(session, report, read, "after Tue", &[("defaults", "Tue")])?
    {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, THU, 30)?;
    properties_pane::reads(
        session,
        report,
        read,
        "after Thu",
        &[("defaults", "Tue|Thu")],
    )
}
