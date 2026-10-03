//! Where each Find hit sits in the page's show operators, and the rewrite of
//! each operator that replaces every hit inside it.
//!
//! Contract: a hit is rewritten only when its whole text lies inside one show
//! operator that draws nothing on any other line; every other hit is returned
//! as a [`Skip`] with its reason. Rewrites come back in the order they must be
//! applied: per page and content buffer, last operator first, so an applied
//! rewrite never moves a span still to be used.

use std::ops::Range;

use pdfcer_core::annot_author::Quad;
use pdfcer_core::span::ByteSpan;
use pdfcer_core::text_edit::{BlockRecognitionOptions, EditTarget, EditableTextModel};
use pdfcer_core::text_extract::PageText;

use crate::app::state::OpenDoc;
use crate::canvas::textedit::pin::operators_in_run;

/// One hit the bar found: its page and its box in PDF user space.
#[derive(Debug, Clone, Copy)]
pub(super) struct Wanted {
    pub page: usize,
    pub quad: Quad,
}

/// One show operator's new text.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Rewrite {
    pub page: usize,
    pub span: ByteSpan,
    pub target: EditTarget,
    /// The operator's text as extracted, which the edit must find under the pin.
    pub old: String,
    pub new: String,
    /// How many hits this rewrite replaces.
    pub hits: usize,
}

/// Why a hit was left as it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Skip {
    /// No line on the page holds the whole hit (it wraps, or spans two lines).
    NotOnOneLine,
    /// The hit runs across a change of font, size or colour.
    CrossesStyles,
    /// The operator drawing it also draws text on another line.
    SharedOperator,
}

/// The rewrites, and the hits left alone.
#[derive(Debug, Default)]
pub(super) struct Located {
    pub rewrites: Vec<Rewrite>,
    pub skipped: Vec<Skip>,
}

/// Locate every wanted hit and plan the rewrites that replace them with `with`.
pub(super) fn locate(
    doc: &OpenDoc,
    query: &str,
    case_sensitive: bool,
    with: &str,
    wanted: &[Wanted],
) -> Located {
    let mut out = Located::default();
    let mut pages: Vec<usize> = wanted.iter().map(|w| w.page).collect();
    pages.dedup();
    for page in pages {
        let quads: Vec<Quad> = wanted
            .iter()
            .filter(|w| w.page == page)
            .map(|w| w.quad)
            .collect();
        let Some(text) = doc.provenance_page_text(page) else {
            out.skipped
                .extend(std::iter::repeat_n(Skip::NotOnOneLine, quads.len()));
            continue;
        };
        on_page(&text, page, query, case_sensitive, with, &quads, &mut out);
    }
    out
}

/// The work of [`locate`] for one page.
fn on_page(
    text: &PageText,
    page: usize,
    query: &str,
    case_sensitive: bool,
    with: &str,
    quads: &[Quad],
    out: &mut Located,
) {
    let model = EditableTextModel::recognize(text, &BlockRecognitionOptions::default());
    let mut claimed = vec![false; quads.len()];
    // (run, operator index) → the hit ranges inside it.
    let mut held: Vec<(usize, usize, Vec<Range<usize>>)> = Vec::new();
    let mut operators = Vec::with_capacity(text.runs.len());
    for (r, run) in text.runs.iter().enumerate() {
        let ops = operators_in_run(&model, text, r);
        for range in matches(&run.text, query, case_sensitive) {
            let Some(q) = anchor(run, range.start).and_then(|at| claim(quads, &claimed, at)) else {
                continue;
            };
            claimed[q] = true;
            let Some(o) = ops.iter().position(|op| contains(&op.text, &range)) else {
                out.skipped.push(Skip::CrossesStyles);
                continue;
            };
            match held.iter_mut().find(|(hr, ho, _)| *hr == r && *ho == o) {
                Some((_, _, ranges)) => ranges.push(range),
                None => held.push((r, o, vec![range])),
            }
        }
        operators.push(ops);
    }
    out.skipped
        .extend(claimed.iter().filter(|c| !**c).map(|_| Skip::NotOnOneLine));
    let mut rewrites = Vec::new();
    for (r, o, ranges) in held {
        let op = &operators[r][o];
        let shared = operators.iter().enumerate().any(|(other, ops)| {
            other != r
                && ops
                    .iter()
                    .any(|x| x.pin.span == op.pin.span && x.pin.target == op.pin.target)
        });
        if shared {
            out.skipped
                .extend(std::iter::repeat_n(Skip::SharedOperator, ranges.len()));
            continue;
        }
        let old = &text.runs[r].text[op.text.clone()];
        let local: Vec<Range<usize>> = ranges
            .iter()
            .map(|h| h.start - op.text.start..h.end - op.text.start)
            .collect();
        rewrites.push(Rewrite {
            page,
            span: op.pin.span,
            target: op.pin.target,
            old: old.to_owned(),
            new: splice(old, &local, with),
            hits: ranges.len(),
        });
    }
    // Last operator first within each buffer: an applied rewrite changes the
    // bytes after its own span only.
    rewrites.sort_by_key(|r| std::cmp::Reverse(r.span.start));
    out.rewrites.extend(rewrites);
}

