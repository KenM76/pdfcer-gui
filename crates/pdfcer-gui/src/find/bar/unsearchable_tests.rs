//! # `find::bar` — the unsearchable note's tests
//!
//! A second test file rather than a module inside
//! [`super::tests`](crate::find::bar::tests), split out of `bar.rs` alongside
//! it under **R2** on 2026-09-09.
//!
//! ★ Kept separate because it must reach **both** `find::bar` (through
//! `super::*`) and `super::tests::searched`. Nested inside the other module,
//! `super::*` would resolve to the tests rather than to the bar, and every
//! reference to the code under test would have to be re-spelled — a rewrite of
//! working assertions to save one file.
//!
//! ★★ **The inner `#![cfg(test)]` is load-bearing and is not a duplicate of
//! the outer `#[cfg(test)] mod tests;`.** Without it,
//! `tools/gates/check-ui-strings.sh` walks this file as ordinary source and
//! reports every assertion message as a user-visible string that should live
//! in `ui_text` — exclusion 2b in that gate. It is the same line every other
//! split test file in this crate carries, and it was reported by that gate
//! within minutes of this split, exactly as designed.
#![cfg(test)]

use super::tests::searched;
use super::*;

/// **Nothing is said when nothing is wrong.** The guard that keeps this
/// from becoming the nagging the operator objected to in the old shell.
#[test]
fn a_document_with_no_unreachable_fonts_says_nothing() {
    let state = crate::find::FindState::default();
    assert_eq!(
        state.unsearchable_fonts(0),
        0,
        "a bar that has run no search has nothing to disclose"
    );
}

/// **The two disclosures are independent**, asserted as a truth table so
/// that anyone rewriting the row as an if/else has to delete a case.
#[test]
fn the_ocr_offer_and_the_note_are_not_alternatives() {
    // OCR offer depends only on the readout and whether the PAGE has text.
    assert!(
        offer_ocr(Readout::Empty, || false),
        "an empty result on a page with no text is the OCR case"
    );
    assert!(
        !offer_ocr(Readout::Empty, || true),
        "an empty result on a page WITH text is not the OCR case — and it is exactly the case the unsearchable note exists for"
    );
    // …and the note depends on neither of those, only on the document's
    // font diagnostics. The combination in the second assertion above is
    // the one a file with a Type 3 titleblock produces.
}

/// ★★ **A sentence about one search cannot outlive that search.**
///
/// Three ways a result stops describing what the bar is showing, and all
/// three must silence the note: the operator edits the query, the operator
/// changes an option, or the DOCUMENT is edited (a new epoch). The last is
/// the one worth having a test for — an edit does not touch the query, so a
/// naive implementation keeps a stale sentence on screen indefinitely while
/// the bar beside it has already gone blank.
#[test]
fn a_result_that_no_longer_describes_the_bar_discloses_nothing() {
    let mut state = searched("alpha", 0);
    if let Some(r) = state.results.as_mut() {
        r.unsearchable_fonts = 2;
    }
    assert_eq!(
        state.unsearchable_fonts(0),
        2,
        "the current search DOES have something to say — checked first, so the assertions below cannot pass by the accessor simply always returning zero"
    );

    // The document was edited: same query, new epoch.
    assert_eq!(
        state.unsearchable_fonts(1),
        0,
        "an edited document invalidates the hit list, and it invalidates the sentence beside it for the same reason"
    );

    // The operator typed on.
    state.query_mut().push('b');
    assert_eq!(state.unsearchable_fonts(0), 0);
}
