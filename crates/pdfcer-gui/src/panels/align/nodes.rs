//! # `panels::align::nodes` — the panel in node mode.
//!
//! With two or more anchors of one path selected at the Node rung, the
//! panel offers Inkscape's node rows instead of the object ones: align the
//! anchors on a vertical or horizontal line, or spread them evenly. The
//! anchors are measured in canvas space, moved by
//! `alignlayout::rearrange::{align_nodes, distribute_nodes}`, and committed
//! as one [`VectorAction::MoveNodes`] — one undo step.
//!
//! `EditSession::move_nodes` moves anchors of one object per call, so node
//! mode acts on the entered object's anchors; anchors Shift-selected on a
//! second path are not moved, and the panel says so.

use egui::{Pos2, Ui, Vec2};
use pdfcer_core::vector::Point;
use pdfcer_gui_base::alignlayout::Axis;
use pdfcer_gui_base::alignlayout::rearrange::{self, NodeRelative};

use super::button;
use crate::app::actions::{Action, VectorAction};
use crate::app::state::OpenDoc;
use crate::canvas::selection::SelectionLevel;
use crate::icons::Icon;
use crate::text::panels::align as t;

/// The node rows' buttons: align on a vertical line, on a horizontal line,
/// distribute horizontally, distribute vertically.
pub const NODE_REGIONS: [&str; 4] = [
    "align.node.0",
    "align.node.1",
    "align.node.2",
    "align.node.3", // ui-text-exempt: diagnostic region names
];
/// The node buttons' glyphs, in [`NODE_REGIONS`] order.
const NODE_ICONS: [Icon; 4] = [
    Icon::NodesAlignVertical,
    Icon::NodesAlignHorizontal,
    Icon::NodesSpreadAcross,
    Icon::NodesSpreadDown,
];

/// One node button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeOp {
    Align(Axis),
    Distribute(Axis),
}

impl NodeOp {
    const ALL: [NodeOp; 4] = [
        NodeOp::Align(Axis::X),
        NodeOp::Align(Axis::Y),
        NodeOp::Distribute(Axis::X),
        NodeOp::Distribute(Axis::Y),
    ];
}

/// The anchors node mode acts on: the entered object's, in selection order,
/// with their page positions and how many selected anchors lie elsewhere.
pub struct Nodes {
    pub page: usize,
    pub object: usize,
    pub nodes: Vec<(usize, Point)>,
    pub elsewhere: usize,
}

/// Measure the node selection, or `None` when the selection is not at the
/// Node rung on a page object of the current page.
#[must_use]
pub fn measure(doc: &OpenDoc) -> Option<Nodes> {
    let page = doc.view.page_index;
    if doc.selection.level() != SelectionLevel::Node {
        return None;
    }
    let entered = doc.selection.entered_object()?;
    let object = entered.object.page_object_index()?;
    let provider = doc.page_objects()?;
    let points = provider.object_node_points(object);
    let mut nodes: Vec<(usize, Point)> = Vec::new();
    let mut elsewhere = 0;
    for e in doc.selection.in_selection_order() {
        let Some(node) = e.node else { continue };
        if e.page != page || e.object != entered.object {
            elsewhere += 1;
            continue;
        }
        if nodes.iter().any(|(i, _)| *i == node) {
            continue;
        }
        let at = points
            .iter()
            .find_map(|(i, p)| (*i == node).then_some(*p))?;
        nodes.push((node, at));
    }
    (!nodes.is_empty()).then_some(Nodes {
        page,
        object,
        nodes,
        elsewhere,
    })
}

/// A node button's action, or `None` when nothing would move.
#[must_use]
pub fn plan(doc: &OpenDoc, m: &Nodes, op: NodeOp, rel: NodeRelative) -> Option<Action> {
    let sheet = doc.pages.get(m.page)?;
    // Canvas space is f32.
    #[allow(clippy::cast_possible_truncation)]
    let canvas: Vec<(f64, f64)> = m
        .nodes
        .iter()
        .map(|(_, p)| {
            let c = crate::viewer::pdf_space_to_canvas(Pos2::new(p.x as f32, p.y as f32), sheet)?;
            Some((f64::from(c.x), f64::from(c.y)))
        })
        .collect::<Option<_>>()?;
    let deltas = match op {
        NodeOp::Align(axis) => rearrange::align_nodes(&canvas, axis, rel),
        NodeOp::Distribute(axis) => rearrange::distribute_nodes(&canvas, axis),
    };
    let moves: Vec<(usize, Point)> = m
        .nodes
        .iter()
        .zip(canvas.iter().zip(deltas))
        .filter(|(_, (_, d))| !pdfcer_gui_base::alignlayout::negligible(*d))
        .filter_map(|((i, p), (_, (dx, dy)))| {
            // Canvas space is f32.
            #[allow(clippy::cast_possible_truncation)]
            let d = crate::canvas::moving::page_delta(Vec2::new(dx as f32, dy as f32), sheet)?;
            Some((*i, Point::new(p.x + d.dx, p.y + d.dy)))
        })
        .collect();
    (!moves.is_empty()).then(|| {
        VectorAction::MoveNodes {
            page: m.page,
            object: m.object,
            moves,
        }
        .into()
    })
}

/// Draw the node rows.
pub fn body(ui: &mut Ui, doc: &OpenDoc, rel: &mut NodeRelative, actions: &mut Vec<Action>) {
    let m = measure(doc);
    let n = m.as_ref().map_or(0, |m| m.nodes.len());
    ui.label(t::nodes_selected(n));
    if let Some(m) = &m
        && m.elsewhere > 0
    {
        ui.label(t::nodes_elsewhere(m.elsewhere));
    }
    ui.horizontal(|ui| {
        ui.label(t::relative_to());
        egui::ComboBox::from_id_salt("align.node.relative") // ui-text-exempt: widget id
            .selected_text(t::node_relative(*rel))
            .show_ui(ui, |ui| {
                for choice in [
                    NodeRelative::Last,
                    NodeRelative::First,
                    NodeRelative::Middle,
                    NodeRelative::Min,
                    NodeRelative::Max,
                ] {
                    ui.selectable_value(rel, choice, t::node_relative(choice));
                }
            });
    });
    let mut pressed = None;
    ui.horizontal_wrapped(|ui| {
        for (index, op) in NodeOp::ALL.into_iter().enumerate() {
            let (label, tip) = t::node_button(index);
            if button(
                ui,
                n >= 2,
                NODE_ICONS[index],
                label,
                tip,
                NODE_REGIONS[index],
            ) {
                pressed = Some(op);
            }
        }
    });
    let (Some(op), Some(m)) = (pressed, m) else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("align-pressed op=Node({op:?}) n={n} relative={rel:?}")
    });
    match plan(doc, &m, op, *rel) {
        Some(action) => actions.push(action),
        None => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("align-declined op=Node({op:?}) reason=nothing-moves n={n}")
            });
            crate::app::actions::record_note(doc.edit_epoch, t::already_aligned().to_owned());
        }
    }
}
