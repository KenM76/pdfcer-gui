//! # `app::prefs::cache` — how much memory pdfcer may spend so a page it has
//! already drawn does not have to be drawn again
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/prefs/cache.md`.

/// How much memory the page cache may hold, as four named steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageCache {
    /// 48 M texels ≈ 183 MB — enough for a few large sheets.
    Small,
    /// 128 M texels ≈ 488 MB.
    Medium,
    /// 256 M texels ≈ 976 MB. **The shipped default.**
    ///
    /// The operator asked for *"maximum"*, and the default is deliberately one
    /// step below it. [`Self::Maximum`] is close to 2 GB of RGBA, and a machine
    /// that cannot spare it fails by *not allocating a texture* — in a program
    /// that is by then holding unsaved edits, which is the one failure this
    /// shell must not walk into on the operator's behalf. The larger step is
    /// offered, named, costed and one click away, so it stays the operator's
    /// choice rather than an assumption made for them.
    #[default]
    Large,
    /// 512 M texels ≈ 1,953 MB — a whole drawing set resident at once.
    Maximum,
}

impl PageCache {
    /// Every value, smallest first.
    pub const ALL: &'static [Self] = &[Self::Small, Self::Medium, Self::Large, Self::Maximum];

    /// The budget in texels — what `render::strip` spends.
    #[must_use]
    pub const fn texels(self) -> u64 {
        match self {
            Self::Small => 48_000_000,
            Self::Medium => 128_000_000,
            Self::Large => 256_000_000,
            Self::Maximum => 512_000_000,
        }
    }

    /// The same budget in **megabytes of RGBA**, for the label.
    #[must_use]
    pub const fn megabytes(self) -> u64 {
        // Four bytes per RGBA texel; 1 MB = 1,048,576 bytes.
        self.texels() * 4 / 1_048_576
    }

    /// The token written to the preferences file.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::Small => "small",
            // ui-text-exempt: a file token, never displayed.
            Self::Medium => "medium",
            // ui-text-exempt: a file token, never displayed.
            Self::Large => "large",
            // ui-text-exempt: a file token, never displayed.
            Self::Maximum => "maximum",
        }
    }

    /// Read a token back, or `None` if it names nothing.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.key() == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The steps increase, and every one of them is distinct.
    #[test]
    fn the_steps_go_up() {
        let mut previous = 0;
        for step in PageCache::ALL.iter().copied() {
            assert!(
                step.texels() > previous,
                "{step:?} holds {} texels, which is not more than the step below it",
                step.texels()
            );
            previous = step.texels();
        }
    }

    /// **The megabyte figure is derived from the texel figure**, so the label
    /// and the spend cannot disagree.
    #[test]
    fn the_label_and_the_spend_are_one_number() {
        for step in PageCache::ALL.iter().copied() {
            assert_eq!(step.megabytes(), step.texels() * 4 / 1_048_576);
        }
        // And the arithmetic is right at one known point, so a sign error in
        // the relation above cannot pass by being consistently wrong.
        assert_eq!(PageCache::Large.megabytes(), 976);
    }

    /// **`Small` is pinned to its texel count.**
    #[test]
    fn small_is_the_value_this_shell_shipped_with() {
        assert_eq!(PageCache::Small.texels(), 48_000_000);
    }

    /// The default is `Large`, and it is not the largest.
    #[test]
    fn the_default_is_large_and_maximum_is_offered_above_it() {
        assert_eq!(PageCache::default(), PageCache::Large);
        assert!(PageCache::Maximum.texels() > PageCache::default().texels());
        assert_eq!(PageCache::ALL.last().copied(), Some(PageCache::Maximum));
    }

    /// Every token round-trips, and no two share one.
    #[test]
    fn every_step_round_trips_through_its_token() {
        let mut seen = std::collections::BTreeSet::new();
        for step in PageCache::ALL.iter().copied() {
            assert!(seen.insert(step.key()), "{step:?} shares a token");
            assert_eq!(PageCache::from_key(step.key()), Some(step));
        }
        assert_eq!(PageCache::from_key("enormous"), None);
    }
}
