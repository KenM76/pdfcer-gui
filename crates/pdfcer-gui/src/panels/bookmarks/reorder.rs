//! # `panels::bookmarks::reorder` — dragging a bookmark to a new place, and
//! the triangle that opens or closes one
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/bookmarks/reorder.md`.

use egui::{Pos2, Rect, Ui};
use pdfcer_core::edit::OutlinePlacement;
use pdfcer_core::object::ObjId;
use pdfcer_core::outline::OutlineItem;

use crate::app::actions::Action;
use crate::app::actions::bookmarks::BookmarkAction;

/// The region name the insertion caret publishes.
pub const REGION_CARET: &str = "bookmarks.drop-caret"; // ui-text-exempt: trace region name, never displayed

/// The prefix of the per-row disclosure-triangle regions; the item's object
/// **number** is appended.
pub const REGION_DISCLOSE_PREFIX: &str = "bookmarks.disclose."; // ui-text-exempt: trace region name, never displayed

/// How thick the insertion caret is drawn, in points.
const CARET_PTS: f32 = 2.0;

/// How much of the caret's colour survives when the drop would change nothing.
const CARET_DIMMED: f32 = 0.35;

/// How much survives when the drop would be **refused**.
const CARET_REFUSED: f32 = 0.15;

/// One row of the outline **as it was actually drawn**, in draw order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibleRow {
    /// Which bookmark this row is.
    pub id: ObjId,
    /// Its nesting depth: `0` for a top-level bookmark, matching
    /// `OutlineItem::level`.
    pub level: usize,
    /// The full-width strip the row occupies, in the scroll area's own
    /// coordinate space. The **band test** reads its vertical extent; the caret
    /// reads its horizontal one.
    pub rect: Rect,
    /// Where this row's own content begins horizontally — the left edge of its
    /// disclosure triangle. This is the x a caret at *this row's depth* starts
    /// from, and it is measured rather than computed from `level` so an indent
    /// the theme changes cannot make the mark and the rows disagree.
    pub indent_left: f32,
    /// Whether this bookmark is **open**, from `OutlineItem::open` — the
    /// shell's read of `/Count`'s sign. A row that is closed drew no children,
    /// so its subtree run in this list is empty.
    pub open: bool,
    /// Whether it has any children at all, from the tree rather than from
    /// `/Count`. Decides whether a triangle is drawn: a leaf carries no
    /// `/Count` and has no open-or-closed state to set.
    pub has_children: bool,
}

/// Which third of a row the pointer is in, and therefore which placement.
///
/// The conventional three-band split every tree control uses. See the module
/// header's table, and its note on why the middle band is `LastChild`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// The top quarter — land in front of this row, beside it.
    Before,
    /// The middle half — land inside this row, at the end of whatever is
    /// already there.
    Into,
    /// The bottom quarter — land behind this row, beside it.
    After,
}

/// Fraction of a row's height each edge band occupies.
const EDGE_BAND: f32 = 0.25;

/// Which band `y` falls in, within `rect`.
#[must_use]
pub fn band_at(rect: Rect, y: f32) -> Band {
    let height = rect.height();
    if height <= 0.0 {
        return Band::Into;
    }
    let edge = height * EDGE_BAND;
    if y < rect.top() + edge {
        Band::Before
    } else if y > rect.bottom() - edge {
        Band::After
    } else {
        Band::Into
    }
}

/// The bottom of `rows[index]`'s **subtree**, as it was drawn.
#[must_use]
pub fn subtree_bottom(rows: &[VisibleRow], index: usize) -> f32 {
    subtree_bottom_by(rows, index, |r| r.level, |r| r.rect.bottom())
}

/// [`subtree_bottom`] for any drawn tree: `depth` and `bottom` read a row.
#[must_use]
pub fn subtree_bottom_by<R>(
    rows: &[R],
    index: usize,
    depth: impl Fn(&R) -> usize,
    bottom: impl Fn(&R) -> f32,
) -> f32 {
    let Some(anchor) = rows.get(index) else {
        return 0.0;
    };
    let level = depth(anchor);
    let mut lowest = bottom(anchor);
    for row in &rows[index.saturating_add(1)..] {
        if depth(row) <= level {
            break;
        }
        lowest = bottom(row);
    }
    lowest
}

/// What releasing on a landing would do — the three answers the caret has to
/// be able to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing {
    /// The bookmark would move. The caret is drawn at full strength.
    Lands,
    /// The bookmark is already there. Dimmed; the release raises nothing and
    /// says nothing.
    NoChange,
    /// The destination is the bookmark itself or somewhere inside it. Fainter
    /// still; the release **raises the move anyway**, so the engine can refuse
    /// it and the refusal can be worded.
    OwnSubtree,
}

/// Where a drag in flight would land, resolved during the layout pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropTarget {
    /// The placement a release would ask the engine for.
    pub placement: OutlinePlacement,
    /// The line to draw, in the scroll area's own coordinate space. Its left
    /// edge carries the destination **depth**; see the module header.
    pub caret: Rect,
    /// What releasing here would actually do.
    pub landing: Landing,
}

/// Where a bookmark sits in the tree — its parent, its siblings, and its place
/// among them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// The bookmark it is filed under, or `None` for the top level — which is
    /// [`OutlinePlacement::FirstChild`]'s and `add_outline_item`'s own spelling
    /// for the outline root.
    pub parent: Option<ObjId>,
    /// Its siblings, in document order, including itself.
    pub siblings: Vec<ObjId>,
    /// Its index within [`Self::siblings`].
    pub index: usize,
}

/// Find `id`'s [`Location`] in the real outline.
///
/// One line, because the recursion — the part with something to get wrong —
/// lives in [`locate_in`], which **can** be tested. Same split, same reason, as
/// [`super::tree::find`]: `OutlineItem` is `#[non_exhaustive]` and a walk
/// written directly over it is a walk no test in this crate can reach.
#[must_use]
pub fn locate(items: &[OutlineItem], id: ObjId) -> Option<Location> {
    locate_in(
        items,
        None,
        id,
        |item| item.id,
        |item| item.children.as_slice(),
    )
}

