//! # `dialogs::print::ink` — which parts of a rendered sheet actually carry ink
//!
//! ## The question this module exists to answer, and why it is asked HERE
//!
//! Operator request O113, 2026-09-03:
//!
//! > *"can you make it so the red pattern you put over the page if it is going
//! > to print beyond the printable borders is only over the areas that extend
//! > beyond the printable page? Our drawing get drawn 1:1 and the area that
//! > isn't printed is just empty border."*
//!
//! The print preview hatches the **whole overhanging region** whenever the
//! placement reports a clip. `Placement::clipped` is a *geometric* verdict —
//! the page box exceeds the printable rectangle — and on a CAD sheet printed
//! 1:1 the part that exceeds it is empty paper. So the hatch shouted about
//! losing something on every drawing, and nothing was being lost.
//!
//! ★★★ **That is a disclosure which is technically true and practically
//! false, which is the worst kind.** An operator who sees the same red band on
//! every 1:1 drawing learns to ignore it, and then does not see it on the one
//! sheet where the border really does have a title block in it. A warning that
//! fires on the common harmless case trains the operator out of reading it.
//!
//! ## ★★ Why the raster, and NOT an engine content bounding box
//!
//! `OPERATOR_REQUESTS.md`'s O113 row offers two routes and leans toward the
//! first: an engine verb returning a page's ink extent (a real content bbox,
//! not the `/MediaBox`), described there as *"the honest general answer"*; or
//! sampling the preview raster this shell has already rendered, described as
//! *"cheaper … and approximate at the edges."* The row has the ranking
//! backwards, and it is worth stating why rather than merely choosing.
//!
//! **A content bounding box is a RECTANGLE, and a rectangle is conservative in
//! exactly the wrong direction for this question.** Ink at two opposite
//! corners of a sheet produces a bbox that covers everything between them,
//! including an overhang band with no ink anywhere in it. The engine verb
//! would therefore report *"content reaches into the overhang"* on the very
//! drawing shape O113 is about — a title block at the bottom-right and a
//! revision block at the top-right make a bbox spanning the whole right edge,
//! whether or not the strip that will actually be cropped holds a single
//! stroke. It would be a *different* proxy for the operator's question, not a
//! better answer to it.
//!
//! [`super::preview::texture_for`] already calls
//! `pdfcer_render::render_page_with_view` and holds the resulting
//! `rendered.pixmap` — **the very pixels that are about to print**, rendered
//! through the same `RenderOptions` the spooler uses (see
//! [`super::render_options`]). The question *"is anything lost?"* is a
//! question about those pixels. Sampling them answers it **directly** rather
//! than by proxy, and the "approximate at the edges" caveat is bounded and
//! stated: see [`CELLS_LONG_SIDE`] and [`InkMask::ink_extent`], which are
//! approximate only in the direction of hatching **slightly more** than is
//! strictly lost, never less.
//!
//! A real engine content-extent verb would still be worth having for other
//! questions (auto-crop, fit-to-content, trim detection). It is not the better
//! answer to *this* one.
//!
//! ## ★★★ What "ink" turned out to MEAN, measured rather than assumed
//!
//! This was the one thing in the change that could be silently backwards, so
//! it was verified against real rasters before a line of the algorithm was
//! written. Two measurements, both taken by rendering through the same entry
//! point the preview uses, `render_page_with_view` with the default
//! `RenderOptions`:
//!
//! **1. The page is composited on OPAQUE WHITE, so alpha says nothing.**
//! `pdfcer-render` starts the page group fully transparent (ISO 32000-1
//! §11.4.7 — the page group is *isolated*, so its initial backdrop is `U`) and
//! then adds the paper in one final pass, `flatten_page_group_over_white`,
//! which sets **every** pixel's alpha to 255. `RenderOptions::backdrop`
//! defaults to `PageBackdrop::White`, and `super::render_options` never calls
//! `with_backdrop`, so the preview always gets the white-composited result.
//!
//! Measured on three documents (`src/app/assets/blank-a4.pdf`,
//! `fixtures/a1-titleblock.pdf`, `fixtures/paragraph.pdf`): **`non_opaque = 0`
//! on all three**, every pixel alpha 255. A transparency test would therefore
//! find no ink anywhere and hatch **nothing, ever** — the failure mode is
//! silent, and it would look like the fix working perfectly.
//!
//! **2. Blank paper is NOT reliably `(255, 255, 255)`, and this is the
//! finding that sets the threshold.** The blank A4 template renders as
//! `(255, 255, 255, 255)` in every pixel, as expected. But
//! `fixtures/a1-titleblock.pdf` — a large-format CAD sheet, i.e. exactly the
//! document population this operator prints — renders its paper as
//! **`(249, 249, 249)`**: 236,443 of its 250,916 pixels, against only 7,246
//! at pure white. The sheet carries a near-white background fill.
//!
//! ⇒ A naive `min(R, G, B) < 255` ink test classifies **97% of that sheet as
//! ink**, including every square millimetre of its empty border, and hatches
//! the entire overhang. It would reproduce O113's defect exactly while
//! appearing to fix it. [`INK_MAX_LEVEL`] exists because of that number, and
//! its doc comment carries the measurement.
//!
//! ## Rule 4 — this is disclosure, and it stays on the preview
//!
//! `pdfce_FeatureRequests/README.md` rule 4, clause 2: *"No badge, tint, red
//! flag, dashed outline or 'provisional' layer drawn into the page view."*
//! The hatch is painted by [`super::preview::paint`] into the **print
//! dialog's preview canvas**, over the diagram of a piece of paper. Nothing in
//! this module or its caller touches a `RenderOptions`, an `EditSession`, a
//! staging buffer or a saved byte; the pixmap it reads is borrowed immutably
//! and dropped. **The page as rendered for printing is bit-for-bit unchanged
//! by this change** — the one-line test is that a screenshot of what prints is
//! identical before and after, and it is identical because the print path
//! never calls into here at all.
//!
//! Making the hatch *narrower* also moves in the safe direction for clause 2:
//! there is strictly less non-content drawn over the operator's page than
//! before.

