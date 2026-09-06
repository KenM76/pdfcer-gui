//! # `app::actions::reorder` — putting a page's annotations in a new order
//!
//! One verb, split out of [`super::forms`] on 2026-09-02 under R2 when that file
//! crossed the 1,500-line ceiling. `OPERATOR_REQUESTS.md` O99.
//!
//! ## ★★ Why it is worth its own file rather than a shorter comment
//!
//! Because the interesting part is not the call — it is one line — but the
//! **three things the operator did not ask for** and which the engine reports so
//! they can be said. A tab order is a list of *fields*; `/Annots` order is more
//! than that, and every one of the three below is a consequence an operator
//! would not predict from the gesture they made.
//!
//! ## ★★★ Two callers, one verb, and OPPOSITE surprises — 2026-09-06
//!
//! Until today the only caller was the **form-field tab-order panel**, and the
//! three disclosures below are written for it. [`arrange`] is the second, and it
//! arrives from the other end of the same array:
//!
//! | the operator did | what they meant | what the engine reports | the surprise |
//! |---|---|---|---|
//! | dragged a row in the **tab-order** list | *this field comes second* | `non_widgets_moved` | the **drawing order** changed |
//! | pressed **Bring to front** on a mark | *draw this on top* | `moved - non_widgets_moved` | the **tab order** changed |
//!
//! ⇒ `/Annots` order is **two lists at once** — paint order for every
//! annotation and the tab sequence for the widgets among them — so whichever
//! one the operator was thinking about, the other moved. That is why the two
//! callers do not share a note set even though they share every line of the
//! call: the disclosure is *"the thing you were not looking at"*, and they were
//! looking at different things.
//!
//! ★ The second number is a **subtraction**, not a field. `AnnotsReorder`
//! reports `moved` and `non_widgets_moved`; how many widgets moved is
//! `moved - non_widgets_moved`, computed at the one call site that cares rather
//! than asked of the engine, because it is a fact about this caller's intent
//! rather than about the reorder.

use crate::app::state::OpenDoc;
use pdfcer_core::object::ObjId;

/// **Put a page's annotations in a new order** — `OPERATOR_REQUESTS.md` O99.
///
/// # ★★★ Three disclosures, and two of them are about things the operator did
/// not ask for
///
/// A tab order is a list of *fields*. `/Annots` order is more than that, and the
/// engine reports the difference rather than letting it happen quietly:
///
/// * **`non_widgets_moved`** — `/Annots` order is **paint order** for every
///   annotation, so moving a widget past a `/Link` or a markup changes which is
///   drawn on top where they overlap. The operator arranged a tab order and got
///   a z-order change; that has to be said.
/// * **`pinned`** — entries written as direct dictionaries have no object id to
///   be named by, so they cannot be moved and stay at their index while the rest
///   flow around them. A list that did not fully take, disclosed rather than
///   discovered.
/// * **`array_copied`** — the page's `/Annots` was shared with another page and
///   had to be copied first. Nothing is wrong, and it is a structural change to
///   the file that nobody asked for.
///
/// ★★ `moved == 0` is a **success with nothing to say**: the order given was the
/// order the page already had, the engine recorded no command, and there is
/// nothing to disclose. It is the common case for a drag that ends where it
/// started, and it must not read as a refusal.
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
                // ★ `moved=` beside `entries=`, because a reorder that moved
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
// ★★★ Z-ORDER — the operator's half of the same array
// ===========================================================================

/// **Where an Arrange command puts the selected mark.**
///
/// # ★★ Why all four, when only two were asked for
///
/// The brief for this work said to ship the two ends *"and consider Bring
/// forward / Send backward too if the verb supports a single-step move cheaply;
/// if it only takes a whole array, say so and ship the two that are honest."*
///
/// `EditSession::reorder_annotations` **takes a whole array** — and that is
/// exactly what makes the single step cheap rather than what forbids it. A
/// one-place move is a permutation like any other: the same list with two
/// entries exchanged. There is no per-step verb to be missing, no second engine
/// call, and no partial support to disclose. What a whole-array verb would have
/// made expensive is the *opposite* pair — a move that could not name every
/// entry — and this shell has to name them all anyway, because the engine
/// refuses a list that is not a permutation of the page's indirect entries
/// (`AnnotsNotAPermutation`) rather than silently dropping the ones a caller
/// forgot.
///
/// ⇒ So all four ship, and the honest statement is the one in this paragraph:
/// nothing about the single step is approximated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrangeTo {
    /// Last in `/Annots`, so it is painted over everything else.
    ///
    /// ★ **Last, not first.** §12.5.6 paints annotations in array order, so the
    /// *end* of the array is the top of the stack — the opposite of what "front"
    /// suggests to anyone thinking of a list. Getting this backwards is a defect
    /// that looks correct in every review and is obvious the first time a mark
    /// is arranged, which is why
    /// [`tests::front_is_the_end_of_the_array`] exists.
    Front,
    /// One place later — over the next thing it currently sits under.
    Forward,
    /// One place earlier — under the next thing it currently sits over.
    Backward,
    /// First in `/Annots`, so everything else is painted over it.
    Back,
}

