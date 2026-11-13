//! Output Preview's print-production answers, driven on two fixtures.
//!
//! `output_preview_overprint` — the panel says whether the page uses
//! overprint. `output-preview.pdf` sets it on a patch whose four inks are all
//! non-zero, so it is set and changes nothing (`set=1 effect=0`);
//! `print-shop.pdf` overprints magenta onto cyan, so it takes effect
//! (`set=1 effect=1`). Each fixture is the other's control: a panel that
//! always says one answer fails on one of them.
//!
//! `output_preview_soft_proof` — the panel's *Simulate* list offers the
//! document's output intent, choosing it proofs the canvas (a new render, not
//! only a new menu label), and *Simulate paper colour* turns on black ink too
//! and renders again.
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
const PROOF: &str = "output-preview-proof"; // ui-text-exempt: a trace event name, never displayed
const PROOF_LIST: &str = "output-preview.proof";
const PROOF_INTENT: &str = "output-preview.proof:intent";
const PAPER: &str = "output-preview.paper";
const BLACK_INK: &str = "output-preview.black-ink";
const RENDER: &str = "render-spawn"; // ui-text-exempt: a trace event name, never displayed

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

/// [`overprint`] on `fixture`.
fn drive_on(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: (&str, &str),
    effect: &str,
) -> Result<Option<String>> {
    let name = format!("output_preview_overprint.{}", fixture.0);
    with_preview(ctx, report, &name, fixture, |report, session, _, mark| {
        overprint(report, session, fixture.0, effect, mark)
    })
}

/// Launch on `fixture`, switch the preview on, run `body`, and park the pointer.
fn with_preview(
    ctx: &CheckContext,
    report: &mut CheckReport,
    name: &str,
    fixture: (&str, &str),
    body: impl FnOnce(&mut CheckReport, &Session, &ScriptedPointer, usize) -> Result<Option<String>>,
) -> Result<Option<String>> {
    let (session, pointer) = launch_on(ctx, report, name, None, fixture)?;
    let outcome =
        switch_on(&session, &pointer).and_then(|mark| body(report, &session, &pointer, mark));
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Open the panel unless it is up, press the switch, and return the trace
/// mark taken before the press.
fn switch_on(session: &Session, pointer: &ScriptedPointer) -> Result<usize> {
    // Sessions of one check share a profile, so a later one opens with the
    // panel an earlier one left up; pressing the item again would close it.
    if declared_in(&session.trace()?, UI_RECT, OUTPUT_SWITCH).is_none() {
        press(session, pointer, VIEW_TAB)?;
        press(session, pointer, OUTPUT_ITEM)?;
    }
    let mark = session.trace()?.mark();
    press(session, pointer, OUTPUT_SWITCH)?;
    Ok(mark)
}

/// See the module documentation.
pub struct OutputPreviewSimulatesThePress;

impl Check for OutputPreviewSimulatesThePress {
    fn name(&self) -> &'static str {
        "output_preview_soft_proof"
    }

    fn defect(&self) -> &'static str {
        "Output Preview cannot simulate the document's output intent, or choosing a simulation or paper colour changes the panel without redrawing the canvas"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match with_preview(ctx, &mut report, self.name(), FIXTURE, soft_proof) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Choose the output intent, then paper colour; each owes its proof line and
/// a render started after the press.
fn soft_proof(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    mark: usize,
) -> Result<Option<String>> {
    if await_line(session, PROOF, mark, |l| l.get("source") == Some("none"))?.is_none() {
        return Ok(Some(format!(
            "the preview was switched on and no `{PROOF} source=none` line followed."
        )));
    }
    let mark = session.trace()?.mark();
    press(session, pointer, PROOF_LIST)?;
    if declared_in(&session.trace()?, UI_RECT, PROOF_INTENT).is_none() {
        return Ok(Some(format!(
            "the simulation list was opened and `{PROOF_INTENT}` was not in it: the document's output intent is not offered."
        )));
    }
    press(session, pointer, PROOF_INTENT)?;
    let owed = [("source", "intent"), ("paper", "0"), ("black", "0")];
    if let Some(failure) = proofed(report, session, mark, &owed)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(session, pointer, PAPER)?;
    let owed = [("source", "intent"), ("paper", "1"), ("black", "1")];
    if let Some(failure) = proofed(report, session, mark, &owed)? {
        return Ok(Some(failure));
    }
    if declared_in(&session.trace()?, UI_RECT, BLACK_INK).is_none() {
        return Ok(Some(format!(
            "a profile is simulated and `{BLACK_INK}` was never drawn."
        )));
    }
    Ok(None)
}

/// The proof line `owed` after `mark`, and a render spawned after it.
fn proofed(
    report: &mut CheckReport,
    session: &Session,
    mark: usize,
    owed: &[(&str, &str)],
) -> Result<Option<String>> {
    let matches = |l: &crate::trace::TraceLine| owed.iter().all(|(k, v)| l.get(k) == Some(v));
    let Some(line) = await_line(session, PROOF, mark, matches)? else {
        let last = session.trace()?.last(PROOF).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "no `{PROOF}` line with {owed:?} followed; last was {last:?}."
        )));
    };
    report.note(format!("proof: `{}`", line.raw));
    if await_line(session, RENDER, mark, |_| true)?.is_none() {
        return Ok(Some(format!(
            "`{}` was traced and no `{RENDER}` followed: the canvas kept the picture drawn before the simulation changed.",
            line.raw
        )));
    }
    Ok(None)
}

/// The overprint line `fixture` owes after the switch at `mark`.
fn overprint(
    report: &mut CheckReport,
    session: &Session,
    fixture: &str,
    effect: &str,
    mark: usize,
) -> Result<Option<String>> {
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
