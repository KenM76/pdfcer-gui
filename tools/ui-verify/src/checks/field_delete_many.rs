//! `several_form_fields_delete_as_one_undo_step` — a text field, a check box
//! and both buttons of one radio group selected by Shift-click, deleted by one
//! press of Delete, and all of it brought back by one Undo, judged by the
//! boxes a saved copy lists on its page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_delete_many.md`.

use super::dimdrive::{Fixture, click_on, launch_with};
use super::multiselect::{after, save, shift_click, step};
use super::savedannots::listed_annots;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "all-field-kinds.pdf",
    method: "Rebuild it with `python fixtures/all-field-kinds.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 420.0,
        height_pt: 620.0,
    },
};
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const TOGGLED: &str = "form-field-toggled"; // ui-text-exempt: a trace event name
const APPLIED: &str = "delete-widgets-applied"; // ui-text-exempt: a trace event name

/// A box's name, the lower-left corner of its `/Rect` (which finds it in a
/// saved copy), and a point on it.
type Spot = (&'static str, (f64, f64), (f64, f64));
const TEXT: Spot = ("TextOne", (170.0, 520.0), (270.0, 531.0)); // ui-text-exempt: a fixture field name
const CHECK: Spot = ("CheckOne", (170.0, 486.0), (178.0, 494.0)); // ui-text-exempt: a fixture field name
/// Both members of `RadioGroup`, one field with two boxes, so deleting both
/// tests that the second box is still found after the first renumbers it.
const RADIO_A: Spot = ("RadioGroup #0", (170.0, 452.0), (178.0, 460.0)); // ui-text-exempt: a fixture field name
const RADIO_B: Spot = ("RadioGroup #1", (260.0, 452.0), (268.0, 460.0)); // ui-text-exempt: a fixture field name
/// Not selected, so it must survive.
const COMBO: Spot = ("ComboOne", (170.0, 412.0), (270.0, 423.0)); // ui-text-exempt: a fixture field name
/// The fixture's page lists ten widgets.
const LISTED: usize = 10;

/// See the module documentation.
pub struct SeveralFormFieldsDeleteAsOneUndoStep;

impl Check for SeveralFormFieldsDeleteAsOneUndoStep {
    fn name(&self) -> &'static str {
        "several_form_fields_delete_as_one_undo_step"
    }

    fn defect(&self) -> &'static str {
        "Delete with several form fields selected deletes only the primary one, or misses the \
         second box of a radio group because the first renumbered it, or takes one Undo per \
         box to bring them back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let deleted = ctx.out("field-delete-many.deleted.pdf");
        let undone = ctx.out("field-delete-many.undone.pdf");
        for p in [&deleted, &undone] {
            let _ = std::fs::remove_file(p);
        }
        let env = [(
            SAVE_PATH_ENV,
            format!("{};{}", deleted.display(), undone.display()),
        )];
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit",
            "field-delete-many",
            &env,
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer, &deleted, &undone);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    deleted: &std::path::Path,
    undone: &std::path::Path,
) -> Result<Option<String>> {
    // --- TextOne by a click; the check box and both radio boxes by
    //     Shift-click --------------------------------------------------------
    click_on(ctx, session, pointer, &FIXTURE, 0, TEXT.2)?;
    session.settle(20);
    for (n, (name, _, at)) in [CHECK, RADIO_A, RADIO_B].into_iter().enumerate() {
        let mark = session.trace()?.mark();
        shift_click(ctx, session, pointer, &FIXTURE, at)?;
        let toggled = step!(after(
            session,
            TOGGLED,
            mark,
            &format!("{name} was Shift-clicked")
        ));
        let before = (n + 1).to_string();
        if toggled.get("before") != Some(before.as_str()) {
            return Ok(Some(format!(
                "Shift-clicking {name} with {before} field(s) selected traced `{}`.",
                toggled.raw
            )));
        }
    }

    // --- one Delete: four boxes, one step -----------------------------------
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Delete", None)?;
    session.settle(40);
    let applied = step!(after(session, APPLIED, mark, "Delete was pressed"));
    report.note(format!("`{}`", applied.raw));
    if applied.get("deleted") != Some("4/4") || applied.get("one_step") != Some("true") {
        return Ok(Some(format!(
            "Delete over four selected boxes traced `{}`: four deleted as one undo step was \
             wanted.",
            applied.raw
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let gone = [
        (TEXT, false),
        (CHECK, false),
        (RADIO_A, false),
        (RADIO_B, false),
        (COMBO, true),
    ];
    if let Some(why) = listed(deleted, &gone, LISTED - 4)? {
        return Ok(Some(format!("after Delete, {why}")));
    }

    // --- one Undo brings all four back --------------------------------------
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let back = gone.map(|(b, _)| (b, true));
    if let Some(why) = listed(undone, &back, LISTED)? {
        return Ok(Some(format!(
            "after ONE Undo of the delete, {why} The deletes were not one undo step."
        )));
    }
    Ok(None)
}

/// The failure sentence when the saved page lists the wrong boxes, or the
/// wrong number of them.
fn listed(path: &std::path::Path, want: &[(Spot, bool)], count: usize) -> Result<Option<String>> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let annots = listed_annots(&bytes);
    for ((name, corner, _), present) in want {
        let found = annots
            .iter()
            .any(|a| (a.rect[0] - corner.0).abs() < 0.5 && (a.rect[1] - corner.1).abs() < 0.5);
        if found != *present {
            let state = if found { "still lists" } else { "lists no" };
            return Ok(Some(format!(
                "{} {state} box for {name} at ({}, {}).",
                path.display(),
                corner.0,
                corner.1
            )));
        }
    }
    if annots.len() != count {
        return Ok(Some(format!(
            "{} lists {} boxes on the page; {count} were wanted.",
            path.display(),
            annots.len()
        )));
    }
    Ok(None)
}
