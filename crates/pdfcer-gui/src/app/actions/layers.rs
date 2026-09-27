//! # `app::actions::layers` — `Action::Layer`: create, change and delete a layer
//!
//! Contract: each [`LayerAction`] is one `EditSession` verb through the edit
//! funnel, so it is one undo entry and its sentence reaches the status bar.
//! A verb that reports `changed: false` wrote nothing and is still `Ok`; the
//! refusals the operator can act on are worded by
//! [`crate::text::panels::layeredit::LayerRefusal`].

use pdfcer_core::edit::{EditError, EditSession, LayerContentPolicy, LayerEdit, LayerEditOutcome};
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::layeraction::LayerAction;

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::panels::layeredit::{self as t, LayerRefusal};

/// Apply one layer act.
pub(super) fn apply(doc: &mut OpenDoc, action: LayerAction) {
    let label = action.label();
    super::apply::vector_edit(doc, label, 0, 1, |session| {
        run(session, action).inspect_err(word_refusal)
    });
}

fn run(session: &mut EditSession, action: LayerAction) -> Result<Vec<String>, EditError> {
    match action {
        LayerAction::Add { name } => {
            let id = session.add_layer(&name, &LayerEdit::new())?;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("layer-added id={}_{} name={name:?}", id.num, id.generation)
            });
            Ok(vec![t::added(&name)])
        }
        LayerAction::Edit { layer, edit } => {
            let out: LayerEditOutcome = session.set_layer_properties(layer, &edit)?;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "layer-edited id={}_{} changed={}",
                    layer.num, layer.generation, out.changed
                )
            });
            Ok(Vec::new())
        }
        LayerAction::Delete { layer, policy } => {
            let name = name_of(session, layer);
            let out = session.delete_layer(layer, policy)?;
            let removed = policy == LayerContentPolicy::RemoveContent;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "layer-deleted id={}_{} changed={} removed={removed} sections={} annotations={}",
                    layer.num, layer.generation, out.changed, out.sections, out.annotations
                )
            });
            Ok(vec![t::deleted(&name, removed, &out)])
        }
    }
}

/// The layer's display name, read before the delete takes it away.
fn name_of(session: &EditSession, layer: ObjId) -> String {
    pdfcer_core::layers::read_layers(&session.view())
        .layers
        .into_iter()
        .find(|l| l.id == layer)
        .map(|l| l.name)
        .unwrap_or_default()
}

/// Word the refusals the operator can act on; the rest keep the funnel's
/// generic sentence and the engine's words on the trace.
fn word_refusal(error: &EditError) {
    let why = match error {
        EditError::EmptyLayerName => LayerRefusal::EmptyName,
        EditError::LayerNotFound { .. } => LayerRefusal::NotFound,
        EditError::LayerInMembership { .. } => LayerRefusal::InMembership,
        EditError::LayerHasWidget { .. } => LayerRefusal::HasWidget,
        EditError::LayerContentNotRewritable { .. } => LayerRefusal::ContentNotRewritable,
        _ => return,
    };
    decline::record_layer(why);
}
