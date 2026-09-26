//! # `dialogs::settings::measuring` — the group the source did not have
//!
//! One setting: the angular tolerance below which two lines are dimensioned as
//! a **distance** rather than as an **angle**.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/measuring.md`.

use egui::Ui;
use pdfcer_core::settings::{MAX_PARALLEL_EPSILON_DEGREES, MIN_PARALLEL_EPSILON_DEGREES};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How close to parallel counts as parallel.
pub fn parallel(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::parallel_title(),
        t::parallel_silence(),
        t::parallel_radius(),
    );
    ui.add(
        egui::Slider::new(
            &mut draft.working.parallel_epsilon_degrees,
            MIN_PARALLEL_EPSILON_DEGREES..=MAX_PARALLEL_EPSILON_DEGREES,
        )
        .suffix(t::degree_suffix())
        .text(t::parallel_slider_label()),
    );
    ui.label(egui::RichText::new(t::parallel_note()).small().weak());
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::settings::{SettingNote, Settings};

    /// **The slider's range is the STORE's, and a hand-edited legal value
    /// survives opening this window.**
    #[test]
    fn a_hand_edited_value_inside_the_stores_range_is_not_rewritten() {
        // Well inside the store's range and well outside any "usable band" a
        // designer would pick: 30° is what the first attempt at this control
        // would have clamped to 1.0.
        let mut notes = Vec::new();
        let parsed = Settings::parse(
            "parallel_epsilon_degrees = 30.0
",
            &mut notes,
        );
        assert!(
            notes.is_empty(),
            "the store clamped a value this window offers: {notes:?}"
        );
        assert!(
            (parsed.parallel_epsilon_degrees - 30.0).abs() < f64::EPSILON,
            "the store did not keep the hand-edited value"
        );
        // …and the slider must be able to represent it, or opening the window
        // is what would rewrite it.
        assert!(
            (MIN_PARALLEL_EPSILON_DEGREES..=MAX_PARALLEL_EPSILON_DEGREES)
                .contains(&parsed.parallel_epsilon_degrees),
            "the slider cannot represent a value the file legally holds"
        );
    }

    /// The store clamps beyond its own range, and says so — so the ceiling is
    /// real rather than decorative.
    #[test]
    fn the_store_clamps_beyond_its_range_and_discloses_it() {
        let mut notes = Vec::new();
        let parsed = Settings::parse(
            "parallel_epsilon_degrees = 400.0
",
            &mut notes,
        );
        assert!(
            notes
                .iter()
                .any(|n| matches!(n, SettingNote::Clamped { .. })),
            "an out-of-range tolerance was accepted silently: {notes:?}"
        );
        assert!(
            (parsed.parallel_epsilon_degrees - MAX_PARALLEL_EPSILON_DEGREES).abs() < f64::EPSILON,
            "the clamp did not land on the range's own ceiling"
        );
    }

    /// The shipped default sits inside the offered range.
    #[test]
    fn the_shipped_default_is_reachable_on_the_slider() {
        let default = Settings::default().parallel_epsilon_degrees;
        assert!(
            (MIN_PARALLEL_EPSILON_DEGREES..=MAX_PARALLEL_EPSILON_DEGREES).contains(&default),
            "the shipped tolerance {default} is outside the slider's range"
        );
    }
}
