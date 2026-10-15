//! Arrow keys over a multi-item selection: several annotations, several form
//! fields, and a ce dimension each move by one press as one undo step, through
//! the same `MoveMixed` a group drag commits.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/groupnudge.md`.

use super::annot_group_drag::{LOCKED, select_four};
use super::annot_multiselect::{CIRCLE, FIXTURE as ANNOTS, LINK, SQUARE};
use super::dimdrive::run;
use super::dimension_text_select::select_by_line;
use super::driving;
use super::field_group_drag::{CHECK, COMBO, FIXTURE as FIELDS, TEXT, select_three};
use super::multiselect::{Saves, after, run_saving, saved_tops};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The bare arrow's step, page points.
const STEP: f64 = 1.0;
const APPLIED: &str = "move-mixed-applied"; // ui-text-exempt: a trace event name
const DISCLOSURE: &str = "status-group:edit-disclosure"; // ui-text-exempt: a trace region name

/// Press ArrowDown once and return the `move-mixed-applied` it wrote, or the
/// failure sentence when it wrote none or the wrong one.
fn nudge_down(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    key: &str,
    want: (&str, &str),
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, key, None)?;
    session.settle(40);
    let applied = match after(session, APPLIED, mark, &format!("{key} was pressed"))? {
        Ok(line) => line,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("`{}`", applied.raw));
    if applied.get("gesture") != Some("nudge")
        || applied.get(want.0) != Some(want.1)
        || applied.get("one_step") != Some("true")
    {
        return Ok(Some(format!(
            "one {key} traced `{}`: gesture=nudge, {}={} and one undo step were wanted.",
            applied.raw, want.0, want.1
        )));
    }
    Ok(None)
}

/// Undo once and judge the second saved copy against `restored`.
fn undo_restores(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &Saves,
    restored: &[(&str, f64)],
) -> Result<Option<String>> {
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    Ok(saved_tops(
        ctx,
        session,
        pointer,
        &saves.undone,
        restored,
        "after ONE Undo",
    )?
    .map(|why| format!("{why} The nudge was not one undo step.")))
}

/// See the module documentation.
pub struct SeveralAnnotationsNudgeAsOneStep;

impl Check for SeveralAnnotationsNudgeAsOneStep {
    fn name(&self) -> &'static str {
        "several_annotations_nudge_as_one_step"
    }

    fn defect(&self) -> &'static str {
        "an arrow key over several selected markups and links moves only the primary, or \
         nothing because the primary is locked, or moves a locked one, or takes one Undo each"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_saving(self, ctx, &ANNOTS, "mode.edit", "annot-group-nudge", annots)
    }
}

fn annots(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &Saves,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    // The locked square is selected last, so it is the primary.
    if let Some(why) = select_four(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    if let Some(why) = nudge_down(session, pointer, report, "ArrowDown", ("annots", "3/4"))? {
        return Ok(Some(why));
    }
    session.settle(20);
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE).is_none() {
        return Ok(Some(format!(
            "the locked annotation stayed and `{DISCLOSURE}` was not drawn."
        )));
    }
    let moved = [
        (SQUARE.0, SQUARE.2 - STEP),
        (CIRCLE.0, CIRCLE.2 - STEP),
        (LINK.0, LINK.2 - STEP),
        (LOCKED.0, LOCKED.2),
    ];
    if let Some(why) = saved_tops(
        ctx,
        session,
        pointer,
        &saves.moved,
        &moved,
        "after the nudge",
    )? {
        return Ok(Some(why));
    }
    let restored = [SQUARE, CIRCLE, LINK, LOCKED].map(|(key, _, top)| (key, top));
    undo_restores(ctx, session, pointer, saves, &restored)
}

/// See the module documentation.
pub struct SeveralFormFieldsNudgeAsOneStep;

impl Check for SeveralFormFieldsNudgeAsOneStep {
    fn name(&self) -> &'static str {
        "several_form_fields_nudge_as_one_step"
    }

    fn defect(&self) -> &'static str {
        "an arrow key over several selected form fields moves none of them, or only one, or \
         takes one Undo each"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_saving(self, ctx, &FIELDS, "mode.edit", "field-group-nudge", fields)
    }
}

fn fields(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    saves: &Saves,
) -> Result<Option<String>> {
    if let Some(why) = select_three(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    if let Some(why) = nudge_down(session, pointer, report, "ArrowDown", ("widgets", "3/3"))? {
        return Ok(Some(why));
    }
    let moved = [TEXT, CHECK, COMBO].map(|(key, _, top)| (key, top - STEP));
    if let Some(why) = saved_tops(
        ctx,
        session,
        pointer,
        &saves.moved,
        &moved,
        "after the nudge",
    )? {
        return Ok(Some(why));
    }
    let restored = [TEXT, CHECK, COMBO].map(|(key, _, top)| (key, top));
    undo_restores(ctx, session, pointer, saves, &restored)
}

/// See the module documentation.
pub struct ACeDimensionIsNudged;

impl Check for ACeDimensionIsNudged {
    fn name(&self) -> &'static str {
        "a_ce_dimension_is_nudged"
    }

    fn defect(&self) -> &'static str {
        "an arrow key over a selected ce dimension refuses, telling the operator to drag it, \
         although a drag and a nudge move it the same way"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, "mode.edit", "dim-nudge", dimension)
    }
}

fn dimension(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    select_by_line(ctx, session, pointer)?;
    let declined_mark = session.trace()?.mark();
    let failure = nudge_down(session, pointer, report, "ArrowRight", ("annots", "1/1"))?;
    if let Some(refused) = session
        .trace()?
        .last_after("annot-nudge-declined", declined_mark)
    // ui-text-exempt: a trace event name
    {
        return Ok(Some(format!("the nudge was refused: `{}`.", refused.raw)));
    }
    Ok(failure)
}
