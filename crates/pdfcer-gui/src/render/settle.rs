//! # `render::settle` — is the picture on screen still a picture of what I am looking at?
//!
//! The per-frame raster decision: what is stale, what to re-rasterize now,
//! what to debounce until a zoom gesture stops, and which of a continuous
//! strip's several visible pages to render next.
//!
//! ## Where the boundaries are, on both sides
//!
//! [`crate::app::state`] answers *"what is open, and what is the operator
//! looking at?"*. This module answers *"what does the picture need to be, and
//! what should be done about it this frame?"*. The two change for different
//! reasons — the first when a document gains a property, the second when
//! rendering gains a strategy — and only the second belongs in `render/`,
//! beside the worker it schedules and the texture cache it prunes.
//!
//! The child module `absorb` takes the other half of that second question:
//! not what the picture should be, but what to do with the one that came back.
//! It changes when the **renderer** gains a failure mode, which is a third
//! reason again, and it is where a raster refusal becomes a zoom ceiling.
//!
//! ## Rendering happens on state change, never per frame
//!
//! egui redraws continuously; rasterizing a PDF page at 60 Hz would be absurd.
//! Staleness is a [`RenderKey`] comparison — page, raster scale, annotation
//! visibility, layer-override generation — and there is deliberately no second
//! field list to keep in step with it: the key the worker labelled a texture
//! with is compared against the key the current view wants, and a field added
//! to the type changes both sides at once.
//!
//! **Two staleness policies apply**, split by the key's own
//! `discrete_inputs` / `scale_bits` categories, and the difference is the
//! whole of why zoom feels smooth:
//!
//! - **Discrete change — commit immediately.** A page step, an annotation
//!   toggle, a layer toggle. None has a gesture in flight and none has an
//!   intermediate value on the way to it, so any delay is pure latency; for a
//!   page change there is not even a stale texture worth showing, because it
//!   is a picture of a different page.
//! - **Zoom change — debounce by [`ZOOM_SETTLE`]**, drawing the existing
//!   texture scaled to the new size in the meantime. A Ctrl+wheel gesture
//!   emits dozens of zoom values on the way to the one the operator wants;
//!   rasterizing each would burn CPU producing images nobody sees. The interim
//!   scaled texture is soft, not blank or blocky — which is exactly what every
//!   other document viewer does, so it reads as normal rather than as a
//!   glitch. A **discrete** command (Ctrl+0, Ctrl+Plus) bypasses the debounce
//!   through `ViewFrame::zoom_commanded`: there is no gesture in flight, so
//!   waiting would just feel unresponsive.
//!
//! ## ★ The strip, and the priority that keeps it affordable
//!
//! Under a continuous mode several pages are visible at once, and
//! [`crate::render::strip`]'s header sets out the whole scheduling rule. This
//! module is where it is enforced, in one order that is a **priority** rather
//! than a sequence:
//!
//! 1. **the current page, always first.** It is the largest thing on screen
//!    and the one the operator is reading. If it is stale, the worker is
//!    pointed at it — cancelling a strip page mid-render if necessary, which
//!    is exactly what `RenderWorker::spawn` does when handed a different key.
//! 2. **rehome, before anything is requested.** Scrolling changes which page
//!    is "current" without changing any *picture*, so the outgoing page's
//!    texture is moved into the strip cache and the incoming page's is moved
//!    out of it. Without this, scrolling a continuous strip would re-render
//!    every page at the moment it became current — the pages would flash
//!    undrawn as they passed the middle of the viewport, which is the exact
//!    opposite of what a continuous mode is for.
//! 3. **then one strip page**, the nearest visible page that has no current
//!    raster. One, because the worker is single-slot: asking for two would
//!    cancel the first and deliver neither.
//!
//! Steps 2 and 3 do nothing at all when the strip is empty, which is every
//! frame of every single-page session.
//!
//! **A consequence that surprises every driven check written against step 3:**
//! the strip prefetch only has work to do after a *discontinuity*. Because step
//! 2 rehomes the outgoing current page's texture into the strip cache, a smooth
//! continuous scroll makes every page it passes current in turn, and so fills
//! the cache as it travels. The step-3 scan then never finds a visible page
//! that is both not-current and unrastered, and emits no request. A check that
//! wants to observe strip rastering must create the discontinuity deliberately
//! — set the page number, or Ctrl+End. Scrolling to a page is precisely the
//! gesture that guarantees the page is already cached.

use std::time::{Duration, Instant};

use crate::app::PdfcerApp;
use crate::app::state::{OpenDoc, Status};
use crate::render::prefetch;
use crate::render::raster::PageTexture;
use crate::render::strip::{PageRaster, PageState};
use crate::render::worker::RenderKey;
use crate::viewer;

/// The other half of the exchange this module schedules: requesting a page,
/// collecting the result, and turning whatever came back into a texture, a
/// backdrop, an ink census or a learned zoom ceiling. Declared here rather
/// than in `render/mod.rs` because nothing outside `settle` calls into it —
/// the same shape as `render/worker.rs` and its `worker/key.rs`.
mod absorb;