use egui::Rect;

/// Cells across the mask's **longer** side.
///
/// # ★ What this number buys, and what it costs, as arithmetic rather than a
/// # guess
///
/// The mask is a downsample of the preview raster, whose longest side is
/// capped at `super::preview::MAX_SIDE_PX` = 2200 px. At 256 cells on the long
/// side, one cell is at most `2200 / 256 ≈ 8.6` raster pixels on a side.
///
/// **Memory.** The grid is `Vec<bool>`, one byte per cell. The worst case is a
/// square page, `256 × 256 = 65,536` cells = **64 KiB**. The raster it
/// describes is up to `2200 × 2200 × 4` bytes = **19.3 MiB**, so the mask
/// costs **0.33%** of the picture it summarises, and is built once per raster
/// rather than once per frame. A bit-packed grid would take it to 8 KiB and is
/// deliberately not done: 64 KiB against 19 MiB is not a cost worth trading
/// legibility for, and this module's whole value is that a reader can check
/// its arithmetic.
///
/// **Spatial resolution, in the units the operator cares about.** On a US
/// Letter sheet (612 × 792 pt) the long side is 792 pt, so a cell is
/// `792 / 256 ≈ 3.1 pt ≈ 1.1 mm`. On an ANSI E sheet (2448 × 3168 pt) a cell
/// is `3168 / 256 ≈ 12.4 pt ≈ 4.4 mm`. Both are far finer than any margin
/// decision an operator makes from a preview, and the error is **always in the
/// direction of hatching slightly more** than is strictly lost — see
/// [`InkMask::from_rgba_premultiplied`], where a cell is inked if *any* pixel
/// in it is.
///
/// **Why not simply test the raster pixels directly and skip the mask?** The
/// overhang band would be re-scanned on every frame, at up to a few million
/// pixels a frame, for an answer that cannot change until the raster does. The
/// mask is computed exactly once per raster and cached beside it under the
/// same key — see `super::preview::PreviewKey` and the field it keys.
const CELLS_LONG_SIDE: u32 = 256;

