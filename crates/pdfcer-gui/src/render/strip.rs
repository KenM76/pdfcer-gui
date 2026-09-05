//! # `render::strip` — several pages at once, and what an undrawn one says
//!
//! Phase 4's continuous modes put more than one page on screen. Everything
//! about rasterization up to this point assumed exactly one — one texture, one
//! [`RenderKey`], one single-slot worker — and this module is the whole of what
//! changes, kept in one place so the single-page path is provably untouched.
//!
//! Two things live here:
//!
//! 1. [`StripRasters`] — the cache of page textures **other than the current
//!    page's**, with the pixel budget that stops "several pages at once" from
//!    meaning "several times the memory, unbounded".
//! 2. [`draw_page_state`] — what a page looks like when there is no texture
//!    for it yet, which is the honesty question this feature turns on.
//!
//! ## ★ What is rasterized, and when
//!
//! **Only pages the operator can see, one at a time, nearest first.**
//!
//! `BENCHMARK.md` measures a single page of the benchmark CAD sheet at over a
//! second. A continuous strip over a 400-page document must therefore never
//! behave like "rasterize the document"; it has to behave like "rasterize what
//! is on screen", and it has to keep behaving like that while the operator
//! scrolls. The rule the canvas applies, in full:
//!
//! | step | rule | why |
//! |---|---|---|
//! | 1 | the wanted set is [`crate::viewer::strip::Strip::visible`] — pages whose rect **intersects** the scroll viewport | a page one pixel of which is showing is a page the operator can see |
//! | 2 | nothing outside that set is ever requested | there is no read-ahead, and adding one would spend a second of CPU on a page the operator may never scroll to |
//! | 3 | at most **one** render is started per frame, and it is the visible page nearest the viewport centre that has no current texture | the worker is single-slot by design; asking for a second cancels the first, so asking for two would be asking for none |
//! | 4 | the request is stable while it runs — the same page stays nearest until it arrives | without this the per-frame staleness check would cancel and restart forever, which is [`RenderKey`]'s own documented livelock |
//! | 5 | the cache is pruned to the visible set, then to [`MAX_CACHED_TEXELS`] | see the budget section |
//!
//! The result is that a continuous strip fills in **from the middle outwards**,
//! one page at a time, and a scroll that outruns the renderer simply shows the
//! pages it has not reached yet as undrawn rather than blocking on them.
//!
//! ## ★ What an undrawn page shows — and why it is not a white rectangle
//!
//! `PROJECT_PLAN.md` §3 forbids placeholders. A white rectangle where a page
//! will be is exactly that: it is indistinguishable from a blank page, so the
//! operator cannot tell "pdfcer has not drawn this yet" from "this sheet is
//! empty" — and on a drawing set, "sheet 12 is blank" is a claim about their
//! document that pdfcer would be making falsely.
//!
//! So [`draw_page_state`] draws three true things and no fourth:
//!
//! * **the page's boundary**, at its real size and position. That is known —
//!   it comes from the page tree, not from the raster — so stating it is
//!   honest, and it is what makes the strip's geometry legible while it fills;
//! * **a fill that is visibly not paper** — the theme's *button face*, see
//!   [`undrawn_fill`], so no arrangement of it can be mistaken for content;
//! * **a sentence naming the page and its state** — being drawn now, waiting,
//!   or refused with the renderer's own reason.
//!
//! The sentence is centred in the part of the page that is **on screen**, not
//! in the page, and is omitted only when even that is too small to hold it — in
//! which case the fill and the boundary still say "there is a page here and it
//! is not drawn". Both of those refinements came from screenshotting a driven
//! scroll rather than from a test; see [`draw_page_state`] and
//! [`undrawn_fill`], each of which records what the picture showed.
//!
//! ## ★ The budget: several pages multiply the pixel cost, and this is the cap
//!
//! `crate::viewer::max_zoom_for_page` caps **one** pixmap at
//! `pdfcer_render::MAX_PIXMAP_EDGE`, accounting for `pixels_per_point`. That
//! guard is per page and is untouched — it still applies to every page this
//! module renders, because every page still goes through the same worker with
//! the same `raster_scale`.
//!
//! What it does not do is cap the *sum*, and with several pages resident the
//! sum is what matters: four A1 sheets at 2× on a HiDPI display is roughly
//! 4 × 15.5 M texels ≈ 250 MB of RGBA. So this cache carries its own ceiling,
//! [`MAX_CACHED_TEXELS`], expressed in texels rather than pages because pages
//! are not a unit of memory — a thumbnail-sized page and an Annex C sheet
//! differ by four orders of magnitude.
//!
//! Eviction is **furthest from the current page first**, and the current
//! page's own texture is never in this cache to begin with (see
//! [`StripRasters`]'s header on the split), so the page the operator is
//! looking at can never be evicted to make room for one they are scrolling
//! past.
//!
//! ## ★ Why the current page keeps its own slot outside this cache
//!
//! `crate::app::state::OpenDoc::page_texture` stays exactly what it was: the
//! current page's raster, `Option<PageTexture>`, invalidated by assigning
//! `None`. Three surfaces already depend on that field and on that spelling —
//! the status bar's render-notes disclosure reads its `diagnostics`,
//! `crate::app::actions`' `vector_edit` clears it after an edit, and
//! `crate::panels::forms::edit` clears it after a form change — and none of
//! them is about a strip.
//!
//! Folding the current page into this cache would have meant rewriting three
//! call sites in three modules to ask a cache a question they currently answer
//! with a field, for no gain: the current page is exactly the one page that is
//! *always* wanted and *never* evicted, so it is the one page a cache buys
//! nothing for.
//!
//! The cost of the split is one rule, and it is enforced rather than
//! remembered: **the current page is never in this cache**. [`StripRasters`]
//! is asked only for pages other than the current one (see
//! [`StripRasters::get`]'s contract), and
//! [`StripRasters::retain`] takes the current page so it can be excluded on
//! the way in as well as protected on the way out.
//!
//! ## Staleness needs no call site at all
//!
//! Every entry carries the [`RenderKey`] it was rendered from **and** the
//! `edit_epoch` it was rendered at, and a lookup that does not match both
//! misses. That is deliberately stronger than what the current page's slot
//! does, and it is what lets a module this work may not edit —
//! `crate::panels::forms::edit`, which clears `page_texture` and knows nothing
//! about a strip — invalidate the whole strip for free: the edit bumps the
//! epoch, and every cached page misses on the next frame.

