//! Tests for [`super`] — the page-thumbnail cache and its policy.
//!
//! Split out on 2026-09-08 for R2 (no source file over 1,500 lines) when the
//! O151 rewrite pushed `thumbnails.rs` past the limit. **Nothing else moved**:
//! this is the same `mod tests` block, de-indented, with its parent's private
//! fields still reachable because a child module can see them.
//!
//! ★ One assertion changed as a consequence and it is the interesting one.
//! `only_the_operator_may_untick_previews` scans the parent's source through
//! `include_str!`, and it used to cut the file at `#[cfg(test)]` to avoid
//! reading its own assertion strings. With the harness in a *different file*
//! the cut is unnecessary and, worse, would be a silent no-op — so it is gone,
//! and the test asserts instead that the marker it looks for is absent from
//! this file. That assertion is what keeps the scan honest now.

#![cfg(test)]

use super::*;

/// A cache with `ready` pages recorded, for the scheduling tests.
///
/// Textures need an `egui::Context` and a live renderer; the *policy*
/// does not, and the policy is what these tests are about. So the state
/// is set through `unavailable`, which produces the same "not pending"
/// answer from [`ThumbnailCache::state`] by a route a headless test can
/// take.
fn settled(pages: &[usize]) -> ThumbnailCache {
    let mut cache = ThumbnailCache::default();
    for p in pages {
        cache
            .unavailable
            .insert(*p, Unavailable::Failed(String::new()));
    }
    cache
}

/// **★ The current page is drawn first when it is on screen.**
///
/// It carries the highlight ring, so it is the tile the operator is using
/// to answer "where am I" — and a ring around a tile reading "not drawn
/// yet" answers that with the page number they already had.
#[test]
fn the_current_page_is_drawn_before_its_neighbours() {
    let cache = ThumbnailCache::default();
    let visible = [4, 5, 6, 7, 8];
    assert_eq!(cache.next_to_render(&visible, 6), Some(6));
    // …and when the current page is NOT on screen, reading order wins.
    assert_eq!(cache.next_to_render(&visible, 40), Some(4));
}

/// Nothing off-screen is ever drawn.
///
/// The property that makes a 900-page document affordable at all: the
/// grid's cost is bounded by what fits on screen, not by the document.
#[test]
fn only_visible_pages_are_candidates() {
    let cache = ThumbnailCache::default();
    assert_eq!(cache.next_to_render(&[100, 101], 0), Some(100));
    assert_eq!(
        cache.next_to_render(&[], 0),
        None,
        "an empty viewport must schedule nothing at all"
    );
}

/// A settled viewport schedules nothing — the steady state, and the
/// reason this is cheap to call sixty times a second.
#[test]
fn a_fully_drawn_viewport_asks_for_nothing() {
    let cache = settled(&[4, 5, 6]);
    assert_eq!(cache.next_to_render(&[4, 5, 6], 5), None);
}

/// **★★★ A SKIPPED PAGE DOES NOT STOP THE GRID — O151, the operator:**
///
/// > *"the drawing page previews checkbox should never automatically turn
/// > off."*
///
/// This is the regression test for the whole change, and it is written
/// against the two things that were true before it and must never be true
/// again: the tick going out on its own, and every other page losing its
/// picture because one page was expensive.
///
/// ⚠ Note what it does NOT assert — that page 2 gets a picture. It does
/// not; the budget abandoned it, and `Unavailable::Abandoned` is the
/// honest record of that. What it asserts is that page 2's misfortune is
/// **page 2's alone**.
#[test]
fn a_skipped_page_leaves_the_feature_on_and_its_neighbours_drawn() {
    let mut cache = ThumbnailCache {
        skipped: Some(SkippedPage {
            page_index: 2,
            millis: 2000,
        }),
        ..ThumbnailCache::default()
    };
    cache.unavailable.insert(2, Unavailable::Abandoned);

    assert!(
        cache.previews_on(),
        "an abandoned page unticked the operator's checkbox — the O151 defect"
    );
    assert_eq!(
        cache.state(2),
        TileState::Abandoned,
        "the page that ran over must say so on its own tile"
    );
    assert_eq!(
        cache.state(0),
        TileState::NotDrawnYet,
        "a neighbour of a skipped page must still be queued for a picture"
    );
    assert_eq!(
        cache.next_to_render(&[0, 1, 2, 3], 1),
        Some(1),
        "the grid must carry on past the page it gave up on"
    );
}

