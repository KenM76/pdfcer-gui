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

use pdfcer_core::text_edit::{
    BlockEditOptions, EditableTextModel, GlyphRef, Line, TextPosition, detect_cell_regions,
    reflow_recognition_options,
};
use pdfcer_gui_base::text::textedit::EnterRefusal;

use super::{Anchor, Draft};
use crate::app::state::OpenDoc;

/// The options a paragraph draft is previewed and committed under: the
/// caret's typing disposition and the installed faces, no fallback face, and
/// `wrap` when the paragraph names one.
#[must_use]
pub fn options(doc: &OpenDoc, wrap: Option<f64>) -> BlockEditOptions {
    BlockEditOptions::new()
        .with_edit_options(super::installed::augmented(
            doc,
            pdfcer_gui_base::editmodel::disposition::typing(),
        ))
        .with_wrap_width_opt(wrap)
}

/// Points added to a paragraph's width to wrap it at. The engine's default
/// wrap is the box width, against which the box's own widest line is
/// measured with no tolerance, so an unchanged line can fail to fit and
/// split.
const WRAP_SLACK: f64 = 0.5;

/// The width to rewrite a paragraph of box width `width` at: `None` in a
/// table cell, whose inner width the engine wraps at when given none.
fn wrap_for(width: f64, in_cell: bool) -> Option<f64> {
    (!in_cell).then_some(width + WRAP_SLACK)
}

/// A run's paragraph: its engine index, its box (PDF user space,
/// `llx, lly, urx, ury`) and its text either side of the run.
struct Paragraph {
    block: usize,
    run: usize,
    bbox: (f64, f64, f64, f64),
    /// The wrap width it is rewritten at; see [`wrap_for`].
    wrap: Option<f64>,
    lines: usize,
    breaks: usize,
    before: String,
    after: String,
}

/// Why a run did not open into its paragraph, with the trace token.
struct Declined {
    why: EnterRefusal,
    token: &'static str,
}

impl Declined {
    const fn no_paragraph(token: &'static str) -> Self {
        Self {
            why: EnterRefusal::NoParagraph,
            token,
        }
    }
}

/// **Re-anchor `draft` (an `Anchor::Run`) on its paragraph**, ready for the
/// line break Enter is about to insert. `Err` leaves `draft` untouched.
///
/// # Errors
///
/// The refusal Enter reports.
pub fn open(ctx: &egui::Context, doc: &OpenDoc, draft: &mut Draft) -> Result<(), EnterRefusal> {
    let Anchor::Run { run, original } = &draft.anchor else {
        return Err(EnterRefusal::NoParagraph);
    };
    let run = *run;
    let result = paragraph(doc, draft.page, run, original).and_then(|p| {
        let promoted = promoted(draft, &p);
        preflight(doc, draft.page, p.block, p.wrap, &promoted)?;
        Ok((p, promoted))
    });
    match result {
        Ok((p, promoted)) => {
            super::history::embed(ctx, &p.before, &p.after);
            *draft = promoted;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-promoted page={} run={run} block={} len={} caret={}",
                    draft.page,
                    p.block,
                    draft.text.chars().count(),
                    draft.caret
                )
            });
            Ok(())
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
/// two or more lines**, its text unchanged. Leaves `draft` as it was when the
/// run is a paragraph of its own or the engine would not rewrite the
/// paragraph as it stands.
pub fn widen(doc: &OpenDoc, draft: &mut Draft) {
    let Anchor::Run { run, original } = &draft.anchor else {
        return;
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
            rewrite(doc, draft.page, p.block, p.wrap, &promoted.text)?;
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
                     ms={ms}",
                    draft.page,
                    p.block,
                    p.lines,
                    p.breaks,
                    draft.text.chars().count()
                )
            });
        }
        Err(d) => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-edit-widen-declined page={} run={run} reason={} ms={ms}",
                draft.page, d.token
            )
        }),
    }
}

