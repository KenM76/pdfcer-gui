//! # `render::prefetch` — filling in the pages he has not scrolled to yet
//!
//! `OPERATOR_REQUESTS.md` O201:
//!
//! > *"on multipage documents I noticed with scanned pdf I have to wait for
//! > pages to load as a I scroll to them. As many pages as we can should be
//! > rendered and ready to be shown as I scroll. The ones on screen should
//! > always take precedence to be rendered first."*
//!
//! ## Contract
//!
//! [`OpenDoc::prefetch_candidate`] answers *"which page would render-ahead
//! like next, if anything"*, and is consulted by `settle::fill_strip` **only
//! when every visible page already has a picture**. [`disclose`] emits the
//! off-canvas report the feature owes. Nothing here spawns anything: the
//! caller owns the worker, so the single-slot rule and the zoom-settle
//! suppression stay in one place.
//!
//! ## Precedence is structural
//!
//! His second sentence is not implemented as a priority number. The band is
//! reached only down the `None` arm of the visible scan, so there is no
//! ordering in which a page he can see loses to a page he cannot. A priority
//! comparison would have to be correct at every call site; an unreachable
//! branch is correct by construction.
//!
//! ## What bounds it
//!
//! Three things, and they are independent on purpose:
//!
//! * **[`PREFETCH_BAND`]** bounds how far a wrong guess about his direction
//!   can run, in pages.
//! * **[`OpenDoc::prefetch_headroom`]** bounds the memory, in texels, against
//!   the operator's own page-cache preference.
//! * **`OpenDoc::strip_page_orderable`** is applied unchanged, so a sheet
//!   above the renderer's pixmap ceiling is no more orderable ahead of time
//!   than it is on arrival.

use crate::app::state::OpenDoc;

/// **How far past the visible band the strip may fill in, in pages** — O201.
///
/// > *"on multipage documents I noticed with scanned pdf I have to wait for
/// > pages to load as a I scroll to them. As many pages as we can should be
/// > rendered and ready to be shown as I scroll."*
///
/// Eight is a bound on how far a *wrong* guess can run, not a target. The
/// band is symmetric about the current page (see [`prefetch_ranking`]), so
/// half of it is behind him, and behind him is where the rasters he has
/// already scrolled past are still resident. What eight really buys is eight
/// pages of lead on the direction he is actually going.
///
/// A count of PAGES rather than of texels because his request is in pages.
/// The memory is bounded separately and absolutely, by
/// [`OpenDoc::prefetch_headroom`].
const PREFETCH_BAND: usize = 8;

/// The [`crate::diag::trace_changed`] slot for **what render-ahead is holding
/// and what it has left to spend** — O201's off-canvas disclosure.
///
/// A prefetched page renders exactly as a page he scrolled to: same request,
/// same raster, no marking of any kind. Rule 4 is satisfied by that, and what
/// it asks for in return is that the inference be **reported somewhere else**.
/// This is that report. It carries the resident band count and the texels
/// against the budget, because the one surprise render-ahead can spring is a
/// page-cache preference that used to sit idle and now fills — the
/// connection between *"pages are ready when I get to them"* and *"this
/// program is holding a gigabyte"* has to be visible rather than inferred.
///
/// Its own slot, per the rule at [`BEYOND_RASTER_SLOT`]: `trace_changed`
/// de-duplicates per slot, so two lines sharing one appear only when they
/// happen to alternate. `band=0` is printed too, on the way out of the regime
/// as well as into it, so a driven run can tell *"never prefetched anything"*
/// from *"prefetched and then stopped"*.
const PREFETCH_SLOT: &str = "strip-prefetch";

