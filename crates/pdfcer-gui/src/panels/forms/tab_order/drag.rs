//! # `panels::forms::tab_order::drag` — reordering the tab list by dragging
//!
//! `OPERATOR_REQUESTS.md` O99, and the operator named the reference himself:
//!
//! > *"the tab order list is supposed to be able to be reordered by dragging
//! > and dropping rows around **like we can with pages in the page preview**,
//! > and have **clear markers** of where the field is going to move to."*
//!
//! So this is deliberately [`crate::panels::pages`]' insertion caret, one panel
//! over: the same gap model (`0` is before the first row, `len` is after the
//! last), the same `Rect`-carrying [`DropTarget`], the same full-strength /
//! dimmed pair for *"releasing here changes nothing"*, and the same rule that a
//! release is read from raw input rather than from a `Response`. Two panels
//! that both mean *"it will go here"* and draw it two different ways is a
//! discoverability defect, and the second one is where it gets introduced.
//!
//! ## ★★★ The one thing that is genuinely different, and it is the hard part
//!
//! **A row is not an array entry.** The page rail reorders pages, and a page is
//! a page — the list the operator drags and the array the engine permutes are
//! the same sequence. Here they are not:
//!
//! * The list holds **widgets a field claims**. The array holds those, plus
//!   unclaimed widgets, plus anonymous ones, plus every `/Link`, `/Text`, stamp
//!   and markup annotation on the page.
//! * `EditSession::reorder_annotations` takes *"the page's indirect `/Annots`
//!   entries, each once, in the wanted order"*. A list built from the rows is
//!   not that; it is a permutation of a **subset**, and the engine refuses it
//!   by name rather than quietly dropping the rest.
//!
//! So [`reordered`] permutes the rows **through their slots**: the widget rows
//! move among the positions widget rows already occupied, and every other entry
//! keeps its index. See that function for why that is the right rule and not
//! merely the convenient one.
//!
//! ## Rule 4
//!
//! The caret is the **cursor**, in the class the rule permits by name — *"snap
//! indicators, hover highlights, rubber-bands and selection handles are the
//! cursor and are welcome"*. It marks no content, tints no field, draws nothing
//! onto the canvas, and is gone the instant the pointer is released.

use pdfcer_core::object::ObjId;

use super::model::PageTabs;

/// A drag in flight over the tab-order list.
///
/// # Why it carries the page and not just the row
///
/// The list is per page and there are as many blocks as the document has pages.
/// A drag that began on page 3 must not be answered by page 4's block — which
/// is not hypothetical, because every block runs the same code in the same
/// frame and would otherwise all believe the drag was theirs.
///
/// Reordering **across** pages is deliberately not offered: moving a widget to
/// another page is a different edit (it changes which sheet the field is on,
/// not merely when it is reached) and `reorder_annotations` cannot express it.
/// A drag that leaves its own block simply finds no gap and lands nowhere.
///
/// `Default` is derived for one reason, and it is the same one
/// [`crate::pagedrag::PageDrag`] records: `egui::IdTypeMap::remove_temp`
/// demands it of anything it can take back out. A defaulted `Drag` — page 0,
/// row 0 — is never constructed here and is not a state the application can
/// reach; [`current`] answers `Option`, so "no drag" is `None` and never a
/// drag of the first row of the first page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Drag {
    /// The 0-based page index whose block the drag started in.
    pub page_index: usize,
    /// The index **within that page's `rows`** of the row being dragged.
    pub from: usize,
}

/// Where a drag in flight would land.
///
/// [`crate::panels::pages`]' `DropTarget`, with the same three fields and the
/// same reasons: a gap has no position until the rows have been laid out, so it
/// is resolved during the layout pass and carried out; the caret is a `Rect`
/// because a line is two endpoints and the layout pass knows nothing about
/// colour or width; and `lands` is computed where the row set is in scope
/// because the paint pass no longer has it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DropTarget {
    /// The gap index: `0` before the first row, `rows.len()` after the last.
    pub gap: usize,
    /// The line to draw, in the scroll area's coordinate space.
    pub caret: egui::Rect,
    /// Whether releasing here would actually change the order.
    pub lands: bool,
}

/// How thick the insertion caret is drawn.
///
/// [`crate::panels::pages`]' `CARET_PTS` verbatim. The two panels draw the same
/// mark and an operator who has learnt one has learnt the other; a caret that
/// were a hair thinner here would read as a different, weaker kind of promise.
const CARET_PTS: f32 = 2.0;

/// How much of the caret's colour survives when the drop would change nothing.
///
/// ★ **Dimmed, not hidden** — the page rail's argument, unchanged and load
/// bearing: drawing nothing over a boundary that would not land cannot be told
/// apart from the panel having stopped tracking the pointer, and the no-op
/// boundary is where **every** drag begins, because a row starts out hovering
/// over its own slot.
const CARET_DIMMED: f32 = 0.35;

