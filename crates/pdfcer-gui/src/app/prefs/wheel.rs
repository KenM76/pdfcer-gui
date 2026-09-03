//! # `prefs::wheel` — what the mouse wheel does when the document is not a scroll
//!
//! ## The request
//!
//! `OPERATOR_REQUESTS.md` O30, 2026-08-24:
//!
//! > *"when in single page view there should be an option on screen near the
//! > button to scroll or flip through pages, or the current way it is now when
//! > the scroll wheel is used."*
//!
//! ## ★ Why this is a choice at all, rather than a behaviour
//!
//! Under a **continuous** display mode there is nothing to decide: the whole
//! document is one scroll and the wheel scrolls it. Under
//! [`crate::viewer::PageDisplay::Single`] and
//! [`crate::viewer::PageDisplay::Facing`] the wheel is ambiguous, and the two
//! answers are both right for somebody:
//!
//! * **Scroll the page.** The wheel moves within the sheet and never leaves
//!   it. This is what the build has always done, so it is the default — an
//!   option that changes what the operator already has is not an option, it is
//!   a surprise.
//! * **Flip pages.** The wheel turns to the next or previous sheet. On a
//!   drawing set opened at fit-page — which is how this shell opens documents
//!   by default — there is *nothing to scroll*, so today's wheel does nothing
//!   at all and the operator reaches for the page buttons every time.
//!
//! ★★ That last sentence is the whole case for the feature. The default
//! behaviour is not merely a matter of taste in the common configuration; it
//! is a dead control.
//!
//! ## The two rules that keep this honest
//!
//! 1. **Ctrl+wheel is untouched.** `egui` routes a modified wheel event to
//!    `zoom_delta` and contributes nothing to the scroll delta, so zoom is not
//!    a case this module has to exclude — it never arrives here. Breaking that
//!    separation is, per `canvas`'s own header, *"the single most common way a
//!    from-scratch viewer feels wrong"*.
//! 2. **The control renders only where the choice exists** — R9. Under a
//!    continuous mode nothing is drawn, rather than a disabled stub explaining
//!    that the setting does not apply.

/// **What a plain wheel does under a one-page-at-a-time display mode.**
///
/// A two-value enum rather than a `bool` for the reason
/// [`super::OpeningFit`] is one: the file token is then a *word* the operator
/// can read and correct, the settings window can give each answer its own
/// sentence, and a third answer (flip only once the page's edge is reached,
/// say) is an added variant rather than a changed type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WheelPaging {
    /// The wheel scrolls within the current page. **Today's behaviour, and
    /// therefore the default.**
    #[default]
    Scroll,
    /// The wheel turns to the next or previous page.
    FlipPages,
}

impl WheelPaging {
    /// Every value, in the order the settings window lists them.
    ///
    /// Scrolling first because it is the default and the one already in the
    /// operator's hands.
    pub const ALL: &'static [Self] = &[Self::Scroll, Self::FlipPages];

    /// The token written to the preferences file.
    ///
    /// Stable across releases and deliberately not the display name — see
    /// [`super::RenderQuality::key`] for the argument in full.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::Scroll => "scroll",
            // ui-text-exempt: a file token, never displayed.
            Self::FlipPages => "flip",
        }
    }

    /// Read a token back, or `None` if it names nothing.
    ///
    /// Derived from [`Self::ALL`] and [`Self::key`] so it cannot drift from
    /// the writer — the same shape [`super::OpeningFit::from_key`] uses.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|w| w.key() == key)
    }

    /// Whether the wheel should turn pages rather than scroll.
    ///
    /// The single predicate the canvas asks. It exists so that the canvas
    /// never matches on this enum: a second `match` would be a second place to
    /// forget a variant, and the canvas's question is genuinely a yes/no even
    /// though the setting is not.
    #[must_use]
    pub fn flips(self) -> bool {
        matches!(self, Self::FlipPages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every value has a distinct, stable file token that survives a round
    /// trip — the shape `OpeningFit`'s own test uses, and the reason a new
    /// variant cannot ship with a duplicate or missing token.
    #[test]
    fn every_wheel_paging_has_a_distinct_stable_token() {
        for value in WheelPaging::ALL {
            assert_eq!(
                WheelPaging::from_key(value.key()),
                Some(*value),
                "a token must read back as the value that wrote it"
            );
        }
        for (i, a) in WheelPaging::ALL.iter().enumerate() {
            for b in &WheelPaging::ALL[i + 1..] {
                assert_ne!(a.key(), b.key(), "two values share a file token");
            }
        }
    }

    /// An unknown token is refused rather than guessed at, so a typo in the
    /// file becomes a `PrefNote::BadValue` the operator is shown.
    #[test]
    fn an_unknown_token_is_refused() {
        assert_eq!(WheelPaging::from_key("page"), None);
        assert_eq!(WheelPaging::from_key(""), None);
    }

    /// ★ The default is today's behaviour, and that is a promise rather than
    /// an accident: a new option that changed what the operator already had
    /// would be a surprise delivered by an upgrade.
    #[test]
    fn the_default_is_the_behaviour_the_build_already_had() {
        assert_eq!(WheelPaging::default(), WheelPaging::Scroll);
        assert!(!WheelPaging::default().flips());
    }

    /// The canvas's predicate agrees with the variant it is derived from.
    #[test]
    fn only_flip_pages_flips() {
        assert!(!WheelPaging::Scroll.flips());
        assert!(WheelPaging::FlipPages.flips());
    }
}
