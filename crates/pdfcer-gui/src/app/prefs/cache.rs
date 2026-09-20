//! # `app::prefs::cache` — how much memory pdfcer may spend so a page it has
//! already drawn does not have to be drawn again
//!
//! One preference. The operator's ask: *"increase cache to maximum for page
//! view so they don't constantly redraw with larger files."*
//!
//! ## A budget only bites if the cache keeps pages that are not on screen
//!
//! `render::strip::StripRasters::retain` evicts by distance from the current
//! page, so the resident set is many times the visible set and the number here
//! is what bounds it. That ordering is what makes this preference mean
//! anything. A cache whose contents *are* the visible set is not a cache; it is
//! a frame buffer with extra steps — scroll a sheet off the top and it is gone,
//! scroll back and it is rendered again from the content stream, which
//! `BENCHMARK.md` measures at **691 ms** for a dense A1 drawing. Over such a
//! cache the eviction loop never runs and raising the budget changes nothing at
//! all, which is worth knowing: *"increase the cache"* is an instruction a
//! reader can carry out by editing one constant and reporting success.
//!
//! ## Why it is a preference and not a bigger constant
//!
//! Because the honest answer to *"how much of this machine's memory may pdfcer
//! spend on page pictures"* is that only the person sitting at it knows. A
//! constant is this project guessing about a machine it cannot see, and the
//! guess has a bad failure mode in one direction: too large is not slow, it is
//! an allocation failure in a program that is now holding unsaved edits.
//!
//! It follows [`crate::dialogs::settings`]' own standing rule, stated by the
//! operator — *where standards are ambiguous those should become settings that
//! the user can choose, with the initial installed default as the best guess of
//! what is usually followed*. There is no standard here, but the shape is the
//! same: a defensible default, named steps, and every one of them stating its
//! cost.
//!
//! ## Every step states its cost in megabytes, and that is not decoration
//!
//! *"Large"* is not a number anybody can budget against. An operator with 8 GB
//! and an operator with 64 GB are making different decisions, and neither can
//! make theirs from an adjective. So the labels carry the figure —
//! `crate::text::settings::shell::page_cache_label` renders it — and the figures
//! are exact rather than rounded up, because a memory number that flatters
//! itself is the one kind of disclosure worse than none.
//!
//! The arithmetic, once: a texel is one RGBA pixel, four bytes, and a megabyte
//! here is 1,048,576 bytes, so 256 M texels is 1,024,000,000 bytes and reports
//! as **976 MB**. There is no compression and no shared storage — these are GPU
//! textures — so the figure is what it says. The texel counts are round in
//! decimal and the megabyte figures therefore are not; the megabyte figure is
//! the one the operator is shown, so it is the one the tables below carry.

/// How much memory the page cache may hold, as four named steps.
///
/// # Why four steps and not a slider
///
/// [`super::quality::RenderQuality`]'s argument applies unchanged: *the useful
/// range is narrow, the middle of it is almost always right, and a slider
/// invites an operator to spend attention tuning a number that will not repay
/// it.* It applies with more force here, because the effect of an intermediate
/// value is unobservable — an operator cannot tell 900 MB from 1,024 MB by
/// using the program, so a slider would be asking for precision that cannot be
/// felt.
///
/// # The steps, and what each is for
///
/// The megabyte column is what [`Self::megabytes`] reports and what the label
/// shows; it is the texel count times four bytes over 1,048,576.
///
/// | | texels | RGBA | roughly |
/// |---|---|---|---|
/// | [`Self::Small`] | 48 M | 183 MB | a few large sheets |
/// | [`Self::Medium`] | 128 M | 488 MB | a report, or a dozen large sheets |
/// | [`Self::Large`] | 256 M | 976 MB | **the default** — about twenty-five large sheets at screen size |
/// | [`Self::Maximum`] | 512 M | 1,953 MB | a whole drawing set resident |
///
/// [`Self::Small`] is kept so an operator who finds the default heavy has a
/// smaller budget available **by name**, rather than having to discover a
/// number.
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
    ///
    /// Smallest-first so the control reads left to right as *less … more*,
    /// which is [`super::quality::RenderQuality::ALL`]'s rule and the direction
    /// a reader expects of a quantity.
    pub const ALL: &'static [Self] = &[Self::Small, Self::Medium, Self::Large, Self::Maximum];

    /// The budget in texels — what `render::strip` spends.
    ///
    /// Texels rather than bytes because that is the unit the cache counts in,
    /// and it counts in texels because **a page is not a unit of memory**: a
    /// thumbnail and an Annex C sheet differ by four orders of magnitude, so a
    /// page count that admitted six of the latter would admit 1.5 GB without
    /// saying so.
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
    ///
    /// Derived from [`Self::texels`] rather than written beside it, which is
    /// this project's recurring lesson applied before it bites: two spellings
    /// of one quantity drift, and the drift here would be a settings window
    /// promising an operator 488 MB while the cache spent 2 GB. It is
    /// `NO_SURFACE.md`'s standing rule — assert the *relation*, because two
    /// copies of one constant cannot disagree — applied to a label.
    #[must_use]
    pub const fn megabytes(self) -> u64 {
        // Four bytes per RGBA texel; 1 MB = 1,048,576 bytes.
        self.texels() * 4 / 1_048_576
    }

    /// The token written to the preferences file.
    ///
    /// Stable across releases and deliberately not the display name: a display
    /// name is operator copy and may be reworded, and a file whose keys moved
    /// when the wording did would silently reset everybody's preference. Same
    /// rule [`super::quality::RenderQuality::key`] follows.
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
    ///
    /// `None` rather than a default, so the loader can *report* an unreadable
    /// value rather than silently substituting one — the per-key recovery
    /// contract in [`super`]'s header.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.key() == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The steps increase, and every one of them is distinct.
    ///
    /// A control whose second entry held less than its first would read as
    /// broken, and the labels are generated from the numbers, so a duplicate
    /// would render as two identical rows.
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
    ///
    /// Asserted as the *relation* rather than against four literals, which is
    /// the whole point: two copies of one constant cannot disagree, so a test
    /// written against literals would pass on a build whose label lied.
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
    ///
    /// The row that makes the default reversible by name. An operator who finds
    /// the default heavy must be able to ask for a smaller budget without
    /// knowing that it is 48 million of anything, so the number behind the name
    /// is held here rather than left free to drift.
    #[test]
    fn small_is_the_value_this_shell_shipped_with() {
        assert_eq!(PageCache::Small.texels(), 48_000_000);
    }

    /// The default is `Large`, and it is not the largest.
    ///
    /// Both halves are the decision recorded on the variant: the operator asked
    /// for the maximum, and taking 2 GB on his behalf risks an allocation
    /// failure in a program holding unsaved edits. The larger step exists and is
    /// one click away.
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