/// The published region name for the caret, so a driven check can see it.
// ui-text-exempt: trace region name, never displayed
pub(super) const REGION_CARET: &str = "forms.tab_order.drop-caret";

/// The prefix of the per-row region names; `page.row` is appended.
///
/// Keyed by **page index and row index**, not by tab position: position counts
/// widgets and shifts when an unclaimed one is registered, and a check that
/// named a row by it would aim at a different row after an unrelated edit.
// ui-text-exempt: trace region name, never displayed
pub(super) const REGION_ROW_PREFIX: &str = "forms.tab_order.row.";

fn key() -> egui::Id {
    egui::Id::new("pdfcer-tab-order-drag") // ui-text-exempt: an id, never displayed
}

/// The drag in flight, if there is one.
pub(super) fn current(ctx: &egui::Context) -> Option<Drag> {
    ctx.data_mut(|d| d.get_temp::<Drag>(key()))
}

/// Start a drag.
pub(super) fn begin(ctx: &egui::Context, drag: Drag) {
    ctx.data_mut(|d| d.insert_temp(key(), drag));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "tab-order-drag-begin page={} from={}",
            drag.page_index, drag.from
        )
    });
}

/// End a drag, returning it.
pub(super) fn end(ctx: &egui::Context) -> Option<Drag> {
    ctx.data_mut(|d| d.remove_temp::<Drag>(key()))
}

/// **Permute a page's `/Annots` so its widget rows sit in a new order.**
///
/// `slots` are the rows' indices into `annots`, ascending — that is,
/// `page.rows.iter().map(|r| r.slot)`. `from` and `to_gap` are in **row**
/// space: `from` is the row being dragged, `to_gap` is the boundary it is
/// dropped at, where `0` is before the first row and `slots.len()` is after the
/// last.
///
/// # ★★★ The rule: widgets move among widget slots; nothing else moves at all
///
/// The alternative — permuting the array wholesale, so a widget travels with
/// whatever entries happen to sit beside it — was considered and rejected. Two
/// reasons, and the second is the stronger:
///
/// 1. **It is not what the gesture says.** The operator dragged a row in a list
///    of form fields. Moving a `/Link` because a text box passed over it is a
///    consequence of the implementation, not of the request.
/// 2. **`/Annots` order is paint order.** Moving a non-widget changes which
///    annotation is drawn on top where two overlap. That is a visible change to
///    the rendered page, produced by a gesture whose entire subject was the
///    order boxes are *reached in*. The engine discloses it
///    (`non_widgets_moved`) precisely because it is surprising — and the right
///    response to a surprising consequence you can avoid is to avoid it, and
///    keep the disclosure for the cases you cannot.
///
/// So this route reports `non_widgets_moved == 0` on every call, by
/// construction. That is not the disclosure being dead code: it is this route
/// being the one that does not need it.
///
/// # Returns the whole array, not the changed part
///
/// Because that is what the verb takes. `annots` in, `annots` permuted out,
/// same length, same multiset — which is exactly the property the engine
/// validates and refuses (`AnnotsNotAPermutation`) rather than trusts.
///
/// # A drag that lands where it started returns the input unchanged
///
/// `to_gap == from` and `to_gap == from + 1` are both the row's own boundaries.
/// The output is then equal to the input, the engine's `moved` is `0`, and
/// nothing is disclosed — which is the common case for a drag an operator
/// thinks better of, and it must not read as a refusal.
pub(super) fn reordered(
    annots: &[ObjId],
    slots: &[usize],
    from: usize,
    to_gap: usize,
) -> Vec<ObjId> {
    let mut out = annots.to_vec();
    if from >= slots.len() || to_gap > slots.len() {
        return out;
    }
    // The order the SOURCE slots are read in, after the move. Working in slot
    // space rather than id space keeps this correct on a malformed file that
    // lists one object twice: the slots are distinct by construction even when
    // the ids are not, so the output is still a rearrangement of the input
    // rather than a duplication of it. (The engine refuses such a file anyway,
    // by name — but it should refuse it for the file's defect, not for one this
    // function introduced.)
    let mut sources: Vec<usize> = slots.to_vec();
    let moved = sources.remove(from);
    // ★ The off-by-one every insertion-caret implementation has to get right: a
    // gap index counts boundaries in the ORIGINAL list, and removing the
    // dragged row has already shifted every boundary after it down by one.
    let at = if to_gap > from { to_gap - 1 } else { to_gap };
    sources.insert(at, moved);
    for (dest, src) in slots.iter().zip(sources.iter()) {
        out[*dest] = annots[*src];
    }
    out
}

