//! `several_annotations_delete_as_one_undo_step` — markups, a link and a
//! locked markup selected by Shift-click, deleted by one press of Delete, the
//! locked one kept and said so, and all of it brought back by one Undo,
//! judged in saved copies.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/annot_delete_many.md`.

use super::annot_multiselect::{CIRCLE, FIXTURE, LINK, SQUARE};
use super::dimdrive::{click_on, launch_with};
use super::driving;
use super::multiselect::{after, save, shift_click, step};
use super::savedannots::listed_annots;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The locked square: its key and a point on it.
const LOCKED: (&str, (f64, f64)) = ("/NM (LO)", (50.0, 35.0)); // ui-text-exempt: a PDF key
const DIMENSION_KEY: &str = "/LineDimension"; // ui-text-exempt: a PDF key
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const TOGGLED: &str = "annot-toggled"; // ui-text-exempt: a trace event name
const APPLIED: &str = "delete-annotations-applied"; // ui-text-exempt: a trace event name
const DISCLOSURE: &str = "status-group:edit-disclosure"; // ui-text-exempt: a trace region name

/// See the module documentation.
pub struct SeveralAnnotationsDeleteAsOneUndoStep;

impl Check for SeveralAnnotationsDeleteAsOneUndoStep {
    fn name(&self) -> &'static str {
        "several_annotations_delete_as_one_undo_step"
    }

    fn defect(&self) -> &'static str {
        "Delete with several markups and links selected deletes only the one clicked first, or \
         takes one Undo each to bring them back, or deletes a locked one, or keeps it without \
         saying so"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let deleted = ctx.out("annot-delete-many.deleted.pdf");
        let undone = ctx.out("annot-delete-many.undone.pdf");
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
            "annot-delete-many",
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
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE).is_some() {
        return Ok(Some(format!(
            "`{DISCLOSURE}` was on screen before anything was done, so the delete's own \
             sentence could not be told from it."
        )));
    }

    // --- the square by a click; the circle, the link and the locked square
    //     by Shift-click ----------------------------------------------------
    click_on(ctx, session, pointer, &FIXTURE, 0, SQUARE.1)?;
    session.settle(20);
    for (n, (key, at)) in [(CIRCLE.0, CIRCLE.1), (LINK.0, LINK.1), LOCKED]
        .into_iter()
        .enumerate()
    {
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

    // --- one Delete: three go, the locked one stays, one step, said ------
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Delete", None)?;
    session.settle(40);
    let applied = step!(after(session, APPLIED, mark, "Delete was pressed"));
    report.note(format!("`{}`", applied.raw));
    if applied.get("deleted") != Some("3/3")
        || applied.get("kept_locked") != Some("1")
        || applied.get("one_step") != Some("true")
    {
        return Ok(Some(format!(
            "Delete over four annotations, one locked, traced `{}`: three deleted, one kept \
             and one undo step was wanted.",
            applied.raw
        )));
    }
    session.settle(20);
    if driving::declared(&session.trace()?, ui_rect, DISCLOSURE).is_none() {
        return Ok(Some(format!(
            "the locked annotation was kept and `{DISCLOSURE}` was not drawn: the operator \
             is not told why it is still there."
        )));
    }
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let gone = [
        (SQUARE.0, false),
        (CIRCLE.0, false),
        (LINK.0, false),
        (LOCKED.0, true),
        (DIMENSION_KEY, true),
    ];
    if let Some(why) = listed(deleted, &gone)? {
        return Ok(Some(format!("after Delete, {why}")));
    }

    // --- one Undo brings all three back -----------------------------------
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    if let Some(why) = save(ctx, session, pointer)? {
        return Ok(Some(why));
    }
    let back = gone.map(|(key, _)| (key, true));
    if let Some(why) = listed(undone, &back)? {
        return Ok(Some(format!(
            "after ONE Undo of the delete, {why} The deletes were not one undo step."
        )));
    }
    Ok(None)
}

/// The failure sentence when an annotation is on the saved page's `/Annots`
/// and should not be, or the reverse.
fn listed(path: &std::path::Path, want: &[(&str, bool)]) -> Result<Option<String>> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    let annots = listed_annots(&bytes);
    for (key, present) in want {
        let found = annots.iter().any(|a| a.body.contains(key));
        if found != *present {
            let state = if found { "still lists" } else { "lists no" };
            return Ok(Some(format!(
                "{} {state} annotation with `{key}`.",
                path.display()
            )));
        }
    }
    Ok(None)
}
