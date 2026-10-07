//! # `text::forms::redraw` — what a redraw decided about a field's text
//!
//! Every verb that redraws a text or choice field as a side effect of another
//! change (a property edit, a box edit, a rotation, a reset, a regeneration)
//! carries the engine's `LayoutDisclosure`. [`redraw_notes`] is the one place
//! that turns it into status-line sentences, so the five routes cannot word
//! the same fact five ways.
//!
//! The fill route keeps its own sentences (`forms_fill_*_note`): a fill STORES
//! what was typed, so its wording is about the value; a redraw changes only
//! the picture, so this wording says the stored value keeps every character.

use pdfcer_core::edit::LayoutDisclosure;
use pdfcer_core::vartext::AutoFitBound;

/// Who the sentences are about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedrawSubject<'a> {
    /// One field, by its fully-qualified name.
    Field(&'a str),
    /// A whole-form redraw (reset, regenerate): the engine reports the last
    /// auto-sized field, unnamed.
    Form,
}

impl RedrawSubject<'_> {
    /// The sentence's subject, capitalised.
    fn named(self) -> String {
        match self {
            Self::Field(name) => format!("“{name}”"),
            Self::Form => "A field of this form".to_owned(),
        }
    }
}

/// The status-line sentences one redraw owes, empty when it decided nothing.
///
/// `AutoFitBound` is `#[non_exhaustive]`; a bound this build does not know,
/// like `None` (a multiline field, where no bound was evaluated), takes the
/// general sentence rather than a claim about a constraint it cannot name.
#[must_use]
pub fn redraw_notes(subject: RedrawSubject<'_>, layout: &LayoutDisclosure) -> Vec<String> {
    let who = subject.named();
    let mut notes = Vec::new();
    if let Some(size) = layout.applied_autosize {
        notes.push(match layout.applied_autosize_bound {
            Some(AutoFitBound::Floor) => format!(
                "⚠ {who} was redrawn too small for its text: pdfcer held the size at {size:.1} pt \
                 so it stays readable, and the text overflows the box."
            ),
            Some(AutoFitBound::Width) => format!(
                "⚠ {who} was redrawn at an automatic text size; pdfcer chose {size:.1} pt to fit \
                 its width, so making it taller will not change it."
            ),
            Some(AutoFitBound::Height) | None | Some(_) => format!(
                "⚠ {who} was redrawn at an automatic text size; pdfcer chose {size:.1} pt. \
                 Another program may choose differently."
            ),
        });
    }
    if layout.da_colour_unmodelled {
        notes.push(format!(
            "⚠ {who} asks for a text colour pdfcer cannot draw, so its text was redrawn in black."
        ));
    }
    if layout.unencodable_chars > 0 {
        notes.push(format!(
            "⚠ {who} was redrawn with {} character(s) its font cannot draw, or past its last \
             box: they show as “?” or not at all. The stored value keeps them.",
            layout.unencodable_chars
        ));
    }
    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> LayoutDisclosure {
        LayoutDisclosure::default()
    }

    #[test]
    fn nothing_decided_says_nothing() {
        assert!(redraw_notes(RedrawSubject::Field("A"), &layout()).is_empty());
    }

    #[test]
    fn each_bound_has_its_own_sentence() {
        let mut l = layout();
        l.applied_autosize = Some(6.0);
        l.applied_autosize_bound = Some(AutoFitBound::Floor);
        let floor = redraw_notes(RedrawSubject::Field("Name"), &l);
        assert!(floor[0].contains("overflows") && floor[0].contains("“Name”"));
        l.applied_autosize_bound = Some(AutoFitBound::Width);
        assert!(redraw_notes(RedrawSubject::Field("Name"), &l)[0].contains("width"));
        l.applied_autosize_bound = None;
        let general = &redraw_notes(RedrawSubject::Form, &l)[0];
        assert!(general.starts_with("⚠ A field of this form") && general.contains("6.0 pt"));
    }

    #[test]
    fn colour_and_characters_are_separate_sentences() {
        let mut l = layout();
        l.da_colour_unmodelled = true;
        l.unencodable_chars = 2;
        let notes = redraw_notes(RedrawSubject::Form, &l);
        assert_eq!(notes.len(), 2);
        assert!(notes[0].contains("black"));
        assert!(notes[1].contains("2 character(s)") && notes[1].contains("stored value keeps"));
    }
}
