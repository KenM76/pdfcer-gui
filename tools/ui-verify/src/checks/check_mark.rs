//! `a_check_boxs_mark_can_be_chosen` — the Properties panel's Mark picker
//! changes another program's check box from a tick to a star, and the box is
//! redrawn with it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/check_mark.md`.

use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "all-field-kinds.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/all-field-kinds.PROVENANCE.py`.";
const FIELD: &str = "CheckOne";
const COMBO: &str = "properties.widget_edit.mark";
/// Star is entry 2 of `CHECK_STYLES`.
const STAR_ENTRY: &str = "properties.widget_edit.mark.2";
const SHOWN: &str = "widget-mark-shown";
const APPLIED: &str = "edit-widget-applied";

/// See the module documentation.
pub struct ACheckBoxsMarkCanBeChosen;

impl Check for ACheckBoxsMarkCanBeChosen {
    fn name(&self) -> &'static str {
        "a_check_boxs_mark_can_be_chosen"
    }

    fn defect(&self) -> &'static str {
        "a check box's Properties panel offers no choice of mark, or a pick never reaches \
         edit_widget, or the box keeps the old program's tick because it was not redrawn"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = properties_pane::launch_on_field(
            ctx,
            &mut report,
            (FIXTURE, METHOD),
            FIELD,
            "check_mark",
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
    let shown = (SHOWN, FIELD);
    // The fixture's `/MK /CA (4)` is ZapfDingbats' tick.
    if let Some(failure) =
        properties_pane::reads(session, report, shown, "before any press", &[("mark", "4")])?
    {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, COMBO, 15)?;
    properties_pane::press(ctx, session, pointer, STAR_ENTRY, 30)?;
    if let Some(failure) = properties_pane::reads(
        session,
        report,
        (APPLIED, FIELD),
        "after Star",
        &[("redrawn", "yes")],
    )? {
        return Ok(Some(failure));
    }
    // `H` is ZapfDingbats' star, read back from the box's `/MK /CA`.
    properties_pane::reads(session, report, shown, "after Star", &[("mark", "H")])
}
