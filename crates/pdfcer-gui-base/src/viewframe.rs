//! # `viewframe` — what one canvas remembers between frames
//!
//! [`ViewFrame`] is the per-**view** bookkeeping a canvas writes at the end of
//! a frame and reads at the start of the next: where the scroll area settled,
//! what the zoom was, where the zoom is anchored, and the deep tier's position
//! when the scroll offset can no longer carry it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/viewframe.md`.

use std::time::Instant;

use crate::deepanchor::DeepAnchor;

/// **What one canvas observed about itself on the previous frame.**
#[derive(Debug, Clone, Copy)]
pub struct ViewFrame {
    /// The zoom seen at the end of the previous frame, used to detect that
    /// the zoom changed at all.
    pub observed_zoom: f32,
    /// The earliest instant at which the current zoom may be committed to a
    /// real rasterization — the `pdfcer_gui::render::settle::ZOOM_SETTLE`
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
    /// **Where the view is, once the scroll offset can no longer say** —
    /// O24 tier 3.
    ///
    /// `None` below the sub-pixel content extent, where `egui::ScrollArea`'s
    /// own `f32` offset is authoritative and nothing about the canvas differs
    /// from before this feature. `Some` above it, where the position is a page
    /// point in `f64` and the screen pixel it sits under.
    ///
    /// Seeded on the way in from wherever the scroll area had settled, and
    /// cleared on the way out — so crossing the threshold in either direction
    /// does not move the page under the operator, and re-entering starts from
    /// the truth rather than from a stale anchor.
    pub deep_anchor: Option<DeepAnchor>,
    /// The zoom [`Self::deep_anchor`] was last valid at, or `None` outside the
    /// deep tier.
    ///
    /// **What makes zoom-to-cursor possible above the threshold.**
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

/// Where the pointer was over the page when a Ctrl+wheel arrived.
#[derive(Debug, Clone, Copy)]
pub struct ZoomAnchor {
    /// The pointer's position as a fraction of the page's drawn size.
    pub frac: (f32, f32),
    /// The scroll offset before the zoom step.
    pub offset_before: (f32, f32),
    /// The page's drawn size before the zoom step.
    pub display_before: (f32, f32),
    /// The scroll viewport, needed for the centring-margin term.
    pub viewport: (f32, f32),
    /// The page every other field is measured against: the one acted on
    /// when the anchor was armed. The anchor is solved a frame later, by
    /// which time the current page may have scrolled on, so the strip offset
    /// must be added for this page, not the current one.
    pub page: usize,
}
