//! **The pixel oracle.** The half of this harness that catches D2.
//!
//! ## Why pixels are an oracle at all
//!
//! Most defects have a state oracle: something is wrong in memory, and a test
//! that reads memory finds it. A whole class does not. Legibility, clipping,
//! occlusion and layout have exactly one honest oracle — what was actually
//! drawn — and no amount of asserting on the widget tree reaches them.
//!
//! `DEFECTS.md` D2 is the canonical example, and it is worth reading closely
//! because it explains why this module is shaped the way it is. Every
//! collapsible section heading in the Settings dialog renders near-white on
//! light grey. The cause is one unset field: the theme assigns
//! `widgets.active.fg_stroke` a near-white and `widgets.active.weak_bg_fill`
//! the accent, but never assigns `widgets.active.bg_fill` — so widgets that
//! paint with `bg_fill` (`egui_tiles` tab buttons, `CollapsingHeader` headers)
//! get near-white text on a light background.
//!
//! Two tests sit directly adjacent to that bug and neither could catch it:
//!
//! * one checks `text` against `surface`/`panel`, and never tests the colour
//!   that is wrong;
//! * the other **asserts that the wrong colour stays light** — correctly, for
//!   its own stated purpose, because that colour also sits over the white page.
//!
//! Both are testing the palette. The defect is in the *pairing*, and the
//! pairing only exists once something is drawn. So: read the picture.
//!
//! ## `contrast_at` — the algorithm, and why not min/max
//!
//! The obvious implementation is "brightest pixel versus darkest pixel in the
//! region". It is wrong in both directions and would make this gate useless:
//!
//! * **False pass.** One stray dark pixel — a window border clipped into the
//!   region, a scrollbar, a single antialiased corner of an unrelated icon —
//!   gives a blank region a 15:1 contrast. The check would pass on a region
//!   containing no text at all, which is exactly the D2 symptom.
//! * **False fail.** Antialiasing puts a continuum between foreground and
//!   background, so the extremes are unrepresentative of the glyph the reader
//!   actually sees.
//!
//! So [`contrast_at`] works on **populations**:
//!
//! 1. Quantise every pixel to a 5-bits-per-channel bucket (32 768 buckets).
//!    Coarse enough to fuse antialiasing into its neighbours, fine enough to
//!    keep a real foreground and a real background apart.
//! 2. The **background** is the most populous bucket. In any region containing
//!    text, most pixels are not text.
//! 3. The **foreground** is the bucket, among those holding at least
//!    [`MIN_FOREGROUND_SHARE`] of the region, whose luminance is furthest from
//!    the background's. The share floor is what makes a single stray pixel
//!    unable to vote.
//! 4. Each bucket reports its **mean colour**, not the bucket centre, so
//!    quantisation does not bias the measurement.
//! 5. Contrast is the WCAG ratio `(L1 + 0.05) / (L2 + 0.05)`, from 1.0
//!    (identical) to 21.0 (black on white).
//!
//! If no bucket clears the share floor, the region is uniform: foreground and
//! background come back equal and the ratio is 1.0. That is the correct answer
//! for a blank region, and it is why [`region_not_uniform`] exists as a
//! separate question — "nothing was drawn here" and "what was drawn is
//! invisible" are different failures with different fixes, and a check should
//! be able to tell the operator which one it found.
//!
//! ## Thresholds
//!
//! WCAG 2.1 asks 4.5:1 for body text and 3:1 for large text. [`AA_LARGE`] is
//! this harness's default because ribbon group captions and dialog section
//! headings are short, styled strings where 3:1 is a defensible floor and is
//! not a matter of taste — it is a published standard, which is what stops a
//! failing check turning into an argument about whether the grey is nice.
//!
//! For calibration: D2's headings measure around **1.1:1**. The threshold does
//! not need to be finely tuned to catch that; it needs to exist.

use crate::geom::PixRect;
use crate::image::{Image, Rgb};

/// WCAG 2.1 AA for large text. The harness default.
pub const AA_LARGE: f64 = 3.0;

/// WCAG 2.1 AA for body text. Available for checks that target running prose.
pub const AA_BODY: f64 = 4.5;

