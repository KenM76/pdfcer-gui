//! # `app::settle` — is the picture on screen still a picture of what I am looking at?
//!
//! The per-frame raster decision: what is stale, what to re-rasterize now,
//! what to debounce until a zoom gesture stops, and which of a continuous
//! strip's several visible pages to render next.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/settle.md`.

use std::time::{Duration, Instant};

use self::absorb::Absorb;
use crate::app::PdfcerApp;
use crate::app::state::{OpenDoc, Status};
use crate::viewer;
use pdfcer_gui_base::stripschedule::prefetch::{self, Prefetch};
pub(crate) use pdfcer_gui_base::stripschedule::{StripOrders, ZoomSettle};

/// The other half of the exchange this module schedules: requesting a page,
/// collecting the result, and turning whatever came back into a texture, a
/// backdrop, an ink census or a learned zoom ceiling. Declared here rather
/// than in `render/mod.rs` because nothing outside `settle` calls into it —
/// the same shape as `renderworker.rs` and its `renderworker/key.rs`.
mod absorb;

/// How long a zoom must stop changing before it is committed to a real
/// rasterization.
pub const ZOOM_SETTLE: Duration = Duration::from_millis(150);

/// The [`crate::diag::trace_changed`] slot for *"the page being looked at was
/// not ordered this frame, because nothing could be made of the order"* —
/// O186's third route.
const CURRENT_UNFILLABLE_SLOT: &str = "current-order-unfillable";

/// The [`crate::diag::trace_changed`] slot for *"how many visible neighbour
/// sheets could not be ordered at this zoom"* — O186.
const BEYOND_RASTER_SLOT: &str = "strip-beyond-raster";

impl PdfcerApp {
    /// Decide whether the cached page textures are still valid and, if not,
    /// whether to re-rasterize now or wait for a zoom gesture to settle.
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

        // O23's "see" half needs to know where this page's ink actually
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

        // Step 2 of the priority (see the module header), and it runs before
        // anything is requested: a scroll that changed which page is current
        // changed no *picture*, so the textures are rehomed rather than
        // re-rendered.
        doc.rehome_current_page(wanted);

        // The staleness comparison, and why it is ONE key.
        //
        // "Is the picture on screen still a picture of what the operator is
        // looking at?" is asked of the same `RenderKey` the worker labelled
        // the texture with. The categories below are the key's own, so the
        // policy lives with the type rather than being re-derived here.
        let current = doc.page_texture.as_ref().map(|t| t.key);
        // No texture at all is "stale" in the discrete sense: there is nothing
        // on screen worth waiting to replace.
        // …and an EDIT is a discrete change too, even though it moves no
        // field of the key.
        //
        // The key answers "is this a picture of the right page, at the right
        // scale, with the right annotation stance". It cannot answer "is it a
        // picture of the right *revision*", because an edit changes none of
        // those. That third term is `page_texture_epoch`, and adding it here is
        // what lets `vector_edit` stop nulling the texture — which is what put
        // a blank page on screen after every edit.
        // Per-page since 2026-08-31 (O74). The third term still answers
        // "is it a picture of the right REVISION" — it is now the revision of
        // *this page* rather than of the document, so an edit on sheet 3 no
        // longer re-rasterises the canvas while it is showing sheet 7.
        let stale_edit = doc.page_texture_epoch != doc.page_epochs.get(doc.view.page_index);
        let stale_discrete =
            stale_edit || current.is_none_or(|k| k.discrete_inputs() != wanted.discrete_inputs());
        let stale_scale = current.is_some_and(|k| k.scale_bits() != wanted.scale_bits());
        // …AND WHETHER IT IS A PICTURE OF THE RIGHT PART OF THE PAGE.
        //
        // `OPERATOR_REQUESTS.md` O25. Above the pixmap ceiling a raster covers
        // the visible region rather than the page, so two textures of the same
        // page at the same scale can show *different places*. Without this
        // term a pan requested nothing at all — the old picture was drawn
        // correctly at its own region and slid off, leaving the newly exposed
        // area blank indefinitely. See `RenderKey::same_region`.
        //
        // Grouped with the SCALE rather than with the discrete inputs, and
        // the reason is the same debounce argument that put the scale there: a
        // region changes under a continuous gesture, and a render started on
        // every frame of a drag would be cancelled by the next one — the
        // worker is single-slot — so the operator would pan for a second and
        // receive nothing at the end of it.
        //
        // It is already rate-limited in a way the scale is not:
        // `render::strategy::region_for` snaps to a half-viewport grid, so a
        // region changes at most once per half-screen of travel however
        // smoothly the pointer moves. The debounce is the second limiter, not
        // the only one, which is why the settle interval can stay tuned for
        // zoom without making a pan feel slow.
        let stale_region = current.is_some_and(|k| !k.same_region(&wanted));

        // A PAGE WHOSE PREVIOUS RENDER FAILED MUST NOT BE RETRIED EVERY
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
        // Gated on both spawn sites at once, by wrapping them, rather than
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

    /// **Prune the strip's cache to what is visible, then start at most one
    /// render for it.**
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
        // `retain` no longer takes the visible set, and that is the whole of
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
        // `trace_changed`, and a COUNT rather than a list of page indices, so
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
            // The SCALE is deliberately absent. Including it would make the
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
            // **O186's raster error, at its source.** A strip page is
            // ordered whole-sheet — `OpenDoc::region_for` gives a region to the
            // current page only — so above the renderer's pixmap ceiling there
            // is no order to place. Asking anyway is what painted
            // *"requested raster size 50411508x32619210 is empty or exceeds
            // MAX_PIXMAP_EDGE"* across a neighbouring sheet of the operator's
            // drawing set while the sheet he was reading drew perfectly.
            //
            // A `filter`, deliberately, and not a check inside the `find`'s
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
                    // Per-page (O74): this is the scan that decides which
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::open_local_fixture;

    /// This repository's `fixtures/four-pages.pdf`, whose four sheets differ.
    const FOUR_DIFFERING_SHEETS: &str = "four-pages.pdf";

    /// Page 0 is `2383.937 × 1683.78` pt, so against the rasterizer's
    /// 16,384-pixel edge limit its whole-sheet raster stops fitting at a raster
    /// scale of `16384 / 2383.937 = 6.87`.
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

    /// **The half that must answer yes, and the half that must answer no.**
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

    /// **A region makes the SHEET's size stop mattering — and puts the
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

    /// **A region belonging to another page is not a region.**
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
    #[test]
    fn a_page_past_the_end_is_never_fillable() {
        let doc = doc();
        assert!(!doc.raster_order_fillable(99, BIG_SHEET_FITS));
        assert!(!doc.raster_order_fillable(usize::MAX, BIG_SHEET_FITS));
    }
}
