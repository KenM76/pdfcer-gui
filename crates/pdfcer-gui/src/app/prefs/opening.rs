//! # `app::prefs::opening` — what an operator is shown when a page first appears
//!
//! Two preferences, both read **exactly once per document open** and never
//! again: how the first page is fitted, and which of the three View ▸ Display
//! overlays are already on.
//!
//! ## ★ Why these are preferences at all, and not just defaults somebody picked
//!
//! Both were compiled-in constants in `crate::viewer::ViewState::default`, and
//! `NO_SURFACE.md` §2 recorded them as such:
//!
//! | | value | where |
//! |---|---|---|
//! | Rulers / grid / guides **default visibility** | all `false` | `viewer/mod.rs:302-304` |
//! | Default fit mode | `FitMode::Page` | `viewer/mod.rs:300` |
//!
//! …with the note *"toggles exist (View ▸ Display); **the default is not
//! settable**"*. That is the exact shape the operator reported on 2026-08-17 —
//! *"there is no surface for changing or editing the settings for them"* — and
//! it is worse here than the phrase implies, because **the toggle is per
//! document**. There is no memory of it at all: `viewer::remembered` persists
//! the page-display arrangement and nothing else, deliberately (see its header).
//! So an operator who works with rulers on flicks the same switch on every
//! document they will ever open, forever, and the program never learns.
//!
//! ## ★ The trio is ONE setting, not three, because they interlock
//!
//! `canvas::guides`' [`ruler_drag`](crate::canvas::guides) states the coupling
//! in its own doc comment:
//!
//! > Registers nothing when the rulers are hidden, which is why the guides
//! > toggle is usable on its own but *creating* a guide needs rulers.
//!
//! So an operator who uses guides needs **two** switches before they can place
//! the first one, and they need them on every document. Presenting the three as
//! three separate settings, each with its own title, its own silence line and
//! its own radius line, would bury that relationship under three copies of the
//! same three sentences. One setting, three switches, one explanation — and the
//! explanation gets to say the thing that actually matters, which is that
//! placing a guide needs the ruler it is dragged from.
//!
//! ## What is deliberately NOT here
//!
//! **The page-display arrangement** — single, continuous, facing. It already
//! has a per-document store (`viewer::remembered`) built to an explicit
//! operator requirement: *"Mode persists per document, not globally — opening a
//! drawing set must not inherit a report's setting."* A global default for it
//! would be a second axis colliding with the per-document one, which is
//! `HANDOFF.md` §9's open question 2 and is deliberately unbuilt.
//!
//! The distinction is worth stating because it looks arbitrary from outside:
//! the display arrangement is remembered per document **because the right
//! answer differs per document** (a drawing set and a report want different
//! things). Ruler visibility does not vary that way — it is a property of how a
//! person works, not of what they are looking at — so a global preference is
//! the right shape for one and the wrong shape for the other.

use crate::viewer::FitMode;

