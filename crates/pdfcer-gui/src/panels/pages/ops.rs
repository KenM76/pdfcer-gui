//! # `panels::pages::ops` — what a page verb acts on, and what a move means
//!
//! The **rules** behind the six page verbs, with no `egui`, no document and no
//! engine call anywhere in the file. Two questions, both of which have exactly
//! one right answer and both of which were previously unanswered because
//! nothing asked them:
//!
//! 1. **What is the operand?** — [`operands`]. The Pages panel's multi-select
//!    when there is one, the current page when there is not.
//! 2. **What does "move up" mean for a set?** — [`move_order`]. A permutation
//!    of `0..page_count`, or a refusal, and the refusal is the interesting half.
//!
//! ## ★ Why this is a module and not two `if`s in a dispatch arm
//!
//! `crate::app::dispatch`'s header states the standing rule — *"the arms route;
//! they do not compute"* — and both of these are computations that can be wrong
//! in ways an operator would notice and a compiler would not. The move
//! permutation in particular is the kind of small index arithmetic that looks
//! obviously right and is off by one at the boundary: `EditSession::reorder_pages`
//! refuses anything that is not a permutation of `0..count`, so a bug here is a
//! verb that silently declines rather than one that mis-orders, which is the
//! quieter and therefore worse failure.
//!
//! Pure, so every one of those boundaries is asserted headlessly below. That is
//! `crate::viewer`'s standing split — *"this module is unit-testable and the
//! widget code is not"* — applied to the one part of a page verb that carries a
//! rule.
//!
//! ## ★ The operand rule, and why it is not "the selection"
//!
//! [`crate::panels::PanelsState::selected_pages`]' own documentation already
//! settled this before anything read it:
//!
//! > Empty is a defined answer, not a missing one: with nothing picked those
//! > commands act on the current page.
//!
//! and `crate::shell::commands`' Pages band says the same from the registry's
//! side — the verbs *"respect the thumbnail rail's selection when there is
//! one"*, and with none they *"act on the current page, which is a defined
//! answer and not a disabled state."* That is why none of the `pages.*`
//! commands is gated on a selection condition and why adding one would be
//! wrong: a rotate with nothing picked is not a refused command, it is a
//! rotate of the sheet the operator is looking at.
//!
//! [`operands`] is the single place that rule is written down, so the six verbs
//! cannot come to disagree about what they act on — the same argument
//! `crate::canvas::selection::SelectionState::deletable_objects_on` makes for
//! the object verbs, and it is made here for the same reason: two statements of
//! a destructive rule is one too many.
//!
//! ## ★ The move rule, in one table
//!
//! `pages` is the operand list, `n` the page count, and the result is what
//! `new_order[i] = ` for each new position `i`.
//!
//! | operand | `n` | up | down |
//! |---|---|---|---|
//! | `{1}` | 4 | `[1,0,2,3]` | `[0,2,1,3]` |
//! | `{1,2}` | 4 | `[1,2,0,3]` | `[0,3,1,2]` |
//! | `{1,3}` | 4 | `[1,0,3,2]` | `[0,2,1,3]` |
//! | `{0}` | 4 | **refused** | `[1,0,2,3]` |
//! | `{0,1}` | 4 | **refused** | `[2,0,1,3]` |
//! | anything | 1 | **refused** | **refused** |
//!
//! A **non-contiguous** selection moves as separate items, each by one, rather
//! than being collapsed into a block. That is what every list control with
//! reorder arrows does, and the alternative — gathering the set together at the
//! topmost member — would silently reorder pages the operator did not name.
//!
//! ## ★ Why a blocked move is a refusal rather than a no-op
//!
//! `EditSession::reorder_pages` would *accept* the identity permutation and
//! return `Ok(())` having recorded nothing, so handing it one would be
//! harmless. It would also be silent, and a control the operator pressed that
//! produced no change and no sentence is the defect class this project is named
//! after. So [`move_order`] returns [`Err`] for a move that cannot happen, and
//! the engine is never asked a question whose answer is "nothing".
//!
//! The two blocked cases are different facts and are reported as two, because
//! the remedy differs: *these sheets are already at the top* is a boundary the
//! operator can fix by picking a different one, while *there is only one page*
//! is about the document and cannot be fixed at all.
//!
//! ## ★ What the caller does with a refusal today, and what it should do
//!
//! `crate::app::dispatch` **traces** it, with the variant's name as the reason
//! token — `command-declined id=pages.move_up reason=at-the-edge` — and does
//! not word it in the status bar.
//!
//! That is a scope statement rather than a judgement that it should stay so.
//! The surface for a worded decline is `crate::app::status::decline`, whose
//! `Declined` enum was being extended by concurrent undo/redo work while this
//! landed; adding variants to a type another session is mid-rewrite on is how
//! two sessions produce one broken file. The two refusals carry **distinct**
//! tokens precisely so that follow-up is a mapping rather than an
//! investigation.
//!
//! The one thing to know before taking it: `Declined::still_true` re-asks the
//! predicate that produced a decline, and the predicate here — *are the picked
//! sheets at the edge?* — depends on the **Pages panel's** state, which lives
//! on `PdfcerApp::panels` and not on the `&OpenDoc` that function is handed. So
//! either the sentence retires only on the operator's next *command* (which is
//! `Declined::SaveFailed`'s documented answer, and leaves the sentence stale
//! after a mere click on a different thumbnail), or `live`'s plumbing grows a
//! third source. That is a real decision, not a line of wiring, which is the
//! other half of why it is not taken here.