/// The lightest `min(R, G, B)` a pixel may have and still count as **ink**.
///
/// # ★★★ This constant is the whole change, and it is set from a MEASUREMENT
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
///
/// One cell is inked when *any* raster pixel inside it is ink (see
/// [`INK_MAX_LEVEL`]). That "any" is what makes the approximation safe: a
/// hairline a third of a pixel wide still lights its cell, so the mask never
/// reports blank where the raster has a mark. The cost is that a cell holding
/// one stroke and a great deal of paper reads as fully inked, which hatches at
/// most one cell too far in each direction — quantified in
/// [`CELLS_LONG_SIDE`].
///
/// # Coordinates: normalised page space, on purpose
///
/// [`Self::ink_extent`] takes and returns rectangles in **0..1 page space**,
/// where `(0, 0)` is the page's top-left corner and `(1, 1)` its
/// bottom-right — the same convention `egui::Painter::image`'s UV rectangle
/// uses, which is what the preview already speaks when it draws the page.
///
/// It deliberately does **not** speak screen points. The screen rectangle
/// depends on the fit, the zoom and the pan, all of which change every frame;
/// a mask that spoke screen coordinates would have to be rebuilt on a pan.
/// Normalised page space depends on nothing but the page, which is exactly the
/// lifetime the mask has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InkMask {
    /// Cells across, at least 1.
    cols: usize,
    /// Cells down, at least 1.
    rows: usize,
    /// Row-major, `cols * rows` entries. `true` = at least one ink pixel.
    cells: Vec<bool>,
}

