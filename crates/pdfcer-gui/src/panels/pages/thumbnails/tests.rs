//! Tests for [`super`] — the page-thumbnail cache and its policy.
//!
//!
//! One assertion changed as a consequence and it is the interesting one.
//! `only_the_operator_may_untick_previews` scans the parent's source through
//! `include_str!`, and it used to cut the file at `#[cfg(test)]` to avoid
//! reading its own assertion strings. With the harness in a *different file*
//! the cut is unnecessary and, worse, would be a silent no-op — so it is gone,
//! and the test asserts instead that the marker it looks for is absent from
//! this file. That assertion is what keeps the scan honest now.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/pages/thumbnails/tests.md`.

#![cfg(test)]

use super::*;

/// A cache with `ready` pages recorded, for the scheduling tests.
fn settled(pages: &[usize]) -> ThumbnailCache {
    let mut cache = ThumbnailCache::default();
    for p in pages {
        cache
            .unavailable
            .insert(*p, Unavailable::Failed(String::new()));
    }
    cache
}

/// **The current page is drawn first when it is on screen.**
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

/// **A SKIPPED PAGE DOES NOT STOP THE GRID — O151, the operator:**
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

/// **Raising the limit gives the skipped page another go.**
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

    cache.set_budget(Some(Duration::from_secs(8)));

    assert_eq!(cache.budget(), Some(Duration::from_secs(8)));
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

/// **The limit is clamped on the VALUE, not only on the control.**
#[test]
fn the_limit_cannot_be_set_outside_what_is_useful() {
    let mut cache = ThumbnailCache::default();
    cache.set_budget(Some(Duration::ZERO));
    assert_eq!(
        cache.budget(),
        Some(MIN_PAGE_BUDGET),
        "a zero DURATION is an instant give-up and is raised to the floor"
    );
    cache.set_budget(Some(Duration::from_secs(60 * 60)));
    assert_eq!(cache.budget(), Some(MAX_PAGE_BUDGET));
}

/// **Zero milliseconds is `None`, and `None` is not a small number.**
#[test]
fn zero_milliseconds_means_no_limit_and_survives_the_clamp() {
    assert_eq!(
        budget_from_millis(0),
        None,
        "0 is the operator's word for never"
    );
    assert_eq!(millis_from_budget(None), 0, "and it must go back out as 0");

    assert_eq!(
        budget_from_millis(1),
        Some(MIN_PAGE_BUDGET),
        "1 ms is an off switch wearing a number, so it is raised to the floor"
    );
    assert_eq!(
        budget_from_millis(u64::MAX),
        Some(MAX_PAGE_BUDGET),
        "a hand-edited absurdity is held at the ceiling, not honoured"
    );

    // An ordinary mid-range value, written here rather than taken from
    // `PAGE_BUDGET_DEFAULT`: the default is *no limit*, and a round-trip
    // test fed the sentinel would assert the `0` case twice and the ordinary
    // case never.
    let ordinary = Duration::from_secs(2);
    let ms = millis_from_budget(Some(ordinary));
    assert_eq!(
        budget_from_millis(ms),
        Some(ordinary),
        "an ordinary value must survive a trip through the file unchanged"
    );

    let mut cache = ThumbnailCache::default();
    cache.set_budget(budget_from_millis(0));
    assert_eq!(
        cache.budget(),
        None,
        "set_budget's own clamp must not resurrect a limit the operator removed"
    );
    assert_eq!(
        millis_from_budget(cache.budget()),
        0,
        "and the write-back must not hand the file a default it never asked for"
    );
}

/// **Setting the same limit twice is free.**
#[test]
fn setting_the_same_limit_again_changes_nothing() {
    // The limit is armed FIRST and the abandoned page planted after it. The
    // shipped default is *no limit*, so planting first and then arming would
    // make the first call a real change — which correctly clears what was
    // planted, and would leave this test asserting the opposite of its name.
    let mut cache = ThumbnailCache::default();
    let limit = Some(Duration::from_secs(2));
    cache.set_budget(limit);
    cache.unavailable.insert(2, Unavailable::Abandoned);
    cache.skipped = Some(SkippedPage {
        page_index: 2,
        millis: 2000,
    });
    cache.set_budget(limit);
    assert_eq!(cache.state(2), TileState::Abandoned);
    assert_eq!(cache.skipped().map(|s| s.page_index), Some(2));
}

