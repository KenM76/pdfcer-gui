//! # `viewer::display` — how many pages are on screen, and in what arrangement
//!
//! One enum, [`PageDisplay`], and the three rules that hang off it: which
//! modes scroll, which modes pair pages into spreads, and what a fresh
//! profile gets in each ribbon mode.
//!
//! ## ★ Continuous is an option, not a replacement — and that is the whole point
//!
//! The operator's instruction, verbatim, 2026-08-12:
//!
//! > *"continuous scroll should be an option under the view tab as the way I
//! > move around a page is great when working with drafting drawings."*
//!
//! [`PageDisplay::Single`] is therefore **not** a legacy mode and is not on a
//! path to removal. Paging one sheet at a time is the right model for drafting
//! review — a drawing sheet is a unit of work, and a page boundary is a
//! deliberate step rather than an interruption — and it stays the default
//! everywhere except Read. `GUI_ROADMAP.md`'s decision table records the same
//! ruling in one line: *"An option, not a replacement. Single page stays the
//! default; the four modes sit together on View."*
//!
//! The consequence for anybody editing this module: a change that makes
//! `Single` a degenerate `Continuous` — one page in a scrolling strip, with
//! the strip's gaps, the strip's scroll range and the strip's current-page
//! tracking — has failed even if every test stays green. [`crate::viewer::strip`]
//! is built so that `Single` produces a one-row strip whose size **is** the
//! page's drawn size and whose scroll range **is** the page's own, so the
//! single-page experience is bit-for-bit what it was before Phase 4. The tests
//! in that module assert exactly that, and they are the ones to keep honest.
//!
//! ## ★ Read defaults to continuous; every other mode keeps single page
//!
//! `MODES_AND_PANELS.md`'s table, and the operator decision of 2026-08-13 that
//! settled it:
//!
//! > *"Read defaults to continuous scroll; Review and Edit default to single
//! > page. … Reading a document is a continuous act — you scroll through it,
//! > and a page boundary is an interruption. Marking up and editing a drawing
//! > is a per-sheet act — you work on one sheet, and paging is how you move
//! > between them deliberately. The right default was never global; it was per
//! > mode."*
//!
//! [`PageDisplay::default_for_mode`] is that sentence in code, and it is the
//! **only** place the rule is written down. It takes a mode id as a `&str`
//! rather than a `crate::app::modes` type on purpose: the three ids are the
//! manifest's own (`"read"`, `"review"`, `"edit"`), a customized manifest may
//! declare others, and an unknown mode has to fall back to something rather
//! than fail. It falls back to `Single`, because `Single` is the default and an
//! unrecognised mode is not evidence that the operator wants a different one.
//!
//! ## Why the on-disk spelling lives here
//!
//! [`PageDisplay::id`] and [`PageDisplay::from_id`] are the persistence
//! format, and they sit beside the enum rather than in the store
//! ([`crate::viewer::remembered`]) for the reason every "one spelling" rule in
//! this project has: a variant added here with no line in `id` would round-trip
//! as something else, and the pair is asserted exhaustively by
//! [`tests::every_mode_round_trips_through_its_on_disk_spelling`] over
//! [`PageDisplay::ALL`] — so adding a variant and forgetting its spelling is a
//! test failure rather than a silent data loss the operator discovers on the
//! next launch.

/// **How many pages the canvas shows, and how they are arranged.**
///
/// The four positions of View ▸ Page display. Exactly one is active at a time
/// — it is a radio, not four toggles — which is why this is an enum on
/// [`crate::viewer::ViewState`] rather than a pair of booleans. Two booleans
/// would admit a fifth state ("facing, but also single") that means nothing,
/// and the ribbon would have to reconstruct which of them is "on".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum PageDisplay {
    /// One page, filling the canvas. Paging is how you move between pages.
    ///
    /// **The default**, everywhere except Read mode. See the module header.
    #[default]
    Single,
    /// Every page in one vertical scroll, one page per row.
    Continuous,
    /// One spread — up to two pages side by side — at a time.
    Facing,
    /// Every spread in one vertical scroll.
    FacingContinuous,
}

