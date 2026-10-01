//! A partial preview spliced into its line: the engine lays out only the part
//! of the line it rewrites, and the rest keeps its page positions.
//!
//! Contract: the result's stops are one per character of the whole draft plus
//! the end, so the caret, the selection and the hit test index it exactly as
//! they index a whole-line preview. Only the touched operators' glyph area is
//! blanked; the untouched glyphs stay visible in the page render beneath,
//! which is where the commit will leave them.

use egui::Pos2;
use pdfcer_core::text_edit::TextEditPreview;
use pdfcer_core::text_extract::TextRun;
use pdfcer_gui_base::text::previewfallback::PreviewFallback;

use super::shaped::Shaped;
use crate::app::state::OpenDoc;

/// What a partial preview lays out: bytes `span` of the run's text, rewritten
/// as `replacement`.
pub struct Part<'a> {
    pub span: std::ops::Range<usize>,
    pub replacement: &'a str,
}

/// The whole-line `Shaped` for `draft`, from a preview of `part` alone; the
/// reason when the preview cannot be shaped, `Unpaired` when the counts do not
/// reconcile.
pub fn shape(
    doc: &OpenDoc,
    preview: &TextEditPreview,
    page: usize,
    run: usize,
    part: &Part<'_>,
    draft: &str,
) -> Result<Shaped, PreviewFallback> {
    let mut shaped = super::shaped::shape(doc, preview, part.replacement)?;
    let unpaired = PreviewFallback::Unpaired;
    let text = doc.provenance_page_text(page).ok_or(unpaired)?;
    let line = text.runs.get(run).ok_or(unpaired)?;
    let span = part.span.clone();
    let original = &line.text;
    if span.end > original.len() || !original.is_char_boundary(span.start) {
        return Err(unpaired);
    }
    let before = original[..span.start].chars().count();
    let held = original[span.clone()].chars().count();
    let stops = char_stops(line);
    let mut spliced: Vec<Pos2> = stops[..before].to_vec();
    spliced.extend_from_slice(&shaped.stops[..shaped.stops.len() - 1]);
    if span.end < original.len() {
        spliced.extend_from_slice(&stops[before + held..]);
    } else {
        spliced.push(*shaped.stops.last().ok_or(unpaired)?);
    }
    if spliced.len() != draft.chars().count() + 1 {
        return Err(unpaired);
    }
    shaped.stops = spliced;
    shaped.text = draft.to_owned();
    shaped.blank = Some(glyph_extent(line, span).ok_or(unpaired)?);
    Ok(shaped)
}

/// One caret stop per character of `line` plus the end, in page space. A
/// character no glyph produced — a gap the extractor synthesised — stops at
/// the end of the glyph before it.
fn char_stops(line: &TextRun) -> Vec<Pos2> {
    let end_of = |g: &pdfcer_core::text_extract::ExtractedGlyph| {
        Pos2::new(
            g.x + g.advance * g.direction.0,
            g.y + g.advance * g.direction.1,
        )
    };
    let mut out = Vec::with_capacity(line.text.len() + 1);
    let mut last_end = line
        .glyphs
        .first()
        .map_or(Pos2::ZERO, |g| Pos2::new(g.x, g.y));
    for (at, _) in line.text.char_indices() {
        let owner = line.glyphs.iter().find(|g| {
            let start = g.text_start as usize;
            start <= at && at < start + g.text_len as usize
        });
        match owner {
            Some(g) => {
                out.push(Pos2::new(g.x, g.y));
                last_end = end_of(g);
            }
            None => out.push(last_end),
        }
    }
    out.push(line.glyphs.last().map_or(last_end, end_of));
    out
}

/// The page-space box of the glyphs producing `span` of the line's text.
fn glyph_extent(line: &TextRun, span: std::ops::Range<usize>) -> Option<[f32; 4]> {
    line.glyphs
        .iter()
        .filter(|g| {
            let start = g.text_start as usize;
            span.start <= start && start < span.end
        })
        .map(|g| {
            let (x0, x1) = (g.x, g.x + g.advance * g.direction.0);
            let (y0, y1) = (g.y - g.size * 0.25, g.y + g.size * 0.9);
            [x0.min(x1), y0, x0.max(x1), y1]
        })
        .reduce(|a, b| {
            [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        })
}
