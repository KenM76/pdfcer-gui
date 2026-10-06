//! # `dialogs::recognised` — the recognised-text choice the Text and Word
//! export windows share
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/recognised.md`.

use egui::Ui;

use crate::app::actions::exporttext::{OcrLayerFilter, RECOGNISED_CHOICES};
use crate::app::state::OpenDoc;
use crate::text::export_text as t;

/// How many pages of `doc` carry an OCR layer pdfcer wrote — the same
/// layers File ▸ Recognise ▸ Remove OCR text finds. Zero when the page tree
/// cannot be read, so the choice is not offered.
#[must_use]
pub fn layer_pages(doc: &OpenDoc) -> usize {
    let Ok(layers) = doc.session.find_ocr_layers() else {
        return 0;
    };
    let mut pages: Vec<usize> = layers.iter().map(|l| l.page_index).collect();
    pages.sort_unstable();
    pages.dedup();
    pages.len()
}

/// Draw the choice. Nothing is drawn when `pages` is zero: a document with
/// no OCR layer has nothing to include or leave out (R9).
pub fn section(
    ui: &mut Ui,
    filter: &mut OcrLayerFilter,
    pages: usize,
    region: fn(OcrLayerFilter) -> &'static str,
) {
    if pages == 0 {
        return;
    }
    ui.label(t::recognised_heading());
    for choice in RECOGNISED_CHOICES {
        let response = ui.radio_value(filter, choice, t::recognised_name(choice));
        crate::diag::ui_rect(region(choice), response.rect);
    }
    ui.weak(t::recognised_hint(pages));
}
