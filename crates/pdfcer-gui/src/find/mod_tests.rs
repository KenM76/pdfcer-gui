#![cfg(test)]
//! # `find::mod_tests` - what the find state machine promises, proved headlessly
//!
//!
//! ## Why these are worth having without a GUI
//!
//! Everything here exercises the parts of Find that can be *silently* wrong:
//! the currency test that decides whether a stored result set may still be
//! stepped through, the wrap-around arithmetic, the epoch that invalidates
//! hits after an edit, and the hand-written [`super::FindState::default`]
//! whose whole reason for existing is that a derive would flip one field.
//! None of those would look broken in a screenshot; all of them change what
//! the operator is shown.
//!
//! ★ What is NOT proved here, deliberately: anything about where the view
//! ends up. That is `tools/ui-verify`'s, because it is a property of a
//! running window and R1 says a passing test is not a report of working
//! software. The O179 fix - the *Zoom* option holding the position as well
//! as the zoom - is asserted by driving the binary, not from here.
//!
//! `#![cfg(test)]` is the FIRST line of the file, so nothing here reaches a
//! release build and the module costs the shipped binary nothing.
use super::*;
use crate::app::state::{FOUR_PAGES, open_fixture};

/// A state with `hits` hits on page `page`, already searched for `query`.
fn searched(query: &str, page: usize, hits: usize) -> FindState {
    #[allow(
        clippy::cast_precision_loss,
        reason = "small test-fixture indices, stacked into distinct rows" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let hits: Vec<Hit> = (0..hits)
        .map(|i| Hit {
            page,
            canvas: Some(Rect::from_min_size(
                egui::pos2(10.0, 10.0 * i as f32),
                egui::vec2(40.0, 8.0),
            )),
            text: query.to_owned(),
        })
        .collect();
    FindState {
        query: query.to_owned(),
        results: Some(Results {
            unsearchable_fonts: 0,
            query: query.to_owned(),
            options: FindOptions::default(),
            epoch: 0,
            hits,
            current: 0,
        }),
        ..FindState::default()
    }
}

// =======================================================================
// ★ The trap
// =======================================================================

/// ★ **The default search is literal.**
///
/// The regression test for the defect this whole module's header is
/// about: the old shell's Find bar ran through `EditSession::find_text`,
/// which passes `with_wildcards(true)`, so a typed `?` matched every
/// character on the page. `to_core` is the ONE place a
/// `TextSearchOptions` is built in this crate, so asserting on it is
/// asserting on every search this shell can run.
#[test]
fn the_default_search_is_literal() {
    let core = FindOptions::default().to_core();
    assert!(
        !core.wildcards,
        "a search box must search for what was typed; `?` is a question mark"
    );
    assert!(
        core.case_insensitive,
        "a find bar is forgiving about case by default — Reader's toggle is off, and so \
         is every browser's"
    );
    assert!(!core.whole_word);
    assert_eq!(core.word_boundary, WordBoundary::Alphanumeric);
}

/// ★ **Wildcards are only ever on because the operator asked.**
///
/// The other direction, which matters as much: the control has to work,
/// or the escape hatch from the literal default would be a dead
/// checkbox — the placeholder P3 forbids, in the one place the operator
/// went looking for a feature.
#[test]
fn a_wildcard_search_is_only_ever_asked_for_explicitly() {
    let asked = FindOptions {
        wildcards: true,
        ..FindOptions::default()
    };
    assert!(asked.to_core().wildcards);
}

/// The case control's polarity is inverted exactly once.
///
/// The shell says *Match case* and the engine says `case_insensitive`.
/// A dropped `!` here is a search that ignores the checkbox, which looks
/// like a search that ignores the operator.
#[test]
fn match_case_inverts_into_the_engines_polarity() {
    let sensitive = FindOptions {
        case_sensitive: true,
        ..FindOptions::default()
    };
    assert!(!sensitive.to_core().case_insensitive);
    assert!(FindOptions::default().to_core().case_insensitive);
}

