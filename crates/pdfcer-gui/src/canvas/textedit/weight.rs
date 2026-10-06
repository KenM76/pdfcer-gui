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

use pdfcer_core::text_edit::synth::{detect_at, name_claims_bold, name_claims_italic};
use pdfcer_core::text_edit::{BlockRecognitionOptions, EditableTextModel, GlyphRef, TextPosition};

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

/// Bold and italic for one glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Weight {
    /// The weight axis.
    pub bold: Axis,
    /// The slant axis.
    pub italic: Axis,
}

/// The weight of the first glyph at or after `at` in its run, or `None` when
/// the run has no glyph with provenance there.
#[must_use]
pub fn at(doc: &OpenDoc, page: usize, at: TextPosition) -> Option<Weight> {
    let text = doc.provenance_page_text(page)?;
    let run = text.runs.get(at.run)?;
    let index = run
        .glyphs
        .iter()
        .position(|g| g.text_start as usize >= at.byte_offset)
        .or_else(|| run.glyphs.len().checked_sub(1))?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let p = model.provenance(GlyphRef::new(at.run, index))?;
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
    })
}
