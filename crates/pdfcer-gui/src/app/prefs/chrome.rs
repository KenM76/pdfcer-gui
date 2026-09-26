//! # `app::prefs::chrome` — how big the program's own controls are drawn
//!
//! One preference, and it is the only one in this store that is an
//! **accessibility** control rather than a taste or a speed trade.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/prefs/chrome.md`.

/// The smallest scale offered.
pub const MIN_UI_SCALE: f32 = 0.8;

/// The largest scale offered.
pub const MAX_UI_SCALE: f32 = 2.0;

/// The shipped scale.
pub const DEFAULT_UI_SCALE: f32 = 1.0;

/// The step the control moves in.
pub const UI_SCALE_STEP: f32 = 0.05;

/// Round a scale to the nearest [`UI_SCALE_STEP`] and clamp it to the offered
/// range.
#[must_use]
pub fn normalise_ui_scale(raw: f32) -> f32 {
    let clamped = raw.clamp(MIN_UI_SCALE, MAX_UI_SCALE);
    (clamped / UI_SCALE_STEP).round() * UI_SCALE_STEP
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped scale is the identity, and it is reachable on its control.
    #[test]
    fn the_shipped_scale_is_the_identity_and_is_reachable() {
        assert!((DEFAULT_UI_SCALE - 1.0).abs() < f32::EPSILON);
        assert!((MIN_UI_SCALE..=MAX_UI_SCALE).contains(&DEFAULT_UI_SCALE));
        assert!(
            (normalise_ui_scale(DEFAULT_UI_SCALE) - DEFAULT_UI_SCALE).abs() < 1e-6,
            "the shipped scale is not on the control's own step"
        );
    }

    /// Normalising is idempotent.
    #[test]
    fn normalising_twice_changes_nothing() {
        let mut value = MIN_UI_SCALE;
        while value <= MAX_UI_SCALE {
            let once = normalise_ui_scale(value);
            let twice = normalise_ui_scale(once);
            assert!(
                (once - twice).abs() < 1e-6,
                "{value} normalised to {once} and then to {twice}"
            );
            value += UI_SCALE_STEP / 3.0;
        }
    }

    /// Out-of-range values are pulled to the ends, in both directions.
    #[test]
    fn an_out_of_range_scale_clamps_to_the_offered_range() {
        assert!((normalise_ui_scale(0.1) - MIN_UI_SCALE).abs() < 1e-6);
        assert!((normalise_ui_scale(99.0) - MAX_UI_SCALE).abs() < 1e-6);
    }

    /// Every value the control can produce survives normalising unchanged.
    #[test]
    fn every_value_the_control_offers_round_trips() {
        let steps = ((MAX_UI_SCALE - MIN_UI_SCALE) / UI_SCALE_STEP).round() as i32;
        for i in 0..=steps {
            #[allow(clippy::cast_precision_loss)]
            let value = UI_SCALE_STEP.mul_add(i as f32, MIN_UI_SCALE);
            let back = normalise_ui_scale(value);
            assert!(
                (value - back).abs() < 1e-5,
                "the control can produce {value}, which the loader rewrites to {back}"
            );
        }
    }
}
