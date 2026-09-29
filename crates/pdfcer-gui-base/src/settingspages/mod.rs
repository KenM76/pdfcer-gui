//! # `settingspages` — the Settings window pages that need no document state
//!
//! Each page renders one group of the draft `Prefs` or engine `Settings`.
//! The window itself, its page list and the pages that reach app state
//! (Acrobat, fonts, display, navigation, signatures) are the app's
//! `dialogs::settings`, which re-exports these under their old names.

pub mod appearance;
pub mod colour;
pub mod comments;
pub mod forms;
pub mod images;
pub mod measuring;
pub mod pages;
pub mod preset;
pub mod redaction;
pub mod saving;
pub mod text;
pub mod widgets;

/// The region each theme radio publishes, suffixed with the preset's key.
pub const REGION_THEME_PREFIX: &str = "settings.theme."; // ui-text-exempt: trace region name, never displayed

use pdfcer_core::settings::Settings;

use crate::defaultappsetting as defaultapp;

/// How far the working copy has drifted from what the window opened on.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// **Which page to select when the window opens.**
    ///
    /// `None` opens the first page. `Some` is for a command that routes here
    /// because of one page, such as Tools ▸ Font folders: a route that exists
    /// because of one setting must land on that setting. Taken rather than
    /// read, so it applies once.
    pub focus: Option<&'static str>,
    /// **What the machine says about the default PDF program**, read once
    /// per opening of this window - O173.
    ///
    /// Not a setting and not part of the draft: nothing here is written by
    /// Save and nothing here is discarded by Cancel, because the act it
    /// describes happens immediately and outside pdfcer. It lives on the
    /// draft only because the draft is what has this window's lifetime, and
    /// one probe per opening is the whole point. See
    /// [`defaultapp::State`].
    pub default_app: defaultapp::State,
    /// The edits in progress.
    pub working: Settings,
    /// What the settings were when the window opened.
    pub original: Settings,
    /// **Which preset the operator CHOSE**, as distinct from which one
    /// their settings happen to match.
    ///
    /// The operator's words for what a derived selection looks like: *"in the
    /// settings for the standards compatibility I can only select
    /// (ISO15930-1, -4)"*.
    ///
    /// **A choice is not a value**, and a control that derives one from the
    /// other can only ever show as many states as there are distinct values.
    /// That is fine while the presets differ and silently wrong the moment two
    /// agree — which is not a rare accident here but the **normal** case, since
    /// the standards genuinely make the same demands of a renderer:
    /// `preset::identical_siblings` measures that all eight of the PDF/X and
    /// PDF/A presets apply byte-identical render settings, differing only in
    /// PDF/UA, which leaves `image_minify` alone. Derived from the values, the
    /// dot for every one of the eight lands on whichever `preset::matching`
    /// finds first, and the program looks broken while behaving exactly as
    /// written.
    ///
    /// Two consequences follow, and both are what this field exists to avoid:
    ///
    /// * **Save greyed out** after choosing a standard — a second choice that
    ///   moves no value leaves [`Self::is_dirty`] correctly false about a draft
    ///   that really does equal what was saved.
    /// * **The choice discarded**, so reopening shows whichever standard
    ///   `preset::matching` finds first: choose PDF/X-4, come back, read
    ///   PDF/X-1a.
    ///
    /// **It is persisted**, in [`crate::prefs::Prefs::chosen_standard`],
    /// whose docs carry the operator report and the full argument; this field
    /// is the in-window working copy. Persisting it makes a choice a change,
    /// which makes Save live, and lets the window say next week what was asked
    /// for.
    pub chosen_preset: Option<&'static str>,
    /// The shell's own preferences, in progress.
    ///
    /// **A second pair, not a second draft.** The window edits two stores —
    /// `pdfcer_core::settings` and `crate::prefs` — and the operator must
    /// not be able to tell: one Cancel discards both, one Save writes both, and
    /// `is_dirty` is true if *either* moved. A separate draft per store would
    /// give the window two Save buttons or one that lied.
    ///
    /// They are two stores because they answer different questions — see
    /// `crate::prefs`' header — and that is an implementation fact the
    /// operator has no business meeting.
    pub working_prefs: crate::prefs::Prefs,
    /// What the preferences were when the window opened.
    pub original_prefs: crate::prefs::Prefs,
}

impl Draft {
    /// Start editing from the **live** configuration.
    #[must_use]
    pub fn new(current: &Settings, prefs: &crate::prefs::Prefs) -> Self {
        Self::focused_on(current, prefs, None)
    }

    /// [`Self::new`], opened at one page. See [`Self::focus`].
    #[must_use]
    pub fn focused_on(
        current: &Settings,
        prefs: &crate::prefs::Prefs,
        focus: Option<&'static str>,
    ) -> Self {
        Self {
            focus,
            // Deliberately empty rather than probed: filling it here would
            // spawn subprocesses from a constructor that unit tests call.
            // See `defaultapp::State`.
            default_app: defaultapp::State::default(),
            working: current.clone(),
            original: current.clone(),
            // Seeded from the operator's PERSISTED choice, so the window
            // opens saying what they asked for rather than what their values
            // happen to resemble. `preset::live_choice` still filters it
            // through `still_holds`, so a stored id whose settings have since
            // been changed by hand is not shown — the fallback reading takes
            // over exactly where the claim stops being true.
            //
            // Resolved to the `&'static str` the choice list owns rather than
            // kept as the stored `String`: an id this build does not know is
            // not a choice it can offer, and dropping it here means every later
            // reader is comparing against the real list instead of a string
            // that might match nothing.
            chosen_preset: preset::resolve_id(prefs.chosen_standard.as_deref()),
            working_prefs: prefs.clone(),
            original_prefs: prefs.clone(),
        }
    }

    /// Whether anything has actually changed since the window opened.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.working != self.original || self.working_prefs != self.original_prefs
    }

    /// Whether every value is still pdfcer's own answer.
    #[must_use]
    pub fn is_all_default(&self) -> bool {
        self.working == Settings::default() && self.working_prefs == crate::prefs::Prefs::default()
    }
}