use std::collections::BTreeSet;

/// Which way a reorder verb moves its operand.
///
/// A two-variant enum rather than a signed step, because the two directions do
/// **not** share an implementation — the up rule scans ascending and pins from
/// the top, the down rule scans descending and pins from the bottom — and a
/// `delta: i32` would invite a single body with a sign flip in it that is
/// correct in one direction and off by one in the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    /// Toward page 1. `pages.move_up`.
    Up,
    /// Toward the last page. `pages.move_down`.
    Down,
}

/// Why a reorder verb produced no permutation.
///
/// Returned rather than folded into a bare `None` so the caller can say *which*
/// nothing happened — the same argument `crate::app::dispatch` makes for
/// tracing `no-circle-fit-to-finish` separately from
/// `mode-cannot-author-measure`. A reader of a trace from a machine they cannot
/// see should not have to guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveRefusal {
    /// Every operand is already against the edge it was asked to move toward.
    ///
    /// Reachable with one page picked (page 1, moved up) and with several (a
    /// run that already starts at page 1). Both are the same fact from the
    /// operator's side: *these sheets are already as far that way as they go*.
    AtTheEdge,
    /// There is nothing to reorder — no operand, or a document with fewer than
    /// two pages.
    ///
    /// Distinct from [`Self::AtTheEdge`] because the remedy is different: one
    /// is fixed by picking a different sheet, the other cannot be fixed at all.
    NothingToMove,
}

/// **The pages a `pages.*` verb acts on.**
///
/// The panel's multi-select when it has one, the current page when it does not.
/// See the module header for why that is the rule and where it was written down
/// before anything read it.
///
/// Every index is checked against `page_count`, so a selection that has not yet
/// been reconciled with a shrunken document cannot hand the engine an operand
/// it would refuse the whole batch over. That is belt to
/// [`super::select::PageSelection::retain_below`]'s braces: the panel clamps on
/// its next frame, and a verb invoked from a **keyboard chord** can arrive
/// before that frame has been drawn.
///
/// # Returns
///
/// Ascending, de-duplicated (both by construction, from a `BTreeSet`) and
/// in-range. **Empty** when the document has no pages at all, which is a legal
/// PDF (`/Count 0`) and is why the callers check rather than assume — although
/// the commands are additionally gated on `doc.pages`, so an empty return is
/// reachable only through a customized keymap.
#[must_use]
pub fn operands(selected: &BTreeSet<usize>, current: usize, page_count: usize) -> Vec<usize> {
    if page_count == 0 {
        return Vec::new();
    }
    let picked: Vec<usize> = selected
        .iter()
        .copied()
        .filter(|p| *p < page_count)
        .collect();
    if picked.is_empty() {
        // The current page, clamped. `ViewState::page_index` is kept in range
        // by `go_to_page`, so the clamp is unreachable in a shipped build and
        // is here because "unreachable" and "checked" cost the same.
        return vec![current.min(page_count - 1)];
    }
    picked
}

