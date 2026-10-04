//! # `app::actions::layers` — `Action::Layer`: create, change, delete, merge and flatten layers, and arrange the list
//!
//! Contract: each [`LayerAction`] is one `EditSession` verb through the edit
//! funnel, so it is one undo entry and its sentence reaches the status bar.
//! A verb that reports `changed: false` wrote nothing and is still `Ok`; the
//! refusals the operator can act on are worded by
//! [`crate::text::panels::layeredit::LayerRefusal`].

use pdfcer_core::edit::{
    EditError, EditSession, HiddenLayerPolicy, LayerContentPolicy, LayerEdit, LayerEditOutcome,
    LayerFlattenOutcome, LayerMergeOutcome,
};
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::layeraction::{LayerAction, OrderAction};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::panels::layeredit::{self as t, LayerRefusal};

/// Apply one layer act.
pub(super) fn apply(doc: &mut OpenDoc, action: LayerAction) {
    if let LayerAction::Order(OrderAction::Move(m)) = &action
        && !super::layerorder::worth_moving(doc, m)
    {
        return;
    }
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
        LayerAction::Merge { target, merged } => {
            let names: Vec<String> = merged.iter().map(|&l| name_of(session, l)).collect();
            let into = name_of(session, target);
            let out: LayerMergeOutcome = session.merge_layers(target, &merged)?;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "layer-merged into={}_{} changed={} layers={} bindings={} annotations={} xobjects={} memberships={}",
                    target.num,
                    target.generation,
                    out.changed,
                    out.layers,
                    out.bindings,
                    out.annotations,
                    out.xobjects,
                    out.memberships
                )
            });
            Ok(vec![t::merged(&names, &into, &out)])
        }
        LayerAction::Flatten { hidden } => {
            let out: LayerFlattenOutcome = session.flatten_layers(hidden)?;
            let removed = hidden == HiddenLayerPolicy::Remove;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "layer-flattened changed={} removed={removed} layers={} hidden={} sections={} paints={}",
                    out.changed, out.layers, out.hidden_layers, out.sections, out.paints
                )
            });
            Ok(vec![t::flattened(&out)])
        }
        LayerAction::Order(op) => super::layerorder::run(session, op),
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
        EditError::HiddenLayersNeedPolicy { .. } => LayerRefusal::HiddenNeedChoice,
        EditError::NotALayerFolder { .. } => LayerRefusal::NotAFolder,
        EditError::LayerOrderPathNotFound { .. } => LayerRefusal::OrderPathNotFound,
        EditError::LayerOrderInexpressible => LayerRefusal::OrderInexpressible,
        EditError::LayerOrderNotEditable { .. } => LayerRefusal::OrderNotEditable,
        _ => return,
    };
    decline::record_layer(why);
}
