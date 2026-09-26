//! # `canvas::textedit::lines` — the caret's arithmetic inside a MULTI-LINE draft
//!
//!
//! The operator, `OPERATOR_REQUESTS.md` **O127**:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/lines.md`.

/// **Where each line of `text` starts and ends**, as character offsets.
#[must_use]
pub fn spans(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for (i, c) in text.chars().enumerate() {
        if c == '\n' {
            out.push((start, i));
            start = i + 1;
        }
    }
    out.push((start, text.chars().count()));
    out
}

/// **Which line `caret` is on, and how far along it.** `(line, column)`, both
/// zero-based and both in characters.
#[must_use]
pub fn locate(text: &str, caret: usize) -> (usize, usize) {
    let spans = spans(text);
    let caret = caret.min(text.chars().count());
    for (line, (from, to)) in spans.iter().enumerate() {
        // `caret <= to` and not `<`, because the caret may sit at the very end
        // of a line — which is where it is after typing the last character of
        // it, and the commonest position there is.
        if caret <= *to {
            return (line, caret - from);
        }
    }
    // Unreachable: `spans` always ends at the length and the caret is clamped
    // to it. Answered rather than `unreachable!()`, because a caret arithmetic
    // slip is not worth a crash in the frame that is drawing the operator's
    // draft — the honest fallback is the end of the text, which is where an
    // out-of-range caret was heading anyway.
    let last = spans.len() - 1;
    (last, spans[last].1 - spans[last].0)
}

/// **The character offset of `column` on `line`**, clamped both ways.
#[must_use]
pub fn offset_of(text: &str, line: usize, column: usize) -> usize {
    let spans = spans(text);
    let (from, to) = spans[line.min(spans.len() - 1)];
    from + column.min(to - from)
}

/// **Press Up.** The same column on the line above, or `None` at the top.
#[must_use]
pub fn up(text: &str, caret: usize) -> Option<usize> {
    let (line, column) = locate(text, caret);
    (line > 0).then(|| offset_of(text, line - 1, column))
}

/// **Press Down.** The same column on the line below, or `None` at the bottom.
#[must_use]
pub fn down(text: &str, caret: usize) -> Option<usize> {
    let (line, column) = locate(text, caret);
    (line + 1 < spans(text).len()).then(|| offset_of(text, line + 1, column))
}

/// **Press Home.** The first character of the line the caret is on.
#[must_use]
pub fn start_of_line(text: &str, caret: usize) -> usize {
    let (line, _) = locate(text, caret);
    spans(text)[line].0
}

/// **Press End.** The position just before the line's break, or the end of the
/// draft on the last line.
#[must_use]
pub fn end_of_line(text: &str, caret: usize) -> usize {
    let (line, _) = locate(text, caret);
    spans(text)[line].1
}

