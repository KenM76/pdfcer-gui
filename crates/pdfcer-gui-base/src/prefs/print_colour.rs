//! The Print window's colour-management answers — the printer's ICC profile,
//! a press condition to simulate, the rendering intent, black point
//! compensation and paper simulation — and their lines in `preferences.txt`.
//! A group of [`super::printing::PrintPrefs`]; its parser and writer are
//! called from that group's.
//!
//! A profile is stored by its file path, never its description: a path names
//! one file, a description need not, and a description may carry `=`.

use pdfcer_print::icc_output::RenderingIntent;

use super::printing::{KeyOutcome, PrintPrefs};

/// The remembered colour-management settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintColourPrefs {
    /// The printer's RGB or grey profile; `None` sends sRGB unconverted.
    pub printer_profile: Option<String>,
    /// The press condition simulated on the printer; `None` simulates nothing.
    pub proof_profile: Option<String>,
    /// The intent of the conversion into the printer profile, or of the press
    /// leg when a condition is simulated.
    pub intent: RenderingIntent,
    /// Black point compensation on that same conversion.
    pub black_point_compensation: bool,
    /// Print the simulated paper's tint and black rather than the printer's.
    pub simulate_paper: bool,
}

impl Default for PrintColourPrefs {
    fn default() -> Self {
        Self {
            printer_profile: None,
            proof_profile: None,
            intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: true,
            simulate_paper: false,
        }
    }
}

impl PrintColourPrefs {
    /// Whether any profile is chosen, so sheets are converted.
    #[must_use]
    pub fn managed(&self) -> bool {
        self.printer_profile.is_some() || self.proof_profile.is_some()
    }
}

/// Every intent, in the order offered, with its file token.
// ui-text-exempt: file tokens, never displayed.
pub const INTENTS: [(RenderingIntent, &str); 4] = [
    (RenderingIntent::RelativeColorimetric, "relative"),
    (RenderingIntent::Perceptual, "perceptual"),
    (RenderingIntent::Saturation, "saturation"),
    (RenderingIntent::AbsoluteColorimetric, "absolute"),
];

/// The file token for `intent`.
#[must_use]
pub fn intent_key(intent: RenderingIntent) -> &'static str {
    INTENTS
        .iter()
        .find(|(i, _)| *i == intent)
        .map_or("relative", |(_, t)| t) // ui-text-exempt: file token
}

/// A path value, or `None` for an empty one.
fn path_from(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

/// Read one line into [`PrintColourPrefs`], if it is one of this group's.
pub(super) fn parse_key(prefs: &mut PrintPrefs, key: &str, value: &str) -> KeyOutcome {
    let p = &mut prefs.colour;
    let bool_from = super::opening::bool_from_key;
    let stored = match key {
        // ui-text-exempt: file KEYS, parsed out of preferences.txt.
        "print_printer_profile" => {
            p.printer_profile = path_from(value);
            Some(())
        }
        "print_proof_profile" => {
            p.proof_profile = path_from(value);
            Some(())
        }
        "print_colour_intent" => INTENTS
            .iter()
            .find(|(_, t)| *t == value)
            .map(|(i, _)| p.intent = *i),
        "print_colour_bpc" => bool_from(value).map(|v| p.black_point_compensation = v),
        "print_simulate_paper" => bool_from(value).map(|v| p.simulate_paper = v),
        _ => return super::print_edges::parse_key(prefs, key, value),
    };
    if stored.is_some() {
        KeyOutcome::Accepted
    } else {
        KeyOutcome::BadValue
    }
}

/// A path as a value the file can hold on one line: a line break would end
/// it, so such a path is not remembered.
fn path_value(path: Option<&String>) -> String {
    path.filter(|p| !p.contains(['\n', '\r']))
        .cloned()
        .unwrap_or_default()
}

/// Append this group's lines.
pub(super) fn write_block(prefs: &PrintPrefs, out: &mut String) {
    out.push_str(
        "# print_printer_profile: the printer's ICC profile file, or empty to\n\
         # send colour unconverted.\n\
         # print_proof_profile: an ICC profile whose press or paper is simulated\n\
         # on the printer, or empty.\n\
         # print_colour_intent: relative | perceptual | saturation | absolute.\n\
         # print_colour_bpc, print_simulate_paper: true | false.\n",
    );
    let p = &prefs.colour;
    let bool_key = super::opening::bool_key;
    // ui-text-exempt: file KEYS, as above.
    for (key, value) in [
        (
            "print_printer_profile",
            path_value(p.printer_profile.as_ref()),
        ),
        ("print_proof_profile", path_value(p.proof_profile.as_ref())),
        ("print_colour_intent", intent_key(p.intent).to_owned()),
        (
            "print_colour_bpc",
            bool_key(p.black_point_compensation).to_owned(),
        ),
        (
            "print_simulate_paper",
            bool_key(p.simulate_paper).to_owned(),
        ),
    ] {
        out.push_str(key);
        out.push_str(" = "); // ui-text-exempt: the file's key-value separator.
        out.push_str(&value);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(prefs: &PrintPrefs) -> PrintPrefs {
        let mut out = String::new();
        write_block(prefs, &mut out);
        let mut back = PrintPrefs::default();
        for line in out.lines().filter(|l| !l.starts_with('#')) {
            let (k, v) = line.split_once('=').expect("key = value");
            assert_eq!(
                parse_key(&mut back, k.trim(), v.trim()),
                KeyOutcome::Accepted,
                "{line}"
            );
        }
        back
    }

    #[test]
    fn every_setting_round_trips_through_the_file() {
        let prefs = PrintPrefs {
            colour: PrintColourPrefs {
                printer_profile: Some(r"C:\Windows\System32\spool\drivers\color\desk.icm".into()),
                proof_profile: Some(r"D:\profiles\CoatedFOGRA39.icc".into()),
                intent: RenderingIntent::AbsoluteColorimetric,
                black_point_compensation: false,
                simulate_paper: true,
            },
            ..PrintPrefs::default()
        };
        assert_eq!(round_trip(&prefs).colour, prefs.colour);
    }

    #[test]
    fn no_profile_round_trips_as_none() {
        let prefs = PrintPrefs::default();
        let back = round_trip(&prefs);
        assert_eq!(back.colour, PrintColourPrefs::default());
        assert!(!back.colour.managed());
    }

    #[test]
    fn an_intent_not_offered_is_refused() {
        let mut prefs = PrintPrefs::default();
        assert_eq!(
            parse_key(&mut prefs, "print_colour_intent", "vivid"),
            KeyOutcome::BadValue
        );
        assert_eq!(prefs.colour.intent, RenderingIntent::RelativeColorimetric);
    }
}
