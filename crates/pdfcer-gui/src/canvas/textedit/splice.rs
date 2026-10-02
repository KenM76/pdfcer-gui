//! A partial preview spliced into its line: the engine lays out only the part
//! of the line it rewrites, and the rest keeps its page positions.
//!
//! Contract: the result's stops are one per character of the whole draft plus
//! the end, so the caret, the selection and the hit test index it exactly as
//! they index a whole-line preview. The commit re-lays the rest of the show
//! operator that holds the rewritten part, so those glyphs (the tail) are drawn
//! moved by the part's change in advance; glyphs of other operators stay where
//! they are, visible in the page render beneath, which is where the commit
//! will leave them.

use egui::{Pos2, Vec2};
use pdfcer_core::text_edit::TextEditPreview;
use pdfcer_core::text_extract::{ExtractedGlyph, TextRun};
use pdfcer_gui_base::text::previewfallback::PreviewFallback;
use pdfcer_render::tiny_skia::Path;

use super::shaped::Shaped;
use crate::app::settings::SettingsExt;
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
    let tail = tail(line, &span, &preview.font_resource);
    let end = *shaped.stops.last().ok_or(unpaired)?;
    let delta = if tail.is_empty() {
        Vec2::ZERO
    } else {
        end - char_stops(line, &[], Vec2::ZERO)[before + held]
    };
    let stops = char_stops(line, &tail, delta);
    let mut spliced: Vec<Pos2> = stops[..before].to_vec();
    spliced.extend_from_slice(&shaped.stops[..shaped.stops.len() - 1]);
    if span.end < original.len() {
        spliced.extend_from_slice(&stops[before + held..]);
    } else {
        spliced.push(end);
    }
    if spliced.len() != draft.chars().count() + 1 {
        return Err(unpaired);
    }
    let touched = line.glyphs.iter().filter(|g| {
        let start = g.text_start as usize;
        span.start <= start && start < span.end
    });
    let blank = extent(touched.chain(tail.iter().copied()), Vec2::ZERO).ok_or(unpaired)?;
    if let Some(moved) = extent(tail.iter().copied(), delta) {
        shaped
            .outlines
            .extend(outlines(doc, preview, &tail, delta)?);
        let b = &mut shaped.bbox;
        let moved = moved.map(f64::from);
        *b = [
            b[0].min(moved[0]),
            b[1].min(moved[1]),
            b[2].max(moved[2]),
            b[3].max(moved[3]),
        ];
    }
    shaped.stops = spliced;
    shaped.text = draft.to_owned();
    shaped.blank = Some(blank);
    Ok(shaped)
}

/// The glyphs after `span` that the commit re-lays: the rest of the show
/// operator that produced the span's last glyph, when that operator shows in
/// the preview's font (`font`), so the preview's codes name the same glyphs.
fn tail<'l>(
    line: &'l TextRun,
    span: &std::ops::Range<usize>,
    font: &[u8],
) -> Vec<&'l ExtractedGlyph> {
    let owner = line
        .glyphs
        .iter()
        .rfind(|g| {
            let start = g.text_start as usize;
            span.start <= start && start < span.end
        })
        .and_then(|g| g.provenance.as_ref());
    let Some(owner) = owner else {
        return Vec::new();
    };
    line.glyphs
        .iter()
        .filter(|g| {
            g.text_start as usize >= span.end
                && g.provenance.as_ref().is_some_and(|p| {
                    p.content_stream == owner.content_stream
                        && p.operator_span == owner.operator_span
                        && p.font_resource.as_deref() == Some(font)
                })
        })
        .collect()
}

/// The outlines of `tail`, each moved by `delta`, laid out through the
/// preview's own font and glyph matrix; `NoOutlines` when the font yields none.
fn outlines(
    doc: &OpenDoc,
    preview: &TextEditPreview,
    tail: &[&ExtractedGlyph],
    delta: Vec2,
) -> Result<Vec<Option<Path>>, PreviewFallback> {
    let template = *preview.glyphs.first().ok_or(PreviewFallback::Unpaired)?;
    let mut moved = preview.clone();
    moved.glyphs = tail
        .iter()
        .map(|g| {
            let mut glyph = template;
            glyph.ch = None;
            glyph.code = g.code;
            glyph.matrix[4] = f64::from(g.x + delta.x);
            glyph.matrix[5] = f64::from(g.y + delta.y);
            glyph
        })
        .collect();
    let laid = pdfcer_render::edit_preview::preview_outlines(
        &doc.session.view(),
        &moved,
        &doc.settings.render_options().fonts,
    );
    if laid.skipped.is_some() {
        return Err(PreviewFallback::NoOutlines);
    }
    Ok(laid.glyphs)
}

/// One caret stop per character of `line` plus the end, in page space, with
/// the glyphs of `moved` displaced by `delta`. A character no glyph produced —
/// a gap the extractor synthesised — stops at the end of the glyph before it.
fn char_stops(line: &TextRun, moved: &[&ExtractedGlyph], delta: Vec2) -> Vec<Pos2> {
    let shift = |g: &ExtractedGlyph| {
        if moved.iter().any(|m| std::ptr::eq(*m, g)) {
            delta
        } else {
            Vec2::ZERO
        }
    };
    let end_of = |g: &ExtractedGlyph| {
        Pos2::new(
            g.x + g.advance * g.direction.0,
            g.y + g.advance * g.direction.1,
        ) + shift(g)
    };
    let mut out = Vec::with_capacity(line.text.len() + 1);
    let mut last_end = line
        .glyphs
        .first()
        .map_or(Pos2::ZERO, |g| Pos2::new(g.x, g.y) + shift(g));
    for (at, _) in line.text.char_indices() {
        let owner = line.glyphs.iter().find(|g| {
            let start = g.text_start as usize;
            start <= at && at < start + g.text_len as usize
        });
        match owner {
            Some(g) => {
                out.push(Pos2::new(g.x, g.y) + shift(g));
                last_end = end_of(g);
            }
            None => out.push(last_end),
        }
    }
    out.push(line.glyphs.last().map_or(last_end, end_of));
    out
}

/// The page-space box `[x0, y0, x1, y1]` of `glyphs`, each moved by `delta`.
fn extent<'g>(glyphs: impl Iterator<Item = &'g ExtractedGlyph>, delta: Vec2) -> Option<[f32; 4]> {
    glyphs
        .map(|g| {
            let (x0, x1) = (g.x, g.x + g.advance * g.direction.0);
            let (y0, y1) = (g.y - g.size * 0.25, g.y + g.size * 0.9);
            [
                x0.min(x1) + delta.x,
                y0 + delta.y,
                x0.max(x1) + delta.x,
                y1 + delta.y,
            ]
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
