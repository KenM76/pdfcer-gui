//! # `app::actions::reorder` — putting a page's annotations in a new order
//!
//! One verb, its own file under R2 rather than a section of
//! [`super::forms`]. `OPERATOR_REQUESTS.md` O99.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/reorder.md`.

use crate::app::state::OpenDoc;
use pdfcer_core::object::ObjId;

/// **Put a page's annotations in a new order** — `OPERATOR_REQUESTS.md` O99.
pub(super) fn reorder_annotations(
    doc: &mut OpenDoc,
    page: usize,
    order: &[pdfcer_core::object::ObjId],
) {
    super::apply::vector_edit(doc, "reorder-annotations", page, 1, |session| {
        session.reorder_annotations(page, order).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                //
                // `moved=` beside `entries=`, because a reorder that moved
                // nothing and a reorder that moved everything produce the same
                // `entries` and want opposite readings.
                format!(
                    "reorder-annotations-applied page={page} entries={} moved={} \
                     non_widgets={} pinned={} copied={}",
                    outcome.entries,
                    outcome.moved,
                    outcome.non_widgets_moved,
                    outcome.pinned,
                    outcome.array_copied
                )
            });
            let mut notes = Vec::new();
            if outcome.non_widgets_moved > 0 {
                notes.push(crate::text::forms::reorder_moved_non_widgets(
                    outcome.non_widgets_moved,
                ));
            }
            if outcome.pinned > 0 {
                notes.push(crate::text::forms::reorder_pinned(outcome.pinned));
            }
            if outcome.array_copied {
                notes.push(crate::text::forms::reorder_copied_shared_array().to_owned());
            }
            notes
        })
    });
}

// ===========================================================================
// Z-ORDER — the operator's half of the same array
// ===========================================================================

pub use pdfcer_gui_base::subactions::ArrangeTo;

/// **Put one markup annotation at a new depth in its page's paint order.**
pub(super) fn arrange(doc: &mut OpenDoc, page: usize, id: ObjId, to: ArrangeTo) {
    let Some(order) = plan(doc, page, id, to) else {
        return;
    };
    super::apply::vector_edit(doc, "arrange-annotation", page, 1, |session| {
        session.reorder_annotations(page, &order).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                //
                // `widgets=` beside `non_widgets=`, because this caller's
                // disclosure is the difference and a reader of the trace should
                // not have to do the subtraction to check it.
                format!(
                    "arrange-annotation-applied page={page} to={to:?} entries={} moved={} \
                     widgets={} non_widgets={} pinned={} trap_net={} copied={}",
                    outcome.entries,
                    outcome.moved,
                    outcome.moved.saturating_sub(outcome.non_widgets_moved),
                    outcome.non_widgets_moved,
                    outcome.pinned,
                    outcome.trap_net_pinned,
                    outcome.array_copied
                )
            });
            let mut notes = Vec::new();
            // FIRST, because it is the one that says the command may not have
            // done what the operator asked. `record_notes` joins them behind one
            // lead-in and *"the first sentence is the one an operator reads if
            // they read only one"*.
            if outcome.pinned > 0 {
                notes.push(crate::text::arrange::pinned(outcome.pinned));
            }
            if outcome.trap_net_pinned {
                notes.push(crate::text::arrange::trap_net_stays_last().to_owned());
            }
            let widgets_moved = outcome.moved.saturating_sub(outcome.non_widgets_moved);
            if widgets_moved > 0 {
                notes.push(crate::text::arrange::tab_order_changed(widgets_moved));
            }
            if outcome.array_copied {
                notes.push(crate::text::arrange::copied_shared_list().to_owned());
            }
            notes
        })
    });
}

