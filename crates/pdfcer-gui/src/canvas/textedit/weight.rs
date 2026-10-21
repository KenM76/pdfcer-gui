//! # `canvas::textedit::weight` — is this text bold, is it italic, and how
//!
//! The read behind the pressed state of the ribbon's Bold and Italic toggles
//! and behind what a press does (`OPERATOR_REQUESTS.md` O273): an axis read
//! present, as the face or as synthesis, is asked off.
//!
//! Read from the glyph at a text position, so a range inside a run answers for
//! its own first letter rather than for the run's.
//!
//! Synthesis is the engine's reading of the glyph (`synth::detect_at`): a
//! fill-and-stroke render mode whose line width is within the band a
//! synthesized bold uses, or a sheared text matrix.
//!
//! Underline and strikethrough are the engine's decoration markers read back
//! from the content (`decoration::page_decorations`), so a line drawn by
//! Format ▸ Font presses its toggle like a bold face presses Bold.

use pdfcer_core::text_edit::decoration::{DecorationSet, PageDecorations, page_decorations};
use pdfcer_core::text_edit::synth::{detect_at, name_claims_bold, name_claims_italic};
use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel, GlyphRef, TextPosition};
use pdfcer_core::text_extract::GlyphProvenance;

use crate::app::state::OpenDoc;

/// How one style axis is present on a piece of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Not present.
    Absent,
    /// The face itself is bold (or italic).
    Face,
    /// Drawn by pdfcer-style synthesis: a stroke, or a sheared matrix.
    Synthetic,
}

impl Axis {
    /// Whether the axis is present at all.
    #[must_use]
    pub const fn present(self) -> bool {
        !matches!(self, Self::Absent)
    }
}

/// Bold, italic and the decoration lines for one glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Weight {
    /// The weight axis.
    pub bold: Axis,
    /// The slant axis.
    pub italic: Axis,
    /// The underline and strikethrough the engine's marker gives the glyph.
    pub lines: DecorationSet,
}

/// The provenance of the first glyph at or after `at` in its run, handed to
/// `read`; `None` when the run has no glyph with provenance there.
fn with_glyph<T>(
    doc: &OpenDoc,
    page: usize,
    at: TextPosition,
    read: impl FnOnce(&GlyphProvenance) -> Option<T>,
) -> Option<T> {
    let text = doc.provenance_page_text(page)?;
    let run = text.runs.get(at.run)?;
    let index = run
        .glyphs
        .iter()
        .position(|g| g.text_start as usize >= at.byte_offset)
        .or_else(|| run.glyphs.len().checked_sub(1))?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    read(model.provenance(GlyphRef::new(at.run, index))?)
}

/// The decoration lines on the glyph `p` describes; empty when the page's
/// content does not decode.
fn lines_of(doc: &OpenDoc, page: usize, p: &GlyphProvenance) -> DecorationSet {
    decorations(doc, page).map_or(DecorationSet::NONE, |d| d.of(p))
}

/// The decoration markers on `page`; `None` when its content does not decode.
fn decorations(doc: &OpenDoc, page: usize) -> Option<PageDecorations> {
    page_decorations(&doc.session.view(), doc.pages.get(page)?).ok()
}

/// The underline and strikethrough on the first glyph at or after `at`.
#[must_use]
pub fn lines_at(doc: &OpenDoc, page: usize, at: TextPosition) -> Option<DecorationSet> {
    with_glyph(doc, page, at, |p| Some(lines_of(doc, page, p)))
}

/// The weight of the first glyph at or after `at` in its run, or `None` when
/// the run has no glyph with provenance there.
#[must_use]
pub fn at(doc: &OpenDoc, page: usize, at: TextPosition) -> Option<Weight> {
    with_glyph(doc, page, at, |p| weight_of(doc, page, p))
}

/// What every glyph from `from` up to `to` (exclusive) has in common: an
/// axis is present only when every glyph carries it, and a line only when
/// every glyph has it. `None` when no glyph in the range has provenance.
#[must_use]
pub fn across(doc: &OpenDoc, page: usize, from: TextPosition, to: TextPosition) -> Option<Weight> {
    let text = doc.provenance_page_text(page)?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let lines = decorations(doc, page);
    let mut common: Option<Weight> = None;
    for run in from.run..=to.run {
        let Some(glyphs) = text.runs.get(run).map(|r| &r.glyphs) else {
            continue;
        };
        for (index, glyph) in glyphs.iter().enumerate() {
            let start = glyph.text_start as usize;
            if (run == from.run && start < from.byte_offset)
                || (run == to.run && start >= to.byte_offset)
            {
                continue;
            }
            let Some(one) = model.provenance(GlyphRef::new(run, index)).and_then(|p| {
                face_weight(
                    doc,
                    p,
                    lines.as_ref().map_or(DecorationSet::NONE, |d| d.of(p)),
                )
            }) else {
                continue;
            };
            common = Some(common.map_or(one, |c| shared(c, one)));
        }
    }
    common
}

