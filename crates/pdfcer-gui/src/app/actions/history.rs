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
///
/// An enum rather than a `bool`: `history_step(doc, true)` at a call site says
/// nothing, and the two call sites are one line apart in the same `match`,
/// which is exactly the distance at which a transposition survives review.
///
/// **Named `Direction` rather than `History`** because
/// `crate::app::status::decline::History` already means something else one
/// module away — *what the two stacks currently hold* — and two types with one
/// name in one crate is a grep that answers the wrong question. This one is a
/// direction of travel; that one is a state.
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
    ///
    /// `EditSession::undo_kind`/`redo_kind`, which take `&self` — so this is
    /// askable before the render worker is stopped and before `Arc::get_mut` is
    /// attempted, which is what lets an empty stack be declined without paying
    /// for a cancelled raster. `None` is the empty stack, and it is the same
    /// answer `can_undo`/`can_redo` give, from the same field.
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
    ///
    /// Distinct from [`Self::event`] on purpose, and it is the same two-line
    /// vocabulary `markup-commit` / `add-markup` already uses: the first line is
    /// **the shell decided**, and carries the `CommandKind`; the second is
    /// **the engine did it**, and carries the epoch. A harness that wants to
    /// know whether the caches were invalidated reads the second, and a harness
    /// that wants to know what the operator took back reads the first.
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
///
/// The whole of [`Action::Undo`] and [`Action::Redo`].
///
/// # ★ Why this goes through [`vector_edit`] rather than doing the four steps
/// itself
///
/// Because an undo **is** an edit, and the only thing that distinguishes it
/// from a delete or a markup is which engine verb runs in step 2. Every reason
/// the protocol exists applies here unchanged:
///
/// | step | why an undo needs it |
/// |---|---|
/// | cancel the render worker | `EditSession::undo` takes `&mut self`, and `OpenDoc::session` is an `Arc` a rasterizing worker holds a clone of. Without the cancel, `Arc::get_mut` returns `None` **whenever the page happens to be rendering** — an undo that works or does not depending on how fast the sheet drew |
/// | mutate through `Arc::get_mut` | the same soundness argument, from the other end |
/// | bump `edit_epoch` | ★ **the step that makes the undo visible.** Every epoch-keyed cache — the page decomposition, the page-text extraction, the font inventory, the canvas selection's resolution, the Objects panel's count — believes it still describes the document until this moves. An undo that skipped it would restore the bytes and leave the operator looking at the state they just took back |
/// | drop the cached texture | `settle_and_rasterize` keys the page texture on the page index and the raster scale, and an undo changes neither, so nothing else would notice. This is what re-rasters the page |
///
/// Writing those four again here would be the fifth hand-written copy of a
/// protocol whose entire reason for existing is that hand-written copies omit
/// steps. The rule is one choke point.
///
/// # The disclosure list is empty, and that is a statement
///
/// The vector verbs return prose when the surgery had to change an operator's
/// *form* to express their request. An undo restores recorded `before` values —
/// it changes no form that was not already changed and disclosed when the
/// original command ran — so there is nothing new to disclose, and the empty
/// list makes [`vector_edit`] drop the **previous** edit's sentence, which is
/// exactly right: that sentence described a revision the operator has just left.
///
/// # Why the empty stack is checked HERE and not in the dispatcher
///
/// The dispatcher's arms route, and routing is all they do. This function has
/// to ask the session what is on the log before it can act anyway, so asking in
/// both places would be two spellings of one question — and the one that
/// drifted
/// would produce a control that is greyed while the bar says something else.
/// The decline is recorded through `crate::app::status::decline`, in the apply
/// phase, exactly as `crate::app::save`'s failure is and for the reason that
/// module's own call site documents: `decline::retire` runs at the *top* of
/// `dispatch_command`, so a sentence recorded here survives the frame that
/// raised it.
///
/// # What `page=` means on the trace line, and why it is not a lie
///
/// [`vector_edit`]'s `page` and `operands` exist so the trace can say which
/// verb ran over what. An undo is the one caller whose operands are **not**
/// page-scoped: a `CommandKind` may be a page rotation, a document-level
/// attachment or a form field, and the engine's command log does not carry a
/// page at all. What is passed is therefore the page **on screen**, and the
/// `undo` line above it carries the `CommandKind`, which is the field that
/// actually says what moved. A reader who wants the page an undo touched has to
/// read the kind; there is no honest number to put here, and inventing a
/// sentinel would be a second thing to explain.
pub(super) fn history_step(doc: &mut OpenDoc, direction: Direction) {
    let event = direction.event();
    let Some(kind) = direction.peek(&doc.session) else {
        // ★ Unreachable from a control and reachable from a chord. See
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
}
