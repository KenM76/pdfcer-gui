//! # `canvas::textedit::promote` — Enter in a line of existing text opens its
//! paragraph
//!
//! A run draft edits one show operator, which cannot hold a line break; its
//! paragraph can, through `EditSession::edit_block_text`. [`open`] re-anchors a
//! run draft on the run's paragraph: the paragraph's text with the run's slice
//! replaced by the draft, the caret, selection and draft history shifted to
//! match. It declines, and leaves the draft as it was, when the engine would
//! refuse the rewrite or the paragraph mixes looks the rewrite would flatten.
//! [`widen`] does the same for a click on any line of a paragraph of two or
//! more lines, so the paragraph re-opens as one draft.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/promote.md`.

use pdfcer_core::content::ContentStream;
use pdfcer_core::text_edit::{
    BlockEditOptions, EditableTextModel, GlyphRef, Line, LineEnd, LineEndSource, LineMarks,
    TextPosition, detect_cell_regions, reflow_recognition_options,
};
use pdfcer_gui_base::text::draftnote::DraftNote;
use pdfcer_gui_base::text::textedit::EnterRefusal;

use super::{Anchor, Draft};
use crate::app::state::OpenDoc;

/// The options a paragraph draft is previewed and committed under: the
/// caret's typing disposition and the installed faces, no fallback face, and
/// the engine's own wrap width (the box's, or a table cell's inner width).
#[must_use]
pub fn options(doc: &OpenDoc) -> BlockEditOptions {
    BlockEditOptions::new().with_edit_options(super::installed::augmented(
        doc,
        pdfcer_gui_base::editmodel::disposition::typing(),
    ))
}

/// A run's paragraph: its engine index, its box (PDF user space,
/// `llx, lly, urx, ury`) and its text either side of the run.
struct Paragraph {
    block: usize,
    run: usize,
    bbox: (f64, f64, f64, f64),
    lines: usize,
    breaks: usize,
    /// Whether its line ends were read from pdfcer's marks or inferred.
    source: LineEndSource,
    before: String,
    after: String,
}

/// Why a run did not open into its paragraph, with the trace token.
struct Declined {
    why: EnterRefusal,
    token: &'static str,
}

impl Declined {
    /// What the operator is told when a click opened less than the paragraph;
    /// `None` where the run is all there is (one line, no paragraph).
    fn note(&self) -> Option<DraftNote> {
        match self.token {
            "mixed-looks" => Some(DraftNote::LineOnlyMixedLooks),
            "engine-refuses" => Some(DraftNote::LineOnlyRefused),
            "recognised-word" => Some(DraftNote::RecognisedWordOnly),
            _ => None,
        }
    }

    const fn no_paragraph(token: &'static str) -> Self {
        Self {
            why: EnterRefusal::NoParagraph,
            token,
        }
    }
}

/// **Re-anchor `draft` (an `Anchor::Run`) on its paragraph**, ready for the
/// line break Enter is about to insert, and say where its line ends were
/// read from. `Err` leaves `draft` untouched.
///
/// # Errors
///
/// The refusal Enter reports.
pub fn open(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &mut Draft,
) -> Result<LineEndSource, EnterRefusal> {
    let Anchor::Run { run, original } = &draft.anchor else {
        return Err(EnterRefusal::NoParagraph);
    };
    let run = *run;
    let result = paragraph(doc, draft.page, run, original).and_then(|p| {
        let promoted = promoted(draft, &p);
        preflight(doc, draft.page, p.block, &promoted)?;
        Ok((p, promoted))
    });
    match result {
        Ok((p, promoted)) => {
            super::history::embed(ctx, &p.before, &p.after);
            *draft = promoted;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-promoted page={} run={run} block={} len={} caret={} source={}",
                    draft.page,
                    p.block,
                    draft.text.chars().count(),
                    draft.caret,
                    p.source.as_str()
                )
            });
            Ok(p.source)
        }
        Err(d) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-enter-declined page={} run={run} reason={}",
                    draft.page, d.token
                )
            });
            Err(d.why)
        }
    }
}

