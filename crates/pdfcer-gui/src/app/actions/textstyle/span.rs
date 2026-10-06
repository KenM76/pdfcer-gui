//! Restyling and decorating a character range (`OPERATOR_REQUESTS.md` O273).
//!
//! A range is two page-text positions and the text they covered when the
//! gesture was made. Both verbs re-extract first and decline with
//! `SpanMoved` if the range no longer reads that text, so a stale range never
//! touches the wrong letters.
//!
//! A restyle is one `format_text` call per show operator the range touches:
//! an operator wholly inside is addressed by pin alone; one the range cuts is
//! addressed by pin plus the covered characters as `find`, and by which of
//! their non-overlapping matches in that operator the cut is
//! (`FormatRequest::occurrence`), so the second `M10` of `M10 x M10` is the
//! one restyled. A cut that overlaps an earlier copy of itself (`aa` from the
//! middle of `aaa`) is no match in that count and is declined as
//! `SpanAmbiguous`.
//!
//! A decoration is a text-markup mark written into the page content under
//! the range's quads, in the text's own colour. It is not tied to the text:
//! moving or re-wrapping the text later leaves the line where it was.

use pdfcer_core::annot_author::{Color, MarkupSpec, TextMarkupKind};
use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::text_edit::{
    BlockRecognitionOptions, EditableTextModel, FormatRequest, TextPosition,
};
use pdfcer_core::text_extract::TextColor;

use super::StyleChange;
use crate::app::actions::text::Decoration;
use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::canvas::textsel::{self, PageContext, TextSelection};
use crate::text::status as t;

/// A range's two ends, start inclusive and end exclusive.
pub(in crate::app::actions) type Span = (TextPosition, TextPosition);

/// The range read against the current page, if it still names `expected`.
fn fresh(doc: &OpenDoc, page: usize, span: Span, expected: &str) -> Option<TextSelection> {
    let text = doc.provenance_page_text(page)?;
    let page_ref = doc.pages.get(page)?;
    let ctx = PageContext {
        text: &text,
        page: page_ref,
        index: page,
        epoch: doc.edit_epoch,
    };
    textsel::span(&ctx, span.0, span.1).filter(|s| s.text == expected)
}

/// One `format_text` call of a range restyle.
struct Piece {
    run: usize,
    req: FormatRequest,
}

/// The requests that restyle exactly `span`, in content order.
fn pieces(doc: &OpenDoc, page: usize, span: Span) -> Result<Vec<Piece>, t::TextStyleRefusal> {
    let text = doc
        .provenance_page_text(page)
        .ok_or(t::TextStyleRefusal::Unpinnable)?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let (from, to) = span;
    let mut out = Vec::new();
    for run in from.run..=to.run {
        let Some(r) = text.runs.get(run) else {
            continue;
        };
        let lo = if run == from.run { from.byte_offset } else { 0 };
        let hi = if run == to.run {
            to.byte_offset
        } else {
            r.text.len()
        };
        for op in crate::canvas::textedit::pin::operators_in_run(&model, &text, run) {
            let (a, b) = (lo.max(op.text.start), hi.min(op.text.end));
            if a >= b {
                continue;
            }
            let crate::canvas::textedit::pin::Pinned {
                span: pin, target, ..
            } = op.pin;
            let req = if a == op.text.start && b == op.text.end {
                FormatRequest::whole_operator(page, pin)
            } else {
                let whole = r.text.get(op.text.clone());
                let cut = r.text.get(a..b);
                let (Some(whole), Some(cut)) = (whole, cut) else {
                    return Err(t::TextStyleRefusal::SpanAmbiguous);
                };
                let n = occurrence(whole, cut, a - op.text.start)
                    .ok_or(t::TextStyleRefusal::SpanAmbiguous)?;
                FormatRequest::new(page, cut).pinned(pin).occurrence(n)
            };
            out.push(Piece {
                run,
                req: req.target(target),
            });
        }
    }
    Ok(out)
}

/// Which of `cut`'s non-overlapping matches in `whole`, counted from the left,
/// starts at byte `at`; `None` when none does.
fn occurrence(whole: &str, cut: &str, at: usize) -> Option<usize> {
    whole.match_indices(cut).position(|(start, _)| start == at)
}