/// **The permutation a move verb asks `EditSession::reorder_pages` for.**
///
/// `pages` is [`operands`]' output — ascending, de-duplicated, in range.
/// `page_count` is the document's current length.
///
/// # Returns
///
/// `Ok(order)` where `order[i]` is the **current** 0-based index of the page
/// that should end up at position `i`, which is `reorder_pages`' contract
/// verbatim. It is a permutation of `0..page_count` by construction — the
/// vector starts as the identity and is only ever mutated by
/// [`slice::swap`] — which matters because the engine refuses anything else
/// with [`EditError::NotAPermutation`], and a refusal there would be a verb
/// that declined for a reason no operator could act on.
///
/// `Err` for a move that cannot happen; see [`MoveRefusal`] and the module
/// header for why that is a refusal rather than an identity permutation the
/// engine would cheerfully accept and ignore.
///
/// # The algorithm, and the invariant that makes it correct
///
/// For [`MoveDirection::Up`]: walk the operands **ascending**, keeping a
/// `ceiling` — the number of pages already pinned against the top. An operand
/// sitting exactly at the ceiling cannot move, so it raises the ceiling and is
/// skipped; every other operand swaps with the position above it.
///
/// The invariant is that when operand `p` is reached, **position `p` still
/// holds page `p`**: the only swaps performed so far touched positions
/// `p' - 1` and `p'` for operands `p' < p`, and `p' < p` gives `p' <= p - 1`,
/// so the highest position touched is `p - 1`. Position `p` is therefore
/// untouched, and `swap(p, p - 1)` moves the right page.
///
/// [`MoveDirection::Down`] is the mirror image: descending, a `floor` counting
/// the pages pinned against the bottom, and `swap(p, p + 1)`.
///
/// [`EditError::NotAPermutation`]: pdfcer_core::edit::EditError::NotAPermutation
pub fn move_order(
    pages: &[usize],
    page_count: usize,
    direction: MoveDirection,
) -> Result<Vec<usize>, MoveRefusal> {
    if pages.is_empty() || page_count < 2 {
        return Err(MoveRefusal::NothingToMove);
    }
    let mut order: Vec<usize> = (0..page_count).collect();
    let mut moved = false;

    match direction {
        MoveDirection::Up => {
            let mut ceiling = 0usize;
            for &page in pages {
                if page >= page_count {
                    // Unreachable through `operands`, which filters. Skipped
                    // rather than refused: a stale index must not cost the
                    // operator the rest of a move they can see happening.
                    continue;
                }
                if page == ceiling {
                    ceiling += 1;
                    continue;
                }
                order.swap(page, page - 1);
                moved = true;
            }
        }
        MoveDirection::Down => {
            let mut floor = page_count;
            for &page in pages.iter().rev() {
                if page >= page_count {
                    continue;
                }
                if page + 1 == floor {
                    floor -= 1;
                    continue;
                }
                order.swap(page, page + 1);
                moved = true;
            }
        }
    }

    if moved {
        Ok(order)
    } else {
        Err(MoveRefusal::AtTheEdge)
    }
}

