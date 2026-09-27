//! Which pages the page strip may order, at what raster scale, and what it holds
//! for each; how long a zoom must settle; and render-ahead in [`prefetch`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/stripschedule.md`.

use std::time::Duration;

use crate::opendoc::OpenDoc;
use crate::raster::PageTexture;
use crate::renderstrip::{PageRaster, PageState};
use crate::renderworker::RenderKey;

/// Render-ahead: which page outside the viewport to fill next, and what bounds it.
pub mod prefetch;

/// The committed-zoom delay this document's preferences set.
pub trait ZoomSettle {
    fn zoom_settle(&self) -> Duration;
}

impl ZoomSettle for OpenDoc {
    /// How long this document's zoom must stop changing before it is committed.
    fn zoom_settle(&self) -> Duration {
        Duration::from_millis(self.prefs.zoom_settle_ms)
    }
}

/// Which strip pages can be ordered at a raster scale, and what the strip holds for each.
pub trait StripOrders {
    fn rehome_current_page(&mut self, wanted: RenderKey);
    #[must_use]
    fn strip_page_orderable(&self, page: usize, raster_scale: f32) -> bool;
    #[must_use]
    fn raster_order_fillable(&self, page: usize, raster_scale: f32) -> bool;
    #[must_use]
    fn strip_page_state(&self, page: usize, key: RenderKey) -> Option<PageState>;
    #[must_use]
    fn strip_page_texture(&self, page: usize, key: RenderKey) -> Option<&PageTexture>;
}

impl StripOrders for OpenDoc {
    /// **Move the current page's texture into the strip, and the incoming
    /// page's out of it.**
    fn rehome_current_page(&mut self, wanted: RenderKey) {
        let holding = self.page_texture.as_ref().map(|t| t.key.page());
        if holding == Some(self.view.page_index) {
            return;
        }
        if let Some(outgoing) = self.page_texture.take() {
            let (page, key) = (outgoing.key.page(), outgoing.key);
            self.strip_rasters.insert(
                page,
                key,
                self.edit_epoch,
                PageRaster::Ready(Box::new(outgoing)),
            );
        }
        // A page-scoped refusal follows the page it is about, so scrolling on
        // to a page that would not draw states the reason immediately rather
        // than re-attempting the render that already failed.
        match self.strip_rasters.take(
            self.view.page_index,
            wanted,
            // Per-page (O74).
            self.page_epochs.get(self.view.page_index),
        ) {
            Some(PageRaster::Ready(texture)) => {
                self.page_texture = Some(*texture);
                self.render_error = None;
                self.render_refused = None;
            }
            Some(PageRaster::Failed(message)) => {
                self.page_texture = None;
                self.render_error = Some(message);
                // The refusal was filed under a key, and `take` only returned
                // it because that key is `wanted`. Adopting it as the memo is
                // what stops the page being re-asked the instant it becomes
                // current -- without this, arriving at a page that already
                // refused costs one more dead render before the hold takes.
                self.render_refused = Some((wanted, self.page_epochs.get(self.view.page_index)));
            }
            // Nothing cached for the incoming page. That clears the OUTGOING
            // page's sentence -- which is the whole reason this arm assigns at
            // all -- but it must not clear a sentence about the page now
            // current. A refusal nulls the texture, so this function stops
            // early-returning and runs every frame afterwards; assigning
            // `None` unconditionally is what erased the disclosure one frame
            // after it was set, and with it the spawn gate's only evidence.
            None => {
                if !self
                    .render_refused
                    .is_some_and(|(key, _)| key.page() == self.view.page_index)
                {
                    self.render_error = None;
                }
            }
        }
    }

    /// **Can this strip page be ordered at all at this raster scale?** —
    /// O186, 2026-09-12.
    fn strip_page_orderable(&self, page: usize, raster_scale: f32) -> bool {
        self.pages.get(page).is_some_and(|p| {
            crate::rasterstrategy::whole_page_raster_fits(
                crate::viewer::page_extent_pts(p),
                raster_scale,
            )
        })
    }

