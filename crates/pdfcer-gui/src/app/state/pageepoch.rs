//! # `app::state::pageepoch` — **which PAGE changed, not just that something
//! did**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/state/pageepoch.md`.

/// **A revision number per page, plus a document-wide floor.**
///
/// Read [`Self::get`] first — it is the whole contract, and the `max` in it is
/// what makes every other method safe to call in any order.
#[derive(Debug, Clone, Default)]
pub struct PageEpochs {
    /// One counter per page, indexed by page position.
    ///
    /// May be **shorter** than the document — a page inserted between a
    /// `resize` and the next frame is out of range, and [`Self::get`] answers
    /// such a page with `all` rather than with a fresh-looking zero.
    per_page: Vec<u64>,
    /// The floor every page's answer is raised to.
    ///
    /// Raised by every edit that has not proved itself single-page, which is
    /// most of them and is the point. See this module's header, §"Why the
    /// default is `bump_all`".
    all: u64,
    /// **One monotonic issuer for BOTH kinds of bump.** It is what makes
    /// `bump_all` mean what it says, and it must not be split in two.
    ///
    /// *Two counters compared with `max` do not compose.* Increment `all` and
    /// `per_page[page]` independently and `get` = `max(all, per_page[page])` is
    /// not enough: narrow page 2 (`per_page[2]` = 1, `all` = 0), then raise
    /// everything (`all` = 1), and page 2 answers `max(1, 1)` = 1 — *the number
    /// it already had*. A document-wide bump would then skip exactly the pages
    /// most recently edited individually, leaving a cache holding a picture of
    /// content the operator had already changed on precisely the page he was
    /// working on.
    ///
    /// ⇒ Drawing both bumps from one counter makes every number strictly larger
    /// than every number before it, so any bump that reaches a page changes that
    /// page's answer. The invariant is a property of the issuer rather than of
    /// the arithmetic in `get`, which is why `get` can stay a plain `max`.
    next: u64,
}

impl PageEpochs {
    /// **The revision of one page**, and the only reader anything outside this
    /// module should need.
    ///
    /// `max(all, per_page[page])`, which is the invariant the whole design
    /// rests on: a document-wide bump raises every page at once and **cannot
    /// be undercut** by a per-page counter that happens to be lower, whatever
    /// order the two were written in. A cache comparing this number against the
    /// one its entry was built at is therefore correct without knowing anything
    /// about which verbs are narrowed and which are not.
    ///
    /// A page index past the end answers `all`. That is not a bounds-check
    /// convenience — it is the conservative answer, chosen because the
    /// alternative (`0`) would look *older* than any real page and would keep a
    /// cache entry the caller has no evidence for.
    #[must_use]
    pub fn get(&self, page: usize) -> u64 {
        self.per_page
            .get(page)
            .copied()
            .map_or(self.all, |p| p.max(self.all))
    }

    /// **Everything changed** — or, much more often, *this verb has not proved
    /// that everything did not.*
    ///
    /// The default for every edit. See the header for the list of verbs that
    /// must keep calling this and why each one is dangerous to narrow.
    pub fn bump_all(&mut self) {
        self.next = self.next.wrapping_add(1);
        self.all = self.next;
    }

    /// **Exactly one page changed**, and the caller has established it.
    ///
    /// Only call this where the confinement is a *property of the verb*, not
    /// an observation about the last time it ran. The two narrowings that ship
    /// today are named at their call sites, each with its own argument.
    ///
    /// Grows `per_page` if the page is past the end, so a caller never has to
    /// sequence this against [`Self::resize`]. The pages it grows through
    /// start at the current `all`, so a page that has never been named
    /// individually reports the document-wide floor rather than zero — the
    /// same conservative choice [`Self::get`] makes, made once here so the two
    /// cannot disagree.
    ///
    /// The number it issues comes from [`Self::next`], shared with
    /// [`Self::bump_all`] — see that field for why a second counter here would
    /// let a document-wide bump fail to move a recently narrowed page.
    pub fn bump(&mut self, page: usize) {
        if self.per_page.len() <= page {
            self.per_page.resize(page + 1, self.all);
        }
        self.next = self.next.wrapping_add(1);
        self.per_page[page] = self.next;
    }