/// **The permutation a DRAG asks `EditSession::reorder_pages` for** — the
/// operands lifted out and re-inserted at one chosen gap.
///
/// `pages` is [`operands`]' output: ascending, de-duplicated, in range.
/// `page_count` is the document's current length. `gap` is a **gap index**,
/// not a page index — see below, because it is the one thing about this
/// function that is easy to get wrong.
///
/// # ★ Gap indices, and why they are not page indices
///
/// There are `page_count + 1` places a block can land in a document of
/// `page_count` pages, and only `page_count` pages. Numbering the landing
/// places by the page they precede would leave the last one — *after the final
/// sheet* — unnameable, which is the commonest drag on a drawing set.
///
/// So `gap` counts the boundaries:
///
/// ```text
///   gap:   0      1      2      3
///        │ p0   │ p1   │ p2   │
/// ```
///
/// `gap == 0` is before the first page; `gap == page_count` is after the last.
/// This is the same distinction `pdfcer_core::pageops::InsertPosition` draws
/// with `Before(n)` / `After(n)` / `Start` / `End`, and the mapping is exact:
/// `Before(n)` is gap `n`, `After(n)` is gap `n + 1`, `Start` is gap 0, `End`
/// is gap `page_count`. The Insert-from-file dialog names its destination in
/// those words, and a drag has to mean the same thing or the two surfaces
/// would disagree about where "here" is.
///
/// # ★ The gap is measured against the document BEFORE the lift
///
/// This is the subtle half and it is what [`drag_is_a_no_op`] exists for.
/// The operator points at a boundary in the grid they can see — a grid that
/// still contains the pages they are dragging. Lifting those pages out closes
/// up the gaps they occupied, so a landing place at gap `g` is, after the
/// lift, wherever `g` ends up once every operand below it has been removed.
///
/// The implementation therefore does exactly that, in the order a person
/// would: take the operands out in document order, count how many of them sat
/// below `gap`, and splice the block back in at `gap` minus that count. A
/// version that "corrected" the gap up front would have to reason about
/// operands *straddling* the boundary, which is where an off-by-one lives.
///
/// # Returns
///
/// `Ok(order)` with `order[i]` = the **current** 0-based index of the page
/// that should end up at position `i` — `reorder_pages`' contract verbatim,
/// and a permutation of `0..page_count` by construction because it is built by
/// removing every operand from the identity and re-inserting all of them.
///
/// `Err(MoveRefusal::NothingToMove)` for an empty operand set or a document
/// too short to reorder, and `Err(MoveRefusal::AtTheEdge)` for a drag that
/// lands the block back where it started. The second is the important one: the
/// engine would accept an identity permutation and record an undo entry for
/// it, so an operator who picked a page up and put it down would get a
/// document marked as edited and a `Ctrl+Z` that does nothing visible. That is
/// the same argument [`move_order`] makes for refusing at the edge, arriving
/// from a different direction.
pub fn drop_order(
    pages: &[usize],
    page_count: usize,
    gap: usize,
) -> Result<Vec<usize>, MoveRefusal> {
    if pages.is_empty() || page_count < 2 {
        return Err(MoveRefusal::NothingToMove);
    }
    let gap = gap.min(page_count);
    let moving: BTreeSet<usize> = pages.iter().copied().filter(|p| *p < page_count).collect();
    if moving.is_empty() {
        return Err(MoveRefusal::NothingToMove);
    }
    if drag_is_a_no_op(&moving, gap) {
        return Err(MoveRefusal::AtTheEdge);
    }

    // Everything that is NOT moving, in document order — the sequence the
    // block is spliced into.
    let mut order: Vec<usize> = (0..page_count).filter(|p| !moving.contains(p)).collect();
    // How far into that sequence the chosen boundary now sits: the boundary
    // was at `gap` in the full document, and every operand strictly below it
    // has been lifted out from underneath.
    let lifted_below = moving.iter().filter(|p| **p < gap).count();
    let at = gap.saturating_sub(lifted_below).min(order.len());
    // Reversed, because each `insert` pushes the previous one along: inserting
    // the largest first at a fixed position leaves them ascending. Splicing a
    // whole slice would say the same thing and is not available on `Vec`
    // without `splice`, whose range semantics are a second thing to get right.
    for &page in moving.iter().rev() {
        order.insert(at, page);
    }
    Ok(order)
}

/// Whether landing `moving` at `gap` would leave the document exactly as it
/// is.
///
/// True in two cases, and both are ordinary rather than pathological — they
/// are what a drag that has not gone anywhere yet looks like:
///
/// 1. **The boundary is inside the block.** Dragging pages 4–6 and hovering
///    the gap between 5 and 6 asks for them to be re-inserted where they
///    already are.
/// 2. **The boundary is either lip of a contiguous block.** Dragging 4–6 to
///    the gap immediately before 4, or immediately after 6, is the same
///    request.
///
/// A **non-contiguous** selection dropped at its own first lip is NOT a no-op
/// — pages 1, 5 and 9 dropped before page 1 become 1, 5, 9 adjacent at the
/// top, which is a real edit — so the predicate tests contiguity rather than
/// only the endpoints. Getting that wrong would refuse the single most useful
/// thing a drag does on a drawing set: gathering scattered sheets together.
#[must_use]
pub fn drag_is_a_no_op(moving: &BTreeSet<usize>, gap: usize) -> bool {
    let (Some(&lo), Some(&hi)) = (moving.iter().next(), moving.iter().next_back()) else {
        return true;
    };
    let contiguous = hi - lo + 1 == moving.len();
    if !contiguous {
        return false;
    }
    (lo..=hi + 1).contains(&gap)
}

