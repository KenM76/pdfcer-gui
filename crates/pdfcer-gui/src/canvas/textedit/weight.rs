//! # `canvas::textedit::weight` — is this text bold, is it italic, and how
//!
//! The read behind the pressed state of the ribbon's Bold and Italic toggles
//! and behind what a press does (`OPERATOR_REQUESTS.md` O273): a real face is
//! taken off by choosing another face, a synthetic weight by clearing the
//! render mode that makes it.
//!
//! Read from the glyph at a text position, so a range inside a run answers for
//! its own first letter rather than for the run's.
//!
//! The stroke width is passed to `synth::detect` as `0`: `GlyphProvenance`
//! carries no line width, so every fill-and-stroke render mode on a face whose
//! name does not claim bold reads as synthetic bold. That is what pdfcer itself
//! writes, and an outlined display face written by another producer is the one
//! case it misreads.

use pdfcer_core::text_edit::synth::{detect, name_claims_bold, name_claims_italic};
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
    let tm = p.text_matrix.map(f64::from);
    // `AmbientValue::value` is the operand as written; a render mode is an
    // integer 0..=7.
    #[allow(clippy::cast_possible_truncation)]
    let mode = p.text_state.render_mode.value as i64;
    let synth = detect(&base, mode, 0.0, f64::from(p.tf_size), tm);
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
