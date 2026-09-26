//! # `panels::pages::select` — which pages the operator has picked
//!
//! A page selection, and the three-modifier click rule that builds it. Pure:
//! no `egui`, no document, no rendering. That is deliberate and it is what
//! makes the rule testable — the interesting part of a multi-select is the
//! *policy* (what does Shift extend from? what does a plain click discard?),
//! and the policy is the part that can be wrong in a way an operator would
//! notice.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/pages/select.md`.

use std::collections::BTreeSet;

/// Which pages are picked, and where a range would extend from.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PageSelection {
    /// The picked pages, 0-based, ascending by construction.
    pages: BTreeSet<usize>,
    /// The page a Shift+click would extend *from*, or `None` before the
    /// operator has named one.
    ///
    /// See the module header: moved by a plain click and by a Ctrl+click,
    /// never by a Shift+click.
    anchor: Option<usize>,
}

/// What a click on a tile asked for, beyond changing the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickOutcome {
    /// Whether the canvas should navigate to the clicked page.
    pub navigate: bool,
}

impl PageSelection {
    /// The picked pages, in document order.
    #[must_use]
    pub fn pages(&self) -> &BTreeSet<usize> {
        &self.pages
    }

    /// How many pages are picked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Whether nothing is picked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Whether `page` is picked.
    #[must_use]
    pub fn contains(&self, page: usize) -> bool {
        self.pages.contains(&page)
    }

    /// Pick nothing.
    pub fn clear(&mut self) {
        self.pages.clear();
        self.anchor = None;
    }

    /// **Apply a click on `page`, with the modifiers that came with it.**
    pub fn click(&mut self, page: usize, ctrl: bool, shift: bool) -> ClickOutcome {
        if ctrl {
            if !self.pages.remove(&page) {
                self.pages.insert(page);
            }
            // The anchor moves even when the click REMOVED the page: the
            // operator named this page deliberately, and a following
            // Shift+click means "from here". An anchor left on a page that
            // was deselected three clicks ago is the case that makes range
            // extension feel random.
            self.anchor = Some(page);
            return ClickOutcome { navigate: false };
        }
        if shift && let Some(anchor) = self.anchor {
            let (lo, hi) = if anchor <= page {
                (anchor, page)
            } else {
                (page, anchor)
            };
            // Replaces rather than unions, which is what makes an overshoot
            // correctable: Shift+click too far, Shift+click back, and the
            // range is the one you meant. A union would leave the overshoot
            // permanently selected with no gesture that removes it.
            self.pages = (lo..=hi).collect();
            // Deliberately NOT moved — see the module header.
            return ClickOutcome { navigate: false };
        }
        // Plain click, and Shift+click with no anchor to extend from: the
        // second is the first click of a session, and treating it as a plain
        // click is the only defined answer that leaves the operator somewhere
        // sensible.
        self.pages.clear();
        self.pages.insert(page);
        self.anchor = Some(page);
        ClickOutcome { navigate: true }
    }

    /// **Make a right-click's operand list agree with what was pointed at.**
    pub fn right_click(&mut self, page: usize) -> bool {
        if self.pages.contains(&page) {
            return false;
        }
        self.pages.clear();
        self.pages.insert(page);
        self.anchor = Some(page);
        true
    }

    /// Drop any picked page at or beyond `page_count`, and the anchor with
    /// it.
    pub fn retain_below(&mut self, page_count: usize) -> bool {
        let before = self.pages.len();
        self.pages.retain(|p| *p < page_count);
        if self.anchor.is_some_and(|a| a >= page_count) {
            self.anchor = None;
        }
        self.pages.len() != before
    }

    /// **Follow the picked pages across a reorder.**
    pub fn remap(&mut self, landed: &[usize]) {
        self.pages = self
            .pages
            .iter()
            .filter_map(|p| landed.get(*p).copied())
            .collect();
        self.anchor = self.anchor.and_then(|a| landed.get(a).copied());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
