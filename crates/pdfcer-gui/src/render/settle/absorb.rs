//! # `render::settle::absorb` — what to do with the picture that came back
//!
//! The parent module decides what the picture *should* be: what is stale, what
//! to re-rasterize now, what to debounce, which strip page is next. This one
//! handles the other half of that exchange — handing a page to the worker,
//! collecting the result, and turning whatever came back into a cached texture,
//! a promoted backdrop, an ink census or a learned zoom ceiling.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/render/settle/absorb.md`.

use crate::app::state::OpenDoc;
use crate::render::pressure::Surface;
use crate::render::raster;
use crate::render::strip::PageRaster;
use crate::render::worker::{RefusalKind, RenderKey, RenderOutcome};

impl OpenDoc {
    /// Hand a page to the worker and, if it beats the in-frame budget, absorb
    /// the result immediately.
    pub(super) fn rasterize(&mut self, ctx: &egui::Context, page_index: usize, raster_scale: f32) {
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
        // Record WHAT IS BEING ASKED FOR before the inline wait.
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
    fn absorb_render(&mut self, ctx: &egui::Context, result: RenderOutcome) {
        match result {
            Ok(pixels) => {
                let page = pixels.key.page();
                let key = pixels.key;
                // A page blended in ink, recorded before anything else is done
                // with the result.
                //
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
                let texture = raster::texture_from_pixels(ctx, Surface::Canvas, &pixels);
                if page == self.view.page_index {
                    // **PROMOTE IT TO THE BACKDROP** if it is a small
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
                    // The handle is CLONED, not copied: at a fit zoom the
                    // backdrop and the live texture are the same pixels and
                    // cost nothing, and they only diverge once the operator
                    // zooms past it.
                    if key.region().is_none() && raster::within_base_budget(&pixels) {
                        self.base_texture = Some(texture.clone());
                        // Per-page (O74), like the live texture below it.
                        // The backdrop is a picture of ONE page, so an edit on
                        // another sheet has nothing to say about it.
                        self.base_texture_epoch = self.page_epochs.get(page);
                        // Published, because the backdrop is invisible when
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
                    // Per-page (O74): an edit on sheet 3 must not
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
                        // Per-page (O74) — see this module's note on the
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
                // A failure carries no key — `RenderWorker` reports the
                // refusal alone — so it is attributed to whatever the worker
                // was rendering. That is exactly the page it is about: the
                // worker is single-slot, and `render_in_flight` was read
                // *before* the poll took the slot (see `poll_render`). Without
                // that reading, a strip page that would not draw would blank
                // the whole canvas by landing in `render_error`.
                //
                // **The refusal carries a KIND as well as a sentence** — O186.
                // See [`RefusalKind`]: the whole of this arm's new behaviour
                // turns on `BeyondRaster`, and none of it may turn on reading
                // the sentence, which is a string from `crate::text` that the
                // operator is free to have reworded.
                match self.render_in_flight.take() {
                    Some(key) if key.page() != self.view.page_index => {
                        // **A neighbour's limit is still a FACT about that
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
                        // Pulling the zoom back here instead would be the
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
                            // A refusal is filed at the same per-page number
                            // as a picture, so an edit to that page gets it a
                            // second attempt and an edit elsewhere does not.
                            self.page_epochs.get(key.page()),
                            PageRaster::Failed(refusal.message),
                        );
                    }
                    slot => {
                        // **O186's FIRST THREE CLAUSES, and they are one
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
                        // * and the operator ends this frame at a zoom the page
                        //   is believed able to draw. That is what
                        //   [`Self::learn_raster_ceiling`] returns, and it is a
                        //   question about the resulting VIEW rather than about
                        //   the event.
                        //
                        // **Where he stands, not what this refusal
                        // taught** — O218: *"sometimes when I zoom in I still
                        // get the error … instead of the rasterizer just
                        // stopping at the last zoom level that it
                        // accomplished."*
                        //
                        // A refusal is a fact about the scale that was
                        // *ordered*, and a render at a deep scale takes long
                        // enough that by the time it is refused the operator can
                        // be nowhere near it — he wheeled in fast, the deep
                        // raster was refused, and he has already wheeled back
                        // out to a zoom that draws perfectly. Two conditions
                        // that stood here painted the sentence across exactly
                        // that frame, and both for the same reason:
                        //
                        // * *the ceiling is news.* A refusal from above a
                        //   ceiling already learned teaches nothing, and `None`
                        //   was read as *"the clamp already ran and did not
                        //   hold"* — where it far more often means *"this
                        //   refusal is stale"*.
                        // * *the zoom moved.* A clamp that finds him already
                        //   below the ceiling cannot move him. Being below the
                        //   ceiling is the state the clamp exists to produce,
                        //   so reaching it by another route was scored as a
                        //   failure to reach it at all.
                        //
                        // ⇒ Both resolve correctly once the predicate is
                        // *is he under the ceiling*. The genuinely stuck case
                        // is unchanged and still falls through: a ceiling below
                        // `viewer::MIN_ZOOM` means there is no zoom this page
                        // can be drawn at, and a blank canvas with no sentence
                        // anywhere is the one outcome worse than the sentence.
                        //
                        // When `handled`, `render_error` is deliberately NOT
                        // set and `page_texture` is deliberately NOT cleared.
                        // Both are the ordinary, correct responses to a failed
                        // render and both are wrong here:
                        //
                        // * the sentence is the thing he asked us to stop
                        //   painting across his drawing, and its replacement is
                        //   `crate::app::status::rasterstop` on the bottom bar
                        //   — his fourth clause;
                        // * the texture on screen is a picture of this page at
                        //   a zoom under the ceiling — either because the clamp
                        //   has just brought him there or because he was
                        //   already there. Clearing it would blank the canvas
                        //   at the exact moment the view became drawable, and
                        //   *"can still function"* is the clause that forbids
                        //   it.
                        let handled = refusal.kind == RefusalKind::BeyondRaster
                            && slot.is_some_and(|key| self.learn_raster_ceiling(ctx, key));
                        if handled {
                            return;
                        }
                        // THE MEMO THAT STOPS THE RETRY STORM.
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
    fn learn_raster_ceiling(&mut self, ctx: &egui::Context, key: RenderKey) -> bool {
        let page = key.page();
        let epoch = self.page_epochs.get(page);
        let news = self
            .raster_ceiling
            .learn(page, key.raster_scale(), epoch)
            .is_some();
        // The ceiling IN FORCE, which is a different question from what this
        // refusal taught. `learn` declines a repeat as well as a degenerate
        // scale, and only the second of those means nothing is known.
        let Some(ceiling) = self.raster_ceiling.for_page(page, epoch) else {
            return false;
        };
        // The whole of `raster_density`, not the display density alone — the
        // ceiling is a raster scale and `raster_scale` multiplied the operator's
        // render quality into it. O218.
        let target = crate::viewer::zoom_for_raster_scale(
            ceiling,
            ctx.pixels_per_point(),
            self.prefs.render_quality,
        );
        let before = self.view.zoom;
        // `set_zoom(target, target)` rather than a ceiling computed from
        // `zoom_ceiling`: this IS the ceiling, freshly read, and asking
        // `zoom_ceiling` for it one statement later would be the same number
        // through a longer path — with the hazard that the two disagree for one
        // frame if anything else in the expression moved.
        if before > target {
            self.view.set_zoom(target, target);
        }
        let after = self.view.zoom;
        // The verdict the caller acts on: `false` only when the clamp bottomed
        // out at `MIN_ZOOM` still above the ceiling.
        let under = after <= target;
        // Published because the whole effect of a success here is that nothing
        // happens: no error, no blank page, no further refusal. An absence is
        // the one thing a driven check cannot assert, so the act announces
        // itself.
        //
        // `moved` and `under` both appear and they answer differently.
        // `moved=false under=true` is the stale refusal this function exists to
        // stop misreading, and a check seeing only one of them would have to
        // guess which state it was in. `news` separates a first measurement from
        // a repeat, which is what bounds the line rate: the ratchet converges
        // downward and a stale refusal arrives once.
        let moved = after < before;
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "raster-ceiling-learned page={page} scale={ceiling:.1} zoom={before:.2} \
                 to={after:.2} moved={moved} under={under} news={news}"
            )
        });
        under
    }

    /// Collect a background render, if one has finished.
    ///
    /// Called once per frame. Returns whether anything was absorbed, so the
    /// caller can request the repaint that draws it.
    pub(super) fn poll_render(&mut self, ctx: &egui::Context) -> bool {
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
}
