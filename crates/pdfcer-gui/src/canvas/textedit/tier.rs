//! The narrowed fallback: when the request for the whole line finds nothing to
//! edit, send the engine only the show operators the edit touches.
//!
//! The whole-line request stays first because it is the one that lands a
//! change spread over per-glyph CAD operators of one text object. A line a
//! word processor wrote — every fragment its own `BT … ET`, a trailing object
//! holding one space — matches nothing as a whole, and the engine answers
//! `NotFound`; the narrowed request then reaches the one operator that
//! changed. An edit that genuinely spans text objects stays refused: the
//! engine cannot yet match across `BT … ET` (request G074).

use std::ops::Range;

use pdfcer_core::text_edit::{EditError, EditRequest, RefusalClass, RefusalKind};
use pdfcer_core::text_extract::{GlyphProvenance, PageText, TextRun};
use pdfcer_gui_base::editmodel::narrow::{self, Touched};

/// The request reaching only the touched operators, and what it touches.
pub struct Narrowed {
    pub request: EditRequest,
    pub touched: Touched,
}

/// Which request an attempt ended on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Line,
    Narrowed,
}

/// The narrowed request for changing `original` to `draft` in `run`, or `None`
/// when the run is one operator, its text is not `original`, its glyphs do
/// not group into contiguous operators, or the change lies only in text no
/// operator holds.
///
/// One touched operator is replaced whole — robust to kerning gaps the
/// extractor wrote as spaces — unless that operator also draws glyphs in
/// another run, where a whole replacement would erase them; then the
/// operator's own text in this run is found and replaced inside it.
#[must_use]
pub fn narrowed(
    text: &PageText,
    page: usize,
    run: usize,
    original: &str,
    draft: &str,
) -> Option<Narrowed> {
    let line = text.runs.get(run).filter(|r| r.text == original)?;
    let operators = operator_ranges(line)?;
    if operators.len() < 2 {
        return None;
    }
    let ranges: Vec<Range<usize>> = operators.iter().map(|(_, r)| r.clone()).collect();
    let touched = narrow::touched(&ranges, original, draft)?;
    let first = operators[touched.first].0;
    let span = first.operator_span;
    let mut request = if touched.operators() > 1 {
        EditRequest::spanning_from(
            page,
            span,
            &original[touched.original.clone()],
            &touched.replacement,
        )
    } else if drawn_elsewhere(text, run, first) {
        EditRequest::find_replace(
            page,
            &original[touched.original.clone()],
            &touched.replacement,
        )
        .pinned(span)
    } else {
        EditRequest::whole_operator(page, span, &touched.replacement)
    };
    request.target = super::pin::target_of(first);
    Some(Narrowed { request, touched })
}

/// Each show operator in `run` with the byte range of the run text its glyphs
/// produce, in order; `None` if a glyph has no provenance or an operator's
/// glyphs are interleaved with another's.
#[must_use]
pub fn operator_ranges(run: &TextRun) -> Option<Vec<(&GlyphProvenance, Range<usize>)>> {
    let mut out: Vec<(&GlyphProvenance, Range<usize>)> = Vec::new();
    for glyph in &run.glyphs {
        let p = glyph.provenance.as_ref()?;
        let start = glyph.text_start as usize;
        let end = start + glyph.text_len as usize;
        match out.last_mut() {
            Some((held, range)) if same_operator(held, p) => {
                range.start = range.start.min(start);
                range.end = range.end.max(end);
            }
            _ => {
                if out.iter().any(|(held, _)| same_operator(held, p)) {
                    return None;
                }
                out.push((p, start..end));
            }
        }
    }
    Some(out)
}

fn same_operator(a: &GlyphProvenance, b: &GlyphProvenance) -> bool {
    a.operator_span == b.operator_span && a.content_stream == b.content_stream
}

fn drawn_elsewhere(text: &PageText, run: usize, operator: &GlyphProvenance) -> bool {
    text.runs.iter().enumerate().any(|(i, r)| {
        i != run
            && r.glyphs
                .iter()
                .filter_map(|g| g.provenance.as_ref())
                .any(|p| same_operator(p, operator))
    })
}

/// Whether a refusal of the line request is worth the narrowed retry: the
/// engine found nothing to edit, and the pin itself was not stale.
#[must_use]
pub fn retries(error: &EditError) -> bool {
    error.refusal_kind() == RefusalKind::NotFound
        && !matches!(error, EditError::PinnedSpanNotFound { .. })
}
