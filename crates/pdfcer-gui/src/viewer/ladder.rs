//! # `viewer::ladder` — the zoom levels the `+` and `−` buttons step through
//!
//! One subject: **given a zoom, what is the next one up or down?** Split out
//! of [`super`] under R2 when that file reached 1,540 lines, and the seam is a
//! real one — everything here answers that question and nothing here knows
//! what a page, a viewport or a raster is.
//!
//! ## Why a ladder at all
//!
//! A fixed list of round percentages makes zoom-in and zoom-out **exactly
//! reversible** and makes every step land somewhere a person can name. A zoom
//! that arrived from somewhere else — Ctrl+wheel, a fit mode, a saved
//! document — is off the ladder, and the two functions here take the next rung
//! strictly above or below it, so the ladder doubles as a *snap back to
//! sanity*.
//!
//! ## ★★ Past the ladder's end, the two must stay inverses
//!
//! The named rungs stop at 800 %, which was the maximum zoom until O24 raised
//! it. Above that [`ladder_step_up`] doubles and [`ladder_step_down`] halves —
//! a constant ratio, so a constant number of presses per decade, and the same
//! number of presses back.
//!
//! ★ Both branches were needed and only one was written. `ladder_step_up`
//! grew its doubling when the ceiling was raised; `ladder_step_down` kept a
//! plain reverse search and therefore returned **800 %** from anywhere above
//! it. One press discarded a hundred-fold magnification, which is
//! `OPERATOR_REQUESTS.md` O24g. A pair of controls that disagree about what a
//! step is breaks the one property an operator relies on to explore without
//! losing their place, so the reversibility is pinned as a round-trip test
//! rather than against fixed numbers.

use super::{MAX_ZOOM, MIN_ZOOM};

/// The zoom levels the +/− buttons step through. Ascending, and it
/// contains `1.0` so "actual size" is always reachable by stepping.
pub const ZOOM_LADDER: &[f32] = &[
    0.10, 0.25, 0.33, 0.50, 0.67, 0.75, 1.00, 1.25, 1.50, 2.00, 3.00, 4.00, 6.00, 8.00,
];

/// The next ladder rung strictly above `zoom`, or [`MAX_ZOOM`] if none
/// is (i.e. the caller is already at or past the top).
#[must_use]
pub fn ladder_step_up(zoom: f32) -> f32 {
    // `> zoom + epsilon` rather than `> zoom` so a value that is a
    // floating-point hair below a rung (0.9999999 vs 1.0) advances past
    // that rung instead of "stepping up" to a visually identical scale.
    let threshold = zoom + zoom.abs() * 1e-4;
    ZOOM_LADDER
        .iter()
        .copied()
        .find(|&rung| rung > threshold)
        // ★★★ PAST THE LADDER'S END, KEEP DOUBLING — O24.
        //
        // This returned `MAX_ZOOM` and therefore stalled at 800 %, whatever
        // the operator had configured. `zoom_ceiling` would honour a maximum
        // of 500,000 % and the `+` button would refuse to climb to it: the
        // setting honoured by every path except the one he actually uses,
        // which is the same silently-inert control in a subtler place.
        //
        // `OPERATOR_REQUESTS.md` O24 predicted this in its own words — *"the
        // buttons stop working exactly where the setting starts mattering"* —
        // and `the_zoom_ladder_can_climb_to_a_configured_maximum` caught it
        // before it shipped.
        //
        // ★ Doubling rather than continuing the hand-tuned 1-2-5 spacing. The
        // named rungs exist so ordinary zooms land on round percentages a
        // person recognises; past 800 % there are no round numbers left worth
        // hitting, and a constant ratio gives a constant NUMBER OF PRESSES per
        // decade — eleven from 800 % to a million — where a fixed increment
        // would need thousands.
        .unwrap_or_else(|| (zoom * 2.0).max(MAX_ZOOM))
}