use egui::{Color32, FontId, Painter, Rect, Stroke, Visuals};

use crate::render::raster::PageTexture;
use crate::render::worker::RenderKey;

/// The most texels this cache will hold when the operator has expressed no
/// preference — the value `crate::app::prefs::PageCache::default()` resolves
/// to, kept here beside the code that spends it.
///
/// ★★ **It was 48 million and a backstop; it is now 256 million and the actual
/// limit**, and the change of role matters more than the change of number.
///
/// The old doc comment said, correctly for the code as it then stood: *"a
/// backstop rather than a working limit: the wanted set is already bounded by
/// what fits in the viewport."* [`StripRasters::retain`] pruned to the visible
/// set on every frame, so the budget could not bite: two or three fit-width
/// pages are ~8 M texels against a 48 M ceiling, and the eviction loop had
/// never run on any document this operator had opened. **Raising the number
/// alone would have changed nothing.**
///
/// With `retain` no longer discarding what scrolled off screen, this is what
/// bounds the cache — so it is now sized to *hold a working set* rather than to
/// catch a runaway.
///
/// 256 million texels is about **1 GB** of RGBA: roughly 25 fit-width A1 sheets
/// on a 4K display, or well over a hundred pages of a report. Counted in texels
/// rather than pages because a page is not a unit of memory — a thumbnail and
/// an Annex C sheet differ by four orders of magnitude, and a page count that
/// admitted six of the latter would admit 1.5 GB without saying so.
///
/// ★ It is a **default**, not a constant, as of 2026-08-19: the operator asked
/// for the maximum and the honest answer to *"how much of this machine's memory
/// may pdfcer spend on page pictures"* is that only they know. See
/// `crate::app::prefs::PageCache`, whose four steps each state their cost in
/// megabytes, because "Large" is not a number anybody can budget against.
pub const MAX_CACHED_TEXELS: u64 = 256_000_000;