/// **★★ Raising the limit gives the skipped page another go.**
///
/// The trap this avoids: a dial that can only ever remove pictures. The
/// operator's sole reason for touching the box is the tile reading "Not
/// finished", so if raising the number left that tile exactly as it was,
/// they would reasonably conclude the box does nothing.
///
/// It also asserts the half that must NOT happen — a page the renderer
/// *refused* is not retried, because it will be refused again and a
/// `DragValue` reports a change on every pixel of a drag.
#[test]
fn raising_the_limit_retries_what_it_skipped_and_nothing_else() {
    let mut cache = ThumbnailCache::default();
    cache.unavailable.insert(2, Unavailable::Abandoned);
    cache
        .unavailable
        .insert(3, Unavailable::Failed("no".to_owned()));
    cache.skipped = Some(SkippedPage {
        page_index: 2,
        millis: 2000,
    });

    cache.set_budget(Duration::from_secs(8));

    assert_eq!(cache.budget(), Duration::from_secs(8));
    assert_eq!(
        cache.state(2),
        TileState::NotDrawnYet,
        "the abandoned page was not queued again, so the box appears to do nothing"
    );
    assert_eq!(
        cache.state(3),
        TileState::Failed,
        "a page the renderer refused must not be retried on every keystroke"
    );
    assert_eq!(
        cache.skipped(),
        None,
        "the note still quotes a limit that is no longer in force"
    );
}

/// **★ The limit is clamped on the VALUE, not only on the control.**
///
/// A control narrower than what the value may legally hold silently
/// rewrites it — the argument `canvas::markup::swatch` makes for taking
/// the pen's own range. Here the risk runs the other way: a future caller
/// that is not the `DragValue` could set a budget of zero, which is a
/// build where every page is abandoned while the checkbox reads "on".
#[test]
fn the_limit_cannot_be_set_outside_what_is_useful() {
    let mut cache = ThumbnailCache::default();
    cache.set_budget(Duration::ZERO);
    assert_eq!(cache.budget(), MIN_PAGE_BUDGET);
    cache.set_budget(Duration::from_secs(60 * 60));
    assert_eq!(cache.budget(), MAX_PAGE_BUDGET);
}

/// **★ Setting the same limit twice is free.**
///
/// `DragValue` reports a change on every pixel of a drag. If `set_budget`
/// dropped the abandoned entries unconditionally, a drag across the box
/// would re-queue — and therefore re-render — the expensive page dozens of
/// times, on the UI thread, while the operator was still choosing a number.
#[test]
fn setting_the_same_limit_again_changes_nothing() {
    let mut cache = ThumbnailCache::default();
    cache.unavailable.insert(2, Unavailable::Abandoned);
    cache.skipped = Some(SkippedPage {
        page_index: 2,
        millis: 2000,
    });
    cache.set_budget(PAGE_BUDGET_DEFAULT);
    assert_eq!(cache.state(2), TileState::Abandoned);
    assert_eq!(cache.skipped().map(|s| s.page_index), Some(2));
}

/// **★ Turning previews off by hand stops the grid, and stops explaining.**
///
/// The tick is now the only thing that stops the grid, and when it is
/// clear the skip note must go quiet: every tile is blank for a reason the
/// operator already knows, and a sentence about page 3's time limit beside
/// eleven other blank tiles points at the wrong cause.
#[test]
fn turning_previews_off_by_hand_stops_explaining_a_skip() {
    let mut cache = ThumbnailCache {
        skipped: Some(SkippedPage {
            page_index: 2,
            millis: 2000,
        }),
        ..ThumbnailCache::default()
    };
    assert_eq!(cache.skipped().map(|s| s.page_index), Some(2));

    cache.force_on(false);
    assert!(!cache.previews_on());
    assert_eq!(cache.next_to_render(&[0, 1, 2, 3], 1), None);
    assert_eq!(cache.state(3), TileState::PreviewsOff);
    assert_eq!(cache.skipped(), None);

    // …and it comes back with the tick, because the page is still skipped.
    cache.force_on(true);
    assert_eq!(cache.skipped().map(|s| s.page_index), Some(2));
}

