//! Commit `VectorAction::Restack`: move page objects in paint order with one
//! `EditSession::restack_objects` call, one undo entry
//! (`CommandKind::RestackObjects`).
//!
//! The engine answers each object's new index; the selection is moved onto
//! them so the same objects stay selected and a second press steps them again.
//! What did not move, or moved only part of the way, is said on the status line.

use pdfcer_core::vector::{RestackLimitReason, RestackOutcome, StackMove};

use crate::app::actions::reorder::ArrangeTo;
use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;
use crate::text::arrange as words;

/// Move page `page`'s `objects` (paint-order indices) toward `to`.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, objects: &[usize], to: ArrangeTo) {
    let mut landed: Option<Vec<usize>> = None;
    super::apply::vector_edit_on_page(doc, "restack-objects", page, objects.len(), |session| {
        let outcome = session.restack_objects(page, objects, stack_move(to))?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "restack-applied page={page} to={} asked={} moved={} limited={} indices={}",
                to_token(to),
                list(objects),
                list(&outcome.moved),
                outcome.limited.len(),
                list(&outcome.indices),
            )
        });
        let said = sentences(&outcome, to);
        landed = Some(outcome.indices);
        Ok::<_, pdfcer_core::edit::EditError>(said)
    });
    if let Some(indices) = landed {
        let hits: Vec<TargetId> = indices
            .into_iter()
            .map(|i| TargetId::Object(i as u64))
            .collect();
        doc.selection.marquee(page, &hits, false);
    }
}

/// The engine's name for each end.
const fn stack_move(to: ArrangeTo) -> StackMove {
    match to {
        ArrangeTo::Front => StackMove::Front,
        ArrangeTo::Forward => StackMove::Forward,
        ArrangeTo::Backward => StackMove::Backward,
        ArrangeTo::Back => StackMove::Back,
    }
}

/// What the operator is owed: why nothing moved, or which objects stopped
/// short and why.
fn sentences(outcome: &RestackOutcome, to: ArrangeTo) -> Vec<String> {
    let scoped = outcome
        .limited
        .iter()
        .filter(|l| l.reason == RestackLimitReason::Scope)
        .count();
    let unmoved = outcome.limited.len() - scoped;
    let mut said = Vec::new();
    if outcome.moved.is_empty() && outcome.limited.is_empty() {
        let all_the_way = matches!(to, ArrangeTo::Front | ArrangeTo::Back);
        said.push(words::objects_already_there(to.toward_front(), all_the_way).to_owned());
    }
    if scoped > 0 {
        said.push(words::objects_held_by_scope(scoped));
    }
    if unmoved > 0 {
        said.push(words::objects_not_moved(unmoved));
    }
    said
}

/// The trace's spelling of each end.
const fn to_token(to: ArrangeTo) -> &'static str {
    // ui-text-exempt: trace tokens, never displayed.
    match to {
        ArrangeTo::Front => "front",
        ArrangeTo::Forward => "forward",
        ArrangeTo::Backward => "backward",
        ArrangeTo::Back => "back",
    }
}

/// Indices comma-joined for the trace; `-` for none.
fn list(indices: &[usize]) -> String {
    if indices.is_empty() {
        return "-".to_owned();
    }
    indices
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