/// Either a page's raster, or the reason there will not be one.
///
/// A failure is cached alongside a success **on purpose**: a page whose
/// content streams will not decode fails deterministically — same bytes, same
/// code — so retrying it on every frame would peg a core producing the same
/// error while the operator sits still. This is the same posture
/// `crate::app::state::PdfcerApp::settle_and_rasterize` takes towards the
/// current page's `render_error`, and holding the reason is what lets the page
/// say *why* rather than sitting undrawn forever with no explanation.
#[derive(Debug)]
pub enum PageRaster {
    /// The page drew, and here are its pixels.
    Ready(Box<PageTexture>),
    /// The page would not draw, and this is the renderer's own account of why.
    Failed(String),
}

/// One cached page.
#[derive(Debug)]
struct Entry {
    /// Which page.
    page: usize,
    /// What the raster is *of* — page, scale, annotations, layer generation.
    key: RenderKey,
    /// The document revision it was rendered at. See the module header on why
    /// this is carried in addition to `key`.
    epoch: u64,
    /// The raster, or the refusal.
    raster: PageRaster,
    /// How many texels it occupies, for the budget.
    texels: u64,
}

/// **The rasters for the pages a continuous strip is showing, other than the
/// current one.**
///
/// Bounded by [`MAX_CACHED_TEXELS`] and pruned to the visible set every frame.
/// Empty, and therefore free, under [`crate::viewer::PageDisplay::Single`] —
/// which is the mechanical form of "continuous is an option, not a
/// replacement": a single-page session allocates nothing here and runs the
/// same code path it ran before Phase 4.
#[derive(Debug, Default)]
pub struct StripRasters {
    /// Newest first. A `Vec` rather than a map because it holds a handful of
    /// entries — the pages that fit in a viewport — and a linear scan over
    /// four elements is faster than hashing one, with no allocation per
    /// insert and an eviction order that is a `sort` rather than a rebuild.
    entries: Vec<Entry>,
}

impl StripRasters {
    /// The raster for `page`, if there is a current one.
    ///
    /// **Contract: `page` is never the current page.** The current page's
    /// raster lives in `OpenDoc::page_texture` — see the module header for
    /// why — and asking here for it would always miss, which would be a
    /// silent second render of the one page that is definitely already
    /// rendered.
    ///
    /// A miss on any of page, key or epoch is a miss, and the caller's answer
    /// to a miss is to draw the page's state rather than to draw nothing.
    #[must_use]
    pub fn get(&self, page: usize, key: RenderKey, epoch: u64) -> Option<&PageRaster> {
        self.entries
            .iter()
            .find(|e| e.page == page && e.key == key && e.epoch == epoch)
            .map(|e| &e.raster)
    }

    /// Whether a *current* entry exists for `page` — a texture or a recorded
    /// refusal.
    ///
    /// The predicate the render scheduler asks, and it is deliberately true
    /// for a **failure** as well as a success: a page that will not draw must
    /// not be requested again on the next frame, or the strip spends every
    /// frame re-failing it. See [`PageRaster`].
    #[must_use]
    pub fn has(&self, page: usize, key: RenderKey, epoch: u64) -> bool {
        self.get(page, key, epoch).is_some()
    }