/// Whether releasing at `to_gap` would change anything.
///
/// Its own function, and named, because it is the *same* question
/// [`crate::panels::pages`]' `ops::drag_is_a_no_op` answers for pages, and
/// because it is what decides whether the caret is drawn at full strength or
/// dimmed. Getting it wrong in the dim direction makes a working drop look
/// refused; getting it wrong the other way promises an edit that will not
/// happen.
pub(super) const fn lands(from: usize, to_gap: usize) -> bool {
    to_gap != from && to_gap != from + 1
}

/// Draw the insertion caret.
///
/// The colour is `visuals().selection.stroke.color` — the theme's, never a
/// literal, and the same source the page rail's caret, the current-page ring
/// and the canvas guide preview all take, so a preset that changes the accent
/// changes every one of them together.
pub(super) fn paint(ui: &egui::Ui, drop: Option<&DropTarget>) {
    let Some(drop) = drop else {
        return;
    };
    let base = ui.visuals().selection.stroke.color;
    let colour = if drop.lands {
        base
    } else {
        base.gamma_multiply(CARET_DIMMED)
    };
    // HORIZONTAL, where the page rail's is vertical — the one place the two
    // caret implementations legitimately differ, and it is not a style choice:
    // the rail is a grid that flows left to right, this is a list that flows
    // top to bottom, and an insertion mark runs across the flow.
    ui.painter().line_segment(
        [drop.caret.left_top(), drop.caret.right_top()],
        egui::Stroke::new(CARET_PTS, colour),
    );
    crate::diag::ui_rect_visible(REGION_CARET, drop.caret.expand(CARET_PTS), ui.clip_rect());
}

/// Resolve the gap a row's rectangle implies for the pointer, and keep the
/// nearest.
///
/// Called once per row during the layout pass, with the row's rectangle and the
/// pointer. The row's own midpoint splits it: above means *before* this row,
/// below means *after* it.
pub(super) fn consider(
    row: egui::Rect,
    index: usize,
    pointer: egui::Pos2,
    drag: Drag,
    drop: &mut Option<DropTarget>,
) {
    // Horizontal extent is not tested. A list row is full width and an operator
    // dragging down a narrow dock wanders out of it constantly; requiring the
    // pointer to stay inside would make the caret flicker off exactly when the
    // drag is longest. The page rail's grid has to test both axes because its
    // tiles tile in both; a single-column list does not.
    let gap = if pointer.y < row.center().y {
        index
    } else {
        index + 1
    };
    let y = if gap == index {
        row.top()
    } else {
        row.bottom()
    };
    let caret = egui::Rect::from_min_max(egui::pos2(row.left(), y), egui::pos2(row.right(), y));
    // Nearest wins. Two adjacent rows both claim the boundary between them —
    // one as "after me", one as "before me" — and they agree on the gap index,
    // so which one is kept does not matter. What does matter is that a pointer
    // far below the last row keeps the last row's answer rather than the first
    // row's, which is what this comparison delivers.
    let better = drop
        .is_none_or(|existing| (y - pointer.y).abs() < (existing.caret.top() - pointer.y).abs());
    if better {
        *drop = Some(DropTarget {
            gap,
            caret,
            lands: lands(drag.from, gap),
        });
    }
}

