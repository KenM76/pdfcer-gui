//! # `render::ceiling` — the zoom ceiling this document *taught* the shell
//!
//!
//! > *"If this error is caused by some other limitation that will always
//! > happen, zoom should stop at the limit and not end up showing an error —
//! > the canvas will just stop zooming in and can still function. The error can
//! > still be shown on the bottom bar so the user has some idea as to why
//! > zooming stopped short of 1 trillion percent."*
//!
//! Four separable requirements, and it is worth naming them separately because
//! three of them are *not* about rendering at all:
//!
//! 1. **Stop at the limit.** The zoom ceiling must come down to where the page
//!    can actually be drawn — [`RasterCeiling`], this file.
//! 2. **Do not show an error.** The canvas must keep a picture, which follows
//!    from (1): if the zoom never goes past the wall, no render is ever
//!    refused, so there is nothing to paint a sentence about.
//! 3. **Still function.** Panning, selecting, editing all keep working,
//!    because the zoom was clamped rather than the canvas disabled.
//! 4. **Say why, on the bottom bar.** `crate::app::status::rasterstop`.
//!
//! ## ★★★ Why the ceiling has to be LEARNED rather than derived
//!
//! This project already has two *derived* ceilings and they are both in
//! [`crate::viewer::ceiling`]: the whole-page pixmap limit
//! (`max_zoom_for_page`, arithmetic on the page's `/MediaBox` and
//! `MAX_PIXMAP_EDGE`) and the `f32` positioning limit
//! (`SUB_PIXEL_CONTENT_EXTENT`). Both are closed-form, both are exact, and
//! neither is the wall the operator met.
//!
//! The wall he met is inside `tiny-skia`, and it is **content-dependent**. The
//! engine's own measurement, quoted in `render::worker`'s `RasterizerLimit`
//! arm: an E-size sheet failed at scale **284,964** where a business card
//! reached **8,053,069** — a factor of 28 between two pages, decided by the
//! magnitude of the coordinates in their content streams and the size of the
//! intermediate buffers those produce. There is no expression of the page's
//! metadata that predicts it. The engine cannot predict it either; it catches
//! the panic and reports the scale that failed.
//!
//! ⇒ So the only honest source of the number is **the refusal itself**. One
//! render fails, the shell writes down the scale, and that page never goes
//! that far again.
//!
//! ## The property that makes one observation enough
//!
//! Both raster refusals are overflows of a product of *the page's own extent*
//! and *the scale*. They are therefore **monotonic in the scale**: a page that
//! refused at `s` refuses at every scale above `s`. That single fact is what
//! turns one failure into a permanent ceiling instead of a guess — and it is
//! recorded on [`crate::render::worker::RefusalKind::BeyondRaster`] as the
//! variant's defining property, because it is the reason that variant exists
//! apart from `Other`.
//!
//! ## ★★ Why the ceiling is BELOW the scale that failed, and by how much
//!
//! A ceiling *at* the failing scale would be a ceiling the page cannot draw
//! at: the refusal happened there. So the learned value is backed off by
//! [`BACKOFF`].
//!
//! The back-off is 0.75 rather than something finer, and the reasoning is the
//! shape of the failure rather than a taste for round numbers. `BadRasterSize`
//! has a sharp boundary (a pixel count crossing `MAX_PIXMAP_EDGE`) and 0.99
//! would be enough for it. `RasterizerLimit` does not: it is a `usize` index
//! computed from a path's transformed coordinates, and the engine's reply
//! called the panic text *"third-party text and explicitly not a contract"* —
//! which means the shell cannot know how far below the first observed failure
//! the last *success* lies. A quarter of an order of magnitude is about two
//! wheel notches at these magnitudes, invisible against a zoom of 28 million
//! percent, and it buys a margin no arithmetic here can justify precisely.
//!
//! ★ And it does not have to be right, only safe-in-the-limit: the ceiling
//! **ratchets**. [`RasterCeiling::learn`] keeps the minimum, so if 0.75 of the
//! first failure still refuses, the second refusal lowers it again. The
//! sequence converges downward and costs one dead render per step — which is
//! why a conservative factor is preferred to an optimistic one, but why
//! neither can be wrong for long.
//!
//! ## ★ Why it is per page and keyed on the page's epoch
//!
//! Per page, because the limit is 28× different between two pages of the same
//! document (above). Per **epoch** — `crate::app::state::pageepoch::PageEpochs`
//! — because an edit can change the content that caused the overflow: deleting
//! the one enormous path raises the wall, and a remembered ceiling would then
//! be a magnification limit the document no longer has. A stale entry is
//! discarded rather than cleared, for the reason
//! `crate::app::actions::last_edit_disclosure` gives about its own epoch
//! comparison: *state that must be cleared is state that will one day be shown
//! against the wrong document.*
//!
//! ## What this module deliberately does NOT do
//!
//! It does not touch the zoom. Learning is a record; *applying* the record is
//! [`crate::viewer::zoom_ceiling`]'s job, and pulling the current zoom back to
//! it is `crate::render::settle::absorb`'s, at the one point a refusal is
//! absorbed.
//! Keeping the three apart is what lets the arithmetic be unit-tested with no
//! document, no renderer and no frame.

