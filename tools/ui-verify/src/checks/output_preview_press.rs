//! Output Preview's print-production answers, driven on two fixtures.
//!
//! `output_preview_overprint` — the panel says whether the page uses
//! overprint. `output-preview.pdf` sets it on a patch whose four inks are all
//! non-zero, so it is set and changes nothing (`set=1 effect=0`);
//! `print-shop.pdf` overprints magenta onto cyan, so it takes effect
//! (`set=1 effect=1`). Each fixture is the other's control: a panel that
//! always says one answer fails on one of them.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/output_preview_press.md`.

use crate::checks::driving::declared_in;
use crate::checks::print_shop::{
    OUTPUT_ITEM, OUTPUT_SWITCH, VIEW_TAB, await_line, launch_on, press,
};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: (&str, &str) = (
    "output-preview.pdf",
    "Rebuild it with `python fixtures/output-preview.PROVENANCE.py`.",
);
const PRINT_SHOP: (&str, &str) = (
    "print-shop.pdf",
    "Rebuild it with `python fixtures/print-shop.PROVENANCE.py`.",
);
const OVERPRINT: &str = "output-preview-overprint"; // ui-text-exempt: a trace event name, never displayed
const OVERPRINT_LINE: &str = "output-preview.overprint";
const UI_RECT: &str = "ui-rect";

/// See the module documentation.
pub struct OutputPreviewSaysWhetherThePageOverprints;

impl Check for OutputPreviewSaysWhetherThePageOverprints {
    fn name(&self) -> &'static str {
        "output_preview_overprint"
    }

    fn defect(&self) -> &'static str {
        "Output Preview does not say whether the page uses overprint, or gives the same answer \
         for a page where overprint changes the print and one where it changes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut outcome = Ok(None);
        for (fixture, effect) in [(FIXTURE, "0"), (PRINT_SHOP, "1")] {
            outcome = drive_on(ctx, &mut report, fixture, effect);
            if !matches!(outcome, Ok(None)) {
                break;
            }
        }
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Launch on `fixture`, run [`overprint`], and park the pointer.
fn drive_on(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: (&str, &str),
    effect: &str,
) -> Result<Option<String>> {
    let name = format!("output_preview_overprint.{}", fixture.0);
    let (session, pointer) = launch_on(ctx, report, &name, None, fixture)?;
    let outcome = overprint(report, &session, &pointer, fixture.0, effect);
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Switch the preview on and require the overprint line `fixture` owes.
fn overprint(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    fixture: &str,
    effect: &str,
) -> Result<Option<String>> {
    // Both sessions share one profile, so the second opens with the panel the
    // first left up; pressing the item again would close it.
    if declared_in(&session.trace()?, UI_RECT, OUTPUT_SWITCH).is_none() {
        press(session, pointer, VIEW_TAB)?;
        press(session, pointer, OUTPUT_ITEM)?;
    }
    let mark = session.trace()?.mark();
    press(session, pointer, OUTPUT_SWITCH)?;
    let Some(line) = await_line(session, OVERPRINT, mark, |_| true)? else {
        return Ok(Some(format!(
            "on {fixture} the preview was switched on and no `{OVERPRINT}` line followed."
        )));
    };
    report.note(format!("{fixture}: `{}`", line.raw));
    let owed = [("set", "1"), ("effect", effect), ("whole", "1")];
    if let Some((key, want)) = owed.iter().find(|(k, v)| line.get(k) != Some(v)) {
        return Ok(Some(format!(
            "on {fixture} `{OVERPRINT}` owes {key}={want}; it traced `{}`.",
            line.raw
        )));
    }
    if declared_in(&session.trace()?, UI_RECT, OVERPRINT_LINE).is_none() {
        return Ok(Some(format!(
            "on {fixture} the overprint answer was traced but `{OVERPRINT_LINE}` was never drawn."
        )));
    }
    Ok(None)
}
