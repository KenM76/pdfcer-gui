//! # `app::layers` — **which optional-content groups are hidden, and whose
//! answer that is**
//!
//! Four methods and one private helper, split out of [`crate::app::state`] on
//! 2026-09-01 under R2. The seam is a real subject rather than a line count:
//! everything here is about the **override relationship** between the operator
//! and the document's own `/D` optional-content configuration (ISO 32000-1
//! §8.11.4.3), and nothing else in `OpenDoc` participates in it.
//!
//! ## ★★ The rule the whole file exists to hold
//!
//! `OpenDoc::layers.hidden` is `Option<BTreeSet<ObjId>>`, and the `Option` is
//! **three states, not two**:
//!
//! | value | meaning |
//! |---|---|
//! | `None` | **obey the document.** Whatever `/D` says is off, is off |
//! | `Some(set)` | the operator's **complete** answer, replacing `/D` entirely |
//! | `Some(∅)` | the operator has explicitly revealed **everything**, including layers the document turns off |
//!
//! The last two are the pair that gets collapsed. `reset_layers` restores the
//! document's configuration; `set_hidden_layers(BTreeSet::new())` shows every
//! layer the producer had deliberately hidden — a watermark, a plot-only
//! border, a set of construction lines. They are different acts with different
//! results and T-12.9 is the record of the argument.
//!
//! ## Why the override REPLACES rather than merges
//!
//! Because a merge has no expressible way to say *"show a layer the document
//! turns off"*. A caller therefore starts from [`OpenDoc::hidden_layers`],
//! which is the **complete current answer** — the override if there is one,
//! and the document's own resolution otherwise — and hands back a complete new
//! one. Handing in only the groups the operator touched would reveal every
//! layer the document had turned off, on the first click of any layer control.
//!
//! ## Why `hidden_layers` is computed and not cached
//!
//! It is read when a control is clicked, not per frame, and a cached copy
//! would be one more thing to invalidate on an edit that adds a layer. The
//! generation counter beside it is the cheap thing that *is* kept, and it is
//! what makes a page texture stale.

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
    /// with it (T-12.9), so a caller starts from the complete current answer
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
    /// Distinct from hiding nothing, and the distinction is the whole of
    /// T-12.9: this restores the document's own `/D` configuration, whereas
    /// `set_hidden_layers(BTreeSet::new())` reveals every layer the document
    /// turns off.
    pub fn reset_layers(&mut self) {
        self.layers.hidden = None;
        self.layers.generation = self.layers.generation.wrapping_add(1);
    }

    /// The override to hand a render, or `None` to obey the document.
    pub(super) fn layer_visibility(&self) -> Option<LayerVisibility> {
        self.layers
            .hidden
            .as_ref()
            .map(|hidden| LayerVisibility::hiding(hidden.iter().copied()))
    }
}
