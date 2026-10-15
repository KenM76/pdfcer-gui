//! `several_annotations_drag_as_one_undo_step` — three markups and a link
//! selected, one of them locked, dragged by a press on a member that is not
//! the primary; the three unlocked ones move by the same distance, the locked
//! one stays and is said so, and one Undo puts all of it back, judged in saved
//! copies.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/annot_group_drag.md`.

use super::annot_multiselect::{CIRCLE, FIXTURE, LINK, SQUARE};
use super::dimdrive::click_on;
use super::driving;
use super::multiselect::{Saves, after, run_saving, save, shift_click, step, tops, window_at};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The locked square: its key, a point on it and its top edge.
pub(super) const LOCKED: (&str, (f64, f64), f64) = ("/NM (LO)", (50.0, 35.0), 50.0); // ui-text-exempt: a PDF key
/// How far down the drag goes, in page points.
const DROP: f64 = 30.0;
const TOGGLED: &str = "annot-toggled"; // ui-text-exempt: a trace event name
const APPLIED: &str = "move-mixed-applied"; // ui-text-exempt: a trace event name
const DISCLOSURE: &str = "status-group:edit-disclosure"; // ui-text-exempt: a trace region name

/// See the module documentation.
pub struct SeveralAnnotationsDragAsOneUndoStep;

impl Check for SeveralAnnotationsDragAsOneUndoStep {
    fn name(&self) -> &'static str {
        "several_annotations_drag_as_one_undo_step"
    }

    fn defect(&self) -> &'static str {
        "dragging one of several selected markups and links moves only that one, or nothing, \
         or takes one Undo each to put back, or moves a locked one, or leaves it without saying \
         so"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_saving(self, ctx, &FIXTURE, "mode.edit", "annot-group-drag", drive)
    }
}

/// Select the square by a click, then the circle, the link and the locked
/// square by Shift-click, so the locked one is the primary.
pub(super) fn select_four(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &FIXTURE, 0, SQUARE.1)?;
    session.settle(20);
    for (n, (key, at, _)) in [CIRCLE, LINK, LOCKED].into_iter().enumerate() {
        let mark = session.trace()?.mark();
        shift_click(ctx, session, pointer, &FIXTURE, at)?;
        let toggled = step!(after(
            session,
            TOGGLED,
            mark,
            &format!("`{key}` was Shift-clicked")
        ));
        let now = (n + 2).to_string();
        if toggled.get("after") != Some(now.as_str()) {
            return Ok(Some(format!(
                "Shift-clicking `{key}` traced `{}`: after={now} was wanted.",
                toggled.raw
            )));
        }
    }
    Ok(None)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &Saves,
) -> Result<Option<String>> {
    let (dragged, undone) = (saves.moved.as_path(), saves.undone.as_path());
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE).is_some() {
        return Ok(Some(format!(
            "`{DISCLOSURE}` was on screen before anything was done, so the drag's own \
             sentence could not be told from it."
        )));
    }

    if let Some(why) = select_four(ctx, session, pointer)? {
        return Ok(Some(why));
    }

    // --- one drag on the circle: three move, the locked one stays, said ---
    let from = CIRCLE.1;
    let to = (from.0, from.1 - DROP);
    let (wf, wt) = (
        window_at(ctx, session, &FIXTURE, from)?,
        window_at(ctx, session, &FIXTURE, to)?,
    );
    let mark = session.trace()?.mark();
    pointer.drag(session, wf, wt, 8)?;
    session.settle(40);
    let applied = step!(after(session, APPLIED, mark, "the circle was dragged down"));
    report.note(format!("`{}`", applied.raw));
    if applied.get("gesture") != Some("drag")
        || applied.get("annots") != Some("3/4")
        || applied.get("one_step") != Some("true")
    {
        return Ok(Some(format!(
            "a drag over four selected annotations, one locked, traced `{}`: gesture=drag, \
             three of four moved and one undo step was wanted.",
            applied.raw
        )));
    }
    session.settle(20);
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE).is_none() {
        return Ok(Some(format!(
            "the locked annotation stayed and `{DISCLOSURE}` was not drawn: the operator is \
             not told why it did not move."
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let moved = [
        (SQUARE.0, SQUARE.2 - DROP),
        (CIRCLE.0, CIRCLE.2 - DROP),
        (LINK.0, LINK.2 - DROP),
        (LOCKED.0, LOCKED.2),
    ];
    if let Some(why) = tops(dragged, &moved)? {
        return Ok(Some(format!("after the drag, {why}")));
    }

    // --- one Undo puts all three back -------------------------------------
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let restored = [SQUARE, CIRCLE, LINK, LOCKED].map(|(key, _, top)| (key, top));
    if let Some(why) = tops(undone, &restored)? {
        return Ok(Some(format!(
            "after ONE Undo of the drag, {why} The moves were not one undo step."
        )));
    }
    Ok(None)
}