/// Restyle exactly the characters `span` covers, then keep them selected.
pub(in crate::app::actions) fn apply(
    doc: &mut OpenDoc,
    page: usize,
    span: Span,
    expected: &str,
    change: &StyleChange,
) {
    if fresh(doc, page, span, expected).is_none() {
        refuse(page, t::TextStyleRefusal::SpanMoved, "moved");
        return;
    }
    let pieces = match pieces(doc, page, span) {
        Ok(pieces) if !pieces.is_empty() => pieces,
        Ok(_) => return refuse(page, t::TextStyleRefusal::NoRun, "no-glyphs"),
        Err(why) => return refuse(page, why, "pieces"),
    };
    let policy = doc.settings.style_policy;
    let total = pieces.len();
    let mut carried: Vec<String> = Vec::new();
    let mut applied = 0_usize;
    // Last-first: an edit only moves bytes at or after itself.
    for piece in pieces.into_iter().rev() {
        let req = change.stamp(piece.req);
        match super::format_op(doc, page, policy, &req) {
            Ok(notes) => {
                applied += 1;
                carried.extend(notes);
            }
            Err(error) => {
                if applied > 0 {
                    decline::record_text_style(t::TextStyleRefusal::PartOnly);
                }
                let run = piece.run;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "text-style-declined page={page} run={run} applied={applied} \
                         pieces={total} detail={error}"
                    )
                });
                break;
            }
        }
    }
    if applied > 0 {
        // One range is one piece of text to the operator, however many
        // operators it crossed, so no "N runs" sentence.
        super::emit_carried(doc, page, applied, 1, &carried);
    }
    doc.text_selection = fresh(doc, page, span, expected);
    let kept = doc.text_selection.is_some();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "text-span-style-applied page={page} change={} applied={applied} pieces={total} \
             reselected={kept}",
            change.label()
        )
    });
}

/// Draw an underline or strikethrough under exactly the characters `span`
/// covers, then keep them selected.
pub(in crate::app::actions) fn decorate(
    doc: &mut OpenDoc,
    page: usize,
    span: Span,
    expected: &str,
    kind: Decoration,
) {
    let Some(selection) = fresh(doc, page, span, expected) else {
        refuse(page, t::TextStyleRefusal::SpanMoved, "moved");
        return;
    };
    let color = colour_at(doc, page, span.0.run);
    let spec = MarkupSpec::TextMarkup {
        kind: match kind {
            Decoration::Underline => TextMarkupKind::Underline,
            Decoration::Strikethrough => TextMarkupKind::StrikeOut,
        },
        quads: selection.page_quads,
        color,
    };
    let label = "decorate-text"; // ui-text-exempt: funnel trace label
    crate::app::actions::markupdest::author(
        doc,
        label,
        page,
        &spec,
        &MarkupOptions::default(),
        true,
    );
    doc.text_selection = fresh(doc, page, span, expected);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("text-decorate-applied page={page} kind={kind:?}")
    });
}

/// The fill of `run`, as an annotation colour; black where it is unreadable.
fn colour_at(doc: &OpenDoc, page: usize, run: usize) -> Color {
    let fill = crate::canvas::textedit::pin::inspect(doc, page, run).and_then(|i| i.style.fill);
    match fill {
        Some(TextColor::Rgb(r, g, b)) => Color::Rgb(f64::from(r), f64::from(g), f64::from(b)),
        Some(TextColor::Gray(v)) => Color::Gray(f64::from(v)),
        Some(TextColor::Cmyk(c, m, y, k)) => {
            Color::Cmyk(f64::from(c), f64::from(m), f64::from(y), f64::from(k))
        }
        _ => Color::Gray(0.0),
    }
}

/// Say why on the bar and in the trace; change nothing.
fn refuse(page: usize, why: t::TextStyleRefusal, reason: &str) {
    decline::record_text_style(why);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("text-style-declined page={page} reason={reason}")
    });
}

#[cfg(test)]
mod tests {
    use super::occurrence;

    #[test]
    fn the_second_copy_is_the_second_match() {
        assert_eq!(occurrence("M10 x M10", "M10", 0), Some(0));
        assert_eq!(occurrence("M10 x M10", "M10", 6), Some(1));
    }

    #[test]
    fn a_cut_overlapping_an_earlier_copy_is_no_match() {
        assert_eq!(occurrence("aaa", "aa", 1), None);
        assert_eq!(occurrence("aaa", "aa", 0), Some(0));
    }
}
