//! # `app::status::ocrlayer` — what the OCR text layer is showing
//!
//! One line, drawn only while `view.ocr_overlay` is on.
//!
//! ## Why it exists at all
//!
//! Two facts, neither of which the canvas can carry.
//!
//! **Where the slider is.** The blend is a continuum, and a half-faded pale
//! scan looks much like a differently-half-faded pale scan. The number is the
//! only place the position is legible.
//!
//! **Whether the page has any recognised text.** A page that was never OCR'd
//! draws nothing under this mode, and *nothing* is what a broken feature also
//! draws. R8b's second guard is explicit that an inference the operator cannot
//! see owes an off-canvas report; *this page carries no recognised text* is
//! exactly such an inference, and the canvas may not be marked to say it.
//!
//! ## It quotes the painter's own arithmetic
//!
//! Through [`crate::canvas::ocrlayer::painted_fraction`], which is the
//! function the alphas are derived from — never the raw field and never
//! [`crate::viewer::normalise_ocr_overlay`]. That one answers *where did the
//! operator leave the slider* and lands a corrupt preference at a useful
//! default; this bar has to answer *what is on screen*, and the two diverge on
//! exactly the input where the painter draws nothing.
//!
//! ## Cost
//!
//! One `Option` read, and — while the mode is on — one walk of the extracted
//! page's runs. With the mode off, nothing is read at all.
//!
//! The extraction behind that walk is `OpenDoc::page_text`'s, keyed on
//! `(page, edit_epoch)`, so it is done once per page per edit and every later
//! caller that frame gets the cached answer. Usually the canvas is the one
//! that pays, because it asks first. **At a slider position of zero it is
//! this bar that pays**, because `canvas::ocrlayer::draw_text` returns on a
//! zero alpha before it reaches the cache — the one position where the
//! operator can see the layer is on and see nothing drawn, which is precisely
//! the position where the count is worth the most.

use crate::app::state::OpenDoc;

/// Put the line on the bar, if there is one to put.
pub(super) fn show(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(raw) = doc.view.ocr_overlay else {
        return;
    };
    let percent = (crate::canvas::ocrlayer::painted_fraction(raw) * 100.0).round() as u32;
    let blocks = doc.page_text().map_or(0, |text| {
        text.runs
            .iter()
            .filter(|run| crate::canvas::ocrlayer::is_ocr_run(run))
            .count()
    });
    ui.label(crate::text::status::ocr_layer_line(percent, blocks));
    ui.separator();
}
