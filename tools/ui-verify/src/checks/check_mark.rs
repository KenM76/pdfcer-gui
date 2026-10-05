//! `a_check_boxs_mark_can_be_chosen` and `a_radio_buttons_mark_can_be_chosen`
//! — the Properties panel's Mark picker changes another program's check box
//! from a tick, and its radio button from a dot, to a star, and the widget is
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
const COMBO: &str = "properties.widget_edit.mark";
/// Star is entry 2 of `CHECK_STYLES`.
const STAR_ENTRY: &str = "properties.widget_edit.mark.2";
const SHOWN: &str = "widget-mark-shown";
const APPLIED: &str = "edit-widget-applied";

/// One widget kind's drive: the field, and the `/MK /CA` character the
/// fixture gives it before any press.
struct Case {
    field: &'static str,
    before: &'static str,
    label: &'static str,
}

/// `CheckOne`'s `/MK /CA (4)` is ZapfDingbats' tick.
const CHECK_BOX: Case = Case {
    field: "CheckOne",
    before: "4",
    label: "check_mark",
};

/// `RadioGroup`'s first widget carries `/MK /CA (l)`, ZapfDingbats' dot.
const RADIO: Case = Case {
    field: "RadioGroup",
    before: "l",
    label: "radio_mark",
};

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
        drive(
            ctx,
            CheckReport::new(self.name(), self.defect()),
            &CHECK_BOX,
        )
    }
}

/// See the module documentation.
pub struct ARadioButtonsMarkCanBeChosen;

impl Check for ARadioButtonsMarkCanBeChosen {
    fn name(&self) -> &'static str {
        "a_radio_buttons_mark_can_be_chosen"
    }

    fn defect(&self) -> &'static str {
        "a radio button's Properties panel offers no choice of mark, or a pick never reaches \
         edit_widget, or the button keeps the old program's dot because it was not redrawn"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        drive(ctx, CheckReport::new(self.name(), self.defect()), &RADIO)
    }
}

fn drive(ctx: &CheckContext, mut report: CheckReport, case: &Case) -> CheckReport {
    let driven = properties_pane::launch_on_field(
        ctx,
        &mut report,
        (FIXTURE, METHOD),
        case.field,
        case.label,
    )
    .and_then(|(session, pointer)| {
        let outcome = steps(ctx, &mut report, &session, &pointer, case);
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

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    case: &Case,
) -> Result<Option<String>> {
    let shown = (SHOWN, case.field);
    if let Some(failure) = properties_pane::reads(
        session,
        report,
        shown,
        "before any press",
        &[("mark", case.before)],
    )? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, COMBO, 15)?;
    properties_pane::press(ctx, session, pointer, STAR_ENTRY, 30)?;
    if let Some(failure) = properties_pane::reads(
        session,
        report,
        (APPLIED, case.field),
        "after Star",
        &[("redrawn", "yes")],
    )? {
        return Ok(Some(failure));
    }
    // `H` is ZapfDingbats' star, read back from the widget's `/MK /CA`.
    properties_pane::reads(session, report, shown, "after Star", &[("mark", "H")])
}
