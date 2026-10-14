//! Several annotations deleted as one undo step: what
//! [`AnnotAction::DeleteMany`](super::super::annot::AnnotAction::DeleteMany)
//! runs.
//!
//! The engine deletes one annotation per command and its only batch delete,
//! `cut_selection`, refuses what a paste could not put back, which Delete has
//! no reason to. So this is `cut_selection`'s own recipe spelled outside the
//! engine (G191): pre-flight every id with `annotation_deletion_preview` so a
//! refusal deletes nothing, delete each id a cascade has not already taken,
//! then `coalesce_last` over the commands pushed.

use pdfcer_core::edit::{CommandKind, EditError, EditSession};
use pdfcer_core::object::ObjId;

use crate::app::actions::foldeach::{OnRefusal, fold_each, within_undo_depth};
use crate::app::state::OpenDoc;
use crate::text::markup as t;

/// What [`commit`] managed.
#[derive(Default)]
struct Report {
    deleted: usize,
    folded: bool,
}

/// Delete `ids`, all on `page`, as one undo step. `kept_locked` counts the
/// selected annotations left out for being locked, for the status line.
pub(in crate::app::actions) fn delete_many(
    doc: &mut OpenDoc,
    page: usize,
    ids: &[ObjId],
    kept_locked: usize,
) {
    let mut report = Report::default();
    super::super::apply::vector_edit(doc, "delete-annotations", page, ids.len(), |session| {
        commit(session, ids, kept_locked, &mut report)
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "delete-annotations-applied page={page} deleted={}/{} kept_locked={kept_locked} \
             one_step={}",
            report.deleted,
            ids.len(),
            report.folded
        )
    });
    // An outline around a deleted annotation promises a second Delete would
    // do something.
    doc.selection.clear_annot();
}

/// Pre-flight, delete, fold. `Err` before anything is deleted, or from a
/// verb refusing after the pre-flight passed, in which case what was deleted
/// is still folded and counted in `report`.
fn commit(
    session: &mut EditSession,
    ids: &[ObjId],
    kept_locked: usize,
    report: &mut Report,
) -> Result<Vec<String>, EditError> {
    within_undo_depth(ids.len())?;
    for &id in ids {
        session.annotation_deletion_preview(id)?;
    }
    let (mut popup, mut cleared, mut orphaned, mut promoted) = (false, false, 0, 0);
    let done = fold_each(
        session,
        ids.iter().copied(),
        CommandKind::DeleteAnnotation,
        OnRefusal::Stop,
        |session, id| {
            // A cascade (a pop-up going with its parent) may already have taken it.
            if session.value(id).is_none() {
                return Ok(None);
            }
            let gone = session.delete_annotation(id)?;
            popup |= gone.popup_removed;
            cleared |= gone.parent_popup_cleared;
            orphaned += gone.replies_orphaned;
            promoted += gone.group_members_promoted;
            Ok::<_, EditError>(Some(Vec::new()))
        },
    )?;
    let (pushed, folded) = (done.run.pushed, done.folded);
    *report = Report {
        deleted: pushed,
        folded,
    };
    if let Some(e) = done.run.failure {
        return Err(e);
    }
    let mut said: Vec<String> = t::deleted_collateral(popup, cleared, orphaned, promoted)
        .into_iter()
        .collect();
    if kept_locked > 0 {
        said.push(t::deleted_kept_locked(pushed, kept_locked));
    }
    if !folded {
        said.push(t::deleted_not_one_undo().to_owned());
    }
    Ok(said)
}
