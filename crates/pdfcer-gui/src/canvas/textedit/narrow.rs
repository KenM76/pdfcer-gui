//! # `canvas::textedit::narrow` — edit the show operator the change touched, not the whole line
//!
//! ## The contract
//!
//! In: the run the caret is in, the text it held, and the text the operator
//! typed. Out: **one** show operator inside that run, and the whole new text of
//! that operator — the `(pin, replacement)` pair an
//! [`EditRequest`](pdfcer_core::text_edit::EditRequest) needs for the
//! whole-operator form (`pinned_span` set, `find` empty). [`None`] when the
//! change does not lie inside a single operator, which is the caller's signal
//! to keep the spanning form.
//!
//! Pure: a `PageText`, a model and two strings in, an answer out. No session,
//! no document, no engine call.
//!
//! ## Why it exists
//!
//! A producer that emits one glyph per show operator writes a title-block line
//! as several independently positioned fragments. Asking the engine to replace
//! the **whole run** then spans those fragments, and a spanning edit puts the
//! replacement into the operator holding the match's *end* and empties the ones
//! before it. Where the engine's compensation for that collapse declines to
//! run, the line is drawn from its final fragment's origin instead of its
//! first — measured at **+240.16 pt** on the operator's own sheet, against an
//! advance delta of 0.499 pt. Engine request `G028`.
//!
//! Narrowing removes the condition rather than the symptom: with the pin on the
//! one operator the change touched, `operators_spanned` is 1, no operator is
//! emptied, and there is nothing to compensate. Measured on the same line of
//! the same sheet, the left edge holds at 474.16 pt to the last decimal and the
//! right edge does not move at all.
//!
//! ⚠ This is a **workaround for `G028` and comes out when `G028` lands** — at
//! which point the spanning form is correct for every shape and narrowing is an
//! optimisation nobody needs. Deleting this module is the whole of that change.
//!
//! ## What it deliberately does not do
//!
//! It does not narrow the `find` string. A narrow `find` is ambiguous — the
//! changed span alone (`"n"` → `"nt"`) occurs 33 times on one measured page,
//! where the whole run occurs once — and [`super::plan`]'s header carries the
//! argument for why that ambiguity is unacceptable on a signed drawing. **A pin
//! is not a string**: it names one byte span in one named buffer, so the
//! whole-operator form has no occurrence to choose between. The ambiguity
//! `plan` rejected and the narrowing done here are different things that share
//! a word.
//!
//! ## Derived characters, and why a gap refuses
//!
//! `text_extract` synthesises inter-glyph spacing, so a run's text contains
//! characters no show operator wrote. A change falling in such a gap belongs to
//! no operator and cannot be expressed as a whole-operator replacement; this
//! module answers [`None`] there and the caller keeps the spanning form. That
//! is the honest direction: the spanning form is wrong about *placement* on
//! some files, whereas a guessed operator would be wrong about *which text*.

use pdfcer_core::text_edit::{EditableTextModel, GlyphRef};
use pdfcer_core::text_extract::PageText;

use super::pin::Pinned;

/// One show operator, and the text it must now show.
#[derive(Debug, Clone, PartialEq)]
pub struct Narrowed {
    /// The operator to pin. `find` must be cleared beside it.
    pub pin: Pinned,
    /// The **whole** new text of that operator — not the changed part.
    pub replacement: String,
}

/// The byte extent one show operator covers in its run's text.
#[derive(Debug, Clone, Copy)]
struct Extent {
    start: usize,
    end: usize,
}

/// **The changed region of `original`, as a byte range plus the bytes that
/// replace it.**
///
/// Common prefix and common suffix, walked over `char_indices` so both bounds
/// land on character boundaries and can index either string. Returns
/// `(start, end_in_original, inserted)`.
///
/// A pure insertion gives `start == end`; a pure deletion gives an empty
/// `inserted`. Both are real edits and both are handled by [`narrow`].
fn changed_region(original: &str, replacement: &str) -> (usize, usize, String) {
    let prefix = original
        .char_indices()
        .zip(replacement.char_indices())
        .take_while(|((_, a), (_, b))| a == b)
        .map(|((i, c), _)| i + c.len_utf8())
        .last()
        .unwrap_or(0);

    // ★ The suffix walk starts AFTER the prefix on both strings, so a repeated
    // tail cannot be counted at both ends — `"aa"` → `"aaa"` has a one-character
    // prefix and must not also claim a two-character suffix. Unbounded, the two
    // overlap and the subtraction below underflows.
    let (otail, rtail) = (&original[prefix..], &replacement[prefix..]);
    let mut suffix = 0usize;
    for (a, b) in otail.chars().rev().zip(rtail.chars().rev()) {
        if a != b {
            break;
        }
        suffix += a.len_utf8();
    }

    let end = original.len() - suffix;
    let inserted = replacement[prefix..replacement.len() - suffix].to_owned();
    (prefix, end, inserted)
}

/// **The byte extent of every show operator in `run`, in content order.**
///
/// One entry per operator, paired with the pin that names it. An operator's
/// extent runs from its first glyph's first byte to its last glyph's last byte
/// in the run's text — contiguous by the engine's own measured guarantee
/// (29,246 operator groups over its corpus, zero non-contiguous).
fn extents(
    model: &EditableTextModel<'_>,
    page_text: &PageText,
    run: usize,
) -> Vec<(Extent, Pinned)> {
    let mut out: Vec<(Extent, Pinned)> = Vec::new();
    let Some(text) = page_text.runs.get(run) else {
        return out;
    };
    for (index, glyph) in text.glyphs.iter().enumerate() {
        let Some(p) = model.provenance(GlyphRef::new(run, index)) else {
            continue;
        };
        let start = glyph.text_start as usize;
        let end = start + glyph.text_len as usize;
        match out.last_mut() {
            Some((extent, pin)) if pin.span == p.operator_span => {
                extent.start = extent.start.min(start);
                extent.end = extent.end.max(end);
            }
            _ => out.push((
                Extent { start, end },
                Pinned {
                    span: p.operator_span,
                    target: super::pin::target_of(p),
                    text_matrix: p.text_matrix,
                    ctm: p.ctm,
                },
            )),
        }
    }
    out
}

