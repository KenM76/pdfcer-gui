//! # `render::settle` — is the picture on screen still a picture of what I am looking at?
//!
//! The per-frame raster decision: what is stale, what to re-rasterize now,
//! what to debounce until a zoom gesture stops, and — from Phase 4 — which of
//! a continuous strip's several visible pages to render next.
//!
//! ## Why this is a module of its own, and where it came from
//!
//! It was the second half of [`crate::app::state`], whose header described
//! itself as holding two things: *"the shape of what, if anything, is open"*
//! and *"the raster bookkeeping"*. Phase 4 made the second half considerably
//! larger — one texture became a texture plus a bounded cache, and one
//! staleness question became two — and the file was already at 1,435 of rule
//! R2's 1,500 lines.
//!
//! The seam is a real one rather than arithmetic, and it is the one that file
//! had already named: everything left in `state.rs` answers *"what is open,
//! and what is the operator looking at?"*, while everything here answers
//! *"what does the picture need to be, and what should be done about it this
//! frame?"*. The two change for different reasons — the first when a document
//! gains a property, the second when rendering gains a strategy — and only the
//! second belongs in `render/`, beside the worker it schedules and the texture
//! cache it prunes.
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
//!   through `OpenDoc::zoom_commanded`: there is no gesture in flight, so
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

use std::time::{Duration, Instant};

use crate::app::PdfcerApp;
use crate::app::state::{OpenDoc, Status};
use crate::render::raster::{self, PageTexture};
use crate::render::strip::{PageRaster, PageState};
use crate::render::worker::{RenderKey, RenderedPixels};
use crate::viewer;