/// **★★★ Nothing in this module writes the operator's tick.**
///
/// The tripwire for O151 rather than a test of behaviour, and it is
/// written as a source scan because the defect it guards against is a
/// *future* line of code, not a current output. Two green tests sat beside
/// the old rule and neither could have caught it: they asserted that the
/// automatic stop worked, which it did.
///
/// `force_on` is the one sanctioned writer and it is called from exactly
/// one place — the checkbox. Any other assignment to `self.on` is pdfcer
/// deciding on the operator's behalf again.
///
/// # ⚠ It failed on its first run by reading its own assertion, and the
/// fix is why this file exists where it does
///
/// The first draft lived *inside* `thumbnails.rs` and correctly reported
/// **itself** as a writer: the expected value `"self.on = on;"` and the
/// filter literal `"self.on = "` are both lines containing the marker. That
/// is the `include_str!` trap this project has hit before — a check that
/// reads its own expected string and thereafter passes on a file which has
/// never contained the subject.
///
/// The R2 split solved it structurally: the harness is a different file now,
/// so `include_str!` cannot reach these strings at all. That is a stronger
/// guarantee than the string-cutting it replaced, and it is **asserted rather
/// than assumed** — a scan over a file that no longer contains the subject
/// finds no violations and reports success, which is exactly what "the gate
/// stopped running and nobody noticed" looks like from the outside.
///
/// ⇒ If this test is ever moved back into `thumbnails.rs`, the first
/// assertion below fires immediately and says why.
#[test]
fn only_the_operator_may_untick_previews() {
    const MARKER: &str = "self.on = ";
    let source = include_str!("../thumbnails.rs");
    assert!(
        !source.contains("only_the_operator_may_untick_previews"),
        "this test moved back into the file it scans, so it is now reading its \
         own assertion strings and can no longer fail"
    );
    assert!(
        source.len() > 20_000,
        "the scan read almost nothing — check the `include_str!` path"
    );

    let writers: Vec<&str> = source
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//"))
        .filter(|l| l.contains(MARKER))
        .collect();
    assert_eq!(
        writers,
        vec!["self.on = on;"],
        "something other than `force_on` writes the operator's checkbox"
    );
}

/// **★ Every not-ready state has its own tile word.**
///
/// The no-placeholders rule for pictures. Four distinct states must map
/// to four distinct sentences, or the tile is guessing on the operator's
/// behalf. Asserted against the catalog itself, so a future edit that
/// makes two of them read alike fails here.
#[test]
fn the_four_undrawn_states_say_four_different_things() {
    use crate::text::pages as t;
    let words = [
        t::thumbnail_not_drawn_yet(),
        t::thumbnail_previews_off(),
        t::thumbnail_abandoned(),
        t::thumbnail_failed(),
    ];
    for (i, a) in words.iter().enumerate() {
        assert!(!a.trim().is_empty(), "an undrawn tile must say something");
        for b in &words[i + 1..] {
            assert_ne!(a, b, "two different states read identically");
        }
    }
}

/// A failure and an abandonment are different tiles, and neither is
/// retried.
#[test]
fn a_recorded_outcome_is_not_scheduled_again() {
    let mut cache = ThumbnailCache::default();
    cache
        .unavailable
        .insert(3, Unavailable::Failed("bad stream".to_owned()));
    cache.unavailable.insert(4, Unavailable::Abandoned);
    assert_eq!(cache.state(3), TileState::Failed);
    assert_eq!(cache.state(4), TileState::Abandoned);
    assert_eq!(
        cache.next_to_render(&[3, 4], 3),
        None,
        "a deterministic failure retried every frame pegs a core"
    );
}

