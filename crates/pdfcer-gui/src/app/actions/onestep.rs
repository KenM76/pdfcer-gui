//! # `app::actions::onestep` — several actions taken back by one Undo
//!
//! [`Action::OneUndoStep`](super::Action::OneUndoStep): the actions run in
//! order through the ordinary router, then every command they pushed is
//! folded into one undo entry, labelled with the first one's kind. A step the
//! engine refuses pushes nothing and is not counted; what was applied stays
//! applied and is still folded.

use pdfcer_core::edit::MAX_UNDO_DEPTH;

use crate::app::PdfcerApp;
use crate::app::state::Status;

/// The undo stack and edit epoch before the first step.
pub(super) struct Mark {
    depth: usize,
    epoch: u64,
}

impl PdfcerApp {
    /// Where the steps start from; `None` with no document open.
    pub(super) fn undo_mark(&self) -> Option<Mark> {
        let Status::Open(doc) = &self.status else {
            return None;
        };
        Some(Mark {
            depth: doc.session.undo_depth(),
            epoch: doc.edit_epoch,
        })
    }

    /// Fold what the steps since `mark` pushed, and say so when it cannot.
    pub(super) fn fold_since(&mut self, mark: Option<Mark>) {
        let (Some(mark), Status::Open(doc)) = (mark, &mut self.status) else {
            return;
        };
        let depth = doc.session.undo_depth();
        // A full stack drops its oldest entry per push, so its depth stops
        // counting; every applied edit still bumps the epoch once.
        let pushed = if depth < MAX_UNDO_DEPTH {
            depth.saturating_sub(mark.depth)
        } else {
            usize::try_from(doc.edit_epoch.wrapping_sub(mark.epoch)).unwrap_or(usize::MAX)
        };
        let folded = pushed < 2 || {
            let kind = doc.session.undo_kinds().nth(pushed - 1);
            doc.render_worker.cancel_and_wait();
            match (kind, std::sync::Arc::get_mut(&mut doc.session)) {
                (Some(kind), Some(session)) => session.coalesce_last(pushed, kind),
                _ => false,
            }
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "one-undo-step pushed={pushed} folded={folded} undo_depth={}",
                doc.session.undo_depth()
            )
        });
        if !folded {
            crate::app::actions::disclosure::record_note(
                doc.edit_epoch,
                crate::text::arrange::takes_several_undo_presses(pushed),
            );
        }
    }
}