/// What two weights have in common.
#[must_use]
pub fn shared(a: Weight, b: Weight) -> Weight {
    let axis = |x: Axis, y: Axis| match (x, y) {
        (Axis::Absent, _) | (_, Axis::Absent) => Axis::Absent,
        (Axis::Face, Axis::Face) => Axis::Face,
        _ => Axis::Synthetic,
    };
    Weight {
        bold: axis(a.bold, b.bold),
        italic: axis(a.italic, b.italic),
        lines: DecorationSet::NONE
            .with_underline(a.lines.underline && b.lines.underline)
            .with_strikethrough(a.lines.strikethrough && b.lines.strikethrough),
    }
}

/// Bold, italic and lines for one glyph's provenance.
fn weight_of(doc: &OpenDoc, page: usize, p: &GlyphProvenance) -> Option<Weight> {
    face_weight(doc, p, lines_of(doc, page, p))
}

/// Bold and italic for one glyph's provenance, with its `lines` already read.
fn face_weight(doc: &OpenDoc, p: &GlyphProvenance, lines: DecorationSet) -> Option<Weight> {
    let key = p.font_resource.as_ref()?;
    let key = String::from_utf8_lossy(key);
    let base = doc
        .font_inventory()
        .fonts
        .iter()
        .find(|record| record.resource_names.iter().any(|name| *name == key))
        .and_then(|record| record.base_font.clone())
        .unwrap_or_default();
    let synth = detect_at(&base, p);
    let axis = |face: bool, synthetic: bool| {
        if face {
            Axis::Face
        } else if synthetic {
            Axis::Synthetic
        } else {
            Axis::Absent
        }
    };
    Some(Weight {
        bold: axis(name_claims_bold(&base), synth.bold()),
        italic: axis(name_claims_italic(&base), synth.italic()),
        lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::EditSession;

    /// One line: "BOLD" in Helvetica-Bold, then "PLAIN" in Helvetica.
    fn mixed() -> OpenDoc {
        let content = "BT /F2 12 Tf 10 20 Td (BOLD) Tj ET BT /F1 12 Tf 60 20 Td (PLAIN) Tj ET";
        let bodies = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 40] /Contents 4 0 R \
             /Resources << /Font << /F1 5 0 R /F2 6 0 R >> >> >>"
                .to_owned(),
            format!(
                "<< /Length {} >>\nstream\n{content}\nendstream",
                content.len()
            ),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".to_owned(),
        ];
        let mut buf = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in bodies.iter().enumerate() {
            offsets.push(buf.len());
            buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref = buf.len();
        buf.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", bodies.len() + 1).as_bytes(),
        );
        for o in offsets {
            buf.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        buf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                bodies.len() + 1
            )
            .as_bytes(),
        );
        let doc = Document::from_bytes(buf).expect("parses");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        OpenDoc::new("mixed.pdf".into(), EditSession::new(doc), pages)
    }

    fn run_of(doc: &OpenDoc, text: &str) -> usize {
        let page = doc.provenance_page_text(0).expect("text");
        page.runs
            .iter()
            .position(|r| r.text.contains(text))
            .unwrap_or_else(|| panic!("no run reads {text}"))
    }

    /// A selection reads bold only when every letter in it is, so a sweep
    /// across a bold word and a plain one shows Bold unpressed, as a word
    /// processor does, while either word alone reads as itself.
    #[test]
    fn a_mixed_selection_is_not_bold() {
        let doc = mixed();
        let (bold, plain) = (run_of(&doc, "BOLD"), run_of(&doc, "PLAIN"));
        let whole = |run| {
            (
                TextPosition::new(run, 0),
                TextPosition::new(run, usize::MAX),
            )
        };
        let (a, b) = whole(bold);
        assert_eq!(across(&doc, 0, a, b).map(|w| w.bold), Some(Axis::Face));
        let (a, b) = whole(plain);
        assert_eq!(across(&doc, 0, a, b).map(|w| w.bold), Some(Axis::Absent));
        let (from, to) = if bold < plain {
            (whole(bold).0, whole(plain).1)
        } else {
            (whole(plain).0, whole(bold).1)
        };
        assert_eq!(
            across(&doc, 0, from, to).map(|w| w.bold),
            Some(Axis::Absent)
        );
        assert_eq!(at(&doc, 0, whole(bold).0).map(|w| w.bold), Some(Axis::Face));
    }
}
