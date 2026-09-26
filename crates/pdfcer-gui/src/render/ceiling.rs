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
#[derive(Debug, Default, Clone)]
pub struct RasterCeiling {
    /// `page index -> (highest raster scale believed drawable, page epoch the
    /// observation belongs to)`.
    learned: BTreeMap<usize, (f32, u64)>,
}

impl RasterCeiling {
    /// **Record that `page` could not be rasterized at `refused_scale`.**
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