use std::collections::BTreeMap;

/// How far below the scale that refused the learned ceiling sits.
///
/// See the module header for why this is 0.75 and why being approximately
/// right is sufficient.
const BACKOFF: f32 = 0.75;

/// **Per-page raster scales this document has been measured unable to reach.**
///
/// Empty for every document that has never been zoomed past a rasterizer
/// limit, which is every document the operator opens at a reading zoom. The
/// map only ever gains the handful of pages he has driven to the wall.
///
/// `BTreeMap` rather than `HashMap` so that iteration — which only diagnostics
/// do — is in page order and a trace line is reproducible between runs.
#[derive(Debug, Default, Clone)]
pub struct RasterCeiling {
    /// `page index -> (highest raster scale believed drawable, page epoch the
    /// observation belongs to)`.
    learned: BTreeMap<usize, (f32, u64)>,
}

impl RasterCeiling {
    /// **Record that `page` could not be rasterized at `refused_scale`.**
    ///
    /// Returns `Some(ceiling)` when this changed the page's ceiling, and `None`
    /// when it did not — either because the entry already stood at or below the
    /// new value, or because the inputs were not finite and positive.
    ///
    /// ★ The return value is what the caller traces and what it decides to pull
    /// the zoom back on, so that **re-learning the same wall is not an event**.
    /// A refusal can be absorbed more than once for the same key (a repaint
    /// ordering the same render after a cache eviction, a strip page becoming
    /// current), and a caller that corrected the zoom on every absorption would
    /// fight the operator's own zoom-out.
    ///
    /// `epoch` is the page's epoch at the moment of the refusal. An observation
    /// from a different epoch replaces rather than joins: the older one was
    /// about content this page no longer has.
    pub fn learn(&mut self, page: usize, refused_scale: f32, epoch: u64) -> Option<f32> {
        if !refused_scale.is_finite() || refused_scale <= 0.0 {
            return None;
        }
        let ceiling = refused_scale * BACKOFF;
        // Guard the product as well as the input: a subnormal scale times the
        // back-off can underflow to zero, and a ceiling of zero would make the
        // page unzoomable rather than bounded.
        if !ceiling.is_finite() || ceiling <= 0.0 {
            return None;
        }
        match self.learned.get(&page) {
            // Same epoch and we already knew as much or more — no event.
            Some(&(known, known_epoch)) if known_epoch == epoch && known <= ceiling => None,
            _ => {
                self.learned.insert(page, (ceiling, epoch));
                Some(ceiling)
            }
        }
    }

