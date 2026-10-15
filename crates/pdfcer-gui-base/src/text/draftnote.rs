//! # `text::draftnote` — what a keystroke did to a draft that the operator
//! might not expect, for the status bar
//!
//! Worded here; raised by `pdfcer_gui::canvas::textedit::note`.

/// Something the last keystroke or paste changed on the way into a draft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftNote {
    /// A paste into a line already on the page held line breaks, which became
    /// spaces.
    LinesJoined,
    /// Tab was typed as this many spaces.
    TabAsSpaces(usize),
    /// A paragraph opened as one draft whose typed breaks were judged from
    /// its layout, not read from marks pdfcer wrote.
    BreaksInferred,
    /// A line of a paragraph opened alone because the paragraph mixes fonts,
    /// sizes or colours.
    LineOnlyMixedLooks,
    /// A line of a paragraph opened alone because the engine would not
    /// re-write the paragraph as it stands.
    LineOnlyRefused,
    /// A recognised (OCR) word opened alone: the text layer stores each word
    /// separately.
    RecognisedWordOnly,
}

impl DraftNote {
    /// A stable token for the diagnostic trace; never displayed.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::LinesJoined => "lines-joined", // ui-text-exempt: a trace token, never displayed
            Self::TabAsSpaces(_) => "tab-as-spaces", // ui-text-exempt: a trace token, never displayed
            Self::BreaksInferred => "breaks-inferred", // ui-text-exempt: a trace token, never displayed
            Self::LineOnlyMixedLooks => "line-only-mixed-looks", // ui-text-exempt: a trace token, never displayed
            Self::LineOnlyRefused => "line-only-refused", // ui-text-exempt: a trace token, never displayed
            Self::RecognisedWordOnly => "recognised-word-only", // ui-text-exempt: a trace token, never displayed
        }
    }
}

/// The status-bar sentence for `note`.
#[must_use]
pub fn line(note: DraftNote) -> String {
    match note {
        DraftNote::LinesJoined => "The pasted lines were joined with spaces: a line already on \
                                   the page cannot hold a line break."
            .to_owned(),
        DraftNote::TabAsSpaces(1) => "Tab was typed as a space to the next half-inch stop: text \
                                      in a PDF has no tab stops."
            .to_owned(),
        DraftNote::TabAsSpaces(n) => format!(
            "Tab was typed as {n} spaces to the next half-inch stop: text in a PDF has no tab \
             stops."
        ),
        DraftNote::BreaksInferred => "Where this paragraph's lines were broken by hand was \
                                      judged from its layout: a break after a nearly full line \
                                      reads as a wrap and joins the next line."
            .to_owned(),
        DraftNote::LineOnlyMixedLooks => "Only this line opened: its paragraph mixes fonts, \
                                          sizes or colours, and editing it as one would set all \
                                          of it in the first line's look."
            .to_owned(),
        DraftNote::LineOnlyRefused => "Only this line opened: pdfcer cannot re-write its \
                                       paragraph as a whole."
            .to_owned(),
        DraftNote::RecognisedWordOnly => "Recognised text opens one word at a time: the text \
                                          layer stores each word on its own."
            .to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{DraftNote, line};

    #[test]
    fn a_join_and_two_tab_counts_read_differently() {
        let all = [
            DraftNote::LinesJoined,
            DraftNote::TabAsSpaces(1),
            DraftNote::TabAsSpaces(4),
            DraftNote::BreaksInferred,
            DraftNote::LineOnlyMixedLooks,
            DraftNote::LineOnlyRefused,
            DraftNote::RecognisedWordOnly,
        ];
        let lines: std::collections::HashSet<String> = all.iter().map(|n| line(*n)).collect();
        assert_eq!(lines.len(), all.len());
        assert!(line(DraftNote::TabAsSpaces(4)).contains("4 spaces"));
        assert_ne!(
            DraftNote::LinesJoined.token(),
            DraftNote::TabAsSpaces(2).token()
        );
    }
}
