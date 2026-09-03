//! # `viewer::strip` — where every page sits, in one coordinate space
//!
//! `GUI_ROADMAP.md` Phase 4.1 asks for *"`ViewState` holds a page **range**
//! rather than one `page_index`"*. This module is that range, expressed as
//! geometry rather than as a pair of indices — because "which pages are on
//! screen" is not a fact the view *holds*, it is a fact that falls out of
//! where the pages are and where the viewport is.
//!
//! ## The one space, and its relationship to the three that already exist
//!
//! [`crate::canvas::mapping`]'s header names three frames: **screen**,
//! **canvas** (page-device points, Y-down, `/Rotate` applied) and **PDF user**
//! (Y-up, un-rotated). This module introduces exactly one more, and names it
//! so it cannot be confused with canvas space:
//!
//! | frame | origin | unit | who speaks it |
//! |---|---|---|---|
//! | **strip** | the top-left of the whole laid-out run of pages | logical points **at the current zoom** | this module, the scroll area's content box |
//! | **canvas** | one page's top-left | PDF user-space units (zoom 1.0) | `PageMapping`, the provider, the raster |
//!
//! A [`Placement`] is the bridge: `rect` is where one page sits **in strip
//! space**, already multiplied by the zoom, which is exactly the rect
//! `PageMapping::new` wants as its `image_rect` once the strip's own screen
//! origin has been added. Nothing downstream ever converts between strip and
//! canvas space by hand; it asks for a `Placement` and builds a mapping from
//! it.
//!
//! ## ★ Single page is a one-row strip, and that is load-bearing
//!
//! The operator's constraint is that continuous scroll is *an option, not a
//! replacement* — see [`super::display`]'s header. The way that constraint is
//! honoured mechanically rather than by care is that
//! [`PageDisplay::Single`] produces a strip whose row set is **the current
//! page alone**, so:
//!
//! * `size()` is the page's drawn size — identical to the `display_size` the
//!   canvas computed before Phase 4;
//! * the page's rect is `(0,0)..size` — so the scroll range, the centring
//!   margin, the pan clamp and the zoom anchor are all the arithmetic they
//!   already were;
//! * there is no row gap, because there is one row.
//!
//! [`tests::single_page_reproduces_the_pre_phase_4_geometry_exactly`] asserts
//! that as an equality against the old expression rather than as a comment. A
//! change that gives `Single` a gap, a margin or a scroll range it did not have
//! is a test failure, which is the only way a "do not degrade the default"
//! instruction survives contact with a refactor.
//!
//! ## Cost, and why there is no cache
//!
//! [`Strip::new`] is one pass over the rows it lays out, with one `Vec`
//! allocation. For [`PageDisplay::Single`] and [`PageDisplay::Facing`] that is
//! one row — a two-element allocation, per frame, which is free. For the
//! continuous modes it is one row per page (or per spread) in the **whole
//! document**, because a scroll range is a property of the whole document and
//! cannot be computed from a window of it.
//!
//! That is O(n) per frame, and it is deliberately not cached. The measured
//! shape of the work is a `page_device_geometry` call and about a dozen float
//! operations per row; a cache would need a key over (page vector, display
//! mode, zoom) held behind a `RefCell` on `OpenDoc`, invalidated on every zoom
//! notch — i.e. rebuilt on exactly the frames that are already the expensive
//! ones. See the module's own measurement note in `BENCHMARK.md` terms: the
//! rasterization this feature schedules is measured in **hundreds of
//! milliseconds**, and this is measured in **microseconds**. Optimising the
//! second while the first exists would be optimising the wrong thing, and the
//! cache would be a staleness hazard bought with nothing.
//!
//! **The honest limit, stated rather than discovered:** strip coordinates are
//! `f32`, and a strip is as tall as the document. At the top of the zoom ladder
//! (8×) a 1,000-page US Letter document is ~6.4 million points tall, where
//! `f32` resolves to about 0.5 pt — sub-pixel, but no longer exact. Beyond
//! roughly 2,000 pages at maximum zoom the accumulated row tops begin to
//! quantise visibly. The fix, if it is ever wanted, is `f64` row tops converted
//! to `f32` per placement, and this paragraph is where that decision would
//! attach. It is not made now because the error at any realistic combination is
//! far below one pixel and because carrying two float widths through the
//! geometry has its own cost in confusion.

use egui::{Pos2, Rect, Vec2, pos2, vec2};
use pdfcer_core::page_tree::Page;

use super::display::PageDisplay;
use super::{max_zoom_for_page, page_extent_pts};

/// The gap between two rows of the strip, in points at zoom 1.0.
///
/// **Scaled by the zoom**, like everything else here, which is what keeps the
/// strip's geometry exactly linear in the zoom and therefore makes the zoom
/// anchor's "measure, then re-place" solve exact rather than approximate. A
/// constant *screen* gap would be more conventional and would make
/// [`crate::canvas::geometry::zoom_anchor_offset`] wrong by the gap on every
/// step, in a way that accumulates over a long scroll.
///
/// 12 points is about a sixth of an inch: wide enough that two white pages do
/// not read as one tall page, narrow enough that scrolling does not feel like
/// it is mostly gap.
pub const ROW_GAP: f32 = 12.0;

/// The gap between the two pages of a facing spread, in points at zoom 1.0.
///
/// Deliberately **smaller** than [`ROW_GAP`], and the asymmetry is the whole
/// point of a spread: the two halves are one opening and must read as a pair,
/// while the next spread is a separate thing. Equal gaps would produce a grid
/// of pages rather than a run of spreads.
pub const SPREAD_GAP: f32 = 6.0;

