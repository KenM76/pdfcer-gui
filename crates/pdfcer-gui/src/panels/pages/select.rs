//! # `panels::pages::select` — which pages the operator has picked
//!
//! A page selection, and the three-modifier click rule that builds it. Pure:
//! no `egui`, no document, no rendering. That is deliberate and it is what
//! makes the rule testable — the interesting part of a multi-select is the
//! *policy* (what does Shift extend from? what does a plain click discard?),
//! and the policy is the part that can be wrong in a way an operator would
//! notice.
//!
//! ## ★ This is a SECOND selection in the application, and it is not the
//! canvas's
//!
//! `crate::panels::ObjectTreeUi::focus`'s own docs refuse to become a second
//! selection, and `the_panel_focus_has_not_quietly_become_a_selection`
//! defends that refusal — so a new selection type arriving two modules over
//! needs its reason stated rather than assumed.
//!
//! The reason is that this one selects a **different kind of thing**. The
//! canvas's [`crate::canvas::selection::SelectionState`] selects *objects on
//! a page*; this selects *pages in a document*. They cannot be confused,
//! they cannot drift into each other, and no command reads both:
//!
//! | | canvas selection | page selection |
//! |---|---|---|
//! | operand | paint-order indices on one page | 0-based page indices |
//! | condition | `selection.any` | — (the `pages.*` commands are gated on `doc.pages`) |
//! | verbs | `format.delete`, the move verbs | `pages.delete`, `pages.extract`, `pages.move_*`, `pages.rotate_*` |
//! | survives a page change | yes | yes — it *is* about pages |
//! | survives an edit | resolved against the new decomposition | **cleared**, see [`PageSelection::retain_below`] |
//!
//! The hazard the Objects panel's focus was guarding against — a *report*
//! surface arming a destructive command that acts on something else — does
//! not arise, because the destructive commands that read this selection are
//! the ones whose tooltips already say *"the selected pages"*. There is no
//! other candidate for what that phrase means.
//!
//! ## The three-modifier rule
//!
//! Standard, and standard on purpose: this is the idiom every file list, every
//! layer palette and every other PDF reader's page rail uses, so an operator
//! arrives already knowing it. Deviating would be a novelty tax paid on every
//! click.
//!
//! | Gesture | Selection | Navigates |
//! |---|---|---|
//! | click | exactly this page | **yes** |
//! | Ctrl+click | toggles this page's membership | no |
//! | Shift+click | every page between the anchor and this one | no |
//!
//! **Only a plain click navigates**, and that is the load-bearing half. A
//! Ctrl+click that also moved the canvas would make building a set of five
//! pages a five-page journey through the document, re-rendering the canvas
//! each time — on the benchmark drawing, five renders of ~0.8 s to perform a
//! gesture that changes nothing about what the operator is looking at.
//!
//! ## The anchor, and why Shift+click needs one
//!
//! A range needs two ends. The anchor is the last page the operator named
//! *deliberately* — by a plain click or a Ctrl+click — and it is **not**
//! moved by a Shift+click, so a second Shift+click adjusts the same range
//! rather than growing it from wherever the first one landed. That is the
//! behaviour of every list that gets this right, and the difference is only
//! visible when someone overshoots and corrects, which is exactly when it
//! matters.

use std::collections::BTreeSet;

