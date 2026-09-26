//! # `printink` — which parts of a rendered sheet actually carry ink
//!
//! ## The question this module exists to answer, and why it is asked HERE
//!
//!
//! > *"can you make it so the red pattern you put over the page if it is going
//! > to print beyond the printable borders is only over the areas that extend
//! > beyond the printable page? Our drawing get drawn 1:1 and the area that
//! > isn't printed is just empty border."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printink.md`.

use egui::Rect;

/// Cells across the mask's **longer** side.
const CELLS_LONG_SIDE: u32 = 256;

/// The lightest `min(R, G, B)` a pixel may have and still count as **ink**.
///
/// # This constant is the whole change, and it is set from a MEASUREMENT
///
/// The intuitive test is `min(R, G, B) < 255` — "anything that is not pure
/// white is ink". It is wrong, and wrong on exactly the document class O113 is
/// about.
///
/// `fixtures/a1-titleblock.pdf` is a large-format CAD sheet. Rendered through
/// the preview's own path, the histogram of its `min(R, G, B)` is:
///
/// ```text
///   value 249  ->  236,443 pixels     <- the sheet's own near-white paper fill
///   value 255  ->    7,246 pixels
///   248..=252  ->       12 pixels     <- antialiasing between the two
///   <= 246     ->    7,215 pixels     <- the linework, the title block, the text
/// ```
///
/// The paper is `(249, 249, 249)`, not white: the exporter painted a near-white
/// background rectangle over the whole sheet. Under a `< 255` test, **97% of
/// that page is "ink"**, its empty border included, and the hatch covers the
/// entire overhang — which is precisely the defect being fixed, reintroduced by
/// a wrong definition of the word.
///
/// 246 is 9 levels (3.5%) below white. It clears the measured 249 paper with
/// three levels of margin for a differently-rounded exporter, and it is still
/// only about 4% grey — lighter than any mark an operator would describe as
/// content. The two failure directions are not symmetric and the choice is
/// made deliberately toward the safe one:
///
/// - **Too high** (closer to 255) ⇒ near-white paper reads as ink ⇒ the
///   hatch fires on empty borders ⇒ **O113 all over again**, and the operator
///   is trained to ignore the warning.
/// - **Too low** ⇒ a genuinely very faint mark in the overhang is not
///   disclosed. This is bounded by `Placement::clipped` still being true, by
///   the job-wide clip count still being shown, and by the commit button still
///   naming the sheet — see `super::preview::column`. Nothing goes silent.
///
/// The test [`tests::near_white_cad_paper_is_not_ink`] pins the measured 249
/// against this constant, so the two cannot drift apart without a failure that
/// says which one moved.
const INK_MAX_LEVEL: u8 = 246;

/// A downsampled record of **where a rendered page carries ink**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InkMask {
    /// Cells across, at least 1.
    cols: usize,
    /// Cells down, at least 1.
    rows: usize,
    /// Row-major, `cols * rows` entries. `true` = at least one ink pixel.
    cells: Vec<bool>,
}

