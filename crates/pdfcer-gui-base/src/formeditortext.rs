//! # `formeditortext` — how a form field's in-place editor sets its text
//!
//! Point size from the editor's on-screen height, and alignment from the field's `/Q`.

use egui::Align;
use pdfcer_core::vartext::Quadding;

/// The proportion of an editor's height the text is set at.
pub const EDITOR_TEXT_RATIO: f32 = 0.62;

/// The smallest and largest point size the editor will set text at.
pub const EDITOR_TEXT_RANGE: (f32, f32) = (9.0, 22.0);

/// The point size the editor sets text at, for a box `height` points tall on
/// screen.
#[must_use]
pub fn editor_font_size(height: f32) -> f32 {
    (height * EDITOR_TEXT_RATIO).clamp(EDITOR_TEXT_RANGE.0, EDITOR_TEXT_RANGE.1)
}

/// Which end of the editor the operator's text is set against, for a field's
/// `/Q`.
#[must_use]
pub fn editor_align(quadding: Quadding) -> Align {
    match quadding {
        Quadding::Left => Align::LEFT,
        Quadding::Center => Align::Center,
        Quadding::Right => Align::RIGHT,
    }
}