/// **★ Eviction keeps the neighbourhood the operator is in.**
///
/// The property LRU gets wrong: scrolling down and back must not
/// re-render the whole way home.
#[test]
fn the_furthest_page_from_the_viewport_is_evicted() {
    // Oldest first. The operator is looking at page 50.
    let order = [1, 48, 49, 51, 52, 200];
    assert_eq!(evict_victim(&order, 50, 53), Some(200));
    // Move the viewport to the front of the document and the far end of
    // the cache changes with it — which is the whole difference from LRU.
    assert_eq!(evict_victim(&order, 1, 2), Some(200));
    assert_eq!(evict_victim(&order, 200, 199), Some(1));
}

/// Ties break toward the older entry rather than at random.
#[test]
fn an_equidistant_pair_evicts_the_older_one() {
    // 40 and 60 are both 10 away from 50; 40 was cached first.
    assert_eq!(evict_victim(&[40, 60], 50, 55), Some(40));
    assert_eq!(evict_victim(&[60, 40], 50, 55), Some(60));
}

/// The page about to be inserted is never the victim, and an empty cache
/// has no victim at all.
#[test]
fn eviction_never_chooses_the_incoming_page_or_an_empty_cache() {
    assert_eq!(evict_victim(&[], 0, 0), None);
    assert_eq!(
        evict_victim(&[900], 0, 900),
        None,
        "evicting the page being inserted is a cache that is always full \
         and always empty"
    );
}

/// **★ A page change must not drop a single picture.**
///
/// The invalidation key is the edit epoch, not the page index. Keying on
/// the page would re-rasterize the visible grid on every Page Down — a
/// second of frozen UI per keystroke, to redraw pictures that were
/// already right.
#[test]
fn navigating_keeps_the_cache_and_editing_drops_it() {
    use crate::app::state::pageepoch::PageEpochs;

    let mut epochs = PageEpochs::default();
    epochs.resize(8);
    let mut cache = ThumbnailCache::default();
    cache.sync(&epochs, 2.0);
    cache.unavailable.insert(7, Unavailable::Abandoned);
    cache.built_at.insert(7, epochs.get(7));

    cache.sync(&epochs, 2.0);
    assert_eq!(cache.state(7), TileState::Abandoned, "nothing changed");

    epochs.bump_all();
    cache.sync(&epochs, 2.0);
    assert_eq!(
        cache.state(7),
        TileState::NotDrawnYet,
        "an edit changes what the pages look like"
    );

    // A density change invalidates for a different reason: every texture
    // is now the wrong resolution.
    cache.unavailable.insert(7, Unavailable::Abandoned);
    cache.built_at.insert(7, epochs.get(7));
    cache.sync(&epochs, 1.5);
    assert_eq!(cache.state(7), TileState::NotDrawnYet);
}

/// ★★★ **The O74 assertion, and the one that would have caught the
/// original defect**: an edit on one page leaves every other page's
/// picture alone.
///
/// `OPERATOR_REQUESTS.md` O74 — *"all of the page previews get re-rendered
/// instead of just the one that is being changed"*. The old `sync` keyed
/// the whole cache on a document-wide epoch and cleared it wholesale, so
/// this test could not have been written against it: there was no per-page
/// input to vary.
#[test]
fn an_edit_on_one_page_leaves_the_other_pages_pictures_alone() {
    use crate::app::state::pageepoch::PageEpochs;

    let mut epochs = PageEpochs::default();
    epochs.resize(4);
    let mut cache = ThumbnailCache::default();
    cache.sync(&epochs, 2.0);
    for page in 0..4 {
        cache.unavailable.insert(page, Unavailable::Abandoned);
        cache.built_at.insert(page, epochs.get(page));
    }

    epochs.bump(2);
    cache.sync(&epochs, 2.0);

    assert_eq!(cache.state(2), TileState::NotDrawnYet, "the edited page");
    for page in [0, 1, 3] {
        assert_eq!(
            cache.state(page),
            TileState::Abandoned,
            "page {page} was not edited and must keep its entry"
        );
    }
}

