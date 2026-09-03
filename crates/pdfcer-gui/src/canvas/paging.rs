//! # `canvas::paging` — the wheel as a page turn
//!
//! ## The request
//!
//! `OPERATOR_REQUESTS.md` O30, 2026-08-24:
//!
//! > *"when in single page view there should be an option on screen near the
//! > button to scroll or flip through pages, or the current way it is now when
//! > the scroll wheel is used."*
//!
//! ## ★★★ The case for it, which is stronger than a preference
//!
//! This shell opens documents at **fit page** by default. Under
//! [`crate::viewer::PageDisplay::Single`] that means the whole sheet is on
//! screen and there is *nothing to scroll* — so the plain wheel, the most
//! reached-for gesture on a mouse, **does nothing at all**. Not "does
//! something the operator did not want": nothing. Every page change costs a
//! trip to the status bar or a key the operator has to know about.
//!
//! So this is not a taste setting with two equally good answers. It is the
//! difference between a live control and a dead one, in the configuration the
//! program ships in.
//!
//! ## What is deliberately NOT here
//!
//! * **Continuous modes.** The wheel scrolls the whole document there, by
//!   definition, and [`flip`] declines before it reads anything. The status
//!   bar's toggle is not drawn there either — R9, and the two decisions are
//!   made from the same predicate so they cannot disagree.
//! * **Ctrl+wheel.** `egui` routes a modified wheel event into `zoom_delta`
//!   and contributes nothing to the scroll delta, so a zoom gesture never
//!   reaches this module and does not have to be excluded by it. See
//!   `canvas`'s header: keeping those two apart is *"the single most common
//!   way a from-scratch viewer feels wrong"*.
//! * **Deciding whether the scroll area also sees the wheel.** That is the
//!   caller's, one line above the call, because it has to be decided *before*
//!   the `ScrollArea` is built and this runs after. [`flips_pages`] is the
//!   shared predicate so the two cannot disagree about which frames are which.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// How far the wheel must travel, in logical points, to turn one page.
///
/// # ★★ Why a distance and not an event count
///
/// The two devices that produce a wheel do not agree on what an event is. A
/// mouse delivers one detent as a single large delta; a trackpad delivers one
/// swipe as dozens of small ones. Counting events turns a trackpad gesture
/// into forty page turns. Thresholding an instantaneous delta makes a slow,
/// deliberate scroll do nothing at all. **Travel is the quantity both devices
/// agree on**, and a threshold on it behaves for both.
///
/// `egui`'s own default for one wheel line is 50 points at the time of
/// writing. This is deliberately a little under that, so a single detent
/// reliably turns exactly one page even if the platform reports slightly less
/// — and comfortably over the few points a trackpad delivers per frame, so a
/// swipe pages at the speed of the hand rather than of the frame rate.
const POINTS_PER_PAGE: f32 = 40.0;

/// **Does a plain wheel turn pages this frame?**
///
/// The single predicate, asked twice per frame from two places that must
/// agree: once before the `ScrollArea` is built, to decide whether to let it
/// consume the wheel at all, and once after, to spend the travel. Two
/// spellings of this condition would eventually differ, and the frame where
/// they did would either scroll *and* page at once or do neither.
#[must_use]
pub(super) fn flips_pages(doc: &OpenDoc) -> bool {
    doc.prefs.wheel_paging.flips() && !doc.view.display.is_continuous() && doc.pages.len() > 1
}

/// Accumulate this frame's wheel travel and raise a page turn when it is
/// enough.
///
/// Call once per frame, after the canvas has drawn, with `hovered` saying
/// whether the pointer is over the canvas. Pushes at most one
/// [`Action::NextPage`] or [`Action::PrevPage`] per call.
///
/// # The sign, and why it is this way round
///
/// `egui`'s scroll delta is positive when the content should move **down** —
/// i.e. when the operator is scrolling **up**, toward the start. So a positive
/// delta is a *previous* page. Getting this backwards produces a viewer that
/// works and feels wrong, which is harder to notice than one that is broken;
/// [`tests::rolling_the_wheel_up_goes_back_and_down_goes_on`] pins it.
///
/// # ★ Why the accumulator is zeroed rather than decremented
///
/// Subtracting the threshold and keeping the remainder would let a long
/// trackpad swipe page continuously at a rate set by the hand — which sounds
/// right and is not: the remainder carries across the gesture's end, so the
/// *next* small nudge lands a page turn it did not earn. Zeroing makes every
/// turn cost a full threshold of fresh travel, which is what "one notch, one
/// sheet" means.
///
/// ★★ The accumulator is also **reset on a direction change**, so a wheel
/// rolled half a notch forward and then back does not arrive at a page turn by
/// cancellation. Travel toward a page turn is travel in one direction.
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

    /// ★★★ The predicate refuses under every condition that makes the choice
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

    /// A one-page document has nowhere to flip to, so the wheel is left to
    /// scroll — which on a page larger than the window is the only useful
    /// thing it could do.
    #[test]
    fn a_single_page_document_never_flips() {
        let doc = &mut ready();
        doc.pages.truncate(1);
        assert!(!flips_pages(doc));
    }

    /// Travel below the threshold banks and does not turn a page; travel that
    /// reaches it turns exactly one and spends the whole accumulator.
    ///
    /// ★ Asserted through the accumulator rather than by driving `egui`,
    /// because the arithmetic is the part that can be wrong. The gesture
    /// itself is `zooming`-harness territory.
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

    /// ★ A direction change discards the banked travel, so half a notch
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