/// Depth-first search for a node's parent, siblings and index.
pub fn locate_in<'a, T>(
    items: &'a [T],
    parent: Option<ObjId>,
    id: ObjId,
    id_of: impl Fn(&T) -> ObjId + Copy,
    children: impl Fn(&'a T) -> &'a [T] + Copy,
) -> Option<Location> {
    if let Some(index) = items.iter().position(|item| id_of(item) == id) {
        return Some(Location {
            parent,
            siblings: items.iter().map(id_of).collect(),
            index,
        });
    }
    for item in items {
        if let Some(found) = locate_in(children(item), Some(id_of(item)), id, id_of, children) {
            return Some(found);
        }
    }
    None
}

/// **Would this move do anything, and would pdfcer allow it?**
#[must_use]
pub fn landing_for(items: &[OutlineItem], dragged: ObjId, to: OutlinePlacement) -> Landing {
    let anchor = match to {
        OutlinePlacement::Before { sibling } | OutlinePlacement::After { sibling } => Some(sibling),
        OutlinePlacement::FirstChild { parent } | OutlinePlacement::LastChild { parent } => parent,
        // `OutlinePlacement` is `#[non_exhaustive]`. A variant this build has
        // never seen cannot be reasoned about, so it gets the answer that asks
        // the engine rather than the answer that quietly refuses — the panel
        // constructs every placement it uses, so this arm is unreachable today
        // and must not become a silent veto if that stops being true.
        _ => return Landing::Lands,
    };
    // The refusal first, because it outranks the no-op: dropping a bookmark on
    // itself is both "already there" and "inside itself", and the sentence the
    // operator needs is the one that says pdfcer will not do it.
    if let Some(anchor) = anchor
        && (anchor == dragged || is_inside(items, dragged, anchor))
    {
        return Landing::OwnSubtree;
    }
    let Some(here) = locate(items, dragged) else {
        // The dragged id no longer resolves — the ordinary state one frame
        // after an undo. Nothing can be forecast about a bookmark that is not
        // there, so the engine is asked and answers `OutlineItemNotFound`.
        return Landing::Lands;
    };
    let unchanged = match to {
        OutlinePlacement::Before { sibling } => here
            .siblings
            .get(here.index.saturating_add(1))
            .is_some_and(|next| *next == sibling),
        OutlinePlacement::After { sibling } => here
            .index
            .checked_sub(1)
            .and_then(|before| here.siblings.get(before))
            .is_some_and(|previous| *previous == sibling),
        OutlinePlacement::LastChild { parent } => {
            here.parent == parent && here.index + 1 == here.siblings.len()
        }
        OutlinePlacement::FirstChild { parent } => here.parent == parent && here.index == 0,
        _ => false,
    };
    if unchanged {
        Landing::NoChange
    } else {
        Landing::Lands
    }
}

