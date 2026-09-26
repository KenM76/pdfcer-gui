//! # `ocrlayerpref` — the colour the recognised text is drawn in
//!
//! One preference, `OPERATOR_REQUESTS.md` **O229**: *"we should be able to
//! change the editing colour of the ocr text layer just for editing, and
//! remember the user's setting."* This file holds the notation the preferences
//! file uses for it, in the shape [`super::fonts`] holds its own.
//!
//! ## Why the default is not written here
//!
//! `pdfcer_gui::canvas::ocrlayer::DEFAULT_COLOUR` is the one home, and the
//! painter's module is the right one because the *reason* for the value is a
//! rendering argument — magenta is a colour a scanned drawing is unlikely to
//! contain. A second literal here would be a number that drifts from the
//! sentence justifying it.
//!
//! ## Why hex, and why a bad value is reported rather than replaced
//!
//! `#CC0099` is what a colour is called everywhere an operator has met one,
//! and it round-trips through a text file with no separator question. Three
//! comma-separated numbers would need a rule for whitespace, a rule for a
//! fourth number, and a rule for `300`.
//!
//! A value this cannot read keeps the operator's **previous** colour and
//! raises a note naming the line, which is [`super::file`]'s standing posture:
//! someone who hand-edited a colour and typed it wrongly was trying to change
//! it, and silently substituting the default would be the one outcome they did
//! not ask for.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocrlayerpref.md`.

/// Read a colour out of the preferences file.
#[must_use]
pub fn parse(value: &str) -> Option<[u8; 3]> {
    let digits = value.trim().trim_start_matches('#');
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let pair = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).ok();
    match digits.len() {
        6 => Some([pair(0)?, pair(2)?, pair(4)?]),
        // Doubled, not shifted: `#F80` is `#FF8800`, so the shorthand's
        // brightest digit stays the brightest byte. A shift would map `F` to
        // `F0` and quietly darken every colour written this way.
        3 => {
            let nibble = |at: usize| {
                u8::from_str_radix(&digits[at..=at], 16)
                    .ok()
                    .map(|n| n * 17)
            };
            Some([nibble(0)?, nibble(1)?, nibble(2)?])
        }
        _ => None,
    }
}

/// Write a colour for the preferences file, in the notation [`parse`] reads.
///
/// Upper case, because the file is read by eye and `#CC0099` is easier to
/// compare against a swatch than `#cc0099`.
#[must_use]
pub fn format(rgb: [u8; 3]) -> String {
    let [r, g, b] = rgb;
    format!("#{r:02X}{g:02X}{b:02X}")
}