/// A bucket must hold at least this fraction of the region to be considered
/// the foreground.
///
/// 0.5%: a 200×30 caption region is 6 000 pixels, so this is 30 pixels — about
/// one glyph stroke, and far more than any stray edge pixel. Raising it makes
/// the oracle blind to thin text; lowering it lets a scrollbar sliver vote.
pub const MIN_FOREGROUND_SHARE: f64 = 0.005;

/// Bits kept per channel when bucketing. 5 bits = 32 levels per channel.
const QUANT_BITS: u8 = 5;

/// What [`contrast_at`] found.
#[derive(Clone, Copy, Debug)]
pub struct ContrastReport {
    /// Mean colour of the most populous bucket.
    pub background: Rgb,
    /// Its share of the sampled pixels, `0.0..=1.0`.
    pub background_share: f64,
    /// Mean colour of the furthest-luminance bucket clearing the share floor.
    /// Equal to [`Self::background`] when the region is uniform.
    pub foreground: Rgb,
    /// Its share of the sampled pixels.
    pub foreground_share: f64,
    /// WCAG contrast ratio, `1.0..=21.0`.
    pub ratio: f64,
    /// How many pixels were actually read.
    ///
    /// Reported because a region that has drifted off the edge of the image
    /// samples fewer pixels than it should, and a check that printed only the
    /// ratio would present a miscalibrated region as a legibility verdict.
    pub sampled: usize,
}

impl ContrastReport {
    /// Did this region clear a threshold?
    #[must_use]
    pub fn meets(&self, threshold: f64) -> bool {
        self.ratio >= threshold
    }

    /// A one-line summary for a check's report.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{:.2}:1 (fg {} {:.1}% on bg {} {:.1}%, {} px sampled)",
            self.ratio,
            self.foreground,
            self.foreground_share * 100.0,
            self.background,
            self.background_share * 100.0,
            self.sampled
        )
    }
}

/// How varied a region is.
#[derive(Clone, Copy, Debug)]
pub struct UniformityReport {
    /// Distinct quantised buckets present.
    pub distinct: usize,
    /// The most populous bucket's share, `0.0..=1.0`.
    pub dominant_share: f64,
    /// Pixels read.
    pub sampled: usize,
}

impl UniformityReport {
    /// Is this region effectively a flat colour?
    ///
    /// Two conditions, and both are needed. A gradient has many buckets and no
    /// dominant one; a flat fill with a single antialiased pixel has two
    /// buckets and a 99.99% dominant one. Only requiring "more than one
    /// bucket" would call the second varied.
    #[must_use]
    pub fn is_uniform(&self) -> bool {
        self.distinct <= 1 || self.dominant_share > 0.999
    }

    /// A one-line summary.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{} distinct colour bucket(s), dominant {:.2}%, {} px sampled",
            self.distinct,
            self.dominant_share * 100.0,
            self.sampled
        )
    }
}

