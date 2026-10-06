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

use pdfcer_core::text_edit::decoration::{DecorationSet, page_decorations};
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
    let Some(page_ref) = doc.pages.get(page) else {
        return DecorationSet::NONE;
    };
    page_decorations(&doc.session.view(), page_ref).map_or(DecorationSet::NONE, |d| d.of(p))
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

/// Bold, italic and lines for one glyph's provenance.
fn weight_of(doc: &OpenDoc, page: usize, p: &GlyphProvenance) -> Option<Weight> {
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
        lines: lines_of(doc, page, p),
    })
}