/// Declines a recognised (invisible) word: it stays a run draft, so the edit is
/// drawn by the same preview that draws the OCR layer (`canvas::ocrink`), in
/// the word's own size and scale, which a paragraph rewrite would flatten.
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
            wrap: p.wrap,
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
fn preflight(
    doc: &OpenDoc,
    page: usize,
    block: usize,
    wrap: Option<f64>,
    promoted: &Draft,
) -> Result<(), Declined> {
    let mut probe = promoted.clone();
    let at = super::keys::take_selection(&mut probe);
    super::caret::newline(&mut probe.text, at);
    rewrite(doc, page, block, wrap, &probe.text)
}

/// Whether the engine would rewrite paragraph `block` as `text`.
fn rewrite(
    doc: &OpenDoc,
    page: usize,
    block: usize,
    wrap: Option<f64>,
    text: &str,
) -> Result<(), Declined> {
    doc.session
        .edit_block_text_preview(page, block, text, &options(doc, wrap))
        .map(|_| ())
        .map_err(|e| Declined {
            why: EnterRefusal::Unrewritable {
                detail: e.to_string(),
            },
            token: "engine-refuses",
        })
}

/// The paragraph `run` sits in, numbered as `edit_block_text` reads it, and
/// the text either side of the run, its lines joined as [`joint`] decides.
fn paragraph(
    doc: &OpenDoc,
    page: usize,
    run: usize,
    original: &str,
) -> Result<Paragraph, Declined> {
    let text = doc
        .provenance_page_text(page)
        .ok_or(Declined::no_paragraph("no-text"))?;
    let cells = detect_cell_regions(&doc.session.view(), page)
        .map_err(|_| Declined::no_paragraph("no-cells"))?;
    let model =
        EditableTextModel::recognize_with_cells(&text, &reflow_recognition_options(), &cells);
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
        wrap: walk.wrap,
        lines: b.line_indices.len(),
        breaks: walk.breaks,
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
    /// Line breaks kept by [`joint`].
    breaks: usize,
    /// The wrap width [`joint`] measured against.
    wrap: Option<f64>,
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

/// **What joins line `a` to the next line `b`.** A line break when `b`'s
/// first word would have fitted on `a` within `width`, the wrap the rewrite
/// is given: `edit_block_text` packs greedily at it, so a space there would pull the
/// word up and merge two lines broken by hand. Otherwise a space, the wrap
/// the engine makes again.
///
/// `space` is the widest space glyph in the paragraph, `0` when it has none,
/// which errs towards keeping a break. Horizontal text: widths are advances.
fn joint(model: &EditableTextModel<'_>, a: &Line, b: &Line, width: f64, space: f64) -> char {
    let word: f64 = b
        .glyphs
        .iter()
        .map_while(|&g| {
            let x = model.glyph(g)?;
            (!glyph_text(model, g).trim().is_empty()).then_some(f64::from(x.advance))
        })
        .sum();
    let used = a.bbox.urx - a.bbox.llx;
    if used + space + word <= width {
        '\n'
    } else {
        ' '
    }
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
        let space = lines
            .iter()
            .flat_map(|l| l.glyphs.iter())
            .filter(|&&g| glyph_text(model, g) == " ") // ui-text-exempt: a space glyph, never displayed
            .filter_map(|&g| model.glyph(g).map(|x| f64::from(x.advance)))
            .fold(0.0, f64::max);
        let wrap = wrap_for(b.bbox.urx - b.bbox.llx, b.cell_rect.is_some());
        // A cell wraps at its inner width, which is at least the text's.
        let width = wrap.unwrap_or_else(|| b.cell_rect.map_or(0.0, |c| c.urx - c.llx));
        let mut joined = String::new();
        let mut span: Option<(usize, usize)> = None;
        let mut closed = false;
        let mut looks: Vec<Look> = Vec::new();
        let mut breaks = 0;
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                let joint = joint(model, lines[i - 1], line, width, space);
                breaks += usize::from(joint == '\n');
                joined.push(joint);
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
            wrap,
        })
    }
}
