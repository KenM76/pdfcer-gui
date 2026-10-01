//! # `renderquality` — how sharply a page is drawn, and how long zoom waits
//!
//! [`RenderQuality`] and the zoom-settle bounds: the two preferences that
//! change what a rendered frame costs.
//!
//! ## Why these two are one module and the opening view is another
//!
//! Not by size. These two are both about **the cost of drawing** — a trade of
//! sharpness or responsiveness against the time a machine takes — and both are
//! read on the hot path, by `pdfcer_gui::viewer::raster_scale` and by
//! `pdfcer_gui::app::settle` respectively. `pdfcer_gui::app::prefs::opening`'s preferences are
//! about **what an operator is shown first** and are read exactly once per
//! document, in the open path.
//!
//! That is the seam a future reader would look for, and it is the one that
//! decides where a new preference goes: *does this change what a frame costs,
//! or what the first frame contains?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/renderquality.md`.

/// How sharply a page is rasterised, as a multiplier on the natural scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderQuality {
    /// 0.75× — fewer pixels, softer lines, quicker.
    Faster,
    /// 1× — one raster pixel per device pixel. The default.
    #[default]
    Normal,
    /// 1.5× — more pixels than the display can show, for small text.
    Sharper,
}

impl RenderQuality {
    /// Every value, in the order the settings window lists them.
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
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|q| q.key() == key)
    }
}

/// The shortest zoom-settle delay offered, in milliseconds.
pub const MIN_SETTLE_MS: u64 = 20;

/// The longest offered.
pub const MAX_SETTLE_MS: u64 = 1000;

/// The shipped settle, in milliseconds.
pub const DEFAULT_SETTLE_MS: u64 = 150;

#[cfg(test)]
mod tests {
    use super::*;

    /// The tokens are stable and distinct.
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
