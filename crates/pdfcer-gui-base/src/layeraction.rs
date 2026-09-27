//! # `layeraction` — the verbs that author a layer rather than show or hide it
//!
//! Carried by `Action::Layer` from the Layers panel to
//! `pdfcer_gui::app::actions::layers`, which calls the matching
//! `EditSession` verb through the edit funnel. Each is one undo entry.

use pdfcer_core::edit::{LayerContentPolicy, LayerEdit};
use pdfcer_core::object::ObjId;

/// One authoring act on the document's optional-content groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerAction {
    /// `EditSession::add_layer`: a new, empty layer, last in the panel.
    Add {
        /// The new layer's `/Name`; never empty.
        name: String,
    },
    /// `EditSession::set_layer_properties`: only the fields `edit` sets change.
    Edit {
        /// The group's object id, as `Layers::layers` reports it.
        layer: ObjId,
        /// The fields to change.
        edit: LayerEdit,
    },
    /// `EditSession::delete_layer`, keeping or removing what the layer draws.
    Delete {
        /// The group's object id.
        layer: ObjId,
        /// What becomes of the layer's drawing.
        policy: LayerContentPolicy,
    },
}

impl LayerAction {
    /// A short, stable name for the diagnostic trace.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Add { .. } => "layer-add",
            Self::Edit { .. } => "layer-edit",
            Self::Delete { .. } => "layer-delete",
        }
    }
}
