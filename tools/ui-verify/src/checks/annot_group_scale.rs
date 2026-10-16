//! `several_annotations_resize_and_turn_as_one_step` — two markups selected,
//! the selection's corner grip dragged outward, then the rotate handle swung a
//! quarter turn: both change each time, as one undo step, judged in saved
//! copies for the resize.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/annot_group_scale.md`.

use super::annot_group_drag::select_members;
use super::annot_multiselect::{CIRCLE, FIXTURE, SQUARE};
use super::driving;
use super::leaftransform::point;
use super::multiselect::{Saves, after, run_saving, save, step, tops};
use super::savedannots::listed_on_page;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const RESIZED: &str = "resize-annotations-applied"; // ui-text-exempt: a trace event name
const TURNED: &str = "rotate-annotations-applied"; // ui-text-exempt: a trace event name
/// Mirrors `canvas::overlay::SELECTION_OUTLINE_REGION`.
const OUTLINE: &str = "canvas.selection-outline"; // ui-text-exempt: a trace region name
/// Mirrors `canvas::overlay::ROTATE_HANDLE_REGION`.
const ROTATE_HANDLE: &str = "canvas.rotate-handle"; // ui-text-exempt: a trace region name
/// Mirrors `panels::properties::tool::REGION_SCALE_DISTORT`.
const DISTORT: &str = "properties.tool.scale.distort"; // ui-text-exempt: a trace region name
const MODIFIERS: &str = "resize-modifiers"; // ui-text-exempt: a trace event name
/// How far the south-east grip is dragged outward, window points per axis.
const GROW_PX: f32 = 40.0;

/// See the module documentation.
pub struct SeveralAnnotationsResizeAndTurnAsOneStep;

impl Check for SeveralAnnotationsResizeAndTurnAsOneStep {
    fn name(&self) -> &'static str {
        "several_annotations_resize_and_turn_as_one_step"
    }

    fn defect(&self) -> &'static str {
        "with several markups selected, a corner grip or the rotate handle changes only the \
         primary one, or takes one Undo per markup to put back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_saving(self, ctx, &FIXTURE, "mode.edit", "annot-group-scale", drive)
    }
}

fn drive(
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
    // The fixture's markups were not drawn by pdfcer, and a drag is never
    // exactly proportional: without this switch the engine refuses both, one
    // markup or several.
    let mark = session.trace()?.mark();
    pointer.click(
        session,
        WindowPoint::centre_of(region(session, ui_rect, DISTORT)?),
    )?;
    session.settle(20);
    let switched = step!(after(
        session,
        MODIFIERS,
        mark,
        "the distortion switch was clicked"
    ));
    if switched.get("distort") != Some("true") {
        return Ok(Some(format!(
            "clicking `{DISTORT}` traced `{}`: distort=true was wanted.",
            switched.raw
        )));
    }
    if let Some(why) = select_members(ctx, session, pointer, &[CIRCLE])? {
        return Ok(Some(why));
    }
    let outline = region(session, ui_rect, OUTLINE)?;
    let mark = session.trace()?.mark();
    let grip = point(outline.max.x, outline.max.y);
    let out = point(outline.max.x + GROW_PX, outline.max.y + GROW_PX);
    pointer.drag(session, grip, out, 8)?;
    session.settle(40);
    let line = step!(after(session, RESIZED, mark, "the corner grip was dragged"));
    report.note(format!("resize: `{}`", line.raw));
    if let Some(why) = both_in_one_step(&line, "the corner grip") {
        return Ok(Some(why));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    if let Some(why) = judge(saves, report)? {
        return Ok(Some(why));
    }

    if let Some(why) = select_members(ctx, session, pointer, &[CIRCLE])? {
        return Ok(Some(why));
    }
    let outline = region(session, ui_rect, OUTLINE)?;
    let handle = region(session, ui_rect, ROTATE_HANDLE)?;
    let from = WindowPoint::centre_of(handle);
    let centre = WindowPoint::centre_of(outline);
    // Due north of the centre to due east of it: a quarter turn.
    let to = point(centre.x() + (centre.y() - from.y()), centre.y());
    let mark = session.trace()?.mark();
    pointer.drag(session, from, to, 12)?;
    session.settle(40);
    let line = step!(after(session, TURNED, mark, "the rotate handle was swung"));
    report.note(format!("rotate: `{}`", line.raw));
    Ok(both_in_one_step(&line, "the rotate handle"))
}

/// The failure sentence unless `line` says both markups changed, none was
/// left out, and the commands folded into one undo step.
fn both_in_one_step(line: &TraceLine, what: &str) -> Option<String> {
    (line.get("changed") != Some("2/2")
        || line.get("left_out") != Some("0")
        || line.get("one_step") != Some("true"))
    .then(|| {
        format!(
            "{what} over two selected markups traced `{}`: changed=2/2 left_out=0 \
             one_step=true was wanted.",
            line.raw
        )
    })
}

/// After the resize both markups are taller than after the Undo, and the Undo
/// put both tops back.
fn judge(saves: &Saves, report: &mut CheckReport) -> Result<Option<String>> {
    if let Some(why) = tops(&saves.undone, &[(SQUARE.0, SQUARE.2), (CIRCLE.0, CIRCLE.2)])? {
        return Ok(Some(format!(
            "after ONE Undo of the resize, {why} The resizes were not one undo step."
        )));
    }
    for (key, _, _) in [SQUARE, CIRCLE] {
        let (grown, back) = (height(&saves.moved, key)?, height(&saves.undone, key)?);
        report.note(format!("`{key}` height {back:.1} -> {grown:.1}"));
        if grown < back * 1.05 {
            return Ok(Some(format!(
                "after the corner drag `{key}` is {grown:.1} pt tall, and {back:.1} pt before: \
                 it did not grow with the rest of the selection."
            )));
        }
    }
    Ok(None)
}

/// The height of page 1's annotation carrying `key` in the saved `path`.
fn height(path: &std::path::Path, key: &str) -> Result<f64> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    listed_on_page(&bytes, 0)
        .into_iter()
        .find(|a| a.body.contains(key))
        .map(|a| a.rect[3] - a.rect[1])
        .ok_or_else(|| Error::new(format!("{} lists no `{key}` on page 1", path.display())))
}

fn region(session: &Session, ui_rect: &str, name: &str) -> Result<LRect> {
    driving::declared(&session.trace()?, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region with two markups selected. Trace: {}.",
            session.trace_path().display()
        ))
    })
}
