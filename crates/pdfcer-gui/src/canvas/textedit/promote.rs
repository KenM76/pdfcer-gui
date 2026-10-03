//! # `canvas::textedit::promote` — Enter in a line of existing text opens its
//! paragraph
//!
//! A run draft edits one show operator, which cannot hold a line break; its
//! paragraph can, through `EditSession::edit_block_text`. [`open`] re-anchors a
//! run draft on the run's paragraph: the paragraph's text with the run's slice
//! replaced by the draft, the caret, selection and draft history shifted to
//! match. It declines, and leaves the draft as it was, when the engine would
//! refuse the rewrite or the paragraph mixes looks the rewrite would flatten.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/promote.md`.

use pdfcer_core::text_edit::{
    BlockEditOptions, EditableTextModel, TextPosition, detect_cell_regions,
    reflow_recognition_options,
};
use pdfcer_gui_base::text::textedit::EnterRefusal;

use super::{Anchor, Draft};
use crate::app::state::OpenDoc;

/// The options a paragraph draft is previewed and committed under: the
/// caret's typing disposition and the installed faces, and no fallback face.
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
    doc.session
        .edit_block_text_preview(page, block, &probe.text, &options(doc))
        .map(|_| ())
        .map_err(|e| Declined {
            why: EnterRefusal::Unrewritable {
                detail: e.to_string(),
            },
            token: "engine-refuses",
        })
}

/// The paragraph `run` sits in, numbered as `edit_block_text` reads it, and
/// the text either side of the run as `BlockHit::text` spells it (lines
/// joined by single spaces).
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
        let page = model.sourced_view();
        let mut joined = String::new();
        let mut span: Option<(usize, usize)> = None;
        let mut closed = false;
        let mut looks: Vec<Look> = Vec::new();
        for (i, &li) in b.line_indices.iter().enumerate() {
            let Some(line) = model.lines().get(li) else {
                continue;
            };
            if i > 0 {
                joined.push(' ');
                closed |= span.is_some();
            }
            for &g in &line.glyphs {
                let slice = model
                    .glyph(g)
                    .and_then(|x| {
                        let s = x.text_start as usize;
                        page.runs.get(g.run)?.text.get(s..s + x.text_len as usize)
                    })
                    .unwrap_or("");
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
        })
    }
}
