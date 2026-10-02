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
}

impl DraftNote {
    /// A stable token for the diagnostic trace; never displayed.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::LinesJoined => "lines-joined", // ui-text-exempt: a trace token, never displayed
            Self::TabAsSpaces(_) => "tab-as-spaces", // ui-text-exempt: a trace token, never displayed
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
        ];
        let lines: std::collections::HashSet<String> = all.iter().map(|n| line(*n)).collect();
        assert_eq!(lines.len(), 3);
        assert!(line(DraftNote::TabAsSpaces(4)).contains("4 spaces"));
        assert_ne!(
            DraftNote::LinesJoined.token(),
            DraftNote::TabAsSpaces(2).token()
        );
    }
}
