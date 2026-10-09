//! `a_part_of_a_placed_drawing_can_be_copied_and_pasted` — a polyline inside
//! a form XObject is selected, copied and pasted; the paste must land as page
//! content.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/form_part_copy.md`.

use super::dimdrive::press_on_tab;
use super::os_image_paste::ClipGuard;
use crate::checks::form_node_move::{enter_leaf_at, run_body};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "../../fixtures/form-parts.pdf";
const COPIED: &str = "clipboard-copy"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "paste-objects-applied"; // ui-text-exempt: a trace event name, never displayed
const EDIT_TAB: &str = "ribbon.tab.edit";
const PASTE: &str = "ribbon.item.edit.paste";
/// Inside the polyline's first leg, leaf 1 of `form-parts.pdf`.
const ON_THE_POLYLINE: (f64, f64) = (240.0, 120.0);

/// See the module documentation.
pub struct APartOfAPlacedDrawingCanBeCopiedAndPasted;

impl Check for APartOfAPlacedDrawingCanBeCopiedAndPasted {
    fn name(&self) -> &'static str {
        "a_part_of_a_placed_drawing_can_be_copied_and_pasted"
    }

    fn defect(&self) -> &'static str {
        "a part inside a placed drawing is selected and Copy copies nothing, copies the whole \
         drawing, or pastes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let outcome = run_body(ctx, &mut report, FIXTURE, self.name(), copy_and_paste);
        // The copy wrote the OS clipboard; put back what was there before.
        guard.adopt();
        report.note(guard.release());
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn copy_and_paste(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    enter_leaf_at(ctx, session, pointer, page, ON_THE_POLYLINE)?;
    let mark = session.trace()?.mark();
    pointer.copy(session, None)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(copied) = trace.last_after(COPIED, mark) else {
        return Ok(Some(format!(
            "Copy of a part of a placed drawing wrote no `{COPIED}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("copy: `{}`", copied.raw));
    let right = copied.get("kind") == Some("form-leaves")
        && copied.get("page") == Some("0")
        && copied.get("leaves") == Some("1")
        && copied.get_usize("items") == Some(1);
    if !right {
        return Ok(Some(format!(
            "the copy must read `kind=form-leaves page=0 leaves=1 items=1`; it read `{}`. \
             Trace: {}.",
            copied.raw,
            session.trace_path().display()
        )));
    }

    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, PASTE, EDIT_TAB)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(applied) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "Edit ▸ Paste after the copy wrote no `{APPLIED}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("paste: `{}`", applied.raw));
    if applied.get_usize("pasted") != Some(1) || applied.get("page") != Some("0") {
        return Ok(Some(format!(
            "the paste must put one object on page 0; it read `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    Ok(None)
}
