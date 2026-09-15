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

use std::collections::BTreeSet;

use pdfcer_core::object::ObjId;

/// Which optional-content groups the operator has hidden, if any.
///
/// # ★ `None` is not "hide nothing"
///
/// `pdfcer_render::LayerVisibility` **replaces** the document's own default
/// configuration rather than merging with it (core API trap T-12.9). So:
///
/// | state | meaning |
/// |---|---|
/// | `hidden: None` | obey the document's `/D` configuration (§8.11.4.3) |
/// | `hidden: Some({})` | show **every** layer, including ones the document turns off |
/// | `hidden: Some({…})` | exactly these are hidden |
///
/// Collapsing the first two would silently reveal every layer a document had
/// turned off, which on a drawing with a "Confidential" watermark layer is a
/// disclosure defect rather than a cosmetic one.
///
/// That is also why a set is stored rather than operator *deltas*: the
/// renderer wants the complete answer, so the complete answer is what is
/// held. A delta would have to be resolved against the document's defaults at
/// render time, in a second place, with the merge rules the engine
/// deliberately refused to define.
///
/// # ★ The operator's toggle is session-only, and nothing here can save it
///
/// §8.11.2.1 puts the live state outside the document entirely: the toggle is
/// *"session-only state, held nowhere the save path can see it"*, lost on
/// reopen. That is a property of the format rather than a gap in this build,
/// it is what `crate::text::panels::layers_session_only_note` discloses, and
/// it is why changing it must not bump [`OpenDoc::edit_epoch`].
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