    /// **Is there anything a worker could actually fill for this page at
    /// this raster scale?** — O186's THIRD route, measured 2026-09-12.
    ///
    /// [`Self::strip_page_orderable`] answers this for a page that is handed no
    /// region. This answers it for any page, including the current one, by
    /// asking the extra question the current page brings with it: **it may have
    /// a region, and if it has one the sheet's size stops mattering.**
    ///
    /// ```text
    /// region present  ->  fillable at any scale (the request is viewport-sized)
    /// region absent   ->  fillable only while the WHOLE SHEET still fits
    /// ```
    ///
    /// # The measurement that made this necessary
    ///
    /// `ui-verify`'s `the_strip_never_orders_a_raster_it_cannot_fill` climbed a
    /// continuous strip and reported, at a zoom of 438:
    ///
    /// ```text
    /// canvas-unavailable reason=nothing-visible
    /// render-spawn gen=52 page=0 scale=438.84824
    /// bad-raster-size px=1046187x738924 page=0 scale=438.8 region=0
    /// raster-ceiling-learned page=0 scale=329.1 zoom=438.85 to=329.14 moved=true
    /// ```
    ///
    /// Read that in order. **Nothing was on screen** — `canvas::deep`'s anchor
    /// is not clamped, so the view had been carried off the sheet (O186 stage
    /// one, a separate defect and not this one). With no visible part of the
    /// page, `canvas::tier::decide`'s region tier has nothing to intersect and
    /// leaves `OpenDoc::raster_region` as `None`. The *same* frame then asked
    /// for a raster, and a request with no region is a request for the whole
    /// sheet — 1,046,187 × 738,924 device pixels of it.
    ///
    /// **And the damage is not the refusal, it is the ceiling learned from
    /// it.** `absorb`'s `absorb_render` turns a refusal into a zoom ceiling, so a page that
    /// renders perfectly through the region tier at ten billion percent —
    /// measured, on `fixtures/four-pages.pdf` — had its zoom capped at 329×
    /// because of one order that should never have been placed. The operator's
    /// O186: *"the canvas will just stop zooming in"*. It did, at a number that
    /// was an internal mistake rather than a limit of anything.
    ///
    /// # ⚠ Why declining costs nothing
    ///
    /// The only way to reach `region_for() == None` while the whole sheet will
    /// not fit is for **no part of the page to be on screen** — either it has
    /// been carried off, or the canvas was not laid out this frame. There are no
    /// pixels to show either way, so there is no picture being withheld. This is
    /// not a clamp and it hides nothing; a clamp would be
    /// [`crate::rasterceiling`], which still holds for real refusals.
    ///
    /// # What it deliberately does NOT ask
    ///
    /// `strategy::for_page`, for the reason [`Self::strip_page_orderable`] gives
    /// at length: that is the union of this hard pixmap limit with the soft ink
    /// one, and an ink page above the CMYK buffer ceiling answers `Region` from
    /// it while its whole-page raster allocates perfectly well. Asking the union
    /// here would decline orders that would have succeeded.
    /// # ⚠⚠ DRIVEN COVERAGE IS OWED, AND NOTHING HERE IS EVIDENCE YET
    ///
    ///
    /// Why neither check reaches it, which is the useful half:
    ///
    ///   * `the_strip_never_orders_a_raster_it_cannot_fill` now stops at zoom
    ///     ~22. That is deliberate — its measuring window closes at ~35 — and it
    ///     is four hundred times too shallow. The route this guard closes needs
    ///     the view to have been carried off the sheet, which on
    ///     `fixtures/four-pages.pdf` happens near zoom 440.
    ///   * `the_raster_wall_stops_the_zoom_instead_of_painting_an_error` climbs
    ///     to 36 million per cent and does reach a frame where this returns
    ///     `false` — `current-order-unfillable page=0 fillable=false` appears in
    ///     its trace — but on that frame nothing was stale, so the spawn below
    ///     would not have happened even unguarded. **Reaching the predicate is
    ///     not reaching the defect**, and a check that confused the two would
    ///     report coverage it does not have.
    ///
    /// **The check that will falsify this is O186 stage one's.** That stage
    /// fixes the unclamped `canvas::deep` anchor, and its driven check has to
    /// reproduce the terminal `canvas-unavailable reason=nothing-visible` first
    /// in order to assert it is gone. That reproduction IS this guard's
    /// falsification: with the guard removed, the same blank frame orders a whole
    /// sheet and `raster-ceiling-learned … to=329.14` follows. So the assertion
    /// to add there is not *"the canvas is not blank"* alone but also **"no
    /// ceiling was learned while it was"**.
    ///
    /// Until that lands, the evidence for this guard is: the dated trace quoted
    /// above, the unit test below, and the `current-order-unfillable` transitions
    /// that prove the predicate is live in a real build. That is weaker than this
    /// project's bar and is not being described as meeting it.
    ///
    fn raster_order_fillable(&self, page: usize, raster_scale: f32) -> bool {
        // `region_for` and not `raster_region`: it is the one that checks the
        // region belongs to THIS page. A region computed for page 4 does not
        // make page 5's order fillable, and both rectangles are valid, so the
        // mistake would be silent. The same argument is on `region_for` itself.
        self.region_for(page).map_or_else(
            || self.strip_page_orderable(page, raster_scale),
            // A REGION IS NOT AUTOMATICALLY SMALL. See
            // `strategy::region_raster_fits` for the two rectangles that arrive
            // here wearing one type: the visible-rect tier's box is a multiple
            // of the window and so the same size at every zoom, while the
            // off-page halo box is fixed in the PAGE's space and therefore
            // grows with the zoom exactly as the whole sheet does — and being
            // the bigger rectangle, it reaches the wall first.
            //
            // The scale this asks at is the one the order will be RENDERED at.
            // The canvas chose the region a frame earlier, at the scale that
            // frame was drawn at, so on any frame the zoom steps up the region
            // in hand was validated against a smaller number than the one about
            // to be used. That one-frame skew is the whole defect: on the
            // operator's site plan the halo box was picked at scale 6 and
            // ordered at scale 8, came back `BadRasterSize` at 19073 × 13471,
            // and `absorb_render` turned the refusal into the document's zoom
            // ceiling — pinning a drawing that reaches 39,321,600 % at 600 %.
            //
            // ⚠ Declining withholds nothing and does not persist. The canvas
            // re-decides every frame, so the next frame asks `halo::region` at
            // the new scale, is told `None`, falls back to the whole sheet, and
            // places an order that fills. The cost is one frame of the previous
            // texture drawn soft — which is the staleness signal the canvas is
            // built on — against a permanent ceiling learned from an order this
            // shell should never have placed.
            |region| crate::rasterstrategy::region_raster_fits(region, raster_scale),
        )
    }

