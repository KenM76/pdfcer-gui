//! # `prefs::modelview` — the 3D viewer's defaults
//!
//! Keys: every key beginning `model3d_`, each a word or number written by
//! *Save as default* or the Settings window and read back by the 3D viewer (see `pdfcer_gui::dialogs::model3d::memory`).
//! This module keeps them as text: what each means belongs to the viewer, so
//! a setting the viewer adds needs no change here. A value the viewer cannot
//! read falls back to that setting's default there.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::printing::KeyOutcome;

/// The prefix every key of this group carries.
pub const PREFIX: &str = "model3d_";

/// Claim `key = value` if it is this group's.
pub(super) fn parse_key(pref: &mut BTreeMap<String, String>, key: &str, value: &str) -> KeyOutcome {
    if !key.starts_with(PREFIX) {
        return KeyOutcome::NotMine;
    }
    let value = value.trim();
    if value.is_empty() || value.contains(['\n', '\r']) {
        return KeyOutcome::BadValue;
    }
    pref.insert(key.to_owned(), value.to_owned());
    KeyOutcome::Accepted
}

/// Append this group's block to a preferences file; nothing when empty.
pub(super) fn write_block(pref: &BTreeMap<String, String>, out: &mut String) {
    if pref.is_empty() {
        return;
    }
    out.push_str(
        "\n\
         # model3d_*: the 3D viewer's defaults, written by Save as default and\n\
         # the Settings window. Delete a line to restore the built-in value.\n",
    );
    for (key, value) in pref {
        let _ = writeln!(out, "{key} = {value}"); // ui-text-exempt: a preferences-file line, never displayed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_and_other_keys_are_left() {
        let mut map = BTreeMap::new();
        map.insert("model3d_stairs".to_owned(), "1".to_owned());
        map.insert("model3d_speed".to_owned(), "1.50".to_owned());
        let mut out = String::new();
        write_block(&map, &mut out);
        let mut back = BTreeMap::new();
        for line in out.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            let (key, value) = line.split_once(" = ").unwrap();
            assert!(matches!(
                parse_key(&mut back, key, value),
                KeyOutcome::Accepted
            ));
        }
        assert_eq!(back, map);
        assert!(matches!(
            parse_key(&mut back, "walk_eye_height_mm", "3"),
            KeyOutcome::NotMine
        ));
        assert!(matches!(
            parse_key(&mut back, "model3d_x", " "),
            KeyOutcome::BadValue
        ));
    }

    #[test]
    fn nothing_is_written_for_no_settings() {
        let mut out = String::new();
        write_block(&BTreeMap::new(), &mut out);
        assert!(out.is_empty());
    }
}
