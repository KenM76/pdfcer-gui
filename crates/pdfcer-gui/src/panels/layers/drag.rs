//! # `panels::layers::drag` — dragging a layer or folder to a new place
//!
//! Contract: a drag starts on a row's name and is remembered by the entry's
//! path. While it is in flight the pointer's row is split into the three
//! bands every tree control uses (`bookmarks::reorder::band_at`): the top
//! quarter lands before the row, the middle half inside it at the end, the
//! bottom quarter after it and its open subtree. The caret is drawn with
//! `bookmarks::reorder::paint_line`, dimmed when the drop changes nothing and
//! fainter still when the entry would go inside itself. Releasing raises one
//! [`OrderAction::Move`]; a drop that changes nothing raises nothing, and one
//! into itself is raised so the refusal is said.

use egui::{Rect, Ui};
use pdfcer_gui_base::layeraction::{LayerAction, OrderAction};
use pdfcer_gui_base::layerorder::{self as order, Move, Node, Resolved};

use crate::app::actions::Action;
use crate::panels::bookmarks::reorder::{self, Band, Landing};

/// Where the dragged entry's path is kept while the button is held.
// ui-text-exempt: a memory key, never displayed.
const DRAG_KEY: &str = "panel-layers-drag";

/// The region the insertion caret publishes.
// ui-text-exempt: trace region name, never displayed.
pub const REGION_CARET: &str = "panel.layers.drop-caret";

/// One row as it was drawn, in draw order.
pub(super) struct Drawn {
    /// The entry's path in the tree.
    pub path: Vec<usize>,
    /// The full-width strip the row occupies.
    pub rect: Rect,
    /// Where the row's own content starts horizontally.
    pub indent_left: f32,
}

/// Begin a drag from `name` if one starts there and none is in flight.
pub(super) fn begin(ui: &Ui, name: &egui::Response, path: &[usize]) {
    if !name.drag_started_by(egui::PointerButton::Primary) || dragging(ui).is_some() {
        return;
    }
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new(DRAG_KEY), path.to_vec()));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("layer-drag-begin path={}", order::dotted(path))
    });
}

fn dragging(ui: &Ui) -> Option<Vec<usize>> {
    ui.ctx()
        .data(|d| d.get_temp::<Vec<usize>>(egui::Id::new(DRAG_KEY)))
}

/// Draw the caret for a drag in flight and, on release, raise the move.
pub(super) fn finish(ui: &Ui, tree: &[Node], rows: &[Drawn], actions: &mut Vec<Action>) {
    let Some(from) = dragging(ui) else {
        return;
    };
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    let target = ui.ctx().pointer_latest_pos().and_then(|p| {
        let span = (ui.max_rect().right(), ui.spacing().indent);
        target_at(tree, rows, &from, p.y, span)
    });
    if let Some((_, caret, landing)) = &target {
        reorder::paint_line(ui, *caret, *landing);
        crate::diag::ui_rect_visible(REGION_CARET, caret.expand(2.0), ui.clip_rect());
    }
    if !ui
        .ctx()
        .input(|i| i.pointer.button_released(egui::PointerButton::Primary))
    {
        return;
    }
    ui.ctx()
        .data_mut(|d| d.remove::<Vec<usize>>(egui::Id::new(DRAG_KEY)));
    crate::diag::trace(|| match &target {
        // ui-text-exempt: diagnostic trace, never displayed
        Some((m, _, landing)) => format!(
            "layer-drop from={} parent={} index={} landing={landing:?}",
            order::dotted(&m.from),
            order::dotted(&m.parent),
            m.index
        ),
        None => format!("layer-drop from={} landing=none", order::dotted(&from)),
    });
    if let Some((m, _, Landing::Lands | Landing::OwnSubtree)) = target {
        actions.push(Action::Layer(LayerAction::Order(OrderAction::Move(m))));
    }
}

/// The move, the caret and what the drop would do, for a pointer at `y`.
fn target_at(
    tree: &[Node],
    rows: &[Drawn],
    from: &[usize],
    y: f32,
    (right, indent): (f32, f32),
) -> Option<(Move, Rect, Landing)> {
    let i = rows.iter().position(|r| r.rect.y_range().contains(y))?;
    let row = rows.get(i)?;
    let (&last, parent) = row.path.split_last()?;
    let mv = |parent: &[usize], index| Move {
        from: from.to_vec(),
        parent: parent.to_vec(),
        index,
    };
    let (m, caret_y, left) = match reorder::band_at(row.rect, y) {
        Band::Before => (mv(parent, last), row.rect.top(), row.indent_left),
        Band::Into => {
            let held = order::node_at(tree, &row.path).map_or(0, |n| n.children.len());
            let bottom = reorder::subtree_bottom_by(rows, i, |r| r.path.len(), |r| r.rect.bottom());
            (mv(&row.path, held), bottom, row.indent_left + indent)
        }
        Band::After => {
            let bottom = reorder::subtree_bottom_by(rows, i, |r| r.path.len(), |r| r.rect.bottom());
            (mv(parent, last + 1), bottom, row.indent_left)
        }
    };
    let landing = match order::engine_move(tree, &m) {
        Resolved::Engine { .. } => Landing::Lands,
        Resolved::Unchanged => Landing::NoChange,
        Resolved::IntoItself => Landing::OwnSubtree,
        Resolved::NotFound => return None,
    };
    let caret = Rect::from_min_max(
        egui::pos2(left, caret_y),
        egui::pos2(right.max(left), caret_y),
    );
    Some((m, caret, landing))
}