impl ArrangeTo {
    /// Whether this end of the pair is the **front** — which is what
    /// [`crate::text::arrange::already_there`] needs in order to say which
    /// command the operator pressed.
    #[must_use]
    const fn toward_front(self) -> bool {
        matches!(self, Self::Front | Self::Forward)
    }
}

/// **Put one markup annotation at a new depth in its page's paint order.**
///
/// # ★★★ The order is computed HERE, at apply time, and not at the press
///
/// The obvious arrangement is for the dispatcher — which has the selection and
/// the document in front of it — to work out the new array and put it on the
/// action. It is wrong, and the reason is the action queue itself: an action is
/// raised on one frame and drained on another, with every action queued ahead of
/// it applied first. A permutation computed at the press is a permutation of the
/// `/Annots` the page had **before** whatever ran in between, and the engine
/// refuses a stale one by name (`AnnotsNotAPermutation`) rather than applying it
/// approximately.
///
/// So the action carries the **intent** — this mark, that end — and the array is
/// read from the revision the edit is actually applied to. That is the same rule
/// [`crate::app::actions::annot::AnnotAction::Move`] follows by carrying a delta
/// rather than a rectangle, and for the same reason: *a value resolved at the
/// press is a value that may have moved under you.*
///
/// # ★★ What is held still, and it is not the operator's choice
///
/// A `/TrapNet` annotation **shall be the last element** of `/Annots`
/// (ISO 32000-1 §12.5.6.21, restated §14.11.6.2 — the trap network prints after
/// everything else). The engine enforces it with `TrapNetMustStayLast` for any
/// list that tries to move it. This shell never builds such a list: the trap
/// network is lifted out of the permutation, everything else is arranged, and it
/// is put back on the end.
///
/// ⇒ A *Bring to front* on such a page therefore puts the mark in front of
/// everything the operator can see and behind one thing they cannot, and
/// [`crate::text::arrange::trap_net_stays_last`] says so. Saying nothing would
/// leave a command that visibly worked and technically did not.
///
/// **Entries with no object id are not listed at all**, which is how a caller
/// asks the engine to pin them — they are written into the page as direct
/// dictionaries, nothing can name them, and they keep the index they had while
/// the rest flow around them. That is a list which *did not fully take*, and it
/// is disclosed rather than discovered.
pub(super) fn arrange(doc: &mut OpenDoc, page: usize, id: ObjId, to: ArrangeTo) {
    let Some(order) = plan(doc, page, id, to) else {
        return;
    };
    super::apply::vector_edit(doc, "arrange-annotation", page, 1, |session| {
        session.reorder_annotations(page, &order).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                //
                // ★ `widgets=` beside `non_widgets=`, because this caller's
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
            // ★ FIRST, because it is the one that says the command may not have
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
/// Split out of [`arrange`] because the two halves are different subjects: this
/// decides *what order*, and its caller decides *what to say about the result*.
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
        // ★★ A command that changes nothing must SAY so. The engine would report
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

    /// ★★★ **Front is the END of the array.**
    ///
    /// §12.5.6 paints annotations in `/Annots` order, so the last entry is drawn
    /// last and therefore on top. "Front" and "last" are the same place, and a
    /// reader who thinks of the array as a list will reach for the opposite
    /// answer — which would compile, pass a review, and paint every *Bring to
    /// front* underneath everything.
    ///
    /// ★ Falsified: swapping the `Front` and `Back` arms in [`plan`] leaves this
    /// green and turns [`the_ends_of_the_array_are_the_ends_of_the_stack`] red,
    /// which is the division of labour intended — this one pins the *vocabulary*
    /// and that one pins the *arithmetic*.
    #[test]
    fn front_is_the_end_of_the_array() {
        assert!(ArrangeTo::Front.toward_front());
        assert!(ArrangeTo::Forward.toward_front());
        assert!(!ArrangeTo::Back.toward_front());
        assert!(!ArrangeTo::Backward.toward_front());
    }

    /// **The four destinations are four different requests.**
    ///
    /// A guard against the copy-paste that gives two variants one arm — which in
    /// this enum would be silent, because both would still produce a legal
    /// permutation.
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
    ///
    /// ★★ A free function mirroring [`plan`]'s middle, because `plan` needs an
    /// `OpenDoc` and the thing worth pinning is the index maths. This is
    /// **two statements of one rule**, which this crate usually refuses — it is
    /// accepted here for `dispatch::routes`' stated reason, that the two sit in
    /// one small file where a reader meets both at once, and because the
    /// alternative is a rule with no test at all.
    fn moved(len: usize, from: usize, to: ArrangeTo) -> usize {
        let last = len.saturating_sub(1);
        match to {
            ArrangeTo::Front => last,
            ArrangeTo::Back => 0,
            ArrangeTo::Forward => (from + 1).min(last),
            ArrangeTo::Backward => from.saturating_sub(1),
        }
    }

    /// ★★★ **The ends of the array are the ends of the stack**, and a step is
    /// one place.
    #[test]
    fn the_ends_of_the_array_are_the_ends_of_the_stack() {
        assert_eq!(moved(5, 2, ArrangeTo::Front), 4, "front is LAST");
        assert_eq!(moved(5, 2, ArrangeTo::Back), 0, "back is FIRST");
        assert_eq!(moved(5, 2, ArrangeTo::Forward), 3);
        assert_eq!(moved(5, 2, ArrangeTo::Backward), 1);
    }

    /// **Neither end runs off the array.**
    ///
    /// A *Bring forward* on the topmost mark and a *Send backward* on the
    /// bottom-most both resolve to where they already are, which is what
    /// [`plan`] then answers with a sentence rather than an edit. Without the
    /// clamps, one is an out-of-range insert and the other is an underflow —
    /// and `usize` underflow here would ask for index `usize::MAX`.
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
    // ★★★ Against a REAL document, because the arithmetic above is a
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

    /// ★★★ **Bring to front puts the mark LAST in the file**, read back from
    /// the document rather than from the plan.
    ///
    /// The oracle is `page_annotations` **after** the edit, which is not the
    /// code under test: `plan` builds a list, `EditSession::reorder_annotations`
    /// writes the array, and this reads what the array became. A `plan` that
    /// returned the ends the wrong way round would satisfy every assertion in
    /// the arithmetic tests above — they are a mirror of it — and fail here.
    ///
    /// ★ Falsified: swapping the `Front` and `Back` arms of [`plan`]'s `match`
    /// turns both halves of this red, and leaves
    /// [`the_ends_of_the_array_are_the_ends_of_the_stack`] green. That is the
    /// point of writing both.
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

    /// ★★ **A single step moves exactly one place**, and it is the claim the
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

    /// ★★★ **A mark already at the front changes nothing and SAYS so.**
    ///
    /// The sentence is the point. The engine reports `moved == 0` and
    /// [`reorder_annotations`] calls that *"a success with nothing to say"* —
    /// correctly, for a drag that ended where it started. A **command** is not a
    /// drag: the operator pressed a labelled button on purpose, and a press that
    /// neither moves anything nor says anything is indistinguishable from a
    /// broken control.
    ///
    /// ★ Falsified: deleting the `if target == from` branch in [`plan`] turns
    /// the second assertion red — the edit goes through the funnel, the engine
    /// reports `moved == 0`, and the operator is told nothing.
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
    ///
    /// Reachable after an undo or an external reload, when a selection names an
    /// object the page no longer lists. Building a permutation that silently
    /// omitted it would ask the engine to pin every entry it could not name,
    /// which is a page whose annotations quietly stop being reorderable.
    #[test]
    fn an_id_this_page_does_not_list_plans_nothing() {
        let doc = crate::app::state::open_local_fixture(FIXTURE);
        assert!(
            plan(&doc, 0, ObjId::new(9999, 0), ArrangeTo::Front).is_none(),
            "an id the page does not list cannot be permuted into it"
        );
    }
}
