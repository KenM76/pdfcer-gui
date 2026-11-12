//! The Objects tree follows the selection: when the objects selected on the
//! page change other than by a click in the tree, the layer headings over
//! them unfold and the list scrolls the front-most of them into view, unless
//! it is in view already. Design: `DESIGNS.md` § O416.

use super::provider::ObjectModelProvider;
use super::{ObjectTreeRow, layered};
use crate::panels::ObjectTreeUi;

/// Frames after a row click during which a selection change is the tree's
/// own and is not followed: the click's action lands a frame or two later.
const QUIET_FRAMES: u64 = 3;

/// What the tree last followed, kept with the rest of the tree's state.
#[derive(Default)]
pub(crate) struct Followed {
    /// The page and its selected objects last seen.
    seen: Option<(usize, Vec<usize>)>,
    /// The frame of the last row click.
    clicked: Option<u64>,
    /// The list's scroll offset and height last frame, points.
    view: (f32, f32),
}

impl Followed {
    /// A row was clicked on frame `frame`: the change it makes is not
    /// followed.
    pub(crate) fn row_clicked(&mut self, frame: u64) {
        self.clicked = Some(frame);
    }

    /// The list was drawn scrolled to `offset` and `high` points tall.
    pub(crate) fn drawn(&mut self, offset: f32, high: f32) {
        self.view = (offset, high);
    }
}

/// When `selected`, the objects selected on `page`, differ from last frame's
/// and the change is not the tree's own, unfold the layers holding them;
/// whether to scroll to them.
pub(super) fn unfold(
    tree: &mut ObjectTreeUi,
    provider: &ObjectModelProvider,
    (page, frame): (usize, u64),
    selected: &[usize],
) -> bool {
    let now = Some((page, selected.to_vec()));
    if tree.followed.seen == now {
        return false;
    }
    tree.followed.seen = now;
    let own = tree
        .followed
        .clicked
        .is_some_and(|at| frame.saturating_sub(at) <= QUIET_FRAMES);
    if own || selected.is_empty() {
        return false;
    }
    for (group, members) in layered::groups(provider).unwrap_or_default() {
        if members.iter().any(|m| selected.contains(m)) {
            tree.layers_collapsed.remove(&group);
        }
    }
    true
}

/// The scroll offset that centres the front-most selected row, `step`
/// points per row, or `None` when that row shows already or is not listed.
pub(super) fn offset(
    followed: &Followed,
    rows: &[ObjectTreeRow],
    selected: &[usize],
    step: f32,
) -> Option<f32> {
    let row = rows
        .iter()
        .position(|r| matches!(r, ObjectTreeRow::Object { index } if selected.contains(index)))?;
    let at = row as f32 * step;
    let (top, high) = followed.view;
    let shows = at >= top && at + step <= top + high;
    let offset = (at - (high - step) / 2.0).max(0.0);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("objects-follow row={row} shows={shows} offset={offset:.1}")
    });
    (!shows).then_some(offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<ObjectTreeRow> {
        (0..100)
            .rev()
            .map(|index| ObjectTreeRow::Object { index })
            .collect()
    }

    #[test]
    fn a_row_out_of_view_is_centred_and_one_in_view_is_left() {
        let mut f = Followed::default();
        f.drawn(0.0, 200.0);
        // Object 49 is row 50 of a front-most-first list.
        assert_eq!(offset(&f, &rows(), &[49], 20.0), Some(910.0));
        assert_eq!(offset(&f, &rows(), &[98], 20.0), None);
    }

    #[test]
    fn an_unlisted_object_scrolls_nowhere() {
        assert_eq!(offset(&Followed::default(), &rows(), &[500], 20.0), None);
    }
}
