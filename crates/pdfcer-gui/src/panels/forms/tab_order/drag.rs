//! # `panels::forms::tab_order::drag` — reordering the tab list by dragging
//!
//! `OPERATOR_REQUESTS.md` O99, and the operator named the reference himself:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/forms/tab_order/drag.md`.

use pdfcer_core::object::ObjId;

use super::model::PageTabs;

/// A drag in flight over the tab-order list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Drag {
    /// The 0-based page index whose block the drag started in.
    pub page_index: usize,
    /// The index **within that page's `rows`** of the row being dragged.
    pub from: usize,
}

/// Where a drag in flight would land.
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
const CARET_PTS: f32 = 2.0;

/// How much of the caret's colour survives when the drop would change nothing.
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
    // The off-by-one every insertion-caret implementation has to get right: a
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
pub(super) const fn lands(from: usize, to_gap: usize) -> bool {
    to_gap != from && to_gap != from + 1
}

/// Draw the insertion caret.
pub(super) fn paint(ui: &egui::Ui, drop: Option<&DropTarget>) {
    let Some(drop) = drop else {
        return;
    };
    let base = egui_shell::theme::Theme::canvas_selection_ink(ui.ctx());
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

    /// The off-by-one, both sides of it. Dropping at a row's own two
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

    /// The property this module exists for: a `/Link` between two widgets
    /// keeps its index while the widgets move around it.
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
    #[test]
    fn a_drag_whose_row_has_vanished_does_nothing() {
        let a = ids(3);
        assert_eq!(reordered(&a, &[0, 1, 2], 9, 1), a);
        assert_eq!(reordered(&a, &[0, 1, 2], 0, 9), a);
        assert_eq!(reordered(&a, &[], 0, 0), a);
    }
}
