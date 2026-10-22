//! # `prefs::pictures` — how the page view draws a picture enlarged past its own pixels
//!
//! Key: `smooth_enlarged_pictures` (`true` | `false`, default `true`). On, a
//! picture shown larger than its own resolution — a scanned page zoomed in —
//! is smoothed between its pixels, as Acrobat draws it; off, each pixel is a
//! square block. Screen only: print and every export keep the engine's own
//! default, so a saved or printed file is never changed by it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/pictures.md`.

use std::fmt::Write as _;

use pdfcer_core::settings::MagnifyFilter;

use super::printing::KeyOutcome;

/// The file key.
const SMOOTH_KEY: &str = "smooth_enlarged_pictures";

/// The page view's picture preferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PicturePrefs {
    /// Smooth a picture drawn larger than its own pixels.
    pub smooth_enlarged: bool,
}

impl Default for PicturePrefs {
    fn default() -> Self {
        Self {
            smooth_enlarged: true,
        }
    }
}

impl PicturePrefs {
    /// The engine filter the canvas renders with.
    #[must_use]
    pub const fn magnify(&self) -> MagnifyFilter {
        if self.smooth_enlarged {
            MagnifyFilter::Smooth
        } else {
            MagnifyFilter::PointSample
        }
    }
}

/// Claim `key = value` if it is this group's.
pub(super) fn parse_key(prefs: &mut PicturePrefs, key: &str, value: &str) -> KeyOutcome {
    if key != SMOOTH_KEY {
        return KeyOutcome::NotMine;
    }
    match value.trim().parse::<bool>() {
        Ok(on) => {
            prefs.smooth_enlarged = on;
            KeyOutcome::Accepted
        }
        Err(_) => KeyOutcome::BadValue,
    }
}

/// Append this group's block to a preferences file.
pub(super) fn write_block(prefs: &PicturePrefs, out: &mut String) {
    out.push_str(
        "\n\
         # smooth_enlarged_pictures: true or false (default true). Smooths a\n\
         # picture shown larger than its own pixels, such as a zoomed-in scan.\n\
         # Screen only; printing and exports are not affected.\n",
    );
    let _ = writeln!(out, "{SMOOTH_KEY} = {}", prefs.smooth_enlarged); // ui-text-exempt: a preferences-file line, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_round_trips_and_a_bad_value_is_refused() {
        let mut prefs = PicturePrefs::default();
        assert_eq!(prefs.magnify(), MagnifyFilter::Smooth);
        let mut out = String::new();
        write_block(
            &PicturePrefs {
                smooth_enlarged: false,
            },
            &mut out,
        );
        let line = out.lines().last().unwrap();
        let (key, value) = line.split_once(" = ").unwrap();
        assert!(matches!(
            parse_key(&mut prefs, key, value),
            KeyOutcome::Accepted
        ));
        assert_eq!(prefs.magnify(), MagnifyFilter::PointSample);
        assert!(matches!(
            parse_key(&mut prefs, key, "maybe"),
            KeyOutcome::BadValue
        ));
        assert!(matches!(
            parse_key(&mut prefs, "other", "true"),
            KeyOutcome::NotMine
        ));
    }
}