/// The whole-word flag and the rule travel independently.
///
/// `TextSearchOptions::with_word_boundary`'s own docs require this:
/// choosing a rule must not switch the option on, and switching the
/// option off and on again must not reset the rule.
#[test]
fn the_word_rule_and_the_whole_word_flag_are_independent() {
    let rule_only = FindOptions {
        word_boundary: WordBoundary::NonSpace,
        ..FindOptions::default()
    };
    let core = rule_only.to_core();
    assert!(!core.whole_word, "choosing a rule does not switch it on");
    assert_eq!(core.word_boundary, WordBoundary::NonSpace);

    let both = FindOptions {
        whole_word: true,
        ..rule_only
    };
    assert!(both.to_core().whole_word);
    assert_eq!(both.to_core().word_boundary, WordBoundary::NonSpace);
}

/// Every rule the chooser offers is a real variant, and the list is the
/// whole of what a `#[non_exhaustive]` enum lets this crate name.
///
/// The chooser is driven from [`FindOptions::WORD_RULES`] rather than
/// from a `match`, because a wildcard arm over a non-exhaustive enum
/// would silently drop a future variant instead of failing to compile.
/// This is the reminder that the list is the thing to extend.
#[test]
fn every_word_rule_the_chooser_offers_has_a_label() {
    assert_eq!(FindOptions::WORD_RULES.len(), 3);
    for rule in FindOptions::WORD_RULES {
        assert!(!bar::word_rule_label(*rule).is_empty());
    }
}

// =======================================================================
// The readout — the pure rule
// =======================================================================

/// Nothing searched yet reads as nothing, not as "no matches".
#[test]
fn an_unsearched_bar_says_nothing_rather_than_no_matches() {
    let mut state = FindState::default();
    assert_eq!(state.readout(0), Readout::Idle);
    state.query = "total".to_owned();
    assert_eq!(
        state.readout(0),
        Readout::Idle,
        "typing is not searching; the readout must stay blank until Enter"
    );
}

/// A completed search reads one-based, and stepping moves it.
#[test]
fn the_readout_counts_hits_from_one() {
    let mut state = searched("total", 2, 3);
    assert_eq!(
        state.readout(0),
        Readout::At {
            current: 1,
            total: 3
        }
    );
    let results = state.results.as_mut().expect("searched");
    results.current = 2;
    assert_eq!(
        state.readout(0),
        Readout::At {
            current: 3,
            total: 3
        }
    );
}

/// A fruitless search says so, which is different from having not run.
#[test]
fn a_fruitless_search_reports_that_it_ran() {
    let state = searched("nothing", 0, 0);
    assert_eq!(state.readout(0), Readout::Empty);
}

/// ★ **Editing the document makes the results stale, not merely old.**
///
/// The staleness rule this module's header argues for, asserted through
/// both surfaces it governs: the readout says so, and the highlights stop.
#[test]
fn an_edit_makes_the_results_stale_and_stops_the_highlights() {
    let state = searched("total", 1, 4);
    assert!(matches!(state.readout(0), Readout::At { .. }));
    assert!(state.current_hit(0).is_some());
    assert_eq!(state.page_highlights(1, 0).count(), 4);

    // One edit later.
    assert_eq!(state.readout(1), Readout::Stale);
    assert!(
        state.current_hit(1).is_none(),
        "a quad recorded before an edit may cover different glyphs after it"
    );
    assert_eq!(
        state.page_highlights(1, 1).count(),
        0,
        "rule 4 forbids painting a mark over content that does not say what it claims"
    );
}

/// ★ **Staleness is reported ahead of emptiness.**
///
/// A document edited after a fruitless search must not say "No matches":
/// that would be a claim about the current revision, which the search
/// never examined.
#[test]
fn an_edited_document_says_it_changed_rather_than_that_there_are_no_matches() {
    let state = searched("nothing", 0, 0);
    assert_eq!(state.readout(0), Readout::Empty);
    assert_eq!(state.readout(1), Readout::Stale);
}

/// Editing the query blanks the readout rather than staling it.
///
/// A different question is not an out-of-date answer to this one. The
/// operator who starts typing a new term should see the readout clear,
/// not see the old count go on standing next to new text.
#[test]
fn changing_the_query_blanks_the_readout() {
    let mut state = searched("total", 0, 5);
    state.query.push('s');
    assert_eq!(state.readout(0), Readout::Idle);
    assert_eq!(state.page_highlights(0, 0).count(), 0);
}

/// Changing an option does the same thing, by the same rule.
#[test]
fn changing_an_option_blanks_the_readout() {
    let mut state = searched("total", 0, 5);
    state.set_options(FindOptions {
        whole_word: true,
        ..FindOptions::default()
    });
    assert_eq!(state.readout(0), Readout::Idle);
}

