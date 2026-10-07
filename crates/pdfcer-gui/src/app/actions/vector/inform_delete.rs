//! # `app::actions::vector::inform_delete` — the Part and Node rung deletes
//! for content painted inside a form XObject
//!
//! The leaf-indexed twins of `DeleteSubpath`, `DeleteTextLine` and
//! `DeleteNode`. Each is one engine verb through the edit funnel, so it gets an
//! undo entry, a page-scoped cache invalidation and its disclosures surfaced
//! like every other geometry edit. A form drawn on several pages loses the
//! part on every one of them; that is the engine's `FormSurgeryOutcome`, and
//! the funnel line carries its disclosures.

use super::fold_undo;
use crate::app::actions::apply::vector_edit_on_page;
use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;
use pdfcer_core::edit::{CommandKind, EditError};

/// One subpath of an in-form path object.
pub(super) fn subpath(doc: &mut OpenDoc, page: usize, leaf: usize, subpath: usize) {
    vector_edit_on_page(doc, "delete-subpath-in-form", page, 1, |session| {
        session
            .delete_subpath_in_form(page, leaf, subpath)
            .map(|outcome| outcome.disclosures)
    });
}

/// One anchor of an in-form path object.
pub(super) fn node(doc: &mut OpenDoc, page: usize, leaf: usize, node: usize) {
    vector_edit_on_page(doc, "delete-node-in-form", page, 1, |session| {
        session
            .delete_node_in_form(page, leaf, node)
            .map(|outcome| outcome.disclosures)
    });
}

/// One visual line of an in-form text object: every show operator it is
/// written in, DESCENDING, because excising a run shifts every later run of
/// the same object. A line index past the decomposition is a stale selection
/// and does nothing.
pub(super) fn text_line(doc: &mut OpenDoc, page: usize, leaf: usize, line: usize) {
    let Some(runs) = doc
        .page_objects()
        .and_then(|provider| provider.text_line_runs_of(TargetId::Leaf(leaf as u64), line))
    else {
        return;
    };
    let pieces = runs.len();
    vector_edit_on_page(doc, "delete-text-line-in-form", page, pieces, |session| {
        let mut disclosures = Vec::new();
        for run in runs.clone().rev() {
            disclosures.extend(
                session
                    .delete_text_run_in_form(page, leaf, run)?
                    .disclosures,
            );
        }
        fold_undo(
            session,
            pieces,
            CommandKind::DeleteTextRun,
            &mut disclosures,
        );
        Ok::<_, EditError>(disclosures)
    });
}