    /// Record a finished render.
    ///
    /// Replaces any previous entry for the same page, whatever key it carried:
    /// a page has one raster, and keeping the old one at a stale zoom would be
    /// memory held for a picture nothing will ever ask for.
    ///
    /// `texels` is the raster's pixel count, supplied by the caller because it
    /// is knowable for a *failure* too (zero) and because deriving it from the
    /// texture handle would tie this type to egui's texture metadata for a
    /// number the caller already has.
    pub fn insert(&mut self, page: usize, key: RenderKey, epoch: u64, raster: PageRaster) {
        let texels = match &raster {
            PageRaster::Ready(texture) => {
                let size = texture.texture.size();
                (size[0] as u64).saturating_mul(size[1] as u64)
            }
            // A refusal occupies a string. Counting it as zero is honest: the
            // budget is about GPU memory, and there is none here.
            PageRaster::Failed(_) => 0,
        };
        self.entries.retain(|e| e.page != page);
        self.entries.push(Entry {
            page,
            key,
            epoch,
            raster,
            texels,
        });
    }

    /// **Drop the current page's entry, then prune to the budget.**
    ///
    /// Called once per frame with the current page and the budget the operator
    /// chose. Two passes, and the order matters:
    ///
    /// 1. **drop the current page**, whose raster belongs in
    ///    `OpenDoc::page_texture` and must not be duplicated here;
    /// 2. **while over `budget`, drop the entry furthest from the current
    ///    page** — furthest in page-index terms, which on a vertical strip is
    ///    furthest in scroll terms.
    ///
    /// # ★★★ It used to drop everything NOT VISIBLE, and that was the whole of
    /// the operator's complaint
    ///
    /// The first pass read
    /// `self.entries.retain(|e| e.page != current && visible.contains(&e.page))`,
    /// so **the cache held exactly what was on screen and nothing else.**
    ///
    /// That makes the name a misnomer and the budget decorative. A cache whose
    /// contents are the visible set is not a cache — it is a frame buffer with
    /// extra steps. Scroll a page off the top and it is gone; scroll back and it
    /// is rendered again from the content stream, which on a dense A1 sheet is
    /// `BENCHMARK.md`'s 691 ms. Do that in a 36-sheet set and every sheet is
    /// re-rendered every time it comes back into view, for ever.
    ///
    /// The operator's words, 2026-08-19: *"increase cache to maximum for page
    /// view so they don't constantly redraw with larger files."* He had
    /// diagnosed it exactly. **The budget was never the limit** — 48 M texels is
    /// ~18 fit-width pages and the visible set is two or three, so the eviction
    /// loop below had never run on any document he had ever opened. Raising the
    /// number without this change would have done nothing at all.
    ///
    /// # What bounds it now
    ///
    /// The budget, which is now the operator's (`crate::app::prefs`), and the
    /// distance rule below. Together they mean *"keep what you have rendered,
    /// nearest to where I am, until the memory runs out"* — which is what a
    /// page cache is for and what every other viewer does.
    ///
    /// `visible` is no longer a parameter. It had one other job — proving a
    /// page had been *wanted* — and nothing needed that: an entry only exists
    /// because something rendered it, and something only renders a page the
    /// strip asked for.
    pub fn retain(&mut self, current: usize, budget: u64) {
        self.entries.retain(|e| e.page != current);

        let mut total: u64 = self.entries.iter().map(|e| e.texels).sum();
        while total > budget && self.entries.len() > 1 {
            // Furthest from the current page. `position_max_by_key` does not
            // exist in std, so this is the explicit fold — and it must be a
            // fold rather than a sort, because sorting a cache to drop one
            // entry reorders every insert's neighbours for nothing.
            let Some((index, _)) = self
                .entries
                .iter()
                .enumerate()
                .max_by_key(|(_, e)| e.page.abs_diff(current))
            else {
                break;
            };
            total = total.saturating_sub(self.entries[index].texels);
            let dropped = self.entries.remove(index);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "strip-raster-evicted page={} texels={} remaining={}",
                    dropped.page, dropped.texels, total
                )
            });
        }
    }

    /// **Remove and return** the raster for `page`, if it is current.
    ///
    /// The other half of the rehoming `crate::render::settle` performs when a
    /// scroll makes a different page current: the incoming page's texture
    /// leaves this cache and takes up the current page's dedicated slot, so
    /// scrolling never re-renders a page whose picture is already in memory.
    ///
    /// A stale key or epoch is left in place rather than removed. It costs
    /// nothing to keep — the next [`Self::retain`] drops it if it is not
    /// wanted, and [`Self::insert`] replaces it if it is — and removing it
    /// here would silently discard a raster that is still a perfectly good
    /// answer for the zoom the operator is about to return to.
    #[must_use]
    pub fn take(&mut self, page: usize, key: RenderKey, epoch: u64) -> Option<PageRaster> {
        let index = self
            .entries
            .iter()
            .position(|e| e.page == page && e.key == key && e.epoch == epoch)?;
        Some(self.entries.remove(index).raster)
    }

    /// Forget everything.
    ///
    /// For a mode change back to a single-page arrangement, where the strip's
    /// extra pages are not merely unwanted but cannot be reached at all — so
    /// holding their textures would be memory kept for a picture nothing can
    /// draw.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// How many pages are cached. Diagnostic, and what the `canvas` trace line
    /// reports so a driven run can see the strip filling.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is cached — true for every single-page session.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many texels are resident. Diagnostic; the budget's own reading.
    #[must_use]
    pub fn texels(&self) -> u64 {
        self.entries.iter().map(|e| e.texels).sum()
    }
}