impl InkMask {
    /// Build a mask from a rendered page's **premultiplied** RGBA8 bytes.
    pub fn from_rgba_premultiplied(width: u32, height: u32, data: &[u8]) -> Self {
        let (w, h) = (width as usize, height as usize);
        let blank = Self {
            cols: 1,
            rows: 1,
            cells: vec![false],
        };
        if w == 0 || h == 0 || data.len() < w.saturating_mul(h).saturating_mul(4) {
            return blank;
        }

        // The grid is scaled so the LONGER side gets `CELLS_LONG_SIDE` cells
        // and the shorter side gets proportionally fewer. Square-ish cells are
        // not needed for correctness — `ink_extent` works in normalised space
        // either way — but they make the hatch's granularity the same in both
        // axes, which is what a reader comparing a hatch to a drawing expects.
        let long = w.max(h);
        let cells_long = (CELLS_LONG_SIDE as usize).min(long);
        let cols = (w * cells_long).div_ceil(long).max(1);
        let rows = (h * cells_long).div_ceil(long).max(1);
        let mut cells = vec![false; cols * rows];

        // A PIXEL IS AN AREA, NOT A POINT, and this is where that stopped
        // being a pedantic distinction.
        //
        // The obvious mapping is `cell = pixel * cells / length`, which assigns
        // each pixel to the cell its top-left corner lands in. It is **wrong at
        // the far edge** whenever `cells` does not divide `length`: pixel `x`
        // occupies the normalised span `[x/len, (x+1)/len]`, its assigned cell
        // `c = floor(x·cells/len)` covers `[c/cells, (c+1)/cells]`, and
        // `(c+1)/cells` can fall *inside* the pixel — so the mask's idea of
        // where that ink ends stops short of where the ink actually is.
        //
        // Caught by [`tests::one_inked_spot_in_the_overhang_hatches_that_spot_and_no_more`]
        // on a 400 px raster with a 256-cell grid: a mark ending at pixel 353
        // (0.8850 of the page) reported an extent ending at 0.8828, under by
        // one pixel. On a print preview that is about half a point on US
        // Letter — invisible, and in the **unsafe** direction. A hatch that
        // stops short of a mark that will in fact be cropped is a disclosure
        // understating a loss, which is the one error this whole surface
        // exists to avoid, and it would have been unfindable by looking.
        //
        // The fix is to light every cell the pixel's **area** overlaps. Because
        // `cells <= length`, one pixel is never wider than one cell, so it
        // touches at most two cells per axis and the ranges below are one or
        // two entries. The spans are precomputed per row and per column rather
        // than recomputed per pixel: `w + h` divisions instead of `2·w·h`.
        let span = |i: usize, count: usize| {
            let lo = (i * cells_long / long).min(count - 1);
            // The LAST cell the pixel's closed span touches. `(i+1)·cells − 1`
            // is the largest numerator strictly inside the pixel's right edge,
            // so a pixel ending exactly on a cell boundary does not claim the
            // cell beyond it.
            let hi = (((i + 1) * cells_long - 1) / long).min(count - 1);
            (lo, hi.max(lo))
        };
        let col_span: Vec<(usize, usize)> = (0..w).map(|x| span(x, cols)).collect();
        let row_span: Vec<(usize, usize)> = (0..h).map(|y| span(y, rows)).collect();

        // One pass over the raster. Scanning by pixel rather than by cell is
        // deliberate: it touches every byte exactly once and its memory access
        // is sequential, where a cell-major loop would stride across rows and
        // re-read cache lines. The two-by-two write happens only for pixels
        // that are ink, which on a drawing is a few percent of them.
        //
        // Iterated as rows of four-byte pixels zipped against the precomputed
        // spans rather than by index, so the bounds check happens once per row
        // instead of once per pixel and there is no offset arithmetic to get
        // wrong. `chunks_exact` also makes the four-byte stride structural: a
        // slice whose length is not a multiple of four cannot silently
        // misalign the channels, it simply leaves a remainder never read.
        for (&(row_lo, row_hi), row_bytes) in row_span.iter().zip(data.chunks_exact(w * 4)) {
            for (&(col_lo, col_hi), px) in col_span.iter().zip(row_bytes.chunks_exact(4)) {
                if !is_ink(px[0], px[1], px[2], px[3]) {
                    continue;
                }
                for row in row_lo..=row_hi {
                    let base = row * cols;
                    for col in col_lo..=col_hi {
                        cells[base + col] = true;
                    }
                }
            }
        }

        Self { cols, rows, cells }
    }

