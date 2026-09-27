//! # `app::actions::annots::flatten` — make one markup part of the page
//!
//! Contract: [`flatten`] is one `EditSession::flatten_annotations` call for one
//! annotation, one undo entry, its sentences on the status bar. [`refusal`] is
//! the engine's per-annotation answer the right-click menu asks before it
//! offers the row, so a row is only drawn where the verb will accept it.

use pdfcer_core::edit::{AnnotFlattenRefusal, AnnotFlattenRefusalReason, EditError};
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;

/// Burn annotation `id` on `page` into the page and remove it.
pub(in crate::app::actions) fn flatten(doc: &mut OpenDoc, page: usize, id: ObjId) {
    super::super::apply::vector_edit(doc, "flatten-annotation", page, 1, |session| {
        let out = session
            .flatten_annotations(page, Some(&[id]))
            .inspect_err(|e| {
                if let EditError::AnnotationNotFlattenable { reason, .. } = e {
                    crate::app::status::decline::record_markup_flatten(*reason);
                }
            })?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "annotation-flattened id={}_{} changed={} flattened={} grouped={} layered={} popups={} replies={} skipped={}",
                id.num,
                id.generation,
                out.changed,
                out.flattened,
                out.grouped,
                out.layered,
                out.popups_removed,
                out.replies_unlinked,
                out.skipped.len()
            )
        });
        Ok::<_, EditError>(if out.changed {
            crate::text::flattenannot::flattened(&out)
        } else {
            vec![crate::text::flattenannot::nothing_flattened().to_owned()]
        })
    });
    doc.selection.clear_annot();
}

/// Burn every markup on `page` the engine will burn; what it leaves is named.
pub(in crate::app::actions) fn flatten_page(doc: &mut OpenDoc, page: usize) {
    super::super::apply::vector_edit(doc, "flatten-page-annotations", page, 1, |session| {
        let out = session.flatten_annotations(page, None)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "annotation-flattened page={page} changed={} flattened={} grouped={} layered={} popups={} replies={} skipped={}",
                out.changed,
                out.flattened,
                out.grouped,
                out.layered,
                out.popups_removed,
                out.replies_unlinked,
                out.skipped.len()
            )
        });
        let mut lines = if out.changed {
            crate::text::flattenannot::flattened(&out)
        } else {
            vec![crate::text::flattenannot::nothing_flattened().to_owned()]
        };
        lines.extend(crate::text::flattenannot::kept(
            out.skipped
                .iter()
                .map(|AnnotFlattenRefusal { reason, .. }| *reason),
        ));
        Ok::<_, EditError>(lines)
    });
    doc.selection.clear_annot();
}

/// Why the engine would refuse to flatten `id` on `page`, or `None` when it
/// would accept. A document-level refusal (encrypted, certified) counts as a
/// refusal with no per-annotation reason.
pub(crate) fn refusal(
    doc: &OpenDoc,
    page: usize,
    id: ObjId,
) -> Option<Option<AnnotFlattenRefusalReason>> {
    if doc.session.flatten_refusal().is_some() {
        return Some(None);
    }
    match doc.session.annotation_flatten_refusals(page) {
        // The subtype is the engine's record of what it refused; the menu
        // only needs whether and why.
        Ok(refused) => refused.into_iter().find_map(
            |AnnotFlattenRefusal {
                 id: refused_id,
                 subtype: _,
                 reason,
                 ..
             }| (refused_id == Some(id)).then_some(Some(reason)),
        ),
        Err(_) => Some(None),
    }
}
