//! # `app::state::layers` — **which optional-content groups the operator has
//! hidden**
//!
//! Split out of [`super`] on 2026-09-12 under **R2**, when O177's one-shot
//! pushed `app/state.rs` past the 1,500-line ceiling.
//!
//! ## Why this is a seam rather than an arbitrary cut
//!
//! Everything else in `app::state` answers *"what is open, and what is the
//! operator looking at?"*. This answers a narrower and quite different
//! question: **what does the renderer have to be told about optional
//! content**, and it carries a whole argument of its own about a core API trap
//! — that `pdfcer_render::LayerVisibility` *replaces* the document's own
//! `/D` configuration rather than merging with it, so the three-state shape
//! below is load-bearing and collapsing two of its states is a disclosure
//! defect. That argument has nothing to say about page rasters, selection,
//! undo depth or window identity, which is the test for a seam: a reader here
//! for layers needs none of the rest, and a reader there needs none of this.
//!
//! ## Visibility
//!
//! [`LayerOverride`] is `pub(in crate::app)` rather than `pub(super)`, which
//! reproduces **exactly** the reach it had while it was declared one level up:
//! `pub(super)` inside `app/state.rs` meant "visible in `crate::app`", and
//! `pub(super)` written here would mean "visible in `crate::app::state`" —
//! narrower than the `OpenDoc::layers` field that holds it, which is a
//! private-interface error rather than a tightening. Spelled absolutely so the
//! move cannot change what can see it.

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
