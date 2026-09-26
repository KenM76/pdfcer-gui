//! # `viewer::ladder` — the zoom levels the `+` and `−` buttons step through
//!
//! One subject: **given a zoom, what is the next one up or down?** Split out
//! of [`super`] under R2 when that file reached 1,540 lines, and the seam is a
//! real one — everything here answers that question and nothing here knows
//! what a page, a viewport or a raster is.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/viewer/ladder.md`.

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
        // PAST THE LADDER'S END, KEEP DOUBLING — O24.
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
        // Doubling rather than continuing the hand-tuned 1-2-5 spacing. The
        // named rungs exist so ordinary zooms land on round percentages a
        // person recognises; past 800 % there are no round numbers left worth
        // hitting, and a constant ratio gives a constant NUMBER OF PRESSES per
        // decade — eleven from 800 % to a million — where a fixed increment
        // would need thousands.
        .unwrap_or_else(|| (zoom * 2.0).max(MAX_ZOOM))
}

/// The next ladder rung strictly below `zoom`, or [`MIN_ZOOM`] if none
/// is — **halving first**, above the ladder's end.
#[must_use]
pub fn ladder_step_down(zoom: f32) -> f32 {
    let threshold = zoom - zoom.abs() * 1e-4;
    // Above the ladder's top rung, halve — but never below that rung, so the
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
    // `ViewState` stays in the parent: it is the *state* the ladder is
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

    /// O24g — **the two zoom buttons must be inverses of each other**,
    /// above the ladder as well as on it.
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

    /// The descent must LAND ON the ladder, not vault over it.
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

    /// A zoom below the ladder's top is unaffected — the whole point of the
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

    /// **Stepping DOWN saturates; stepping UP no longer does** — O24.
    #[test]
    fn ladder_stepping_climbs_past_its_end_and_still_saturates_downward() {
        assert_eq!(ladder_step_up(MAX_ZOOM), MAX_ZOOM * 2.0);
        assert_eq!(ladder_step_up(999.0), 1998.0);
        assert_eq!(ladder_step_down(MIN_ZOOM), MIN_ZOOM);
        assert_eq!(ladder_step_down(0.001), MIN_ZOOM);

        // And the ceiling is what actually stops a climb, not the ladder.
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