/// How long a zoom must stop changing before it is committed to a real
/// rasterization.
///
/// Long enough to swallow a whole wheel gesture, short enough that a
/// deliberate single step does not feel laggy. 150 ms is the value the old
/// shell settled on against real CAD sheets; it is a constant rather than a
/// literal so the next person to tune it does so once, with a paper trail.
pub const ZOOM_SETTLE: Duration = Duration::from_millis(150);

/// The [`crate::diag::trace_changed`] slot for *"the page being looked at was
/// not ordered this frame, because nothing could be made of the order"* —
/// O186's third route.
///
/// Its own slot for the same reason [`BEYOND_RASTER_SLOT`] has one: the
/// de-duplication is per slot, so two lines sharing one suppress each other and
/// each appears only when the two happen to alternate.
///
/// ★★ The line is emitted on the way OUT of the regime as well as into it, and
/// that is not decoration. The subject is an absence — no order, no refusal, no
/// learned ceiling — and a run that cannot tell *"never entered it"* from
/// *"still in it"* cannot assert an absence at all. Same argument as
/// `strip-beyond-raster pages=0`.
const CURRENT_UNFILLABLE_SLOT: &str = "current-order-unfillable";

/// The [`crate::diag::trace_changed`] slot for *"how many visible neighbour
/// sheets could not be ordered at this zoom"* — O186.
///
/// Its own slot, not shared with any other line in this module, because
/// `trace_changed` de-duplicates **per slot**: sharing one would make each line
/// suppress the other and the count would appear only when it happened to
/// alternate. See `canvas::trace`'s slot table for the same rule stated once for
/// the canvas.
const BEYOND_RASTER_SLOT: &str = "strip-beyond-raster";

impl OpenDoc {
    /// How long this document's zoom must stop changing before it is committed.
    ///
    /// ★ **The operator's, as of 2026-08-17.** [`ZOOM_SETTLE`] was the whole
    /// answer and is now only the *default* — `manifest::DIRECTED` carried this
    /// as *"partial G — `ZOOM_SETTLE` is a compiled-in constant today"*, and
    /// that was accurate: the control was missing, not the value.
    ///
    /// Read from the document's preferences **snapshot** rather than from the
    /// application, for the same reason its settings snapshot exists: this is a
    /// per-frame read inside a `&mut doc` borrow, and reaching back to
    /// `PdfcerApp` would be a second borrow of the whole struct.
    ///
    /// The snapshot cannot be meaningfully stale here — `adopt_settings` writes
    /// it and drops every raster in the same statement, so a settle read after
    /// a change is a settle for a cache that no longer exists.
    fn zoom_settle(&self) -> Duration {
        Duration::from_millis(self.prefs.zoom_settle_ms)
    }
}

impl OpenDoc {
    /// ★ **Move the current page's texture into the strip, and the incoming
    /// page's out of it.**
    ///
    /// Called when the scroll position has made a different page current. See
    /// the module header, step 2: without this, every page of a continuous
    /// strip would re-render at the moment it passed the middle of the
    /// viewport — visibly flashing undrawn on the way through, which is the
    /// opposite of what a continuous mode exists for.
    ///
    /// `wanted` is the key the *current* page needs this frame. The incoming
    /// page is taken out of the cache only if its raster matches that key,
    /// because a raster at a stale zoom is not a raster the current page can
    /// use — leaving it in the cache costs nothing and the ordinary staleness
    /// path re-renders it.
    ///
    /// The outgoing texture is filed at `self.edit_epoch`, and that is exact
    /// rather than approximate: the current page's slot is cleared outright by
    /// every edit (`crate::app::actions`' `vector_edit` and
    /// `crate::panels::forms::edit` both assign `page_texture = None`), so a
    /// texture that is still here has not survived an edit.
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
            // ★ Per-page (O74).
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
            // ★ Nothing cached for the incoming page. That clears the OUTGOING
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

