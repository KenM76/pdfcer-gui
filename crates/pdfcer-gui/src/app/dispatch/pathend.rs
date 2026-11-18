//! # `app::dispatch::pathend` — Format ▸ Nodes ▸ Continue path and Open path
//! here, on the one selected node of a path
//!
//! Open path here raises its edit at once. Continue path arms the canvas run
//! in `canvas::pathextend`, which raises the edit when the operator finishes.
//! A refusal the engine owns (not an end node, a closed or already open
//! subpath) is said when the edit is attempted.

use crate::app::PdfcerApp;
use crate::app::actions::{Action, VectorAction};
use crate::app::dispatch::nodeshape::shapeable_nodes;
use crate::app::state::{OpenDoc, Status};
use crate::canvas::target::TargetId;
use crate::text::pathedit::PathEditRefusal;
use pdfcer_gui_base::subactions::{NodeHost, PathEdit};

/// `format.path_continue`
pub(crate) const CONTINUE: &str = "format.path_continue"; // ui-text-exempt: a command id
/// `format.path_open`
pub(crate) const OPEN: &str = "format.path_open"; // ui-text-exempt: a command id

/// Whether this module owns `id`.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    id == CONTINUE || id == OPEN
}

/// Dispatch one of the two.
pub(crate) fn dispatch(app: &PdfcerApp, ctx: &egui::Context, id: &str, actions: &mut Vec<Action>) {
    let picked = match &app.status {
        Status::Open(doc) if app.capabilities().edit_content => {
            shapeable_nodes(doc).map(|picked| (doc, picked))
        }
        _ => None,
    };
    let Some((doc, (page, host, nodes))) = picked else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=no-nodes-selected")
        });
        return;
    };
    let [node] = nodes[..] else {
        crate::app::status::decline::record_path_edit(PathEditRefusal::NotOneNode);
        return;
    };
    if id == OPEN {
        actions.push(Action::Vector(VectorAction::PathEnd {
            page,
            host,
            node,
            edit: PathEdit::Open,
        }));
        return;
    }
    match node_point(doc, host, node) {
        Some(start) => crate::canvas::pathextend::arm(ctx, page, host, node, start),
        None => crate::app::status::decline::record_path_edit(PathEditRefusal::Stale),
    }
}

/// Where node `node` of `host` sits, PDF page space.
fn node_point(doc: &OpenDoc, host: NodeHost, node: usize) -> Option<(f64, f64)> {
    let target = match host {
        NodeHost::Page(o) => TargetId::Object(o as u64),
        NodeHost::Leaf(l) => TargetId::Leaf(l as u64),
    };
    let provider = doc.page_objects()?;
    provider
        .object_node_points_of(target)
        .into_iter()
        .find(|&(index, _)| index == node)
        .map(|(_, p)| (p.x, p.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ids_are_claimed_and_no_neighbour_is() {
        assert!(claims(CONTINUE) && claims(OPEN));
        assert!(!claims("format.node_insert"));
        assert!(!crate::app::dispatch::nodeshape::claims(CONTINUE));
        assert!(!crate::app::dispatch::nodeshape::claims(OPEN));
    }
}
