//! # `stripschedule::prefetch` — filling in the pages he has not scrolled to yet
//!
//! `OPERATOR_REQUESTS.md` O201:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/stripschedule/prefetch.md`.

use crate::opendoc::OpenDoc;
use crate::stripschedule::StripOrders;

/// **How far past the visible band the strip may fill in, in pages** — O201.
const PREFETCH_BAND: usize = 8;

/// The [`crate::diag::trace_changed`] slot for **what render-ahead is holding
/// and what it has left to spend** — O201's off-canvas disclosure.
const PREFETCH_SLOT: &str = "strip-prefetch";

/// **The band, in the order it must be offered** — nearest the current page
/// first, and forward before backward at equal distance.
#[must_use]
fn prefetch_ranking(current: usize, last: usize, band: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(band * 2);
    for step in 1..=band {
        if let Some(ahead) = current.checked_add(step)
            && ahead <= last
        {
            out.push(ahead);
        }
        if let Some(behind) = current.checked_sub(step) {
            out.push(behind);
        }
    }
    out
}

/// Render-ahead: whether and which page to rasterise beyond the viewport.
pub trait Prefetch {
    #[must_use]
    fn prefetch_headroom(&self) -> bool;
    #[must_use]
    fn prefetch_candidate(&self, visible: &[usize], raster_scale: f32) -> Option<usize>;
}

impl Prefetch for OpenDoc {
    /// **Is there room for one more page picture?** — O201's whole memory
    /// bound, and the reason render-ahead cannot become a busy loop.
    fn prefetch_headroom(&self) -> bool {
        let budget = self.prefs.page_cache.texels();
        let resident = self.strip_rasters.texels();
        let pages = self.strip_rasters.len() as u64;
        let per_page = resident.checked_div(pages).unwrap_or(0);
        resident + per_page <= budget
    }

    /// **The nearest page outside the viewport that has no picture yet** —
    /// O201's candidate, or `None` when there is nothing worth asking for.
    fn prefetch_candidate(&self, visible: &[usize], raster_scale: f32) -> Option<usize> {
        if !self.prefetch_headroom() {
            return None;
        }
        let last = self.pages.len().checked_sub(1)?;
        prefetch_ranking(self.view.page_index, last, PREFETCH_BAND)
            .into_iter()
            .filter(|page| !visible.contains(page))
            .filter(|&page| self.strip_page_orderable(page, raster_scale))
            .find(|&page| {
                !self.strip_rasters.has(
                    page,
                    self.render_key_for(page, raster_scale),
                    self.page_epochs.get(page),
                )
            })
    }
}

/// **Say what render-ahead is holding, and what is left to spend.**
pub fn disclose(doc: &OpenDoc) {
    let band = doc.strip_rasters.len();
    let texels = doc.strip_rasters.texels();
    let budget = doc.prefs.page_cache.texels();
    crate::diag::trace_changed(PREFETCH_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("strip-prefetch band={band} texels={texels} budget={budget}")
    });
}

#[cfg(test)]
mod tests {
    //! The band's ORDER, which is the one thing here that fails silently.
    //!
    //! A wrong order does not render the wrong page — every page it offers is
    //! a real page, and every one of them will eventually be wanted. It grinds
    //! instead: `StripRasters::retain` evicts furthest-first, so an order that
    //! is not nearest-first asks for the page eviction is about to drop, and
    //! the two alternate for as long as he scrolls. Nothing on screen says so.
    //!
    //! `prefetch_ranking` is pure, so the whole of that property is testable
    //! without a document, a cache, or a frame.

    use super::*;

    /// The distance from `current` never decreases along the list.
    fn is_nearest_first(current: usize, band: &[usize]) -> bool {
        band.windows(2)
            .all(|w| w[0].abs_diff(current) <= w[1].abs_diff(current))
    }

    /// **The reverse of the eviction order**, in the middle of a document
    /// where nothing is clamped.
    #[test]
    fn the_band_is_offered_nearest_page_first() {
        let band = prefetch_ranking(50, 99, 8);
        assert!(is_nearest_first(50, &band), "{band:?}");
        assert_eq!(band.len(), 16, "eight ahead and eight behind");
        assert_eq!(&band[..4], &[51, 49, 52, 48], "forward wins each tie");
    }

    /// Near the end of the document the forward half runs out, and what is
    /// left is the backward half — not a shorter band, and not page indices
    /// past the last page.
    #[test]
    fn the_band_is_clamped_to_the_last_page() {
        let band = prefetch_ranking(97, 99, 8);
        assert!(
            band.iter().all(|&p| p <= 99),
            "no page past the end is ever offered: {band:?}"
        );
        assert!(band.contains(&98) && band.contains(&99));
        assert!(is_nearest_first(97, &band), "{band:?}");
    }

    /// And at the front of the document the backward half runs out without
    /// wrapping to the end of the file.
    #[test]
    fn the_band_does_not_wrap_past_the_first_page() {
        let band = prefetch_ranking(1, 99, 8);
        assert_eq!(band.iter().filter(|&&p| p < 1).count(), 1, "{band:?}");
        assert!(band.contains(&0));
        assert!(is_nearest_first(1, &band), "{band:?}");
    }

    /// A document shorter than the band offers every other page once and
    /// nothing else.
    #[test]
    fn a_short_document_offers_its_real_pages_only() {
        let mut band = prefetch_ranking(1, 2, 8);
        assert_eq!(band.len(), 2, "three pages, one of them current: {band:?}");
        band.sort_unstable();
        assert_eq!(band, vec![0, 2]);
    }

    /// A one-page document offers nothing at all, rather than page 0 twice or
    /// a page that does not exist.
    #[test]
    fn a_single_page_document_has_no_band() {
        assert!(prefetch_ranking(0, 0, 8).is_empty());
    }

    /// The current page is never in its own band.
    ///
    /// It has its own texture slot and its own scan; offering it here would
    /// re-render the page he is reading behind his back.
    #[test]
    fn the_current_page_is_not_a_candidate() {
        for current in [0, 1, 50, 99] {
            let band = prefetch_ranking(current, 99, 8);
            assert!(!band.contains(&current), "current={current} {band:?}");
        }
    }

    /// No page is offered twice, so the caller's first-miss scan cannot spend
    /// two frames deciding about the same sheet.
    #[test]
    fn no_page_appears_twice() {
        let band = prefetch_ranking(50, 99, 8);
        let mut sorted = band.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), band.len(), "{band:?}");
    }
}
