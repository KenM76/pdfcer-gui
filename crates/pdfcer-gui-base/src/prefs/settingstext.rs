//! # `prefs::settingstext` — how large the Settings window draws its text
//!
//! Key: `settings_text_scale`, a multiplier on the program's own size, 0.8 to
//! 2.5 in steps of 0.05, default 1.25. It applies to the Settings window
//! alone, on top of `ui_scale`, and is written the moment it changes rather
//! than on Save: it is how the window is read, not something it configures.

use std::fmt::Write as _;

use super::printing::KeyOutcome;

/// The file key.
const KEY: &str = "settings_text_scale";

/// The smallest scale offered.
pub const MIN: f32 = 0.8;
/// The largest scale offered.
pub const MAX: f32 = 2.5;
/// The shipped scale: the Settings window reads larger than the program.
pub const DEFAULT: f32 = 1.25;
/// The step the slider and each wheel notch move in.
pub const STEP: f32 = 0.05;

/// Round `raw` to a [`STEP`] inside [`MIN`]..=[`MAX`]; the default for a
/// value that is not a number.
#[must_use]
pub fn normalise(raw: f32) -> f32 {
    if !raw.is_finite() {
        return DEFAULT;
    }
    (raw.clamp(MIN, MAX) / STEP).round() * STEP
}

/// Claim `key = value` if it is this group's.
pub(super) fn parse_key(pref: &mut f32, key: &str, value: &str) -> KeyOutcome {
    if key != KEY {
        return KeyOutcome::NotMine;
    }
    match value.trim().parse::<f32>() {
        Ok(raw) if raw.is_finite() => {
            *pref = normalise(raw);
            KeyOutcome::Accepted
        }
        _ => KeyOutcome::BadValue,
    }
}

/// Append this group's block to a preferences file.
pub(super) fn write_block(pref: f32, out: &mut String) {
    out.push_str(
        "\n\
         # settings_text_scale: 0.8 to 2.5 in steps of 0.05 (default 1.25).\n\
         # How large the Settings window draws its text, on top of ui_scale.\n\
         # Set by the slider beside its search box, or Ctrl+wheel in it.\n",
    );
    let _ = writeln!(out, "{KEY} = {pref:.2}"); // ui-text-exempt: a preferences-file line, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scale_round_trips() {
        for scale in [MIN, DEFAULT, 1.6, MAX] {
            let mut out = String::new();
            write_block(scale, &mut out);
            let (key, value) = out.lines().last().unwrap().split_once(" = ").unwrap();
            let mut back = 0.0;
            assert!(matches!(
                parse_key(&mut back, key, value),
                KeyOutcome::Accepted
            ));
            assert!((back - scale).abs() < 1e-5, "{scale} read back {back}");
        }
    }

    #[test]
    fn out_of_range_is_clamped_and_words_refused() {
        let mut pref = DEFAULT;
        assert!(matches!(
            parse_key(&mut pref, KEY, "9"),
            KeyOutcome::Accepted
        ));
        assert!((pref - MAX).abs() < 1e-5);
        for bad in ["big", "nan", "inf"] {
            assert!(matches!(
                parse_key(&mut pref, KEY, bad),
                KeyOutcome::BadValue
            ));
        }
        assert!(matches!(
            parse_key(&mut pref, "other", "1"),
            KeyOutcome::NotMine
        ));
        assert!((normalise(1.234) - 1.25).abs() < 1e-5);
    }
}