    /// ★★★ **Can this strip page be ordered at all at this raster scale?** —
    /// O186, 2026-09-12.
    ///
    /// A strip page is always handed `region: None`: `OpenDoc::region_for`
    /// refuses a region for any page but the current one, deliberately, because
    /// a region is expressed in one page's own coordinate space and applying
    /// page 4's rectangle to page 5 would rasterize the wrong part of the
    /// neighbour with nothing reporting an error. So for a strip page the
    /// renderer's whole-sheet pixmap ceiling is not a *tier boundary* — it is a
    /// wall, and above it there is nothing to ask for.
    ///
    /// # Why this exists as one function rather than two conditions
    ///
    /// Three callers need the same answer and they must never disagree:
    ///
    /// * [`Self::fill_strip`] uses it to **not place an order** it knows cannot
    ///   be filled — the fix for the operator's
    ///   `requested raster size 50411508x32619210` (see
    ///   [`crate::render::strategy::whole_page_raster_fits`] for the full
    ///   measurement, including why the failing sheet was never the one he was
    ///   looking at);
    /// * [`Self::strip_page_state`] uses it to say the **true** thing about the
    ///   resulting empty page. If only the first caller existed, the page would
    ///   report itself as `Waiting` — *"not drawn yet"* — for a picture that is
    ///   never coming at this zoom. That is the wrong-refusal-sentence class of
    ///   defect: the sentence is read as an answered question and nobody
    ///   investigates.
    /// * `render::prefetch` applies it to every band candidate, so a sheet
    ///   that cannot be ordered on arrival is not ordered ahead of time
    ///   either. Without it render-ahead would spend its whole budget
    ///   re-offering the same unorderable A1 every frame.
    ///
    /// # What it deliberately does NOT ask
    ///
    /// `strategy::for_page`. That is the union of this hard limit and the soft
    /// ink one, and an ink page above the CMYK buffer ceiling answers `Region`
    /// from it while its whole-page raster allocates perfectly well — so asking
    /// the union here would leave a neighbour sheet blank at an ordinary zoom
    /// to avoid a failure that was never going to happen.
    ///
    /// A page index past the end answers `false`: there is no sheet to order,
    /// and [`Self::rasterize`] would discard the request anyway.
    #[must_use]
    pub fn strip_page_orderable(&self, page: usize, raster_scale: f32) -> bool {
        self.pages.get(page).is_some_and(|p| {
            crate::render::strategy::whole_page_raster_fits(
                viewer::page_extent_pts(p),
                raster_scale,
            )
        })
    }

    /// ★★★ **Is there anything a worker could actually fill for this page at
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
    /// ★★ **And the damage is not the refusal, it is the ceiling learned from
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
    /// [`crate::render::ceiling`], which still holds for real refusals.
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
    /// ★ **The check that will falsify this is O186 stage one's.** That stage
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
    #[must_use]
    pub fn raster_order_fillable(&self, page: usize, raster_scale: f32) -> bool {
        // ★ `region_for` and not `raster_region`: it is the one that checks the
        // region belongs to THIS page. A region computed for page 4 does not
        // make page 5's order fillable, and both rectangles are valid, so the
        // mistake would be silent. The same argument is on `region_for` itself.
        self.region_for(page).map_or_else(
            || self.strip_page_orderable(page, raster_scale),
            // ★★★ A REGION IS NOT AUTOMATICALLY SMALL. See
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
            |region| crate::render::strategy::region_raster_fits(region, raster_scale),
        )
    }

