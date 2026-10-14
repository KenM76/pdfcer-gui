//! `several_form_fields_drag_as_one_undo_step` — three form fields selected
//! and dragged by a press on one that is not the primary; all three move by
//! the same distance, and one Undo puts them back, judged in saved copies.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_group_drag.md`.

use super::dimdrive::{Fixture, click_on, launch_with};
use super::multiselect::{after, save, shift_click, step, tops, window_at};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
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
/// Each field's key in a saved copy, a point on it and its top edge.
type Spot = (&'static str, (f64, f64), f64);
const TEXT: Spot = ("/T (TextOne)", (270.0, 531.0), 542.0); // ui-text-exempt: a PDF key
const CHECK: Spot = ("/T (CheckOne)", (178.0, 494.0), 502.0); // ui-text-exempt: a PDF key
const COMBO: Spot = ("/T (ComboOne)", (270.0, 423.0), 434.0); // ui-text-exempt: a PDF key
/// How far down the drag goes, in page points.
const DROP: f64 = 30.0;
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const TOGGLED: &str = "form-field-toggled"; // ui-text-exempt: a trace event name
const APPLIED: &str = "move-mixed-applied"; // ui-text-exempt: a trace event name
/// The single-box drag's line, which a group drag never writes.
const SINGLE: &str = "widget-drag"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct SeveralFormFieldsDragAsOneUndoStep;

impl Check for SeveralFormFieldsDragAsOneUndoStep {
    fn name(&self) -> &'static str {
        "several_form_fields_drag_as_one_undo_step"
    }

    fn defect(&self) -> &'static str {
        "dragging one of several selected form fields moves only that one, or nothing, or \
         replaces the selection, or takes one Undo each to put back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let dragged = ctx.out("field-group-drag.dragged.pdf");
        let undone = ctx.out("field-group-drag.undone.pdf");
        for p in [&dragged, &undone] {
            let _ = std::fs::remove_file(p);
        }
        let env = [(
            SAVE_PATH_ENV,
            format!("{};{}", dragged.display(), undone.display()),
        )];
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit",
            "field-group-drag",
            &env,
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer, &dragged, &undone);
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
    dragged: &std::path::Path,
    undone: &std::path::Path,
) -> Result<Option<String>> {
    // --- TextOne by a click; CheckOne and ComboOne by Shift-click ----------
    click_on(ctx, session, pointer, &FIXTURE, 0, TEXT.1)?;
    session.settle(20);
    for (n, (key, at, _)) in [CHECK, COMBO].into_iter().enumerate() {
        let mark = session.trace()?.mark();
        shift_click(ctx, session, pointer, &FIXTURE, at)?;
        let toggled = step!(after(
            session,
            TOGGLED,
            mark,
            &format!("`{key}` was Shift-clicked")
        ));
        let before = (n + 1).to_string();
        if toggled.get("before") != Some(before.as_str()) {
            return Ok(Some(format!(
                "Shift-clicking `{key}` with {before} field(s) selected traced `{}`.",
                toggled.raw
            )));
        }
    }

    // --- one drag on CheckOne, which is not the primary: all three move ----
    let from = CHECK.1;
    let to = (from.0, from.1 - DROP);
    let (wf, wt) = (
        window_at(ctx, session, &FIXTURE, from)?,
        window_at(ctx, session, &FIXTURE, to)?,
    );
    let mark = session.trace()?.mark();
    pointer.drag(session, wf, wt, 8)?;
    session.settle(40);
    let applied = step!(after(session, APPLIED, mark, "CheckOne was dragged down"));
    report.note(format!("`{}`", applied.raw));
    if applied.get("gesture") != Some("drag")
        || applied.get("widgets") != Some("3/3")
        || applied.get("one_step") != Some("true")
    {
        return Ok(Some(format!(
            "a drag over three selected fields traced `{}`: gesture=drag, three moved and one \
             undo step was wanted.",
            applied.raw
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let moved = [TEXT, CHECK, COMBO].map(|(key, _, top)| (key, top - DROP));
    if let Some(why) = tops(dragged, &moved)? {
        return Ok(Some(format!("after the drag, {why}")));
    }

    // --- one Undo puts all three back ---------------------------------------
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let restored = [TEXT, CHECK, COMBO].map(|(key, _, top)| (key, top));
    if let Some(why) = tops(undone, &restored)? {
        return Ok(Some(format!(
            "after ONE Undo of the drag, {why} The moves were not one undo step."
        )));
    }

    // --- one field selected alone still drags alone ------------------------
    pointer.key(session, None, "Escape", None)?;
    session.settle(20);
    click_on(ctx, session, pointer, &FIXTURE, 0, TEXT.1)?;
    session.settle(20);
    let from = TEXT.1;
    let (wf, wt) = (
        window_at(ctx, session, &FIXTURE, from)?,
        window_at(ctx, session, &FIXTURE, (from.0, from.1 - DROP))?,
    );
    let mark = session.trace()?.mark();
    pointer.drag(session, wf, wt, 8)?;
    session.settle(40);
    let single = step!(after(
        session,
        SINGLE,
        mark,
        "TextOne alone was dragged down"
    ));
    report.note(format!("`{}`", single.raw));
    if let Some(group) = session.trace()?.last_after(APPLIED, mark) {
        return Ok(Some(format!(
            "with only TextOne selected, its drag moved a group: `{}`.",
            group.raw
        )));
    }
    Ok(None)
}