/// Is `candidate` somewhere below `ancestor` in the tree?
#[must_use]
fn is_inside(items: &[OutlineItem], ancestor: ObjId, candidate: ObjId) -> bool {
    super::tree::find(items, ancestor)
        .is_some_and(|item| super::tree::find(&item.children, candidate).is_some())
}

/// **Where a drag would land**, from the rows as drawn and the pointer as it
/// is.
#[must_use]
pub fn resolve_at(
    rows: &[VisibleRow],
    items: &[OutlineItem],
    dragged: ObjId,
    pointer: Pos2,
    indent: f32,
    right: f32,
) -> Option<DropTarget> {
    let index = rows.iter().position(|row| row.rect.contains(pointer))?;
    let row = rows[index];
    let band = band_at(row.rect, pointer.y);
    let (placement, y, left) = match band {
        Band::Before => (
            OutlinePlacement::Before { sibling: row.id },
            row.rect.top(),
            row.indent_left,
        ),
        // Both lower bands sit at the END of the row's subtree, and differ only
        // by one indent. See the module header for why that is the whole trick:
        // the mark's height stops moving and its depth is the thing the
        // operator is choosing.
        Band::Into => (
            OutlinePlacement::LastChild {
                parent: Some(row.id),
            },
            subtree_bottom(rows, index),
            row.indent_left + indent,
        ),
        Band::After => (
            OutlinePlacement::After { sibling: row.id },
            subtree_bottom(rows, index),
            row.indent_left,
        ),
    };
    Some(DropTarget {
        placement,
        caret: Rect::from_min_max(Pos2::new(left, y), Pos2::new(right.max(left), y)),
        landing: landing_for(items, dragged, placement),
    })
}

/// [`resolve_at`], with the pointer and the theme read from the `Ui`.
#[must_use]
pub fn resolve(
    ui: &Ui,
    rows: &[VisibleRow],
    items: &[OutlineItem],
    dragged: Option<ObjId>,
) -> Option<DropTarget> {
    let dragged = dragged?;
    let pointer = ui.ctx().pointer_latest_pos()?;
    resolve_at(
        rows,
        items,
        dragged,
        pointer,
        ui.spacing().indent,
        // `max_rect`, not `min_rect`: the caret spans the LIST, and `min_rect`
        // after the rows have been drawn is the bounding box of the titles —
        // so a document of short chapter numbers would get a mark that stopped
        // a third of the way across the panel and read as a hairline artefact.
        // It is the same rectangle the rows were measured against; see
        // `super::body`'s `strip`.
        ui.max_rect().right(),
    )
}

