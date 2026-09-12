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
use crate::render::worker::{RefusalKind, RenderKey, RenderOutcome};
use crate::viewer;

/// How long a zoom must stop changing before it is committed to a real
/// rasterization.
///
/// Long enough to swallow a whole wheel gesture, short enough that a
/// deliberate single step does not feel laggy. 150 ms is the value the old
/// shell settled on against real CAD sheets; it is a constant rather than a
/// literal so the next person to tune it does so once, with a paper trail.
pub const ZOOM_SETTLE: Duration = Duration::from_millis(150);

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
        // ★ Record WHAT IS BEING ASKED FOR before the inline wait.
        //
        // `poll_render` does this for the asynchronous path by reading
        // `rendering_key()` before it polls, because a failure arrives as a
        // bare message with no key of its own. The inline path has no poll to
        // read before -- and it is not the rare path for a failure, it is the
        // usual one: a worker that panics drops its end of the channel, so
        // `recv_timeout` returns `Disconnected` at once and `spawn` hands the
        // refusal straight back here. Every one of the 573 panics that
        // `OpenDoc::render_refused` exists to stop arrived this way, with an
        // empty slot, which is why the refusal could not be attributed and so
        // could not be held.
        let asked = RenderKey::of(&request);
        self.render_in_flight = Some(asked);
        if let Some(result) = self.render_worker.spawn(request) {
            self.absorb_render(ctx, result);
        }
        // Consumed by the absorb above if it failed; cleared here for the
        // ordinary case, so nothing downstream reads a stale slot as a render
        // still running.
        self.render_in_flight = None;
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
    ///
    /// # ★★★ It is also where a raster refusal becomes a zoom CEILING — O186
    ///
    /// The operator, 2026-09-12: *"zoom should stop at the limit and not end up
    /// showing an error — the canvas will just stop zooming in and can still
    /// function."* This function is the one place in the shell a render refusal
    /// is absorbed, so it is necessarily the place that clause is executed: see
    /// the `Err` arm's own commentary for the learn-and-pull-back, and
    /// [`crate::render::ceiling`] for why the number can only come from a
    /// refusal that has already happened.
    fn absorb_render(&mut self, ctx: &egui::Context, result: RenderOutcome) {
        match result {
            Ok(pixels) => {
                let page = pixels.key.page();
                let key = pixels.key;
                // A page blended in ink, recorded before anything else is done
                // with the result.
                //
                // This comment said "the ONE write to `OpenDoc::ink_pages`"
                // until 2026-09-11, and it had stopped being true when
                // `OpenDoc::learn_ink` began asking
                // `pdfcer_render::page_composites_in_ink` directly. There are
                // TWO writers now, and `app::state`'s field doc is the one
                // that says so and explains why the pair is safe: the engine
                // ships a test asserting the ask and the render agree on every
                // fixture, so this is a second observer that can only confirm
                // the first, never contradict it.
                //
                // An exclusivity claim is the shape of comment that a later,
                // correct change falsifies WITHOUT touching the file the
                // sentence lives in - nothing fails to compile and no gate
                // counts writers. Prefer "a writer, and here is the invariant
                // that makes several safe" over "the writer".
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
                    // A picture arrived, so whatever was refused before is no
                    // longer the answer for this page. Cleared unconditionally
                    // rather than by comparing keys: any success at all means
                    // the page draws, and a memo kept past that would refuse a
                    // request the evidence says would work.
                    self.render_refused = None;
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
            Err(refusal) => {
                // ★ A failure carries no key — `RenderWorker` reports the
                // refusal alone — so it is attributed to whatever the worker
                // was rendering. That is exactly the page it is about: the
                // worker is single-slot, and `render_in_flight` was read
                // *before* the poll took the slot (see `poll_render`). Without
                // that reading, a strip page that would not draw would blank
                // the whole canvas by landing in `render_error`.
                //
                // ★★ **The refusal carries a KIND as well as a sentence** — O186.
                // See [`RefusalKind`]: the whole of this arm's new behaviour
                // turns on `BeyondRaster`, and none of it may turn on reading
                // the sentence, which is a string from `crate::text` that the
                // operator is free to have reworded.
                match self.render_in_flight.take() {
                    Some(key) if key.page() != self.view.page_index => {
                        // ★★ **A neighbour's limit is still a FACT about that
                        // neighbour**, so it is written down — but the zoom is
                        // NOT pulled back for it.
                        //
                        // The distinction is the whole of why the learn and the
                        // clamp are separated. A ceiling belongs to the page it
                        // was measured on (`RefusalKind::BeyondRaster` records
                        // the 28x spread measured between two pages of one
                        // document), and `crate::viewer::zoom_ceiling` is asked
                        // only about the page the operator is looking at. So
                        // learning here costs nothing today and pays when he
                        // navigates onto that sheet: his zoom is already capped
                        // where it can be drawn, having never shown him an
                        // error at all.
                        //
                        // ★ Pulling the zoom back here instead would be the
                        // defect: it would cap the sheet he IS looking at —
                        // which renders perfectly — because a different sheet in
                        // the same window cannot be drawn that far in.
                        if refusal.kind == RefusalKind::BeyondRaster {
                            self.raster_ceiling.learn(
                                key.page(),
                                key.raster_scale(),
                                self.page_epochs.get(key.page()),
                            );
                        }
                        self.strip_rasters.insert(
                            key.page(),
                            key,
                            // ★ A refusal is filed at the same per-page number
                            // as a picture, so an edit to that page gets it a
                            // second attempt and an edit elsewhere does not.
                            self.page_epochs.get(key.page()),
                            PageRaster::Failed(refusal.message),
                        );
                    }
                    slot => {
                        // ★★★ **O186's FIRST THREE CLAUSES, and they are one
                        // act:** *"zoom should stop at the limit and not end up
                        // showing an error — the canvas will just stop zooming
                        // in and can still function."*
                        //
                        // `handled` is true when all three were honoured,
                        // which requires every one of these to hold:
                        //
                        // * the refusal is a raster wall and not a broken page
                        //   (`BeyondRaster` — a page that will not decode fails
                        //   at the fit zoom too, and capping the zoom for it
                        //   would word a real defect as a magnification limit);
                        // * the refusal is attributable, so there is a page and
                        //   a scale to write down;
                        // * the ceiling is news — `learn` returns `None` for a
                        //   repeat, and a repeat means the clamp below already
                        //   ran and did not hold, which is a condition to
                        //   report rather than to silently re-apply;
                        // * the resulting zoom is strictly BELOW where he
                        //   stands. This is the guard that keeps the silence
                        //   honest: if the clamp cannot actually move the view
                        //   down — the arithmetic bottomed out at
                        //   `viewer::MIN_ZOOM`, or the scale that refused was
                        //   somehow at or below the current one — then nothing
                        //   has been fixed, and swallowing the error would
                        //   leave a blank page with no explanation anywhere.
                        //   Fall through and tell him.
                        //
                        // ★★ When `handled`, `render_error` is deliberately NOT
                        // set and `page_texture` is deliberately NOT cleared.
                        // Both are the ordinary, correct responses to a failed
                        // render and both are wrong here:
                        //
                        // * the sentence is the thing he asked us to stop
                        //   painting across his drawing, and its replacement is
                        //   `crate::app::status::rasterstop` on the bottom bar
                        //   — his fourth clause;
                        // * the texture on screen is a picture of this page at
                        //   a LOWER zoom, and the zoom has just been pulled
                        //   down toward it. Clearing it would blank the canvas
                        //   at the exact moment the clamp made it more nearly
                        //   right, and *"can still function"* is the clause
                        //   that forbids it.
                        let handled = refusal.kind == RefusalKind::BeyondRaster
                            && slot.is_some_and(|key| self.learn_raster_ceiling(ctx, key));
                        if handled {
                            return;
                        }
                        // ★★★ THE MEMO THAT STOPS THE RETRY STORM.
                        //
                        // `render_error` alone cannot do this job -- see
                        // `OpenDoc::render_refused` for the two independent
                        // reasons, both measured. The key is the whole
                        // invalidation rule: a different zoom, region,
                        // annotation stance or page is a different key and
                        // gets its own attempt; the epoch does the same for an
                        // edit.
                        //
                        // `slot` is `None` only if a refusal arrives with no
                        // record of what was asked, which the stamp in
                        // `rasterize` and the one in `poll_render` between them
                        // prevent. If it ever does, the honest response is to
                        // hold nothing and try again -- an unattributable
                        // refusal is not evidence about any particular request.
                        self.render_refused =
                            slot.map(|key| (key, self.page_epochs.get(self.view.page_index)));
                        self.page_texture = None;
                        self.render_error = Some(refusal.message);
                    }
                }
            }
        }
    }

    /// **Write down what this page's raster limit turned out to be, and bring
    /// the zoom back under it** — O186.
    ///
    /// Returns whether the refusal was *fully absorbed*: a `true` means the
    /// operator has been moved to a zoom this page can be drawn at, so the
    /// caller must not also file an error. A `false` means nothing useful could
    /// be concluded and the ordinary refusal path must run — the four
    /// conditions are enumerated at the call site, which is the only caller.
    ///
    /// # Why the clamp is expressed in ZOOM and the ceiling in raster SCALE
    ///
    /// They are different quantities and the conversion between them is the
    /// display's density: a raster scale is *device pixels per PDF point*,
    /// which is `zoom * pixels_per_point`. The ceiling is stored in scale
    /// because that is what the renderer refused and the only quantity the
    /// refusal is a fact about — move the window to a 200 % monitor and the
    /// same zoom orders twice the pixels, so a ceiling stored in zoom would be
    /// wrong by a factor of two on the other screen, silently, and only on the
    /// machine it was not measured on.
    ///
    /// ★ So `pixels_per_point` is read here, at the moment of the clamp, and
    /// again on every frame by [`crate::viewer::zoom_ceiling`]. Neither caches
    /// it. A dragged window is a real gesture on this operator's desk — he runs
    /// two monitors at different densities — and a cached density is a ceiling
    /// that is wrong exactly after the drag.
    ///
    /// # Why it floors at [`crate::viewer::MIN_ZOOM`] and then checks again
    ///
    /// `set_zoom` clamps into `[MIN_ZOOM, max]` itself, so handing it a smaller
    /// number cannot produce an absurd view. What it *can* produce is a zoom
    /// that did not move, and that is the case this function must not report as
    /// handled: an unmoved zoom means the page still cannot be drawn, and
    /// returning `true` would hide the one failure the operator would have no
    /// way to diagnose — a permanently blank sheet with no sentence anywhere.
    /// Hence the `<` comparison *after* the clamp rather than a prediction
    /// before it.
    fn learn_raster_ceiling(&mut self, ctx: &egui::Context, key: RenderKey) -> bool {
        let page = key.page();
        let Some(ceiling) =
            self.raster_ceiling
                .learn(page, key.raster_scale(), self.page_epochs.get(page))
        else {
            return false;
        };
        let pixels_per_point = crate::viewer::sane_pixels_per_point(ctx.pixels_per_point());
        let target = ceiling / pixels_per_point;
        let before = self.view.zoom;
        // `set_zoom(target, target)` rather than a ceiling computed from
        // `zoom_ceiling`: this IS the ceiling, freshly measured, and asking
        // `zoom_ceiling` for it one statement after teaching it would be the
        // same number through a longer path — with the hazard that the two
        // disagree for one frame if anything else in the expression moved.
        self.view.set_zoom(target, target);
        let moved = self.view.zoom < before;
        // Published because the whole effect of a success here is that nothing
        // happens: no error, no blank page, no further refusal. An absence is
        // the one thing a driven check cannot assert, so the act announces
        // itself. Low cardinality by construction — at most one line per page
        // per edit, because `learn` returns `None` for a repeat.
        let after = self.view.zoom;
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "raster-ceiling-learned page={page} scale={ceiling:.1} zoom={before:.2} to={after:.2} moved={moved}"
            )
        });
        moved
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
    /// Two callers need the same answer and they must never disagree:
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
        // ★★ Why this asks `render_refused` and not `render_error`. It used to
        // read `doc.render_error.is_some() && !stale_discrete`, and that could
        // not fire at all -- `render_error` did not survive the frame it was
        // set in, and `stale_discrete` is unconditionally true once a texture
        // has been nulled, which is the first thing a refusal does. Measured
        // 2026-09-10: 611 spawns and ~573 dead worker threads in one check.
        // The full argument is on `OpenDoc::render_refused`.
        let current_held = doc.render_refused.is_some_and(|(key, epoch)| {
            key == wanted && epoch == doc.page_epochs.get(doc.view.page_index)
        });

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

        // ★★ **The decline, published.** O186, 2026-09-12.
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
