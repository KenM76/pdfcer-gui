//! # `app::state::pageepoch` — **which PAGE changed, not just that something
//! did**
//!
//! One type, [`PageEpochs`], and one rule that the whole thing exists to
//! enforce: **the safe answer is the default, and precision is opted into per
//! verb.**
//!
//! ## The report this closes
//!
//! `OPERATOR_REQUESTS.md` row **O74**, in the operator's words:
//!
//! > *"When I make edits or even just fill out a form I notice all of the page
//! > previews get re-rendered instead of just the one that is being changed,
//! > and it seems to really slow down clicking a checkbox in a form. The last
//! > thing that should matter is updating the preview, and it should just
//! > update the pages that were actually altered."*
//!
//! He has given the priority rule as well as the bug, and the priority rule is
//! the more valuable half: **a thumbnail is the lowest-priority work in the
//! program and must never sit between a click and its result.**
//!
//! ## What was measured, before anything was built
//!
//! [`OpenDoc::edit_epoch`](crate::app::state::OpenDoc::edit_epoch) is a
//! **document-wide** counter, and three separate caches of **per-page** derived
//! state were using it as their invalidation key:
//!
//! | cache | what it holds | what one edit cost |
//! |---|---|---|
//! | `panels::pages::thumbnails::ThumbnailCache` | a picture per page, in the rail | every tile, re-rendered inline on the UI thread |
//! | `render::strip::StripRasters` | a full-size raster per page, for continuous scroll | every cached page missing on the next frame |
//! | `OpenDoc::page_texture_epoch` | the canvas's own raster | re-rasterised even when the edit was on another sheet |
//!
//! On `SW41177.pdf` — the operator's own 36-sheet SolidWorks set — twelve
//! visible tiles cost **666 ms of UI-thread work after every single edit**,
//! with a 282 ms worst frame. On the benchmark CAD drawing a full-size strip
//! raster is ~950 ms *per page*.
//!
//! `strip.rs`' own header stated the global behaviour as though it were the
//! design — *"the edit bumps the epoch, and every cached page misses on the
//! next frame"* — which is why it was never questioned. It is not a design; it
//! is the coarsest key that was available when it was written.
//!
//! ## ★★★ Why the default is `bump_all` and precision is opt-in
//!
//! Because getting this wrong is **worse than the slowness it fixes.**
//!
//! A thumbnail kept because its page's epoch did not move is a claim that the
//! picture is current. If that claim is ever false, the rail shows the
//! operator content he has already changed — and under rule 4 (*"fuzzy, never
//! sneaky"*) a stale picture presented as a current one is precisely the
//! category of defect that outranks a performance complaint. A slow program is
//! annoying; a program that shows you the wrong drawing is dangerous.
//!
//! So the asymmetry is deliberate and structural, in three places at once:
//!
//! 1. **Both epoch-bumping sites call [`PageEpochs::bump_all`] unless the
//!    caller has *proved* the edit was confined to one page.** A verb that has
//!    not been examined behaves exactly as it did before this module existed.
//! 2. **[`PageEpochs::get`] returns `max(all, per_page[page])`**, so a
//!    `bump_all` can never be undercut by a stale per-page number, in any
//!    ordering, ever.
//! 3. **A page index out of range answers `all`**, not zero — a question about
//!    a page that does not exist is answered with the most conservative number
//!    available rather than with a value that would look fresh.
//!
//! The verbs that must never be narrowed, and the reason each is dangerous:
//!
//! - **Anything that changes the page SET** — insert, delete, reorder, merge,
//!   extract. Page *n* is a different sheet afterwards, so every per-page
//!   number describes the wrong page. `pages::resync` already computes exactly
//!   this fact (`renumbered`) and it is what drives the `bump_all`.
//! - **Undo and redo**, which run any command backwards and cannot say which
//!   page they landed on.
//! - **Anything document-scoped**: font embed/unembed, `/AcroForm` `/DR`
//!   changes, `RegenerateAppearances`, `Flatten`, metadata.
//! - **A form field whose widgets straddle sheets.** A single `/T` may have
//!   widgets on several pages; filling it changes all of them.
//!
//! ## What `edit_epoch` still means, and why it is untouched
//!
//! Everything. `page_objects`, `page_text`, `form_runs`, `saved_epoch` and
//! every rule-4 disclosure slot still key on it and still behave identically.
//! This is a **finer answer laid beside the existing one**, not a replacement:
//! the change is additive, so a reader who does not know this module exists
//! cannot be wrong about anything.
//!
//! ## Why a module and not a field on `OpenDoc`
//!
//! R2. `app/state.rs` is at 1,481 lines against the 1,500 ceiling, and a field
//! documented to this codebase's standard plus the type it needs would not fit.
//! `app/state/` already holds `heldpreview.rs`, `identity.rs` and `fixtures.rs`
//! on the same argument.

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
    /// ★★★ **One monotonic issuer for BOTH kinds of bump**, and it is the
    /// thing that makes `bump_all` mean what it says.
    ///
    /// # The hole this closes, found by this module's own test
    ///
    /// The first version incremented `all` and `per_page[page]` independently.
    /// With `get` = `max(all, per_page[page])` that is **not enough**: narrow
    /// page 2 (`per_page[2]` = 1, `all` = 0), then raise everything (`all` =
    /// 1), and page 2's answer is `max(1, 1)` = 1 — *the number it already
    /// had*. A document-wide bump silently failed to invalidate exactly the
    /// pages that had most recently been edited individually.
    ///
    /// That is the failure this whole module is written to make impossible: a
    /// cache would keep a picture of content the operator had already changed,
    /// which is rule 4's *sneaky*, and it would happen on precisely the page
    /// he had been working on.
    ///
    /// ⇒ **Both bumps draw from one counter**, so every number ever issued is
    /// strictly larger than every number issued before it, and any bump that
    /// reaches a page changes that page's answer. The invariant is now a
    /// property of the issuer rather than of the arithmetic in `get`, which is
    /// why `get` can stay a plain `max`.
    ///
    /// ★ The lesson, worth the sentence: *two counters compared with `max` do
    /// not compose.* `max(a, b)` is only monotonic in the pair if the two
    /// sequences are ordered against each other, and independent counters are
    /// not. The test that caught it (`a_page_that_leaves_and_returns_…`) was
    /// written to check a different property and found this instead.
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
    /// ★ The number it issues comes from [`Self::next`], shared with
    /// [`Self::bump_all`]. See that field: two independent counters compared
    /// with `max` let a document-wide bump fail to move a recently narrowed
    /// page, which is the one outcome this module exists to prevent.
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
    /// ★ It does **not** bump anything. A page count changing is not by itself
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

    /// ★★★ The invariant the design rests on: a document-wide bump raises
    /// every page, **whatever order the two kinds of bump arrived in**.
    ///
    /// This is the assertion that makes it safe to narrow a verb without
    /// auditing every other verb: getting the ordering wrong cannot produce a
    /// page that looks fresh.
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

    /// ★★★ **A document-wide bump moves a page that was JUST narrowed.**
    ///
    /// The regression test for the hole [`PageEpochs::next`] documents, and
    /// the reason both bumps draw from one counter. Under the first version —
    /// two independent counters compared with `max` — this failed: narrowing
    /// page 1 and then raising everything left page 1 answering the number it
    /// already had, so a document-wide invalidation skipped exactly the page
    /// the operator had most recently edited.
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
    ///
    /// The situation is real: delete a page, undo the delete. The page comes
    /// back, and whatever number its slot held before is meaningless — the
    /// caller raises `bump_all` for a renumbering, and this asserts the resize
    /// itself does not hand back a stale value.
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
