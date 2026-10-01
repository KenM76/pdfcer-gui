//! Which show operators a line edit touches: the common prefix and suffix of
//! the original and the draft, mapped onto each operator's byte range in the
//! run text.
//!
//! Contract: `operators` are byte ranges into `original`, sorted and
//! non-overlapping, one per show operator in the run's order. Bytes between
//! two ranges belong to no operator — they are characters the extractor
//! synthesised (a word gap), which no content-stream operator holds. An edit
//! that changes only such characters touches nothing an operator can carry,
//! and [`touched`] answers `None`.

use std::ops::Range;

/// The operators an edit changes, and the text that replaces theirs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Touched {
    /// Index into the `operators` slice of the first touched operator.
    pub first: usize,
    /// Index of the last touched operator; equal to `first` for one.
    pub last: usize,
    /// Bytes of `original` from the first operator's start to the last's end.
    pub original: Range<usize>,
    /// What replaces `original[self.original]`: the operators' text with the
    /// edit applied, and nothing outside them.
    pub replacement: String,
}

impl Touched {
    /// How many operators the edit reaches, counting any between the ends.
    #[must_use]
    pub fn operators(&self) -> usize {
        self.last - self.first + 1
    }
}

/// The operators `draft` changes relative to `original`, or `None` when the
/// two are equal, a range is malformed, or the change lies wholly in bytes no
/// operator holds.
///
/// An insertion at the boundary between two operators extends the one before
/// it: typing after a word appends to that word's operator, which is what a
/// reader expects when the next operator is Word's trailing space object.
#[must_use]
pub fn touched(operators: &[Range<usize>], original: &str, draft: &str) -> Option<Touched> {
    if original == draft || !well_formed(operators, original) {
        return None;
    }
    let prefix = common_prefix(original, draft);
    let suffix = common_suffix(&original[prefix..], &draft[prefix..]);
    let changed = prefix..original.len() - suffix;
    let (first, last) = if changed.is_empty() {
        let at = operators
            .iter()
            .position(|r| r.start < prefix && prefix <= r.end)
            .or_else(|| operators.iter().position(|r| r.start == prefix))?;
        (at, at)
    } else {
        let hit = |r: &Range<usize>| r.start < changed.end && r.end > changed.start;
        let first = operators.iter().position(hit)?;
        let last = operators.iter().rposition(hit)?;
        if operators[first].start > changed.start || operators[last].end < changed.end {
            return None;
        }
        (first, last)
    };
    let span = operators[first].start..operators[last].end;
    let mut replacement = String::with_capacity(span.len() + draft.len() - original.len());
    replacement.push_str(&original[span.start..prefix]);
    replacement.push_str(&draft[prefix..draft.len() - suffix]);
    replacement.push_str(&original[changed.end..span.end]);
    Some(Touched {
        first,
        last,
        original: span,
        replacement,
    })
}

fn well_formed(operators: &[Range<usize>], text: &str) -> bool {
    let mut floor = 0;
    operators.iter().all(|r| {
        let ok = r.start >= floor
            && r.start < r.end
            && r.end <= text.len()
            && text.is_char_boundary(r.start)
            && text.is_char_boundary(r.end);
        floor = r.end;
        ok
    })
}

/// The part of `find` the engine rewrites when a match spans several
/// operators, and its replacement: the engine trims the common prefix, then
/// the common suffix of the rest, and keeps at least one `find` character.
/// `None` when nothing trims, as the engine then rewrites the whole match.
///
/// Mirrors `pdfcer_core::text_edit` `narrow_span`, whose preview reports
/// glyphs for the trimmed replacement only and does not say which part it is.
#[must_use]
pub fn engine_trim(find: &str, replace: &str) -> Option<(Range<usize>, String)> {
    if find.is_empty() {
        return None;
    }
    let mut pre = common_prefix(find, replace);
    let mut suf = common_suffix(&find[pre..], &replace[pre..]);
    if pre + suf == find.len() {
        if let Some(c) = find[..pre].chars().next_back() {
            pre -= c.len_utf8();
        } else if let Some(c) = find[find.len() - suf..].chars().next() {
            suf -= c.len_utf8();
        }
    }
    if pre == 0 && suf == 0 {
        return None;
    }
    Some((
        pre..find.len() - suf,
        replace[pre..replace.len() - suf].to_owned(),
    ))
}

/// Bytes shared at the front, ending on a character boundary of both.
fn common_prefix(a: &str, b: &str) -> usize {
    a.char_indices()
        .zip(b.chars())
        .find(|((_, x), y)| x != y)
        .map_or_else(|| a.len().min(b.len()), |((i, _), _)| i)
}