    /// Track a change in the number of pages.
    ///
    /// Called from `actions::pages::resync`, which is the one place that knows
    /// the page vector was replaced. **Growth fills with `all`**, so a newly
    /// inserted page reports the document-wide floor and no cache mistakes it
    /// for a page it has a picture of.
    ///
    /// It does **not** bump anything. A page count changing is not by itself
    /// an edit to any page's content, and the caller that knows the pages were
    /// *renumbered* raises `bump_all` separately — two facts, two calls, so
    /// neither is inferred from the other. A document that gains a page at the
    /// end has not changed page 0, and a rail that redrew page 0 for it would
    /// be this module's own defect.
    pub fn resize(&mut self, page_count: usize) {
        self.per_page.resize(page_count, self.all);
    }
}

#[cfg(test)]
mod tests {
    use super::PageEpochs;

    /// A fresh set answers the same number for every page, including pages it
    /// has never heard of.
    #[test]
    fn a_fresh_set_is_uniform() {
        let e = PageEpochs::default();
        assert_eq!(e.get(0), e.get(5));
        assert_eq!(e.get(0), e.get(99_999));
    }

    /// Bumping one page moves that page and no other.
    #[test]
    fn one_page_moves_alone() {
        let mut e = PageEpochs::default();
        e.resize(4);
        let before: Vec<u64> = (0..4).map(|p| e.get(p)).collect();
        e.bump(2);
        assert_eq!(e.get(0), before[0]);
        assert_eq!(e.get(1), before[1]);
        assert_ne!(e.get(2), before[2], "the named page must move");
        assert_eq!(e.get(3), before[3]);
    }

    /// The invariant the design rests on: a document-wide bump raises
    /// every page, **whatever order the two kinds of bump arrived in**.
    #[test]
    fn a_document_wide_bump_cannot_be_undercut() {
        for narrow_first in [true, false] {
            let mut e = PageEpochs::default();
            e.resize(3);
            let before = e.get(1);
            if narrow_first {
                e.bump(1);
                e.bump_all();
            } else {
                e.bump_all();
                e.bump(1);
            }
            assert_ne!(e.get(0), before, "page 0 must move on a document-wide bump");
            assert_ne!(e.get(1), before, "page 1 must move whichever came first");
        }
    }

    /// Repeated document-wide bumps keep moving every page, so a cache that
    /// missed one edit does not accidentally match on the next.
    #[test]
    fn every_document_wide_bump_is_a_new_number() {
        let mut e = PageEpochs::default();
        e.resize(2);
        let mut seen = vec![e.get(0)];
        for _ in 0..5 {
            e.bump_all();
            let now = e.get(0);
            assert!(
                !seen.contains(&now),
                "epoch {now} repeated within five bumps"
            );
            seen.push(now);
        }
    }

    /// A page added by [`PageEpochs::resize`] reports the document-wide floor,
    /// not zero — so nothing mistakes a brand-new page for one it has a
    /// picture of.
    #[test]
    fn a_new_page_starts_at_the_floor_not_at_zero() {
        let mut e = PageEpochs::default();
        e.resize(1);
        e.bump_all();
        e.bump_all();
        let floor = e.get(0);
        e.resize(3);
        assert_eq!(e.get(2), floor, "a page added late must not look older");
    }

    /// Bumping a page past the end grows rather than panicking, and the pages
    /// it grows through keep the floor.
    #[test]
    fn bumping_past_the_end_grows_without_disturbing_neighbours() {
        let mut e = PageEpochs::default();
        e.resize(1);
        e.bump_all();
        let floor = e.get(0);
        e.bump(3);
        assert_eq!(e.get(0), floor);
        assert_eq!(e.get(2), floor, "a page skipped over must not move");
        assert_ne!(e.get(3), floor);
    }

    /// **A document-wide bump moves a page that was JUST narrowed.**
    #[test]
    fn a_narrowed_page_is_still_moved_by_a_document_wide_bump() {
        let mut e = PageEpochs::default();
        e.resize(3);
        e.bump(1);
        let narrowed = e.get(1);
        e.bump_all();
        assert_ne!(
            e.get(1),
            narrowed,
            "a document-wide bump must move a page that was narrowed a moment ago"
        );
        // …and it must not have been achieved by making the OTHER pages stale
        // in some different way: every page moves, and they all agree.
        assert_eq!(e.get(0), e.get(1));
        assert_eq!(e.get(1), e.get(2));
    }

    /// Shrinking and re-growing must not resurrect a page's old number.
    #[test]
    fn a_page_that_leaves_and_returns_does_not_bring_its_old_number_back() {
        let mut e = PageEpochs::default();
        e.resize(3);
        e.bump(2);
        let narrowed = e.get(2);
        e.resize(2);
        e.bump_all();
        e.resize(3);
        assert_ne!(
            e.get(2),
            narrowed,
            "the returned page must not look unchanged"
        );
    }
}
