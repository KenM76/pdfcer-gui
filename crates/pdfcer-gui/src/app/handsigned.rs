//! Which signature boxes carry a hand signature, read from the document.
//!
//! Contract: [`refresh`] re-measures `OpenDoc::hand_signed` whenever
//! `OpenDoc::edit_epoch` has moved since the last measurement, by reading
//! `pdfcer_core::hand_sig::hand_signatures` on every page holding an unsigned
//! `/Sig` box. The document is the record, so a signed box stays signed across
//! save and reopen, and undo or redo of a signature is measured, not tracked.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/handsign.md`.

use std::collections::BTreeSet;

use pdfcer_gui_base::handsign::SignedMark;

use crate::app::state::OpenDoc;

/// Bring `doc.hand_signed` up to `doc.edit_epoch`; a no-op when current.
pub fn refresh(ctx: &egui::Context, doc: &mut OpenDoc) {
    let epoch = doc.edit_epoch;
    if doc.hand_signed.is_current(epoch) {
        return;
    }
    let placed = crate::canvas::forms::placed(ctx, doc);
    let pages: BTreeSet<usize> = placed.unsigned.iter().map(|t| t.page).collect();
    let mut fields = Vec::new();
    let mut failed = 0usize;
    {
        let view = doc.session.view();
        for &p in &pages {
            let Some(page) = doc.pages.get(p) else {
                failed += 1;
                continue;
            };
            match pdfcer_core::hand_sig::hand_signatures(&view, page) {
                Ok(marks) => fields.extend(marks.into_iter().map(|m| SignedMark {
                    field: m.field,
                    page: p,
                    bounds: m.bounds,
                })),
                Err(_) => failed += 1,
            }
        }
    }
    doc.hand_signed.measured(epoch, fields);
    let signed = doc.hand_signed.signed_count();
    crate::diag::trace_changed("hand-signed-read", || {
        // ui-text-exempt: diagnostic trace, never displayed. No field names:
        // they are text from the operator's own document.
        format!(
            "hand-signed-read signed={signed} pages={} failed={failed}",
            pages.len()
        )
    });
}