/// **Re-anchor a clicked run draft on its paragraph when the paragraph has
/// two or more lines**, its text unchanged, and say where its line ends were
/// read from. Leaves `draft` as it was, and returns `None`, when the run is a
/// paragraph of its own or the engine would not rewrite the paragraph as it
/// stands.
///
/// # Errors
///
/// The note owed to the operator when the paragraph exists and could not be
/// opened whole; `Err(None)` when the run is all there is.
pub fn widen(doc: &OpenDoc, draft: &mut Draft) -> Result<LineEndSource, Option<DraftNote>> {
    let Anchor::Run { run, original } = &draft.anchor else {
        return Err(None);
    };
    let run = *run;
    let started = std::time::Instant::now();
    let result = recognised(doc, draft.page, run)
        .and_then(|()| paragraph(doc, draft.page, run, original))
        .and_then(|p| {
            if p.lines < 2 {
                return Err(Declined::no_paragraph("one-line"));
            }
            let promoted = promoted(draft, &p);
            rewrite(doc, draft.page, p.block, &promoted.text)?;
            Ok((p, promoted))
        });
    let ms = started.elapsed().as_millis();
    match result {
        Ok((p, promoted)) => {
            *draft = promoted;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-widened page={} run={run} block={} lines={} breaks={} len={} \
                     source={} ms={ms}",
                    draft.page,
                    p.block,
                    p.lines,
                    p.breaks,
                    draft.text.chars().count(),
                    p.source.as_str()
                )
            });
            Ok(p.source)
        }
        Err(d) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-widen-declined page={} run={run} reason={} ms={ms}",
                    draft.page, d.token
                )
            });
            Err(d.note())
        }
    }
}

/// Declines a recognised (invisible) word: it stays a run draft, so the edit is
/// previewed in the word's own size and scale, as the OCR layer renders it
/// (`canvas::ocrink`), which a paragraph rewrite would flatten.
fn recognised(doc: &OpenDoc, page: usize, run: usize) -> Result<(), Declined> {
    let ocr = doc.provenance_page_text(page).is_some_and(|text| {
        text.runs
            .get(run)
            .is_some_and(crate::canvas::ocrlayer::is_ocr_run)
    });
    if ocr {
        Err(Declined::no_paragraph("recognised-word"))
    } else {
        Ok(())
    }
}

/// `draft` re-anchored on `p`, its caret and selection shifted past `before`.
fn promoted(draft: &Draft, p: &Paragraph) -> Draft {
    let shift = p.before.chars().count();
    let text = format!("{}{}{}", p.before, draft.text, p.after);
    let (llx, lly, urx, ury) = p.bbox;
    Draft {
        anchor: Anchor::Block {
            block: p.block,
            run: p.run,
            llx,
            lly,
            urx,
            ury,
            original: format!("{}{}{}", p.before, original_of(draft), p.after),
        },
        text,
        caret: draft.caret + shift,
        mark: draft.mark.map(|m| m + shift),
        ..draft.clone()
    }
}

fn original_of(draft: &Draft) -> &str {
    match &draft.anchor {
        Anchor::Run { original, .. } => original,
        _ => "",
    }
}

/// The engine's own answer to the rewrite Enter would make: the promoted text
/// with the line break inserted, previewed.
fn preflight(doc: &OpenDoc, page: usize, block: usize, promoted: &Draft) -> Result<(), Declined> {
    let mut probe = promoted.clone();
    let at = super::keys::take_selection(&mut probe);
    super::caret::newline(&mut probe.text, at);
    rewrite(doc, page, block, &probe.text)
}

/// Whether the engine would rewrite paragraph `block` as `text`.
fn rewrite(doc: &OpenDoc, page: usize, block: usize, text: &str) -> Result<(), Declined> {
    doc.session
        .edit_block_text_preview(page, block, text, &options(doc))
        .map(|_| ())
        .map_err(|e| Declined {
            why: EnterRefusal::Unrewritable {
                detail: e.to_string(),
            },
            token: "engine-refuses",
        })
}

