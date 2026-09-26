//! # `app::state::layers` — **which optional-content groups the operator has
//! hidden**
//!
//! The subject is one question: **what does the renderer have to be told about
//! optional content?** Its whole weight is the core API trap that
//! `pdfcer_render::LayerVisibility` *replaces* the document's own `/D`
//! configuration rather than merging with it, which makes the three-state shape
//! below load-bearing — collapsing two of its states is a disclosure defect.
//!
//! [`LayerOverride`] is `pub(in crate::app)` rather than `pub(super)`
//! deliberately: `pub(super)` here would mean "visible in `crate::app::state`",
//! narrower than the `OpenDoc::layers` field that holds it, which is a
//! private-interface error. The reach is spelled absolutely so moving the type
//! cannot change what can see it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/state/layers.md`.

use std::collections::BTreeSet;

use pdfcer_core::object::ObjId;

/// Which optional-content groups the operator has hidden, if any.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::app) struct LayerOverride {
    /// The complete hidden set, or `None` to obey the document.
    pub(in crate::app) hidden: Option<BTreeSet<ObjId>>,
    /// How many times the above has changed.
    ///
    /// The render staleness key — see
    /// [`crate::render::worker::RenderKey`], whose own docs explain why a
    /// counter beats comparing the set on every frame. `0` is the
    /// never-touched state, which is exactly `hidden: None`.
    pub(in crate::app) generation: u64,
}
