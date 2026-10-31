//! # `text::panels::modelphone` — the 3D viewer's phone layout
//!
//! Every string `pdfcer_gui::dialogs::model3d::phone` shows.

/// How to move the camera with fingers.
#[must_use]
pub fn view_hint_touch() -> &'static str {
    "Drag to turn the model, pinch to zoom, drag with two fingers to move it."
}
