//! Several annotations deleted as one undo step: what
//! [`AnnotAction::DeleteMany`](super::super::annot::AnnotAction::DeleteMany)
//! runs.
//!
//! One `EditSession::delete_annotations` call. The engine pre-flights every
//! id, skips one a cascade already took (a pop-up going with its parent), and
//! is all-or-nothing: a refusal deletes nothing, a success is one undo entry.

use pdfcer_core::edit::{EditError, EditSession};
use pdfcer_core::object::ObjId;

use crate::app::actions::foldeach::within_undo_depth;
use crate::app::state::OpenDoc;
use crate::text::markup as t;

/// Delete `ids`, all on `page`, as one undo step. `kept_locked` counts the
/// selected annotations left out for being locked, for the status line.
pub(in crate::app::actions) fn delete_many(
    doc: &mut OpenDoc,
    page: usize,
    ids: &[ObjId],
    kept_locked: usize,
) {
    let mut deleted = 0_usize;
    super::super::apply::vector_edit(doc, "delete-annotations", page, ids.len(), |session| {
        commit(session, ids, kept_locked, &mut deleted)
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "delete-annotations-applied page={page} deleted={deleted}/{} \
             kept_locked={kept_locked} one_step={} engine=delete_annotations",
            ids.len(),
            deleted > 0
        )
    });
    // An outline around a deleted annotation promises a second Delete would
    // do something.
    doc.selection.clear_annot();
}

/// One `delete_annotations` call; `deleted` counts the ids it deleted by
/// name. `Err` with nothing deleted.
fn commit(
    session: &mut EditSession,
    ids: &[ObjId],
    kept_locked: usize,
    deleted: &mut usize,
) -> Result<Vec<String>, EditError> {
    within_undo_depth(ids.len())?;
    let gone = session.delete_annotations(ids)?;
    *deleted = gone.len();
    let popup = gone.iter().any(|g| g.popup_removed);
    let cleared = gone.iter().any(|g| g.parent_popup_cleared);
    let orphaned = gone.iter().map(|g| g.replies_orphaned).sum();
    let promoted = gone.iter().map(|g| g.group_members_promoted).sum();
    let mut said: Vec<String> = t::deleted_collateral(popup, cleared, orphaned, promoted)
        .into_iter()
        .collect();
    if kept_locked > 0 {
        said.push(t::deleted_kept_locked(gone.len(), kept_locked));
    }
    Ok(said)
}
