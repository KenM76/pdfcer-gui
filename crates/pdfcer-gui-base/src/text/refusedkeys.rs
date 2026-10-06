//! # `text::refusedkeys` — the notice beside a text edit that names every key
//! the run's font could not take, and offers a face that can
//!
//! Worded here; drawn by `pdfcer_gui::canvas::textedit::refused`.

/// `Q`, `Q or R`, `Q, R or W` — the characters as a sentence names them.
#[must_use]
pub fn list(chars: &[char]) -> String {
    let named: Vec<String> = chars.iter().map(|c| quoted(*c)).collect();
    match named.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
    }
}

/// A character as the operator would recognise it; a space is otherwise invisible.
fn quoted(c: char) -> String {
    if c.is_whitespace() {
        "a space".to_owned()
    } else {
        format!("\u{2018}{c}\u{2019}")
    }
}

/// Why `font` would not take `chars`: it lacks some, and draws those in
/// `two_ways` (`RunRepertoire::ambiguous`) with more than one code.
#[must_use]
pub fn cause(chars: &[char], two_ways: &[char], font: &str) -> String {
    let (doubled, absent): (Vec<char>, Vec<char>) =
        chars.iter().partition(|c| two_ways.contains(c));
    match (absent.is_empty(), doubled.is_empty()) {
        (_, true) => format!("{font} has no {}", list(&absent)),
        (true, false) => format!(
            "{font} draws {} two different ways and pdfcer will not choose one",
            list(&doubled)
        ),
        (false, false) => format!(
            "{font} has no {} and draws {} two different ways",
            list(&absent),
            list(&doubled)
        ),
    }
}

/// The notice's first line.
#[must_use]
pub fn named(chars: &[char], two_ways: &[char], font: &str) -> String {
    let what = if chars.len() == 1 {
        "it was"
    } else {
        "they were"
    };
    format!("{}, so {what} not typed.", cause(chars, two_ways, font))
}

/// The one-click button.
#[must_use]
pub fn use_face(face: &str, count: usize) -> String {
    let what = if count == 1 { "it" } else { "them" };
    format!("Use {face}, which has {what}")
}

/// Said instead of a button when no face this page could use takes them all.
#[must_use]
pub fn no_face(chars: &[char]) -> String {
    format!(
        "No font pdfcer can use for this text has {}. Leave them out, or add the text as a new \
         text box in a font that has them.",
        list(chars)
    )
}

/// The notice's line when the keys went in, planned for another face.
#[must_use]
pub fn planned(chars: &[char], two_ways: &[char], font: &str, face: &str) -> String {
    let what = if chars.len() == 1 { "it" } else { "they" };
    format!(
        "{}, so {what} will be set in {face} when you commit.",
        cause(chars, two_ways, font)
    )
}

/// The button changing the whole line to the face instead.
#[must_use]
pub fn use_whole(face: &str) -> String {
    format!("Use {face} for the whole line instead")
}

/// The whole line's face change landed, with the planned keys already in.
#[must_use]
pub fn now_whole(face: &str) -> String {
    format!("This line is now in {face}.")
}

/// The face change was asked for and has not landed yet.
#[must_use]
pub fn switching(face: &str) -> String {
    format!("Changing this text to {face}\u{2026}")
}

/// The face change landed and the held keys were typed.
#[must_use]
pub fn retyped(face: &str, chars: &[char]) -> String {
    format!("This text is now in {face}, and {} went in.", list(chars))
}

/// The face change landed, but the caret had moved, so the keys were not put back.
#[must_use]
pub fn swapped_type_again(face: &str, chars: &[char]) -> String {
    format!("This text is now in {face}. Type {} again.", list(chars))
}

/// The face change was refused; the status bar says why.
#[must_use]
pub fn swap_refused(face: &str) -> String {
    format!("pdfcer could not change this text to {face}. The status bar says why.")
}

#[cfg(test)]
mod tests {
    use super::{list, named, planned, use_face};

    #[test]
    fn every_character_is_named() {
        assert_eq!(list(&['Q']), "\u{2018}Q\u{2019}");
        assert_eq!(list(&['Q', 'R']), "\u{2018}Q\u{2019} or \u{2018}R\u{2019}");
        assert_eq!(
            list(&['Q', 'R', ' ']),
            "\u{2018}Q\u{2019}, \u{2018}R\u{2019} or a space"
        );
        assert!(named(&['Q', 'R', 'W'], &[], "Arial").contains("\u{2018}W\u{2019}"));
        assert!(named(&['Q'], &[], "Arial").ends_with("so it was not typed."));
        assert!(use_face("Helvetica", 3).ends_with("has them"));
    }

    #[test]
    fn a_letter_drawn_two_ways_is_not_said_missing() {
        let both = named(&['A'], &['A'], "Box");
        assert!(both.contains("draws \u{2018}A\u{2019} two different ways"));
        assert!(!both.contains("has no"));
        let mixed = planned(&['Q', 'A'], &['A'], "Box", "Helvetica");
        assert!(mixed.starts_with("Box has no \u{2018}Q\u{2019} and draws \u{2018}A\u{2019}"));
    }
}
