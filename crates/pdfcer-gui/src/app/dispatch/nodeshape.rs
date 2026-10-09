//! # `app::dispatch::nodeshape` — Format ▸ Nodes: add a node, and make the
//! selected nodes corner, smooth or symmetric and the segments leaving them
//! lines or curves
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/nodeshape.md`.

use crate::app::PdfcerApp;
use crate::app::actions::{Action, VectorAction};
use crate::app::state::{OpenDoc, Status};
use crate::canvas::selection::SelectionLevel;
use pdfcer_gui_base::subactions::{NodeHost, NodeShape};

/// The six command ids and what each does.
#[must_use]
pub(crate) fn shape_of(id: &str) -> Option<NodeShape> {
    match id {
        "format.node_insert" => Some(NodeShape::Insert),
        "format.node_corner" => Some(NodeShape::Corner),
        "format.node_smooth" => Some(NodeShape::Smooth),
        "format.node_symmetric" => Some(NodeShape::Symmetric),
        "format.segment_line" => Some(NodeShape::Line),
        "format.segment_curve" => Some(NodeShape::Curve),
        _ => None,
    }
}

/// Whether this module owns `id`.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    shape_of(id).is_some()
}

/// The page, the path and its selected nodes a Nodes command acts on: the
/// Node rung, on a page path or a path inside a placed drawing. Only a path
/// has nodes, so a non-empty node set is the test.
#[must_use]
pub(crate) fn shapeable_nodes(doc: &OpenDoc) -> Option<(usize, NodeHost, Vec<usize>)> {
    if doc.selection.level() != SelectionLevel::Node {
        return None;
    }
    let entry = doc.selection.entered_object()?;
    let host = match entry.object.leaf_index() {
        Some(leaf) => NodeHost::Leaf(leaf),
        None => NodeHost::Page(entry.object.page_object_index()?),
    };
    let nodes = doc.selection.selected_nodes_on(entry.page, entry.object);
    (!nodes.is_empty()).then_some((entry.page, host, nodes))
}

/// Dispatch one of the six.
pub(crate) fn dispatch(app: &PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    let Some(shape) = shape_of(id) else {
        return;
    };
    if let Status::Open(doc) = &app.status
        && app.capabilities().edit_content
        && let Some((page, host, nodes)) = shapeable_nodes(doc)
    {
        actions.push(Action::Vector(VectorAction::NodeShape {
            page,
            host,
            nodes,
            shape,
        }));
        return;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("command-declined id={id} reason=no-nodes-selected")
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDS: [&str; 6] = [
        "format.node_insert",
        "format.node_corner",
        "format.node_smooth",
        "format.node_symmetric",
        "format.segment_line",
        "format.segment_curve",
    ];

    #[test]
    fn every_id_is_claimed_and_none_shares_a_shape() {
        let shapes: Vec<NodeShape> = IDS.iter().filter_map(|id| shape_of(id)).collect();
        assert_eq!(shapes.len(), IDS.len());
        for (i, a) in shapes.iter().enumerate() {
            assert!(!shapes[i + 1..].contains(a), "{a:?} twice");
        }
        assert!(!claims("format.node_tools"));
    }
}
