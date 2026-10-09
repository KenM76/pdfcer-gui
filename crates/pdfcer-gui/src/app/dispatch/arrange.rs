//! # `app::dispatch::arrange` — the eight commands whose subject is a mark's
//! or a page object's DEPTH
//!
//! Page objects selected in a mode that edits content are restacked in the
//! page's paint order; otherwise a selected markup is moved in `/Annots`. The
//! chords are bound to the `markup.*` ids and reach both.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/arrange.md`.

use crate::app::PdfcerApp;
use crate::app::actions::annot::AnnotAction;
use crate::app::actions::reorder::ArrangeTo;
use crate::app::actions::{Action, VectorAction};
use crate::app::state::{OpenDoc, Status};
use crate::canvas::selection::AnnotKind;

/// Whether `id` is one of the four Arrange commands.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    destination(id).is_some()
}

/// Which end of the stack each command means.
fn destination(id: &str) -> Option<ArrangeTo> {
    // ui-text-exempt: registered command ids, never displayed.
    match id {
        "markup.bring_to_front" => Some(ArrangeTo::Front),
        "markup.bring_forward" => Some(ArrangeTo::Forward),
        "markup.send_backward" => Some(ArrangeTo::Backward),
        "markup.send_to_back" => Some(ArrangeTo::Back),
        "edit.bring_to_front" => Some(ArrangeTo::Front),
        "edit.bring_forward" => Some(ArrangeTo::Forward),
        "edit.send_backward" => Some(ArrangeTo::Backward),
        "edit.send_to_back" => Some(ArrangeTo::Back),
        _ => None,
    }
}

/// The page and the page objects an Edit ▸ Arrange command would move: the
/// selection's first page, at the Object rung, page objects only. A part of a
/// placed drawing has no verb here.
#[must_use]
pub(crate) fn restackable_objects(doc: &OpenDoc) -> Option<(usize, Vec<usize>)> {
    if doc.selection.level() != crate::canvas::selection::SelectionLevel::Object {
        return None;
    }
    let page = doc.selection.entries().first()?.page;
    let objects = doc.selection.object_indices_on(page);
    (!objects.is_empty()).then_some((page, objects))
}

/// Dispatch one of the eight.
pub(super) fn dispatch(app: &mut PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    let Some(to) = destination(id) else {
        return;
    };
    if let Status::Open(doc) = &app.status
        && app.capabilities().edit_content
        && let Some((page, objects)) = restackable_objects(doc)
    {
        actions.push(Action::Vector(VectorAction::Restack { page, objects, to }));
        return;
    }
    if id.starts_with("edit.") {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=no-page-objects-selected")
        });
        return;
    }
    if !app.capabilities().author_markup {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=mode-cannot-author-markup")
        });
        return;
    }
    let Status::Open(doc) = &app.status else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=no-document")
        });
        return;
    };
    // Rule 15, guarded by the `AnnotKind` match the compiler checks and not by
    // a `/Subtype` string. A **ce dimension** is pdfcer-authored and its depth is
    // not this verb's to change — its label and witness lines are a group, and
    // `reorder_annotations` would move the `/Line` and leave them behind. A
    // **pdf dimension** is CAD-exported page content, is not an annotation at
    // all, and cannot reach here: it has no `AnnotTarget`.
    let Some(annot) = doc
        .selection
        .annot()
        .filter(|annot| annot.target.kind == AnnotKind::Markup)
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=no-markup-selected")
        });
        return;
    };
    if annot.target.locked {
        // A sentence, not just a trace — this is the one gate an operator
        // meets having done nothing wrong, and unlike the Delete key's locked
        // refusal there is **no standing sentence on screen about it**: the
        // Properties panel says a locked mark's *appearance* cannot be changed,
        // which is a true statement about a different control.
        //
        // Recorded through `record_note` rather than `status::decline`, and
        // the distinction is that module's own: a decline reports *a gesture
        // just failed*, and the lock is a **standing property of the open
        // document** — true from the moment it was opened, true whether or not
        // anything was pressed. The epoch stamp retires it at the next real
        // edit, which is exactly right for a fact about this revision.
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::arrange::locked_cannot_arrange().to_owned(),
        );
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=annot-locked")
        });
        return;
    }
    actions.push(Action::Annot(AnnotAction::Arrange {
        page: annot.target.page,
        id: annot.target.id,
        to,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every claimed id has a destination, and no other id is claimed.**
    #[test]
    fn the_predicate_and_the_mapping_are_one_statement() {
        for id in [
            "markup.bring_to_front",
            "markup.bring_forward",
            "markup.send_backward",
            "markup.send_to_back",
            "edit.bring_to_front",
            "edit.bring_forward",
            "edit.send_backward",
            "edit.send_to_back",
        ] {
            assert!(claims(id), "{id}");
            assert!(destination(id).is_some(), "{id}");
        }
        for id in [
            "markup.rectangle",
            "markup.finish",
            "markup.comments",
            "format.delete",
            "",
        ] {
            assert!(!claims(id), "{id} is not this module's");
        }
    }

    /// **The four ids mean four different ends.**
    #[test]
    fn no_two_commands_mean_the_same_end() {
        let mut ends: Vec<String> = [
            destination("markup.bring_to_front"),
            destination("markup.bring_forward"),
            destination("markup.send_backward"),
            destination("markup.send_to_back"),
        ]
        .iter()
        .map(|end| format!("{end:?}"))
        .collect();
        let total = ends.len();
        ends.sort_unstable();
        ends.dedup();
        assert_eq!(
            ends.len(),
            total,
            "two Arrange commands share a destination"
        );
    }

    /// **The front pair really is the front pair.**
    #[test]
    fn bring_to_front_means_the_end_of_the_array() {
        assert_eq!(destination("markup.bring_to_front"), Some(ArrangeTo::Front));
        assert_eq!(destination("markup.send_to_back"), Some(ArrangeTo::Back));
        for (markup, edit) in [
            ("markup.bring_to_front", "edit.bring_to_front"),
            ("markup.bring_forward", "edit.bring_forward"),
            ("markup.send_backward", "edit.send_backward"),
            ("markup.send_to_back", "edit.send_to_back"),
        ] {
            assert_eq!(destination(markup), destination(edit), "{edit}");
        }
    }
}