/// How long a zoom must stop changing before it is committed to a real
/// rasterization.
///
/// Long enough to swallow a whole wheel gesture, short enough that a
/// deliberate single step does not feel laggy. 150 ms is the value the old
/// shell settled on against real CAD sheets; it is a constant rather than a
/// literal so the next person to tune it does so once, with a paper trail.
pub const ZOOM_SETTLE: Duration = Duration::from_millis(150);

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
    /// Hand a page to the worker and, if it beats the in-frame budget, absorb
    /// the result immediately.
    ///
    /// `page_index` is passed rather than read from the view because this is
    /// also how a **strip** page is requested, and a strip page is by
    /// definition not the current one. Routing of the result is by the key's
    /// own page — see [`OpenDoc::absorb_render`] — so a render that finishes
    /// after the operator has scrolled lands wherever that page now belongs
    /// rather than wherever it belonged when it started.
    fn rasterize(&mut self, ctx: &egui::Context, page_index: usize, raster_scale: f32) {
        let Some(request) = self.render_request_for(page_index, raster_scale) else {
            // No such page. For the current page that means the index is past
            // the end, which `clamp_page_index` normally prevents; clearing
            // the texture is the honest response either way.
            if page_index == self.view.page_index {
                self.page_texture = None;
            }
            return;
        };
        // `spawn` waits a bounded number of milliseconds inline, so a page that
        // rasterizes quickly returns its pixels here and never touches the
        // asynchronous path — behaviour identical to a synchronous render. A
        // page that misses that budget returns `None` and is collected by
        // `poll_render` on a later frame, with the previous texture staying on
        // screen meanwhile.
        if let Some(result) = self.render_worker.spawn(request) {
            self.absorb_render(ctx, result);
        }
        // Rasterization happens *after* the canvas has already been laid out
        // this frame, so the new texture cannot be drawn until the next one.
        // Without this the display would wait for whatever unrelated input
        // happened to arrive next, which on an idle window is "until the
        // operator wiggles the mouse".
        ctx.request_repaint();
    }

    /// Turn a finished rasterization into a cached texture — the current
    /// page's slot, or the strip's cache.
    ///
    /// Shared by the in-frame fast path and the per-frame poll so the two
    /// cannot drift: a render that beat the budget and one that took a minute
    /// must produce exactly the same canvas state.
    ///
    /// # ★ The routing is by the render's own page, not by what asked for it
    ///
    /// A render is labelled with the [`RenderKey`] it was run from, so the
    /// page it is *of* is knowable from the result alone. That is what makes
    /// the scroll case correct: a strip page whose render finishes after the
    /// operator has scrolled onto it lands in the current page's slot, and the
    /// current page's render that finishes after they have scrolled past it
    /// lands in the strip. Routing by "which slot asked" would have to
    /// remember the request, and would be wrong in exactly those two cases.
    fn absorb_render(&mut self, ctx: &egui::Context, result: Result<RenderedPixels, String>) {
        match result {
            Ok(pixels) => {
                let page = pixels.key.page();
                let key = pixels.key;
                // A page blended in ink, recorded before anything else is done
                // with the result. The ONE write to `OpenDoc::ink_pages`, and
                // the observation `render::strategy::Ink` rests on -- see that
                // type for why the shell learns this rather than assuming it.
                //
                // **Either counter**: `engaged` means the colorant buffer was
                // used, `refused` means it was wanted and would have exceeded
                // its ceiling. Both mean the page ASKED to be blended in ink,
                // and it is the asking that decides the tier. Keying on
                // `engaged` alone would be self-defeating in the exact case
                // this exists for: a page already past the ceiling reports
                // `engaged = 0`, so the tier would never move down and the
                // colours would never come back.
                // `engaged` is a bool and `refused` is a count; they are
                // spelled differently because they answer differently-shaped
                // questions, and mixing that up is a compile error rather than
                // a silent one.
                if pixels.diagnostics.cmyk_buffer_engaged
                    || pixels.diagnostics.cmyk_buffer_refused > 0
                {
                    // Traced on the transition only. This runs on every
                    // completed raster, and a line per raster would bury the
                    // one that matters.
                    if self.ink_pages.insert(page) {
                        crate::diag::trace(move || {
                            // ui-text-exempt: diagnostic trace, never displayed in the UI
                            format!("ink-page page={page}")
                        });
                    }
                }
                let texture = raster::texture_from_pixels(ctx, &pixels);
                if page == self.view.page_index {
                    // ★★★ **PROMOTE IT TO THE BACKDROP** if it is a small
                    // whole-page picture. See `OpenDoc::base_texture`.
                    //
                    // Three conditions, and each excludes a specific way this
                    // could go wrong:
                    //
                    // * `region().is_none()` — a REGION raster is a picture of
                    //   part of the page, and a backdrop that covered part of
                    //   the page would leave exactly the gap it exists to fill.
                    // * under the pixel budget — the whole-page rasters just
                    //   below the region tier are hundreds of megapixels, and
                    //   retaining one would trade a blank page for an
                    //   out-of-memory.
                    // * the epoch matches — a backdrop from before an edit
                    //   would show content the document no longer has.
                    //
                    // ★ The handle is CLONED, not copied: at a fit zoom the
                    // backdrop and the live texture are the same pixels and
                    // cost nothing, and they only diverge once the operator
                    // zooms past it.
                    if key.region().is_none() && raster::within_base_budget(&pixels) {
                        self.base_texture = Some(texture.clone());
                        // ★ Per-page (O74), like the live texture below it.
                        // The backdrop is a picture of ONE page, so an edit on
                        // another sheet has nothing to say about it.
                        self.base_texture_epoch = self.page_epochs.get(page);
                        // ★ Published, because the backdrop is invisible when
                        // it is working: it only shows in the gaps a sharp
                        // raster leaves, and at a fit zoom there are none. A
                        // check that could not see it being kept would have to
                        // infer its existence from the absence of a symptom.
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed in the UI
                            format!(
                                "backdrop-kept page={page} px={}x{}",
                                pixels.pixmap.width(),
                                pixels.pixmap.height()
                            )
                        });
                    }
                    self.page_texture = Some(texture);
                    // Stamped with the epoch it is a picture of, exactly as the
                    // strip's own `insert` two lines below has always been. See
                    // `OpenDoc::page_texture_epoch`.
                    // ★ Per-page (O74): an edit on sheet 3 must not
                    // re-rasterise the canvas while it is showing sheet 7.
                    self.page_texture_epoch = self.page_epochs.get(page);
                    self.render_error = None;
                } else {
                    self.strip_rasters.insert(
                        page,
                        key,
                        // ★★★ Per-page (O74) — see this module's note on the
                        // stamp above. `strip_rasters` already took the epoch
                        // as a parameter of every one of `get`/`has`/`take`/
                        // `insert`, so the whole change here is which number is
                        // passed.
                        self.page_epochs.get(page),
                        PageRaster::Ready(Box::new(texture)),
                    );
                }
            }
            Err(message) => {
                // ★ A failure carries no key — `RenderWorker` reports the
                // message alone — so it is attributed to whatever the worker
                // was rendering. That is exactly the page it is about: the
                // worker is single-slot, and `render_in_flight` was read
                // *before* the poll took the slot (see `poll_render`). Without
                // that reading, a strip page that would not draw would blank
                // the whole canvas by landing in `render_error`.
                match self.render_in_flight.take() {
                    Some(key) if key.page() != self.view.page_index => {
                        self.strip_rasters.insert(
                            key.page(),
                            key,
                            // ★ A refusal is filed at the same per-page number
                            // as a picture, so an edit to that page gets it a
                            // second attempt and an edit elsewhere does not.
                            self.page_epochs.get(key.page()),
                            PageRaster::Failed(message),
                        );
                    }
                    _ => {
                        self.page_texture = None;
                        self.render_error = Some(message);
                    }
                }
            }
        }
    }

    /// Collect a background render, if one has finished.
    ///
    /// Called once per frame. Returns whether anything was absorbed, so the
    /// caller can request the repaint that draws it.
    fn poll_render(&mut self, ctx: &egui::Context) -> bool {
        // Read what the worker is on BEFORE polling: `poll` takes the
        // in-flight slot, and a failure arrives with no key of its own. See
        // `absorb_render`.
        self.render_in_flight = self.render_worker.rendering_key();
        let Some(result) = self.render_worker.poll() else {
            self.render_in_flight = None;
            return false;
        };
        self.absorb_render(ctx, result);
        self.render_in_flight = None;
        true
    }

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
            }
            Some(PageRaster::Failed(message)) => {
                self.page_texture = None;
                self.render_error = Some(message);
            }
            None => self.render_error = None,
        }
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
        if (doc.observed_zoom - doc.view.zoom).abs() > f32::EPSILON {
            doc.observed_zoom = doc.view.zoom;
            doc.zoom_commit_at = if doc.zoom_commanded {
                now // discrete command: no gesture in flight, do not wait
            } else {
                now + doc.zoom_settle()
            };
        }
        doc.zoom_commanded = false;

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

        // A page whose previous render failed must not be retried every frame:
        // the failure is deterministic (same bytes, same code), so retrying
        // would peg a core producing the same error. Any discrete change is a
        // genuinely different request and clears the hold — hiding annotations
        // can be exactly what makes a page that would not draw draw.
        //
        // The strip is still serviced below: one page that will not draw must
        // not stop the pages around it from filling in.
        let current_held = doc.render_error.is_some() && !stale_discrete;

        if !current_held {
            if stale_discrete {
                let page = doc.view.page_index;
                doc.rasterize(ctx, page, wanted_scale);
            } else if stale_scale || stale_region {
                if now >= doc.zoom_commit_at {
                    let page = doc.view.page_index;
                    doc.rasterize(ctx, page, wanted_scale);
                } else {
                    // Nothing else will wake egui up when the debounce
                    // expires, so schedule it.
                    ctx.request_repaint_after(doc.zoom_commit_at - now);
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
        // It used to drop every entry not on screen, so a sheet scrolled past
        // was rendered again from the content stream the moment it came back —
        // 691 ms on a dense A1 (`BENCHMARK.md`). What bounds the cache now is
        // the operator's own budget and the distance rule inside `retain`.
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
        let settling = now < doc.zoom_commit_at;
        let next = visible
            .iter()
            .copied()
            .filter(|&page| page != current)
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

        if let Some(page) = next {
            if settling {
                // The wake-up. See this function's docs: without it the
                // deadline passes on an idle window with no frame to notice
                // it, and the strip stops filling until the operator moves the
                // mouse.
                ctx.request_repaint_after(doc.zoom_commit_at - now);
            } else if !doc.render_worker.is_rendering() {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "strip-raster-requested page={page} visible={}",
                        visible.len()
                    )
                });
                doc.rasterize(ctx, page, raster_scale);
            }
            // The third case — a render already in flight — needs nothing:
            // `settle_and_rasterize` asks for a frame on every frame a worker
            // is running, so the next one arrives without help.
        }
        doc.strip_visible = visible;
    }
}
