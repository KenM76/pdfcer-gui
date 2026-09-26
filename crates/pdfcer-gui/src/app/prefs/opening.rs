//! # `app::prefs::opening` — what an operator is shown when a page first appears
//!
//! Two preferences, both read **exactly once per document open** and never
//! again: how the first page is fitted, and which of the three View ▸ Display
//! overlays are already on.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/prefs/opening.md`.

use crate::viewer::FitMode;

/// How the first page of a newly opened document is sized to the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpeningFit {
    /// The whole page is visible. pdfcer's shipped answer.
    ///
    /// `ViewState::default`'s own comment argues it and the argument still
    /// holds: *"Opening at a raw 100% produces a wildly different first
    /// impression depending on the page size — a business card fills a thumb's
    /// worth of the window, an A0 poster overflows it — and both read as a bug
    /// even though nothing is wrong."*
    #[default]
    Page,
    /// The page's full width is visible; its height may run off the bottom.
    Width,
    /// The page's full height is visible; its width may run off the side.
    ///
    /// O29's mirror of [`Self::Width`], and the useful one for a landscape
    /// drawing sheet in a portrait window.
    Height,
    /// One page point per screen point, whatever that shows.
    ActualSize,
}

impl OpeningFit {
    /// Every value, in the order the settings window lists them.
    pub const ALL: &'static [Self] = &[Self::Page, Self::Width, Self::Height, Self::ActualSize];

    /// The token written to the preferences file.
    ///
    /// Stable across releases and deliberately not the display name, for the
    /// reason [`super::RenderQuality::key`] states at length.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::Page => "page",
            // ui-text-exempt: a file token, never displayed.
            Self::Width => "width",
            // ui-text-exempt: a file token, never displayed.
            Self::Height => "height",
            // ui-text-exempt: a file token, never displayed.
            Self::ActualSize => "actual",
        }
    }

    /// Read a token back, or `None` if it names nothing.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|f| f.key() == key)
    }

    /// The `(fit, zoom)` pair a [`crate::viewer::ViewState`] opens with.
    #[must_use]
    pub const fn to_view(self) -> (FitMode, f32) {
        match self {
            Self::Page => (FitMode::Page, 1.0),
            Self::Width => (FitMode::Width, 1.0),
            Self::Height => (FitMode::Height, 1.0),
            Self::ActualSize => (FitMode::None, 1.0),
        }
    }
}

/// Which of the three View ▸ Display overlays are already on when a document
/// opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PageChrome {
    /// Rulers down the top and left gutters.
    ///
    /// The one overlay with a **measurable** cost when on: it takes
    /// `canvas::rulers::THICKNESS_PTS` off two edges of every canvas, for every
    /// operator, on every document. That is why it ships off and why the
    /// setting's copy says what turning it on costs rather than presenting it
    /// as free.
    pub rulers: bool,
    /// The drafting grid over the page.
    pub grid: bool,
    /// Draggable guides.
    ///
    /// **Turning this on does not let an operator place a guide.** A guide is
    /// dragged out of a ruler gutter, so `rulers` must be on as well —
    /// `canvas::guides::ruler_drag` registers nothing without them. The setting
    /// says so, because the alternative is an operator switching one of the two
    /// on and concluding the feature is broken.
    ///
    /// It is also the one overlay that can be turned on **without this
    /// preference**, by a document that has remembered guides: `OpenDoc::assemble`
    /// reads `canvas::guides::opening`, whose rule is *"the presence of the work
    /// is the preference"*. That override still wins — see
    /// [`super::Prefs::seed_view`].
    pub guides: bool,
}

impl PageChrome {
    /// Whether every overlay is off — the shipped state.
    #[must_use]
    pub const fn all_hidden(self) -> bool {
        !self.rulers && !self.grid && !self.guides
    }
}

/// The token a `bool` preference is written as.
#[must_use]
pub const fn bool_key(value: bool) -> &'static str {
    if value {
        // ui-text-exempt: a file token, never displayed.
        "true"
    } else {
        // ui-text-exempt: a file token, never displayed.
        "false"
    }
}

/// Read a `bool` token back, or `None` if it is neither spelling.
#[must_use]
pub fn bool_from_key(key: &str) -> Option<bool> {
    match key {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tokens are stable and distinct.
    #[test]
    fn every_opening_fit_has_a_distinct_stable_token() {
        for f in OpeningFit::ALL {
            assert_eq!(OpeningFit::from_key(f.key()), Some(*f));
        }
        let keys: Vec<&str> = OpeningFit::ALL.iter().map(|f| f.key()).collect();
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j]);
            }
        }
        assert!(OpeningFit::from_key("nonesuch").is_none());
    }

    /// The shipped opening view is `fit: FitMode::Page, zoom: 1.0`, with every
    /// overlay off.
    #[test]
    fn the_shipped_opening_fit_is_what_the_constant_held() {
        let (fit, zoom) = OpeningFit::default().to_view();
        assert_eq!(fit, FitMode::Page);
        assert!((zoom - 1.0).abs() < f32::EPSILON);
        assert!(
            PageChrome::default().all_hidden(),
            "the three overlays shipped off and a build that omits nothing must \
             still open with them off"
        );
    }

    /// Every opening fit produces a usable zoom.
    #[test]
    fn every_opening_fit_yields_a_positive_zoom() {
        for f in OpeningFit::ALL {
            let (_, zoom) = f.to_view();
            assert!(zoom > 0.0, "{f:?} opens at a zoom of {zoom}");
        }
    }

    /// Only `ActualSize` pins the zoom.
    #[test]
    fn exactly_one_opening_fit_stops_following_the_window() {
        let pinned: Vec<OpeningFit> = OpeningFit::ALL
            .iter()
            .copied()
            .filter(|f| f.to_view().0 == FitMode::None)
            .collect();
        assert_eq!(pinned, vec![OpeningFit::ActualSize], "{pinned:?}");
    }

    /// A `bool` round-trips, and nothing else is accepted.
    #[test]
    fn a_bool_round_trips_and_a_typo_does_not_parse() {
        for value in [true, false] {
            assert_eq!(bool_from_key(bool_key(value)), Some(value));
        }
        for typo in ["ture", "yes", "on", "1", "True", ""] {
            assert!(bool_from_key(typo).is_none(), "{typo:?} parsed as a bool");
        }
    }
}
