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
pub(crate) fn dispatch(
    app: &mut PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    actions: &mut Vec<Action>,
) {
    let editable = app.capabilities().edit_content;
    let picked = match &mut app.status {
        Status::Open(doc) if editable => shapeable_nodes(doc).map(|picked| (doc, picked)),
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
    if let Err(why) = preflight(doc, page, host, node, id == OPEN) {
        crate::diag::trace(|| format!("path-end-preflight id={id} node={node} ok=false"));
        crate::app::status::decline::record_path_edit(why);
        return;
    }
    crate::diag::trace(|| format!("path-end-preflight id={id} node={node} ok=true"));
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
/// Whether the engine would take the edit at `node`, asked before a run of
/// clicks is armed so a refusal comes at the press, not after the drawing. A
/// session still held by a render is not asked; the commit then says any
/// refusal.
fn preflight(
    doc: &mut OpenDoc,
    page: usize,
    host: NodeHost,
    node: usize,
    open: bool,
) -> Result<(), PathEditRefusal> {
    let Some(session) = crate::app::actions::funnel::session_to_ask(doc) else {
        return Ok(());
    };
    let answer = match (host, open) {
        (NodeHost::Page(o), false) => session.can_extend_path(page, o, node),
        (NodeHost::Page(o), true) => session.can_open_subpath(page, o, node),
        (NodeHost::Leaf(l), false) => session.can_extend_path_in_form(page, l, node),
        (NodeHost::Leaf(l), true) => session.can_open_subpath_in_form(page, l, node),
    };
    answer.map_err(|why| PathEditRefusal::of_edit(&why))
}

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
