//! # `text::previewfallback` — why a text edit's live preview is drawn in a
//! stand-in font rather than the text's own
//!
//! The preview of an edit to existing text is the engine's layout, in the
//! run's own font. When it cannot be, the canvas draws the draft in the
//! shell's font at the run's size, and the status bar says why in one of the
//! sentences below. The canvas carries no mark of the substitution.

/// Why the live preview of an edit to existing text is not in the text's own
/// font. Owned by `pdfcer_gui::canvas::textedit`; worded by [`line`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PreviewFallback {
    /// The engine refused to lay the edit out; committing it would be refused
    /// the same way.
    Refused,
    /// The font has no outlines pdfcer can draw (a Type 3 or an unreadable
    /// program).
    NoOutlines,
    /// The font's glyphs do not pair one to one with the characters typed, so
    /// no caret can be placed between them.
    Unpaired,
    /// At this zoom the text is larger than the preview's ink buffer.
    TooLarge,
    /// Some typed characters are not in the text's font and will be set in
    /// the nearest font that has them.
    Reface,
}

impl PreviewFallback {
    /// A stable token for the diagnostic trace; never displayed.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Refused => "refused", // ui-text-exempt: a trace token, never displayed
            Self::NoOutlines => "no-outlines", // ui-text-exempt: a trace token, never displayed
            Self::Unpaired => "unpaired", // ui-text-exempt: a trace token, never displayed
            Self::TooLarge => "too-large", // ui-text-exempt: a trace token, never displayed
            Self::Reface => "reface",   // ui-text-exempt: a trace token, never displayed
        }
    }
}

/// The status-bar sentence for `why`.
#[must_use]
pub const fn line(why: PreviewFallback) -> &'static str {
    match why {
        PreviewFallback::Refused => {
            "Your typing is shown in a stand-in font: pdfcer cannot make this change in the \
             text's own font, so committing it will be refused."
        }
        PreviewFallback::NoOutlines => {
            "Your typing is shown in a stand-in font: pdfcer cannot draw this text's font for a \
             preview. The saved text uses the text's own font."
        }
        PreviewFallback::Unpaired => {
            "Your typing is shown in a stand-in font: this font's characters and shapes do not \
             match one for one, so pdfcer cannot place a cursor in it. The saved text uses the \
             text's own font."
        }
        PreviewFallback::TooLarge => {
            "Your typing is shown in a stand-in font because the text is too large to preview at \
             this zoom. Zoom out to see it in its own font."
        }
        PreviewFallback::Reface => {
            "Your typing is shown in a stand-in font: some of the characters you typed are not in \
             this text's font. When you commit, pdfcer sets them in the nearest font that has \
             them and says which."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PreviewFallback as P, line};

    #[test]
    fn the_five_reasons_have_distinct_sentences_and_tokens() {
        let all = [
            P::Refused,
            P::NoOutlines,
            P::Unpaired,
            P::TooLarge,
            P::Reface,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(line(*a), line(*b));
                assert_ne!(a.token(), b.token());
            }
            assert!(line(*a).starts_with("Your typing is shown in a stand-in font"));
        }
    }
}
