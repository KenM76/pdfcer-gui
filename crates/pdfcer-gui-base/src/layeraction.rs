//! # `layeraction` — the verbs that author a layer rather than show or hide it
//!
//! Carried by `Action::Layer` from the Layers panel to
//! `pdfcer_gui::app::actions::layers`, which calls the matching
//! `EditSession` verb through the edit funnel. Each is one undo entry.
//! [`OrderAction`] arranges the panel's list (`/D /Order`) and nothing else.

use pdfcer_core::edit::{HiddenLayerPolicy, LayerContentPolicy, LayerEdit};
use pdfcer_core::object::ObjId;

use crate::layerorder::Move;

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
    /// `EditSession::merge_layers`: what `merged` draws is drawn on `target`,
    /// and `merged` leaves the list.
    Merge {
        /// The layer that stays.
        target: ObjId,
        /// The layers folded into it.
        merged: Vec<ObjId>,
    },
    /// `EditSession::flatten_layers`: the document ends with no layers.
    Flatten {
        /// What becomes of layers hidden when the document opens.
        hidden: HiddenLayerPolicy,
    },
    /// A folder or a move in the panel's list.
    Order(OrderAction),
    /// Put the selection on `layer`, or on no layer (`None`):
    /// `EditSession::set_objects_layer` for page content,
    /// `EditSession::set_annotation_layer` for an annotation or a form
    /// field's widget. The operand is read from the selection when applied.
    Assign {
        /// The group's object id, or `None` for no layer.
        layer: Option<ObjId>,
    },
}

/// One arrangement of the panel's list; paths are `layerorder` paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderAction {
    /// `EditSession::add_layer_folder`: an empty folder before the child at
    /// `index` of `parent`.
    AddFolder {
        /// The folder's parent.
        parent: Vec<usize>,
        /// Its slot among the parent's children.
        index: usize,
        /// Its label; never empty.
        label: String,
    },
    /// `EditSession::rename_layer_folder`.
    RenameFolder {
        /// The folder.
        at: Vec<usize>,
        /// The new label.
        label: String,
    },
    /// `EditSession::delete_layer_folder`: what it held takes its place.
    DeleteFolder {
        /// The folder.
        at: Vec<usize>,
    },
    /// `EditSession::move_layer_node`, given in the tree as it is now and
    /// resolved by `layerorder::engine_move`.
    Move(Move),
}

impl LayerAction {
    /// A short, stable name for the diagnostic trace.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Add { .. } => "layer-add",
            Self::Edit { .. } => "layer-edit",
            Self::Delete { .. } => "layer-delete",
            Self::Merge { .. } => "layer-merge",
            Self::Flatten { .. } => "layer-flatten",
            Self::Order(OrderAction::AddFolder { .. }) => "layer-folder-add",
            Self::Order(OrderAction::RenameFolder { .. }) => "layer-folder-rename",
            Self::Order(OrderAction::DeleteFolder { .. }) => "layer-folder-remove",
            Self::Order(OrderAction::Move(_)) => "layer-move",
            Self::Assign { .. } => "layer-assign",
        }
    }
}