/// What a fit mode fits, and the highest zoom it may fit to.
///
/// The two facts about the **current row** that are needed *before* the strip
/// can be laid out, because both are inputs to
/// [`crate::viewer::ViewState::apply_fit`] and the strip's geometry depends on
/// the zoom that produces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RowMetrics {
    /// The row's extent in PDF user-space units — see
    /// [`Strip::row_extent`].
    pub extent: (f32, f32),
    /// The tightest per-page raster ceiling in the row — see
    /// [`Strip::row_max_zoom`].
    pub max_zoom: f32,
}

/// The current row's metrics, without laying out the whole strip.
///
/// ★ **This exists to keep the frame to one O(n) pass rather than two.** The
/// fit modes need the row's extent and ceiling in order to decide the zoom, and
/// the strip needs the zoom in order to lay out — so a naive frame would build
/// a strip, read two numbers off it, apply the fit and build it again. On a
/// continuous strip over a large document that doubles the only per-frame cost
/// this feature has.
///
/// A row is at most two pages, so this is O(1) whatever the document's size,
/// and it produces the *same* two numbers [`Strip::row_extent`] and
/// [`Strip::row_max_zoom`] do — asserted by
/// [`tests::row_metrics_agrees_with_the_laid_out_strip`], because two
/// derivations of a fit scale is exactly how "Fit page" and the page it fits
/// come to disagree.
#[must_use]
pub fn row_metrics(
    pages: &[Page],
    display: PageDisplay,
    current: usize,
    pixels_per_point: f32,
) -> RowMetrics {
    let current = super::clamp_page_index(current, pages.len());
    let range = display.pages_in_row(display.row_of(current), pages.len());
    if range.is_empty() {
        // Same fallback `Strip::row_extent` gives an empty strip: something
        // finite for the fit arithmetic to divide by, on a document that draws
        // nothing anyway.
        return RowMetrics {
            extent: (612.0, 792.0),
            max_zoom: super::MAX_ZOOM,
        };
    }
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    let mut max_zoom = super::MAX_ZOOM;
    for (slot, page) in range.take(2).enumerate() {
        let extent = page_extent_pts(&pages[page]);
        if slot > 0 {
            width += SPREAD_GAP;
        }
        width += extent.0;
        height = height.max(extent.1);
        max_zoom = max_zoom.min(max_zoom_for_page(extent, pixels_per_point));
    }
    RowMetrics {
        extent: (width, height),
        max_zoom,
    }
}

/// The metrics a **fit mode** should be computed from.
///
/// # ★ Why this is not always `row_metrics`, and the bug that says so
///
/// Under a continuous mode the current page is **derived from the scroll**
/// (`Strip::page_at_view`, greatest visible area). Feeding that page's row
/// into a per-frame fit closes a loop:
///
/// ```text
/// page A is current -> fit to A -> zoom changes -> the strip re-lays out
///                   -> page B is now centred -> fit to B -> zoom changes
///                   -> page A is centred again -> ...
/// ```
///
/// On a document whose pages are all one size the loop is invisible, because
/// every row wants the same scale. On a **mixed-size** document it oscillates
/// visibly: measured at `page=0 zoom=1.4773` flip-flopping with
/// `page=1 zoom=0.9559` for as long as the wheel was moving.
///
/// This is `PROJECT_PLAN.md`'s R128 in a new place — *content-driven size plus
/// a per-frame fit is a measured feedback loop* — and `row_metrics`' own
/// header had already noticed one half of it: *"the strip's geometry depends
/// on the zoom this produces, so it cannot be the source of it"*. The half it
/// missed is that under a continuous mode `current` is part of that geometry.
///
/// # The fix: fit the tightest row, not the current one
///
/// Under a continuous mode this returns the row needing the **smallest**
/// scale — so "Fit page" means *a page fits*, for every page, and the answer
/// does not depend on where the operator has scrolled to. Scroll-independence
/// is the property; fitting every page is the reason that particular
/// scroll-independent choice is the right one rather than merely a stable one.
///
/// Under Single and Facing the current row is still exactly right: those modes
/// show one row and the operator chose it, so there is no loop to close.
///
/// **On a document whose pages are all the same size this changes nothing** —
/// every row is the tightest — which is why the fix costs nothing in the
/// common case and only differs on documents that were previously broken.
///
/// # Cost
///
/// O(pages) per frame under a continuous mode, against O(1) before. It is a
/// few float operations per page and no allocation: on a 2,000-page document
/// that is well under the ~16 us the ruler overlay already spends. Measuring
/// it was preferred to caching it, because a cache keyed on the page list
/// would be a second thing to invalidate when a page is inserted.
#[must_use]
pub fn fit_metrics(
    pages: &[Page],
    display: PageDisplay,
    current: usize,
    pixels_per_point: f32,
) -> RowMetrics {
    if !display.is_continuous() {
        return row_metrics(pages, display, current, pixels_per_point);
    }
    let rows = display.row_count(pages.len());
    if rows == 0 {
        return row_metrics(pages, display, current, pixels_per_point);
    }
    // "Tightest" is decided by AREA rather than by either axis alone: a fit
    // scale is `min(vw/pw, vh/ph)`, and which axis binds depends on the
    // viewport this function does not have. The row with the largest extent in
    // both axes taken together is the one that will need the smallest scale
    // whatever the viewport turns out to be, and taking the per-axis maxima
    // separately is what makes that true even for a document mixing portrait
    // and landscape sheets — neither row alone is the widest AND the tallest,
    // so fitting either one would leave the other overflowing.
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    let mut max_zoom = super::MAX_ZOOM;
    for row in 0..rows {
        let first = display.pages_in_row(row, pages.len()).next().unwrap_or(0);
        let m = row_metrics(pages, display, first, pixels_per_point);
        width = width.max(m.extent.0);
        height = height.max(m.extent.1);
        max_zoom = max_zoom.min(m.max_zoom);
    }
    RowMetrics {
        extent: (width, height),
        max_zoom,
    }
}

