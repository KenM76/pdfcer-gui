//! The part list's tree: guide lines from each part to the parts under it,
//! and a boxed + or − that folds or unfolds a branch. Folding changes only
//! which rows the list shows, never what the picture draws. Design:
//! `DESIGNS.md` § O415.
//!
//! The list is the engine's node order, a parent before its children; a
//! node's place among its siblings comes from the parents, not the order, so
//! a list that interleaves siblings still draws whole lines.

use std::collections::BTreeSet;

use egui::{Rect, Sense, Ui, Vec2};

/// One row's fold box; the suffix is the node's index in the list.
pub const REGION_FOLD_PREFIX: &str = "model3d.part.fold."; // ui-text-exempt: trace region name, never displayed

/// Points each tree level is indented: one guide column.
const INDENT: f32 = 16.0;
/// The fold box's side, in points.
const BOX: f32 = 9.0;
/// Room around the fold box a click still takes, in points.
const BOX_GRAB: f32 = 3.0;

/// The tree as each node's parent, and each node's place in it: whether it
/// is its parent's last child, and whether it has children of its own.
pub(super) struct Shape {
    parents: Vec<Option<usize>>,
    last: Vec<bool>,
    parent: Vec<bool>,
}

impl Shape {
    /// The tree whose node `i` has parent `parents[i]`.
    pub fn of(parents: Vec<Option<usize>>) -> Self {
        let mut last = vec![true; parents.len()];
        let mut parent = vec![false; parents.len()];
        let mut later: BTreeSet<Option<usize>> = BTreeSet::new();
        for (i, &up) in parents.iter().enumerate().rev() {
            last[i] = later.insert(up);
            if let Some(p) = up.and_then(|p| parent.get_mut(p)) {
                *p = true;
            }
        }
        Self {
            parents,
            last,
            parent,
        }
    }

    /// The indices of `i`'s ancestors, nearest first.
    fn ancestors(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        let up = |j: usize| self.parents.get(j).copied().flatten();
        std::iter::successors(up(i), move |&p| up(p))
    }

    /// The rows shown: every node with no folded ancestor, in list order.
    pub fn visible(&self, folded: &BTreeSet<usize>) -> Vec<usize> {
        (0..self.parents.len())
            .filter(|&i| folded.is_empty() || !self.ancestors(i).any(|a| folded.contains(&a)))
            .collect()
    }

    /// Every node with children: what *Fold every part* folds.
    pub fn branches(&self) -> BTreeSet<usize> {
        self.parents.iter().filter_map(|p| *p).collect()
    }

    pub fn has_children(&self, i: usize) -> bool {
        self.parent.get(i).copied().unwrap_or(false)
    }

    fn is_last(&self, i: usize) -> bool {
        self.last.get(i).copied().unwrap_or(true)
    }
}

/// Node `i`'s guide lines and fold box, in a strip one column per level and
/// one for the box; whether the box was clicked.
///
/// Column `c` belongs to the ancestor at depth `c`: a line runs down it
/// while that ancestor has children below this row, and the row's own
/// parent's column turns right into the row. Lines reach half the row gap
/// past the row so consecutive rows join.
pub(super) fn guides(ui: &mut Ui, shape: &Shape, i: usize, folded: bool) -> bool {
    // The path from depth 1 down to the node itself; the root draws no line.
    let mut chain: Vec<usize> = std::iter::once(i).chain(shape.ancestors(i)).collect();
    chain.pop();
    chain.reverse();
    let depth = chain.len();
    let high = ui.spacing().interact_size.y;
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(INDENT * (depth + 1) as f32, high), Sense::hover());
    let gap = ui.spacing().item_spacing.y / 2.0;
    let (top, mid, bottom) = (rect.top() - gap, rect.center().y, rect.bottom() + gap);
    let x = |column: usize| rect.left() + INDENT * (column as f32 + 0.5);
    let line = ui.visuals().widgets.noninteractive.bg_stroke;
    let branch = shape.has_children(i);
    let painter = ui.painter();
    for (column, &n) in chain.iter().enumerate() {
        let below = !shape.is_last(n);
        if column + 1 == depth {
            painter.vline(x(column), top..=if below { bottom } else { mid }, line);
            let reach = if branch {
                x(depth) - BOX / 2.0
            } else {
                x(depth)
            };
            painter.hline(x(column)..=reach, mid, line);
        } else if below {
            painter.vline(x(column), top..=bottom, line);
        }
    }
    if !branch {
        return false;
    }
    let at = Rect::from_center_size(egui::pos2(x(depth), mid), Vec2::splat(BOX));
    if !folded {
        painter.vline(x(depth), at.bottom()..=bottom, line);
    }
    let response = ui
        .interact(
            at.expand(BOX_GRAB),
            ui.id().with(("part-fold", i)),
            Sense::click(),
        )
        .on_hover_text(crate::text::model3d::part_fold_tooltip());
    // ui-text-exempt: trace region name, never displayed
    let region = format!("{REGION_FOLD_PREFIX}{i}");
    crate::diag::ui_rect_visible(&region, at, ui.clip_rect());
    let ink = ui.style().interact(&response).fg_stroke;
    let painter = ui.painter();
    painter.rect_filled(at, 1.0, ui.visuals().panel_fill);
    painter.rect_stroke(at, 1.0, line, egui::StrokeKind::Inside);
    let inner = at.shrink(2.0);
    painter.hline(inner.left()..=inner.right(), at.center().y, ink);
    if folded {
        painter.vline(at.center().x, inner.top()..=inner.bottom(), ink);
    }
    response.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0 ─┬ 1 ─ 2
    ///    └ 3
    fn tree() -> Shape {
        Shape::of(vec![None, Some(0), Some(1), Some(0)])
    }

    #[test]
    fn last_children_and_branches_come_from_the_parents() {
        let shape = tree();
        assert!(!shape.is_last(1));
        assert!(shape.is_last(2));
        assert!(shape.is_last(3));
        assert!(shape.has_children(0) && shape.has_children(1));
        assert!(!shape.has_children(2) && !shape.has_children(3));
        assert_eq!(shape.branches(), BTreeSet::from([0, 1]));
    }

    #[test]
    fn a_folded_node_hides_everything_under_it_only() {
        let shape = tree();
        assert_eq!(shape.visible(&BTreeSet::from([1])), vec![0, 1, 3]);
        assert_eq!(shape.visible(&BTreeSet::from([0])), vec![0]);
        assert_eq!(shape.visible(&BTreeSet::new()), vec![0, 1, 2, 3]);
    }
}