/// The next ladder rung strictly below `zoom`, or [`MIN_ZOOM`] if none
/// is — **halving first**, above the ladder's end.
///
/// # ★★★ Why the search is not simply reversed
///
/// `OPERATOR_REQUESTS.md` **O24g**, 2026-08-22:
///
/// > *"clicking the negative button to zoom back snaps me back to 800% when I
/// > am over 800%."*
///
/// Exactly what a plain reverse search does. The ladder ends at 8.0, so from
/// 4,155 % the highest rung *below* is 8.00 — one press and a hundred-fold
/// magnification is gone. [`ladder_step_up`] had already grown the doubling
/// branch for the same reason on the way up; **only one half of the pair was
/// given it**, which made the two buttons stop being inverses of each other
/// exactly where the new range begins.
///
/// ★ That asymmetry is the defect, more than the snap itself. This module's
/// own header promises *"zoom-in/zoom-out exactly reversible"*, and a pair of
/// controls that disagree about what a step is breaks the one property an
/// operator relies on to explore without losing their place.
///
/// Halving mirrors the doubling above, so eleven presses out of a million
/// percent is eleven presses back in, and the last halving hands over to the
/// named rungs at the top of the ladder rather than jumping past them.
#[must_use]
pub fn ladder_step_down(zoom: f32) -> f32 {
    let threshold = zoom - zoom.abs() * 1e-4;
    // ★ Above the ladder's top rung, halve — but never below that rung, so the
    // descent lands ON the ladder and every press after it is a named
    // percentage. Without the clamp a zoom of 8.5 would halve to 4.25 and skip
    // the 800 %, 600 %, 400 % sequence entirely.
    let top = ZOOM_LADDER.last().copied().unwrap_or(MAX_ZOOM);
    if threshold > top {
        return (zoom / 2.0).max(top);
    }
    ZOOM_LADDER
        .iter()
        .copied()
        .rev()
        .find(|&rung| rung < threshold)
        .unwrap_or(MIN_ZOOM)
}

#[cfg(test)]
mod tests {
    use super::*;
    // ★ `ViewState` stays in the parent: it is the *state* the ladder is
    // applied to, not part of the ladder. One test drives a step through it
    // to check the clamp, which is the only coupling in either direction.
    use crate::viewer::ViewState;

    #[test]
    fn ladder_is_ascending_and_contains_actual_size() {
        assert!(ZOOM_LADDER.windows(2).all(|w| w[0] < w[1]));
        assert!(ZOOM_LADDER.contains(&1.0));
        assert_eq!(ZOOM_LADDER.first().copied(), Some(MIN_ZOOM));
        assert_eq!(ZOOM_LADDER.last().copied(), Some(MAX_ZOOM));
    }

    /// ★★★ O24g — **the two zoom buttons must be inverses of each other**,
    /// above the ladder as well as on it.
    ///
    /// The operator: *"clicking the negative button to zoom back snaps me back
    /// to 800% when I am over 800%."* `ladder_step_up` grew a doubling branch
    /// when the maximum zoom was raised; `ladder_step_down` did not, so from
    /// 4,155 % one press discarded a hundred-fold magnification.
    ///
    /// ★ Asserted as a ROUND TRIP rather than against fixed numbers. The
    /// property this module's header promises is reversibility, and a test of
    /// two constants would keep passing if both were changed together in a way
    /// that broke it.
    #[test]
    fn stepping_up_then_down_returns_to_where_it_started_above_the_ladder() {
        for start in [9.0_f32, 16.0, 41.55, 1_000.0, 20_000.0, 1e6] {
            let up = ladder_step_up(start);
            assert!(up > start, "{start} did not climb");
            let back = ladder_step_down(up);
            assert!(
                (back - start).abs() <= start * 1e-3,
                "{start} climbed to {up} and came back to {back}, not to {start}"
            );
        }
    }

