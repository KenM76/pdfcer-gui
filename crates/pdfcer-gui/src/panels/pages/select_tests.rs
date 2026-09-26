//! Tests for `crate::panels::pages::select`, kept in the gui because they reach gui modules.

use crate::panels::pages::select::*;
use std::collections::BTreeSet;

/// A plain click picks exactly one page and navigates.
#[test]
fn a_plain_click_replaces_the_selection_and_navigates() {
    let mut sel = PageSelection::default();
    assert_eq!(sel.click(3, false, false), ClickOutcome { navigate: true });
    assert_eq!(sel.pages(), &BTreeSet::from([3]));

    assert_eq!(sel.click(7, false, false), ClickOutcome { navigate: true });
    assert_eq!(
        sel.pages(),
        &BTreeSet::from([7]),
        "a plain click discards what was picked; it does not add"
    );
}

/// Ctrl+click toggles, and does not move the canvas.
#[test]
fn ctrl_click_toggles_membership_without_navigating() {
    let mut sel = PageSelection::default();
    sel.click(1, false, false);
    assert_eq!(sel.click(4, true, false), ClickOutcome { navigate: false });
    assert_eq!(sel.pages(), &BTreeSet::from([1, 4]));

    // …and again removes it.
    assert_eq!(sel.click(4, true, false), ClickOutcome { navigate: false });
    assert_eq!(sel.pages(), &BTreeSet::from([1]));
}

/// Shift+click selects the range between the anchor and the click,
/// in either direction.
#[test]
fn shift_click_extends_a_range_from_the_anchor_both_ways() {
    let mut sel = PageSelection::default();
    sel.click(5, false, false);
    sel.click(8, false, true);
    assert_eq!(sel.pages(), &BTreeSet::from([5, 6, 7, 8]));

    // Backwards from the SAME anchor, which is still 5.
    sel.click(2, false, true);
    assert_eq!(sel.pages(), &BTreeSet::from([2, 3, 4, 5]));
}

/// **A second Shift+click adjusts the same range rather than growing
/// it.**
#[test]
fn correcting_an_overshoot_shrinks_the_range() {
    let mut sel = PageSelection::default();
    sel.click(5, false, false);
    sel.click(20, false, true);
    assert_eq!(sel.len(), 16);
    sel.click(8, false, true);
    assert_eq!(
        sel.pages(),
        &BTreeSet::from([5, 6, 7, 8]),
        "the anchor moved with the first Shift+click, so the correction \
         extended from 20 instead of from 5"
    );
}

/// Shift+click with nothing to extend from behaves as a plain click,
/// rather than doing nothing.
///
/// Doing nothing would be a click that visibly failed, which is the
/// defect class this project is named after.
#[test]
fn shift_click_with_no_anchor_is_a_plain_click() {
    let mut sel = PageSelection::default();
    assert_eq!(sel.click(6, false, true), ClickOutcome { navigate: true });
    assert_eq!(sel.pages(), &BTreeSet::from([6]));
}

/// **A right-click over an unpicked page picks it first…**
#[test]
fn a_right_click_over_an_unpicked_page_picks_it() {
    let mut sel = PageSelection::default();
    sel.click(1, false, false);
    sel.click(2, true, false);
    sel.click(3, true, false);
    assert!(sel.right_click(9));
    assert_eq!(sel.pages(), &BTreeSet::from([9]));
}

/// …and a right-click inside an existing pick changes nothing.
#[test]
fn a_right_click_inside_the_selection_keeps_it() {
    let mut sel = PageSelection::default();
    sel.click(1, false, false);
    sel.click(8, false, true);
    let before = sel.clone();
    assert!(!sel.right_click(4));
    assert_eq!(
        sel, before,
        "right-clicking one sheet of a run must not collapse it"
    );
}

/// **A page index is a position, not an identity.**
#[test]
fn shrinking_the_document_drops_the_picks_that_fell_off_the_end() {
    let mut sel = PageSelection::default();
    sel.click(0, false, false);
    sel.click(5, false, true);
    assert!(sel.retain_below(3));
    assert_eq!(sel.pages(), &BTreeSet::from([0, 1, 2]));
    // The anchor was page 0, which still exists, so it survives.
    sel.click(2, false, true);
    assert_eq!(sel.pages(), &BTreeSet::from([0, 1, 2]));

    // An anchor that fell off the end is forgotten, so the next
    // Shift+click starts fresh rather than extending from nowhere.
    let mut sel = PageSelection::default();
    sel.click(9, false, false);
    sel.retain_below(4);
    assert!(sel.is_empty());
    assert_eq!(sel.click(1, false, true), ClickOutcome { navigate: true });
}

/// **A reorder carries the picked pages to their new positions.**
#[test]
fn a_reorder_carries_the_picked_pages_with_it() {
    use crate::panels::pages::ops::{MoveDirection, move_order};

    let mut sel = PageSelection::default();
    sel.click(1, false, false);
    sel.click(2, true, false);

    let order = move_order(&[1, 2], 4, MoveDirection::Up).expect("a legal move");
    sel.remap(&crate::panels::pages::ops::inverse(&order));
    assert_eq!(
        sel.pages(),
        &BTreeSet::from([0, 1]),
        "the two picked sheets moved up one place, so the picks must too"
    );

    // …and the anchor came with them, so a following Shift+click still
    // extends from the sheet the operator named rather than from wherever
    // that index now points.
    sel.click(3, false, true);
    assert_eq!(
        sel.pages(),
        &BTreeSet::from([1, 2, 3]),
        "the anchor was page 2, which the move carried to position 1"
    );
}

/// An index the permutation does not mention is dropped rather than kept.
#[test]
fn a_remap_that_cannot_place_a_page_drops_it() {
    let mut sel = PageSelection::default();
    sel.click(0, false, false);
    sel.click(5, true, false);
    // A three-page permutation cannot say where page 5 went.
    sel.remap(&[2, 0, 1]);
    assert_eq!(sel.pages(), &BTreeSet::from([2]));
}

/// Clearing forgets the anchor as well as the pages.
#[test]
fn clearing_forgets_the_anchor_too() {
    let mut sel = PageSelection::default();
    sel.click(4, false, false);
    sel.clear();
    assert!(sel.is_empty());
    // With no anchor, a Shift+click is a plain click — which is the
    // observable consequence of the anchor having gone.
    assert_eq!(sel.click(9, false, true), ClickOutcome { navigate: true });
    assert_eq!(sel.pages(), &BTreeSet::from([9]));
}

/// Ctrl wins over Shift when both are held — one stated rule rather
/// than a fourth undocumented gesture.
#[test]
fn ctrl_shift_click_toggles_rather_than_inventing_a_gesture() {
    let mut sel = PageSelection::default();
    sel.click(2, false, false);
    sel.click(6, true, true);
    assert_eq!(sel.pages(), &BTreeSet::from([2, 6]));
}