// =======================================================================
// Stepping
// =======================================================================

/// ★ **Stepping wraps in both directions**, including the underflow case
/// that a naive `- 1` gets wrong.
#[test]
fn stepping_wraps_at_both_ends() {
    assert_eq!(next_index(0, 3, Step::Next), 1);
    assert_eq!(
        next_index(2, 3, Step::Next),
        0,
        "the last hit wraps to the first"
    );
    assert_eq!(next_index(1, 3, Step::Previous), 0);
    assert_eq!(
        next_index(0, 3, Step::Previous),
        2,
        "the first hit's predecessor is the last; `- 1` on a usize underflows here"
    );
    // One hit is its own neighbour in both directions.
    assert_eq!(next_index(0, 1, Step::Next), 0);
    assert_eq!(next_index(0, 1, Step::Previous), 0);
}

/// An empty list cannot be stepped into a panic.
///
/// Unreachable through [`step_to`], which checks the readout first, and
/// handled anyway: an action can be raised from a customized keymap in
/// any state, and an index into an empty `Vec` is a crash waiting for
/// somebody to find it.
#[test]
fn stepping_an_empty_result_set_is_not_a_panic() {
    assert_eq!(next_index(0, 0, Step::Next), 0);
    assert_eq!(next_index(0, 0, Step::Previous), 0);
}

// =======================================================================
// Lifecycle
// =======================================================================

/// Closing the bar takes the highlights with it and keeps the query.
#[test]
fn closing_clears_the_hits_and_keeps_the_query() {
    let mut state = searched("total", 0, 3);
    state.open();
    assert!(state.is_open());
    state.close();
    assert!(!state.is_open());
    assert_eq!(state.query(), "total", "what was typed survives a close");
    assert_eq!(
        state.readout(0),
        Readout::Idle,
        "a closed bar must not leave marks on the page with nothing to explain them"
    );
}

/// Opening asks for focus exactly once per request.
///
/// Re-requesting focus every frame is how a find bar becomes a trap the
/// operator cannot click out of.
#[test]
fn opening_asks_for_focus_once() {
    let mut state = FindState::default();
    state.open();
    assert!(state.take_focus_request());
    assert!(!state.take_focus_request());

    // …and Ctrl+F on an already-open bar asks again, which is what every
    // browser does and is the recovery after clicking on the page.
    state.open();
    assert!(state.take_focus_request());
}

/// The toggle reports the state it produced.
#[test]
fn the_toggle_reports_where_it_landed() {
    let mut state = FindState::default();
    assert!(state.toggle());
    assert!(!state.toggle());
}

/// ★ **A document change forgets the hits and keeps the operator's
/// settings.**
///
/// Page indices and page-space rectangles describe one file. Carrying
/// them into another is not staleness — the epoch would still match,
/// because a freshly opened document's epoch is 0 — it is nonsense, and
/// it is why this seam exists rather than relying on the epoch alone.
#[test]
fn opening_another_document_forgets_the_hits_but_not_the_query() {
    let mut state = searched("total", 3, 9);
    let options = FindOptions {
        whole_word: true,
        ..FindOptions::default()
    };
    state.set_options(options);
    state.forget_document();
    assert_eq!(state.readout(0), Readout::Idle);
    assert_eq!(state.query(), "total");
    assert_eq!(state.options(), options);
}

/// Only the current page's hits are handed to the overlay, and exactly
/// one of them is marked current.
#[test]
fn the_overlay_is_given_this_pages_hits_with_one_marked_current() {
    let mut state = searched("total", 2, 3);
    // Move one hit onto another page, the way a real multi-page result
    // set looks.
    state.results.as_mut().expect("searched").hits.push(Hit {
        page: 5,
        canvas: Some(Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1.0, 1.0),
        )),
        text: "total".to_owned(),
    });

    let on_two: Vec<FindHighlight> = state.page_highlights(2, 0).collect();
    assert_eq!(on_two.len(), 3);
    assert_eq!(on_two.iter().filter(|h| h.current).count(), 1);
    assert!(on_two[0].current, "the search landed on the first hit");

    let on_five: Vec<FindHighlight> = state.page_highlights(5, 0).collect();
    assert_eq!(on_five.len(), 1);
    assert!(
        !on_five[0].current,
        "the current hit is on page 2, so page 5's is not it"
    );

    assert_eq!(state.page_highlights(0, 0).count(), 0);
}

