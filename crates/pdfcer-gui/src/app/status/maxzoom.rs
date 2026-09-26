//! # `app::status::maxzoom` — the maximum-zoom popup, behind the zoom readout
//!
//!
//! > *"put the max zoom setting on the bar at the bottom."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/maxzoom.md`.

use crate::app::prefs::{MAX_MAX_ZOOM_PERCENT, MIN_MAX_ZOOM_PERCENT};
use crate::text::maxzoom as t;

/// The presets offered, ascending.
const PRESETS: [f32; 6] = [
    800.0,
    1_000.0,
    100_000.0,
    1_000_000.0,
    1_000_000_000.0,
    MAX_MAX_ZOOM_PERCENT,
];

/// Draw the popup's body into a `Ui` the caller has opened.
///
/// Returns nothing; the preference is written in place and the caller decides
/// whether it moved.
pub(super) fn popup(ui: &mut egui::Ui, max_zoom_percent: &mut f32) {
    // Not `.strong()` — `tools/gates/check-strong-text.sh` rejects it, and
    // defect D11 is why: egui resolves it to the accent-filled widget state,
    // which is pale text on a pale background. The hierarchy is position and
    // the separator beneath.
    ui.label(t::heading());
    ui.separator();
    ui.label(t::crossover_note());
    ui.separator();

    for (index, percent) in PRESETS.into_iter().enumerate() {
        let current = (*max_zoom_percent - percent).abs() < f32::EPSILON;
        let label = if current {
            format!("{}{}", t::preset(percent), t::current_suffix())
        } else {
            t::preset(percent)
        };
        let row = ui.selectable_label(current, label);
        // Published per row, keyed by INDEX rather than by label: labels are
        // operator copy and get reworded, an index is stable, and a harness is
        // choosing positionally anyway.
        crate::diag::ui_rect(&format!("{}:{index}", super::REGION_MAXZOOM_ROW), row.rect);
        if row.clicked() {
            *max_zoom_percent = percent.clamp(MIN_MAX_ZOOM_PERCENT, MAX_MAX_ZOOM_PERCENT);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "max-zoom-set percent={percent}"
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every preset is inside the range the preference will accept.**
    #[test]
    fn every_preset_is_a_value_the_preference_accepts() {
        for percent in PRESETS {
            assert!(
                (MIN_MAX_ZOOM_PERCENT..=MAX_MAX_ZOOM_PERCENT).contains(&percent),
                "{percent} is outside the accepted range"
            );
            assert_eq!(
                percent.clamp(MIN_MAX_ZOOM_PERCENT, MAX_MAX_ZOOM_PERCENT),
                percent,
                "{percent} would be clamped, so the row would lie"
            );
        }
    }

    /// The list ascends, so the popup reads as a scale rather than a set.
    #[test]
    fn the_presets_ascend() {
        for pair in PRESETS.windows(2) {
            assert!(pair[0] < pair[1], "{:?} is not ascending", pair);
        }
    }

    /// **The shipped default is one of the rows**, so the popup always has a
    /// current selection to show. Without this a fresh install would open the
    /// popup with nothing marked, which reads as "no maximum is set".
    #[test]
    fn the_default_is_one_of_the_presets() {
        assert!(
            PRESETS
                .iter()
                .any(|p| (*p - crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT).abs() < f32::EPSILON),
            "the default {} is not offered as a preset",
            crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT
        );
    }
}