/// **The one operator the change lies inside, and its whole new text.**
///
/// `None` — keep the spanning form — when the run has fewer than two
/// operators, when the change spans two of them, when it falls in a synthesised
/// gap between them, or when nothing changed at all.
///
/// A pure insertion at a boundary is attributed to the operator **ending** at
/// that byte as well as to the one starting there, and two candidates refuse:
/// guessing which fragment a boundary keystroke belongs to would put the
/// character on the wrong side of a positioning operator, which moves it across
/// the gap rather than typing it.
#[must_use]
pub fn narrow(
    model: &EditableTextModel<'_>,
    page_text: &PageText,
    run: usize,
    original: &str,
    replacement: &str,
) -> Option<Narrowed> {
    if original == replacement {
        return None;
    }
    let (start, end, inserted) = changed_region(original, replacement);
    let ops = extents(model, page_text, run);
    if ops.len() < 2 {
        // One operator, or none. `plan`'s existing whole-operator form already
        // covers that and does not need a second route to the same request.
        return None;
    }

    let mut hit: Option<&(Extent, Pinned)> = None;
    for op in &ops {
        let touches = if start == end {
            start >= op.0.start && start <= op.0.end
        } else {
            start < op.0.end && end > op.0.start
        };
        if !touches {
            continue;
        }
        if hit.is_some() {
            return None;
        }
        hit = Some(op);
    }
    let (extent, pin) = hit?;

    // A range that touched exactly one extent can still begin in the gap before
    // it or end in the gap after it, and neither is expressible as a
    // whole-operator replacement.
    if start < extent.start || end > extent.end {
        return None;
    }

    let mut text = original.get(extent.start..start)?.to_owned();
    text.push_str(&inserted);
    text.push_str(original.get(end..extent.end)?);
    Some(Narrowed {
        pin: *pin,
        replacement: text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_substitution_in_the_middle_is_the_changed_region() {
        let (s, e, ins) = changed_region("SPACERS", "SPACERT");
        assert_eq!((s, e, ins.as_str()), (6, 7, "T"));
    }

    #[test]
    fn an_insertion_has_an_empty_range_and_is_not_a_deletion() {
        let (s, e, ins) = changed_region("ABC", "ABXC");
        assert_eq!((s, e, ins.as_str()), (2, 2, "X"));
    }

    #[test]
    fn a_deletion_has_an_empty_insertion() {
        let (s, e, ins) = changed_region("ABXC", "ABC");
        assert_eq!((s, e, ins.as_str()), (2, 3, ""));
    }

    /// ★ **A repeated tail is not counted twice.** Without the prefix bound on
    /// the suffix walk, `"aa"` → `"aaa"` reports a one-character prefix *and* a
    /// two-character suffix; they overlap, and `original.len() - suffix`
    /// underflows into a panic on a commit the operator can type by accident.
    #[test]
    fn a_repeated_character_does_not_let_the_prefix_and_suffix_overlap() {
        let (s, e, ins) = changed_region("aa", "aaa");
        assert!(s <= e, "the range must not be inverted: {s}..{e}");
        assert_eq!(ins.len(), 1);
    }

    /// ★★ **A multi-byte character does not split.** Both walks step whole
    /// `char`s, so the bounds are character boundaries and the slices in
    /// [`narrow`] cannot panic inside a code point.
    #[test]
    fn a_multibyte_character_gives_char_boundaries() {
        let (s, e, ins) = changed_region("café", "cafe");
        assert!("café".is_char_boundary(s) && "café".is_char_boundary(e));
        assert_eq!(ins, "e");
    }

    /// **An unchanged string is a degenerate region**, which is why [`narrow`]
    /// tests for equality before it looks at a single glyph: a zero-width range
    /// at the end of the run would otherwise be attributed to the last
    /// operator and rewrite it with its own text.
    #[test]
    fn an_unchanged_string_has_a_degenerate_region() {
        let (s, e, ins) = changed_region("ABC", "ABC");
        assert_eq!((s, e), (3, 3));
        assert!(ins.is_empty());
    }

    /// ★★★ **The operator's own line, reduced to the shape that matters.**
    ///
    /// The text of `#2 USE SPACERS 8 9 10 11 IF REQUIRED.` as his producer
    /// writes it: the first fifteen characters in one show operator, the rest
    /// in eight more. Correcting the `S` must land wholly inside the first,
    /// and the replacement must be that operator's *whole* text rather than the
    /// changed letter.
    #[test]
    fn the_changed_region_of_his_line_lies_inside_the_first_operator() {
        let line = "#2 USE SPACERS 8 9 10 11 IF REQUIRED.";
        let fixed = "#2 USE SPACERT 8 9 10 11 IF REQUIRED.";
        let (s, e, ins) = changed_region(line, fixed);
        assert_eq!(ins, "T");
        assert!(
            e <= 15,
            "the change must fall inside the first operator's 15 characters, got {s}..{e}"
        );
        assert_eq!(&line[..s], "#2 USE SPACER");
    }
}