/// The list to hand the engine, or `None` when there is nothing to do and the
/// reason has already been said.
///
fn plan(doc: &OpenDoc, page: usize, id: ObjId, to: ArrangeTo) -> Option<Vec<ObjId>> {
    let page_ref = doc.pages.get(page)?;
    let all = pdfcer_core::annot::page_annotations(&doc.session.graph(), page_ref.id);

    // The two lists the permutation is built from. `page_annotations` skips null
    // and non-dictionary entries, so its positions and the raw array's diverge —
    // which is exactly why the engine takes ids and checks them rather than
    // trusting an index, and why nothing here counts positions in the file.
    let mut movable: Vec<ObjId> = Vec::with_capacity(all.len());
    let mut trap_nets: Vec<ObjId> = Vec::new();
    for annot in &all {
        let Some(annot_id) = annot.id else {
            // No id, so nothing can name it: omitting it IS the request to pin
            // it. Counted by the engine and disclosed by the caller.
            continue;
        };
        if annot.subtype == b"TrapNet" {
            trap_nets.push(annot_id);
        } else {
            movable.push(annot_id);
        }
    }

    let Some(from) = movable.iter().position(|entry| *entry == id) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("arrange-annotation-declined page={page} reason=not-listed-on-this-page")
        });
        return None;
    };

    let last = movable.len().saturating_sub(1);
    let target = match to {
        ArrangeTo::Front => last,
        ArrangeTo::Back => 0,
        ArrangeTo::Forward => (from + 1).min(last),
        ArrangeTo::Backward => from.saturating_sub(1),
    };
    if target == from {
        // A command that changes nothing must SAY so. The engine would report
        // `moved == 0` and that is *"a success with nothing to say"* for a drag
        // that ended where it started — but this operator pressed a labelled
        // button on purpose, and a press that neither moves anything nor says
        // anything is indistinguishable from a broken control. Answered before
        // the engine is called, so no command is recorded and no epoch moves.
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::arrange::already_there(to.toward_front()).to_owned(),
        );
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("arrange-annotation-declined page={page} to={to:?} reason=already-there")
        });
        return None;
    }

    let entry = movable.remove(from);
    movable.insert(target, entry);
    // The trap network goes back on the end, where §12.5.6.21 requires it.
    movable.extend(trap_nets);
    Some(movable)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Front is the END of the array.**
    #[test]
    fn front_is_the_end_of_the_array() {
        assert!(ArrangeTo::Front.toward_front());
        assert!(ArrangeTo::Forward.toward_front());
        assert!(!ArrangeTo::Back.toward_front());
        assert!(!ArrangeTo::Backward.toward_front());
    }

    /// **The four destinations are four different requests.**
    #[test]
    fn the_four_destinations_are_distinct() {
        let all = [
            ArrangeTo::Front,
            ArrangeTo::Forward,
            ArrangeTo::Backward,
            ArrangeTo::Back,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    /// The permutation arithmetic, without a document.
    fn moved(len: usize, from: usize, to: ArrangeTo) -> usize {
        let last = len.saturating_sub(1);
        match to {
            ArrangeTo::Front => last,
            ArrangeTo::Back => 0,
            ArrangeTo::Forward => (from + 1).min(last),
            ArrangeTo::Backward => from.saturating_sub(1),
        }
    }

    /// **The ends of the array are the ends of the stack**, and a step is
    /// one place.
    #[test]
    fn the_ends_of_the_array_are_the_ends_of_the_stack() {
        assert_eq!(moved(5, 2, ArrangeTo::Front), 4, "front is LAST");
        assert_eq!(moved(5, 2, ArrangeTo::Back), 0, "back is FIRST");
        assert_eq!(moved(5, 2, ArrangeTo::Forward), 3);
        assert_eq!(moved(5, 2, ArrangeTo::Backward), 1);
    }

    /// **Neither end runs off the array.**
    #[test]
    fn a_step_past_either_end_stays_where_it_is() {
        assert_eq!(moved(5, 4, ArrangeTo::Forward), 4);
        assert_eq!(moved(5, 0, ArrangeTo::Backward), 0);
        assert_eq!(moved(5, 4, ArrangeTo::Front), 4);
        assert_eq!(moved(5, 0, ArrangeTo::Back), 0);
        // …and the degenerate page: one annotation, or none.
        assert_eq!(moved(1, 0, ArrangeTo::Front), 0);
        assert_eq!(moved(0, 0, ArrangeTo::Front), 0);
    }
    // -----------------------------------------------------------------
    // Against a REAL document, because the arithmetic above is a
    // mirror and a mirror cannot catch the two halves disagreeing
    // -----------------------------------------------------------------

    /// The fixture. Three annotations on one page — `/Square`, `/Text`,
    /// `/FreeText` — all indirect, which is what a permutation is expressed
    /// over. `tools/gen-annots-with-everything-fixture.py` carries the argument
    /// for every key in it.
    const FIXTURE: &str = "annots-with-everything.pdf";

    /// The fixture's page-0 annotations, in `/Annots` order, by object id.
    fn ids(doc: &OpenDoc) -> Vec<ObjId> {
        let page = doc.pages.first().expect("the fixture has a page");
        pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id)
            .iter()
            .filter_map(|annot| annot.id)
            .collect()
    }

    /// **Bring to front puts the mark LAST in the file**, read back from
    /// the document rather than from the plan.
    #[test]
    fn bring_to_front_puts_the_mark_last_in_the_file() {
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let before = ids(&doc);
        assert!(
            before.len() >= 3,
            "the fixture must hold enough annotations for an order to be wrong: {before:?}"
        );
        let first = before[0];
        arrange(&mut doc, 0, first, ArrangeTo::Front);
        let after = ids(&doc);
        assert_eq!(
            after.last(),
            Some(&first),
            "front is the END of /Annots, because §12.5.6 paints in array order"
        );
        assert_eq!(
            after.len(),
            before.len(),
            "a reorder never adds or drops an entry"
        );
        let mut sorted_before = before.clone();
        let mut sorted_after = after.clone();
        sorted_before.sort_unstable_by_key(|id| (id.num, id.generation));
        sorted_after.sort_unstable_by_key(|id| (id.num, id.generation));
        assert_eq!(
            sorted_before, sorted_after,
            "the new order must be a PERMUTATION — the engine refuses anything else by name"
        );
    }

    /// **Send to back puts the mark first**, read back the same way.
    #[test]
    fn send_to_back_puts_the_mark_first_in_the_file() {
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let before = ids(&doc);
        let last = *before.last().expect("the fixture has annotations");
        arrange(&mut doc, 0, last, ArrangeTo::Back);
        let after = ids(&doc);
        assert_eq!(after.first(), Some(&last), "back is the START of /Annots");
    }

    /// **A single step moves exactly one place**, and it is the claim the
    /// brief asked to be honest about: a whole-array verb does not make a
    /// one-place move approximate, it makes it a permutation like any other.
    #[test]
    fn a_single_step_moves_exactly_one_place() {
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let before = ids(&doc);
        let middle = before[1];
        arrange(&mut doc, 0, middle, ArrangeTo::Forward);
        let after = ids(&doc);
        assert_eq!(
            after.iter().position(|id| *id == middle),
            Some(2),
            "Bring forward moves one place, not to the end: {before:?} -> {after:?}"
        );
        // …and the two it stepped over are otherwise undisturbed.
        assert_eq!(after[0], before[0], "nothing below it moved");
        assert_eq!(after[1], before[2], "exactly one entry swapped past it");
    }

    /// **A mark already at the front changes nothing and SAYS so.**
    #[test]
    fn a_mark_already_at_the_front_is_told_so() {
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let before = ids(&doc);
        let last = *before.last().expect("the fixture has annotations");
        let epoch = doc.edit_epoch;
        arrange(&mut doc, 0, last, ArrangeTo::Front);
        assert_eq!(ids(&doc), before, "nothing moved, so nothing changed");
        assert_eq!(
            doc.edit_epoch, epoch,
            "a command that changes nothing must not enter the undo log"
        );
        let said = crate::app::actions::last_edit_disclosure(epoch)
            .expect("a press that did nothing owes a sentence");
        assert!(
            said.notes
                .iter()
                .any(|note| note == crate::text::arrange::already_there(true)),
            "and the sentence must name WHICH end: {:?}",
            said.notes
        );
    }

    /// **A mark that is not on the page it claims is refused, not guessed at.**
    #[test]
    fn an_id_this_page_does_not_list_plans_nothing() {
        let doc = crate::app::state::open_local_fixture(FIXTURE);
        assert!(
            plan(&doc, 0, ObjId::new(9999, 0), ArrangeTo::Front).is_none(),
            "an id the page does not list cannot be permuted into it"
        );
    }
}
