//! # `app::layers` — **which optional-content groups are hidden, and whose
//! answer that is**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/layers.md`.

use std::collections::BTreeSet;

use pdfcer_core::object::ObjId;
use pdfcer_render::LayerVisibility;

use super::state::OpenDoc;

impl OpenDoc {
    /// The complete set of optional-content groups currently hidden.
    #[must_use]
    pub fn hidden_layers(&self) -> BTreeSet<ObjId> {
        self.layers.hidden.clone().unwrap_or_else(|| {
            pdfcer_core::annot::optional_content_default_off(&self.session.view())
        })
    }

    /// Replace the operator's optional-content override with `hidden`.
    pub fn set_hidden_layers(&mut self, hidden: BTreeSet<ObjId>) {
        self.layers.hidden = Some(hidden);
        self.layers.generation = self.layers.generation.wrapping_add(1);
    }

    /// Show or hide one optional-content group.
    pub fn set_layer_visible(&mut self, group: ObjId, visible: bool) {
        let mut hidden = self.hidden_layers();
        if visible {
            hidden.remove(&group);
        } else {
            hidden.insert(group);
        }
        self.set_hidden_layers(hidden);
    }

    /// Drop the operator's override and go back to obeying the document.
    pub fn reset_layers(&mut self) {
        self.layers.hidden = None;
        self.layers.generation = self.layers.generation.wrapping_add(1);
    }

    /// The override to hand a render, or `None` to obey the document.
    pub(crate) fn layer_visibility(&self) -> Option<LayerVisibility> {
        self.layers
            .hidden
            .as_ref()
            .map(|hidden| LayerVisibility::hiding(hidden.iter().copied()))
    }
}
