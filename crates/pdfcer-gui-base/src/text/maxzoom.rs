//! # `text::maxzoom` — every word the maximum-zoom control says
//!
//! The strings for the popup behind the status bar's zoom readout —
//! `OPERATOR_REQUESTS.md` O24, and the operator's follow-up:
//!
//! > *"put the max zoom setting on the bar at the bottom."*
//!
//! ## Why the copy here is unusually plain
//!
//! Because his own sentence removed the thing this control would otherwise
//! have to hedge about:
//!
//! > *"I'm not concerned about the practicality of offering such a high zoom.
//! > it is up to the user to determine how much of a performance hit they want
//! > to take."*
//!
//! So there is **no warning, no "are you sure", and no advice**. A control that
//! nagged about a choice he has already made would be exactly the *"nagging and
//! red flagging"* he reported as a defect in the shell this one replaces.
//!
//! What is left is a statement of fact about where the crossover is, because
//! that is genuinely useful — it is the difference between panning being free
//! and panning costing a redraw — and it is not a judgement about what he
//! should pick.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/maxzoom.md`.

/// The heading over the popup.
#[must_use]
pub fn heading() -> &'static str {
    "Maximum zoom"
}

/// The one sentence of context, under the heading.
#[must_use]
pub fn crossover_note() -> &'static str {
    "Below about 1000% pdfcer draws the whole page, so panning is instant. \
     Above it, only what is on screen is drawn and panning redraws."
}

/// Hover text for the zoom readout, which opens this popup.
///
#[must_use]
pub fn readout_tooltip() -> &'static str {
    "The current zoom. The − and + buttons step a fixed ladder of familiar \
     percentages, so zooming in and back out returns you exactly where you \
     started. Click to set the maximum zoom."
}

/// One preset row's label, from its percentage.
#[must_use]
pub fn preset(percent: f32) -> String {
    let value = percent.round() as u64;
    let mut digits = value.to_string();
    let mut out = String::new();
    while digits.len() > 3 {
        let split = digits.len() - 3;
        out.insert_str(0, &format!(",{}", &digits[split..]));
        digits.truncate(split);
    }
    out.insert_str(0, &digits);
    out.push('%');
    out
}

/// The label on the row that is currently in force.
#[must_use]
pub fn current_suffix() -> &'static str {
    " (current)"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`f32` cannot hold a trillion exactly**, and the label says what is
    /// actually stored rather than what was asked for.
    #[test]
    fn the_top_preset_reads_as_what_is_actually_stored() {
        assert_eq!(
            preset(crate::viewer::ceiling::MAX_MAX_ZOOM_PERCENT),
            "999,999,995,904%"
        );
    }

    #[test]
    fn ordinary_percentages_are_grouped_and_plain() {
        assert_eq!(preset(800.0), "800%");
        assert_eq!(preset(5_000.0), "5,000%");
        assert_eq!(preset(1_000_000.0), "1,000,000%");
    }

    /// No exponent may reach a label, at any magnitude a preset can carry.
    #[test]
    fn no_preset_label_contains_an_exponent() {
        for p in [800.0_f32, 5_000.0, 100_000.0, 1_000_000.0, 1e9, 1e12] {
            let label = preset(p);
            assert!(!label.contains('e'), "{label} should not use an exponent");
        }
    }
}