    /// What state a **strip** page is in, for
    /// [`crate::renderstrip::draw_page_state`].
    fn strip_page_state(&self, page: usize, key: RenderKey) -> Option<PageState> {
        // Per-page (O74): a page whose own revision has not moved keeps its
        // raster through an edit made on another sheet.
        match self
            .strip_rasters
            .get(page, key, self.page_epochs.get(page))
        {
            Some(PageRaster::Ready(_)) => None,
            Some(PageRaster::Failed(detail)) => Some(PageState::Refused(detail.clone())),
            // Asked FIRST among the empty cases, and from the key's own
            // scale rather than from a second reading of the view — see
            // [`Self::strip_page_orderable`]. `Drawing` cannot be true here
            // after `fill_strip` stopped ordering these pages, and `Waiting`
            // would be a promise this zoom cannot keep.
            //
            // A `Ready` or `Failed` entry still wins above, which is right: the
            // key carries the scale, so an entry that matches this frame's key
            // is an entry made at a scale that fit. A page cached at a shallower
            // zoom does not match and falls through to here.
            None if !self.strip_page_orderable(page, key.raster_scale()) => {
                Some(PageState::BeyondRaster)
            }
            None if self.render_worker.rendering_key().map(|k| k.page()) == Some(page) => {
                Some(PageState::Drawing)
            }
            None => Some(PageState::Waiting),
        }
    }

    /// The texture for a **strip** page, if there is a current one.
    fn strip_page_texture(&self, page: usize, key: RenderKey) -> Option<&PageTexture> {
        // Per-page (O74).
        match self
            .strip_rasters
            .get(page, key, self.page_epochs.get(page))
        {
            Some(PageRaster::Ready(texture)) => Some(texture),
            _ => None,
        }
    }
}