/// **The band, in the order it must be offered** — nearest the current page
/// first, and forward before backward at equal distance.
///
/// Pure, and separated from the cache and the page tree on purpose: the order
/// is the one thing here that can be silently wrong, and a wrong order does not
/// fail — it grinds. The tests at the foot of this file pin it.
///
/// # Why nearest-first is not a preference
///
/// [`crate::render::strip::StripRasters::retain`] evicts the entry **furthest**
/// from the current page. A prefetch offered in any other order therefore asks
/// for the page that eviction most wants to drop, and the two mechanisms
/// alternate — render, evict, render — for as long as he keeps scrolling. The
/// prefetch order has to be the exact reverse of the eviction order or the
/// feature is a busy loop that also costs memory.
///
/// # Why forward wins a tie, when nothing here measures a direction
///
/// Because the pages behind him are the ones already rendered. He arrived at
/// the current page by scrolling through them, so `current - 1` is nearly
/// always resident and `current + 1` nearly always is not — which means the
/// symmetric band spends almost all of its budget forwards without being told
/// to, and the tie-break only decides the first page of a document opened in
/// the middle. Measuring the scroll direction to reach the same result would
/// be two fields of state answering a question the cache already answers.
///
/// `last` is the highest page index in the document, so the band is clamped to
/// it rather than offering a page [`OpenDoc::rasterize`] would discard.
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

impl OpenDoc {
    /// **Is there room for one more page picture?** — O201's whole memory
    /// bound, and the reason render-ahead cannot become a busy loop.
    ///
    /// `true` when the strip cache could take another page the size of the
    /// ones it is already holding without crossing
    /// [`crate::app::prefs::PageCache::texels`].
    ///
    /// # Why the estimate, rather than simply *"under budget"*
    ///
    /// Because *under budget* is satisfied again the instant eviction runs.
    /// [`crate::render::strip::StripRasters::retain`] trims until the total is
    /// **at or below** the budget, so a prefetch gated on `texels() < budget`
    /// fills past it, is trimmed back under it, and fills again — for ever,
    /// re-rendering one page every few frames on an idle window. Requiring room
    /// for the page about to be added is what makes the loop terminate: once it
    /// is false it stays false until something is evicted for a reason other
    /// than this.
    ///
    /// The estimate is the mean of what is resident. A scanned document — the
    /// case he reported — has pages of one size, so the mean is the answer; a
    /// mixed document gets an answer that is wrong by less than one page, which
    /// is inside what `retain` would correct on the next frame anyway.
    ///
    /// An empty cache answers `true`: there is nothing to average, and a
    /// document whose visible pages have not rendered yet is not reached here
    /// at all.
    #[must_use]
    pub(super) fn prefetch_headroom(&self) -> bool {
        let budget = self.prefs.page_cache.texels();
        let resident = self.strip_rasters.texels();
        let pages = self.strip_rasters.len() as u64;
        let per_page = resident.checked_div(pages).unwrap_or(0);
        resident + per_page <= budget
    }

    /// **The nearest page outside the viewport that has no picture yet** —
    /// O201's candidate, or `None` when there is nothing worth asking for.
    ///
    /// Every gate a visible page passes is applied here unchanged:
    /// [`Self::strip_page_orderable`] for O186's pixmap wall, the render key
    /// and the per-page epoch for staleness. The two it adds are
    /// [`Self::prefetch_headroom`] and the band itself.
    ///
    /// `visible` is excluded rather than assumed absent: the caller only
    /// reaches this after every visible page has a raster, so a visible page
    /// could not be returned anyway — but the exclusion is what makes that
    /// true by construction instead of by the caller's good behaviour.
    #[must_use]
    pub(super) fn prefetch_candidate(&self, visible: &[usize], raster_scale: f32) -> Option<usize> {
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
///
/// Rule 4's other half. A prefetched page renders exactly as a page he
/// scrolled to — same request, same raster, no badge, no tint — so the only
/// place the inference can be reported is off the canvas, and this is it.
/// What it carries is the connection between *"pages are ready when I get to
/// them"* and *"this program is holding a gigabyte"*: the resident count and
/// the texels against the budget the operator chose.
pub(super) fn disclose(doc: &OpenDoc) {
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
    ///
    /// Stated as a property rather than as a literal expected vector, because
    /// it is the property `retain` is the mirror of — a future change that
    /// reorders the band for some other good reason has to break this, not a
    /// hand-written list that could be updated to match.
    fn is_nearest_first(current: usize, band: &[usize]) -> bool {
        band.windows(2)
            .all(|w| w[0].abs_diff(current) <= w[1].abs_diff(current))
    }

    /// ★ **The reverse of the eviction order**, in the middle of a document
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
    ///
    /// Wrapping would be defensible in the abstract and is wrong here: page 0
    /// and the last page of a hundred-page scan have nothing to do with each
    /// other, and eviction — which this order mirrors — does not wrap either.
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
