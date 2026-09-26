//! # `render::ceiling` — the zoom ceiling this document *taught* the shell
//!
//!
//! > *"If this error is caused by some other limitation that will always
//! > happen, zoom should stop at the limit and not end up showing an error —
//! > the canvas will just stop zooming in and can still function. The error can
//! > still be shown on the bottom bar so the user has some idea as to why
//! > zooming stopped short of 1 trillion percent."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/ceiling.md`.

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
    /// The return value is what the caller traces and what it decides to pull
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
    /// # There is no `forget_all`, and that is a measurement rather than an
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
    /// **And if that ever stops being true, the failure is bounded and
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

    /// The ratchet: a second refusal lowers the ceiling, a repeat does not
    /// move it, and a *higher* refusal does not raise it.
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

    /// **A page is known INDEPENDENTLY of its neighbours**, which is the
    /// property that makes a per-page map the right shape rather than a single
    /// document-wide number.
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