    /// What state a **strip** page is in, for
    /// [`crate::render::strip::draw_page_state`].
    ///
    /// `None` means "there is a current raster for it" — the caller draws the
    /// texture. Asked by the canvas while drawing, which is why it takes the
    /// key rather than deriving one: the canvas already knows this frame's
    /// raster scale and deriving a second one here is how the drawn page and
    /// the requested page come to disagree.
    #[must_use]
    pub fn strip_page_state(&self, page: usize, key: RenderKey) -> Option<PageState> {
        // ★ Per-page (O74): a page whose own revision has not moved keeps its
        // raster through an edit made on another sheet.
        match self
            .strip_rasters
            .get(page, key, self.page_epochs.get(page))
        {
            Some(PageRaster::Ready(_)) => None,
            Some(PageRaster::Failed(detail)) => Some(PageState::Refused(detail.clone())),
            // ★★★ Asked FIRST among the empty cases, and from the key's own
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
    #[must_use]
    pub fn strip_page_texture(&self, page: usize, key: RenderKey) -> Option<&PageTexture> {
        // ★ Per-page (O74).
        match self
            .strip_rasters
            .get(page, key, self.page_epochs.get(page))
        {
            Some(PageRaster::Ready(texture)) => Some(texture),
            _ => None,
        }
    }
}

impl PdfcerApp {
    /// Decide whether the cached page textures are still valid and, if not,
    /// whether to re-rasterize now or wait for a zoom gesture to settle.
    ///
    /// See the module docs. Called once per frame, **after** the frame has
    /// been laid out and its actions applied — which is what makes
    /// `strip_visible` (published by the canvas during layout) available and
    /// current.
    pub fn settle_and_rasterize(&mut self, ctx: &egui::Context, pixels_per_point: f32) {
        let Status::Open(doc) = &mut self.status else {
            return;
        };

        // The page object count, if it can have changed since it was last
        // reported. Here rather than in the open path because "on open" is
        // only one of the three occasions `PROJECT_PLAN.md` §4.3 requirement 3
        // asks for — the others are a page change and an edit, both of which
        // have already been applied by the time this runs. One call site that
        // cannot be forgotten beats three that can.
        doc.trace_object_count();

        // ★★★ O23's "see" half needs to know where this page's ink actually
        // reaches, and that answer comes from a decomposition — 469 ms on the
        // operator's benchmark sheet.
        //
        // Built HERE and not in `canvas::present`, and the placement is the
        // whole design: this runs once per frame *after* the layout, the
        // actions and the render decision, so the cost lands on a frame in
        // which the picture has already been asked for rather than inside the
        // one the operator is waiting on. `canvas::present` peeks at the
        // result through `OpenDoc::content_bounds_if_known` and gets `None`
        // until this has run, which is why a huge drawing shows its sheet on
        // one frame and its off-page content on the next.
        //
        // Idempotent: `ensure_page_objects` records its `(page, content
        // generation)` key before doing the work, so this is a `Cell` compare
        // on every frame but the first of each key — including for a page that
        // will not decompose at all.
        doc.ensure_content_bounds();

        // Collect a background render FIRST, before deciding staleness. Order
        // matters: a render that finished since the last frame has already
        // updated a texture's key, so polling first is what stops the
        // staleness test below from seeing the pre-render state and spawning a
        // second render for a page that just arrived.
        if doc.poll_render(ctx) {
            ctx.request_repaint();
        }
        // While one is in flight, keep the frames coming. Nothing else wakes
        // egui when a worker finishes — without this the finished page would
        // sit in the channel until the operator moved the mouse.
        if doc.render_worker.is_rendering() {
            ctx.request_repaint();
        }

        // Did the zoom change since last frame, and by what route?
        let now = Instant::now();
        if (doc.frame.observed_zoom - doc.view.zoom).abs() > f32::EPSILON {
            doc.frame.observed_zoom = doc.view.zoom;
            doc.frame.zoom_commit_at = if doc.frame.zoom_commanded {
                now // discrete command: no gesture in flight, do not wait
            } else {
                now + doc.zoom_settle()
            };
        }
        doc.frame.zoom_commanded = false;

        let wanted_scale =
            viewer::raster_scale(doc.view.zoom, pixels_per_point, doc.prefs.render_quality);
        let wanted = doc.render_key(wanted_scale);

        // ★ Step 2 of the priority (see the module header), and it runs before
        // anything is requested: a scroll that changed which page is current
        // changed no *picture*, so the textures are rehomed rather than
        // re-rendered.
        doc.rehome_current_page(wanted);

        // ★ The staleness comparison, and why it is ONE key.
        //
        // "Is the picture on screen still a picture of what the operator is
        // looking at?" is asked of the same `RenderKey` the worker labelled
        // the texture with. The categories below are the key's own, so the
        // policy lives with the type rather than being re-derived here.
        let current = doc.page_texture.as_ref().map(|t| t.key);
        // No texture at all is "stale" in the discrete sense: there is nothing
        // on screen worth waiting to replace.
        // ★ …and an EDIT is a discrete change too, even though it moves no
        // field of the key.
        //
        // The key answers "is this a picture of the right page, at the right
        // scale, with the right annotation stance". It cannot answer "is it a
        // picture of the right *revision*", because an edit changes none of
        // those. That third term is `page_texture_epoch`, and adding it here is
        // what lets `vector_edit` stop nulling the texture — which is what put
        // a blank page on screen after every edit.
        // ★★★ Per-page since 2026-08-31 (O74). The third term still answers
        // "is it a picture of the right REVISION" — it is now the revision of
        // *this page* rather than of the document, so an edit on sheet 3 no
        // longer re-rasterises the canvas while it is showing sheet 7.
        let stale_edit = doc.page_texture_epoch != doc.page_epochs.get(doc.view.page_index);
        let stale_discrete =
            stale_edit || current.is_none_or(|k| k.discrete_inputs() != wanted.discrete_inputs());
        let stale_scale = current.is_some_and(|k| k.scale_bits() != wanted.scale_bits());
        // ★★★ …AND WHETHER IT IS A PICTURE OF THE RIGHT PART OF THE PAGE.
        //
        // `OPERATOR_REQUESTS.md` O25. Above the pixmap ceiling a raster covers
        // the visible region rather than the page, so two textures of the same
        // page at the same scale can show *different places*. Without this
        // term a pan requested nothing at all — the old picture was drawn
        // correctly at its own region and slid off, leaving the newly exposed
        // area blank indefinitely. See `RenderKey::same_region`.
        //
        // ★★ Grouped with the SCALE rather than with the discrete inputs, and
        // the reason is the same debounce argument that put the scale there: a
        // region changes under a continuous gesture, and a render started on
        // every frame of a drag would be cancelled by the next one — the
        // worker is single-slot — so the operator would pan for a second and
        // receive nothing at the end of it.
        //
        // ★ It is already rate-limited in a way the scale is not:
        // `render::strategy::region_for` snaps to a half-viewport grid, so a
        // region changes at most once per half-screen of travel however
        // smoothly the pointer moves. The debounce is the second limiter, not
        // the only one, which is why the settle interval can stay tuned for
        // zoom without making a pan feel slow.
        let stale_region = current.is_some_and(|k| !k.same_region(&wanted));

        // ★★★ A PAGE WHOSE PREVIOUS RENDER FAILED MUST NOT BE RETRIED EVERY
        // FRAME: the failure is deterministic -- same bytes through the same
        // code -- so a retry can only reproduce it, and each one costs a
        // thread. Any genuinely different request is a different key or a
        // different epoch and gets its own attempt; hiding annotations can be
        // exactly what makes a page that would not draw draw.
        //
        // The strip is still serviced below: one page that will not draw must
        // not stop the pages around it from filling in.
        //
        let current_held = doc.render_refused.is_some_and(|(key, epoch)| {
            key == wanted && epoch == doc.page_epochs.get(doc.view.page_index)
        });

        //
        // A request with no region is a request for the WHOLE SHEET, and above
        // the renderer's pixmap ceiling there is no such pixmap. Placing the
        // order anyway is how the page being looked at came to refuse at a zoom
        // it could have rendered through the region tier — and worse, how
        // `absorb`'s `absorb_render` came to learn a *ceiling* from that
        // refusal and cap the operator's zoom at a number that was an
        // internal mistake. The whole measurement, and why declining
        // withholds no picture, is on `OpenDoc::raster_order_fillable`.
        //
        // ★ Gated on both spawn sites at once, by wrapping them, rather than
        // repeated inside each: the two arms differ only in *when* they ask, and
        // a guard added to one of two call sites is the shape of defect this
        // project has now corrected more than once.
        let fillable = doc.raster_order_fillable(doc.view.page_index, wanted_scale);
        crate::diag::trace_changed(CURRENT_UNFILLABLE_SLOT, || {
            format!(
                // ui-text-exempt: a diagnostic trace slot, never displayed. The
                // exemption sits INSIDE the macro because `cargo fmt` split this
                // call across lines and left the comment two lines above the
                // literal, out of the range check-ui-strings reads.
                "current-order-unfillable page={} fillable={fillable}",
                doc.view.page_index
            )
        });

        if !current_held && fillable {
            if stale_discrete {
                let page = doc.view.page_index;
                doc.rasterize(ctx, page, wanted_scale);
            } else if stale_scale || stale_region {
                if now >= doc.frame.zoom_commit_at {
                    let page = doc.view.page_index;
                    doc.rasterize(ctx, page, wanted_scale);
                } else {
                    // Nothing else will wake egui up when the debounce
                    // expires, so schedule it.
                    ctx.request_repaint_after(doc.frame.zoom_commit_at - now);
                }
            }
        }

        Self::fill_strip(ctx, doc, wanted_scale, now);
    }

    /// ★ **Prune the strip's cache to what is visible, then start at most one
    /// render for it.**
    ///
    /// Step 3 of the priority. Does nothing at all when `strip_visible` is
    /// empty or holds only the current page, which is every frame of every
    /// single-page session — so this whole feature costs a `Vec::is_empty`
    /// check on the default path.
    ///
    /// # Why exactly one render per frame, and why "nearest" is the order
    ///
    /// `RenderWorker` is single-slot by design: a second `spawn` cancels the
    /// first. So "start every missing page" would start the last one and
    /// abandon the rest, and a strip would fill in from the *bottom* of the
    /// viewport at one page per frame with every earlier page's work thrown
    /// away. One request per frame, always the nearest missing page, fills the
    /// strip outwards from where the operator is looking and never discards
    /// completed work.
    ///
    /// # Why it waits for the current page
    ///
    /// The current page is the largest thing on screen and the one being read.
    /// A strip page requested while it is still stale would cancel its render.
    /// The gate is `!doc.render_worker.is_rendering()` plus a settled current
    /// page: while a zoom is in flight, the strip stops asking entirely, so a
    /// wheel gesture over a continuous document costs the same one debounced
    /// render it costs over a single page.
    ///
    /// # ★ And why it must ASK FOR A FRAME while it is waiting
    ///
    /// **Found by driving the binary, not by a test.** Every gate was green and
    /// the strip did not fill: the trace showed `visible=2 drawn=1` and then
    /// nothing at all, for as long as the window was left alone.
    ///
    /// The cause is that egui is **event-driven**. Opening a document resolves
    /// the fit mode, which moves the zoom, which arms the 150 ms settle
    /// deadline; the current page renders inside the in-frame budget and
    /// requests one more frame to draw itself; on that frame the strip is still
    /// inside the settle window, so it asks for nothing — and nothing else
    /// wakes the process. The deadline passes with no frame to notice it, and
    /// page 2 stays undrawn until the operator moves the mouse.
    ///
    /// So a wait has to schedule its own wake-up, exactly as the zoom debounce
    /// already does for the current page (`ctx.request_repaint_after`). The
    /// symptom of getting this wrong is not a crash and not a wrong pixel; it
    /// is a feature that works perfectly whenever anyone is watching it and
    /// stalls the moment they stop, which is the single hardest kind of defect
    /// to see from a test suite.
    fn fill_strip(ctx: &egui::Context, doc: &mut OpenDoc, raster_scale: f32, now: Instant) {
        let current = doc.view.page_index;
        if doc.strip_visible.is_empty() {
            // Single page, or a frame before the canvas has laid out. Nothing
            // to hold and nothing to request; drop anything left over from a
            // mode the operator has since left.
            if !doc.strip_rasters.is_empty() {
                doc.strip_rasters.clear();
            }
            return;
        }
        let visible = std::mem::take(&mut doc.strip_visible);
        // ★★ `retain` no longer takes the visible set, and that is the whole of
        // the operator's *"they constantly redraw with larger files"*.
        //
        //
        // `visible` is still taken above, because the strip's *request* logic
        // below needs it: what to render next is still "the nearest visible page
        // that has no raster". Only the *keeping* rule changed.
        doc.strip_rasters
            .retain(current, doc.prefs.page_cache.texels());

        // A zoom gesture in flight: the whole strip waits with the current
        // page, for the same reason the current page waits. Requesting pages
        // at a scale the operator is still changing would rasterize a document
        // per wheel notch.
        let settling = now < doc.frame.zoom_commit_at;

        //
        // Without this line the fix below is invisible: the whole point of it is
        // that nothing happens — no request, no failure, no message — and an
        // absence is the one thing a driven check cannot assert. So the count of
        // visible neighbour sheets this frame declined to order is traced,
        // before the scan that skips them.
        //
        // ★ `trace_changed`, and a COUNT rather than a list of page indices, so
        // the cardinality is one line per transition instead of one line per
        // frame of a zoom gesture. Which pages they were is derivable: the
        // canvas already publishes the visible set through
        // `strip-raster-requested visible=`, and every visible page that is not
        // the current one and has no raster is one of these.
        //
        // `pages=0` is printed too, on purpose. Leaving the regime has to be
        // visible or the check cannot tell "never entered it" from "still in
        // it", which is how an absence assertion comes to pass against a
        // planted defect.
        let beyond = visible
            .iter()
            .copied()
            .filter(|&page| page != current && !doc.strip_page_orderable(page, raster_scale))
            .count();
        crate::diag::trace_changed(BEYOND_RASTER_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            // ★ The SCALE is deliberately absent. Including it would make the
            // line change on every wheel notch and defeat the de-duplication
            // this slot exists for; the zoom is already published by
            // `canvas-pos`, once per transition, from the canvas that decided
            // it.
            format!("strip-beyond-raster pages={beyond}")
        });

        let next = visible
            .iter()
            .copied()
            .filter(|&page| page != current)
            // ★★★ **O186's raster error, at its source.** A strip page is
            // ordered whole-sheet — `OpenDoc::region_for` gives a region to the
            // current page only — so above the renderer's pixmap ceiling there
            // is no order to place. Asking anyway is what painted
            // *"requested raster size 50411508x32619210 is empty or exceeds
            // MAX_PIXMAP_EDGE"* across a neighbouring sheet of the operator's
            // drawing set while the sheet he was reading drew perfectly.
            //
            // ★ A `filter`, deliberately, and not a check inside the `find`'s
            // predicate or after it: the scan must CARRY ON to the next
            // candidate. A document whose visible pages are an unorderable A1
            // followed by an orderable letter sheet must still fill the letter
            // sheet, and a `find` that stopped at the first unorderable page
            // would starve it forever.
            //
            // The same question answers what the page says about itself — see
            // `OpenDoc::strip_page_orderable`.
            .filter(|&page| doc.strip_page_orderable(page, raster_scale))
            .find(|&page| {
                !doc.strip_rasters.has(
                    page,
                    doc.render_key_for(page, raster_scale),
                    // ★ Per-page (O74): this is the scan that decides which
                    // page the worker fills next, so a document-wide key here
                    // put every page back in the queue after every edit.
                    doc.page_epochs.get(page),
                )
            });

        // O201 requirement 2, in his words: *"the ones on screen should always
        // take precedence to be rendered first"*. Structurally, the band below is
        // only consulted when `next` is `None`, so a prefetch can never be
        // ORDERED ahead of a visible page. That covers the queue and not the
        // worker: a prefetch already running would still make a page he has just
        // scrolled to wait out a render of a page he has not reached.
        //
        // So a running prefetch is CANCELLED for a visible page. The predicate
        // can only be true while a page that is neither visible nor current is
        // rendering, and spawning makes it false, so this cannot chase itself —
        // which is the livelock `RenderKey` documents and the reason the strip's
        // ordinary requests wait rather than pre-empt.
        let preempting = next.is_some()
            && doc
                .render_worker
                .rendering_key()
                .is_some_and(|key| key.page() != current && !visible.contains(&key.page()));

        let order = match next {
            Some(page) => Some((page, false)),
            // Every visible page is filled, so what is left of the budget goes
            // to the pages he has not reached yet.
            None => doc
                .prefetch_candidate(&visible, raster_scale)
                .map(|page| (page, true)),
        };

        // The off-canvas half of rule 4 for render-ahead: nothing on the
        // page says a picture arrived early, so the count has to be
        // reportable from somewhere. Emitted before the branch that may or
        // may not act, so the resident band is stated every frame the
        // number changes rather than only on the frames that order.
        prefetch::disclose(doc);

        if let Some((page, prefetch)) = order {
            if settling {
                // The wake-up. See this function's docs: without it the
                // deadline passes on an idle window with no frame to notice
                // it, and the strip stops filling until the operator moves the
                // mouse.
                ctx.request_repaint_after(doc.frame.zoom_commit_at - now);
            } else if !doc.render_worker.is_rendering() || preempting {
                let seen = visible.len();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    if prefetch {
                        format!("strip-prefetch-requested page={page} current={current}")
                    } else {
                        format!("strip-raster-requested page={page} visible={seen}")
                    }
                });
                doc.rasterize(ctx, page, raster_scale);
            }
            // The remaining case — a visible render already in flight — needs
            // nothing: `settle_and_rasterize` asks for a frame on every frame a
            // worker is running, so the next one arrives without help.
        }
        doc.strip_visible = visible;
    }
}

