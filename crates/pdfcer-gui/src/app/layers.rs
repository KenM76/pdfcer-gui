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
    ///
    /// The operator's override if there is one, and otherwise the
    /// **document's own** answer from
    /// `pdfcer_core::annot::optional_content_default_off` — which is the
    /// print/export-correct `/D`-initial OFF set (§8.11.4.3), and the same
    /// resolution `pdfcer_core::layers::read_layers` reports per layer as
    /// `visible_by_default`.
    ///
    /// This is what a visibility control reads to compute the *next* set:
    /// the override replaces the document's configuration rather than merging
    /// with it (core API trap T-12.9), so a caller starts from the complete
    /// current answer
    /// and hands back a complete new one. Handing in only the groups the
    /// operator touched would show every layer the document had turned off.
    ///
    /// Computed rather than cached: it is read when a control is clicked, not
    /// per frame, and a cached copy would be one more thing to invalidate on
    /// an edit that added a layer.
    #[must_use]
    pub fn hidden_layers(&self) -> BTreeSet<ObjId> {
        self.layers.hidden.clone().unwrap_or_else(|| {
            pdfcer_core::annot::optional_content_default_off(&self.session.view())
        })
    }

    /// Replace the operator's optional-content override with `hidden`.
    ///
    /// The **complete** hidden set, for the reason above. Bumps the
    /// generation, which is what makes the cached page texture stale.
    ///
    /// Bumps it even when the set is unchanged, deliberately: comparing two
    /// `BTreeSet<ObjId>`s to save a re-render costs more than the re-render
    /// is likely to, and a control that calls this has by definition just
    /// been clicked. A spurious re-render is a wasted rasterization; a missed
    /// one is a control that appears inert, and those are not equally bad.
    pub fn set_hidden_layers(&mut self, hidden: BTreeSet<ObjId>) {
        self.layers.hidden = Some(hidden);
        self.layers.generation = self.layers.generation.wrapping_add(1);
    }

    /// Show or hide one optional-content group.
    ///
    /// The single-checkbox convenience over [`Self::hidden_layers`] and
    /// [`Self::set_hidden_layers`], seeding from the document's own defaults
    /// on the first toggle so the override starts out agreeing with what the
    /// operator is looking at.
    ///
    /// **It does not apply `/RBGroups` radio semantics.** A group in a radio
    /// group may have at most one member visible at a time (Table 101), so
    /// turning one on has to turn its siblings off — and the sibling list
    /// comes from `pdfcer_core::layers::read_layers`, which is the *control's*
    /// reading, not this type's. A control that needs it composes the whole
    /// set and calls [`Self::set_hidden_layers`]; a half-implementation here
    /// would be a second visibility algebra beside the engine's, which is
    /// what the replace-not-merge contract exists to prevent.
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
    ///
    /// Distinct from hiding nothing, and the distinction is the whole of core
    /// API trap T-12.9: this restores the document's own `/D` configuration,
    /// whereas
    /// `set_hidden_layers(BTreeSet::new())` reveals every layer the document
    /// turns off.
    pub fn reset_layers(&mut self) {
        self.layers.hidden = None;
        self.layers.generation = self.layers.generation.wrapping_add(1);
    }

    /// The override to hand a render, or `None` to obey the document.
    ///
    /// `pub(crate)` rather than `pub(super)` for one caller outside `app`:
    /// `crate::clipboard::place`, which produces the vector copy-out and is a
    /// *format* concern rather than an application-state one. Widening costs
    /// nothing — this is a read-only accessor whose whole job is to be handed
    /// to a render — and **every** render this shell performs must pass it, or
    /// the output shows a layer the operator hid.
    pub(crate) fn layer_visibility(&self) -> Option<LayerVisibility> {
        self.layers
            .hidden
            .as_ref()
            .map(|hidden| LayerVisibility::hiding(hidden.iter().copied()))
    }
}
