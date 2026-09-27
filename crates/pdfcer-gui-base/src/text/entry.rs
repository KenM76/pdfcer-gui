//! # `text::entry` — what a value box says when it cannot read what was typed
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/entry.md`.

use crate::entry::EntryError;

/// One sentence naming what is wrong with the typed text.
#[must_use]
pub fn describe(error: &EntryError) -> String {
    match error {
        EntryError::Empty => "Type a value.".to_owned(),
        EntryError::Unexpected(at) => format!("Could not read this from character {}.", at + 1),
        EntryError::Incomplete => "The value stops before it is finished.".to_owned(),
        EntryError::UnknownUnit(word) => {
            format!("\u{201C}{word}\u{201D} is not a unit pdfcer knows.")
        }
        EntryError::UnitNotAllowed(word) => {
            format!("This box holds a plain number, so \u{201C}{word}\u{201D} cannot go in it.")
        }
        EntryError::Mismatch => {
            "That mixes a length and a plain number in a way that has no meaning.".to_owned()
        }
        EntryError::NotFinite => "That divides by zero.".to_owned(),
        EntryError::NotWhole => "This box holds a whole number.".to_owned(),
    }
}

/// Hover help for a length box: what it accepts.
#[must_use]
pub const fn length_help() -> &'static str {
    "Type a length in any unit (12 mm, 1/2\", 4'-6\", 10 px) and arithmetic \
     (12mm + 1/4\"). Start with + - * or / to change the current value: \
     \u{201C}+10px\u{201D} adds ten pixels."
}

/// Hover help for a plain-number box.
#[must_use]
pub const fn number_help() -> &'static str {
    "Arithmetic works here (3 * 25). Start with + - * or / to change the \
     current value: \u{201C}*2\u{201D} doubles it."
}

/// The unit shown after a length box's value: `12.00 mm`.
#[must_use]
pub fn unit_suffix(label: &str) -> String {
    format!(" {label}")
}
