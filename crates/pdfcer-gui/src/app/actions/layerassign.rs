//! # `app::actions::layerassign` — `LayerAction::Assign`: put the selection on a layer
//!
//! Contract: the operand is read from the selection when the act is applied
//! ([`operand`]), and a selection that cannot be moved is refused in words
//! before the engine is called. Page content goes through
//! `EditSession::set_objects_layer` (paint-order indices on one page); an
//! annotation, or a selected form field's widget, through
//! `EditSession::set_annotation_layer`. Each is one undo entry, and a move
//! that changes nothing records none.

use pdfcer_core::edit::{EditError, EditSession};
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::VectorEditError;

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::canvas::selection::SelectionState;
use crate::text::panels::layerassign as t;
use crate::text::panels::layeredit::LayerRefusal;

/// What a move to a layer acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    /// Whole page objects, by paint-order index, on one page.
    Objects {
        /// The page.
        page: usize,
        /// Paint-order indices, ascending.
        indices: Vec<usize>,
    },
    /// One annotation or form-field widget.
    Annotation {
        /// The page it is on.
        page: usize,
        /// Its object id.
        id: ObjId,
    },
}

/// The selection as a move operand, or the refusal that names why it is not
/// one.
///
/// # Errors
///
/// `AssignNothing`, `AssignParts` (a leaf is selected: a part shares its
/// object's layer) or `AssignSeveralPages`.
pub fn operand(doc: &OpenDoc) -> Result<Operand, LayerRefusal> {
    operand_of(doc, &doc.selection)
}

/// [`operand`] for a selection held apart from the document, as the canvas
/// menu holds it on the right-click frame.
///
/// # Errors
///
/// As [`operand`].
pub fn operand_of(doc: &OpenDoc, selection: &SelectionState) -> Result<Operand, LayerRefusal> {
    if let Some(selected) = &doc.selected_field {
        return widget_id(doc, &selected.field, selected.widget)
            .map(|id| Operand::Annotation {
                page: selected.page,
                id,
            })
            .ok_or(LayerRefusal::AssignNothing);
    }
    if let Some(annot) = selection.annot() {
        return Ok(Operand::Annotation {
            page: annot.target.page,
            id: annot.target.id,
        });
    }
    let entries = selection.entries();
    let Some(first) = entries.first() else {
        return Err(LayerRefusal::AssignNothing);
    };
    let page = first.page;
    if entries.iter().any(|e| e.page != page) {
        return Err(LayerRefusal::AssignSeveralPages);
    }
    if !selection.leaf_indices_on(page).is_empty() {
        return Err(LayerRefusal::AssignParts);
    }
    let indices = selection.object_indices_on(page);
    if indices.is_empty() {
        return Err(LayerRefusal::AssignNothing);
    }
    Ok(Operand::Objects { page, indices })
}

/// Whether Move to layer is offered: the document has a layer and
/// `selection` is a movable operand.
#[must_use]
pub fn offered(doc: &OpenDoc, selection: &SelectionState) -> bool {
    operand_of(doc, selection).is_ok() && has_layers(doc)
}

/// Whether the document has at least one layer.
#[must_use]
pub fn has_layers(doc: &OpenDoc) -> bool {
    !pdfcer_core::layers::read_layers(&doc.session.view())
        .layers
        .is_empty()
}

/// The object id of widget `widget` of the field named `field`.
fn widget_id(doc: &OpenDoc, field: &str, widget: usize) -> Option<ObjId> {
    let form = pdfcer_core::forms::parse_acroform(&doc.session.view())?;
    let field = form
        .fields
        .iter()
        .find(|f| f.fully_qualified_name == field)?;
    field.widgets.get(widget).map(|w| w.id)
}

/// Move the selection onto `layer`, or off every layer.
pub(super) fn apply(doc: &mut OpenDoc, layer: Option<ObjId>) {
    let target = match operand(doc) {
        Ok(target) => target,
        Err(why) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("layer-assign-refused reason={why:?}")
            });
            decline::record_layer(why);
            return;
        }
    };
    let (page, count) = match &target {
        Operand::Objects { page, indices } => (*page, indices.len()),
        Operand::Annotation { page, .. } => (*page, 1),
    };
    let label = pdfcer_gui_base::layeraction::LayerAction::Assign { layer }.label();
    let edit = |session: &mut EditSession| run(session, &target, layer).inspect_err(word_refusal);
    match target {
        Operand::Objects { .. } => {
            super::funnel::vector_edit_on_page(doc, label, page, count, edit)
        }
        Operand::Annotation { .. } => super::apply::vector_edit(doc, label, page, count, edit),
    }
}

fn run(
    session: &mut EditSession,
    target: &Operand,
    layer: Option<ObjId>,
) -> Result<Vec<String>, EditError> {
    let name = layer.map(|l| super::layers::name_of(session, l));
    match target {
        Operand::Objects { page, indices } => {
            // Annotated so an engine rename fails the build, not the trace.
            let out: pdfcer_core::edit::ObjectsLayerChange =
                session.set_objects_layer(*page, indices, layer)?;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "layer-assigned kind=objects page={page} moved={} unchanged={} binding={} bound={} layer={}",
                    out.moved,
                    out.unchanged,
                    out.property_name.as_deref().unwrap_or("none"),
                    out.binding_added,
                    id_text(layer)
                )
            });
            let mut lines = vec![t::objects_moved(out.moved, out.unchanged, name.as_deref())];
            lines.extend(out.disclosures);
            Ok(lines)
        }
        Operand::Annotation { id, .. } => {
            // Annotated so an engine rename fails the build, not the trace.
            let out: pdfcer_core::edit::AnnotationLayerChange =
                session.set_annotation_layer(*id, layer)?;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "layer-assigned kind=annotation id={}_{} subtype={} changed={} popup={} layer={}",
                    id.num,
                    id.generation,
                    out.subtype,
                    out.changed,
                    out.popup_written,
                    id_text(out.after)
                )
            });
            Ok(vec![t::annotation_moved(
                out.changed,
                name.as_deref(),
                out.changed && out.before.is_some_and(|b| !is_layer(session, b)),
            )])
        }
    }
}

/// Whether `id` is a registered layer; an `/OC` naming anything else is a
/// membership rule.
fn is_layer(session: &EditSession, id: ObjId) -> bool {
    pdfcer_core::layers::read_layers(&session.view())
        .layers
        .iter()
        .any(|l| l.id == id)
}

/// A layer id for the trace: `num_gen`, or `none`.
fn id_text(layer: Option<ObjId>) -> String {
    layer.map_or_else(
        || "none".to_owned(),
        |l| format!("{}_{}", l.num, l.generation),
    )
}

/// Word the refusals the operator can act on; the rest keep the funnel's
/// generic sentence and the engine's words on the trace.
fn word_refusal(error: &EditError) {
    let why = match error {
        EditError::LayerNotFound { .. } => LayerRefusal::NotFound,
        EditError::LayerContentNotRewritable { .. } => LayerRefusal::AssignOwnRule,
        EditError::AnnotationLocked { .. } => LayerRefusal::AssignLocked,
        EditError::VectorEdit(VectorEditError::LayerSectionHoldsTaggedContent { .. }) => {
            LayerRefusal::AssignTagged
        }
        EditError::VectorEdit(
            VectorEditError::LayerSectionCrossesNesting { .. }
            | VectorEditError::LayerSpanUnbalanced { .. }
            | VectorEditError::OverlappingObjectSpans { .. },
        ) => LayerRefusal::AssignTangled,
        _ => return,
    };
    decline::record_layer(why);
}