/// ★★ …and the safety half, which matters more: a **document-wide** bump
/// still drops everything.
///
/// Without this, the test above passes on a build that never invalidates
/// anything — which would show the operator pictures of content he had
/// already changed. That is rule 4's "sneaky" and it outranks the slowness
/// the per-page key exists to fix, so both directions are asserted.
#[test]
fn a_document_wide_edit_still_drops_every_picture() {
    use crate::app::state::pageepoch::PageEpochs;

    let mut epochs = PageEpochs::default();
    epochs.resize(4);
    let mut cache = ThumbnailCache::default();
    cache.sync(&epochs, 2.0);
    for page in 0..4 {
        cache.unavailable.insert(page, Unavailable::Abandoned);
        cache.built_at.insert(page, epochs.get(page));
    }

    epochs.bump_all();
    cache.sync(&epochs, 2.0);

    for page in 0..4 {
        assert_eq!(
            cache.state(page),
            TileState::NotDrawnYet,
            "page {page} must be dropped by a document-wide edit"
        );
    }
}

/// An entry nothing dated is dropped rather than kept.
///
/// Unreachable today — every insertion stamps `built_at` — and asserted
/// because the polarity is the whole safety argument. "Keep what you
/// cannot date" shows the operator stale content; "drop what you cannot
/// date" costs one render.
#[test]
fn an_undated_entry_is_dropped() {
    use crate::app::state::pageepoch::PageEpochs;

    let mut epochs = PageEpochs::default();
    epochs.resize(2);
    let mut cache = ThumbnailCache::default();
    cache.sync(&epochs, 2.0);
    cache.unavailable.insert(1, Unavailable::Abandoned);
    // …and deliberately no `built_at` entry.
    cache.sync(&epochs, 2.0);
    assert_eq!(cache.state(1), TileState::NotDrawnYet);
}

/// The operator's tick, their limit and the skip note all survive an edit.
///
/// Three instructions and one measurement, and none of them is invalidated
/// by a change to a page's content: an edit does not make an expensive
/// document cheap, and an instruction that evaporates on the next edit was
/// not honoured.
#[test]
fn the_operators_settings_survive_an_edit() {
    use crate::app::state::pageepoch::PageEpochs;

    let mut epochs = PageEpochs::default();
    epochs.resize(2);
    let mut cache = ThumbnailCache::default();
    cache.sync(&epochs, 2.0);
    cache.set_budget(Duration::from_secs(5));
    cache.skipped = Some(SkippedPage {
        page_index: 1,
        millis: 5000,
    });

    epochs.bump_all();
    cache.sync(&epochs, 2.0);

    assert!(
        cache.previews_on(),
        "the operator's instruction did not survive an edit"
    );
    assert_eq!(
        cache.budget(),
        Duration::from_secs(5),
        "the operator's time limit did not survive an edit"
    );
    assert_eq!(cache.skipped().map(|s| s.page_index), Some(1));
    assert_eq!(cache.next_to_render(&[0, 1], 0), Some(0));
}

/// **★ The shipped defaults are previews ON at the default limit.**
///
/// Asserted because `#[derive(Default)]` was removed to get them, and a
/// hand-written `Default` that drifts from its own doc comment is the kind
/// of defect only an operator finds — a build that draws nothing and
/// blames a time limit of zero for it.
#[test]
fn the_shipped_defaults_draw_something() {
    let cache = ThumbnailCache::default();
    assert!(cache.previews_on());
    assert_eq!(cache.budget(), PAGE_BUDGET_DEFAULT);
    assert_eq!(cache.skipped(), None);
    assert_eq!(cache.state(0), TileState::NotDrawnYet);
}

/// The scale is a page-relative number, and a degenerate page cannot
/// produce an infinite one.
///
/// An infinite scale reaches `pdfcer-render`'s pixmap guard and comes back
/// as a refusal, so the tile would read "would not draw" for a page whose
/// only fault is a malformed `/CropBox` — blaming the render for a
/// division this function is responsible for.
#[test]
fn a_thumbnail_scale_is_always_finite() {
    use crate::panels::objects::test_support::engine_fixture;
    let path = engine_fixture("pageops/four-pages.pdf");
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    for page in &pages {
        let scale = raster_scale_for(page, 2.0);
        assert!(scale.is_finite() && scale > 0.0, "scale was {scale}");
    }
}
