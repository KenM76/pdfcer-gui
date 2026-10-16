//! # `canvas::textedit::promote` — Enter in a line of existing text opens its
//! paragraph
//!
//! A run draft edits one show operator, which cannot hold a line break; its
//! paragraph can, through `EditSession::edit_block_text`. [`open`] re-anchors a
//! run draft on the run's paragraph: the paragraph's text with the run's slice
//! replaced by the draft, the caret, selection and draft history shifted to
//! match. It declines, and leaves the draft as it was, when the engine would
//! refuse the rewrite. A paragraph of several looks opens like any other: the
//! engine's rewrite keeps each character's look and discloses a fallback.
//! [`widen`] does the same for a click on any line of a paragraph of two or
//! more lines, so the paragraph re-opens as one draft.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/promote.md`.

use pdfcer_core::content::ContentStream;
use pdfcer_core::text_edit::{
    BlockEditOptions, EditableTextModel, GlyphRef, Line, LineEnd, LineEndSource, LineMarks,
    TextPosition, detect_cell_regions, reflow_recognition_options,
};
use pdfcer_core::text_extract::TextColor;
use pdfcer_gui_base::text::draftnote::DraftNote;
use pdfcer_gui_base::text::textedit::EnterRefusal;

use super::{Anchor, Draft};
use crate::app::state::OpenDoc;

pub mod here;
pub mod objects;

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

/// The live copy of `Prefs::open_paragraphs`, which gates [`widen`] on a
/// click. Memory key; application-scoped, like the chunk boxes.
const ENABLED_KEY: &str = "pdfcer.open-paragraphs.enabled"; // ui-text-exempt: a memory key, never displayed

/// Does a click in a line of a paragraph open the paragraph? On by default.
#[must_use]
pub fn enabled(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new(ENABLED_KEY)))
        .unwrap_or(true)
}

/// Pages whose answer differs from the switch, keyed by (document serial,
/// page index). Memory key; held for the life of the window, never saved.
const PAGES_KEY: &str = "pdfcer.open-paragraphs.pages"; // ui-text-exempt: a memory key, never displayed

type PageAnswers = std::collections::HashMap<(u64, usize), bool>;

/// Does a click on `page` of document `serial` open its paragraph? The
/// page's own answer when it has one, else the switch's.
#[must_use]
pub fn enabled_on(ctx: &egui::Context, serial: u64, page: usize) -> bool {
    ctx.data(|d| d.get_temp::<PageAnswers>(egui::Id::new(PAGES_KEY)))
        .and_then(|m| m.get(&(serial, page)).copied())
        .unwrap_or_else(|| enabled(ctx))
}

/// Give `pages` of document `serial` the answer `on`. A page whose answer
/// matches the switch's drops its own, so it follows the switch again.
pub fn set_pages(ctx: &egui::Context, serial: u64, pages: &[usize], on: bool) {
    let global = enabled(ctx);
    ctx.data_mut(|d| {
        let map = d.get_temp_mut_or_default::<PageAnswers>(egui::Id::new(PAGES_KEY));
        for &page in pages {
            if on == global {
                map.remove(&(serial, page));
            } else {
                map.insert((serial, page), on);
            }
        }
    });
    crate::diag::trace(|| {
        let list: Vec<String> = pages.iter().map(usize::to_string).collect();
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "open-paragraphs-pages enabled={on} pages={}",
            list.join(",")
        )
    });
}

/// Turn paragraph opening on or off for the next click.
pub fn set_enabled(ctx: &egui::Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ENABLED_KEY), on));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("open-paragraphs enabled={on}")
    });
}

/// Mirror the persisted answer into the live one, once per frame.
pub fn sync(ctx: &egui::Context, on: bool) {
    if enabled(ctx) != on {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(ENABLED_KEY), on));
    }
}