/// # Tests — can this order be filled?
///
/// Only [`OpenDoc::raster_order_fillable`] is covered here, and deliberately
/// only it. Everything else in this file is a frame's worth of sequencing
/// against a live `egui::Context`, a worker thread and a wall clock; the one
/// part that is a pure question about a document is the predicate O186's third
/// route turns on, and that is the part a unit test can actually pin.
///
/// ⚠ **What these tests do NOT establish is that the guard is in the right
/// place.** They prove the predicate answers correctly; they cannot see the
/// `if !current_held && fillable` that consults it, and a build with that
/// `&& fillable` deleted passes every one of them — measured, not assumed. The
/// driven coverage that is owed, and the check that will eventually supply it,
/// is named on [`OpenDoc::raster_order_fillable`] itself. This paragraph exists
/// so that a reader who finds four green tests here does not conclude the route
/// is covered.
///
/// ## ★★ Why the fixture is this repository's and not the engine's
///
/// `open_local_fixture("four-pages.pdf")`, **not**
/// `open_fixture(FOUR_PAGES)` — and the difference is not cosmetic. There are
/// two documents on this machine called `four-pages.pdf`:
///
/// | path | pages |
/// |---|---|
/// | engine `synthetic/pageops/four-pages.pdf` | four sheets, **all US Letter** |
/// | this repo's `fixtures/four-pages.pdf` | `2383.937 × 1683.78`, `612 × 792`, `612 × 792`, `306 × 396` |
///
///
/// ★ Opened rather than hand-built, for the reason `app::status::rasterstop`'s
/// tests give: [`Self::strip_page_orderable`] reaches `page_extent_pts`, which
/// reads the real `/MediaBox` and `/Rotate`, so a synthesised page would check
/// the arithmetic against a number this test invented rather than against a
/// document.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::open_local_fixture;

    /// This repository's `fixtures/four-pages.pdf`, whose four sheets differ.
    const FOUR_DIFFERING_SHEETS: &str = "four-pages.pdf";

    /// Page 0 is `2383.937 × 1683.78` pt, so against the rasterizer's
    /// 16,384-pixel edge limit its whole-sheet raster stops fitting at a raster
    /// scale of `16384 / 2383.937 = 6.87`.
    ///
    /// Both constants sit a wide factor either side of that on purpose. A test
    /// that straddled 6.87 closely would be measuring the engine's rounding,
    /// which is the engine's business and not this predicate's — and it would
    /// go red the day `MAX_PIXMAP_EDGE` changes, reporting a defect here that
    /// is not here.
    const BIG_SHEET_FITS: f32 = 3.0;
    const BIG_SHEET_DOES_NOT_FIT: f32 = 100.0;

    /// Page 3 is `306 × 396` pt, six times smaller, so its limit is `41.4` —
    /// which is what makes a scale of 30 comfortable for it and hopeless for
    /// page 0. That asymmetry is the whole point of the fixture.
    const SMALL_SHEET_SCALE: f32 = 30.0;

    fn doc() -> OpenDoc {
        open_local_fixture(FOUR_DIFFERING_SHEETS)
    }

    /// A region covering an arbitrary patch of the named page.
    ///
    /// The rectangle's own size is irrelevant and deliberately small: what the
    /// predicate asks is whether a region *exists for this page*, because the
    /// request a region produces is viewport-sized rather than page-sized. A
    /// test that made the rectangle large would imply the size mattered.
    fn region_on(page: usize) -> (usize, pdfcer_core::page_tree::Rect) {
        (
            page,
            pdfcer_core::page_tree::Rect {
                llx: 0.0,
                lly: 0.0,
                urx: 100.0,
                ury: 100.0,
            },
        )
    }

    /// ★★ **The half that must answer yes, and the half that must answer no.**
    ///
    /// Asserted in one test because either alone is satisfied by a constant: a
    /// predicate hard-wired to `true` passes the first assertion and one
    /// hard-wired to `false` passes the second, so a suite holding only one of
    /// them would be green against a function that had stopped reading its
    /// arguments.
    ///
    /// The third assertion is the one that proves the answer is about the
    /// **page**. Page 3 is a sixth of page 0, so `SMALL_SHEET_SCALE` is fine for
    /// it and far past page 0's limit; an implementation written against a
    /// single document-wide extent — the most likely wrong version of this —
    /// fails here and nowhere else.
    #[test]
    fn with_no_region_an_order_is_fillable_only_while_the_whole_sheet_fits() {
        let doc = doc();
        assert!(
            doc.raster_region.is_none(),
            "a freshly opened document has no region, which is the state this \
             test is about"
        );

        assert!(
            doc.raster_order_fillable(0, BIG_SHEET_FITS),
            "the big sheet's whole-page raster fits at a raster scale of \
             {BIG_SHEET_FITS}"
        );
        assert!(
            !doc.raster_order_fillable(0, BIG_SHEET_DOES_NOT_FIT),
            "at {BIG_SHEET_DOES_NOT_FIT} no pixmap that size can be allocated, \
             so the order cannot be filled and must not be placed"
        );
        assert!(
            doc.raster_order_fillable(3, SMALL_SHEET_SCALE),
            "page 3 is six times smaller and fits whole at a scale that is \
             hopeless for page 0 — this is the assertion that proves the answer \
             is about the page and not about the document"
        );
    }

    /// ★★★ **A region makes the SHEET's size stop mattering — and puts the
    /// REGION's size in its place.**
    ///
    /// The first half is why this predicate is not simply
    /// [`Self::strip_page_orderable`]: page 0's whole-sheet raster is hopeless
    /// at [`BIG_SHEET_DOES_NOT_FIT`] while a 100 pt box on it is 10,000 px a
    /// side and perfectly ordinary. The second half is the one that had to be
    /// learned, and the assertion below is its whole point:
    ///
    /// > a region is not automatically small.
    ///
    /// The visible-rect tier's box is a multiple of the WINDOW, so its device
    /// size is the same at 800 % and at 36,000,000 % — that is the tier this
    /// predicate was written for and it really is scale-free. The **off-page
    /// halo** box is not: it is fixed in the page's own space and grows with
    /// the zoom exactly as the sheet does, and being the larger rectangle it
    /// reaches `MAX_PIXMAP_EDGE` first. Answering `true` for it sent an order
    /// that came back `BadRasterSize`, and `absorb_render` turned that refusal
    /// into the document's zoom ceiling.
    ///
    /// ⚠ The third assertion is the one a rewrite must not drop. Without it the
    /// test is satisfied by `region_for(page).is_some()`, which is the wrong
    /// implementation it replaced.
    #[test]
    fn a_region_is_fillable_only_while_the_region_itself_fits() {
        let mut doc = doc();
        doc.raster_region = Some(region_on(0));

        assert!(
            doc.raster_order_fillable(0, BIG_SHEET_DOES_NOT_FIT),
            "a 100 pt box is 10,000 px a side at {BIG_SHEET_DOES_NOT_FIT}, so \
             the SHEET's size has stopped being the question"
        );
        assert!(
            !doc.raster_order_fillable(0, 1.0e6),
            "but a million-fold in, that same box is 100,000,000 px a side and \
             no such pixmap exists — the region's own size is now the question"
        );

        // The halo box on the operator's site plan, to the point: the union of
        // an A3 sheet with content spilling off it, which is A1-sized. Its
        // whole-page twin fits at scale 8 and it does not, which is exactly the
        // gap the old predicate fell through.
        doc.raster_region = Some((
            0,
            pdfcer_core::page_tree::Rect {
                llx: -113.932,
                lly: -4.179,
                urx: 2270.013,
                ury: 1679.604,
            },
        ));
        assert!(
            !doc.raster_order_fillable(0, 8.0),
            "the off-page halo box is 2,384 pt wide, so at scale 8 it is 19,073 \
             px and cannot be allocated — a region that is BIGGER than the \
             sheet must not be waved through because it is a region"
        );
        assert!(
            doc.raster_order_fillable(0, 6.0),
            "while at scale 6 it is 14,304 px and orders perfectly well, which \
             is what makes the line above a boundary rather than a blanket \
             refusal"
        );
    }

    /// ★ **A region belonging to another page is not a region.**
    ///
    /// This is the assertion that pins [`Self::region_for`] rather than the
    /// `raster_region` field inside the predicate. Both rectangles are valid, so
    /// reading the field directly would answer `true` for every page in the
    /// document the moment any one page had a region — and nothing would report
    /// it, because the consequence is simply that the wrong sheet gets ordered
    /// whole and refused, which is the defect this guard exists to stop.
    ///
    /// The second half is what makes the first half a statement about *whose*
    /// region it is rather than about regions being ignored altogether.
    #[test]
    fn a_region_belonging_to_another_page_does_not_make_this_one_fillable() {
        let mut doc = doc();
        doc.raster_region = Some(region_on(1));

        assert!(
            !doc.raster_order_fillable(0, BIG_SHEET_DOES_NOT_FIT),
            "page 1's region says nothing about page 0"
        );
        assert!(
            doc.raster_order_fillable(1, BIG_SHEET_DOES_NOT_FIT),
            "while page 1's own region does"
        );
    }

    /// A page index past the end is not fillable, and must not panic.
    ///
    /// Reachable in practice on the frame after a page is deleted, before the
    /// view index has been brought back into range. The answer is `false` rather
    /// than `true` because there is no sheet to order at all — a `true` here
    /// would place a request the worker could only discard, spending a thread on
    /// a page that does not exist.
    #[test]
    fn a_page_past_the_end_is_never_fillable() {
        let doc = doc();
        assert!(!doc.raster_order_fillable(99, BIG_SHEET_FITS));
        assert!(!doc.raster_order_fillable(usize::MAX, BIG_SHEET_FITS));
    }
}
