//! # `app::dispatch::arrange` — the four commands whose subject is a mark's
//! DEPTH
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/arrange.md`.

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::actions::reorder::ArrangeTo;
use crate::app::state::Status;
use crate::canvas::selection::AnnotKind;

/// Whether `id` is one of the four Arrange commands.
///
/// Named `claims` rather than `handles` for [`super::markupnodes::claims`]'
/// stated reason: `shell::commands::reach::guards::EVALUATED_GUARDS` is a set of
/// **function names** read out of `dispatch.rs`'s syntax tree, `claims` is
/// already in it, and the register's own note blesses two guards sharing a name.
/// One line in `guard_claiming` and no change to that list.
///
/// Paired with [`destination`] rather than with a second `match` in
/// [`dispatch`], which is `dispatch::routes`' improvement on the
/// membership-test shape: the two statements that could grow apart are **one
/// statement**, so an id this predicate claims and that function cannot place is
/// unrepresentable.
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
        _ => None,
    }
}

/// Dispatch one of the four.
///
/// # Three gates, and only one of them can be met by an operator who did
/// nothing wrong
///
/// | gate | reachable from the ribbon? | how it reports |
/// |---|---|---|
/// | the mode may not author markup | **no** — the Markup tab is not shown in Read | trace only |
/// | no markup is selected | **no** — the group is not drawn (`selection.markup_restylable`) | trace only |
/// | the mark is **locked** | **yes** — the group is drawn and the mark is selected | a sentence on the status row |
///
/// The first two are belt to the ribbon's braces: a customized manifest or a
/// chord reaches any command from any state, so they are written rather than
/// assumed — the same *"push the chord blind, gate the effect in dispatch"* rule
/// every arm in `super` follows. Neither owes the operator a sentence, because
/// neither can happen to one.
///
/// The **lock** can, and does: `selection.markup_restylable` deliberately
/// excludes the lock (§12.5.3 bit 8 is a fact about one annotation, not about
/// the build or the mode), so the four controls are live on a locked mark and
/// pressing one has to say why it did nothing.
pub(super) fn dispatch(app: &mut PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    let Some(to) = destination(id) else {
        return;
    };
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
    }
}