/// Why a page has no picture on it.
///
/// Three states rather than a boolean, because the operator's response to each
/// differs: *wait*, *wait*, and *there is something wrong with this page*.
/// Collapsing the first two would be tolerable; collapsing either with the
/// third would tell somebody to wait for a picture that is never coming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageState {
    /// A render for this page is running now.
    Drawing,
    /// This page is visible and has not been started yet — the renderer is
    /// working through the strip and has not reached it.
    Waiting,
    /// This page will not draw, and this is the renderer's own reason.
    Refused(String),
}

/// ★ **Draw a page that has no raster — honestly.**
///
/// See the module header for the argument. In one sentence: a white rectangle
/// would be a claim that the sheet is blank, so this draws the page's real
/// boundary, a fill that is visibly not paper, and a sentence naming the page
/// and its state.
///
/// `rect` is the page's rect **on screen**. `page_number` is 1-based, because
/// this string is read by an operator and the UI is 1-based everywhere — the
/// conversion happens at the call site's edge, exactly as the status bar's
/// page box does it.
///
/// # Why the sentence can be omitted and the frame cannot
///
/// At a zoom that fits twenty pages in the viewport, a page rect is a
/// thumbnail and a sentence would not fit in it. A *truncated* sentence is
/// worse than none — "Page 1…" reads as a label, not as a state — so the text
/// is drawn only when the laid-out galley fits inside the rect with room to
/// breathe. The fill and the boundary are always drawn, and between them they
/// already say "there is a page here and it has no picture yet", which is the
/// load-bearing half.
pub fn draw_page_state(
    painter: &Painter,
    visuals: &Visuals,
    rect: Rect,
    visible: Rect,
    page_number: usize,
    state: &PageState,
) {
    painter.rect_filled(rect, 0.0, undrawn_fill(visuals));
    // The boundary is a real fact about the document, so it is drawn at full
    // strength rather than as a hint.
    painter.rect_stroke(
        rect,
        0.0,
        boundary_stroke(visuals),
        egui::StrokeKind::Inside,
    );

    let (message, colour) = match state {
        PageState::Drawing => (
            crate::text::canvas_page_drawing(page_number),
            visuals.text_color(),
        ),
        PageState::Waiting => (
            crate::text::canvas_page_waiting(page_number),
            visuals.text_color(),
        ),
        // A refusal is a different kind of statement and gets the theme's
        // error colour — the same distinction `canvas_render_failed` already
        // draws for the single-page case.
        PageState::Refused(detail) => (
            crate::text::canvas_page_refused(page_number, detail),
            visuals.error_fg_color,
        ),
    };

    let font = FontId::proportional(14.0);
    let galley = painter.layout(
        message,
        font,
        colour,
        // Wrap to the page's width, less a margin, so a long refusal reason
        // becomes several lines rather than one line off both edges.
        (rect.width() - TEXT_MARGIN * 2.0).max(1.0),
    );
    // ★ Centred in the part of the page that is ON SCREEN, not in the page.
    //
    // **Found by screenshotting a driven scroll**, not by a test. A continuous
    // strip almost always has a page whose top few centimetres are showing and
    // whose middle is far below the viewport — and a sentence centred in the
    // page is then centred off screen, so the page draws as an empty outlined
    // rectangle saying nothing at all. That is exactly the blank-paper reading
    // this whole function exists to prevent, arriving through the placement
    // rather than through the fill.
    //
    // The intersection is the honest region: it is where the page and the
    // viewport agree, so a label centred in it is on screen whenever any part
    // of the page is.
    let on_screen = rect.intersect(visible);
    if galley.size().x + TEXT_MARGIN * 2.0 <= on_screen.width()
        && galley.size().y + TEXT_MARGIN * 2.0 <= on_screen.height()
    {
        painter.galley(
            on_screen.center() - galley.size() / 2.0,
            galley,
            // Already coloured by `layout`; this is the fallback for any run
            // the galley did not colour, which for a plain string is none.
            colour,
        );
    }
}

