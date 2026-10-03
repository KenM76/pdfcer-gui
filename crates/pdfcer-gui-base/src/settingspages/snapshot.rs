//! # `settingspages::snapshot` — the resolution a snapshot is copied at
//!
//! One control over [`crate::prefs::SnapshotPrefs`]. Its box is declared to
//! the trace as `settings.snapshot.dpi`, and the stored value is traced as
//! `snapshot-dpi-setting dpi=` whenever the page on show draws a new one.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/snapshot.md`.

use egui::Ui;

use super::widgets;
use crate::prefs::{Prefs, SnapshotPrefs};
use crate::text::settings::snapshot as t;

/// The resolution box.
// ui-text-exempt: trace region name, never displayed
pub const DPI_REGION: &str = "settings.snapshot.dpi";

/// The dpi a typed value means: a whole number, `dpi` allowed after it, clamped
/// to the offered range.
#[must_use]
pub fn parse_dpi(typed: &str) -> Option<u32> {
    let digits = typed.trim();
    // ui-text-exempt: the unit an operator may type after the number.
    let digits = digits.strip_suffix("dpi").unwrap_or(digits).trim();
    let dpi = digits.parse::<u32>().ok()?;
    Some(dpi.clamp(SnapshotPrefs::MIN_DPI, SnapshotPrefs::MAX_DPI))
}

/// The snapshot resolution setting.
pub fn dpi(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(ui, t::title(), t::silence(), t::radius());
    let mut dpi = prefs.snapshot.dpi();
    let note = t::range_note(
        SnapshotPrefs::MIN_DPI,
        SnapshotPrefs::MAX_DPI,
        SnapshotPrefs::DEFAULT_DPI,
    );
    let response = widgets::text_value(
        ui,
        // ui-text-exempt: an egui control id, never displayed.
        "settings-snapshot-dpi",
        &mut dpi,
        t::label(),
        Some(&note),
        u32::to_string,
        parse_dpi,
    );
    prefs.snapshot.set_dpi(dpi);
    // The search index draws every page invisibly; only the page on show
    // declares its box and traces its value.
    if ui.is_visible() && crate::diag::ui_rect_visible(DPI_REGION, response.rect, ui.clip_rect()) {
        let dpi = prefs.snapshot.dpi();
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace_on_change("snapshot-dpi-setting", || format!("dpi={dpi}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_typed_value_is_read_with_or_without_its_unit() {
        assert_eq!(parse_dpi("150"), Some(150));
        assert_eq!(parse_dpi(" 150 dpi "), Some(150));
        assert_eq!(parse_dpi("150dpi"), Some(150));
        assert_eq!(parse_dpi("sharp"), None);
    }

    #[test]
    fn a_typed_value_outside_the_range_is_clamped() {
        assert_eq!(parse_dpi("1"), Some(SnapshotPrefs::MIN_DPI));
        assert_eq!(parse_dpi("100000"), Some(SnapshotPrefs::MAX_DPI));
    }
}