impl PageDisplay {
    /// Every variant, in the order View ▸ Page display offers them.
    ///
    /// The order is the ribbon's and it is not arbitrary: it runs from fewest
    /// pages on screen to most, so the group reads as a scale rather than as a
    /// list. Exhaustive by construction — [`tests::all_lists_every_variant`]
    /// fails if a variant is added and not listed, which is what makes the
    /// round-trip and command-id tests below complete rather than merely
    /// passing.
    pub const ALL: &'static [Self] = &[
        Self::Single,
        Self::Continuous,
        Self::Facing,
        Self::FacingContinuous,
    ];

    /// Whether this mode scrolls through the whole document rather than
    /// showing one page (or one spread) at a time.
    ///
    /// The single predicate the rest of the build asks. It is what decides
    /// whether the strip holds every page or only the current row, whether the
    /// current page is derived from the scroll offset or set by navigation,
    /// and whether more than one page can need a raster at once.
    #[must_use]
    pub fn is_continuous(self) -> bool {
        matches!(self, Self::Continuous | Self::FacingContinuous)
    }

    /// Whether this mode pairs pages into spreads.
    #[must_use]
    pub fn is_facing(self) -> bool {
        matches!(self, Self::Facing | Self::FacingContinuous)
    }

    /// The stable id this mode is written to disk as.
    ///
    /// Lowercase, hyphenated, and **never** the enum's `Debug` spelling: a
    /// `Debug` impl is a developer convenience that a `derive` may change,
    /// and a persistence format that changed with it would silently reset
    /// every operator's remembered choice. See
    /// [`crate::viewer::remembered`] for the file these ids appear in.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            // ui-text-exempt: on-disk identifiers, never displayed as copy
            Self::Single => "single",
            // ui-text-exempt: on-disk identifiers, never displayed as copy
            Self::Continuous => "continuous",
            // ui-text-exempt: on-disk identifiers, never displayed as copy
            Self::Facing => "facing",
            // ui-text-exempt: on-disk identifiers, never displayed as copy
            Self::FacingContinuous => "facing-continuous",
        }
    }

    /// The mode `id` names, or `None` if nothing does.
    ///
    /// `None` rather than a default, deliberately: the caller is reading a
    /// file that may have been written by a newer build or edited by hand, and
    /// *"this line names a mode I do not have"* is a different fact from
    /// *"this document has no remembered mode"*. The store treats the first as
    /// a line to drop and the second as a document to give the mode default
    /// to; collapsing them would make an unrecognised entry look like a
    /// deliberate choice of `Single`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|m| m.id() == id)
    }

    /// What a fresh profile gets in the ribbon mode named `mode`.
    ///
    /// **The one place `MODES_AND_PANELS.md`'s per-mode rule is written
    /// down.** Read is continuous; everything else — including an id this
    /// build does not know — is single page. See the module header for the
    /// operator decision behind it and for why an unknown id falls back to
    /// `Single` rather than refusing.
    #[must_use]
    pub fn default_for_mode(mode: &str) -> Self {
        if mode == "read" {
            Self::Continuous
        } else {
            Self::Single
        }
    }

    // -----------------------------------------------------------------
    // The spread rule
    // -----------------------------------------------------------------

    /// Which **row** of the strip `page` belongs to.
    ///
    /// A row is one page under [`Self::Single`] / [`Self::Continuous`] and one
    /// *spread* under the facing modes, so this is the single owner of the
    /// spread-pairing rule and the cover-page exception that goes with it:
    ///
    /// ```text
    ///   row 0 : page 0                (the cover, alone)
    ///   row 1 : pages 1 and 2
    ///   row 2 : pages 3 and 4
    ///   row r : pages 2r-1 and 2r
    /// ```
    ///
    /// # Why the cover is alone, and why that is not configurable here
    ///
    /// Because a document is bound, and the first sheet of a bound document
    /// faces outward with nothing to its left. Pairing 0 with 1 puts every
    /// subsequent spread on the wrong parity — page 3 on the right when it
    /// should be on the left — and on a drawing set with a title sheet that is
    /// visibly wrong rather than merely unconventional. Every reader in the
    /// class does this.
    ///
    /// It is not configurable because there is no control for it: a
    /// `view.facing_cover` toggle would be a fifth command, and
    /// `PROJECT_PLAN.md`'s no-placeholders rule says a rule with no surface is
    /// a rule with one answer. This is where a control would attach.
    #[must_use]
    pub fn row_of(self, page: usize) -> usize {
        if self.is_facing() {
            // page 0 -> 0; 1,2 -> 1; 3,4 -> 2; …
            page.div_ceil(2)
        } else {
            page
        }
    }

    /// The pages in row `row`, clamped to a document of `page_count` pages.
    ///
    /// Returns an empty range for a row past the end, which is what lets a
    /// caller walk rows without first computing how many there are.
    #[must_use]
    pub fn pages_in_row(self, row: usize, page_count: usize) -> std::ops::Range<usize> {
        if !self.is_facing() {
            let start = row.min(page_count);
            return start..(start + 1).min(page_count);
        }
        if row == 0 {
            return 0..1usize.min(page_count);
        }
        // Rows 1.. hold `2r-1` and `2r`. `saturating_*` cannot underflow here
        // (row >= 1) but is written defensively because the arithmetic is the
        // spread rule and a panic in it would be a crash on a page step.
        let start = row.saturating_mul(2).saturating_sub(1);
        let start = start.min(page_count);
        let end = start.saturating_add(2).min(page_count);
        start..end
    }

    /// How many rows a document of `page_count` pages has, in this mode.
    ///
    /// `Single` and `Facing` still report the document's full row count even
    /// though they show one row at a time: the number is what a *strip* would
    /// hold, and the two non-scrolling modes are the same strip with one row
    /// selected. Keeping one definition means a page step and a scroll step
    /// cannot disagree about how many rows there are.
    #[must_use]
    pub fn row_count(self, page_count: usize) -> usize {
        if page_count == 0 {
            return 0;
        }
        self.row_of(page_count - 1) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`PageDisplay::ALL`] really is every variant.
    ///
    /// Everything below iterates `ALL`, so a variant missing from it would
    /// make those tests vacuously pass about the variant that matters. The
    /// exhaustive `match` is what makes this fail to *compile* when a variant
    /// is added, which is stronger than failing to run.
    #[test]
    fn all_lists_every_variant() {
        for mode in PageDisplay::ALL {
            // An exhaustive match: adding a variant breaks the build here, and
            // the fix is to add it to `ALL` as well.
            match mode {
                PageDisplay::Single
                | PageDisplay::Continuous
                | PageDisplay::Facing
                | PageDisplay::FacingContinuous => {}
            }
        }
        assert_eq!(PageDisplay::ALL.len(), 4);
    }

    /// ★ **Every mode round-trips through its on-disk spelling.**
    ///
    /// The persistence format's whole correctness. A variant with no `id` arm
    /// would not compile; a variant whose `id` collides with another's would
    /// fail here, and the symptom in the field would be an operator's
    /// remembered choice quietly becoming a different mode on the next launch.
    #[test]
    fn every_mode_round_trips_through_its_on_disk_spelling() {
        for &mode in PageDisplay::ALL {
            assert_eq!(PageDisplay::from_id(mode.id()), Some(mode));
        }
        // …and the ids are distinct, which the round trip alone does not prove
        // if two variants shared one spelling.
        let mut ids: Vec<&str> = PageDisplay::ALL.iter().map(|m| m.id()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PageDisplay::ALL.len());
    }

    /// An id nothing declares is declined rather than defaulted.
    #[test]
    fn an_unknown_id_is_declined() {
        assert_eq!(PageDisplay::from_id("spiral"), None);
        assert_eq!(PageDisplay::from_id(""), None);
        assert_eq!(PageDisplay::from_id("Single"), None, "ids are lowercase");
    }

    /// ★ **Read defaults to continuous; every other mode keeps single page.**
    ///
    /// The operator decision of 2026-08-13, asserted rather than commented.
    /// An unknown mode falls back to the default rather than refusing — a
    /// customized manifest may declare a fourth mode, and it must open a
    /// document rather than fail to.
    #[test]
    fn only_read_defaults_to_continuous() {
        assert_eq!(
            PageDisplay::default_for_mode("read"),
            PageDisplay::Continuous
        );
        assert_eq!(PageDisplay::default_for_mode("review"), PageDisplay::Single);
        assert_eq!(PageDisplay::default_for_mode("edit"), PageDisplay::Single);
        assert_eq!(
            PageDisplay::default_for_mode("proofread"),
            PageDisplay::Single,
            "an unknown mode gets the default, not a refusal"
        );
        assert_eq!(PageDisplay::default(), PageDisplay::Single);
    }

    /// The two predicates partition the four modes the way the rest of the
    /// build assumes.
    #[test]
    fn the_two_predicates_describe_the_four_modes() {
        assert!(!PageDisplay::Single.is_continuous() && !PageDisplay::Single.is_facing());
        assert!(PageDisplay::Continuous.is_continuous() && !PageDisplay::Continuous.is_facing());
        assert!(!PageDisplay::Facing.is_continuous() && PageDisplay::Facing.is_facing());
        assert!(
            PageDisplay::FacingContinuous.is_continuous()
                && PageDisplay::FacingContinuous.is_facing()
        );
    }

    /// ★ **The cover page is alone, and every later spread is odd-then-even.**
    ///
    /// The spread rule, stated as the mapping a reader can check by eye
    /// against a physical document. Getting the parity backwards puts page 3
    /// on the right of a spread it should open, which on a drawing set with a
    /// title sheet is visibly wrong.
    #[test]
    fn facing_pairs_pages_after_a_solitary_cover() {
        let f = PageDisplay::Facing;
        assert_eq!(f.pages_in_row(0, 10), 0..1, "the cover is alone");
        assert_eq!(f.pages_in_row(1, 10), 1..3);
        assert_eq!(f.pages_in_row(2, 10), 3..5);
        assert_eq!(f.pages_in_row(3, 10), 5..7);
        for page in 0..10 {
            let row = f.row_of(page);
            assert!(
                f.pages_in_row(row, 10).contains(&page),
                "page {page} claims row {row}, which does not contain it"
            );
        }
    }

    /// A last spread with only one page left is a one-page row, not a panic
    /// and not a phantom page.
    #[test]
    fn an_odd_page_count_ends_with_a_half_spread() {
        let f = PageDisplay::Facing;
        // 4 pages: rows are {0}, {1,2}, {3}.
        assert_eq!(f.row_count(4), 3);
        assert_eq!(f.pages_in_row(2, 4), 3..4);
        assert_eq!(f.pages_in_row(3, 4), 4..4, "past the end is empty");
        // 1 page: one row holding the cover.
        assert_eq!(f.row_count(1), 1);
        assert_eq!(f.pages_in_row(0, 1), 0..1);
        // No pages at all: no rows, and asking for one is empty rather than a
        // panic — a `/Count 0` document is legal.
        assert_eq!(f.row_count(0), 0);
        assert_eq!(f.pages_in_row(0, 0), 0..0);
    }

    /// A non-facing mode has one page per row, which is what makes the strip
    /// arithmetic uniform across all four modes.
    #[test]
    fn a_non_facing_row_is_exactly_one_page() {
        for &mode in &[PageDisplay::Single, PageDisplay::Continuous] {
            assert_eq!(mode.row_count(7), 7);
            for page in 0..7 {
                assert_eq!(mode.row_of(page), page);
                assert_eq!(mode.pages_in_row(page, 7), page..page + 1);
            }
            assert_eq!(mode.pages_in_row(7, 7), 7..7);
            assert_eq!(mode.row_count(0), 0);
        }
    }
}
