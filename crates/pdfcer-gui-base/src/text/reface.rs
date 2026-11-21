//! # `text::reface` — what a text edit says when some of its characters were
//! set in another face because the line's own font cannot write them
//!
//! Worded here; raised by `pdfcer_gui::app::actions::reface` and, for
//! [`undo_split`], `pdfcer_gui::app::actions::replace`.

use super::refusedkeys::list;

/// The disclosure on a committed edit: which characters, and which face.
///
/// Those in `two_ways` (`RunRepertoire::ambiguous`) the font has and draws
/// with more than one code; the rest it cannot write for a reason the
/// repertoire does not name, so they are said as *cannot write*, not *lacks*.
#[must_use]
pub fn set_in(chars: &[char], two_ways: &[char], face: &str) -> String {
    let what = if chars.len() == 1 { "it" } else { "them" };
    let (doubled, other): (Vec<char>, Vec<char>) = chars.iter().partition(|c| two_ways.contains(c));
    let why = match (other.is_empty(), doubled.is_empty()) {
        (_, true) => format!(
            "pdfcer cannot write {} in this text's own font",
            list(&other)
        ),
        (true, false) => format!(
            "This text's font draws {} two different ways and pdfcer will not choose one",
            list(&doubled)
        ),
        (false, false) => format!(
            "pdfcer cannot write {} in this text's own font, which also draws {} two different ways",
            list(&other),
            list(&doubled)
        ),
    };
    format!("{why}, so it set {what} in {face}, the nearest font that has {what}.")
}

/// Said when the fallback face is a standard font the file does not embed.
#[must_use]
pub fn not_embedded(face: &str) -> String {
    format!("{face} is not embedded, so another program may show these in a similar font.")
}

/// Said when a gesture's steps could not be joined into one undo.
#[must_use]
pub fn undo_split(steps: usize) -> String {
    format!("Undoing this edit takes {steps} steps of Undo.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_disclosure_names_every_character_and_the_face() {
        let s = set_in(&['q', 'z'], &[], "Helvetica");
        assert!(s.contains("\u{2018}q\u{2019} or \u{2018}z\u{2019}"));
        assert!(s.contains("Helvetica"));
        assert!(set_in(&['q'], &[], "Arial").contains("set it in Arial"));
    }

    #[test]
    fn a_letter_drawn_two_ways_is_said_so() {
        let s = set_in(&['A'], &['A'], "Helvetica");
        assert!(s.starts_with("This text's font draws \u{2018}A\u{2019} two different ways"));
        let mixed = set_in(&['q', 'A'], &['A'], "Helvetica");
        assert!(mixed.contains("cannot write \u{2018}q\u{2019}"));
        assert!(mixed.contains("also draws \u{2018}A\u{2019}"));
    }
}