    /// The **ink extent within `region`**, in normalised 0..1 page space, or
    /// `None` when no cell touching `region` carries ink.
    pub fn ink_extent(&self, region: Rect) -> Option<Rect> {
        let unit = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        if !region.min.x.is_finite()
            || !region.min.y.is_finite()
            || !region.max.x.is_finite()
            || !region.max.y.is_finite()
        {
            return None;
        }
        let region = region.intersect(unit);
        if region.width() <= 0.0 || region.height() <= 0.0 {
            return None;
        }

        let (cols, rows) = (self.cols as f32, self.rows as f32);
        // Floor the minimum and ceil the maximum: cover the region rather than
        // crop it. Both ends are then clamped into the grid, so the ranges are
        // valid indices even for a region flush against 1.0.
        let col0 =
            ((region.min.x * cols).floor() as isize).clamp(0, self.cols as isize - 1) as usize;
        let row0 =
            ((region.min.y * rows).floor() as isize).clamp(0, self.rows as isize - 1) as usize;
        let col1 = (((region.max.x * cols).ceil() as isize).clamp(1, self.cols as isize)) as usize;
        let row1 = (((region.max.y * rows).ceil() as isize).clamp(1, self.rows as isize)) as usize;

        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
        let mut any = false;
        for row in row0..row1 {
            let base = row * self.cols;
            for col in col0..col1 {
                if !self.cells[base + col] {
                    continue;
                }
                any = true;
                x0 = x0.min(col);
                y0 = y0.min(row);
                x1 = x1.max(col + 1);
                y1 = y1.max(row + 1);
            }
        }
        if !any {
            return None;
        }

        // Back to normalised page space, then clipped to the caller's region:
        // the extent must describe what is lost, and a cell straddling the
        // printable boundary is only lost on the side that falls outside it.
        let extent = Rect::from_min_max(
            egui::pos2(x0 as f32 / cols, y0 as f32 / rows),
            egui::pos2(x1 as f32 / cols, y1 as f32 / rows),
        )
        .intersect(region);
        (extent.width() > 0.0 && extent.height() > 0.0).then_some(extent)
    }
}

