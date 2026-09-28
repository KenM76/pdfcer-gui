//! # `pagebox` — the visible area a conforming reader shows
//!
//! `page_tree::Page::crop_box` is the page's `/CropBox` as written. §14.11.2.1
//! makes the visible area its intersection with `/MediaBox`, and the renderer
//! frames by `crop_box`, so a sheet shrunk under its crop box would draw at
//! its old size. Every page list the shell adopts passes through
//! [`clip_crop_to_media`]. The engine is asked to intersect at the source
//! (request `G059`); when it does, this module goes.

use pdfcer_core::page_tree::{Page, Rect};

/// `crop` ∩ `media`, or `media` when they do not overlap (§14.11.2.1 leaves
/// that case to the reader; the sheet is the only area with content).
#[must_use]
pub fn visible(crop: Rect, media: Rect) -> Rect {
    let llx = crop.llx.max(media.llx);
    let lly = crop.lly.max(media.lly);
    let urx = crop.urx.min(media.urx);
    let ury = crop.ury.min(media.ury);
    if urx > llx && ury > lly {
        Rect::from_corners(llx, lly, urx, ury)
    } else {
        media
    }
}

/// Replace every page's `crop_box` with its visible area. Returns how many
/// changed.
pub fn clip_crop_to_media(pages: &mut [Page]) -> usize {
    let mut changed = 0;
    for page in pages {
        let clipped = visible(page.crop_box, page.media_box);
        if clipped != page.crop_box {
            page.crop_box = clipped;
            changed += 1;
        }
    }
    changed
}

/// Whether the page's own crop box hides part of its sheet: the visible area
/// is smaller than the paper, so growing the paper shows nothing new.
#[must_use]
pub fn crop_hides_sheet(page: &Page) -> bool {
    let shown = visible(page.crop_box, page.media_box);
    let area = |r: Rect| r.width() * r.height();
    area(shown) < area(page.media_box) * (1.0 - 1e-9)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(a: f64, b: f64, c: f64, d: f64) -> Rect {
        Rect::from_corners(a, b, c, d)
    }

    /// A crop box larger than a shrunk sheet shows the sheet; one inside a
    /// grown sheet stays and is reported as hiding it.
    #[test]
    fn the_visible_area_is_the_intersection() {
        assert_eq!(
            visible(r(0.0, 0.0, 612.0, 792.0), r(0.0, 0.0, 300.0, 400.0)),
            r(0.0, 0.0, 300.0, 400.0)
        );
        assert_eq!(
            visible(r(10.0, 10.0, 100.0, 100.0), r(0.0, 0.0, 612.0, 792.0)),
            r(10.0, 10.0, 100.0, 100.0)
        );
        assert_eq!(
            visible(r(700.0, 0.0, 800.0, 10.0), r(0.0, 0.0, 612.0, 792.0)),
            r(0.0, 0.0, 612.0, 792.0)
        );
    }
}