/// Bytes shared at the back, ending on a character boundary of both.
fn common_suffix(a: &str, b: &str) -> usize {
    a.chars()
        .rev()
        .zip(b.chars().rev())
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x.len_utf8())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_trim_keeps_one_find_character_for_an_append() {
        assert_eq!(
            engine_trim("Required__ ", "Required__ _"),
            Some((10..11, " _".to_owned()))
        );
    }

    #[test]
    fn the_engine_trim_takes_the_prefix_before_the_suffix() {
        assert_eq!(
            engine_trim("Common-Law ", "CommonXaw "),
            Some((6..8, "X".to_owned()))
        );
        assert_eq!(engine_trim("abc", "xyz"), None);
        assert_eq!(engine_trim("", "x"), None);
    }

    // Word's shape: the words, then a separate object holding one space.
    const LINE: &str = "Date Required__ ";
    fn word_line() -> Vec<Range<usize>> {
        vec![0..15, 15..16]
    }

    #[test]
    fn typing_inside_the_words_touches_only_their_operator() {
        let t = touched(&word_line(), LINE, "Date Xequired__ ").unwrap();
        assert_eq!((t.first, t.last), (0, 0));
        assert_eq!(t.original, 0..15);
        assert_eq!(t.replacement, "Date Xequired__");
    }

    #[test]
    fn typing_at_the_boundary_extends_the_operator_before_it() {
        let t = touched(&word_line(), LINE, "Date Required___ ").unwrap();
        assert_eq!((t.first, t.last), (0, 0));
        assert_eq!(t.replacement, "Date Required___");
    }

    #[test]
    fn typing_at_the_very_start_prepends_to_the_first_operator() {
        let t = touched(&word_line(), LINE, "XDate Required__ ").unwrap();
        assert_eq!((t.first, t.replacement.as_str()), (0, "XDate Required__"));
    }

    #[test]
    fn typing_after_the_trailing_space_lands_in_the_space_object() {
        let t = touched(&word_line(), LINE, "Date Required__ X").unwrap();
        assert_eq!((t.first, t.replacement.as_str()), (1, " X"));
    }

    #[test]
    fn deleting_across_two_operators_touches_both() {
        let t = touched(&word_line(), LINE, "Date Required_").unwrap();
        assert_eq!((t.first, t.last), (0, 1));
        assert_eq!(t.operators(), 2);
        assert_eq!(t.original, 0..16);
        assert_eq!(t.replacement, "Date Required_");
    }

    #[test]
    fn a_mid_word_split_is_two_operators_and_an_edit_reaches_only_one() {
        // "Co" "-" "Law": one word written as three objects.
        let ops = vec![0..2, 2..3, 3..6];
        let t = touched(&ops, "Co-Law", "Co-Laws").unwrap();
        assert_eq!((t.first, t.replacement.as_str()), (2, "Laws"));
        let t = touched(&ops, "Co-Law", "Co Law").unwrap();
        assert_eq!((t.first, t.replacement.as_str()), (1, " "));
    }

    #[test]
    fn a_change_only_to_synthesised_gap_text_touches_nothing() {
        // The extractor wrote one space between operators that hold none.
        let ops = vec![0..3, 4..7];
        // An insertion at a gap's far edge prepends to the operator after it.
        assert_eq!(
            touched(&ops, "abc def", "abc  def").map(|t| t.first),
            Some(1)
        );
        assert!(touched(&ops, "abc def", "abcdef").is_none());
    }

    #[test]
    fn a_change_reaching_into_a_gap_and_an_operator_takes_the_operator_when_it_covers() {
        let ops = vec![0..3, 4..7];
        let t = touched(&ops, "abc def", "abc dXf").unwrap();
        assert_eq!((t.first, t.replacement.as_str()), (1, "dXf"));
        // Starts in the gap: no operator holds the gap byte.
        assert!(touched(&ops, "abc def", "abcXdef").is_none());
    }

    #[test]
    fn multibyte_characters_split_on_boundaries() {
        let text = "Applicant\u{2019}s name ";
        let ops = vec![0..9, 9..12, 12..18, 18..19];
        let t = touched(&ops, text, "Applicant\u{2019}s names ").unwrap();
        assert_eq!((t.first, t.replacement.as_str()), (2, "s names"));
        let t = touched(&ops, text, "Applicant's name ").unwrap();
        assert_eq!((t.first, t.replacement.as_str()), (1, "'"));
    }

    #[test]
    fn repeated_characters_resolve_to_one_insertion_point() {
        let ops = vec![0..2, 2..3];
        let t = touched(&ops, "aa ", "aaa ").unwrap();
        assert_eq!((t.first, t.replacement.as_str()), (0, "aaa"));
    }

    #[test]
    fn no_change_and_malformed_ranges_answer_none() {
        assert!(touched(&word_line(), LINE, LINE).is_none());
        assert!(touched(std::slice::from_ref(&(0..20)), LINE, "x").is_none());
        assert!(touched(&[3..5, 2..4], LINE, "x").is_none());
        assert!(touched(&[], LINE, "x").is_none());
    }
}
