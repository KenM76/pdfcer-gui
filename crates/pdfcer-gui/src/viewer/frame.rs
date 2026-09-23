//! # `viewer::frame` — what one canvas remembers between frames
//!
//! [`ViewFrame`] is the per-**view** bookkeeping a canvas writes at the end of
//! a frame and reads at the start of the next: where the scroll area settled,
//! what the zoom was, where the zoom is anchored, and the deep tier's position
//! when the scroll offset can no longer carry it.
//!
//! ## Why it is not part of [`ViewState`]
//!
//! [`ViewState`] is a record of **choices** — a zoom that was *set*. Its
//! header turns that into a licence to derive `PartialEq` over an `f32`, on
//! the ground that two states which arrived at 1.0 by different routes are
//! genuinely the same state. Nothing here is a choice. `observed_zoom` is a
//! measurement, `zoom_commit_at` is a clock reading, and two views showing the
//! identical thing will hold different values for both. Folding these in would
//! make that equality quietly mean "and was arrived at during the same
//! millisecond", which is not what any caller of it wants.
//!
//! So a view is a *pair*: the stance it was put into, and what its canvas
//! observed while presenting that stance.
//!
//! ## Why it is per view and not per document
//!
//! Every field here is frame bookkeeping about **a** canvas. With one canvas
//! on screen, storing it on the document is indistinguishable from storing it
//! on the view. With two canvases showing
//! one document — `OPERATOR_REQUESTS.md` O226 — a single copy is not a
//! limitation but a defect: the second pane's scroll overwrites the first
//! pane's settled offset, and the first pane then pans from a position it was
//! never at. The same goes for the zoom anchor, which would put pane A's
//! zoom-to-cursor over pane B's cursor.

use std::time::Instant;

use super::ZoomAnchor;

/// **What one canvas observed about itself on the previous frame.**
///
/// Written by [`crate::canvas`] at the end of a frame, read by it at the
/// start of the next. Nothing outside the canvas and the render settle logic
/// has a reason to write any of it.
#[derive(Debug, Clone, Copy)]
pub struct ViewFrame {
    /// The zoom seen at the end of the previous frame, used to detect that
    /// the zoom changed at all.
    pub observed_zoom: f32,
    /// The earliest instant at which the current zoom may be committed to a
    /// real rasterization — the [`crate::render::settle::ZOOM_SETTLE`]
    /// debounce deadline.
    pub zoom_commit_at: Instant,
    /// Set by any *discrete* zoom command during this frame's action
    /// dispatch, and consumed at the end of the frame. It is what
    /// distinguishes "the operator pressed Ctrl+0" (commit at once) from
    /// "the operator is mid-wheel-gesture" (wait for the gesture to settle).
    pub zoom_commanded: bool,
    /// See [`ZoomAnchor`]. Written by the canvas, consumed by the canvas on
    /// the following frame.
    pub zoom_anchor: Option<ZoomAnchor>,
    /// The scroll offset the canvas settled on at the end of the last frame.
    ///
    /// Kept because middle-drag panning has to compute "where the view
    /// should be now" BEFORE the scroll area is built, and the area's own
    /// state is only readable after. Storing last frame's settled value
    /// lets the pan be applied in the same frame as the movement rather
    /// than a frame late — which is the difference between panning that
    /// tracks the hand and panning that lags it.
    pub last_scroll_offset: egui::Vec2,
    /// ★★ **Where the view is, once the scroll offset can no longer say** —
    /// O24 tier 3.
    ///
    /// `None` below the sub-pixel content extent, where `egui::ScrollArea`'s
    /// own `f32` offset is authoritative and nothing about the canvas differs
    /// from before this feature. `Some` above it, where the position is a page
    /// point in `f64` and the screen pixel it sits under.
    ///
    /// ★ Seeded on the way in from wherever the scroll area had settled, and
    /// cleared on the way out — so crossing the threshold in either direction
    /// does not move the page under the operator, and re-entering starts from
    /// the truth rather than from a stale anchor.
    pub deep_anchor: Option<super::deep::DeepAnchor>,
    /// The zoom [`Self::deep_anchor`] was last valid at, or `None` outside the
    /// deep tier.
    ///
    /// ★★ **What makes zoom-to-cursor possible above the threshold.**
    /// `DeepAnchor::zoomed_about` needs the zoom the anchor was written at, so
    /// it can read which page point sits under the cursor *before* re-stating
    /// the anchor at the new scale. The anchor itself deliberately does not
    /// carry a zoom — it is a statement about page space and screen space, and
    /// baking a scale into it would make it stale rather than merely
    /// unfashionable. So the canvas remembers the scale beside it.
    ///
    /// `OPERATOR_REQUESTS.md` O24f: without this the anchor never moved on a
    /// zoom, the anchored page point stayed nailed to the viewport's top-left,
    /// and everything the operator was looking at expanded off the screen.
    /// Cleared on leaving the tier so the first frame back inside seeds from
    /// the scroll area rather than re-anchoring against a scale from minutes
    /// ago.
    pub deep_zoom: Option<f64>,
}

impl ViewFrame {
    /// **A canvas that has not yet presented a frame**, seeded with the zoom
    /// the view opens at.
    ///
    /// `observed_zoom` takes that zoom rather than a sentinel so the first
    /// frame does not read as "the zoom just changed" and schedule a
    /// rasterization the opening render is already doing.
    pub fn new(zoom: f32) -> Self {
        Self {
            observed_zoom: zoom,
            zoom_commit_at: Instant::now(),
            zoom_commanded: false,
            zoom_anchor: None,
            last_scroll_offset: egui::Vec2::ZERO,
            deep_anchor: None,
            deep_zoom: None,
        }
    }
}