    /// ★★ The descent must LAND ON the ladder, not vault over it.
    ///
    /// Halving from 8.5 gives 4.25, which is between two named rungs — so the
    /// next press down would go to 4.00 and the 600 % rung would never be
    /// reachable from above. Clamping the halving at the top rung hands the
    /// descent to the named percentages cleanly.
    #[test]
    fn descending_past_the_ladders_end_lands_on_its_top_rung() {
        let top = *ZOOM_LADDER.last().expect("a ladder");
        for start in [8.5_f32, 9.0, 12.0, 15.9] {
            assert_eq!(
                ladder_step_down(start),
                top,
                "{start} should descend onto the ladder's top rung"
            );
        }
        // …and from the rung itself, the named sequence resumes.
        assert!(ladder_step_down(top) < top);
    }

    /// ★ A zoom below the ladder's top is unaffected — the whole point of the
    /// branch is that it changes nothing an operator has ever seen before.
    #[test]
    fn the_named_rungs_are_untouched_by_the_halving_branch() {
        assert_eq!(ladder_step_down(8.0), 6.0);
        assert_eq!(ladder_step_down(1.0), 0.75);
        assert_eq!(ladder_step_down(0.10), MIN_ZOOM);
    }
    #[test]
    fn ladder_stepping_is_exactly_reversible() {
        // The property the fixed ladder exists to guarantee: in-then-out
        // returns to the same rung, for every rung.
        for &rung in ZOOM_LADDER {
            if rung < MAX_ZOOM {
                assert_eq!(ladder_step_down(ladder_step_up(rung)), rung);
            }
            if rung > MIN_ZOOM {
                assert_eq!(ladder_step_up(ladder_step_down(rung)), rung);
            }
        }
    }

    /// ★★ **Stepping DOWN saturates; stepping UP no longer does** — O24.
    ///
    /// This asserted `ladder_step_up(MAX_ZOOM) == MAX_ZOOM` from the day it was
    /// written until 2026-08-22, and it was right to: the ladder ended at 800 %
    /// and there was nowhere above it to go. With a configurable maximum there
    /// is, and saturating here would make the `+` button inert exactly where
    /// the setting starts mattering.
    ///
    /// ★ So the property changes shape rather than disappearing: **the step
    /// keeps climbing, and what stops it is the CEILING** — `ViewState::zoom_in`
    /// clamps against `zoom_ceiling`, which is where the limit belongs. A
    /// stepper that enforced its own maximum would be a second opinion about
    /// how far the operator may zoom.
    ///
    /// The downward half is unchanged: `MIN_ZOOM` is a floor with nothing
    /// below it, and 10 % of a page is not a number anybody has asked to go
    /// under.
    #[test]
    fn ladder_stepping_climbs_past_its_end_and_still_saturates_downward() {
        assert_eq!(ladder_step_up(MAX_ZOOM), MAX_ZOOM * 2.0);
        assert_eq!(ladder_step_up(999.0), 1998.0);
        assert_eq!(ladder_step_down(MIN_ZOOM), MIN_ZOOM);
        assert_eq!(ladder_step_down(0.001), MIN_ZOOM);

        // ★ And the ceiling is what actually stops a climb, not the ladder.
        let mut view = ViewState {
            zoom: MAX_ZOOM,
            ..ViewState::default()
        };
        view.zoom_in(MAX_ZOOM);
        assert_eq!(
            view.zoom, MAX_ZOOM,
            "with the ceiling at 800% the step must not exceed it"
        );
    }

    #[test]
    fn ladder_snaps_an_off_ladder_zoom_to_a_neighbouring_rung() {
        // Arriving from ctrl+scroll or a fit mode, 137% steps up to 150%
        // and down to 125% — never to 137.0001%.
        assert_eq!(ladder_step_up(1.37), 1.50);
        assert_eq!(ladder_step_down(1.37), 1.25);
    }

    #[test]
    fn a_hair_below_a_rung_still_steps_past_it() {
        // Guards the epsilon in ladder_step_up: without it, a fit scale
        // of 0.99999 would "step up" to 1.0, a visually identical zoom,
        // and the button would look broken.
        assert_eq!(ladder_step_up(0.999_99), 1.25);
        assert_eq!(ladder_step_down(1.000_01), 0.75);
    }
}
