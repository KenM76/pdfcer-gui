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
///
/// # Nobody defines this — not the standard, and not the CAD vendors
///
/// The PDF standard has no view on dimensioning at all, which is a different
/// shape of silence from the twelve settings around it. More usefully: a search
/// of the SolidWorks dimension corpus for an epsilon, a threshold, or a
/// near-parallel snap rule **found none**, and the finding is recorded as
/// unverified rather than as an absence. So the operator would reasonably
/// assume CAD practice had settled it, and the silence line says it has not.
///
/// The shipped `0.5°` is a documented judgement: CAD-exported geometry is
/// usually exact, so a pair a hair off parallel is far more likely an exporter
/// rounding artefact than a deliberate shallow taper.
///
/// It exists as a setting because the operator asked for it on 2026-08-12 —
/// *"We should have an option in our settings and allow the user to set the
/// tolerance for nearly parallel lines"* — which is the standing ambiguity rule
/// applied by the person it exists for.
///
/// # NOT logarithmic, unlike the word-gap slider
///
/// The two sliders in this window differ, and the difference is deliberate. The
/// useful resolution here is **even across the range**: `0.5°` against `1.0°`
/// matters exactly as much as `5°` against `10°`, because both answer the same
/// question — *how wrong may this drawing be before I stop calling it
/// parallel?* A log scale would compress the upper half of a range where the
/// upper half is just as meaningful.
///
/// # Why zero is the floor rather than a rejected value
///
/// `0°` means *exactly parallel only*, which is a legitimate strict choice for
/// exact CAD output rather than a degenerate one. And `45°` is the ceiling
/// because above it the classification inverts in spirit — more pairs called
/// parallel than angled — which is a different feature, not a tolerance.
///
/// # The escape hatch is in the note, on purpose
///
/// An operator reading this control needs to know that a wrong global value is
/// a one-click per-dimension fix, not something they must come back here to
/// adjust. Without that sentence, the natural response to one bad
/// classification is to change a global default on the strength of a single
/// drawing.
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
