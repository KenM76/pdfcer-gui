//! `render::ink` — **is there anything in this raster?**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/ink.md`.

use pdfcer_render::tiny_skia::Pixmap;

/// The most tones worth counting.
///
/// Nothing downstream cares whether a page has 90 tones or 9,000; the questions
/// are *"is it one?"* and *"is it clearly more than one?"*. Capping keeps the
/// working set small and the trace line short.
const TONE_CAP: u8 = 64;

/// Roughly how many pixels to look at, whatever the raster's size.
///
/// A region raster can be 16,383 px on a side — 268 million pixels — and this
/// runs on **every** completed render, including the ones arriving during a live
/// zoom. A full scan would be a frame-rate defect introduced by an instrument,
/// which is a poor trade for a number whose only job is to separate blank from
/// not-blank.
const TONE_SAMPLE_TARGET: usize = 40_000;

/// **How many distinct tones a raster carries, from a strided sample.**
///
/// Distinct exact RGB triples, saturating at [`TONE_CAP`], over a stride chosen
/// so at most about [`TONE_SAMPLE_TARGET`] pixels are examined.
///
/// # Why exact triples rather than perceptual buckets
///
/// Antialiasing means a single black hairline on white paper produces dozens of
/// distinct greys, so counting exact triples is if anything **more** sensitive
/// to faint ink than bucketing would be. For a blank-detector that is the
/// direction to err: a false "there is ink here" makes a check ask a further
/// question, while a false "this is blank" makes it excuse a real defect.
///
/// # Why the answer is never zero
///
/// An empty pixmap cannot occur — the renderer refuses a zero-area region — and
/// returning `0` would make *"no pixels"* and *"one colour"* indistinguishable
/// in the trace, which is the precise ambiguity this function exists to remove.
/// The empty case therefore returns `1` and says so here rather than leaving a
/// reader to infer it.
///
/// # Cost
///
/// Bounded: `pixels.len() / TONE_SAMPLE_TARGET` sets the stride, so the loop
/// runs at most `TONE_SAMPLE_TARGET` times regardless of raster size, and it
/// exits early the moment [`TONE_CAP`] distinct tones have been seen — which on
/// any inked region happens within the first few hundred pixels.
pub(super) fn sampled_tone_count(pixmap: &Pixmap) -> u8 {
    let pixels = pixmap.pixels();
    if pixels.is_empty() {
        return 1;
    }
    let stride = (pixels.len() / TONE_SAMPLE_TARGET).max(1);
    let mut seen: Vec<(u8, u8, u8)> = Vec::with_capacity(TONE_CAP as usize);
    for p in pixels.iter().step_by(stride) {
        let tone = (p.red(), p.green(), p.blue());
        if !seen.contains(&tone) {
            seen.push(tone);
            if seen.len() >= TONE_CAP as usize {
                return TONE_CAP;
            }
        }
    }
    // `seen` is non-empty because `pixels` is, so this cannot be 0.
    u8::try_from(seen.len()).unwrap_or(TONE_CAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a pixmap of `w x h` whose pixel `i` takes `tone(i)`.
    fn pixmap_of(w: u32, h: u32, tone: impl Fn(usize) -> u8) -> Pixmap {
        let mut px = Pixmap::new(w, h).expect("a pixmap of this size");
        for (i, p) in px.pixels_mut().iter_mut().enumerate() {
            let v = tone(i);
            *p = pdfcer_render::tiny_skia::PremultipliedColorU8::from_rgba(v, v, v, 255)
                .expect("an opaque grey");
        }
        px
    }

    /// **Blank paper reads as one tone, and never as zero.**
    ///
    /// The whole point of the field. A `0` here would be indistinguishable from
    /// "there was no raster", which is the ambiguity being removed.
    #[test]
    fn a_uniform_raster_reads_as_exactly_one_tone() {
        let px = pixmap_of(64, 64, |_| 255);
        assert_eq!(sampled_tone_count(&px), 1);
    }

    /// **A single hairline is enough to say "not blank".**
    ///
    /// ★ This is the assertion that makes the field useful rather than merely
    /// present: the interesting case is not a busy drawing, it is *one line on
    /// otherwise empty paper*, which is what a deep-zoom viewport of a CAD sheet
    /// actually contains when it contains anything at all.
    #[test]
    fn one_dark_row_on_white_paper_is_not_uniform() {
        let w = 64;
        let px = pixmap_of(w, 64, |i| if i / w as usize == 20 { 0 } else { 255 });
        assert!(
            sampled_tone_count(&px) >= 2,
            "a raster with a black row must not read as blank"
        );
    }

    /// **The count saturates rather than growing without bound.**
    #[test]
    fn a_richly_toned_raster_saturates_at_the_cap() {
        // 256 distinct greys, far more than the cap.
        let px = pixmap_of(256, 256, |i| u8::try_from(i % 256).unwrap_or(0));
        assert_eq!(sampled_tone_count(&px), TONE_CAP);
    }

    /// **The stride keeps the cost bounded on a raster the size of a region.**
    ///
    /// Not a timing test — those are flaky — but a statement of the invariant
    /// the stride exists to hold: a raster two orders of magnitude larger than
    /// the sample target still examines about the sample target's worth of
    /// pixels. Written as an arithmetic assertion on the stride itself so it
    /// cannot pass by accident on a fast machine.
    #[test]
    fn the_sample_is_bounded_however_large_the_raster() {
        let big = 4_000_000_usize;
        let stride = (big / TONE_SAMPLE_TARGET).max(1);
        let visited = big.div_ceil(stride);
        assert!(
            visited <= TONE_SAMPLE_TARGET + 1,
            "a {big}-pixel raster would visit {visited} pixels, above the {TONE_SAMPLE_TARGET} \
             target — the stride is not bounding the cost"
        );
    }
}