/// A hit whose page would not project is counted and navigable but not
/// drawn.
///
/// "We cannot draw a box on this page" is not "this hit does not exist",
/// and conflating them would make a document with one degenerate page
/// report the wrong number of hits.
#[test]
fn a_hit_with_no_geometry_still_counts() {
    let mut state = searched("total", 0, 2);
    state.results.as_mut().expect("searched").hits[0].canvas = None;
    assert_eq!(
        state.readout(0),
        Readout::At {
            current: 1,
            total: 2
        },
        "the hit is still one of the hits"
    );
    assert_eq!(
        state.page_highlights(0, 0).count(),
        1,
        "…and the one that can be drawn still is"
    );
}

// =======================================================================
// The whole thing, against a real document
// =======================================================================

/// ★ **A real search runs, reports its cost, and lands on its first
/// hit.**
///
/// The end-to-end check that the borrow protocol works: the render worker
/// is stopped, `Arc::get_mut` succeeds, the engine is asked, and the view
/// moves to the page the answer is on. It is deliberately driven through
/// [`apply`] rather than through [`search`] directly, because the thing
/// most likely to be wrong is the wiring rather than the arithmetic.
///
/// The fixture's text is asserted to exist first: a test that searched
/// for a string the fixture does not contain would pass on a build whose
/// search always returned nothing.
#[test]
fn a_real_search_finds_its_text_and_navigates_to_it() {
    let mut doc = open_fixture(FOUR_PAGES);
    let mut state = FindState::default();
    state.open();

    // Whatever this fixture actually says. `Page` is the word the
    // generator stamps on each sheet; if that ever changes, this test
    // fails loudly rather than silently proving nothing.
    state.query_mut().push_str("Page");
    apply(&mut state, &mut doc, FindRequest::Search);

    let Readout::At { current, total } = state.readout(doc.edit_epoch) else {
        panic!(
            "the fixture must contain the search term, or this test proves nothing: {:?}",
            state.readout(doc.edit_epoch)
        )
    };
    assert_eq!(current, 1, "a search lands on its first hit");
    assert!(total >= 1);

    let hit_page = state
        .current_hit(doc.edit_epoch)
        .expect("a current hit")
        .page;
    assert_eq!(
        doc.view.page_index, hit_page,
        "a search that does not go to the page it found is a report, not a search"
    );

    // Stepping moves the readout and stays inside the ring.
    apply(&mut state, &mut doc, FindRequest::Step(Step::Next));
    let Readout::At { current: after, .. } = state.readout(doc.edit_epoch) else {
        panic!("stepping must leave a current hit")
    };
    assert_eq!(after, if total == 1 { 1 } else { 2 });
}

/// An empty query is refused rather than searched.
///
/// The two states must not look alike: "you have not typed anything" is
/// blank, "there is nothing here" is a sentence.
#[test]
fn an_empty_query_is_not_a_search() {
    let mut doc = open_fixture(FOUR_PAGES);
    let mut state = FindState::default();
    apply(&mut state, &mut doc, FindRequest::Search);
    assert_eq!(
        state.readout(doc.edit_epoch),
        Readout::Idle,
        "an empty box must not report `No matches`"
    );
}

/// A search does not look like an edit.
///
/// `find_text_with` takes `&mut EditSession`, which makes it easy to
/// mistake for a mutation and to give it `vector_edit`'s epoch bump and
/// texture drop. Both would be wrong: the bump would make the results
/// stale by their own rule the instant they were produced, and the drop
/// would re-rasterize a CAD sheet on every Enter.
#[test]
fn a_search_bumps_no_epoch_and_drops_no_texture() {
    let mut doc = open_fixture(FOUR_PAGES);
    let mut state = FindState::default();
    state.query_mut().push_str("Page");
    let before = doc.edit_epoch;
    apply(&mut state, &mut doc, FindRequest::Search);
    assert_eq!(
        doc.edit_epoch, before,
        "a search changes nothing about the document"
    );
    assert!(
        !matches!(state.readout(doc.edit_epoch), Readout::Stale),
        "…and must not invalidate the results it just produced"
    );
}