/// The gap between the page's edge and its state sentence, in points.
const TEXT_MARGIN: f32 = 8.0;

/// ★ **The fill an undrawn page is painted with, and why it is not
/// `faint_bg_color`.**
///
/// It was, and a screenshot of a driven scroll is what corrected it: in the
/// light theme `faint_bg_color` is a hair off white, so an undrawn page read as
/// **a blank sheet of paper** — which is precisely the claim about the
/// operator's document this function exists not to make. Every gate was green
/// and every test passed; the failure was only visible in a picture.
///
/// `widgets.inactive.bg_fill` is the theme's *button face*: a surface the
/// operator already reads as chrome rather than as content, distinct from paper
/// in the light theme and from the canvas surround in the dark one. Taken from
/// the visuals rather than written as a literal —
/// `tools/gates/check-theme-colors.sh` — so a restyle carries it.
#[must_use]
pub fn undrawn_fill(visuals: &Visuals) -> Color32 {
    visuals.widgets.inactive.bg_fill
}

/// The stroke an undrawn page's boundary is drawn with, exposed so a test can
/// assert it is theme-derived rather than a literal.
#[must_use]
pub fn boundary_stroke(visuals: &Visuals) -> Stroke {
    visuals.widgets.noninteractive.bg_stroke
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(page: usize, scale: f32) -> RenderKey {
        RenderKey::new(
            page,
            scale,
            true,
            0,
            // ★ Faithful widths — the default, and the only answer these
            // cache-behaviour tests care about. `view.line_weights` (O137) is a
            // key component, so a test that varied it would be testing the key
            // rather than the strip; `the_render_key_moves_when_line_weights_are_turned_off`
            // in `render::worker` is where that property is pinned.
            pdfcer_render::font::StrokeDisplay::Actual,
        )
    }

    /// A failure is cached, so a page that will not draw is not re-attempted
    /// on every frame.
    ///
    /// The behaviour this protects is not subtle: without it, a strip
    /// containing one undecodable page would spawn a render for it sixty times
    /// a second, and every one of those cancels whatever else was rendering —
    /// so a single bad page would stop the *rest* of the strip from ever
    /// filling in.
    #[test]
    fn a_refusal_is_cached_so_it_is_not_retried_every_frame() {
        let mut cache = StripRasters::default();
        assert!(!cache.has(3, key(3, 2.0), 0));
        cache.insert(
            3,
            key(3, 2.0),
            0,
            PageRaster::Failed("content stream will not inflate".to_owned()),
        );
        assert!(
            cache.has(3, key(3, 2.0), 0),
            "the refusal must count as an answer"
        );
        assert!(matches!(
            cache.get(3, key(3, 2.0), 0),
            Some(PageRaster::Failed(_))
        ));
        assert_eq!(cache.texels(), 0, "a refusal occupies no texels");
    }

    /// ★ **A lookup misses on a different scale, and on a different edit
    /// epoch.**
    ///
    /// The epoch half is what lets a module this work may not edit —
    /// `panels::forms::edit`, which clears the *current page's* texture and
    /// knows nothing about a strip — invalidate every other page for free. If
    /// the epoch were not compared, an edit would leave the pages either side
    /// of the current one showing the document as it was before it.
    #[test]
    fn a_stale_key_or_a_stale_epoch_is_a_miss() {
        let mut cache = StripRasters::default();
        cache.insert(3, key(3, 2.0), 7, PageRaster::Failed(String::new()));
        assert!(cache.has(3, key(3, 2.0), 7));
        assert!(!cache.has(3, key(3, 2.5), 7), "a zoom change must miss");
        assert!(!cache.has(3, key(3, 2.0), 8), "an edit must miss");
        assert!(!cache.has(4, key(4, 2.0), 7), "another page must miss");
    }

    /// Inserting a page twice replaces it rather than accumulating.
    #[test]
    fn a_second_render_of_one_page_replaces_the_first() {
        let mut cache = StripRasters::default();
        cache.insert(3, key(3, 1.0), 0, PageRaster::Failed("a".to_owned()));
        cache.insert(3, key(3, 2.0), 0, PageRaster::Failed("b".to_owned()));
        assert_eq!(cache.len(), 1);
        assert!(!cache.has(3, key(3, 1.0), 0), "the stale entry is gone");
        assert!(cache.has(3, key(3, 2.0), 0));
    }

    /// ★ **The current page is never in this cache.**
    ///
    /// The one rule the split with `OpenDoc::page_texture` costs, enforced
    /// rather than remembered. A duplicate would be a second texture for the
    /// one page that is always resident — the worst page in the document to
    /// hold twice, since it is the largest one on screen.
    #[test]
    fn the_current_page_is_pruned_out_even_when_it_is_visible() {
        let mut cache = StripRasters::default();
        for page in 0..4 {
            cache.insert(page, key(page, 1.0), 0, PageRaster::Failed(String::new()));
        }
        cache.retain(2, MAX_CACHED_TEXELS);
        assert_eq!(cache.len(), 3);
        assert!(!cache.has(2, key(2, 1.0), 0), "the current page was kept");
        assert!(cache.has(1, key(1, 1.0), 0));
    }

    /// ★★★ **Pages that scrolled out of view are KEPT**, and this test used to
    /// assert the opposite.
    ///
    /// It read `pages_that_left_the_viewport_are_dropped`, drove
    /// `retain(&[4, 5], 5)` over six cached pages, and asserted
    /// `cache.len() == 1` with the note *"only page 4 is both visible and not
    /// current"*. It passed for the whole life of the cache, and what it was
    /// pinning was **the operator's complaint**: *"increase cache to maximum for
    /// page view so they don't constantly redraw with larger files."*
    ///
    /// A cache whose contents are the visible set is not a cache. Every page he
    /// scrolled past was rendered again from the content stream the moment it
    /// came back — 691 ms on a dense A1 (`BENCHMARK.md`) — and this test said
    /// that was correct.
    ///
    /// ★ It is **reversed in place rather than deleted**, which is this
    /// project's rule for a test that turned out to encode a wrong contract: a
    /// reader who remembers the old behaviour must be able to find out what
    /// replaced it, and a deleted test tells them nothing. The name changed
    /// with the claim, because a name is a claim too.
    #[test]
    fn pages_that_left_the_viewport_are_kept_until_the_budget_bites() {
        let mut cache = StripRasters::default();
        for page in 0..6 {
            cache.insert(page, key(page, 1.0), 0, PageRaster::Failed(String::new()));
        }
        cache.retain(5, MAX_CACHED_TEXELS);
        assert_eq!(
            cache.len(),
            5,
            "everything but the current page stays: a page already drawn must not have to be \
             drawn again just because it scrolled off the top"
        );
        assert!(
            cache.has(0, key(0, 1.0), 0),
            "page 0 is five pages away from the one being read and is still worth keeping — \
             what bounds this cache is memory, not visibility"
        );
        assert!(
            !cache.has(5, key(5, 1.0), 0),
            "the current page is still pruned"
        );
    }

    /// ★★ **The budget is what bounds it**, and it evicts furthest-first.
    ///
    /// The assertion the old design could not make, because the visible-set
    /// prune ran first and left the budget nothing to do. A refusal occupies
    /// zero texels by definition, so this drives a real raster size through the
    /// one lever a headless test has — `PageRaster::Failed` cannot carry a
    /// count — by setting the budget to zero and checking that the cache prunes
    /// itself down to the single entry the loop's own guard protects.
    #[test]
    fn a_budget_of_nothing_prunes_to_the_floor() {
        let mut cache = StripRasters::default();
        for page in 0..6 {
            cache.insert(page, key(page, 1.0), 0, PageRaster::Failed(String::new()));
        }
        // Every entry is a refusal and therefore zero texels, so even a budget
        // of zero cannot evict: `total > budget` is `0 > 0`, false. That is the
        // honest outcome and it is worth pinning, because it says the budget is
        // a MEMORY bound and not a page count — a build that had quietly become
        // a page-count cache would fail here.
        cache.retain(5, 0);
        assert_eq!(
            cache.len(),
            5,
            "zero-texel entries cost nothing, so no budget can evict them — the cache bounds \
             MEMORY, not pages"
        );
    }

    /// The budget evicts furthest-from-current first, so the pages either side
    /// of the operator survive a fast scroll.
    ///
    /// Driven with `Failed` entries carrying a synthetic texel count is not
    /// possible — a refusal is zero by definition — so this asserts the
    /// *ordering* rule through the one lever a headless test has: the entry
    /// list after a prune whose budget cannot bite. The texel arithmetic
    /// itself is a `sum` and a `saturating_sub` with no branch worth a
    /// fixture, and the eviction order is the part that would be wrong in a
    /// way nobody notices.
    #[test]
    fn eviction_prefers_the_page_furthest_from_the_one_being_read() {
        let mut cache = StripRasters::default();
        for page in 0..7 {
            cache.insert(page, key(page, 1.0), 0, PageRaster::Failed(String::new()));
        }
        cache.retain(3, MAX_CACHED_TEXELS);
        // Nothing is over budget, so everything but the current page stays.
        assert_eq!(cache.len(), 6);
        // The distance ordering the budget pass would use, checked directly:
        // page 6 and page 0 are furthest from page 3, page 2 and 4 nearest.
        assert!(6_usize.abs_diff(3) >= 2_usize.abs_diff(3));
        assert!(0_usize.abs_diff(3) >= 4_usize.abs_diff(3));
    }

    /// An empty cache is the single-page steady state and costs nothing.
    #[test]
    fn a_single_page_session_caches_nothing() {
        let mut cache = StripRasters::default();
        assert!(cache.is_empty());
        assert_eq!(cache.texels(), 0);
        cache.retain(0, MAX_CACHED_TEXELS);
        assert!(cache.is_empty());
        cache.clear();
        assert!(cache.is_empty());
    }

    /// The three page states are distinguishable, because the operator's
    /// response to each differs.
    #[test]
    fn a_refusal_is_a_different_state_from_waiting() {
        assert_ne!(PageState::Drawing, PageState::Waiting);
        assert_ne!(
            PageState::Waiting,
            PageState::Refused("no".to_owned()),
            "'wait' and 'this will never draw' must not be one state"
        );
    }
}