/// **Does this draft hold more than one line?**
#[must_use]
pub fn is_multi_line(text: &str) -> bool {
    text.contains('\n')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A draft with no break is one line, and every key still works on
    /// it.**
    #[test]
    fn a_single_line_draft_behaves_exactly_as_it_did() {
        let s = "SHEET 1 OF 4";
        assert!(!is_multi_line(s));
        assert_eq!(spans(s), vec![(0, 12)]);
        assert_eq!(start_of_line(s, 5), 0);
        assert_eq!(end_of_line(s, 5), 12);
        assert_eq!(up(s, 5), None, "there is no line above a one-line draft");
        assert_eq!(down(s, 5), None, "and none below it");
    }

    /// **Up and Down keep the column**, which is the property that makes
    /// them feel like arrow keys rather than like jumps.
    #[test]
    fn vertical_movement_keeps_the_column() {
        let s = "abcdef\nghijkl\nmnopqr";
        // Caret between `c` and `d` on line 0 — column 3.
        assert_eq!(locate(s, 3), (0, 3));
        let down_once = down(s, 3).expect("there is a line below");
        assert_eq!(locate(s, down_once), (1, 3), "column 3 on line 1");
        let down_twice = down(s, down_once).expect("and another below that");
        assert_eq!(locate(s, down_twice), (2, 3));
        assert_eq!(down(s, down_twice), None, "the last line has nothing below");
        let back = up(s, down_twice).expect("and back up again");
        assert_eq!(
            back, down_once,
            "Up must undo Down on lines of equal length"
        );
    }

    /// **A short line clamps the column**, exactly as every editor does.
    #[test]
    fn a_shorter_line_clamps_rather_than_overshooting() {
        let s = "aaaaaaaa\nbb\ncccccccc";
        // Column 7 on the long first line.
        let landed = down(s, 7).expect("there is a line below");
        assert_eq!(
            locate(s, landed),
            (1, 2),
            "line 1 is two characters long, so column 7 clamps to its end"
        );
        assert_eq!(
            end_of_line(s, landed),
            landed,
            "and the clamped position IS the end of that line"
        );
    }

    /// **Home and End are the LINE's, not the draft's.**
    ///
    /// The defect this module fixes for those two keys. Before it, End on the
    /// middle line of a three-line box jumped to the bottom of the draft.
    #[test]
    fn home_and_end_stay_on_their_own_line() {
        let s = "first\nsecond\nthird";
        // Caret inside `second`.
        let inside = 8;
        assert_eq!(locate(s, inside), (1, 2));
        assert_eq!(start_of_line(s, inside), 6, "just after the first break");
        assert_eq!(end_of_line(s, inside), 12, "just before the second break");
        assert_ne!(
            end_of_line(s, inside),
            s.chars().count(),
            "END on a middle line must NOT reach the end of the draft — that is the \
             behaviour this module exists to replace"
        );
    }

    /// **A trailing break leaves the caret on a real, empty line.**
    #[test]
    fn a_trailing_break_makes_a_line_to_stand_on() {
        let s = "one\n";
        assert_eq!(spans(s), vec![(0, 3), (4, 4)]);
        let caret = s.chars().count();
        assert_eq!(
            locate(s, caret),
            (1, 0),
            "on the new empty line, at its start"
        );
        assert_eq!(start_of_line(s, caret), 4);
        assert_eq!(end_of_line(s, caret), 4);
        assert_eq!(up(s, caret), Some(0), "and Up reaches the line just typed");
    }

    /// **A caret survives an accent on every line.**
    #[test]
    fn a_caret_survives_an_accent_on_every_line() {
        let s = "café\nnaïve\nrésumé";
        assert!(s.chars().count() < s.len(), "the fixture must be non-ASCII");
        for (line, (from, to)) in spans(s).iter().enumerate() {
            for column in 0..=(to - from) {
                let at = offset_of(s, line, column);
                assert_eq!(
                    locate(s, at),
                    (line, column),
                    "line {line} column {column} did not round-trip through offset {at}"
                );
                // The offset must name a real character boundary — which is
                // what `chars().nth()` can answer and a byte index cannot.
                assert!(at <= s.chars().count());
            }
        }
    }

    /// **An empty draft is one empty line**, so no key panics on it.
    ///
    /// The state a fresh box draft is in before a single character is typed,
    /// which is where every one of these functions is called first.
    #[test]
    fn an_empty_draft_still_has_a_line() {
        let s = "";
        assert_eq!(spans(s), vec![(0, 0)]);
        assert_eq!(locate(s, 0), (0, 0));
        assert_eq!(start_of_line(s, 0), 0);
        assert_eq!(end_of_line(s, 0), 0);
        assert_eq!(up(s, 0), None);
        assert_eq!(down(s, 0), None);
    }

    /// **A caret past the end is clamped rather than fatal.**
    #[test]
    fn an_overlong_caret_lands_at_the_end() {
        let s = "ab\ncd";
        assert_eq!(locate(s, 99), (1, 2));
        assert_eq!(offset_of(s, 99, 99), 5);
    }
}
