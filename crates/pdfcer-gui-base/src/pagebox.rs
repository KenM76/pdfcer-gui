//! # `pagebox` — whether a page's crop box hides part of its sheet
//!
//! `Page::crop_box` is the effective box — `/CropBox` ∩ `/MediaBox`, as
//! §14.11.2.1 defines the visible area — so the question is one comparison.

use pdfcer_core::page_tree::Page;

/// Whether the page's crop box hides part of its sheet: the visible area is
/// smaller than the paper, so growing the paper shows nothing new.
#[must_use]
pub fn crop_hides_sheet(page: &Page) -> bool {
    let area = |r: pdfcer_core::page_tree::Rect| r.width() * r.height();
    area(page.crop_box) < area(page.media_box) * (1.0 - 1e-9)
}