/// **Turning previews off by hand stops the grid, and stops explaining.**
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

/// **Nothing in this module writes the operator's tick.**
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

/// **Every not-ready state has its own tile word.**
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

/// **Eviction keeps the neighbourhood the operator is in.**
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

/// **A page change must not drop a single picture.**
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

/// **The O74 assertion, and the one that would have caught the
/// original defect**: an edit on one page leaves every other page's
/// picture alone.
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

/// …and the safety half, which matters more: a **document-wide** bump
/// still drops everything.
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
#[test]
fn the_operators_settings_survive_an_edit() {
    use crate::app::state::pageepoch::PageEpochs;

    let mut epochs = PageEpochs::default();
    epochs.resize(2);
    let mut cache = ThumbnailCache::default();
    cache.sync(&epochs, 2.0);
    cache.set_budget(Some(Duration::from_secs(5)));
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
        Some(Duration::from_secs(5)),
        "the operator's time limit did not survive an edit"
    );
    assert_eq!(cache.skipped().map(|s| s.page_index), Some(1));
    assert_eq!(cache.next_to_render(&[0, 1], 0), Some(0));
}

/// **The shipped defaults are previews ON with no time limit.**
#[test]
fn the_shipped_defaults_draw_something() {
    let cache = ThumbnailCache::default();
    assert!(cache.previews_on());
    assert_eq!(
        cache.budget(),
        None,
        "a fresh build must draw every page, however long one takes"
    );
    assert_eq!(cache.skipped(), None);
    assert_eq!(cache.state(0), TileState::NotDrawnYet);
}

/// The scale is a page-relative number, and a degenerate page cannot
/// produce an infinite one.
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

/// **A draft is redrawn finer only when the tiles are wider than it**, and
/// never again at a width that was refused for this revision.
#[test]
fn a_finer_picture_is_wanted_only_for_wider_tiles() {
    assert!(
        !wants_finer(Grade::Draft, None, None),
        "tiles fit the draft"
    );
    assert!(wants_finer(Grade::Draft, Some(192), None));
    assert!(
        !wants_finer(Grade::Fine(192), Some(192), None),
        "already fine"
    );
    assert!(
        wants_finer(Grade::Fine(192), Some(256), None),
        "zoomed in again"
    );
    assert!(
        !wants_finer(Grade::Fine(256), Some(192), None),
        "a larger picture serves a smaller tile"
    );
    assert!(
        !wants_finer(Grade::Draft, Some(192), Some(192)),
        "refused there"
    );
    assert!(
        !wants_finer(Grade::Draft, Some(256), Some(192)),
        "and wider"
    );
    assert!(
        wants_finer(Grade::Draft, Some(128 + 64), Some(256)),
        "narrower may succeed"
    );
}

/// The tile width is stepped, so a splitter drag is not a render per frame,
/// and no fine picture is wanted while the draft is wide enough.
#[test]
fn the_fine_width_is_stepped_and_off_for_narrow_tiles() {
    let mut cache = ThumbnailCache::default();
    cache.set_tile_width(112.0);
    assert_eq!(cache.fine_width, None);
    cache.set_tile_width(141.0);
    assert_eq!(cache.fine_width, Some(192));
    cache.set_tile_width(190.0);
    assert_eq!(cache.fine_width, Some(192));
    cache.set_tile_width(f32::NAN);
    assert_eq!(cache.fine_width, None);
}

/// A page with no picture is never a fine candidate: it gets its draft first.
#[test]
fn a_page_with_no_picture_is_not_drawn_finer_first() {
    let mut cache = ThumbnailCache::default();
    cache.set_tile_width(400.0);
    assert_eq!(cache.next_finer(&[0, 1, 2], 1), None);
    assert_eq!(cache.next_to_render(&[0, 1, 2], 1), Some(1));
    assert_eq!(cache.pending_grade(1), Grade::Draft);
}