/// How the first page of a newly opened document is sized to the window.
///
/// # Why the enum is not [`FitMode`] itself
///
/// [`FitMode`] has three variants and one of them, [`FitMode::None`], means
/// *"the operator pinned an explicit zoom; the viewport no longer influences
/// it"*. It carries no zoom of its own — the zoom lives beside it on
/// [`crate::viewer::ViewState`] — so `FitMode::None` on its own does not
/// describe a state a document can be opened in. It describes the absence of a
/// rule.
///
/// A preference has to name a **complete** opening state, so this enum's third
/// value is [`OpeningFit::ActualSize`], which is `FitMode::None` *and* a zoom
/// of exactly 1.0. [`Self::to_view`] is where the pair is produced, and it is
/// the only place the pairing is stated.
///
/// Storing `FitMode` directly would have shipped a preference file in which
/// `opening_fit = none` was legal and meant nothing.
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
    ///
    /// Whole-page first because it is the default and the least surprising,
    /// then the two single-axis fits, then actual size — which is *most* zoomed on the drawings
    /// this shell is for and therefore reads as the far end of a scale.
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
    ///
    /// # The zoom returned for the two fitting modes is not ignored
    ///
    /// `FitMode::Page`, `FitMode::Width` and `FitMode::Height` are recomputed
    /// every frame against the viewport, so the zoom handed back for them is only what the state
    /// holds until the first frame measures the window. It is `1.0` rather than
    /// `0.0` because a `ViewState` is legal to inspect before any frame has run
    /// — `OpenDoc::assemble` copies it straight into `observed_zoom` — and a
    /// zero there would make the first `observed_zoom` comparison meaningless
    /// and could divide by zero in any geometry that scales by it.
    ///
    /// Returning a pair rather than mutating a `&mut ViewState` keeps this
    /// pure, which is what lets [`tests`] assert the mapping without building a
    /// document.
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
///
/// A struct of three `bool`s rather than three loose fields on [`super::Prefs`],
/// because they are one setting in the window and one line in this module's
/// reasoning — see the header. Grouping them here also means the settings
/// window's control takes one argument rather than three, so a fourth overlay
/// added later changes one signature instead of every call site.
///
/// # ★ These are file-format `bool`s, and they are the first in the project
///
/// `pdfcer_core::settings` has **no boolean settings at all** — every one of its
/// thirteen is a named enum, because a named enum states what each side means
/// and `true`/`false` does not. That is a good rule and it is deliberately not
/// followed here, for a reason that is about the *control* rather than the
/// file: a switch is not a choice between named alternatives, and rendering
/// "rulers shown / rulers hidden" as a two-option radio group would draw six
/// controls where three belong and would imply the six were somehow related.
///
/// The file pays a small price for that — `show_rulers = true` says less than
/// `mask_resample = nearest` does — and the file's own comment block pays it
/// back by naming both legal values.
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
    /// ★ **Turning this on does not let an operator place a guide.** A guide is
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
    ///
    /// Used by the file writer to decide nothing, and by the tests to assert
    /// that a build which omits nothing behaves as the build before this
    /// module did.
    #[must_use]
    pub const fn all_hidden(self) -> bool {
        !self.rulers && !self.grid && !self.guides
    }
}

/// The token a `bool` preference is written as.
///
/// Two spellings and no synonyms. Accepting `yes`/`on`/`1` as well would mean
/// the writer picks one of four and the file then teaches the operator a
/// spelling different from the one they wrote — and a value pdfcer silently
/// rewrites is exactly what [`super::PrefNote`] exists to make impossible.
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
///
/// `None` rather than "anything that is not `true` is `false`", which is the
/// conventional lenient reading and is wrong here: an operator who typed
/// `show_rulers = ture` would get rulers off, which is also what they would get
/// from a correct `false`, and nothing would ever tell them. The per-key
/// recovery contract turns that into a reported [`super::PrefNote::BadValue`].
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
    ///
    /// Same property [`super::RenderQuality`] is held to, and for the same
    /// reason: two values sharing a token makes one unreachable from a
    /// hand-edited file.
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

    /// ★ The shipped opening view is the one the constant it replaced held.
    ///
    /// `ViewState::default` was `fit: FitMode::Page, zoom: 1.0` before this
    /// module existed. A build whose operator never opens the Settings window
    /// has to behave exactly as the build before it did — the standing rule for
    /// a capability becoming choosable, and the check that would catch a
    /// reordering of `ALL` that moved the `#[default]`.
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

    /// ★ Every opening fit produces a usable zoom.
    ///
    /// Not a tautology: `to_view` is the only place the `(FitMode, zoom)`
    /// pairing is stated, and a zero or negative zoom for any of the three
    /// would divide by zero in the canvas geometry rather than fail here. The
    /// two fitting modes recompute against the viewport on the first frame, but
    /// `OpenDoc::assemble` copies the zoom into `observed_zoom` **before** that
    /// frame runs.
    #[test]
    fn every_opening_fit_yields_a_positive_zoom() {
        for f in OpeningFit::ALL {
            let (_, zoom) = f.to_view();
            assert!(zoom > 0.0, "{f:?} opens at a zoom of {zoom}");
        }
    }

    /// ★ Only `ActualSize` pins the zoom.
    ///
    /// The distinction the enum exists to make: two of the three values are
    /// *rules* recomputed every frame, and one is a *pinned number*. If a future
    /// edit made `Width` produce `FitMode::None`, the page would open at the
    /// right size and then stop resizing with the window, which reads as a
    /// layout bug rather than as a preference.
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
    ///
    /// The second half is the point. A lenient parser that read every unknown
    /// token as `false` would give a typo and a correct `false` the same
    /// outcome and no report, which is the silent substitution the whole
    /// note mechanism exists to prevent.
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
