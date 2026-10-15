//! # `app::actions::foldeach` — one engine verb per item, folded into one undo step
//!
//! The engine moves, deletes and pastes most selectable kinds one per
//! command, so a gesture over several runs the verb per item and then
//! `coalesce_last` over what it pushed. This is that recipe, written once:
//! [`within_undo_depth`] refuses a run the undo stack cannot fold before
//! anything is applied, [`each`] runs the verb and keeps its disclosures, and
//! [`fold_each`] is both plus the fold.
//!
//! What was applied before a refusal stays applied and is still folded: the
//! only public way to take a command back is `EditSession::undo`, which leaves
//! it on the Redo stack, where Redo would re-apply a partial edit the operator
//! never saw. The caller words the partial result and an unfolded run.

use pdfcer_core::edit::{CommandKind, EditError, EditSession, MAX_UNDO_DEPTH};

/// What [`each`] does at an item the engine refuses.
#[derive(Clone, Copy)]
pub(super) enum OnRefusal {
    /// Stop: no item after it is tried.
    Stop,
    /// Try the rest.
    Continue,
}

/// What one run managed.
pub(super) struct Run<E> {
    /// Engine commands pushed: one per item applied.
    pub pushed: usize,
    /// The verbs' disclosures, each sentence once, in the order first said.
    pub told: Vec<String>,
    /// The first refusal.
    pub failure: Option<E>,
}

/// [`Run`] plus whether its commands became one undo entry.
pub(super) struct Folded<E> {
    pub run: Run<E>,
    pub folded: bool,
}

/// Refuses a gesture of `commands` engine commands that the undo stack
/// cannot hold, so cannot fold, and puts the reason on the status line.
pub(super) fn within_undo_depth(commands: usize) -> Result<(), EditError> {
    if commands > MAX_UNDO_DEPTH {
        crate::app::status::decline::record_too_many_for_one_undo(commands, MAX_UNDO_DEPTH);
        return Err(EditError::SelectionTooLargeForOneUndo {
            targets: commands,
            limit: MAX_UNDO_DEPTH,
        });
    }
    Ok(())
}

/// Runs `step` per item. `step` returns `Ok(None)` when it pushed nothing for
/// that item (a cascade already took it), and the item's disclosures when it
/// pushed one command.
pub(super) fn each<T, E>(
    session: &mut EditSession,
    items: impl IntoIterator<Item = T>,
    on_refusal: OnRefusal,
    mut step: impl FnMut(&mut EditSession, T) -> Result<Option<Vec<String>>, E>,
) -> Run<E> {
    let mut run = Run {
        pushed: 0,
        told: Vec::new(),
        failure: None,
    };
    for item in items {
        match step(session, item) {
            Ok(None) => {}
            Ok(Some(lines)) => {
                run.pushed += 1;
                for line in lines {
                    if !run.told.contains(&line) {
                        run.told.push(line);
                    }
                }
            }
            Err(e) => {
                run.failure.get_or_insert(e);
                if matches!(on_refusal, OnRefusal::Stop) {
                    break;
                }
            }
        }
    }
    run
}

/// Folds a gesture's `pieces` commands into one undo entry, keeping each of
/// `said`'s sentences once, and says so when the fold does not happen.
pub(super) fn fold_or_say(
    session: &mut EditSession,
    pieces: usize,
    kind: CommandKind,
    said: &mut Vec<String>,
) {
    let mut seen: Vec<String> = Vec::new();
    for sentence in said.drain(..) {
        if !seen.contains(&sentence) {
            seen.push(sentence);
        }
    }
    *said = seen;
    if !session.coalesce_last(pieces, kind) {
        said.push(crate::text::arrange::line_takes_several_undo_presses(
            pieces,
        ));
    }
}

/// [`within_undo_depth`] over the items, [`each`], then one `coalesce_last`
/// labelled `kind` over every command pushed.
pub(super) fn fold_each<T, E>(
    session: &mut EditSession,
    items: impl ExactSizeIterator<Item = T>,
    kind: CommandKind,
    on_refusal: OnRefusal,
    step: impl FnMut(&mut EditSession, T) -> Result<Option<Vec<String>>, E>,
) -> Result<Folded<E>, EditError> {
    within_undo_depth(items.len())?;
    let run = each(session, items, on_refusal, step);
    let folded = session.coalesce_last(run.pushed, kind);
    Ok(Folded { run, folded })
}