/// The paragraph `run` sits in, numbered as `edit_block_text` reads it, and
/// the text either side of the run, its lines joined as the engine's
/// `line_ends` reads them: a break as `\n`, a wrap as a space.
fn paragraph(
    doc: &OpenDoc,
    page: usize,
    run: usize,
    original: &str,
) -> Result<Paragraph, Declined> {
    let text = doc
        .provenance_page_text(page)
        .ok_or(Declined::no_paragraph("no-text"))?;
    let view = doc.session.view();
    let cells = detect_cell_regions(&view, page).map_err(|_| Declined::no_paragraph("no-cells"))?;
    let model =
        EditableTextModel::recognize_with_cells(&text, &reflow_recognition_options(), &cells)
            .with_line_marks(marks(&view, page));
    let block = model
        .block_at(TextPosition::new(run, 0))
        .ok_or(Declined::no_paragraph("no-paragraph"))?;
    let b = model
        .blocks()
        .get(block)
        .ok_or(Declined::no_paragraph("no-paragraph"))?;
    let walk = Walk::of(&model, b, run)?;
    let (start, end) = walk
        .span
        .ok_or(Declined::no_paragraph("run-not-in-paragraph"))?;
    if walk.joined.get(start..end) != Some(original) {
        return Err(Declined::no_paragraph("run-text-differs"));
    }
    if walk.looks > 1 {
        return Err(Declined {
            why: EnterRefusal::MixedLooks,
            token: "mixed-looks",
        });
    }
    Ok(Paragraph {
        block,
        run,
        bbox: (b.bbox.llx, b.bbox.lly, b.bbox.urx, b.bbox.ury),
        lines: b.line_indices.len(),
        breaks: walk.breaks,
        source: model.line_end_source(b),
        before: walk.joined[..start].to_owned(),
        after: walk.joined[end..].to_owned(),
    })
}

/// One pass over a paragraph's glyphs: its text, where `run` sits in it, and
/// how many looks it mixes.
struct Walk {
    joined: String,
    span: Option<(usize, usize)>,
    looks: usize,
    /// Line ends read as typed breaks.
    breaks: usize,
}

/// The page's line-end marks, so a paragraph pdfcer wrote reads its breaks
/// exactly; empty when the page's content cannot be read.
fn marks(view: &pdfcer_core::view::DocumentView<'_>, page: usize) -> LineMarks {
    pdfcer_core::page_tree::pages_in(view)
        .ok()
        .and_then(|pages| ContentStream::from_page(view, pages.get(page)?).ok())
        .map(|content| LineMarks::scan(&content))
        .unwrap_or_default()
}

/// A glyph's text, or `""` when the reference is stale.
fn glyph_text<'m>(model: &'m EditableTextModel<'_>, g: GlyphRef) -> &'m str {
    model
        .glyph(g)
        .and_then(|x| {
            let s = x.text_start as usize;
            model
                .sourced_view()
                .runs
                .get(g.run)?
                .text
                .get(s..s + x.text_len as usize)
        })
        .unwrap_or("")
}

/// A glyph's look as the shell can read it from provenance: the font
/// resource, the size and the fill. A subset of the engine's own
/// `SpanStyle::same_look`, which is not public.
type Look = (
    Option<Vec<u8>>,
    u32,
    Option<pdfcer_core::text_extract::TextColor>,
);

impl Walk {
    fn of(
        model: &EditableTextModel<'_>,
        b: &pdfcer_core::text_edit::Block,
        run: usize,
    ) -> Result<Self, Declined> {
        let lines: Vec<&Line> = b
            .line_indices
            .iter()
            .filter_map(|&li| model.lines().get(li))
            .collect();
        let ends = model.line_ends(b);
        let mut joined = String::new();
        let mut span: Option<(usize, usize)> = None;
        let mut closed = false;
        let mut looks: Vec<Look> = Vec::new();
        let mut breaks = 0;
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                let typed = ends.get(i - 1) == Some(&LineEnd::Break);
                breaks += usize::from(typed);
                joined.push(if typed { '\n' } else { ' ' });
                closed |= span.is_some();
            }
            for &g in &line.glyphs {
                let slice = glyph_text(model, g);
                if g.run == run {
                    if closed {
                        return Err(Declined::no_paragraph("run-split"));
                    }
                    let start = span.map_or(joined.len(), |s| s.0);
                    joined.push_str(slice);
                    span = Some((start, joined.len()));
                } else {
                    closed |= span.is_some();
                    joined.push_str(slice);
                }
                if let Some(p) = model.provenance(g) {
                    let look = (p.font_resource.clone(), p.tf_size.to_bits(), p.fill_color);
                    if !looks.contains(&look) {
                        looks.push(look);
                    }
                }
            }
        }
        Ok(Self {
            joined,
            span,
            looks: looks.len(),
            breaks,
        })
    }
}
