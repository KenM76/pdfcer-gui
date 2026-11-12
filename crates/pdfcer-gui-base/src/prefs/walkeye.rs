//! # `prefs::walkeye` — the 3D walker's eye height
//!
//! Key: `walk_eye_mm` (`auto` | whole millimetres, default 1676, 5 ft 6 in).
//! Set in the 3D viewer's feet-and-inches fields while walking with ghost
//! mode off; `auto` lets the walker's height follow the model's size.
//! The older key `walk_eye_height_mm`, whose default was `auto`, is read for
//! a height only: its `auto` is the old default and gives way to the new.
//! Millimetres because the height is a real-world length: each model's own
//! unit turns it into model units when it is used.

use std::fmt::Write as _;

use super::printing::KeyOutcome;

/// The file key.
const KEY: &str = "walk_eye_mm";
/// The older key, read for a set height only.
const OLD_KEY: &str = "walk_eye_height_mm";
/// The default eye height, millimetres: 5 ft 6 in.
pub const DEFAULT_MM: u32 = 1676;

/// The token for no set height.
const AUTO: &str = "auto";

/// Claim `key = value` if it is this group's. Zero is refused: a height
/// is above the floor.
pub(super) fn parse_key(pref: &mut Option<u32>, key: &str, value: &str) -> KeyOutcome {
    if key != KEY && key != OLD_KEY {
        return KeyOutcome::NotMine;
    }
    let value = value.trim();
    if key == OLD_KEY && value == AUTO {
        return KeyOutcome::Accepted;
    }
    if value == AUTO {
        *pref = None;
        return KeyOutcome::Accepted;
    }
    match value.parse::<u32>() {
        Ok(mm) if mm > 0 => {
            *pref = Some(mm);
            KeyOutcome::Accepted
        }
        _ => KeyOutcome::BadValue,
    }
}

/// Append this group's block to a preferences file.
pub(super) fn write_block(pref: Option<u32>, out: &mut String) {
    out.push_str(
        "\n\
         # walk_eye_mm: auto or whole millimetres (default 1676, 5 ft 6 in). The\n\
         # 3D walker's eye height, set in the 3D viewer with ghost mode off;\n\
         # auto follows the model's size.\n",
    );
    let value = pref.map_or_else(|| AUTO.to_owned(), |mm| mm.to_string());
    let _ = writeln!(out, "{KEY} = {value}"); // ui-text-exempt: a preferences-file line, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_height_and_auto_round_trip() {
        for choice in [None, Some(1676)] {
            let mut out = String::new();
            write_block(choice, &mut out);
            let (key, value) = out.lines().last().unwrap().split_once(" = ").unwrap();
            let mut back = Some(1);
            assert!(matches!(
                parse_key(&mut back, key, value),
                KeyOutcome::Accepted
            ));
            assert_eq!(back, choice);
        }
    }

    #[test]
    fn zero_and_words_are_refused_and_other_keys_ignored() {
        let mut pref = None;
        for bad in ["0", "-3", "tall", "1.5"] {
            assert!(matches!(
                parse_key(&mut pref, KEY, bad),
                KeyOutcome::BadValue
            ));
        }
        assert!(matches!(
            parse_key(&mut pref, "other", "10"),
            KeyOutcome::NotMine
        ));
        assert_eq!(pref, None);
    }

    #[test]
    fn the_older_keys_auto_keeps_the_default_and_its_height_is_taken() {
        let mut pref = Some(DEFAULT_MM);
        assert!(matches!(
            parse_key(&mut pref, OLD_KEY, "auto"),
            KeyOutcome::Accepted
        ));
        assert_eq!(pref, Some(DEFAULT_MM));
        assert!(matches!(
            parse_key(&mut pref, OLD_KEY, "1500"),
            KeyOutcome::Accepted
        ));
        assert_eq!(pref, Some(1500));
    }
}