/// **End a drag** — read the release, build the permutation, raise the action.
///
/// # Why the release is read from raw input
///
/// [`crate::panels::pages`]' `settle_drag`, and its reason applies here
/// unchanged: a drag that began on a row may end anywhere — over the page
/// heading, past the last row, outside the scroll area, or after the pointer
/// has left the window. A `Response` reports releases only inside the widget
/// that produced it, so a release elsewhere would strand the drag in flight
/// with a caret nobody could dismiss.
///
/// # A drag that lands nowhere raises NO action, and says so
///
/// Released over no gap, over its own boundary, or in another page's block: the
/// drag ends, the caret goes, and nothing is committed. Traced, because
/// "nothing happened" and "something happened that did nothing" are the two
/// readings a check has to be able to tell apart.
pub(super) fn settle(
    ui: &egui::Ui,
    page: &PageTabs,
    drop: Option<&DropTarget>,
    actions: &mut Vec<crate::app::actions::Action>,
) {
    let Some(drag) = current(ui.ctx()) else {
        return;
    };
    if drag.page_index != page.page_index {
        return;
    }
    if !ui.input(|i| i.pointer.any_released()) {
        return;
    }
    end(ui.ctx());
    let Some(target) = drop.filter(|t| t.lands) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "tab-order-drag-release page={} from={} gap={} reordered=0",
                page.page_index,
                drag.from,
                drop.map_or(usize::MAX, |t| t.gap)
            )
        });
        return;
    };
    let slots: Vec<usize> = page.rows.iter().map(|r| r.slot).collect();
    let order = reordered(&page.annots, &slots, drag.from, target.gap);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "tab-order-drag-release page={} from={} gap={} reordered=1 entries={}",
            page.page_index,
            drag.from,
            target.gap,
            order.len()
        )
    });
    actions.push(crate::app::actions::Action::Field(
        crate::app::actions::forms::FieldAction::ReorderAnnotations {
            page: page.page_index,
            order,
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(n: u32) -> Vec<ObjId> {
        (0..n).map(|i| ObjId::new(i + 1, 0)).collect()
    }

    /// The plain case: three rows, no other annotations, drag the first to the
    /// end.
    #[test]
    fn a_row_dragged_to_the_end_lands_last() {
        let a = ids(3);
        let out = reordered(&a, &[0, 1, 2], 0, 3);
        assert_eq!(out, vec![a[1], a[2], a[0]]);
    }

    #[test]
    fn a_row_dragged_to_the_front_lands_first() {
        let a = ids(3);
        let out = reordered(&a, &[0, 1, 2], 2, 0);
        assert_eq!(out, vec![a[2], a[0], a[1]]);
    }

    /// ★ The off-by-one, both sides of it. Dropping at a row's own two
    /// boundaries is a no-op, and the array comes back untouched — which is
    /// what makes `moved == 0` the engine's report rather than a spurious edit.
    #[test]
    fn both_of_a_rows_own_boundaries_change_nothing() {
        let a = ids(4);
        for gap in [1, 2] {
            assert_eq!(reordered(&a, &[0, 1, 2, 3], 1, gap), a, "gap {gap}");
            assert!(!lands(1, gap), "gap {gap} should not land");
        }
        assert!(lands(1, 0));
        assert!(lands(1, 3));
    }

    /// ★★★ The property this module exists for: a `/Link` between two widgets
    /// keeps its index while the widgets move around it.
    ///
    /// Slots `0` and `2` are widget rows; slot `1` is something else. Dragging
    /// the first row past the second must swap entries 0 and 2 and leave entry
    /// 1 exactly where it was — which is `non_widgets_moved == 0` at the
    /// engine, by construction.
    #[test]
    fn an_annotation_that_is_not_a_row_never_moves() {
        let a = ids(3);
        let out = reordered(&a, &[0, 2], 0, 2);
        assert_eq!(out, vec![a[2], a[1], a[0]]);
        assert_eq!(out[1], a[1], "the non-row entry must keep its index");
    }

    /// The same, with the non-row entries at the ends rather than the middle —
    /// the arrangement where an implementation that reasoned in row space and
    /// then "shifted by the number of non-widgets before it" goes wrong.
    #[test]
    fn non_row_entries_at_both_ends_are_left_alone() {
        let a = ids(5);
        // slots 1, 2, 3 are rows; 0 and 4 are not.
        let out = reordered(&a, &[1, 2, 3], 2, 0);
        assert_eq!(out, vec![a[0], a[3], a[1], a[2], a[4]]);
    }

    /// Every output is a permutation of the input, for every from/gap pair on a
    /// list with interleaved non-rows. This is the property the engine
    /// validates and refuses, so it is worth asserting exhaustively rather than
    /// at the two or three points a hand-written case would reach.
    #[test]
    fn every_drop_produces_a_permutation() {
        let a = ids(7);
        let slots = [0usize, 2, 3, 6];
        for from in 0..slots.len() {
            for gap in 0..=slots.len() {
                let out = reordered(&a, &slots, from, gap);
                let mut sorted = out.clone();
                sorted.sort_by_key(|id| id.num);
                assert_eq!(sorted, a, "from {from} gap {gap} is not a permutation");
                for slot in [1usize, 4, 5] {
                    assert_eq!(
                        out[slot], a[slot],
                        "from {from} gap {gap} moved slot {slot}"
                    );
                }
            }
        }
    }

    /// An out-of-range drag returns the array untouched rather than panicking.
    ///
    /// Not defensive decoration: the rows are rebuilt from the document every
    /// frame, and an edit landing between the frame a drag began on and the
    /// frame it is released on can shorten the list under it. The honest answer
    /// to "the row you were dragging is gone" is to do nothing.
    #[test]
    fn a_drag_whose_row_has_vanished_does_nothing() {
        let a = ids(3);
        assert_eq!(reordered(&a, &[0, 1, 2], 9, 1), a);
        assert_eq!(reordered(&a, &[0, 1, 2], 0, 9), a);
        assert_eq!(reordered(&a, &[], 0, 0), a);
    }
}