/// Every non-overlapping occurrence of `query` in `text`, as byte ranges,
/// left to right. Case-insensitive compares character by character through
/// `char::to_lowercase`, so a byte range always lies on `text`'s own boundaries.
#[must_use]
pub(super) fn matches(text: &str, query: &str, case_sensitive: bool) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    if query.is_empty() {
        return out;
    }
    let mut from = 0;
    while from < text.len() {
        match match_at(&text[from..], query, case_sensitive) {
            Some(len) => {
                out.push(from..from + len);
                from += len;
            }
            None => {
                from += text[from..].chars().next().map_or(1, char::len_utf8);
            }
        }
    }
    out
}

/// The byte length of `text`'s prefix that matches `query`, if it does.
fn match_at(text: &str, query: &str, case_sensitive: bool) -> Option<usize> {
    let mut have = text.char_indices();
    for want in query.chars() {
        let (_, got) = have.next()?;
        let same = if case_sensitive {
            got == want
        } else {
            got.to_lowercase().eq(want.to_lowercase())
        };
        if !same {
            return None;
        }
    }
    Some(have.next().map_or(text.len(), |(at, _)| at))
}

/// `text` with each of `ranges` (ascending, disjoint) replaced by `with`.
#[must_use]
pub(super) fn splice(text: &str, ranges: &[Range<usize>], with: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    for r in ranges {
        out.push_str(&text[at..r.start]);
        out.push_str(with);
        at = r.end;
    }
    out.push_str(&text[at..]);
    out
}

fn contains(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

/// The origin and size of the glyph whose text holds byte `at`.
fn anchor(run: &pdfcer_core::text_extract::TextRun, at: usize) -> Option<(f64, f64, f64)> {
    run.glyphs
        .iter()
        .find(|g| {
            let s = g.text_start as usize;
            s <= at && at < s + g.text_len as usize
        })
        .map(|g| (f64::from(g.x), f64::from(g.y), f64::from(g.size)))
}

/// The first unclaimed quad whose box, grown by a quarter of the glyph size,
/// holds the glyph origin `at`.
fn claim(quads: &[Quad], claimed: &[bool], at: (f64, f64, f64)) -> Option<usize> {
    let (x, y, size) = at;
    let slack = size.abs() * 0.25;
    quads.iter().enumerate().position(|(i, q)| {
        let xs = [q.ul.0, q.ur.0, q.ll.0, q.lr.0];
        let ys = [q.ul.1, q.ur.1, q.ll.1, q.lr.1];
        let (x0, x1) = (min(&xs) - slack, max(&xs) + slack);
        let (y0, y1) = (min(&ys) - slack, max(&ys) + slack);
        !claimed[i] && (x0..=x1).contains(&x) && (y0..=y1).contains(&y)
    })
}

fn min(v: &[f64; 4]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}

fn max(v: &[f64; 4]) -> f64 {
    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_ignores_case_only_when_asked() {
        assert_eq!(matches("The cat the", "the", false), vec![0..3, 8..11]);
        assert_eq!(matches("The cat the", "the", true), vec![8..11]);
    }

    #[test]
    fn matches_do_not_overlap() {
        assert_eq!(matches("aaaa", "aa", true), vec![0..2, 2..4]);
    }

    #[test]
    fn a_match_lands_on_character_boundaries() {
        assert_eq!(matches("café Café", "CAFÉ", false), vec![0..5, 6..11]);
    }

    #[test]
    fn an_empty_query_matches_nothing() {
        assert!(matches("abc", "", false).is_empty());
    }

    #[test]
    fn splicing_replaces_every_range_in_place() {
        assert_eq!(splice("the cat the", &[0..3, 8..11], "a"), "a cat a");
        assert_eq!(splice("xx", &[], "a"), "xx");
    }
}
