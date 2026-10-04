//! # `app::actions::layerorder` — folders and moves in the Layers panel
//!
//! Contract: each [`OrderAction`] is one `EditSession` arrangement verb
//! (`add_layer_folder`, `rename_layer_folder`, `delete_layer_folder`,
//! `move_layer_node`), so one undo entry that writes `/D /Order` and nothing
//! else. A move arrives in the tree's current coordinates and is resolved here
//! by [`pdfcer_gui_base::layerorder::engine_move`]; a move to where the entry
//! already is raises nothing, and one into itself is refused before the
//! engine is asked. Every receipt says where the entry now is, read back from
//! the document after the edit.

use pdfcer_core::edit::{EditError, EditSession, LayerOrderOutcome};
use pdfcer_gui_base::layeraction::OrderAction;
use pdfcer_gui_base::layerorder::{self as order, Kind, Move, Resolved};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::panels::layeredit::{self as t, LayerRefusal};

/// Whether a move is worth raising: `false` after tracing a no-op, or after
/// recording the refusal for a move into itself or to a vanished entry.
pub(super) fn worth_moving(doc: &OpenDoc, m: &Move) -> bool {
    let tree = current_tree(&doc.session);
    let resolved = order::engine_move(&tree, m);
    let refusal = match resolved {
        Resolved::Engine { .. } => return true,
        Resolved::Unchanged => None,
        Resolved::IntoItself => Some(LayerRefusal::IntoItself),
        Resolved::NotFound => Some(LayerRefusal::OrderPathNotFound),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "layer-order op=move from={} changed=false reason={resolved:?}",
            order::dotted(&m.from)
        )
    });
    if let Some(why) = refusal {
        decline::record_layer(why);
    }
    false
}

/// Run one arrangement act and word its receipt.
pub(super) fn run(session: &mut EditSession, op: OrderAction) -> Result<Vec<String>, EditError> {
    let (word, from, out, mut notes) = match op {
        OrderAction::AddFolder {
            parent,
            index,
            label,
        } => {
            let out = session.add_layer_folder(&parent, index, &label)?;
            let line = t::folder_added(&label, &place(session, &out.path));
            ("add-folder", parent, out, vec![line])
        }
        OrderAction::RenameFolder { at, label } => {
            let was = entry_name(session, &at);
            let out = session.rename_layer_folder(&at, &label)?;
            let line = t::folder_renamed(&was, &label);
            ("rename-folder", at, out, vec![line])
        }
        OrderAction::DeleteFolder { at } => {
            let tree = current_tree(session);
            let held = order::node_at(&tree, &at).map_or(0, |n| n.children.len());
            let was = entry_name(session, &at);
            let out = session.delete_layer_folder(&at)?;
            (
                "remove-folder",
                at,
                out,
                vec![t::folder_removed(&was, held)],
            )
        }
        OrderAction::Move(m) => {
            let Resolved::Engine { parent, index } = order::engine_move(&current_tree(session), &m)
            else {
                return Err(EditError::LayerOrderPathNotFound { path: m.from });
            };
            let name = entry_name(session, &m.from);
            let out = session.move_layer_node(&m.from, &parent, index)?;
            let line = t::moved(&name, &place(session, &out.path));
            ("move", m.from, out, vec![line])
        }
    };
    trace(word, &from, &out);
    if out.follows_layer {
        notes.push(t::folder_follows_layer().to_owned());
    }
    Ok(notes)
}

fn trace(word: &str, from: &[usize], out: &LayerOrderOutcome) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "layer-order op={word} from={} path={} changed={} follows_layer={}",
            order::dotted(from),
            order::dotted(&out.path),
            out.changed,
            out.follows_layer
        )
    });
}

fn current_tree(session: &EditSession) -> Vec<order::Node> {
    order::tree(&pdfcer_core::layers::read_layers(&session.view()).order)
}

/// What the entry at `path` is called: a folder's label, a layer's name.
fn entry_name(session: &EditSession, path: &[usize]) -> String {
    let read = pdfcer_core::layers::read_layers(&session.view());
    let tree = order::tree(&read.order);
    name_of(&read, order::node_at(&tree, path).map(|n| &n.kind))
}

fn name_of(read: &pdfcer_core::layers::Layers, kind: Option<&Kind>) -> String {
    match kind {
        Some(Kind::Folder(label)) => label.clone(),
        Some(Kind::Layer(id)) => read
            .layers
            .iter()
            .find(|l| l.id == *id)
            .map(|l| l.name.clone())
            .unwrap_or_default(),
        Some(Kind::Grouping) | None => String::new(),
    }
}

/// Where the entry at `path` sits, as the receipt says it.
fn place(session: &EditSession, path: &[usize]) -> String {
    let Some((&last, parent)) = path.split_last() else {
        return t::place_top(1);
    };
    let position = last + 1;
    if parent.is_empty() {
        return t::place_top(position);
    }
    let read = pdfcer_core::layers::read_layers(&session.view());
    let tree = order::tree(&read.order);
    match order::node_at(&tree, parent).map(|n| &n.kind) {
        Some(Kind::Grouping) | None => t::place_in_group(position),
        kind => t::place_in(&name_of(&read, kind), position),
    }
}
