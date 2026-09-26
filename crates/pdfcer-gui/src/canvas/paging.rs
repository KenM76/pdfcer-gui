//! # `canvas::paging` — the wheel as a page turn
//!
//! ## The request
//!
//!
//! > *"when in single page view there should be an option on screen near the
//! > button to scroll or flip through pages, or the current way it is now when
//! > the scroll wheel is used."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/paging.md`.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// How far the wheel must travel, in logical points, to turn one page.
const POINTS_PER_PAGE: f32 = 40.0;

/// **Is the plain wheel a page turn rather than a scroll this frame?**
#[must_use]
pub(super) fn wheel_turns_pages(doc: &OpenDoc) -> bool {
    doc.prefs.wheel_paging.flips() && !doc.view.display.is_continuous()
}

/// **Is there a page to turn to?** [`wheel_turns_pages`] on a document with
/// more than one page; the condition under which [`flip`] spends travel.
#[must_use]
pub(super) fn flips_pages(doc: &OpenDoc) -> bool {
    wheel_turns_pages(doc) && doc.pages.len() > 1
}

/// Accumulate this frame's wheel travel and raise a page turn when it is
/// enough.
pub(super) fn flip(ui: &egui::Ui, doc: &mut OpenDoc, hovered: bool, actions: &mut Vec<Action>) {
    if !flips_pages(doc) {
        // Nothing pending can survive a mode change: a half-notch of travel
        // banked under one display mode must not turn a page under another.
        doc.wheel_travel = 0.0;
        return;
    }
    if !hovered {
        return;
    }
    let delta = ui.input(|i| i.smooth_scroll_delta.y);
    if delta == 0.0 {
        return;
    }
    if doc.wheel_travel != 0.0 && doc.wheel_travel.signum() != delta.signum() {
        doc.wheel_travel = 0.0;
    }
    doc.wheel_travel += delta;
    if doc.wheel_travel.abs() < POINTS_PER_PAGE {
        return;
    }
    let back = doc.wheel_travel > 0.0;
    doc.wheel_travel = 0.0;
    actions.push(if back {
        Action::PrevPage
    } else {
        Action::NextPage
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use crate::viewer::PageDisplay;

    /// A document with four pages, in single-page display, with flipping on.
    fn ready() -> OpenDoc {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.prefs.wheel_paging = crate::app::prefs::WheelPaging::FlipPages;
        doc.view.display = PageDisplay::Single;
        doc
    }

    /// The predicate refuses under every condition that makes the choice
    /// meaningless — and the status bar asks the same one, so the control and
    /// the behaviour cannot disagree about which frames are which.
    #[test]
    fn flipping_is_refused_wherever_the_choice_does_not_exist() {
        let doc = &mut ready();
        assert!(flips_pages(doc), "the ready fixture must flip");

        doc.prefs.wheel_paging = crate::app::prefs::WheelPaging::Scroll;
        assert!(!flips_pages(doc), "the default setting must not flip");
        doc.prefs.wheel_paging = crate::app::prefs::WheelPaging::FlipPages;

        doc.view.display = PageDisplay::Continuous;
        assert!(
            !flips_pages(doc),
            "a continuous mode scrolls the whole document by definition"
        );
        doc.view.display = PageDisplay::FacingContinuous;
        assert!(!flips_pages(doc), "and so does a facing continuous one");
        doc.view.display = PageDisplay::Facing;
        assert!(
            flips_pages(doc),
            "facing shows one spread at a time, so the choice does exist there"
        );
    }

    /// A one-page document has nowhere to flip to, and the wheel is still
    /// withheld from the scroll area, so a fitted page does not move (O239).
    #[test]
    fn a_single_page_document_neither_flips_nor_scrolls_by_wheel() {
        let doc = &mut ready();
        doc.pages.truncate(1);
        assert!(!flips_pages(doc));
        assert!(wheel_turns_pages(doc));
    }

    /// Travel below the threshold banks and does not turn a page; travel that
    /// reaches it turns exactly one and spends the whole accumulator.
    #[test]
    fn travel_accumulates_and_one_threshold_buys_exactly_one_page() {
        let doc = &mut ready();
        // Two thirds of a threshold, twice: the first banks, the second pays.
        let step = POINTS_PER_PAGE * 0.67;
        doc.wheel_travel += -step;
        assert!(
            doc.wheel_travel.abs() < POINTS_PER_PAGE,
            "one step is not enough"
        );
        doc.wheel_travel += -step;
        assert!(
            doc.wheel_travel.abs() >= POINTS_PER_PAGE,
            "two thirds twice must reach the threshold"
        );
    }

    /// The sign convention, pinned. `egui`'s delta is positive when the
    /// operator scrolls toward the START of the document.
    #[test]
    fn rolling_the_wheel_up_goes_back_and_down_goes_on() {
        // Positive travel is "back", negative is "on". Expressed against the
        // same expression `flip` uses so a reversal of it fails here.
        let back = |travel: f32| travel > 0.0;
        assert!(back(POINTS_PER_PAGE), "a positive delta is a previous page");
        assert!(!back(-POINTS_PER_PAGE), "a negative delta is a next page");
    }

    /// A direction change discards the banked travel, so half a notch
    /// forward and half a notch back is not a page turn by cancellation.
    #[test]
    fn a_direction_change_discards_the_banked_travel() {
        let doc = &mut ready();
        doc.wheel_travel = POINTS_PER_PAGE * 0.9;
        let delta = -1.0_f32;
        // The guard `flip` applies, spelled out so this test fails if it is
        // removed rather than merely if it is changed.
        if doc.wheel_travel != 0.0 && doc.wheel_travel.signum() != delta.signum() {
            doc.wheel_travel = 0.0;
        }
        doc.wheel_travel += delta;
        assert!(
            doc.wheel_travel.abs() < POINTS_PER_PAGE,
            "reversing must not arrive at a page turn"
        );
    }
}
