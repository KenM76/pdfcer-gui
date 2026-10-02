//! # `text::reface` — what a text edit says when some of its characters were
//! set in another face because the line's own font lacks them
//!
//! Worded here; raised by `pdfcer_gui::app::actions::reface`.

use super::refusedkeys::list;

/// The disclosure on a committed edit: which characters, and which face.
#[must_use]
pub fn set_in(chars: &[char], face: &str) -> String {
    let what = if chars.len() == 1 { "it" } else { "them" };
    format!(
        "This text's font has no {}, so pdfcer set {what} in {face}, the nearest font that has \
         {what}.",
        list(chars)
    )
}

/// Said when the edit's steps could not be joined into one undo.
#[must_use]
pub fn undo_split(steps: usize) -> String {
    format!("Undoing this edit takes {steps} steps of Undo.")
}

/// Why a re-faced edit was not made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stopped {
    /// The line, with stand-ins for the characters, was refused.
    Line,
    /// Setting the characters in the other face was refused.
    Face,
    /// Writing the characters in the other face was refused.
    Chars,
}

/// The status-bar sentence for a re-faced edit that was not made; `detail` is
/// the engine's own reason.
#[must_use]
pub fn stopped(why: Stopped, face: &str, detail: &str) -> String {
    let detail = detail.trim_end().trim_end_matches('.');
    let what = match why {
        Stopped::Line => "pdfcer could not make this edit",
        Stopped::Face => "pdfcer could not set the new characters in",
        Stopped::Chars => "pdfcer could not write the new characters in",
    };
    match why {
        Stopped::Line => format!("{what}: {detail}. Nothing was changed."),
        Stopped::Face | Stopped::Chars => {
            format!("{what} {face}: {detail}. Nothing was changed.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_disclosure_names_every_character_and_the_face() {
        let s = set_in(&['q', 'z'], "Helvetica");
        assert!(s.contains("\u{2018}q\u{2019} or \u{2018}z\u{2019}"));
        assert!(s.contains("Helvetica"));
        assert!(set_in(&['q'], "Arial").contains("set it in Arial"));
    }

    #[test]
    fn every_stop_says_nothing_changed() {
        for why in [Stopped::Line, Stopped::Face, Stopped::Chars] {
            assert!(stopped(why, "Arial", "x.").ends_with("Nothing was changed."));
        }
        assert_ne!(
            stopped(Stopped::Face, "A", "x."),
            stopped(Stopped::Chars, "A", "x.")
        );
    }
}
