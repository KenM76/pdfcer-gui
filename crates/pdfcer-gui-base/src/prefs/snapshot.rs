//! # `prefs::snapshot` — how the snapshot box is copied
//!
//! Two keys, read by the snapshot box when it copies its region:
//! `snapshot_dpi`, the picture's resolution, and `snapshot_copy`, which forms
//! go on the clipboard. Out-of-range resolutions are clamped on read, not
//! refused, so a hand-edited file still opens with the nearest usable value.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/snapshot.md`.

use super::printing::KeyOutcome;

// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const KEY: &str = "snapshot_dpi";
// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const FORM_KEY: &str = "snapshot_copy";

/// Which forms a snapshot copy puts on the clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnapshotForm {
    /// Vectors with every character drawn as its outline, and the picture.
    /// Looks the same in every program; the text cannot be edited.
    #[default]
    Outlines,
    /// Vectors with each run that allows it kept as text, and the picture.
    /// The pasting program draws that text in its own installed font.
    KeepText,
    /// The picture alone, at the snapshot resolution.
    Picture,
}

impl SnapshotForm {
    /// Every form, in the order the setting offers them.
    pub const ALL: [Self; 3] = [Self::Outlines, Self::KeepText, Self::Picture];

    /// The value written to the file, and the trace token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Outlines => "outlines", // ui-text-exempt: a file value and trace token
            Self::KeepText => "text",     // ui-text-exempt: a file value and trace token
            Self::Picture => "picture",   // ui-text-exempt: a file value and trace token
        }
    }

    /// The form a file value names.
    #[must_use]
    pub fn from_token(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.token() == value)
    }
}

/// How the snapshot box is copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotPrefs {
    dpi: u32,
    form: SnapshotForm,
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

    /// Which forms a copy places.
    #[must_use]
    pub const fn form(&self) -> SnapshotForm {
        self.form
    }

    /// Store which forms a copy places.
    pub fn set_form(&mut self, form: SnapshotForm) {
        self.form = form;
    }
}

impl Default for SnapshotPrefs {
    fn default() -> Self {
        Self {
            dpi: Self::DEFAULT_DPI,
            form: SnapshotForm::default(),
        }
    }
}

/// Read one `key = value` line into [`SnapshotPrefs`], if it belongs here.
pub(super) fn parse_key(prefs: &mut SnapshotPrefs, key: &str, value: &str) -> KeyOutcome {
    match key {
        KEY => match value.trim().parse::<u32>() {
            Ok(dpi) => {
                prefs.set_dpi(dpi);
                KeyOutcome::Accepted
            }
            Err(_) => KeyOutcome::BadValue,
        },
        FORM_KEY => match SnapshotForm::from_token(value.trim()) {
            Some(form) => {
                prefs.set_form(form);
                KeyOutcome::Accepted
            }
            None => KeyOutcome::BadValue,
        },
        _ => KeyOutcome::NotMine,
    }
}

/// Write this group's commented block into the file.
pub(super) fn write_block(prefs: &SnapshotPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # How View > Snapshot copies its box:\n\
         #   snapshot_dpi: the picture's resolution, 36 to 2400 (default 300)\n\
         #   snapshot_copy: outlines (default), text (keep text as text) or\n\
         #                  picture (the picture alone)\n\
         ",
    );
    // ui-text-exempt: a file KEY and its separator, never displayed.
    out.push_str("snapshot_dpi = ");
    out.push_str(&prefs.dpi.to_string());
    // ui-text-exempt: a file KEY and its separator, never displayed.
    out.push_str("\nsnapshot_copy = ");
    out.push_str(prefs.form.token());
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
        for form in SnapshotForm::ALL {
            let mut original = SnapshotPrefs::default();
            original.set_dpi(150);
            original.set_form(form);
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
            parse_key(&mut prefs, FORM_KEY, "bitmap"),
            KeyOutcome::BadValue
        );
        assert_eq!(
            parse_key(&mut prefs, "export_image_dpi", "150"),
            KeyOutcome::NotMine
        );
        assert_eq!(prefs, SnapshotPrefs::default());
    }
}
