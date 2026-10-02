//! # `editmodel::reface` — the characters of a line edit set in another face,
//! and the placeholder text that stands in for them while the line commits
//!
//! A *segment* is a maximal run of [`Reface::chars`] in the replacement.
//! Segment `i` is stood in for by a placeholder repeated `floor + i` times,
//! where `floor` is one more than the longest run of the placeholder in any
//! line of the page or in the replacement. Each token is therefore unique on
//! the page once every longer token has been replaced, which is why
//! [`Tokens::segments`] is walked last first.

/// Characters of a line edit that the run's font lacks, to be set in `face`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reface {
    /// The selector the face change sends (`FontSelector::new`).
    pub face: String,
    /// The face as the operator reads its name.
    pub label: String,
    /// The characters to set in `face`.
    pub chars: Vec<char>,
    /// Characters of the run's own text, any of which can stand in for
    /// `chars` while the line commits in the run's font; tried in order.
    pub placeholders: Vec<char>,
}

/// The tokenised replacement and the segments its tokens stand for.
#[derive(Debug, PartialEq, Eq)]
pub struct Tokens {
    /// The replacement with each segment swapped for its token.
    pub text: String,
    /// `(token, segment)`, in reading order.
    pub segments: Vec<(String, String)>,
}

/// Tokenise `replacement` with the first of `reface.placeholders` that works,
/// or `None` when it holds none of `reface.chars` or every placeholder is
/// itself foreign or touches a segment. `lines` is every line on the page.
#[must_use]
pub fn tokenise_any(replacement: &str, reface: &Reface, lines: &[&str]) -> Option<Tokens> {
    reface
        .placeholders
        .iter()
        .find_map(|p| tokenise(replacement, &reface.chars, *p, lines))
}

/// Tokenise `replacement` with `placeholder`; see [`tokenise_any`].
#[must_use]
pub fn tokenise(
    replacement: &str,
    foreign: &[char],
    placeholder: char,
    lines: &[&str],
) -> Option<Tokens> {
    let chars: Vec<char> = replacement.chars().collect();
    let spans = segments(&chars, foreign);
    if spans.is_empty() || foreign.contains(&placeholder) {
        return None;
    }
    let touches = spans.iter().any(|&(s, e)| {
        (s > 0 && chars[s - 1] == placeholder) || chars.get(e) == Some(&placeholder)
    });
    if touches {
        return None;
    }
    let page = lines
        .iter()
        .map(|l| longest_run(l, placeholder))
        .max()
        .unwrap_or(0);
    let floor = 1 + page.max(longest_run(replacement, placeholder));
    let mut text = String::with_capacity(replacement.len());
    let mut out = Vec::with_capacity(spans.len());
    let mut at = 0;
    for (n, &(s, e)) in spans.iter().enumerate() {
        text.extend(&chars[at..s]);
        let token: String = std::iter::repeat_n(placeholder, floor + n).collect();
        text.push_str(&token);
        out.push((token, chars[s..e].iter().collect()));
        at = e;
    }
    text.extend(&chars[at..]);
    Some(Tokens {
        text,
        segments: out,
    })
}

/// The `[start, end)` index spans of maximal runs of `foreign` in `chars`.
fn segments(chars: &[char], foreign: &[char]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if foreign.contains(&chars[i]) {
            let start = i;
            while i < chars.len() && foreign.contains(&chars[i]) {
                i += 1;
            }
            spans.push((start, i));
        } else {
            i += 1;
        }
    }
    spans
}

/// The longest run of `c` in `s`.
fn longest_run(s: &str, c: char) -> usize {
    let (mut best, mut now) = (0, 0);
    for x in s.chars() {
        now = if x == c { now + 1 } else { 0 };
        best = best.max(now);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_segments_get_tokens_longer_than_any_run_on_the_page() {
        let t = tokenise("ABqzCxq", &['q', 'z', 'x'], 'A', &["AA here"]).unwrap();
        assert_eq!(t.text, "ABAAACAAAA");
        assert_eq!(
            t.segments,
            vec![("AAA".into(), "qz".into()), ("AAAA".into(), "xq".into())]
        );
    }

    #[test]
    fn a_placeholder_beside_a_segment_declines() {
        assert_eq!(tokenise("Aq", &['q'], 'A', &[]), None);
        assert_eq!(tokenise("qA", &['q'], 'A', &[]), None);
    }

    #[test]
    fn nothing_foreign_declines() {
        assert_eq!(tokenise("ABC", &['q'], 'B', &[]), None);
    }

    #[test]
    fn the_next_placeholder_is_tried_when_the_first_touches() {
        let reface = Reface {
            face: "Helvetica".into(),
            label: "Helvetica".into(),
            chars: vec!['q'],
            placeholders: vec!['A', 'B'],
        };
        let t = tokenise_any("Aq", &reface, &["AB"]).unwrap();
        assert_eq!(t.text, "ABB");
        assert_eq!(t.segments, vec![("BB".into(), "q".into())]);
    }

    #[test]
    fn a_run_across_two_lines_does_not_raise_the_floor() {
        let t = tokenise("Bq", &['q'], 'A', &["xA", "Ay"]).unwrap();
        assert_eq!(t.segments[0].0, "AA");
    }
}
