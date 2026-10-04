//! Copy for `format.split_text_lines`: why splitting a text object into its
//! lines was declined, and the sentence that counts the pieces.
//!
//! The engine's plan discloses that the line breaks were inferred; this module
//! words the result and the refusals.

use pdfcer_core::edit::EditError;
use pdfcer_core::vector::VectorEditError;

/// Why a text object was not split into its lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunSplitRefusal {
    /// The selection is not one page text object of two or more lines.
    NotOneTextObject,
    /// pdfcer found no line break to cut at.
    NoLineBreaks,
    /// A line takes its position from the end of the line before it.
    LineInheritsPosition,
    /// A line is shown with `'` or `"`, which also moves to the next line.
    LineShowOperator,
    /// A tagged (marked-content) section is open where a line starts.
    InsideTaggedContent,
    /// The document is encrypted.
    Encrypted,
    /// The selected object no longer exists as selected.
    Stale,
    /// Anything else the engine refused.
    Other,
}

impl RunSplitRefusal {
    /// Classify a preflight or engine refusal.
    #[must_use]
    pub fn of_vector(error: &VectorEditError) -> Self {
        match error {
            VectorEditError::EmptySplit | VectorEditError::SplitAtObjectStart => Self::NoLineBreaks,
            VectorEditError::SplitRunInheritsPosition { .. } => Self::LineInheritsPosition,
            VectorEditError::SplitAtLineShowOperator { .. } => Self::LineShowOperator,
            VectorEditError::SplitInsideMarkedContent { .. } => Self::InsideTaggedContent,
            VectorEditError::TextRunOutOfRange { .. }
            | VectorEditError::ObjectOutOfRange { .. } => Self::Stale,
            _ => Self::Other,
        }
    }

    /// Classify the session verb's answer.
    #[must_use]
    pub fn of_edit(error: &EditError) -> Self {
        match error {
            EditError::VectorEdit(inner) => Self::of_vector(inner),
            EditError::DocumentEncrypted => Self::Encrypted,
            EditError::PageOutOfRange { .. } => Self::Stale,
            _ => Self::Other,
        }
    }

    /// The trace token, stable for `ui-verify`.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::NotOneTextObject => "not-one-text-object",
            Self::NoLineBreaks => "no-line-breaks",
            Self::LineInheritsPosition => "line-inherits-position",
            Self::LineShowOperator => "line-show-operator",
            Self::InsideTaggedContent => "inside-tagged-content",
            Self::Encrypted => "encrypted",
            Self::Stale => "stale",
            Self::Other => "other",
        }
    }

    /// The status-line sentence.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::NotOneTextObject => {
                "Splitting into lines needs one selected text object that has more than one line."
            }
            Self::NoLineBreaks => {
                "pdfcer found no line break in this text object, so there is nothing to split. \
                 Nothing was changed."
            }
            Self::LineInheritsPosition => {
                "A line of this text continues from where the line before it ends, so it has no \
                 position of its own to keep once split. Nothing was changed."
            }
            Self::LineShowOperator => {
                "A line of this text is written with an operator that also moves to the next \
                 line, which splitting does not support. Nothing was changed."
            }
            Self::InsideTaggedContent => {
                "A line of this text starts inside a tagged section of the page, so splitting \
                 there would break the tagging. Nothing was changed."
            }
            Self::Encrypted => {
                "This document is encrypted, so its text cannot be split. Nothing was changed."
            }
            Self::Stale => "The selected text changed before the split ran. Select it again.",
            Self::Other => "pdfcer could not split this text into lines. Nothing was changed.",
        }
    }
}

/// The success sentence: how many objects the one became.
#[must_use]
pub fn split_into(pieces: usize) -> String {
    format!("The text was split into {pieces} separate objects, one per line. Undo rejoins them.")
}

#[cfg(test)]
mod tests {
    use super::RunSplitRefusal as R;

    const SAMPLE: [R; 8] = [
        R::NotOneTextObject,
        R::NoLineBreaks,
        R::LineInheritsPosition,
        R::LineShowOperator,
        R::InsideTaggedContent,
        R::Encrypted,
        R::Stale,
        R::Other,
    ];

    #[test]
    fn the_listed_refusals_have_distinct_sentences_and_tokens() {
        for (i, a) in SAMPLE.iter().enumerate() {
            assert!(a.line().ends_with('.'), "{a:?}");
            for b in &SAMPLE[i + 1..] {
                assert_ne!(a.line(), b.line(), "{a:?} and {b:?} share a sentence");
                assert_ne!(a.token(), b.token(), "{a:?} and {b:?} share a token");
            }
        }
    }

    #[test]
    fn the_engine_refusals_classify_by_name() {
        use pdfcer_core::vector::VectorEditError as E;
        assert_eq!(R::of_vector(&E::EmptySplit), R::NoLineBreaks);
        assert_eq!(
            R::of_vector(&E::SplitRunInheritsPosition { index: 2 }),
            R::LineInheritsPosition
        );
        assert_eq!(
            R::of_vector(&E::SplitAtLineShowOperator { index: 1 }),
            R::LineShowOperator
        );
        assert_eq!(
            R::of_vector(&E::SplitInsideMarkedContent { index: 1 }),
            R::InsideTaggedContent
        );
    }
}
