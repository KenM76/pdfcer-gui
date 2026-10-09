//! Commit `VectorAction::NodeShape`: add a node after each selected node, or
//! convert each selected node or the segment leaving it, as one undo entry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/vector/nodeshape.md`.

use pdfcer_core::edit::{CommandKind, EditError, EditSession};
use pdfcer_core::vector::{NodeKind, SegmentKind, VectorEditError};

use super::fold_undo;
use crate::app::actions::apply::vector_edit_on_page;
use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;
use crate::diag::index_list as list;
use crate::text::nodeshape as words;
use pdfcer_gui_base::subactions::{NodeHost, NodeShape};

/// Where on the segment a new node goes: halfway.
const MIDDLE: f64 = 0.5;

/// Apply `shape` at every node of `nodes` (object-scoped, ascending) on the
/// path `host` of page `page`.
pub(super) fn apply(
    doc: &mut OpenDoc,
    page: usize,
    host: NodeHost,
    nodes: &[usize],
    shape: NodeShape,
) {
    if nodes.is_empty() {
        return;
    }
    let mut done: Vec<usize> = Vec::new();
    vector_edit_on_page(doc, label(shape), page, nodes.len(), |session| {
        let mut said = Vec::new();
        let mut refusals: Vec<EditError> = Vec::new();
        // Descending, so an insert never renumbers a node still to visit.
        for &node in nodes.iter().rev() {
            match one(session, page, host, node, shape) {
                Ok(lines) => {
                    said.extend(lines);
                    done.push(node);
                }
                Err(why) => refusals.push(why),
            }
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "node-shape-applied page={page} host={} shape={} asked={} done={} refused={}",
                host_token(host),
                token(shape),
                list(nodes),
                list(&ascending(&done)),
                refusals.len(),
            )
        });
        if done.is_empty() {
            return Err(refusals.swap_remove(0));
        }
        if done.len() > 1 {
            fold_undo(session, done.len(), kind(shape), &mut said);
        }
        if let Some(why) = refusals.first() {
            said.push(words::nodes_skipped(refusals.len(), reason(why)));
        }
        Ok(said)
    });
    if shape == NodeShape::Insert && !done.is_empty() {
        select_inserted(doc, page, host, &done);
    }
}

/// One node's verb, in the host's index space.
fn one(
    session: &mut EditSession,
    page: usize,
    host: NodeHost,
    node: usize,
    shape: NodeShape,
) -> Result<Vec<String>, EditError> {
    match (host, shape) {
        (NodeHost::Page(o), NodeShape::Insert) => session.insert_node(page, o, node, MIDDLE),
        (NodeHost::Page(o), NodeShape::Line | NodeShape::Curve) => {
            session.convert_segment(page, o, node, segment_kind(shape))
        }
        (NodeHost::Page(o), _) => session.convert_node(page, o, node, node_kind(shape)),
        (NodeHost::Leaf(l), NodeShape::Insert) => session
            .insert_node_in_form(page, l, node, MIDDLE)
            .map(|outcome| outcome.disclosures),
        (NodeHost::Leaf(l), NodeShape::Line | NodeShape::Curve) => session
            .convert_segment_in_form(page, l, node, segment_kind(shape))
            .map(|outcome| outcome.disclosures),
        (NodeHost::Leaf(l), _) => session
            .convert_node_in_form(page, l, node, node_kind(shape))
            .map(|outcome| outcome.disclosures),
    }
}

/// Select the nodes an insert added. `done` is descending; the node added
/// after the `i`-th smallest is `node + i + 1`, because each smaller one's
/// insert shifted it up by one.
fn select_inserted(doc: &mut OpenDoc, page: usize, host: NodeHost, done: &[usize]) {
    let object = match host {
        NodeHost::Page(o) => TargetId::Object(o as u64),
        NodeHost::Leaf(l) => TargetId::Leaf(l as u64),
    };
    let picks: Vec<(Option<usize>, usize)> = done
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &node)| (doc.selection.part_of_node(page, object, node), node + i + 1))
        .collect();
    doc.selection
        .select_nodes(page, object, &picks, "node-inserted");
}

/// Why a node was skipped, in the words the status line uses.
fn reason(why: &EditError) -> words::Skipped {
    match why {
        EditError::VectorEdit(VectorEditError::NodeHasOneSide { .. }) => words::Skipped::EndNode,
        EditError::VectorEdit(VectorEditError::NoSegmentHere { .. }) => words::Skipped::NoSegment,
        _ => words::Skipped::Other,
    }
}

const fn node_kind(shape: NodeShape) -> NodeKind {
    match shape {
        NodeShape::Smooth => NodeKind::Smooth,
        NodeShape::Symmetric => NodeKind::Symmetric,
        _ => NodeKind::Corner,
    }
}

const fn segment_kind(shape: NodeShape) -> SegmentKind {
    match shape {
        NodeShape::Curve => SegmentKind::Curve,
        _ => SegmentKind::Line,
    }
}

const fn kind(shape: NodeShape) -> CommandKind {
    match shape {
        NodeShape::Insert => CommandKind::InsertNode,
        NodeShape::Line | NodeShape::Curve => CommandKind::ConvertSegment,
        _ => CommandKind::ConvertNode,
    }
}

/// The funnel's label for each verb.
const fn label(shape: NodeShape) -> &'static str {
    // ui-text-exempt: funnel labels, never displayed.
    match shape {
        NodeShape::Insert => "insert-node",
        NodeShape::Line | NodeShape::Curve => "convert-segment",
        _ => "convert-node",
    }
}

const fn token(shape: NodeShape) -> &'static str {
    // ui-text-exempt: trace tokens, never displayed.
    match shape {
        NodeShape::Insert => "insert",
        NodeShape::Corner => "corner",
        NodeShape::Smooth => "smooth",
        NodeShape::Symmetric => "symmetric",
        NodeShape::Line => "line",
        NodeShape::Curve => "curve",
    }
}

fn host_token(host: NodeHost) -> String {
    // ui-text-exempt: trace tokens, never displayed.
    match host {
        NodeHost::Page(o) => format!("object:{o}"),
        NodeHost::Leaf(l) => format!("leaf:{l}"),
    }
}

fn ascending(nodes: &[usize]) -> Vec<usize> {
    let mut sorted = nodes.to_vec();
    sorted.sort_unstable();
    sorted
}