    /// **The highest raster scale this page is believed able to draw**, if one
    /// has been measured at the page's current epoch.
    ///
    /// `None` means *nothing is known*, which is the answer for every page of
    /// every document until a render is actually refused — and it must stay
    /// distinguishable from *no limit*, because a caller that read `None` as a
    /// number would have to invent one.
    ///
    /// # ★★ There is no `forget_all`, and that is a measurement rather than an
    /// omission
    ///
    /// Pages are identified by INDEX here, as they are in every other per-page
    /// cache in this shell, so inserting, deleting or reordering a page
    /// *renumbers* the observations — and a renumber is not an edit to the page
    /// that moved, so the obvious worry is that the epoch key cannot catch it.
    ///
    /// One was drafted for exactly that worry and then deleted, because the
    /// condition it guarded cannot arise.
    /// [`crate::app::state::pageepoch::PageEpochs::bump_all`] is called by
    /// `crate::app::actions::pages`' `resync` under its `renumbered` flag — and
    /// `renumbered` is computed as *the page-object identity sequence changed*,
    /// which is precisely insert, delete and reorder and precisely not a
    /// content edit or a rotation. `bump_all` moves the document-wide floor,
    /// and [`crate::app::state::pageepoch::PageEpochs::get`] maxes that floor
    /// into *every* page's number, so a renumber retires every learned ceiling
    /// through the same one mechanism an ordinary edit does. A second clearing
    /// path would have been a second rule to keep in step with the first, for
    /// no behaviour.
    ///
    /// ★ **And if that ever stops being true, the failure is bounded and
    /// self-healing** — which is why it is safe to depend on. A stale entry can
    /// only cap one sheet's zoom at a number measured on a different sheet; the
    /// canvas still draws, nothing is lost, and the first refusal at the new
    /// number re-learns it correctly (`learn` ratchets *down* within an epoch
    /// and replaces outright across one). Compare the alternative failure, a
    /// ceiling that was cleared when it should not have been: the operator
    /// meets the wall again, which is the whole of what O186 asked us to stop.
    #[must_use]
    pub fn for_page(&self, page: usize, epoch: u64) -> Option<f32> {
        self.learned
            .get(&page)
            .filter(|&&(_, known_epoch)| known_epoch == epoch)
            .map(|&(ceiling, _)| ceiling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nothing is known until something is refused.
    ///
    /// The load-bearing half of this is the `None`: every document the operator
    /// opens is in this state, and a ceiling that defaulted to a number would
    /// cap a zoom no measurement has anything to say about.
    #[test]
    fn a_page_that_has_never_refused_has_no_learned_ceiling() {
        let ceiling = RasterCeiling::default();
        assert_eq!(ceiling.for_page(0, 0), None);
        assert_eq!(ceiling.for_page(7, 3), None);
    }

    /// One refusal becomes a ceiling below the scale that refused.
    #[test]
    fn a_refusal_is_learned_below_the_scale_that_refused() {
        let mut ceiling = RasterCeiling::default();
        let learned = ceiling
            .learn(4, 284_964.0, 0)
            .expect("a first refusal is an event");
        assert!(
            learned < 284_964.0,
            "a ceiling AT the failing scale is a ceiling the page cannot draw at: {learned}"
        );
        assert_eq!(ceiling.for_page(4, 0), Some(learned));
        // And it says nothing about any other page — the measured spread
        // between an E-size sheet and a business card is 28x.
        assert_eq!(ceiling.for_page(5, 0), None);
    }

    /// ★★★ The ratchet: a second refusal lowers the ceiling, a repeat does not
    /// move it, and a *higher* refusal does not raise it.
    ///
    /// All three in one test because they are one property — `learn` keeps the
    /// minimum — and separating them would let a change satisfy two and break
    /// the third.
    ///
    /// The third clause is the one most likely to be got wrong and the most
    /// consequential: a page can be refused at a *larger* scale than one
    /// already learned (a render ordered before the ceiling took effect, or a
    /// stale request landing late), and raising the ceiling on that evidence
    /// would undo the correction the operator is standing at.
    #[test]
    fn the_ceiling_only_ever_ratchets_down() {
        let mut ceiling = RasterCeiling::default();
        let first = ceiling.learn(1, 1_000_000.0, 0).expect("first is an event");
        let second = ceiling
            .learn(1, 400_000.0, 0)
            .expect("a lower wall is an event");
        assert!(second < first, "{second} must be below {first}");
        assert_eq!(
            ceiling.learn(1, 400_000.0, 0),
            None,
            "re-learning the same wall is not an event, or the zoom correction fights the operator"
        );
        assert_eq!(
            ceiling.learn(1, 9_000_000.0, 0),
            None,
            "a refusal at a HIGHER scale must not raise a ceiling already learned"
        );
        assert_eq!(ceiling.for_page(1, 0), Some(second));
    }

    /// An edit to the page retires what was learned about it.
    ///
    /// Deleting the one enormous path raises the wall, so an observation from
    /// before the edit is a magnification limit the document no longer has.
    #[test]
    fn an_edit_to_the_page_retires_its_ceiling() {
        let mut ceiling = RasterCeiling::default();
        ceiling.learn(2, 500_000.0, 0).expect("first is an event");
        assert!(ceiling.for_page(2, 0).is_some());
        assert_eq!(
            ceiling.for_page(2, 1),
            None,
            "an edit bumps the page's epoch, and the observation was about the old content"
        );
        // …and the next refusal at the new epoch is an event again, even at a
        // scale the old entry already covered.
        assert!(ceiling.learn(2, 500_000.0, 1).is_some());
    }

    /// Nonsense in, nothing learned.
    ///
    /// A non-finite or non-positive scale cannot come from a real render, but a
    /// ceiling of zero or NaN would make the page unzoomable rather than
    /// bounded — a far worse failure than the one being guarded against, and
    /// one that would look like the document being broken.
    ///
    /// ★★ **A plausibly SMALL scale is deliberately not rejected here**, and the
    /// reason is worth stating because it looks like a hole. A raster scale
    /// below 1.0 is an ordinary render of a page zoomed out below 100 %, so a
    /// floor in this function would reject real measurements. The case that
    /// would be catastrophic — `BadRasterSize` firing because the pixmap is
    /// *empty* rather than too large, which happens at a tiny scale and would
    /// pin the zoom near zero — is excluded one layer up, where the width and
    /// height are actually known: see `crate::render::worker`'s
    /// `BadRasterSize` arm, which categorises an empty pixmap as
    /// `RefusalKind::Other` precisely so it can never arrive here.
    #[test]
    fn a_degenerate_scale_teaches_nothing() {
        let mut ceiling = RasterCeiling::default();
        for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                ceiling.learn(0, scale, 0),
                None,
                "a scale of {scale} must teach nothing"
            );
        }
        assert_eq!(ceiling.for_page(0, 0), None);
    }

    /// ★ **A page is known INDEPENDENTLY of its neighbours**, which is the
    /// property that makes a per-page map the right shape rather than a single
    /// document-wide number.
    ///
    /// Stated as a test because the alternative is cheap, tempting and wrong:
    /// one ceiling for the document would be correct only if every page hit the
    /// rasterizer's wall at the same scale, and the engine's own measurement
    /// says two pages of one file differ by a factor of 28. A document-wide
    /// ceiling learned from the E-size sheet would cap the business card at
    /// 3.5 % of where it can actually be drawn.
    #[test]
    fn a_refusal_on_one_page_says_nothing_about_another() {
        let mut ceiling = RasterCeiling::default();
        ceiling.learn(0, 284_964.0, 0);
        assert!(ceiling.for_page(0, 0).is_some());
        assert_eq!(
            ceiling.for_page(9, 0),
            None,
            "page 9 has refused nothing and must be uncapped"
        );
    }
}