/// Draw the insertion caret for a drag in flight.
pub fn paint_caret(ui: &Ui, target: Option<&DropTarget>) {
    let Some(target) = target else {
        return;
    };
    paint_line(ui, target.caret, target.landing);
    crate::diag::ui_rect_visible(REGION_CARET, target.caret.expand(CARET_PTS), ui.clip_rect());
    crate::diag::trace_changed(CARET_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "bookmark-drop-target placement={} anchor={} landing={:?} caret={:?}",
            placement_word(target.placement),
            anchor_number(target.placement),
            target.landing,
            target.caret,
        )
    });
}

/// The insertion line along `caret`'s top edge, at the strength `landing`
/// earns; shared by every reorderable tree.
pub fn paint_line(ui: &Ui, caret: Rect, landing: Landing) {
    let base = egui_shell::theme::Theme::canvas_selection_ink(ui.ctx());
    let colour = match landing {
        Landing::Lands => base,
        Landing::NoChange => base.gamma_multiply(CARET_DIMMED),
        Landing::OwnSubtree => base.gamma_multiply(CARET_REFUSED),
    };
    ui.painter().line_segment(
        [caret.left_top(), caret.right_top()],
        egui::Stroke::new(CARET_PTS, colour),
    );
}

/// Trace slot for the once-per-change caret line.
const CARET_SLOT: &str = "bookmark-drop-target"; // ui-text-exempt: trace slot name, never displayed

/// The placement as one word, for the trace.
fn placement_word(to: OutlinePlacement) -> &'static str {
    match to {
        // ui-text-exempt: diagnostic trace tokens, never displayed
        OutlinePlacement::Before { .. } => "before",
        OutlinePlacement::After { .. } => "after",
        OutlinePlacement::FirstChild { .. } => "first-child",
        OutlinePlacement::LastChild { .. } => "last-child",
        _ => "unknown",
    }
}

/// The anchor's object number, for the trace, or `0` for the outline root.
fn anchor_number(to: OutlinePlacement) -> u32 {
    match to {
        OutlinePlacement::Before { sibling } | OutlinePlacement::After { sibling } => sibling.num,
        OutlinePlacement::FirstChild { parent } | OutlinePlacement::LastChild { parent } => {
            parent.map_or(0, |id| id.num)
        }
        _ => 0,
    }
}

