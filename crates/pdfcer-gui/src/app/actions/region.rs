//! # `app::actions::region` — applying the snapshot region's commands
//!
//! Redact marks the region; Redact outside marks each page's strips around
//! its own rect through `EditSession::mark_redaction_outside`; Delete runs the
//! ordinary delete and puts the region's two counts first on the status line.
//! Nothing here removes content except Delete: marks go on apply.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/region.md`.

use pdfcer_core::annot_author::{Quad, RedactAppearance};
use pdfcer_core::edit::EditError;
use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::regionaction::RegionAction;

use super::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::text::region as t;

/// Apply one region command.
pub(super) fn apply(doc: &mut OpenDoc, action: RegionAction) {
    match action {
        RegionAction::Redact {
            page,
            rect,
            appearance,
        } => redact(doc, page, rect, &appearance),
        RegionAction::RedactOutside {
            keeps,
            skipped,
            appearance,
        } => redact_outside(doc, &keeps, skipped, &appearance),
        RegionAction::Delete {
            delete,
            deleted,
            crossing_kept,
        } => {
            let before = doc.edit_epoch;
            super::vector::apply(doc, delete);
            if doc.edit_epoch != before {
                let mut notes = vec![t::deleted(deleted, crossing_kept)];
                if let Some(d) = crate::app::actions::last_edit_disclosure(doc.edit_epoch) {
                    notes.extend(d.notes);
                }
                crate::app::actions::record_notes(doc.edit_epoch, notes);
            }
        }
    }
}

/// Mark the region as one `/Redact` mark with one quad.
fn redact(doc: &mut OpenDoc, page: usize, rect: Rect, appearance: &RedactAppearance) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "redact-mark-region-requested page={page} bbox={:.1},{:.1},{:.1},{:.1}",
            rect.llx, rect.lly, rect.urx, rect.ury
        )
    });
    let spec = appearance.to_spec(vec![Quad::from_rect(rect)]);
    vector_edit(doc, "redact-mark-region", page, 1, |session| {
        session
            .add_redaction(page, &spec)
            .map(|_| vec![t::marked_region().to_owned()])
    });
}

/// Mark everything outside each page's keep rect, as one undo step. A page
/// whose keep covers everything on it gets no mark and is counted.
fn redact_outside(
    doc: &mut OpenDoc,
    keeps: &[(usize, Rect)],
    skipped: usize,
    appearance: &RedactAppearance,
) {
    let Some(first) = keeps.first().map(|(p, _)| *p) else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "redact-mark-outside-requested pages={} first={first} skipped={skipped}",
            keeps.len()
        )
    });
    vector_edit(doc, "redact-mark-outside", first, keeps.len(), |session| {
        let (mut marked, mut nothing) = (0usize, 0usize);
        for (page, keep) in keeps {
            match session.mark_redaction_outside(*page, *keep, appearance) {
                Ok(_) => marked += 1,
                Err(EditError::RedactOutsideNothing { .. }) => nothing += 1,
                Err(e) => return Err(e),
            }
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("redact-mark-outside-marked marked={marked} nothing={nothing}")
        });
        Ok(vec![t::marked_outside(marked, nothing, skipped)])
    });
}