/// Where each page ended up, as `new_position[old_index]`.
///
/// The inverse of a [`move_order`] permutation, and the one thing every reader
/// of a reorder needs that the permutation itself does not directly say:
/// `order` answers *"which page is at position `i`?"* and this answers *"where
/// did page `p` go?"*, which is the question a selection has to ask in order to
/// follow its pages across the move.
///
/// Kept here beside the rule it inverts rather than in
/// [`super::select`], because `PageSelection` is a set of indices and knows
/// nothing about reordering — the same reason the click policy lives there and
/// not here.
///
/// # Returns
///
/// A vector of length `order.len()`. An entry of `order` that is out of range
/// is skipped, which cannot happen for a [`move_order`] result and is handled
/// so this stays total for a caller that built its own.
#[must_use]
pub fn inverse(order: &[usize]) -> Vec<usize> {
    let mut inverse = vec![0usize; order.len()];
    for (position, &source) in order.iter().enumerate() {
        if let Some(slot) = inverse.get_mut(source) {
            *slot = position;
        }
    }
    inverse
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A drop's landing, as the page order the document ends up in.
    ///
    /// `drop_order` returns the engine's permutation — *which page is at
    /// position i* — and every assertion below is easier to read as the page
    /// sequence itself, which is what an operator sees in the grid.
    fn landing(pages: &[usize], count: usize, gap: usize) -> Vec<usize> {
        drop_order(pages, count, gap).expect("the drop was expected to land")
    }

    /// A contiguous block moves forward, and the gap is read against the
    /// document as it was before the lift.
    ///
    /// ★ The case that catches an off-by-one: dragging pages 0–1 to gap 4 in a
    /// five-page document. Naively splicing at 4 into the three remaining
    /// pages would put them after page 4, not before it. The lift-count
    /// correction is what makes the answer `[2, 3, 0, 1, 4]`.
    #[test]
    fn a_block_dragged_forward_lands_at_the_boundary_that_was_pointed_at() {
        assert_eq!(landing(&[0, 1], 5, 4), vec![2, 3, 0, 1, 4]);
        assert_eq!(landing(&[0], 5, 3), vec![1, 2, 0, 3, 4]);
    }

    /// A block dragged backwards needs no correction, and must not get one.
    #[test]
    fn a_block_dragged_backward_lands_at_the_boundary_that_was_pointed_at() {
        assert_eq!(landing(&[3, 4], 5, 1), vec![0, 3, 4, 1, 2]);
        assert_eq!(landing(&[4], 5, 0), vec![4, 0, 1, 2, 3]);
    }

    /// The last gap is reachable, which is the whole reason gaps are numbered
    /// rather than named by the page they precede.
    #[test]
    fn the_gap_after_the_final_sheet_is_a_real_destination() {
        assert_eq!(landing(&[0], 4, 4), vec![1, 2, 3, 0]);
        // And a gap beyond it is clamped rather than refused: a pointer past
        // the end of the last row means "after everything", which is what the
        // operator is reaching for.
        assert_eq!(landing(&[0], 4, 99), vec![1, 2, 3, 0]);
    }

    /// ★ **A scattered selection gathered at its own first page is a real
    /// edit**, and refusing it would cost the most useful thing a drag does.
    ///
    /// Pages 0, 4 and 8 dropped at gap 0 become adjacent at the top. A
    /// predicate that only compared the drop against the block's endpoints
    /// would call this a no-op, because gap 0 is the lower lip of the
    /// selection's range — and the operator would drag, release, and watch
    /// nothing happen.
    #[test]
    fn gathering_scattered_pages_at_their_own_first_page_is_not_a_no_op() {
        assert!(!drag_is_a_no_op(&set(&[0, 4, 8]), 0));
        assert_eq!(landing(&[0, 4, 8], 9, 0), vec![0, 4, 8, 1, 2, 3, 5, 6, 7]);
    }

    /// A contiguous block dropped on itself, on either lip or anywhere inside,
    /// is refused rather than recorded.
    ///
    /// The engine would accept the identity permutation and write an undo
    /// entry for it, so an operator who picked a page up and put it back would
    /// get a document marked as edited and a `Ctrl+Z` that changes nothing
    /// they can see.
    #[test]
    fn a_block_dropped_on_itself_is_refused_at_every_boundary_it_spans() {
        for gap in 4..=7 {
            assert!(
                drag_is_a_no_op(&set(&[4, 5, 6]), gap),
                "gap {gap} is inside or on the lip of 4..=6"
            );
            assert_eq!(drop_order(&[4, 5, 6], 10, gap), Err(MoveRefusal::AtTheEdge));
        }
        assert!(!drag_is_a_no_op(&set(&[4, 5, 6]), 3));
        assert!(!drag_is_a_no_op(&set(&[4, 5, 6]), 8));
    }

    /// Every result is a permutation of `0..page_count`, which is what
    /// `reorder_pages` refuses anything else for.
    ///
    /// Swept over every operand set and every gap in a small document rather
    /// than spot-checked: the failure this guards against is not one wrong
    /// answer, it is a whole family of drags producing a vector the engine
    /// declines for a reason no operator could act on.
    #[test]
    fn every_landing_is_a_permutation() {
        const COUNT: usize = 6;
        for mask in 1u32..(1 << COUNT) {
            let pages: Vec<usize> = (0..COUNT).filter(|i| mask & (1 << i) != 0).collect();
            for gap in 0..=COUNT {
                let Ok(order) = drop_order(&pages, COUNT, gap) else {
                    continue;
                };
                let seen: BTreeSet<usize> = order.iter().copied().collect();
                assert_eq!(
                    seen,
                    (0..COUNT).collect::<BTreeSet<usize>>(),
                    "pages {pages:?} at gap {gap} produced {order:?}"
                );
                assert_eq!(order.len(), COUNT);
            }
        }
    }

    /// The operands stay in document order and stay together, however
    /// scattered they were.
    ///
    /// A drag says *"put these here"*, and "these" is a set the operator built
    /// by clicking. Re-ordering them relative to each other would be the panel
    /// inventing an intention; leaving gaps between them would make the drag
    /// do half of what it looks like it does.
    #[test]
    fn the_moved_pages_arrive_adjacent_and_in_document_order() {
        let order = landing(&[1, 3, 5], 7, 7);
        assert_eq!(order, vec![0, 2, 4, 6, 1, 3, 5]);
        let positions: Vec<usize> = [1, 3, 5]
            .iter()
            .map(|p| order.iter().position(|q| q == p).expect("present"))
            .collect();
        assert_eq!(positions, vec![4, 5, 6], "adjacent, ascending, in order");
    }

    /// Nothing selected, or nothing to reorder, is a refusal by name.
    #[test]
    fn an_empty_drag_is_refused_rather_than_silently_identity() {
        assert_eq!(drop_order(&[], 5, 2), Err(MoveRefusal::NothingToMove));
        assert_eq!(drop_order(&[0], 1, 0), Err(MoveRefusal::NothingToMove));
        // An out-of-range operand is filtered, and a set that filters to
        // nothing refuses rather than producing the identity.
        assert_eq!(drop_order(&[9], 3, 0), Err(MoveRefusal::NothingToMove));
    }

    /// A set built the way a caller's `BTreeSet` arrives.
    fn set(items: &[usize]) -> BTreeSet<usize> {
        items.iter().copied().collect()
    }

    /// **★ With nothing picked, a verb acts on the current page.**
    ///
    /// The rule `PanelsState::selected_pages` states in words — *"Empty is a
    /// defined answer, not a missing one"* — as a mechanism. A build that
    /// returned an empty operand list here would make every ribbon Pages
    /// control do nothing until the operator discovered the panel, which is the
    /// exact "live control, no effect" defect this work exists to close.
    #[test]
    fn with_nothing_picked_the_operand_is_the_current_page() {
        assert_eq!(operands(&set(&[]), 2, 4), vec![2]);
        assert_eq!(operands(&set(&[]), 0, 1), vec![0]);
    }

    /// With pages picked, the pick wins over the current page.
    #[test]
    fn a_pick_beats_the_current_page() {
        assert_eq!(operands(&set(&[3, 1]), 0, 4), vec![1, 3]);
    }

    /// **★ An operand past the end of the document is dropped, not passed on.**
    ///
    /// `EditSession::delete_pages` resolves **every** index before planning
    /// anything and returns `PageOutOfRange` for the whole batch if one is bad,
    /// so a stale pick would turn "delete these three" into "delete nothing,
    /// silently". Reachable by chord: the panel clamps on its next frame and a
    /// keyboard verb can arrive before that frame is drawn.
    #[test]
    fn a_stale_pick_is_dropped_rather_than_refusing_the_whole_batch() {
        assert_eq!(operands(&set(&[0, 9]), 0, 4), vec![0]);
        // …and a pick list that is entirely stale falls back to the current
        // page rather than to nothing, which keeps the verb doing something
        // rather than failing invisibly.
        assert_eq!(operands(&set(&[7, 9]), 1, 4), vec![1]);
    }

    /// A document with no pages has no operand at all.
    #[test]
    fn a_document_with_no_pages_has_no_operand() {
        assert!(operands(&set(&[]), 0, 0).is_empty());
        assert!(operands(&set(&[0]), 0, 0).is_empty());
    }

    /// One page moves up by one, and the rest of the document is untouched.
    #[test]
    fn one_page_moves_up_by_one() {
        assert_eq!(move_order(&[1], 4, MoveDirection::Up), Ok(vec![1, 0, 2, 3]));
        assert_eq!(move_order(&[3], 4, MoveDirection::Up), Ok(vec![0, 1, 3, 2]));
    }

    /// One page moves down by one.
    #[test]
    fn one_page_moves_down_by_one() {
        assert_eq!(
            move_order(&[1], 4, MoveDirection::Down),
            Ok(vec![0, 2, 1, 3])
        );
        assert_eq!(
            move_order(&[0], 4, MoveDirection::Down),
            Ok(vec![1, 0, 2, 3])
        );
    }

    /// **★ A contiguous run moves as a run, keeping its internal order.**
    ///
    /// The property a naive "swap each with its neighbour" loop gets wrong: run
    /// it ascending without the ceiling and pages 1 and 2 swap with each other
    /// twice and end up back where they started. This asserts the *magnitude*
    /// as well as the direction — the run really is one place earlier — which
    /// is `HANDOFF.md` §2's grid lesson applied to an index.
    #[test]
    fn a_contiguous_run_moves_as_a_run() {
        assert_eq!(
            move_order(&[1, 2], 4, MoveDirection::Up),
            Ok(vec![1, 2, 0, 3]),
            "pages 2 and 3 should land at positions 1 and 2, with page 1 after them"
        );
        assert_eq!(
            move_order(&[0, 1], 4, MoveDirection::Down),
            Ok(vec![2, 0, 1, 3]),
            "pages 1 and 2 should land at positions 2 and 3"
        );
        // …and a run of three, so the property is not an accident of two.
        assert_eq!(
            move_order(&[1, 2, 3], 5, MoveDirection::Up),
            Ok(vec![1, 2, 3, 0, 4])
        );
    }

    /// **★ A non-contiguous pick moves as separate items, each by one.**
    ///
    /// The alternative — gathering the set at its topmost member — would
    /// reorder pages the operator never named, which is the same class of
    /// error `select::PageSelection::right_click` exists to prevent from the
    /// pointer's side.
    #[test]
    fn a_non_contiguous_pick_moves_each_item_by_one() {
        assert_eq!(
            move_order(&[1, 3], 4, MoveDirection::Up),
            Ok(vec![1, 0, 3, 2]),
            "page 2 to position 0 and page 4 to position 2; pages 1 and 3 fill the gaps"
        );
        assert_eq!(
            move_order(&[0, 2], 4, MoveDirection::Down),
            Ok(vec![1, 0, 3, 2])
        );
    }

    /// **★ The first page cannot move up, and it says so rather than
    /// producing an identity the engine would silently accept.**
    ///
    /// `reorder_pages` returns `Ok(())` for the identity, having recorded
    /// nothing. Handing it one would be a control the operator pressed that
    /// changed nothing and said nothing — which is the defect this project is
    /// named after, so the refusal is the result rather than a detail.
    #[test]
    fn the_top_of_the_document_refuses_a_move_up() {
        assert_eq!(
            move_order(&[0], 4, MoveDirection::Up),
            Err(MoveRefusal::AtTheEdge)
        );
        assert_eq!(
            move_order(&[0, 1], 4, MoveDirection::Up),
            Err(MoveRefusal::AtTheEdge),
            "a run that already starts at page 1 is entirely blocked, not partly moved"
        );
        assert_eq!(
            move_order(&[3], 4, MoveDirection::Down),
            Err(MoveRefusal::AtTheEdge)
        );
        assert_eq!(
            move_order(&[2, 3], 4, MoveDirection::Down),
            Err(MoveRefusal::AtTheEdge)
        );
    }

    /// **★ A partly-blocked run still moves the part that can move.**
    ///
    /// Pages 1 and 3 picked, moved up: page 1 is pinned, page 3 is not. The
    /// alternative — refusing the whole gesture because one member is at the
    /// edge — would make a large selection increasingly hard to move, which is
    /// the opposite of what a reorder control is for.
    #[test]
    fn a_partly_blocked_run_moves_the_part_that_can() {
        assert_eq!(
            move_order(&[0, 2], 4, MoveDirection::Up),
            Ok(vec![0, 2, 1, 3]),
            "page 1 stays put and page 3 moves up past page 2"
        );
    }

    /// Nothing to move is its own refusal, distinct from being at the edge.
    #[test]
    fn nothing_to_move_is_distinguished_from_being_at_the_edge() {
        assert_eq!(
            move_order(&[], 4, MoveDirection::Up),
            Err(MoveRefusal::NothingToMove)
        );
        assert_eq!(
            move_order(&[0], 1, MoveDirection::Up),
            Err(MoveRefusal::NothingToMove),
            "a one-page document cannot be reordered in either direction"
        );
        assert_eq!(
            move_order(&[0], 1, MoveDirection::Down),
            Err(MoveRefusal::NothingToMove)
        );
    }

    /// **★★ Every order this module produces is a permutation of
    /// `0..page_count`.**
    ///
    /// The one property `EditSession::reorder_pages` checks and refuses over,
    /// and therefore the one whose failure would be a verb that declines with
    /// nothing an operator could do about it. Asserted exhaustively over every
    /// non-empty subset of a five-page document, in both directions — 62 cases,
    /// which is cheap and is the difference between "the examples above work"
    /// and "the rule is sound".
    #[test]
    fn every_order_is_a_permutation_and_moves_by_exactly_one() {
        const N: usize = 5;
        for mask in 1u32..(1 << N) {
            let pages: Vec<usize> = (0..N).filter(|i| mask & (1 << i) != 0).collect();
            for direction in [MoveDirection::Up, MoveDirection::Down] {
                let Ok(order) = move_order(&pages, N, direction) else {
                    continue;
                };
                let seen: BTreeSet<usize> = order.iter().copied().collect();
                assert_eq!(
                    seen,
                    (0..N).collect::<BTreeSet<_>>(),
                    "{pages:?} {direction:?} produced {order:?}, which reorder_pages would refuse"
                );
                assert_eq!(order.len(), N);

                let landed = inverse(&order);

                // ★ **A picked page moves by exactly one place, or not at
                // all**, and always toward the edge it was sent to. This is
                // what makes the verb "move up" rather than "sort", and a
                // rule that produced a valid permutation by jumping a page
                // three places would pass the permutation check above.
                //
                // The bound is asserted for the PICKED pages only, and that is
                // a fact about the operation rather than a weakening: a run of
                // three moving down displaces the page above it by **three**,
                // because that page has to end up behind all of them. Applying
                // the ±1 bound to every page was this test's first form and it
                // fired on `{1,2}` moving down — correctly, which is the whole
                // value of running an exhaustive fixture.
                for &page in &pages {
                    let position = landed[page];
                    assert!(
                        position.abs_diff(page) <= 1,
                        "{pages:?} {direction:?}: page {page} moved to position {position}, \
                         which is more than one place"
                    );
                    match direction {
                        MoveDirection::Up => assert!(position <= page),
                        MoveDirection::Down => assert!(position >= page),
                    }
                }

                // …and at least one of them really moved. Without this the
                // test is satisfied by a rule that returns the identity, which
                // is the exact failure `MoveRefusal` exists to make impossible.
                assert!(
                    pages.iter().any(|&p| landed[p] != p),
                    "{pages:?} {direction:?} moved none of its operands, so it should have \
                     refused rather than returning a permutation"
                );

                // ★★ **Nothing else is reordered.** The strongest available
                // statement of "this verb moved what it was asked to and left
                // the rest alone": the picked pages keep their relative order
                // among themselves, and so do the unpicked ones. A rule that
                // shuffled two untouched sheets past each other would satisfy
                // every assertion above and would silently reorder a drawing
                // set.
                let picked_after: Vec<usize> = order
                    .iter()
                    .copied()
                    .filter(|p| pages.contains(p))
                    .collect();
                assert_eq!(
                    picked_after, pages,
                    "{pages:?} {direction:?} changed the picked sheets' order among themselves"
                );
                let rest: Vec<usize> = (0..N).filter(|p| !pages.contains(p)).collect();
                let rest_after: Vec<usize> = order
                    .iter()
                    .copied()
                    .filter(|p| !pages.contains(p))
                    .collect();
                assert_eq!(
                    rest_after, rest,
                    "{pages:?} {direction:?} reordered sheets nobody named"
                );
            }
        }
    }

    /// The inverse really inverts.
    #[test]
    fn the_inverse_of_a_permutation_says_where_each_page_landed() {
        let order = move_order(&[1, 2], 4, MoveDirection::Up).expect("a legal move");
        let landed = inverse(&order);
        assert_eq!(landed, vec![2, 0, 1, 3]);
        for (position, &source) in order.iter().enumerate() {
            assert_eq!(landed[source], position);
        }
    }
}