/// A run's paragraph: its engine index, its box (PDF user space,
/// `llx, lly, urx, ury`) and its text either side of the run.
struct Paragraph {
    block: usize,
    run: usize,
    bbox: (f64, f64, f64, f64),
    lines: usize,
    /// Distinct looks among its glyphs, traced so a driven check can tell a
    /// mixed paragraph from a uniform one.
    looks: usize,
    breaks: usize,
    /// Whether its line ends were read from pdfcer's marks or inferred.
    source: LineEndSource,
    before: String,
    after: String,
    places: Vec<Place>,
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
            "engine-refuses" => Some(DraftNote::LineOnlyRefused),
            "recognised-word" => Some(DraftNote::RecognisedWordOnly),
            _ => None,
        }
    }

    /// The engine's sentence as a trailing ` detail=` trace field, when it
    /// refused; empty otherwise. Last on the line because it holds spaces.
    fn detail(&self) -> String {
        match &self.why {
            EnterRefusal::Unrewritable { detail } => format!(" detail={detail}"), // ui-text-exempt: a trace field, never displayed
            _ => String::new(),
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
    let result = recognised(doc, draft.page, run)
        .map_err(|_| Declined {
            why: EnterRefusal::Recognised,
            token: "recognised-word",
        })
        .and_then(|()| paragraph(doc, draft.page, run, original))
        .and_then(|p| {
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
                    "text-edit-promoted page={} run={run} block={} len={} caret={} source={} \
                     looks={}",
                    draft.page,
                    p.block,
                    draft.text.chars().count(),
                    draft.caret,
                    p.source.as_str(),
                    p.looks
                )
            });
            Ok(p.source)
        }
        Err(d) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-enter-declined page={} run={run} reason={}{}",
                    draft.page,
                    d.token,
                    d.detail()
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
/// opened whole; `Err(None)` when the run is all there is or its paragraph
/// answers no: its own answer ([`here`]) when it has one, else `default`.
pub fn widen(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &mut Draft,
    default: bool,
) -> Result<LineEndSource, Option<DraftNote>> {
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
            if !here::opens(ctx, doc.serial, draft.page, p.bbox, default) {
                return Err(Declined::no_paragraph("switched-off"));
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
                     source={} ms={ms} looks={}",
                    draft.page,
                    p.block,
                    p.lines,
                    p.breaks,
                    draft.text.chars().count(),
                    p.source.as_str(),
                    p.looks
                )
            });
            Ok(p.source)
        }
        Err(d) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-widen-declined page={} run={run} reason={} ms={ms}{}",
                    draft.page,
                    d.token,
                    d.detail()
                )
            });
            Err(d.note())
        }
    }
}

/// Declines a recognised (invisible) run — a line, or a word on a page an older
/// pdfcer recognised: it stays a run draft, because a paragraph rewrite
/// re-spaces every word from the line's start, off the scan words the layer
/// sits over (G203).
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
            places: p.places.clone(),
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
    Ok(Paragraph {
        block,
        run,
        bbox: (b.bbox.llx, b.bbox.lly, b.bbox.urx, b.bbox.ury),
        lines: b.line_indices.len(),
        looks: walk.looks,
        breaks: walk.breaks,
        source: model.line_end_source(b),
        before: walk.joined[..start].to_owned(),
        after: walk.joined[end..].to_owned(),
        places: walk.places,
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
    places: Vec<Place>,
}

/// One glyph of an opened paragraph: `len` bytes at byte `at` of its text,
/// and where they start on the page.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    at: usize,
    len: usize,
    pos: TextPosition,
}

/// The page position of byte `byte` of an opened paragraph's text, on a glyph
/// boundary: a range's `end` rounds forward past the glyph it falls in or
/// after, a start rounds back to the glyph it falls in or before. `None` past
/// either end of the paragraph.
#[must_use]
pub fn position(places: &[Place], byte: usize, end: bool) -> Option<TextPosition> {
    if end {
        let p = places.iter().rev().find(|p| p.at < byte)?;
        Some(TextPosition::new(p.pos.run, p.pos.byte_offset + p.len))
    } else {
        places.iter().find(|p| p.at + p.len > byte).map(|p| p.pos)
    }
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
/// `SpanStyle::same_look`, which is not public. An unset fill is the default
/// black `DeviceGray 0` it renders as: the engine's rewrite sets a colour only
/// where the inherited one differs, so the same black reads both ways.
type Look = (Option<Vec<u8>>, u32, TextColor);

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
        let mut places = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                let typed = ends.get(i - 1) == Some(&LineEnd::Break);
                breaks += usize::from(typed);
                joined.push(if typed { '\n' } else { ' ' });
                closed |= span.is_some();
            }
            for &g in &line.glyphs {
                let slice = glyph_text(model, g);
                if let Some(x) = model.glyph(g) {
                    places.push(Place {
                        at: joined.len(),
                        len: slice.len(),
                        pos: TextPosition::new(g.run, x.text_start as usize),
                    });
                }
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
                    let fill = p.fill_color.unwrap_or(TextColor::Gray(0.0));
                    let look = (p.font_resource.clone(), p.tf_size.to_bits(), fill);
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
            places,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Place, position};
    use pdfcer_core::text_edit::TextPosition;

    /// "ab cd": `ab` is run 4 from byte 10, the wrap space has no glyph, `cd`
    /// is run 7 from byte 0.
    fn places() -> Vec<Place> {
        let p = |at, run, byte| Place {
            at,
            len: 1,
            pos: TextPosition::new(run, byte),
        };
        vec![p(0, 4, 10), p(1, 4, 11), p(3, 7, 0), p(4, 7, 1)]
    }

    #[test]
    fn a_range_maps_onto_the_runs_either_side_of_a_wrap() {
        let p = places();
        assert_eq!(position(&p, 0, false), Some(TextPosition::new(4, 10)));
        assert_eq!(position(&p, 2, true), Some(TextPosition::new(4, 12)));
        assert_eq!(position(&p, 2, false), Some(TextPosition::new(7, 0)));
        assert_eq!(position(&p, 5, true), Some(TextPosition::new(7, 2)));
    }

    #[test]
    fn nothing_lies_past_either_end() {
        let p = places();
        assert_eq!(position(&p, 5, false), None);
        assert_eq!(position(&p, 0, true), None);
    }
}