/// **End a drag** — read the release, raise the move, clear the state.
pub fn settle(
    ui: &Ui,
    ui_state: &mut super::BookmarksUi,
    target: Option<&DropTarget>,
    actions: &mut Vec<Action>,
) {
    let Some(dragged) = ui_state.drag else {
        return;
    };
    // Set every frame of the drag rather than once at the start: egui resolves
    // the cursor per frame from whatever asked most recently, so a request made
    // at `drag_started` would be overwritten by the next widget the pointer
    // passed over. `crate::panels::pages` records the same finding.
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    if !ui
        .ctx()
        .input(|i| i.pointer.button_released(egui::PointerButton::Primary))
    {
        return;
    }
    ui_state.drag = None;
    let Some(target) = target else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("bookmark-drag-released item={} landing=none", dragged.num)
        });
        return;
    };
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "bookmark-drag-released item={} placement={} anchor={} landing={:?}",
            dragged.num,
            placement_word(target.placement),
            anchor_number(target.placement),
            target.landing,
        )
    });
    match target.landing {
        // Both of these raise, and the second is the whole of R83's rule:
        // a refusal must be a SENTENCE, never a silence. See this function's
        // header for why the sentence has to be produced by the apply phase
        // rather than here.
        Landing::Lands | Landing::OwnSubtree => {
            actions.push(Action::Bookmark(BookmarkAction::Move {
                item: dragged,
                to: target.placement,
            }));
        }
        // Nothing, and nothing said. The operator asked for the state they
        // are already in, and the caret was dimmed under their pointer before
        // they let go. Raising the action anyway would be honest — the engine
        // answers `moved: false` and writes nothing — and it would cost a
        // status line saying "nothing changed" for a gesture the panel had
        // already declined to promise anything about. `crate::panels::pages`'
        // release makes the identical call for its own no-op.
        Landing::NoChange => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A row list this crate CAN build, at whatever geometry a test wants.
    fn row(num: u32, level: usize, top: f32, height: f32) -> VisibleRow {
        VisibleRow {
            id: ObjId::new(num, 0),
            level,
            rect: Rect::from_min_max(Pos2::new(0.0, top), Pos2::new(200.0, top + height)),
            #[allow(
                clippy::cast_precision_loss,
                reason = "a nesting level is a small integer; the indent is a test fixture" // ui-text-exempt: clippy lint justification, never displayed
            )]
            indent_left: level as f32 * 10.0,
            open: true,
            has_children: false,
        }
    }

    /// **The three bands are three, and the edges are quarters.**
    #[test]
    fn the_row_splits_into_before_into_and_after() {
        let rect = Rect::from_min_max(Pos2::new(0.0, 100.0), Pos2::new(200.0, 120.0));
        assert_eq!(band_at(rect, 100.0), Band::Before, "the very top");
        assert_eq!(band_at(rect, 104.0), Band::Before, "inside the top quarter");
        assert_eq!(band_at(rect, 106.0), Band::Into, "past it");
        assert_eq!(band_at(rect, 110.0), Band::Into, "dead centre");
        assert_eq!(band_at(rect, 114.0), Band::Into, "still the middle half");
        assert_eq!(band_at(rect, 116.0), Band::After, "into the bottom quarter");
        assert_eq!(band_at(rect, 120.0), Band::After, "the very bottom");
        // The middle band is the widest, which is the conventional weighting.
        let edge = rect.height() * EDGE_BAND;
        assert!(
            rect.height() - 2.0 * edge > edge,
            "the nesting band must be wider than either edge band"
        );
    }

    /// A degenerate row answers the middle rather than inventing a boundary.
    #[test]
    fn a_zero_height_row_is_all_middle() {
        let flat = Rect::from_min_max(Pos2::new(0.0, 50.0), Pos2::new(200.0, 50.0));
        assert_eq!(band_at(flat, 50.0), Band::Into);
    }

    /// **The caret for the lower bands sits at the end of the SUBTREE**,
    /// which is what makes it truthful rather than comfortable.
    #[test]
    fn the_subtree_bottom_is_the_last_descendant_not_the_last_child() {
        let rows = vec![
            row(1, 0, 0.0, 10.0),  // Chapter one
            row(2, 1, 10.0, 10.0), // .. section A
            row(3, 1, 20.0, 10.0), // .. section B
            row(4, 2, 30.0, 10.0), // .... section B.i
            row(5, 0, 40.0, 10.0), // Chapter two
        ];
        assert_eq!(subtree_bottom(&rows, 0), 40.0, "past every descendant");
        assert_ne!(subtree_bottom(&rows, 0), 10.0, "not its own bottom");
        assert_ne!(subtree_bottom(&rows, 0), 30.0, "not its last CHILD's");
        assert_eq!(subtree_bottom(&rows, 2), 40.0, "section B holds one");
        assert_eq!(
            subtree_bottom(&rows, 4),
            50.0,
            "the last row has no subtree and answers its own bottom"
        );
    }

    /// **A collapsed row's caret is at its own edge**, because nothing of it
    /// is drawn.
    #[test]
    fn a_collapsed_rows_caret_is_at_its_own_bottom() {
        // Two top-level rows; the first is collapsed, so nothing of its branch
        // was drawn however large it is.
        let mut rows = vec![row(1, 0, 0.0, 10.0), row(2, 0, 10.0, 10.0)];
        rows[0].open = false;
        rows[0].has_children = true;
        assert_eq!(subtree_bottom(&rows, 0), 10.0);
    }

    // ---------------------------------------------------------------------
    // The landing forecast, over a tree this crate can build
    // ---------------------------------------------------------------------

    /// A node type standing in for `OutlineItem`, which is `#[non_exhaustive]`
    /// and cannot be constructed here. Same device, same reason, as
    /// [`super::super::tree`]'s tests.
    struct Node {
        id: ObjId,
        children: Vec<Node>,
    }

    fn node(num: u32, children: Vec<Node>) -> Node {
        Node {
            id: ObjId::new(num, 0),
            children,
        }
    }

    fn kids(n: &Node) -> &[Node] {
        n.children.as_slice()
    }

    fn locate_node(items: &[Node], num: u32) -> Option<Location> {
        locate_in(items, None, ObjId::new(num, 0), |n| n.id, kids)
    }

    /// **A bookmark's place is its parent, its siblings and its index**, and
    /// all three come from one walk.
    #[test]
    fn a_nodes_place_is_found_at_any_depth() {
        let tree = vec![
            node(1, vec![node(3, vec![]), node(4, vec![node(6, vec![])])]),
            node(2, vec![]),
        ];
        let top = locate_node(&tree, 2).expect("present");
        assert_eq!(top.parent, None, "a top-level item has no parent id");
        assert_eq!(top.index, 1);
        assert_eq!(top.siblings.len(), 2);

        let deep = locate_node(&tree, 6).expect("present");
        assert_eq!(deep.parent, Some(ObjId::new(4, 0)));
        assert_eq!(deep.index, 0);
        assert_eq!(deep.siblings.len(), 1);

        let middle = locate_node(&tree, 4).expect("present");
        assert_eq!(middle.parent, Some(ObjId::new(1, 0)));
        assert_eq!(middle.index, 1, "the second child");

        assert!(locate_node(&tree, 99).is_none());
    }

    /// **The two spellings of a bookmark's own slot are recognised as
    /// no-ops.**
    #[test]
    fn the_slot_a_bookmark_already_occupies_is_recognised_from_both_sides() {
        // Modelled with `locate_in` directly rather than `landing_for`, which
        // needs a real `OutlineItem` tree. The arithmetic under test is the
        // index comparison, and this is where it lives.
        let tree = vec![node(1, vec![]), node(2, vec![]), node(3, vec![])];
        let me = locate_node(&tree, 2).expect("present");
        let next = me.siblings.get(me.index + 1).copied();
        let previous = me
            .index
            .checked_sub(1)
            .and_then(|i| me.siblings.get(i))
            .copied();
        assert_eq!(next, Some(ObjId::new(3, 0)), "Before THIS is a no-op");
        assert_eq!(previous, Some(ObjId::new(1, 0)), "After THIS is a no-op");
        // And the far side of each neighbour is a real move.
        assert_ne!(next, Some(ObjId::new(1, 0)));
        assert_ne!(previous, Some(ObjId::new(3, 0)));
    }

    /// **The first and last child are recognised too**, which is what makes
    /// the middle band's caret dim when a bookmark is dropped back into the
    /// parent it is already the last child of.
    #[test]
    fn being_already_the_last_child_is_a_no_op() {
        let tree = vec![node(1, vec![node(2, vec![]), node(3, vec![])])];
        let last = locate_node(&tree, 3).expect("present");
        assert_eq!(last.parent, Some(ObjId::new(1, 0)));
        assert_eq!(last.index + 1, last.siblings.len(), "already last");
        let first = locate_node(&tree, 2).expect("present");
        assert_eq!(first.index, 0, "already first");
        assert_ne!(
            first.index + 1,
            first.siblings.len(),
            "the fixture must be able to tell first from last"
        );
    }

    /// **The three landings are three distinct answers**, so a match on them
    /// cannot silently collapse.
    #[test]
    fn the_three_landings_are_distinguishable() {
        assert_ne!(Landing::Lands, Landing::NoChange);
        assert_ne!(Landing::NoChange, Landing::OwnSubtree);
        assert_ne!(Landing::Lands, Landing::OwnSubtree);
    }

    /// **The three dimming ratios are three**, and they are ordered.
    #[test]
    fn a_refused_landing_is_fainter_than_one_that_merely_does_nothing() {
        const {
            assert!(CARET_REFUSED < CARET_DIMMED);
            assert!(CARET_DIMMED < 1.0);
            // Faint is not invisible: a caret nobody can see is the "no caret
            // at all" this module's constants exist to refuse.
            assert!(CARET_REFUSED > 0.0);
        }
    }

    /// **The caret's DEPTH is the whole of what distinguishes nesting from
    /// reordering**, and the two lower bands sit at the same height.
    #[test]
    fn the_two_lower_bands_differ_by_an_indent_and_not_by_a_height() {
        let rows = vec![row(1, 0, 0.0, 20.0), row(2, 1, 20.0, 20.0)];
        // No `OutlineItem` tree is constructible here, so the landing forecast
        // degrades to `Lands` (the dragged id does not resolve) — which is
        // exactly what this test wants, since it is about geometry.
        let empty: [OutlineItem; 0] = [];
        let indent = 12.0;
        let into = resolve_at(
            &rows,
            &empty,
            ObjId::new(9, 0),
            Pos2::new(50.0, 10.0),
            indent,
            200.0,
        )
        .expect("the pointer is over row one");
        let after = resolve_at(
            &rows,
            &empty,
            ObjId::new(9, 0),
            Pos2::new(50.0, 19.0),
            indent,
            200.0,
        )
        .expect("the pointer is over row one");
        assert_eq!(placement_word(into.placement), "last-child");
        assert_eq!(placement_word(after.placement), "after");
        assert!(
            (into.caret.top() - after.caret.top()).abs() < f32::EPSILON,
            "the mark must not jump vertically between the two lower bands"
        );
        assert!(
            (into.caret.left() - after.caret.left() - indent).abs() < f32::EPSILON,
            "nesting is exactly one indent deeper than landing beside it"
        );
        // And both are at the end of row one's subtree, which is row two.
        assert!((into.caret.top() - 40.0).abs() < f32::EPSILON);
    }

    /// **The top band's caret is at the row's own top edge and its own
    /// depth**, which is the one landing whose mark is beside the pointer.
    #[test]
    fn the_top_band_marks_the_row_it_is_over() {
        let rows = vec![row(1, 0, 0.0, 20.0), row(2, 1, 20.0, 20.0)];
        let empty: [OutlineItem; 0] = [];
        let before = resolve_at(
            &rows,
            &empty,
            ObjId::new(9, 0),
            Pos2::new(50.0, 22.0),
            12.0,
            200.0,
        )
        .expect("the pointer is over row two");
        assert_eq!(placement_word(before.placement), "before");
        assert_eq!(anchor_number(before.placement), 2);
        assert!((before.caret.top() - 20.0).abs() < f32::EPSILON);
        assert!(
            (before.caret.left() - rows[1].indent_left).abs() < f32::EPSILON,
            "a Before caret sits at the row's OWN depth, not one deeper"
        );
    }

    /// The pointer over no row resolves nothing, and the space below the list
    /// is deliberately not *"the end of the top level"*.
    #[test]
    fn a_pointer_over_no_row_lands_nowhere() {
        let rows = vec![row(1, 0, 0.0, 20.0)];
        let empty: [OutlineItem; 0] = [];
        assert!(
            resolve_at(
                &rows,
                &empty,
                ObjId::new(9, 0),
                Pos2::new(50.0, 400.0),
                12.0,
                200.0
            )
            .is_none()
        );
    }

    /// **The root is spelled `0` in the trace**, which is not a legal object
    /// number and so cannot be read as a real anchor.
    #[test]
    fn the_top_level_anchor_traces_as_zero() {
        assert_eq!(
            anchor_number(OutlinePlacement::LastChild { parent: None }),
            0
        );
        assert_eq!(
            anchor_number(OutlinePlacement::FirstChild {
                parent: Some(ObjId::new(7, 0))
            }),
            7
        );
        assert_eq!(
            placement_word(OutlinePlacement::FirstChild { parent: None }),
            "first-child"
        );
    }
}
