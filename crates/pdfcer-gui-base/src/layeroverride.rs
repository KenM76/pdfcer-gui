//! # `layeroverride` — which optional-content groups the operator has hidden
//!
//! `pdfcer_render::LayerVisibility` *replaces* the document's own `/D`
//! configuration rather than merging with it, so the three-state shape below
//! is load-bearing: collapsing two of its states is a disclosure defect.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/layeroverride.md`.

use std::collections::BTreeSet;

use pdfcer_core::object::ObjId;

/// Which optional-content groups the operator has hidden, if any.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LayerOverride {
    /// The complete hidden set, or `None` to obey the document.
    pub hidden: Option<BTreeSet<ObjId>>,
    /// How many times the above has changed.
    ///
    /// The render staleness key — see
    /// [`crate::renderworker::RenderKey`], whose own docs explain why a
    /// counter beats comparing the set on every frame. `0` is the
    /// never-touched state, which is exactly `hidden: None`.
    pub generation: u64,
}