/// One page's rectangle in **strip space**, at the strip's zoom.
///
/// `Copy` and two fields, for the same reason
/// [`crate::canvas::mapping::PageMapping`] is: it is a fact about one frame,
/// and a borrow that outlived the frame would describe a layout that has since
/// moved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The 0-based page index.
    pub page: usize,
    /// Where it sits, in strip space, already multiplied by the zoom.
    pub rect: Rect,
}

/// One row of the strip: one page, or one facing spread.
#[derive(Debug, Clone, Copy)]
struct Row {
    /// The first page in the row.
    first: usize,
    /// How many pages are in it — 1, or 2 for a full spread.
    len: usize,
    /// Each page's extent in PDF user-space units, `/Rotate` applied. Held so
    /// the placement pass does not call `page_device_geometry` a second time.
    extents: [(f32, f32); 2],
    /// The row's top edge, in strip space at zoom 1.0.
    top: f32,
    /// The row's height at zoom 1.0 — the taller of its pages.
    height: f32,
    /// The row's width at zoom 1.0, including [`SPREAD_GAP`] when it holds two
    /// pages.
    width: f32,
}

/// **Where every page the canvas is showing sits, in one space.**
///
/// Built once per frame in [`crate::canvas::show`], immediately from the page
/// vector and the view state, and then asked rather than re-derived. Every
/// question the canvas has about layout — how big is the scroll content, which
/// pages are visible, which page is the pointer over, which page is "current"
/// now that the operator has scrolled — is a method here, so that no two of
/// them can answer from different arithmetic.
#[derive(Debug, Clone)]
pub struct Strip {
    /// Which arrangement this is.
    display: PageDisplay,
    /// Logical points per PDF user-space unit.
    zoom: f32,
    /// The rows actually laid out: every row under a continuous mode, the
    /// current page's row alone otherwise.
    rows: Vec<Row>,
    /// The strip's width at zoom 1.0 — the widest row.
    width: f32,
    /// The strip's height at zoom 1.0, gaps included.
    height: f32,
    /// Index into [`Self::rows`] of the row holding the view's current page.
    current_row: usize,
}

impl Strip {
    /// Lay out the pages this view is showing.
    ///
    /// `current` is [`crate::viewer::ViewState::page_index`], which decides
    /// *which* row a non-continuous mode lays out and, in every mode, which row
    /// [`Self::row_extent`] and [`Self::row_max_zoom`] describe. It is clamped
    /// into the document rather than trusted, because a page count that shrank
    /// under a stale index is the same hazard
    /// [`crate::viewer::clamp_page_index`] exists for.
    ///
    /// An empty page vector yields an empty strip: `size()` is zero and every
    /// query answers `None`. That is a legal document (`/Count 0`) and the
    /// canvas already has a sentence for it, so failing here would replace a
    /// message with a panic.
    #[must_use]
    pub fn new(pages: &[Page], display: PageDisplay, current: usize, zoom: f32) -> Self {
        let page_count = pages.len();
        let current = super::clamp_page_index(current, page_count);
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };

        let current_row_index = display.row_of(current);
        let rows_to_lay_out = if display.is_continuous() {
            0..display.row_count(page_count)
        } else {
            current_row_index..(current_row_index + 1).min(display.row_count(page_count))
        };

        let mut rows: Vec<Row> = Vec::with_capacity(rows_to_lay_out.len());
        let mut top = 0.0_f32;
        let mut width = 0.0_f32;
        for row_index in rows_to_lay_out.clone() {
            let range = display.pages_in_row(row_index, page_count);
            if range.is_empty() {
                continue;
            }
            let mut extents = [(0.0_f32, 0.0_f32); 2];
            let mut row_width = 0.0_f32;
            let mut row_height = 0.0_f32;
            for (slot, page) in range.clone().enumerate() {
                // `extents` is two wide because a row is at most a spread; a
                // `pages_in_row` that ever returned three would silently drop
                // the third here, so the range is trusted only as far as the
                // spread rule guarantees and the extra pages are ignored
                // rather than indexed out of bounds.
                let Some(slot_extent) = extents.get_mut(slot) else {
                    break;
                };
                let extent = page_extent_pts(&pages[page]);
                *slot_extent = extent;
                if slot > 0 {
                    row_width += SPREAD_GAP;
                }
                row_width += extent.0;
                row_height = row_height.max(extent.1);
            }
            let len = range.len().min(2);
            rows.push(Row {
                first: range.start,
                len,
                extents,
                top,
                height: row_height,
                width: row_width,
            });
            width = width.max(row_width);
            top += row_height + ROW_GAP;
        }

        // The trailing gap belongs between rows, not after the last one: a
        // strip that ended with a gap would give the scroll range 12 points of
        // nothing to scroll into and would put the last page's bottom edge
        // above the viewport bottom at full scroll.
        let height = (top - ROW_GAP).max(0.0);

        // Which entry of `rows` is the current page's. Under a continuous mode
        // that is its row index; under Single/Facing exactly one row was laid
        // out, so it is 0. Computed rather than assumed so a future mode that
        // lays out a *window* of rows does not silently index the wrong one.
        let current_row = rows
            .iter()
            .position(|r| current >= r.first && current < r.first + r.len)
            .unwrap_or(0);