/// Is one premultiplied RGBA pixel **ink**?
const fn is_ink(r: u8, g: u8, b: u8, a: u8) -> bool {
    // A pixel the page group did not fully cover is something painted into,
    // and its premultiplied channels understate its colour. Ink, without
    // examining the channels at all.
    if a != 255 {
        return true;
    }
    let min = if r < g { r } else { g };
    let min = if min < b { min } else { b };
    min <= INK_MAX_LEVEL
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a premultiplied-RGBA raster of `w × h` opaque white, then paint
    /// the given pixel rectangles solid black.
    fn raster(w: u32, h: u32, ink: &[(u32, u32, u32, u32)]) -> Vec<u8> {
        let mut data = vec![255u8; (w * h * 4) as usize];
        for &(x0, y0, x1, y1) in ink {
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = ((y * w + x) * 4) as usize;
                    data[i] = 0;
                    data[i + 1] = 0;
                    data[i + 2] = 0;
                    data[i + 3] = 255;
                }
            }
        }
        data
    }

    /// The right-hand overhang band of a page whose right `fraction` runs off
    /// the printable area, in normalised page space.
    fn right_band(fraction: f32) -> Rect {
        Rect::from_min_max(egui::pos2(1.0 - fraction, 0.0), egui::pos2(1.0, 1.0))
    }

    /// **The operator's own case: a 1:1 drawing whose overhang is empty
    /// paper gets NO hatch.**
    #[test]
    fn a_blank_overhang_is_not_hatched_even_though_the_page_is_full_of_ink() {
        // 400 x 400 raster, ink confined to the left three quarters.
        let data = raster(400, 400, &[(20, 20, 300, 380)]);
        let mask = InkMask::from_rgba_premultiplied(400, 400, &data);

        // The right 20% overhangs. It holds no ink.
        assert_eq!(
            mask.ink_extent(right_band(0.20)),
            None,
            "the overhanging band is empty paper, so nothing is lost and nothing \
             may be hatched — this is operator request O113 in one line"
        );
        // And the mask is not simply empty: it can see the ink that IS there.
        assert!(
            mask.ink_extent(Rect::from_min_max(
                egui::pos2(0.0, 0.0),
                egui::pos2(1.0, 1.0)
            ))
            .is_some(),
            "the whole page must report ink, or this test would pass on a mask \
             that found nothing anywhere"
        );
    }

    /// **One inked cell in the overhang is hatched, and nothing else is.**
    #[test]
    fn one_inked_spot_in_the_overhang_hatches_that_spot_and_no_more() {
        // A 256-cell grid over 400 px is 1.5625 px per cell, so a 4 px mark
        // spans about three cells — comfortably more than one and far less
        // than the band.
        let data = raster(400, 400, &[(20, 20, 300, 380), (350, 40, 354, 44)]);
        let mask = InkMask::from_rgba_premultiplied(400, 400, &data);

        let band = right_band(0.20); // x from 0.80 to 1.00
        let extent = mask
            .ink_extent(band)
            .expect("a mark inside the band must be reported as lost");

        // The mark sits at x 350..354 of 400 => 0.875..0.885 normalised, and
        // y 40..44 of 400 => 0.100..0.110. The extent must contain it.
        assert!(
            extent.min.x <= 0.875 && extent.max.x >= 0.885,
            "the extent {extent:?} does not cover the mark's x span 0.875..0.885"
        );
        assert!(
            extent.min.y <= 0.100 && extent.max.y >= 0.110,
            "the extent {extent:?} does not cover the mark's y span 0.100..0.110"
        );

        // And it must be TIGHT. Snapping out to cell boundaries allows one cell
        // of slack on each side; one cell is 1/256 = 0.0039 in normalised
        // space, so a 0.02 tolerance is generous and still far short of the
        // 0.20-wide, 1.00-tall band the old code hatched whole.
        assert!(
            extent.min.x >= 0.855 && extent.max.x <= 0.905,
            "the extent {extent:?} spread beyond the mark in x — the point of \
             O113 is that the REST of the band is not hatched"
        );
        assert!(
            extent.min.y >= 0.080 && extent.max.y <= 0.130,
            "the extent {extent:?} spread beyond the mark in y; the band is the \
             full height of the sheet and almost none of it is lost"
        );
    }

    /// **Ink that is entirely inside the printable rectangle hatches
    /// nothing**, which is the case where the placement reports no clip at all
    /// and is the sanity check on the other two.
    #[test]
    fn ink_wholly_inside_the_printable_rect_is_not_hatched() {
        let data = raster(400, 400, &[(0, 0, 200, 200)]);
        let mask = InkMask::from_rgba_premultiplied(400, 400, &data);
        // Bands on both axes, well clear of the ink.
        assert_eq!(mask.ink_extent(right_band(0.20)), None);
        assert_eq!(
            mask.ink_extent(Rect::from_min_max(
                egui::pos2(0.0, 0.80),
                egui::pos2(1.0, 1.0)
            )),
            None,
            "the bottom band is blank too"
        );
    }

    /// **Near-white CAD paper is not ink**, and this is the measurement
    /// [`INK_MAX_LEVEL`] exists for.
    #[test]
    fn near_white_cad_paper_is_not_ink() {
        const MEASURED_CAD_PAPER: u8 = 249;
        assert!(
            !is_ink(
                MEASURED_CAD_PAPER,
                MEASURED_CAD_PAPER,
                MEASURED_CAD_PAPER,
                255
            ),
            "a CAD exporter's near-white background fill at {MEASURED_CAD_PAPER} must read \
             as PAPER. Measured on fixtures/a1-titleblock.pdf, where it is 236,443 of \
             250,916 pixels — classifying it as ink hatches 97% of the sheet and puts \
             O113's defect straight back."
        );
        const {
            assert!(
                MEASURED_CAD_PAPER > INK_MAX_LEVEL,
                "INK_MAX_LEVEL has drifted to or past the CAD paper level measured on \
                 fixtures/a1-titleblock.pdf (249). At or above it, that sheet's \
                 empty border reads as ink and the whole overhang is hatched again."
            );
        }

        // A whole raster of that paper carries no ink at all.
        let data = [
            MEASURED_CAD_PAPER,
            MEASURED_CAD_PAPER,
            MEASURED_CAD_PAPER,
            255,
        ]
        .repeat(64);
        let mask = InkMask::from_rgba_premultiplied(8, 8, &data);
        assert_eq!(
            mask.ink_extent(Rect::from_min_max(
                egui::pos2(0.0, 0.0),
                egui::pos2(1.0, 1.0)
            )),
            None,
            "a sheet that is nothing but near-white paper has no ink anywhere"
        );
    }

    /// **A transparency test would find nothing**, which is the failure this
    /// module's header warns is silent.
    #[test]
    fn ink_is_decided_by_colour_and_not_by_alpha() {
        assert!(
            is_ink(0, 0, 0, 255),
            "solid black at full alpha is ink; an alpha-based test would call it paper"
        );
        assert!(!is_ink(255, 255, 255, 255), "opaque white is paper");
        // The transparent-backdrop path is not used by the preview today, but
        // the guard must hold if a caller ever engages it.
        assert!(is_ink(255, 255, 255, 0), "an uncovered pixel counts as ink");
    }

    /// A saturated colour is ink even though it is bright — `min(R, G, B)`
    /// rather than a luminance, which is most of what CAD linework is.
    #[test]
    fn saturated_coloured_linework_is_ink() {
        for (name, (r, g, b)) in [
            ("yellow", (255u8, 255u8, 0u8)),
            ("cyan", (0, 255, 255)),
            ("magenta", (255, 0, 255)),
            ("CAD red", (255, 0, 0)),
        ] {
            assert!(
                is_ink(r, g, b, 255),
                "{name} is linework, not paper; a luminance test tuned for grey would \
                 miss it"
            );
        }
    }

    /// **The extent is never SMALLER than the ink**, checked at every pixel
    /// of a row, which is where a top-left-corner mapping quietly fails.
    #[test]
    fn the_extent_never_stops_short_of_the_ink() {
        const W: u32 = 400;
        for x in 0..W {
            let data = raster(W, 8, &[(x, 3, x + 1, 4)]);
            let mask = InkMask::from_rgba_premultiplied(W, 8, &data);
            let extent = mask
                .ink_extent(Rect::from_min_max(
                    egui::pos2(0.0, 0.0),
                    egui::pos2(1.0, 1.0),
                ))
                .unwrap_or_else(|| panic!("one pixel of ink at x={x} was not found at all"));
            let (lo, hi) = (x as f32 / W as f32, (x + 1) as f32 / W as f32);
            assert!(
                extent.min.x <= lo + 1e-6 && extent.max.x >= hi - 1e-6,
                "ink pixel {x} spans {lo}..{hi} of the page but the mask reported \
                 {:?}..{:?} — the extent must never be SMALLER than the ink, or a hatch \
                 stops short of a mark that will be cropped",
                extent.min.x,
                extent.max.x
            );
        }
    }

    /// A hairline narrower than one cell still lights its cell — the "any
    /// pixel in the cell" rule, which is what makes the downsample safe rather
    /// than merely cheap.
    #[test]
    fn a_single_pixel_of_ink_is_enough_to_light_a_cell() {
        let data = raster(400, 400, &[(390, 200, 391, 201)]);
        let mask = InkMask::from_rgba_premultiplied(400, 400, &data);
        assert!(
            mask.ink_extent(right_band(0.10)).is_some(),
            "one pixel of ink in the band must still be disclosed; the mask may \
             hatch more than is lost, never less"
        );
    }

    /// Degenerate rasters and regions produce no hatch and no panic. The
    /// preview reaches this whenever a render fails or a `/MediaBox` is
    /// nonsense, and a panic there would take the dialog down mid-print.
    #[test]
    fn degenerate_inputs_are_blank_rather_than_a_panic() {
        let empty = InkMask::from_rgba_premultiplied(0, 0, &[]);
        assert_eq!(empty.ink_extent(right_band(0.5)), None);

        let short = InkMask::from_rgba_premultiplied(4, 4, &[0u8; 8]);
        assert_eq!(short.ink_extent(right_band(0.5)), None);

        let data = raster(16, 16, &[(0, 0, 16, 16)]);
        let mask = InkMask::from_rgba_premultiplied(16, 16, &data);
        for bad in [
            Rect::from_min_max(egui::pos2(f32::NAN, 0.0), egui::pos2(1.0, 1.0)),
            Rect::from_min_max(egui::pos2(0.5, 0.5), egui::pos2(0.5, 0.5)),
            Rect::from_min_max(egui::pos2(2.0, 2.0), egui::pos2(3.0, 3.0)),
        ] {
            assert_eq!(
                mask.ink_extent(bad),
                None,
                "a degenerate region {bad:?} must yield no hatch"
            );
        }
    }

    /// The grid keeps the page's aspect, so a cell is about as wide as it is
    /// tall and the hatch's granularity does not depend on which axis it runs
    /// along.
    #[test]
    fn the_grid_follows_the_pages_aspect() {
        let data = raster(800, 200, &[]);
        let mask = InkMask::from_rgba_premultiplied(800, 200, &data);
        assert_eq!(mask.cols, CELLS_LONG_SIDE as usize);
        assert_eq!(mask.rows, CELLS_LONG_SIDE as usize / 4);

        // A raster smaller than the grid gets one cell per pixel rather than
        // an inflated grid of mostly-empty cells.
        let small = InkMask::from_rgba_premultiplied(6, 3, &raster(6, 3, &[]));
        assert_eq!((small.cols, small.rows), (6, 3));
    }
}
