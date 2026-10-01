//! # `app::actions::history` — stepping the command log, in both directions
//!
//! `Direction`, its four per-direction answers, and [`history_step`] — a
//! separate file from [`super::apply`] under **R2**'s 1,500-line ceiling.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/history.md`.

use pdfcer_core::edit::{EditError, EditSession};

use super::apply::vector_edit;

use crate::app::state::OpenDoc;

/// Which end of the command log a step moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    /// Take back the most recent command — `edit.undo`, `Ctrl+Z`.
    Undo,
    /// Re-apply the most recently undone one — `edit.redo`, `Ctrl+Y` /
    /// `Ctrl+Shift+Z`.
    Redo,
}

impl Direction {
    /// What a step **would** move, without moving it.
    fn peek(self, session: &EditSession) -> Option<pdfcer_core::edit::CommandKind> {
        match self {
            Self::Undo => session.undo_kind(),
            Self::Redo => session.redo_kind(),
        }
    }

    /// Move the log by one command.
    fn step(self, session: &mut EditSession) -> Option<pdfcer_core::edit::CommandKind> {
        match self {
            Self::Undo => session.undo(),
            Self::Redo => session.redo(),
        }
    }

    /// The trace event naming the request: `undo` / `redo`.
    fn event(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Undo => "undo",
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Redo => "redo",
        }
    }

    /// [`vector_edit`]'s label — the event naming what the engine did.
    fn applied(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Undo => "undo-applied",
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Redo => "redo-applied",
        }
    }

    /// The worded decline for an empty stack.
    fn declined(self) -> crate::app::status::decline::Declined {
        use crate::app::status::decline::Declined;
        match self {
            Self::Undo => Declined::NothingToUndo,
            Self::Redo => Declined::NothingToRedo,
        }
    }
}

/// **Move the command log by one, as the document change it is.**
pub(super) fn history_step(doc: &mut OpenDoc, direction: Direction) {
    let event = direction.event();
    let Some(kind) = direction.peek(&doc.session) else {
        // Unreachable from a control and reachable from a chord. See
        // `Declined::NothingToUndo`: the QAT button is greyed by
        // `undo.available`, and `Ctrl+Z` is offered in every mode because the
        // command is on no tab, so this is the keyboard's path and it is the
        // commonest keystroke in editing. It is both traced and worded — the
        // trace for whoever reads a run from a machine they cannot see, the
        // sentence for the operator who is looking at the page rather than at
        // an 18 pt icon.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{event}-declined reason=empty-stack")
        });
        crate::app::status::decline::record_history_empty(direction.declined());
        return;
    };
    // Before the mutation, so the depth is the one the operator is acting on
    // and the kind is the one they asked to move. Both come from `peek`, which
    // reads the same slot `step` is about to pop.
    let depth = doc.session.undo_depth();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("{event} kind={kind:?} undo_depth={depth}")
    });

    // Hand signatures are followed by undo depth; see `handsign::Ledger`.
    let redo = doc.session.redo_depth();
    doc.hand_signed.reconcile(redo);
    let page = doc.view.page_index;
    vector_edit(doc, direction.applied(), page, 1, |session| {
        // `peek` answered `Some` against this same session and nothing has run
        // between then and here but `Arc::get_mut`, so `None` is unreachable.
        // It is dropped rather than unwrapped because a panic in the apply
        // phase loses the operator's document, and because the honest report of
        // a step that moved nothing is the one `vector_edit` already makes: an
        // epoch bump and an empty disclosure list.
        let _ = direction.step(session);
        // The turbofish is the price of `vector_edit`'s generic error type, and
        // it is paid here alone: this is the one caller whose closure never
        // fails, so it is the one place `E` is unconstrained. Named as the
        // engine's own error rather than as `Infallible`, because that is the
        // type every *other* verb reaching this function reports and a reader
        // comparing the arms should not have to notice a second one.
        Ok::<_, EditError>(Vec::new())
    });
    let undo = doc.session.undo_depth();
    match direction {
        Direction::Undo => doc.hand_signed.undone(undo),
        Direction::Redo => doc.hand_signed.redone(undo),
    }
}
