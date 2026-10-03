//! # `prefs::snapshot` — the resolution a snapshot is copied at
//!
//! One key, `snapshot_dpi`, read by the snapshot box when it copies its region
//! as a picture. Out-of-range values are clamped on read, not refused, so a
//! hand-edited file still opens with the nearest usable value.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/snapshot.md`.

use super::printing::KeyOutcome;

// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const KEY: &str = "snapshot_dpi";

/// The snapshot's picture resolution, in dots per inch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotPrefs {
    dpi: u32,
}

impl SnapshotPrefs {
    /// What a fresh profile copies at.
    pub const DEFAULT_DPI: u32 = 300;
    /// The lowest resolution offered: half a screen's.
    pub const MIN_DPI: u32 = 36;
    /// The highest resolution offered: a fine photo print's, doubled.
    pub const MAX_DPI: u32 = 2400;

    /// The resolution, always within `MIN_DPI..=MAX_DPI`.
    #[must_use]
    pub const fn dpi(&self) -> u32 {
        self.dpi
    }

    /// Store `dpi`, clamped to the offered range.
    pub fn set_dpi(&mut self, dpi: u32) {
        self.dpi = dpi.clamp(Self::MIN_DPI, Self::MAX_DPI);
    }
}

impl Default for SnapshotPrefs {
    fn default() -> Self {
        Self {
            dpi: Self::DEFAULT_DPI,
        }
    }
}

/// Read one `key = value` line into [`SnapshotPrefs`], if it belongs here.
pub(super) fn parse_key(prefs: &mut SnapshotPrefs, key: &str, value: &str) -> KeyOutcome {
    if key != KEY {
        return KeyOutcome::NotMine;
    }
    match value.trim().parse::<u32>() {
        Ok(dpi) => {
            prefs.set_dpi(dpi);
            KeyOutcome::Accepted
        }
        Err(_) => KeyOutcome::BadValue,
    }
}

/// Write this group's commented block into the file.
pub(super) fn write_block(prefs: &SnapshotPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # The resolution View > Snapshot copies its box at, as a picture:\n\
         #   snapshot_dpi: 36 to 2400 (default 300)\n\
         ",
    );
    // ui-text-exempt: a file KEY and its separator, never displayed.
    out.push_str("snapshot_dpi = ");
    out.push_str(&prefs.dpi.to_string());
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(prefs: &SnapshotPrefs) -> String {
        let mut out = String::new();
        write_block(prefs, &mut out);
        out
    }

    #[test]
    fn a_value_round_trips_through_the_block() {
        let mut original = SnapshotPrefs::default();
        original.set_dpi(150);
        let mut read_back = SnapshotPrefs::default();
        for line in block(&original).lines().filter(|l| !l.starts_with('#')) {
            if let Some((key, value)) = line.split_once('=') {
                assert_eq!(
                    parse_key(&mut read_back, key.trim(), value.trim()),
                    KeyOutcome::Accepted
                );
            }
        }
        assert_eq!(read_back, original);
    }

    #[test]
    fn an_out_of_range_value_is_clamped_not_refused() {
        let mut prefs = SnapshotPrefs::default();
        assert_eq!(parse_key(&mut prefs, KEY, "10"), KeyOutcome::Accepted);
        assert_eq!(prefs.dpi(), SnapshotPrefs::MIN_DPI);
        assert_eq!(parse_key(&mut prefs, KEY, "99999"), KeyOutcome::Accepted);
        assert_eq!(prefs.dpi(), SnapshotPrefs::MAX_DPI);
    }

    #[test]
    fn an_unreadable_value_is_ours_and_keeps_the_default() {
        let mut prefs = SnapshotPrefs::default();
        assert_eq!(parse_key(&mut prefs, KEY, "high"), KeyOutcome::BadValue);
        assert_eq!(
            parse_key(&mut prefs, "export_image_dpi", "150"),
            KeyOutcome::NotMine
        );
        assert_eq!(prefs, SnapshotPrefs::default());
    }
}