/// Which pages are picked, and where a range would extend from.
///
/// `BTreeSet` rather than `Vec`: the set is asked *"is page N in it?"* once
/// per tile per frame, and it is handed to commands that want it in document
/// order. A `Vec` would answer the first question in linear time and the
/// second only if every insertion site remembered to keep it sorted.
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
///
/// A struct with one field rather than a bare `bool`, because the caller
/// reads it as *"should I raise `Action::GoToPage`?"* and a bare `bool`
/// returned from `click` reads as *"did the selection change?"* — which is a
/// different question with a different answer.
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
    ///
    /// Clears the anchor too. An anchor without a selection is a range
    /// endpoint for a range nobody started, and leaving it would make the
    /// next Shift+click extend from a page the operator has no reason to
    /// remember naming.
    pub fn clear(&mut self) {
        self.pages.clear();
        self.anchor = None;
    }

    /// **Apply a click on `page`, with the modifiers that came with it.**
    ///
    /// The whole rule, in one place. See the module header for the table and
    /// for why only a plain click navigates.
    ///
    /// `ctrl` wins over `shift` when both are held. Ctrl+Shift+click means
    /// "extend the existing set" in some applications and "extend the range
    /// additively" in others; there is no agreed answer, so pdfcer takes the
    /// one whose behaviour is fully described by a rule already stated
    /// (toggle) rather than inventing a fourth gesture nothing documents.
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
    ///
    /// The same rule `crate::canvas::menus::select_under_right_click` states
    /// for objects, applied to pages, and it is here rather than reasoned out
    /// again at the call site so the two surfaces cannot come to disagree:
    ///
    /// 1. **Over an unpicked page** — pick it, alone. A context menu's
    ///    implicit promise is *"these verbs apply to the thing you pointed
    ///    at"*, and without this step right-clicking page 9 while pages 1–3
    ///    are selected and choosing Delete destroys 1–3.
    /// 2. **Over a page that is already picked** — change nothing. A
    ///    Shift-selected run of eight sheets followed by a right-click on one
    ///    of them must still offer to extract all eight.
    ///
    /// Returns whether the selection changed, for the trace.
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
    ///
    /// **Called after anything that can change how many pages there are.** A
    /// page index is a *position in a document*, not an identity: deleting
    /// page 2 of four does not leave "page 3" selected, it leaves a selection
    /// naming a page that is now a different sheet. Clamping is the only
    /// honest response available without a page-identity model, and it is
    /// cheap.
    ///
    /// Returns whether anything was dropped.
    pub fn retain_below(&mut self, page_count: usize) -> bool {
        let before = self.pages.len();
        self.pages.retain(|p| *p < page_count);
        if self.anchor.is_some_and(|a| a >= page_count) {
            self.anchor = None;
        }
        self.pages.len() != before
    }

    /// **Follow the picked pages across a reorder.**
    ///
    /// `landed[p]` is the position page `p` now occupies —
    /// [`super::ops::inverse`]'s output, which is the inverse of the
    /// permutation handed to `EditSession::reorder_pages`.
    ///
    /// ## ★ Why this remaps where [`Self::retain_below`] clamps
    ///
    /// The two are the same problem — *a page index is a position, not an
    /// identity* — meeting two different edits, and the honest answer differs
    /// because the available information does:
    ///
    /// | edit | what happened to the picked sheets | answer |
    /// |---|---|---|
    /// | delete | they **stopped existing** | there is nothing to point at; the caller clears |
    /// | reorder | they are still here, **somewhere else** | the permutation says exactly where |
    ///
    /// `retain_below`'s docs call clamping *"the only honest response available
    /// without a page-identity model"*, and for a delete it is. A reorder is
    /// the case where a page-identity model is not needed, because the
    /// permutation **is** one for the duration of the edit: it states, per
    /// page, where that page went. Throwing that away and clearing would mean
    /// an operator who moved four sheets up one place had to re-select them to
    /// move them again — which makes the reorder arrows useless for the one
    /// gesture they exist for.
    ///
    /// The anchor moves with its page for the same reason a Shift+click extends
    /// from where the operator last named something: after a move, "from here"
    /// still means that sheet.
    ///
    /// A page whose new position `landed` does not state is **dropped**. That
    /// cannot happen for an `ops::inverse` result over a real permutation and
    /// is defined rather than panicking, because the alternative to dropping is
    /// keeping an index whose meaning nobody can state.
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
    ///
    /// The navigation half is asserted because it is the only gesture that
    /// has it, and a regression that made every click navigate would cost a
    /// canvas re-render per click — ~0.8 s each on the benchmark drawing.
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

    /// **★ A second Shift+click adjusts the same range rather than growing
    /// it.**
    ///
    /// The property the anchor rule exists for, and the only one that is
    /// invisible until somebody overshoots. If the anchor moved with each
    /// Shift+click, correcting an overshoot from 5→20 back to 5→8 would leave
    /// 8–20 selected with no gesture that removes them.
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

    /// **★ A right-click over an unpicked page picks it first…**
    ///
    /// Without this, right-clicking page 9 while 1–3 are picked and choosing
    /// Delete destroys 1–3 — the pointer and the operand list disagreeing,
    /// with an irreversible verb between them.
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

    /// **★ A page index is a position, not an identity.**
    ///
    /// The document shrinking must drop the picks that no longer name a
    /// page. Keeping them would leave a selection pointing at a *different*
    /// sheet, and the next `pages.delete` would remove one nobody chose.
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

    /// **★ A reorder carries the picked pages to their new positions.**
    ///
    /// The property that makes the reorder arrows usable more than once: move
    /// four sheets up, and they are still the four sheets that are picked, so
    /// the next press moves the same four. A build that cleared here — or, far
    /// worse, one that left the indices alone — would leave the second press
    /// acting on whichever sheets happen to sit at those positions now, which
    /// is a *destructive* verb pointed at pages nobody chose the moment the
    /// operator reaches for Delete instead.
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
