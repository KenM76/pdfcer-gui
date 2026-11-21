//! # `text::editcause` — why the engine will not edit a run, in the classes
//! the operator is told apart
//!
//! Contract: [`EditCause::of`] classes a `pdfcer_core::text_edit::UnsupportedCause`;
//! every cause it does not name is [`EditCause::Other`], so a cause the engine
//! adds reads as the generic sentence until it is given its own. The sentences
//! are spoken at three moments: when the caret lands on a run no edit can
//! reach (`editmodel::refusal::Refusal::CannotEdit`), when it lands on one a
//! workaround can reach (`text::draftnote::DraftNote::CannotEditInPlace`), and
//! when a commit is refused (`text::editrefusal::EditRefusal::CannotEdit`).

/// A class of `pdfcer_core::text_edit::UnsupportedCause`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditCause {
    /// `VerticalWriting`.
    Vertical,
    /// `QuoteOperator`: a line drawn with the `'` or `"` operator.
    QuoteOperator,
    /// The run's font cannot be written back: `FontUnresolvable`,
    /// `EncodingNotInvertible`, `CompositeWithoutToUnicode`,
    /// `FontMapNotInvertible`.
    Font,
    /// The text is reached through a form XObject the edit cannot rewrite:
    /// `InsideFormXObject`, `FormNotOnPage`, `FormUndecodable`,
    /// `ReferenceXObject`, `OpiProxy`.
    PlacedDrawing,
    /// Any other cause.
    Other,
}

impl EditCause {
    /// The class of `cause`.
    #[must_use]
    pub const fn of(cause: &pdfcer_core::text_edit::UnsupportedCause) -> Self {
        use pdfcer_core::text_edit::UnsupportedCause as C;
        match cause {
            C::VerticalWriting => Self::Vertical,
            C::QuoteOperator => Self::QuoteOperator,
            C::FontUnresolvable
            | C::EncodingNotInvertible
            | C::CompositeWithoutToUnicode
            | C::FontMapNotInvertible { .. } => Self::Font,
            C::InsideFormXObject { .. }
            | C::FormNotOnPage { .. }
            | C::FormUndecodable { .. }
            | C::ReferenceXObject
            | C::OpiProxy => Self::PlacedDrawing,
            _ => Self::Other,
        }
    }

    /// A stable name for the trace; never displayed.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Vertical => "vertical", // ui-text-exempt: a trace token, never displayed
            Self::QuoteOperator => "quote-operator", // ui-text-exempt: a trace token, never displayed
            Self::Font => "font", // ui-text-exempt: a trace token, never displayed
            Self::PlacedDrawing => "placed-drawing", // ui-text-exempt: a trace token, never displayed
            Self::Other => "other", // ui-text-exempt: a trace token, never displayed
        }
    }

    /// What is different about this text, as one sentence.
    #[must_use]
    pub const fn what(self) -> &'static str {
        match self {
            Self::Vertical => {
                "This text is written top to bottom, and pdfcer cannot edit vertical text in \
                 place."
            }
            Self::QuoteOperator => {
                "This line is drawn with an instruction that also moves down a line, and pdfcer \
                 cannot edit that kind of line in place."
            }
            Self::Font => "pdfcer cannot write in the font this text is drawn with.",
            Self::PlacedDrawing => "This text is inside a placed drawing pdfcer cannot rewrite.",
            Self::Other => "pdfcer cannot edit this text in place.",
        }
    }

    /// The caret's refusal when no workaround reaches the run: what is
    /// different, then the route that remains.
    #[must_use]
    pub const fn refused(self) -> &'static str {
        match self {
            Self::Vertical => {
                "This text is written top to bottom, and pdfcer cannot edit vertical text in \
                 place. Use Add text to write over it; other text in this document may still \
                 edit normally."
            }
            Self::QuoteOperator => {
                "This line is drawn with an instruction that also moves down a line, and pdfcer \
                 cannot edit that kind of line in place. Use Add text to write over it; other \
                 text in this document may still edit normally."
            }
            Self::Font => {
                "pdfcer cannot write in the font this text is drawn with. Use Add text to write \
                 over it; other text in this document may still edit normally."
            }
            Self::PlacedDrawing => {
                "This text is inside a placed drawing pdfcer cannot rewrite. Use Add text to \
                 write over it; other text in this document may still edit normally."
            }
            Self::Other => {
                "pdfcer cannot edit this text in place. Use Add text to write over it; other text \
                 in this document may still edit normally."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EditCause;
    use pdfcer_core::text_edit::UnsupportedCause as C;

    const ALL: [EditCause; 5] = [
        EditCause::Vertical,
        EditCause::QuoteOperator,
        EditCause::Font,
        EditCause::PlacedDrawing,
        EditCause::Other,
    ];

    /// Each class has its own sentence and its own trace name, and a refusal
    /// opens with what the class says is different.
    #[test]
    fn each_class_reads_differently() {
        let what: std::collections::HashSet<_> = ALL.iter().map(|c| c.what()).collect();
        let names: std::collections::HashSet<_> = ALL.iter().map(|c| c.name()).collect();
        assert_eq!(what.len(), ALL.len());
        assert_eq!(names.len(), ALL.len());
        for c in ALL {
            assert!(c.refused().starts_with(c.what()), "{c:?}");
            assert!(c.refused().contains("Add text"), "{c:?}");
        }
    }

    /// The engine's causes land in their classes; one not named is `Other`.
    #[test]
    fn causes_land_in_their_classes() {
        assert_eq!(EditCause::of(&C::VerticalWriting), EditCause::Vertical);
        assert_eq!(EditCause::of(&C::QuoteOperator), EditCause::QuoteOperator);
        assert_eq!(EditCause::of(&C::FontUnresolvable), EditCause::Font);
        assert_eq!(
            EditCause::of(&C::InsideFormXObject { object: None }),
            EditCause::PlacedDrawing
        );
        assert_eq!(EditCause::of(&C::NoContents), EditCause::Other);
    }
}
