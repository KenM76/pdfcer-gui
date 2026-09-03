//! # `app::prefs::quality` — how sharply a page is drawn, and how long zoom waits
//!
//! The two preferences that shipped with [`super`] on 2026-08-17, split out of
//! its `mod.rs` on 2026-08-17 when the opening-view preferences arrived and the
//! file approached rule R2's 1,500-line ceiling.
//!
//! ## Why these two are one module and the opening view is another
//!
//! Not by size. These two are both about **the cost of drawing** — a trade of
//! sharpness or responsiveness against the time a machine takes — and both are
//! read on the hot path, by [`crate::viewer::raster_scale`] and by
//! [`crate::render::settle`] respectively. [`super::opening`]'s preferences are
//! about **what an operator is shown first** and are read exactly once per
//! document, in the open path.
//!
//! That is the seam a future reader would look for, and it is the one that
//! decides where a new preference goes: *does this change what a frame costs,
//! or what the first frame contains?*

/// How sharply a page is rasterised, as a multiplier on the natural scale.
///
/// # What "natural" is, and why this multiplies rather than replaces
///
/// `viewer::raster_scale` is `zoom × pixels_per_point`: one raster pixel per
/// *device* pixel, which is the scale at which a page is exactly as sharp as
/// the display can show and no sharper. That is the right default and it is
/// what [`RenderQuality::Normal`] means.
///
/// The two other values trade against it in opposite directions, and both are
/// real needs on the drawings this shell is for:
///
/// - **Faster** renders at 0.75× and lets the GPU upscale. On the benchmark
///   CAD sheet — 5.6 MB of dense vector site plan — that is roughly half the
///   pixels and therefore roughly half the rasterisation time, at the cost of
///   softness that is most visible on the thin linework such a drawing is
///   made of. An operator panning around a big sheet looking for something may
///   well want it; an operator checking a dimension will not.
/// - **Sharper** renders at 1.5×. Pointless on most content and genuinely
///   better on small text over a hairline grid, where a device pixel straddles
///   two strokes and neither survives.
///
/// # Why three values and not a slider
///
/// Because the useful range is narrow and the middle of it is almost always
/// right. A slider invites an operator to spend attention tuning a number that
/// will not repay it, and — more practically — every intermediate value costs a
/// full re-raster of every visible page to evaluate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderQuality {
    /// 0.75× — fewer pixels, softer lines, quicker.
    Faster,
    /// 1× — one raster pixel per device pixel. The shipped answer.
    #[default]
    Normal,
    /// 1.5× — more pixels than the display can show, for small text.
    Sharper,
}

impl RenderQuality {
    /// Every value, in the order the settings window lists them.
    ///
    /// Worst-to-best rather than best-to-worst, so the control reads left to
    /// right as *less … more* — which is the direction a reader expects of a
    /// quality scale and the opposite of the order the enum's own reasoning
    /// arrived in.
    pub const ALL: &'static [Self] = &[Self::Faster, Self::Normal, Self::Sharper];

    /// The multiplier applied to the natural raster scale.
    #[must_use]
    pub const fn multiplier(self) -> f32 {
        match self {
            Self::Faster => 0.75,
            Self::Normal => 1.0,
            Self::Sharper => 1.5,
        }
    }

    /// The token written to the preferences file.
    ///
    /// Stable across releases and deliberately not the display name: a display
    /// name is operator copy and may be reworded or translated, and a file
    /// whose keys moved when the wording did would silently reset everybody's
    /// preference. Same rule `egui_shell::theme::Preset::key` follows.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::Faster => "faster",
            // ui-text-exempt: a file token, never displayed.
            Self::Normal => "normal",
            // ui-text-exempt: a file token, never displayed.
            Self::Sharper => "sharper",
        }
    }

    /// Read a token back, or `None` if it names nothing.
    ///
    /// `None` rather than a default, so the loader can *report* an unreadable
    /// value rather than silently substituting one — the per-key recovery
    /// contract in the module header.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|q| q.key() == key)
    }
}

/// The shortest zoom-settle delay offered, in milliseconds.
///
/// Zero is excluded and that is a decision. A settle of zero means *rasterise
/// every intermediate value of a wheel gesture*, which on a dense CAD sheet is
/// dozens of full-page renders producing images nobody sees — the exact cost
/// the debounce exists to avoid. 20 ms is short enough to feel immediate and
/// long enough to swallow the burst of events one wheel notch produces.
pub const MIN_SETTLE_MS: u64 = 20;

/// The longest offered.
///
/// Beyond about a second the interim scaled texture stops reading as "still
/// settling" and starts reading as "stuck", which is a worse impression than
/// the CPU cost it saves.
pub const MAX_SETTLE_MS: u64 = 1000;

/// The shipped settle, in milliseconds.
///
/// 150 ms is the value the old shell settled on against real CAD sheets, and it
/// was `render::settle::ZOOM_SETTLE`'s compiled-in constant before this module
/// existed. It stays the default for the standing reason: a build that omits
/// nothing must behave as it did before the choice existed.
pub const DEFAULT_SETTLE_MS: u64 = 150;

#[cfg(test)]
mod tests {
    use super::*;

    /// The tokens are stable and distinct.
    ///
    /// They are what the file holds, so two quality values sharing a token
    /// would make one of them unreachable from a hand-edited file, and a token
    /// that changed with a display name would reset everybody's preference on
    /// upgrade.
    #[test]
    fn every_quality_has_a_distinct_stable_token() {
        for q in RenderQuality::ALL {
            assert_eq!(RenderQuality::from_key(q.key()), Some(*q));
        }
        let keys: Vec<&str> = RenderQuality::ALL.iter().map(|q| q.key()).collect();
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j]);
            }
        }
        assert!(RenderQuality::from_key("nonesuch").is_none());
    }
}