impl InkMask {
    /// Build a mask from a rendered page's **premultiplied** RGBA8 bytes.
    ///
    /// # ★ Premultiplied, and why it does not complicate the test here
    ///
    /// `tiny-skia` stores pixels premultiplied — `[R·A, G·A, B·A, A]` — and
    /// `crate::render::raster`'s header is emphatic that reading them as
    /// straight bytes "silently darkens every partially transparent pixel".
    /// That hazard is real for *upload*, where the bytes are handed to epaint
    /// with a declared convention.
    ///
    /// It does not bite here, and the reason is the measurement in this
    /// module's header rather than luck: the preview's pages are composited
    /// over opaque white, so **`A = 255` in every pixel**, and premultiplying
    /// by 1.0 is the identity. The stored bytes *are* the colour. This
    /// function still reads the alpha byte and refuses to treat a
    /// non-opaque pixel's colour channels at face value — see the guard
    /// below — so it stays correct if a future caller ever hands it a
    /// `PageBackdrop::Transparent` raster, rather than being silently wrong on
    /// one.
    ///
    /// # What is done with a partially transparent pixel
    ///
    /// It is **ink**. On a transparent-backdrop raster, `A < 255` means the
    /// page group did not fully cover that pixel, and a partially covered
    /// pixel is one something was painted into. Treating it as ink is the
    /// conservative reading and matches the "any pixel in the cell" rule.
    ///
    /// # Degenerate inputs
    ///
    /// A zero-sized raster, or a `data` slice shorter than `width * height * 4`
    /// (which cannot happen from `tiny-skia` but is not this function's to
    /// assume), yields a 1 × 1 mask with no ink. Nothing is hatched, the
    /// geometric clip disclosure is untouched, and no index can be out of
    /// bounds.
    pub(super) fn from_rgba_premultiplied(width: u32, height: u32, data: &[u8]) -> Self {
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

        // ★★ A PIXEL IS AN AREA, NOT A POINT, and this is where that stopped
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
    ///
    /// ★ `None` is the whole point of O113. *"No ink in the band ⇒ no hatch at
    /// all"* — the 1:1 CAD sheet whose overhang is empty paper gets no red
    /// pattern, because there is nothing to warn about.
    ///
    /// # The extent is snapped OUT to cell boundaries, never in
    ///
    /// The returned rectangle is the union of the *whole cells* that are both
    /// inked and overlapping `region`, clamped back into `region`. It is
    /// therefore never smaller than the true ink extent and at most one cell
    /// larger on each side. Snapping inward would be the unsafe direction: it
    /// could draw a hatch that stops short of a mark that will in fact be
    /// cropped, which is a disclosure understating a loss — the one error this
    /// whole surface exists to avoid.
    ///
    /// # Which cells "overlap `region`"
    ///
    /// Cell `(col, row)` covers `[col/cols, (col+1)/cols] × [row/rows,
    /// (row+1)/rows]`. A cell is considered when that box overlaps `region`
    /// with positive area, so a region ending exactly on a cell boundary does
    /// not drag in the cell beyond it. The column and row ranges are computed
    /// by flooring the region's minimum and taking the ceiling of its maximum,
    /// which is the same "cover, do not crop" convention
    /// `pdfcer_render::region_device_geometry` documents for its own tiling.
    ///
    /// # Inputs outside the page
    ///
    /// `region` is intersected with the unit square first, so a caller that
    /// hands over a band extending past the page edge — which the print
    /// preview's overhang band routinely does, since the whole point is that
    /// it runs off the printable area — gets an answer about the page rather
    /// than an out-of-range index. An empty or non-finite region yields
    /// `None`.
    pub(super) fn ink_extent(&self, region: Rect) -> Option<Rect> {
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
///
/// See [`INK_MAX_LEVEL`] for the measurement behind the threshold, and
/// [`InkMask::from_rgba_premultiplied`] for why alpha is checked at all when
/// every pixel the preview produces today is opaque.
///
/// `min(R, G, B)` rather than a luminance: a saturated colour has a low
/// minimum channel even when its luminance is high — a pure yellow
/// `(255, 255, 0)` is 89% luminance and unmistakably ink — so the minimum is
/// what catches coloured linework, which is most of what a CAD sheet is made
/// of. A luminance test would have to be tuned per hue to see the same marks.
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
    ///
    /// Opaque white is what `render_page_with_view` actually produces (this
    /// module's header records the measurement), so a fixture built any other
    /// way would be testing a raster the preview never sees.
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

    /// ★★★ **The operator's own case: a 1:1 drawing whose overhang is empty
    /// paper gets NO hatch.**
    ///
    /// > *"Our drawing get drawn 1:1 and the area that isn't printed is just
    /// > empty border."*
    ///
    /// The page carries plenty of ink — a title block in the middle — and the
    /// placement would report a clip, because the page box does exceed the
    /// printable rectangle. The band that will actually be cropped is blank,
    /// so nothing is hatched. This is the request, in one assertion.
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

    /// ★ **One inked cell in the overhang is hatched, and nothing else is.**
    ///
    /// The same page as the test above with a single small mark added out in
    /// the border — a stray revision stamp, a pdf-dimension leader that ran
    /// past the frame (rule 15: CAD-exported page content, not a ce dimension
    /// pdfcer authored),
    /// the corner of a title block. The extent returned must cover that mark
    /// and must not spread to the rest of the band.
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

    /// ★ **Ink that is entirely inside the printable rectangle hatches
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

    /// ★★★ **Near-white CAD paper is not ink**, and this is the measurement
    /// [`INK_MAX_LEVEL`] exists for.
    ///
    /// `fixtures/a1-titleblock.pdf` renders its paper as `(249, 249, 249)` —
    /// 236,443 of 250,916 pixels. Under the intuitive `< 255` test that whole
    /// sheet, empty border included, is ink and the hatch covers everything:
    /// O113's defect, reintroduced by a wrong definition of the word.
    ///
    /// The measured value is written into the fixture rather than described,
    /// so if either the constant or the exporter's paper colour moves, this
    /// says which.
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

    /// ★ **A transparency test would find nothing**, which is the failure this
    /// module's header warns is silent.
    ///
    /// Every pixel the preview renders is alpha 255 — measured on three
    /// documents. A mask built on alpha would return `None` for every region of
    /// every page, hatch nothing ever, and look exactly like the fix working.
    /// This pins that the test used is the colour one.
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

    /// ★★ **The extent is never SMALLER than the ink**, checked at every pixel
    /// of a row, which is where a top-left-corner mapping quietly fails.
    ///
    /// A pixel is an area. Assigning it to the single cell its top-left corner
    /// lands in leaves the far edge of that pixel outside the cell whenever the
    /// grid does not divide the raster — so the reported extent stops short of
    /// the ink by up to one pixel, in the direction that under-discloses a
    /// loss. See [`InkMask::from_rgba_premultiplied`] for the measured case.
    ///
    /// This sweeps a one-pixel mark across a whole row of a raster whose width
    /// (400) is not a multiple of the grid (256), and asserts at every position
    /// that the extent contains the pixel's **full** span. A top-left mapping
    /// fails this at roughly a third of the positions.
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