/// WCAG relative luminance of an sRGB colour, `0.0..=1.0`.
///
/// The gamma expansion is not decoration. A naive `(r+g+b)/3` says mid-grey
/// text on white has plenty of contrast; the perceptual curve says it does
/// not, and the reader agrees with the curve.
#[must_use]
pub fn relative_luminance(c: Rgb) -> f64 {
    fn channel(v: u8) -> f64 {
        let s = f64::from(v) / 255.0;
        if s <= 0.039_28 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

/// The WCAG contrast ratio between two colours, `1.0..=21.0`.
#[must_use]
pub fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// One quantised colour bucket's running totals.
#[derive(Clone, Copy, Default)]
struct Bucket {
    count: u64,
    r: u64,
    g: u64,
    b: u64,
}

impl Bucket {
    fn mean(&self) -> Rgb {
        if self.count == 0 {
            return Rgb::new(0, 0, 0);
        }
        Rgb::new(
            (self.r / self.count) as u8,
            (self.g / self.count) as u8,
            (self.b / self.count) as u8,
        )
    }
}

fn bucketize(img: &Image, region: PixRect) -> (Vec<(u32, Bucket)>, usize) {
    use std::collections::HashMap;
    let shift = 8 - QUANT_BITS;
    let mut map: HashMap<u32, Bucket> = HashMap::new();
    let mut sampled = 0usize;
    for px in img.pixels_in(region) {
        sampled += 1;
        let key = (u32::from(px.r >> shift) << (QUANT_BITS * 2))
            | (u32::from(px.g >> shift) << QUANT_BITS)
            | u32::from(px.b >> shift);
        let e = map.entry(key).or_default();
        e.count += 1;
        e.r += u64::from(px.r);
        e.g += u64::from(px.g);
        e.b += u64::from(px.b);
    }
    let mut v: Vec<(u32, Bucket)> = map.into_iter().collect();
    // Sort by population, then by key, so the result is deterministic when two
    // buckets tie. A non-deterministic oracle is worse than a wrong one.
    v.sort_by(|a, b| b.1.count.cmp(&a.1.count).then(a.0.cmp(&b.0)));
    (v, sampled)
}

/// Measure the luminance gap between the dominant foreground and background in
/// a region.
///
/// See the module docs for the algorithm and for why it is not min/max. A
/// region with no pixels — off the edge of the image, or degenerate — reports
/// black on black at 1.0 with `sampled: 0`, and callers are expected to look
/// at `sampled` before reading the ratio as a verdict.
#[must_use]
pub fn contrast_at(img: &Image, region: PixRect) -> ContrastReport {
    let (buckets, sampled) = bucketize(img, region);
    if sampled == 0 || buckets.is_empty() {
        let black = Rgb::new(0, 0, 0);
        return ContrastReport {
            background: black,
            background_share: 0.0,
            foreground: black,
            foreground_share: 0.0,
            ratio: 1.0,
            sampled,
        };
    }

    let total = sampled as f64;
    let (_, bg_bucket) = buckets[0];
    let bg = bg_bucket.mean();
    let bg_lum = relative_luminance(bg);

    let mut fg = bg;
    let mut fg_share = 0.0;
    let mut best_gap = 0.0;
    for (_, b) in buckets.iter().skip(1) {
        let share = b.count as f64 / total;
        if share < MIN_FOREGROUND_SHARE {
            // Sorted by population, so everything after this is smaller too.
            break;
        }
        let mean = b.mean();
        let gap = (relative_luminance(mean) - bg_lum).abs();
        if gap > best_gap {
            best_gap = gap;
            fg = mean;
            fg_share = share;
        }
    }

    ContrastReport {
        background: bg,
        background_share: bg_bucket.count as f64 / total,
        foreground: fg,
        foreground_share: fg_share,
        ratio: contrast_ratio(fg, bg),
        sampled,
    }
}

/// Is anything at all drawn in this region?
///
/// The companion question to [`contrast_at`], and it exists because "nothing
/// was drawn" and "what was drawn is invisible" are different defects:
///
/// * A **uniform** region where a caption should be means the caption is
///   missing — the 2026-08-08 screenshot audit found two ribbon groups
///   rendering with no caption at all, which a contrast test alone would
///   report as low contrast and misdiagnose.
/// * A uniform region across a whole *window* means the capture is not a
///   picture of the application: the display was asleep, the window was never
///   raised, or the process died before the shot. pdfcer's predecessor script
///   learned this the expensive way — a run of blank screenshots got a
///   plausible invented cause attached to them (a compositor race) before the
///   real one was found (the monitor had powered down), and the fix that
///   mattered was not the one that was tried first, it was refusing to treat a
///   uniform capture as evidence at all.
#[must_use]
pub fn region_not_uniform(img: &Image, region: PixRect) -> UniformityReport {
    let (buckets, sampled) = bucketize(img, region);
    let dominant_share = buckets
        .first()
        .map_or(0.0, |(_, b)| b.count as f64 / sampled.max(1) as f64);
    UniformityReport {
        distinct: buckets.len(),
        dominant_share,
        sampled,
    }
}

/// **How much INK a strip carries, as distinct from how varied it is.**
///
/// # Why this exists beside [`region_not_uniform`]
///
/// `region_not_uniform` answers *"is this region all one colour?"* with
/// `distinct <= 1 || dominant_share > 0.999`. On a 2,112-pixel strip that
/// permits **two** stray pixels — and a panel edge always carries more than
/// two, because a one-pixel border column and the antialiased tips of a few
/// glyphs are furniture, not overflow.
///
/// Measured on `master-detail.png`, 2026-09-05, over the last 8 pt of the
/// Objects pane:
///
/// ```text
/// (232,232,234) x 2098   the panel's own plate
/// (196,198,202) x    8   the pane's right border column
/// four dark values x 1   isolated antialiased pixels
/// ```
///
/// `dominant_share` = 0.9934, below the 0.999 floor, so the strip read as
/// *"ink is running into the pane's right edge"* — about fourteen pixels, six
/// of which were single. **The statistic was maximally sensitive to the one
/// thing always present (antialiasing) and said nothing about the thing it
/// wanted (a run of glyph ink).**
///
/// ⇒ ★★★ **This is a better instrument, not a wider tolerance**, and the
/// distinction is the project's standing rule: when a measurement runs out,
/// read something else — never move the threshold until the failure stops.
/// Text that has genuinely overflowed is **dark** and **contiguous**: a clipped
/// word puts dozens of near-black pixels into several adjacent rows. Isolated
/// pixels and a border column are neither.
///
/// # What it measures
///
/// A pixel is *ink* when its luminance is at least `INK_CONTRAST` below the
/// strip's own dominant colour — so the plate is the reference and no constant
/// has to know what colour the theme is. The verdict is the **longest vertical
/// run** of ink in any single column: one isolated pixel is 1, an antialiased
/// glyph tip is 1 or 2, and a clipped capital is the height of the type.
#[must_use]
pub fn ink_run_into(img: &Image, region: PixRect) -> InkReport {
    /// How far below the plate a pixel must be to count as ink. Measured: the
    /// darkest antialiased stray in the observed strip was 103 against a plate
    /// of 232 — a distance of 129 — so a threshold based on stray VALUES would
    /// not separate them. The run length does the separating; this only
    /// excludes the border column (232 against 197 = 35).
    const INK_CONTRAST: i32 = 60;

    let (buckets, sampled) = bucketize(img, region);
    // The plate is the strip's own dominant bucket, averaged back to a colour,
    // so no constant here has to know what the theme is painting.
    let plate_luma = buckets
        .iter()
        .max_by_key(|(_, b)| b.count)
        .map_or(255 * 3, |(_, b)| {
            let n = b.count.max(1);
            i32::try_from((b.r + b.g + b.b) / n).unwrap_or(255 * 3)
        });

    let mut longest = 0usize;
    let mut ink = 0usize;
    for x in region.x..region.x + region.w {
        let mut run = 0usize;
        for y in region.y..region.y + region.h {
            let Some(px) = img.pixel(x, y) else { continue };
            let luma = i32::from(px.r) + i32::from(px.g) + i32::from(px.b);
            if plate_luma - luma >= INK_CONTRAST * 3 {
                ink += 1;
                run += 1;
                longest = longest.max(run);
            } else {
                run = 0;
            }
        }
    }
    InkReport {
        longest_run: longest,
        ink,
        sampled,
    }
}

/// What [`ink_run_into`] found.
#[derive(Debug, Clone, Copy)]
pub struct InkReport {
    /// The longest vertical run of ink pixels in any one column.
    pub longest_run: usize,
    /// Every ink pixel in the strip, run or not.
    pub ink: usize,
    /// How many pixels were looked at.
    pub sampled: usize,
}

impl InkReport {
    /// **Whether this is text and not furniture.**
    ///
    /// Three pixels, because two adjacent antialiased pixels are reachable on a
    /// steep glyph edge and three are not — while the shortest thing an
    /// operator would call clipped text is a lower-case x-height, which at this
    /// project's smallest shipped size is seven.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        self.longest_run >= 3
    }

    /// A one-line summary for a report.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "longest vertical ink run {} px, {} ink px of {} sampled",
            self.longest_run, self.ink, self.sampled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::Image;

    /// Build a test image: `bg` everywhere, with `fg` painted on `rows` rows
    /// of `cols` columns — a crude but honest stand-in for a line of text.
    fn text_on(bg: Rgb, fg: Rgb, w: u32, h: u32, coverage: f64) -> Image {
        let mut bgra = Vec::new();
        let painted = ((w as f64) * (h as f64) * coverage) as usize;
        for i in 0..(w * h) as usize {
            let c = if i < painted { fg } else { bg };
            bgra.extend_from_slice(&[c.b, c.g, c.r, 0xFF]);
        }
        Image::from_bgra(w, h, bgra).unwrap()
    }

    const WHOLE: PixRect = PixRect::new(0, 0, 100, 40);

    #[test]
    fn black_on_white_is_the_maximum_ratio() {
        let r = contrast_ratio(Rgb::new(0, 0, 0), Rgb::new(255, 255, 255));
        assert!((r - 21.0).abs() < 0.01, "{r}");
    }

    #[test]
    fn identical_colours_are_one_to_one() {
        let c = Rgb::new(70, 70, 70);
        assert!((contrast_ratio(c, c) - 1.0).abs() < 1e-9);
    }

    /// The D2 case, reduced: near-white text on light grey. The measured
    /// defect is around 1.1:1; anything under the threshold is a fail.
    #[test]
    fn near_white_on_light_grey_fails_the_threshold() {
        let img = text_on(
            Rgb::new(232, 232, 232),
            Rgb::new(250, 250, 250),
            100,
            40,
            0.15,
        );
        let c = contrast_at(&img, WHOLE);
        assert!(
            c.ratio < 1.3,
            "expected the D2 pairing to measure near 1:1, got {}",
            c.summary()
        );
        assert!(!c.meets(AA_LARGE));
    }

    #[test]
    fn dark_text_on_light_grey_passes_the_threshold() {
        let img = text_on(Rgb::new(232, 232, 232), Rgb::new(32, 32, 32), 100, 40, 0.15);
        let c = contrast_at(&img, WHOLE);
        assert!(c.meets(AA_LARGE), "{}", c.summary());
    }

    /// The false-pass a min/max implementation would produce, asserted as a
    /// test so nobody "simplifies" the population algorithm away: a blank
    /// region with ONE stray black pixel must not read as high contrast.
    #[test]
    fn a_single_stray_pixel_cannot_fake_contrast() {
        // Exactly one black pixel in a flat light-grey field: 1 / 4000 =
        // 0.025%, well under the 0.5% floor.
        let mut bgra = Vec::new();
        for i in 0..100 * 40 {
            let c = if i == 0 {
                Rgb::new(0, 0, 0)
            } else {
                Rgb::new(232, 232, 232)
            };
            bgra.extend_from_slice(&[c.b, c.g, c.r, 0xFF]);
        }
        let img = Image::from_bgra(100, 40, bgra).unwrap();
        let c = contrast_at(&img, WHOLE);
        assert!(
            (c.ratio - 1.0).abs() < 1e-6,
            "one stray pixel must not vote: {}",
            c.summary()
        );
    }

    #[test]
    fn a_flat_region_is_uniform_and_a_texted_one_is_not() {
        let flat = text_on(Rgb::new(200, 200, 200), Rgb::new(0, 0, 0), 100, 40, 0.0);
        assert!(region_not_uniform(&flat, WHOLE).is_uniform());

        let texted = text_on(Rgb::new(200, 200, 200), Rgb::new(0, 0, 0), 100, 40, 0.1);
        assert!(!region_not_uniform(&texted, WHOLE).is_uniform());
    }

    /// A flat fill with one differing pixel has two buckets, and calling that
    /// "varied" would let a blank capture through the guard.
    #[test]
    fn one_odd_pixel_does_not_make_a_flat_region_varied() {
        let mut bgra = Vec::new();
        for i in 0..100 * 40 {
            let c = if i == 0 {
                Rgb::new(0, 0, 0)
            } else {
                Rgb::new(200, 200, 200)
            };
            bgra.extend_from_slice(&[c.b, c.g, c.r, 0xFF]);
        }
        let img = Image::from_bgra(100, 40, bgra).unwrap();
        assert!(region_not_uniform(&img, WHOLE).is_uniform());
    }

    #[test]
    fn an_empty_region_reports_zero_samples_rather_than_a_verdict() {
        let img = text_on(Rgb::new(1, 1, 1), Rgb::new(2, 2, 2), 10, 10, 0.5);
        let c = contrast_at(&img, PixRect::new(50, 50, 10, 10));
        assert_eq!(c.sampled, 0);
        assert!((c.ratio - 1.0).abs() < 1e-9);
    }
}
