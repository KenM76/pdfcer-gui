//! # `app::status::maxzoom` — the maximum-zoom popup, behind the zoom readout
//!
//! `OPERATOR_REQUESTS.md` O24, and the operator on 2026-08-22:
//!
//! > *"put the max zoom setting on the bar at the bottom."*
//!
//! ## ★ Why the readout rather than a new control
//!
//! The status bar already has a zoom readout — a fixed 46 pt label showing the
//! current percentage, with a tooltip explaining the ladder. It is the one
//! control on the bar that is *about* zoom and does nothing when clicked, and a
//! label that turns out to be a button is the standard shape here: the page box
//! two groups along is a `TextEdit` that looks like a readout for the same
//! reason.
//!
//! Adding a separate control would have cost horizontal space on a bar whose
//! own module documents a fixed 30 pt height and a right-hand cluster that must
//! not move, to say something the readout is already the natural home for.
//!
//! ## ★★ It writes the preference itself, and the caller persists it
//!
//! Same seam as [`super::filter`]: this mutates a `Copy` value, and
//! [`crate::app::frame`] compares before and after and writes the file when it
//! moved. The argument is in that module's header — a comparison at the call
//! site cannot be forgotten by a future row added here, where a dirty flag can.
//!
//! ## What it is not
//!
//! Not a warning, not a confirmation, and not advice. The operator settled the
//! performance question in his own words — *"it is up to the user to determine
//! how much of a performance hit they want to take"* — so the popup states
//! where the crossover is and offers no opinion about it. See
//! [`crate::text::maxzoom`] for the copy and why it is that plain.

use crate::app::prefs::{MAX_MAX_ZOOM_PERCENT, MIN_MAX_ZOOM_PERCENT};
use crate::text::maxzoom as t;

/// The presets offered, ascending.
///
/// ★ **Ascending by powers of ten past 1,000 %**, because the useful question
/// at these magnitudes is *how many orders of magnitude*, not *how many
/// percent*. 800 % is included because it is where the shell used to stop and
/// is the honest "keep it as it was" choice; 1,000 % is roughly where whole-page
/// rasterizing gives out on a large sheet, which makes it the one preset with a
/// behavioural meaning rather than an arithmetic one.
///
/// The top entry is [`MAX_MAX_ZOOM_PERCENT`] rather than a literal `1e12`, so
/// the label says what is actually stored — see
/// [`crate::text::maxzoom::preset`]'s test on why that distinction is kept.
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
    // ★ Not `.strong()` — `tools/gates/check-strong-text.sh` rejects it, and
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

    /// ★★ **Every preset is inside the range the preference will accept.**
    ///
    /// A preset the parser would clamp is a row that silently does something
    /// other than what it says — the operator picks a billion and the file
    /// records something else, with nothing reporting the substitution.
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

    /// ★ **The shipped default is one of the rows**, so the popup always has a
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