        Self {
            display,
            zoom,
            rows,
            width,
            height,
            current_row,
        }
    }

    /// Which arrangement this strip is laid out in.
    #[must_use]
    pub fn display(&self) -> PageDisplay {
        self.display
    }

    /// The strip's drawn size — the scroll area's content size.
    ///
    /// For [`PageDisplay::Single`] this is exactly `extent × zoom` for the
    /// current page, which is the expression `canvas::show` used before Phase 4
    /// and the one every scroll, pan and anchor solve is written against.
    #[must_use]
    pub fn size(&self) -> Vec2 {
        vec2(self.width * self.zoom, self.height * self.zoom)
    }

    /// Whether the strip laid out no pages at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Every page in the strip, with its rectangle.
    ///
    /// Lazy: nothing is allocated, and a caller that only wants the visible
    /// ones pays for the rows it skips and nothing more.
    pub fn placements(&self) -> impl Iterator<Item = Placement> + '_ {
        self.rows.iter().flat_map(move |row| self.place_row(row))
    }

    /// Every page whose rectangle intersects `view`, in strip space.
    ///
    /// `view` is the scroll viewport expressed in strip coordinates — i.e.
    /// `Rect::from_min_size(offset, viewport)`. **This is the set that gets
    /// rasterized**, and it is deliberately intersection rather than
    /// containment: a page one pixel of which is on screen is a page the
    /// operator can see, and drawing nothing there would be a visible hole at
    /// every row boundary.
    pub fn visible(&self, view: Rect) -> impl Iterator<Item = Placement> + '_ {
        self.placements().filter(move |p| p.rect.intersects(view))
    }

    /// Where page `page` sits, or `None` if this strip does not lay it out.
    ///
    /// `None` is the ordinary answer under [`PageDisplay::Single`] for every
    /// page but the current one, and callers rely on that: it is what makes
    /// "draw the find highlights for this page" a no-op on a page that is not
    /// being shown, without a mode check at the call site.
    #[must_use]
    pub fn rect_of(&self, page: usize) -> Option<Rect> {
        let row = self
            .rows
            .iter()
            .find(|r| page >= r.first && page < r.first + r.len)?;
        self.place_row(row).find(|p| p.page == page).map(|p| p.rect)
    }

    /// The page under a **strip-space** point, if the point is on one.
    ///
    /// `None` in the gaps between rows and in the centring margin either side
    /// of a narrow page — which is the truthful answer, and the reason the
    /// canvas asks this before deciding a click landed on anything.
    #[must_use]
    pub fn page_at(&self, point: Pos2) -> Option<usize> {
        self.placements()
            .find(|p| p.rect.contains(point))
            .map(|p| p.page)
    }

    /// ★ **The page the operator is looking at, derived from where they have
    /// scrolled to.**
    ///
    /// `GUI_ROADMAP.md` Phase 4.3's *"scroll-driven current-page tracking"*.
    /// The rule is **the greatest visible area wins**, with the lowest page
    /// index breaking a tie, and both halves matter:
    ///
    /// * *Greatest area*, not "the page under the viewport centre": on a
    ///   drawing sheet zoomed in past the viewport, the centre is always on
    ///   some page and the two rules agree; on a document zoomed out far enough
    ///   to show four pages, the centre rule flips the reported page as soon as
    ///   the boundary crosses the middle, which reads as the page number
    ///   twitching. Area is stable.
    /// * *Lowest index on a tie*, so a spread reports its left-hand page and a
    ///   boundary sitting exactly on the viewport edge does not oscillate
    ///   between two answers on alternate frames.
    ///
    /// `None` when nothing is visible at all, which happens for one frame after
    /// a mode change before the scroll area has settled. The caller leaves the
    /// current page where it was rather than guessing.
    #[must_use]
    pub fn page_at_view(&self, view: Rect) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for placement in self.placements() {
            let overlap = placement.rect.intersect(view);
            let area = overlap.width().max(0.0) * overlap.height().max(0.0);
            if area <= 0.0 {
                continue;
            }
            match best {
                Some((_, best_area)) if best_area >= area => {}
                _ => best = Some((placement.page, area)),
            }
        }
        best.map(|(page, _)| page)
    }

    /// The current row's extent in PDF user-space units — what a fit mode
    /// fits.
    ///
    /// **A row, not a page**, and that is the only thing Phase 4 changes about
    /// fitting. Under `Single` and `Continuous` a row is one page, so this is
    /// [`crate::app::state::OpenDoc::current_extent`] and "Fit page" behaves
    /// exactly as it did. Under a facing mode a row is the spread, and fitting
    /// one page of a spread would leave the other half off screen — which is
    /// not what a control called "Fit page" promises in a mode that shows two.
    ///
    /// Falls back to a US Letter shape for an empty strip, for the same reason
    /// `current_extent` does: the fit arithmetic needs something finite to
    /// divide by, and nothing is drawn in that state anyway.
    #[must_use]
    pub fn row_extent(&self) -> (f32, f32) {
        self.rows
            .get(self.current_row)
            .map_or((612.0, 792.0), |r| (r.width, r.height))
    }

    /// ★ **The highest zoom every page of the current row can still
    /// rasterize at.**
    ///
    /// The per-page raster ceiling, generalised to a row. It is the
    /// **minimum** of [`crate::viewer::max_zoom_for_page`] over the row's
    /// pages, and it is a minimum over *pages* rather than the ceiling of the
    /// row's combined extent for a concrete reason: a spread is two pixmaps,
    /// not one. Computing `max_zoom_for_page(row_extent)` would halve the
    /// available zoom on a facing spread to guard an allocation nobody makes.
    ///
    /// It deliberately does **not** consider the pages a continuous strip is
    /// scrolled past. Those are separate pixmaps too, each guarded by its own
    /// ceiling when it is rendered, and taking a minimum over a whole document
    /// would let one Annex-C-sized sheet cap the zoom on the other 399 pages —
    /// a control that stops working because of a page the operator cannot see.
    /// The consequence, stated rather than hidden: scrolling a continuous strip
    /// onto an enormous page while zoomed in past *its* ceiling shows that page
    /// undrawn with the reason given, rather than silently zooming the whole
    /// document out. See [`crate::render::strip`] for what the page says.
    ///
    /// `pixels_per_point` is threaded through because the ceiling is about
    /// **device pixels** — see `max_zoom_for_page`'s own docs on why omitting
    /// it is how a guard passes its tests and fails on a HiDPI laptop.
    #[must_use]
    pub fn row_max_zoom(&self, pixels_per_point: f32) -> f32 {
        let Some(row) = self.rows.get(self.current_row) else {
            return super::MAX_ZOOM;
        };
        row.extents
            .iter()
            .take(row.len)
            .map(|&extent| max_zoom_for_page(extent, pixels_per_point))
            .fold(super::MAX_ZOOM, f32::min)
    }

    /// The pages of one row, placed.
    ///
    /// The single owner of the placement arithmetic — a row is centred
    /// horizontally in the strip, and each page is centred vertically in its
    /// row so a short page in a tall spread does not hang from the top edge.
    fn place_row(&self, row: &Row) -> impl Iterator<Item = Placement> + '_ {
        let row = *row;
        let zoom = self.zoom;
        let row_left = (self.width - row.width) / 2.0;
        (0..row.len).scan(row_left, move |x, slot| {
            let (w, h) = row.extents[slot];
            if slot > 0 {
                *x += SPREAD_GAP;
            }
            let left = *x;
            *x += w;
            let top = row.top + (row.height - h) / 2.0;
            Some(Placement {
                page: row.first + slot,
                rect: Rect::from_min_size(pos2(left * zoom, top * zoom), vec2(w * zoom, h * zoom)),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::object::{Dict, ObjId};
    use pdfcer_core::page_tree::Rect as PageRect;

    /// A `w`×`h` page with no rotation — enough for the geometry, which reads
    /// only `crop_box` and `rotate`.
    fn page(w: f64, h: f64) -> Page {
        Page {
            id: ObjId::new(1, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, w, h),
            crop_box: PageRect::from_corners(0.0, 0.0, w, h),
            rotate: 0,
            contents: Vec::new(),
            contents_unresolved: 0,
            contents_flattened: 0,
        }
    }

    /// `n` identical US Letter pages.
    fn letter(n: usize) -> Vec<Page> {
        (0..n).map(|_| page(612.0, 792.0)).collect()
    }

    /// ★ **Single page reproduces the pre-Phase-4 geometry exactly.**
    ///
    /// The operator's constraint, as an equality rather than an intention:
    /// continuous is *an option, not a replacement*, and the way that survives
    /// a refactor is that the single-page strip's size **is** the expression
    /// `canvas::show` used before this feature existed — `extent × zoom`, with
    /// the page at the origin and no gap anywhere.
    ///
    /// A change that gives `Single` a row gap, a margin or a scroll range it
    /// did not have fails here.
    #[test]
    fn single_page_reproduces_the_pre_phase_4_geometry_exactly() {
        let pages = letter(5);
        for &zoom in &[0.1_f32, 0.5, 1.0, 2.5, 8.0] {
            for current in 0..5 {
                let strip = Strip::new(&pages, PageDisplay::Single, current, zoom);
                let extent = page_extent_pts(&pages[current]);
                // The exact expression the canvas used: `vec2(extent.0 * zoom,
                // extent.1 * zoom)`.
                assert_eq!(strip.size(), vec2(extent.0 * zoom, extent.1 * zoom));
                let rect = strip
                    .rect_of(current)
                    .expect("the current page is laid out");
                assert_eq!(rect.min, Pos2::ZERO, "the page starts at the strip origin");
                assert_eq!(rect.size(), strip.size());
                // …and nothing else is laid out, so a highlight for another
                // page is a no-op without a mode check at the call site.
                for other in (0..5).filter(|&p| p != current) {
                    assert_eq!(strip.rect_of(other), None);
                }
                assert_eq!(strip.row_extent(), extent);
            }
        }
    }

    /// A continuous strip stacks every page, gap-separated, and is as tall as
    /// the sum of them.
    #[test]
    fn a_continuous_strip_stacks_every_page_with_one_gap_between_each() {
        let pages = letter(4);
        let strip = Strip::new(&pages, PageDisplay::Continuous, 0, 1.0);
        assert_eq!(strip.size().x, 612.0);
        assert_eq!(
            strip.size().y,
            4.0 * 792.0 + 3.0 * ROW_GAP,
            "three gaps, not four"
        );
        for p in 0..4 {
            #[allow(clippy::cast_precision_loss, reason = "four pages")]
            // ui-text-exempt: clippy lint justification, never displayed
            let expected_top = p as f32 * (792.0 + ROW_GAP);
            let rect = strip.rect_of(p).expect("every page is laid out");
            assert!(
                (rect.min.y - expected_top).abs() < 1e-3,
                "page {p}: {rect:?}"
            );
            assert_eq!(rect.min.x, 0.0);
            assert_eq!(rect.size(), vec2(612.0, 792.0));
        }
    }

    /// The strip's geometry is exactly linear in the zoom.
    ///
    /// The property the zoom anchor's "measure, then re-place" solve rests on:
    /// [`crate::canvas::geometry::zoom_anchor_offset`] holds a *fraction* of
    /// the content still across a zoom step, which is only correct if doubling
    /// the zoom doubles every coordinate. A constant screen gap would break it
    /// by the gap, cumulatively, down a long strip.
    #[test]
    fn strip_geometry_is_exactly_linear_in_the_zoom() {
        let pages = letter(6);
        let one = Strip::new(&pages, PageDisplay::Continuous, 0, 1.0);
        let two = Strip::new(&pages, PageDisplay::Continuous, 0, 2.0);
        assert_eq!(two.size(), one.size() * 2.0);
        for p in 0..6 {
            let a = one.rect_of(p).expect("laid out");
            let b = two.rect_of(p).expect("laid out");
            assert!((b.min.y - a.min.y * 2.0).abs() < 1e-3, "page {p}");
            assert!((b.max.x - a.max.x * 2.0).abs() < 1e-3, "page {p}");
        }
    }

    /// Rows of different widths are centred rather than left-aligned, so a
    /// mixed-size document does not read as a ragged left edge.
    #[test]
    fn a_narrow_page_is_centred_in_a_wider_strip() {
        let pages = vec![page(1000.0, 500.0), page(400.0, 500.0)];
        let strip = Strip::new(&pages, PageDisplay::Continuous, 0, 1.0);
        assert_eq!(strip.size().x, 1000.0);
        let wide = strip.rect_of(0).expect("laid out");
        let narrow = strip.rect_of(1).expect("laid out");
        assert_eq!(wide.min.x, 0.0);
        assert!(
            (narrow.center().x - wide.center().x).abs() < 1e-3,
            "the narrow page must share the strip's centre line: {narrow:?}"
        );
    }

    /// A short page in a tall spread is centred vertically in its row rather
    /// than hanging from the top edge.
    #[test]
    fn pages_of_unequal_height_are_centred_within_their_row() {
        let pages = vec![page(300.0, 400.0), page(300.0, 800.0), page(300.0, 400.0)];
        let strip = Strip::new(&pages, PageDisplay::Facing, 1, 1.0);
        // Row 1 is pages 1 and 2: heights 800 and 400.
        let tall = strip.rect_of(1).expect("laid out");
        let short = strip.rect_of(2).expect("laid out");
        assert_eq!(tall.height(), 800.0);
        assert_eq!(short.height(), 400.0);
        assert!(
            (short.center().y - tall.center().y).abs() < 1e-3,
            "the short page must sit on the row's centre line"
        );
        assert_eq!(strip.row_extent(), (300.0 + SPREAD_GAP + 300.0, 800.0));
    }

    /// A facing spread puts its two pages side by side with one
    /// [`SPREAD_GAP`], and the cover is alone.
    #[test]
    fn a_facing_spread_is_two_pages_side_by_side() {
        let pages = letter(5);
        let cover = Strip::new(&pages, PageDisplay::Facing, 0, 1.0);
        assert_eq!(cover.size(), vec2(612.0, 792.0), "the cover is alone");
        assert_eq!(cover.rect_of(1), None);

        let spread = Strip::new(&pages, PageDisplay::Facing, 2, 1.0);
        assert_eq!(spread.size(), vec2(612.0 + SPREAD_GAP + 612.0, 792.0));
        let left = spread.rect_of(1).expect("page 1 is the left half");
        let right = spread.rect_of(2).expect("page 2 is the right half");
        assert_eq!(left.min.x, 0.0);
        assert!((right.min.x - (612.0 + SPREAD_GAP)).abs() < 1e-3);
        assert!(right.min.x > left.max.x, "the halves must not overlap");
    }

    /// ★ **The current page follows the scroll, by greatest visible area.**
    ///
    /// Phase 4.3. Asserted as the behaviour an operator sees — scroll down a
    /// page and the reported page becomes the next one — rather than as the
    /// arithmetic, and with the boundary case that would make a centre-based
    /// rule twitch.
    #[test]
    fn the_current_page_follows_the_scroll_by_greatest_visible_area() {
        let pages = letter(4);
        let strip = Strip::new(&pages, PageDisplay::Continuous, 0, 1.0);
        let viewport = vec2(612.0, 600.0);
        let view_at = |y: f32| Rect::from_min_size(pos2(0.0, y), viewport);

        assert_eq!(strip.page_at_view(view_at(0.0)), Some(0));
        // Scrolled so that most of page 1 is on screen.
        assert_eq!(strip.page_at_view(view_at(792.0 + ROW_GAP)), Some(1));
        // Straddling the boundary with more of page 0 showing. (At y = 500 a
        // 600 pt viewport shows 292 pt of page 0 and 296 pt of page 1, so the
        // area rule correctly answers page 1 — which is exactly the kind of
        // near-tie a hand-picked "obviously page 0" case would have hidden.)
        assert_eq!(strip.page_at_view(view_at(400.0)), Some(0));
        // …and with more of page 1 showing.
        assert_eq!(strip.page_at_view(view_at(700.0)), Some(1));
        // Far past the end: the last page, not a panic and not page 0.
        assert_eq!(strip.page_at_view(view_at(3000.0)), Some(3));
        // Nothing visible at all.
        assert_eq!(
            strip.page_at_view(Rect::from_min_size(pos2(0.0, -5000.0), viewport)),
            None
        );
    }

    /// The visible set is intersection, not containment: a page one pixel of
    /// which is on screen is in the set.
    #[test]
    fn the_visible_set_includes_a_page_only_just_on_screen() {
        let pages = letter(4);
        let strip = Strip::new(&pages, PageDisplay::Continuous, 0, 1.0);
        // A viewport ending one point into page 1.
        let view = Rect::from_min_size(pos2(0.0, 0.0), vec2(612.0, 792.0 + ROW_GAP + 1.0));
        let seen: Vec<usize> = strip.visible(view).map(|p| p.page).collect();
        assert_eq!(seen, vec![0, 1]);

        // A viewport wholly inside the gap sees the pages either side of it.
        let gap = Rect::from_min_size(pos2(0.0, 792.0 + 1.0), vec2(612.0, ROW_GAP - 2.0));
        assert_eq!(strip.visible(gap).count(), 0, "the gap is not a page");
    }

    /// A point in the gap between rows is on no page, and says so.
    #[test]
    fn a_point_in_the_gap_is_on_no_page() {
        let pages = letter(3);
        let strip = Strip::new(&pages, PageDisplay::Continuous, 0, 1.0);
        assert_eq!(strip.page_at(pos2(300.0, 100.0)), Some(0));
        assert_eq!(strip.page_at(pos2(300.0, 792.0 + ROW_GAP / 2.0)), None);
        assert_eq!(strip.page_at(pos2(300.0, 792.0 + ROW_GAP + 10.0)), Some(1));
        assert_eq!(strip.page_at(pos2(-10.0, 100.0)), None, "outside the page");
    }

    /// ★ **The row ceiling is a minimum over pages, not the ceiling of the
    /// spread.**
    ///
    /// A spread is two pixmaps. Guarding it as one would halve the zoom range
    /// on every facing document for an allocation that never happens.
    #[test]
    fn the_row_ceiling_guards_each_page_rather_than_the_spread() {
        // Three Annex-C-sized pages, so row 1 really is a **two-page** spread:
        // with only two pages the cover rule puts page 0 alone in row 0 and
        // page 1 alone in row 1, and the test would prove nothing about a
        // spread. Each page hits the pixmap ceiling well below MAX_ZOOM, and
        // the spread is twice as wide as either.
        let pages = vec![
            page(14_400.0, 14_400.0),
            page(14_400.0, 14_400.0),
            page(14_400.0, 14_400.0),
        ];
        let strip = Strip::new(&pages, PageDisplay::Facing, 1, 1.0);
        assert_eq!(strip.placements().count(), 2, "this row must be a spread");
        let per_page = max_zoom_for_page((14_400.0, 14_400.0), 1.0);
        assert!((strip.row_max_zoom(1.0) - per_page).abs() < 1e-6);
        // The spread's own extent would have produced half of that.
        let as_one_pixmap = max_zoom_for_page(strip.row_extent(), 1.0);
        assert!(
            as_one_pixmap < per_page,
            "this fixture must actually distinguish the two rules"
        );

        // A mixed row takes the tighter of the two.
        let mixed = vec![
            page(612.0, 792.0),
            page(612.0, 792.0),
            page(14_400.0, 14_400.0),
        ];
        let strip = Strip::new(&mixed, PageDisplay::Facing, 1, 1.0);
        assert_eq!(strip.placements().count(), 2);
        assert!((strip.row_max_zoom(1.0) - per_page).abs() < 1e-6);

        // …and a continuous strip is NOT capped by a page it is scrolled past.
        let strip = Strip::new(&mixed, PageDisplay::Continuous, 0, 1.0);
        assert_eq!(strip.row_max_zoom(1.0), super::super::MAX_ZOOM);
    }

    /// A document with no pages lays out nothing and answers `None` to
    /// everything, rather than panicking. `/Count 0` is a legal document.
    #[test]
    fn an_empty_document_lays_out_nothing() {
        let strip = Strip::new(&[], PageDisplay::Continuous, 0, 1.0);
        assert!(strip.is_empty());
        assert_eq!(strip.size(), Vec2::ZERO);
        assert_eq!(strip.rect_of(0), None);
        assert_eq!(strip.page_at(Pos2::ZERO), None);
        assert_eq!(
            strip.page_at_view(Rect::from_min_size(Pos2::ZERO, vec2(100.0, 100.0))),
            None
        );
        assert_eq!(strip.row_extent(), (612.0, 792.0), "something to divide by");
        assert_eq!(strip.placements().count(), 0);
    }

    /// ★ **The cheap row metrics agree with the laid-out strip.**
    ///
    /// [`row_metrics`] exists so a frame costs one O(n) pass rather than two,
    /// and the price of that shortcut is a second derivation of two numbers a
    /// fit mode depends on. Two derivations of a fit scale is exactly how "Fit
    /// page" and the page it fits come to disagree, so the equality is
    /// asserted over every mode, a mixed-size document and both spread
    /// parities rather than left to review.
    #[test]
    fn row_metrics_agrees_with_the_laid_out_strip() {
        let pages = vec![
            page(612.0, 792.0),
            page(1000.0, 400.0),
            page(14_400.0, 14_400.0),
            page(300.0, 900.0),
            page(612.0, 792.0),
        ];
        for &mode in PageDisplay::ALL {
            for current in 0..pages.len() {
                for &ppp in &[1.0_f32, 2.0] {
                    let strip = Strip::new(&pages, mode, current, 1.0);
                    let cheap = row_metrics(&pages, mode, current, ppp);
                    assert_eq!(
                        cheap.extent,
                        strip.row_extent(),
                        "{mode:?} page {current}: extent"
                    );
                    assert!(
                        (cheap.max_zoom - strip.row_max_zoom(ppp)).abs() < 1e-6,
                        "{mode:?} page {current} ppp {ppp}: ceiling {} vs {}",
                        cheap.max_zoom,
                        strip.row_max_zoom(ppp)
                    );
                }
            }
        }
    }

    /// An empty document's row metrics are the same finite fallback the strip
    /// gives, so the fit arithmetic has something to divide by either way.
    #[test]
    fn row_metrics_of_an_empty_document_matches_the_strips_fallback() {
        let cheap = row_metrics(&[], PageDisplay::Continuous, 0, 1.0);
        let strip = Strip::new(&[], PageDisplay::Continuous, 0, 1.0);
        assert_eq!(cheap.extent, strip.row_extent());
        assert_eq!(cheap.max_zoom, strip.row_max_zoom(1.0));
    }

    /// A degenerate zoom is treated as actual size rather than producing a
    /// zero-size or NaN strip — `viewer`'s standing "fail to a finite,
    /// harmless value" discipline.
    #[test]
    fn a_degenerate_zoom_falls_back_to_actual_size() {
        let pages = letter(2);
        for bad in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
            let strip = Strip::new(&pages, PageDisplay::Continuous, 0, bad);
            assert!(strip.size().x.is_finite() && strip.size().y.is_finite());
            assert_eq!(strip.size().x, 612.0);
        }
    }

    /// A page index past the end clamps rather than laying out nothing, the
    /// same way `ViewState::go_to_page` clamps.
    #[test]
    fn a_stale_page_index_clamps_into_the_document() {
        let pages = letter(3);
        let strip = Strip::new(&pages, PageDisplay::Single, 99, 1.0);
        assert_eq!(strip.rect_of(2).map(|r| r.min), Some(Pos2::ZERO));
        assert_eq!(strip.placements().count(), 1);
    }

    // -----------------------------------------------------------------
    // fit_metrics — the fit must not depend on where you scrolled to
    // -----------------------------------------------------------------

    /// ★ **The regression test for the continuous-scroll zoom oscillation.**
    ///
    /// A mixed-size document, asked for its fit metrics from two different
    /// current pages. Under a continuous mode the answer must be the SAME —
    /// because the current page is derived from the scroll, so an answer that
    /// varies with it closes a loop between zoom and scroll position.
    ///
    /// Written as an equality between two calls rather than as an assertion
    /// about a particular scale, because the bug is not "the zoom is wrong".
    /// Either zoom was individually defensible; the defect is that the two
    /// disagreed, so the frame's answer depended on the previous frame's.
    #[test]
    fn a_continuous_fit_does_not_depend_on_the_current_page() {
        let pages = vec![page(1190.0, 841.0), page(612.0, 792.0), page(842.0, 595.0)];
        for display in [PageDisplay::Continuous, PageDisplay::FacingContinuous] {
            let from_first = fit_metrics(&pages, display, 0, 1.0);
            let from_last = fit_metrics(&pages, display, pages.len() - 1, 1.0);
            assert_eq!(
                from_first.extent, from_last.extent,
                "{display:?}: the fit extent moved when the scroll moved, which is the loop"
            );
            assert_eq!(from_first.max_zoom, from_last.max_zoom, "{display:?}");
        }
    }

    /// …and it is the TIGHTEST row, so every page fits.
    ///
    /// Scroll-independence alone would be satisfied by always fitting page 0,
    /// which is stable and wrong: a later, larger sheet would overflow a
    /// control called "Fit page". The per-axis maxima matter for the same
    /// reason — with a portrait and a landscape sheet in one document neither
    /// row is both the widest and the tallest, so fitting either whole row
    /// would leave the other overflowing on one axis.
    #[test]
    fn a_continuous_fit_frames_the_largest_extent_in_each_axis() {
        // Widest is the landscape A3; tallest is the portrait Letter.
        let pages = vec![page(1190.0, 841.0), page(612.0, 792.0)];
        let m = fit_metrics(&pages, PageDisplay::Continuous, 0, 1.0);
        assert!(
            (m.extent.0 - 1190.0).abs() < 0.5,
            "width must come from the widest row: {:?}",
            m.extent
        );
        assert!(
            (m.extent.1 - 841.0).abs() < 0.5,
            "height must come from the tallest row: {:?}",
            m.extent
        );
    }

    /// A one-page-size document is unaffected under **Continuous**.
    ///
    /// The property that makes this fix free in the common case: every row is
    /// one page and every page is the same, so `fit_metrics` and `row_metrics`
    /// agree exactly. Without it, the fix would be a silent behaviour change
    /// for every ordinary document rather than a repair of a broken one.
    #[test]
    fn a_uniform_document_fits_exactly_as_it_did_before() {
        let pages = vec![page(612.0, 792.0); 8];
        for current in [0, 3, 7] {
            assert_eq!(
                fit_metrics(&pages, PageDisplay::Continuous, current, 1.0).extent,
                row_metrics(&pages, PageDisplay::Continuous, current, 1.0).extent,
                "page {current}"
            );
        }
    }

    /// ★ **Facing-continuous is different even on a uniform document, and it
    /// must be.**
    ///
    /// This assertion was written the other way round first — as "a uniform
    /// document is unaffected in every continuous mode" — and it failed,
    /// correctly. Under a facing mode **row 0 is a cover**: one page, while
    /// every row after it is a two-page spread. So on eight identical Letter
    /// pages the rows are genuinely 612 pt and 1,230 pt wide, and they are not
    /// interchangeable.
    ///
    /// Fitting the cover would therefore make every spread in the document
    /// overflow — from a control called "Fit page", on the second row. The
    /// old per-row behaviour did exactly that whenever the operator's scroll
    /// happened to leave page 0 current, which is another face of the same
    /// bug rather than a separate one.
    #[test]
    fn facing_continuous_fits_the_spread_not_the_cover() {
        let pages = vec![page(612.0, 792.0); 8];
        let fit = fit_metrics(&pages, PageDisplay::FacingContinuous, 0, 1.0);
        let cover = row_metrics(&pages, PageDisplay::FacingContinuous, 0, 1.0);
        assert!(
            (cover.extent.0 - 612.0).abs() < 0.5,
            "row 0 is the cover, one page wide: {:?}",
            cover.extent
        );
        assert!(
            fit.extent.0 > cover.extent.0 * 1.9,
            "the fit must frame a two-page spread, not the cover: {:?} vs {:?}",
            fit.extent,
            cover.extent
        );
    }

    /// Single and Facing still fit the row the operator is on.
    ///
    /// They show one row at a time and the operator chose it, so there is no
    /// loop — and fitting the document's largest sheet there would shrink
    /// every other page for no reason. The fix must not leak into them.
    #[test]
    fn a_paged_mode_still_fits_the_current_row() {
        let pages = vec![page(1190.0, 841.0), page(612.0, 792.0)];
        for display in [PageDisplay::Single, PageDisplay::Facing] {
            for current in 0..pages.len() {
                assert_eq!(
                    fit_metrics(&pages, display, current, 1.0).extent,
                    row_metrics(&pages, display, current, 1.0).extent,
                    "{display:?} page {current} must be unchanged"
                );
            }
        }
    }
}
