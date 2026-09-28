//! # `viewer::strip` — where every page sits, in one coordinate space
//!
//! `GUI_ROADMAP.md` Phase 4.1 asks for *"`ViewState` holds a page **range**
//! rather than one `page_index`"*. This module is that range, expressed as
//! geometry rather than as a pair of indices — because "which pages are on
//! screen" is not a fact the view *holds*, it is a fact that falls out of
//! where the pages are and where the viewport is.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/viewer/strip.md`.

use egui::{Pos2, Rect, Vec2, pos2, vec2};
use pdfcer_core::page_tree::Page;

use super::display::PageDisplay;
use super::{max_zoom_for_page, page_extent_pts};

/// The gap between two rows of the strip, in points at zoom 1.0.
pub const ROW_GAP: f32 = 12.0;

/// The gap between the two pages of a facing spread, in points at zoom 1.0.
pub const SPREAD_GAP: f32 = 6.0;

/// What a fit mode fits, and the highest zoom it may fit to.
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
#[must_use]
pub fn row_metrics(
    pages: &[Page],
    display: PageDisplay,
    current: usize,
    pixels_per_point: f32,
    quality: crate::renderquality::RenderQuality,
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
        max_zoom = max_zoom.min(max_zoom_for_page(extent, pixels_per_point, quality));
    }
    RowMetrics {
        extent: (width, height),
        max_zoom,
    }
}

/// The metrics a **fit mode** should be computed from.
///
/// # Why this is not always `row_metrics`, and the bug that says so
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
    quality: crate::renderquality::RenderQuality,
) -> RowMetrics {
    if !display.is_continuous() {
        return row_metrics(pages, display, current, pixels_per_point, quality);
    }
    let rows = display.row_count(pages.len());
    if rows == 0 {
        return row_metrics(pages, display, current, pixels_per_point, quality);
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
        let m = row_metrics(pages, display, first, pixels_per_point, quality);
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
    pub fn visible(&self, view: Rect) -> impl Iterator<Item = Placement> + '_ {
        self.placements().filter(move |p| p.rect.intersects(view))
    }

    /// Where page `page` sits, or `None` if this strip does not lay it out.
    #[must_use]
    pub fn rect_of(&self, page: usize) -> Option<Rect> {
        let row = self
            .rows
            .iter()
            .find(|r| page >= r.first && page < r.first + r.len)?;
        self.place_row(row).find(|p| p.page == page).map(|p| p.rect)
    }

    /// **Where the whole ROW holding `page` sits** — one page under
    /// [`PageDisplay::Single`] and [`PageDisplay::Continuous`], the facing
    /// spread under either facing mode.
    #[must_use]
    pub fn row_rect_of(&self, page: usize) -> Option<Rect> {
        let row = self
            .rows
            .iter()
            .find(|r| page >= r.first && page < r.first + r.len)?;
        self.place_row(row).map(|p| p.rect).reduce(Rect::union)
    }

    /// The page under a **strip-space** point, if the point is on one.
    #[must_use]
    pub fn page_at(&self, point: Pos2) -> Option<usize> {
        self.placements()
            .find(|p| p.rect.contains(point))
            .map(|p| p.page)
    }

    /// **The page the operator is looking at, derived from where they have
    /// scrolled to.**
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
    #[must_use]
    pub fn row_extent(&self) -> (f32, f32) {
        self.rows
            .get(self.current_row)
            .map_or((612.0, 792.0), |r| (r.width, r.height))
    }

    /// **The highest zoom every page of the current row can still
    /// rasterize at.**
    #[must_use]
    pub fn row_max_zoom(
        &self,
        pixels_per_point: f32,
        quality: crate::renderquality::RenderQuality,
    ) -> f32 {
        let Some(row) = self.rows.get(self.current_row) else {
            return super::MAX_ZOOM;
        };
        row.extents
            .iter()
            .take(row.len)
            .map(|&extent| max_zoom_for_page(extent, pixels_per_point, quality))
            .fold(super::MAX_ZOOM, f32::min)
    }

    /// The pages of one row, placed.
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
    /// The quality whose multiplier is one, so every test that is not ABOUT the
    /// render quality asserts the same number it did before the factor entered
    /// the ceiling arithmetic.
    const NORMAL: crate::renderquality::RenderQuality = crate::renderquality::RenderQuality::Normal;

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
            crop_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            bleed_box: PageRect::from_corners(0.0, 0.0, w, h),
            bleed_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            trim_box: PageRect::from_corners(0.0, 0.0, w, h),
            trim_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            art_box: PageRect::from_corners(0.0, 0.0, w, h),
            art_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            rotate: 0,
            contents: Vec::new(),
            contents_unresolved: 0,
            resources_defaulted: false,
            contents_flattened: 0,
        }
    }

    /// `n` identical US Letter pages.
    fn letter(n: usize) -> Vec<Page> {
        (0..n).map(|_| page(612.0, 792.0)).collect()
    }

    /// **Single page reproduces the pre-Phase-4 geometry exactly.**
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

    /// **The current page follows the scroll, by greatest visible area.**
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

    /// **The row ceiling is a minimum over pages, not the ceiling of the
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
        let per_page = max_zoom_for_page((14_400.0, 14_400.0), 1.0, NORMAL);
        assert!((strip.row_max_zoom(1.0, NORMAL) - per_page).abs() < 1e-6);
        // The spread's own extent would have produced half of that.
        let as_one_pixmap = max_zoom_for_page(strip.row_extent(), 1.0, NORMAL);
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
        assert!((strip.row_max_zoom(1.0, NORMAL) - per_page).abs() < 1e-6);

        // …and a continuous strip is NOT capped by a page it is scrolled past.
        let strip = Strip::new(&mixed, PageDisplay::Continuous, 0, 1.0);
        assert_eq!(strip.row_max_zoom(1.0, NORMAL), super::super::MAX_ZOOM);
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

    /// **The cheap row metrics agree with the laid-out strip.**
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
                    for quality in [
                        crate::renderquality::RenderQuality::Normal,
                        crate::renderquality::RenderQuality::Sharper,
                    ] {
                        let strip = Strip::new(&pages, mode, current, 1.0);
                        let cheap = row_metrics(&pages, mode, current, ppp, quality);
                        assert_eq!(
                            cheap.extent,
                            strip.row_extent(),
                            "{mode:?} page {current}: extent"
                        );
                        assert!(
                            (cheap.max_zoom - strip.row_max_zoom(ppp, quality)).abs() < 1e-6,
                            "{mode:?} page {current} ppp {ppp}: ceiling {} vs {}",
                            cheap.max_zoom,
                            strip.row_max_zoom(ppp, quality)
                        );
                    }
                }
            }
        }
    }

    /// An empty document's row metrics are the same finite fallback the strip
    /// gives, so the fit arithmetic has something to divide by either way.
    #[test]
    fn row_metrics_of_an_empty_document_matches_the_strips_fallback() {
        let cheap = row_metrics(&[], PageDisplay::Continuous, 0, 1.0, NORMAL);
        let strip = Strip::new(&[], PageDisplay::Continuous, 0, 1.0);
        assert_eq!(cheap.extent, strip.row_extent());
        assert_eq!(cheap.max_zoom, strip.row_max_zoom(1.0, NORMAL));
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

    /// **The regression test for the continuous-scroll zoom oscillation.**
    #[test]
    fn a_continuous_fit_does_not_depend_on_the_current_page() {
        let pages = vec![page(1190.0, 841.0), page(612.0, 792.0), page(842.0, 595.0)];
        for display in [PageDisplay::Continuous, PageDisplay::FacingContinuous] {
            let from_first = fit_metrics(&pages, display, 0, 1.0, NORMAL);
            let from_last = fit_metrics(&pages, display, pages.len() - 1, 1.0, NORMAL);
            assert_eq!(
                from_first.extent, from_last.extent,
                "{display:?}: the fit extent moved when the scroll moved, which is the loop"
            );
            assert_eq!(from_first.max_zoom, from_last.max_zoom, "{display:?}");
        }
    }

    /// …and it is the TIGHTEST row, so every page fits.
    #[test]
    fn a_continuous_fit_frames_the_largest_extent_in_each_axis() {
        // Widest is the landscape A3; tallest is the portrait Letter.
        let pages = vec![page(1190.0, 841.0), page(612.0, 792.0)];
        let m = fit_metrics(&pages, PageDisplay::Continuous, 0, 1.0, NORMAL);
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
    #[test]
    fn a_uniform_document_fits_exactly_as_it_did_before() {
        let pages = vec![page(612.0, 792.0); 8];
        for current in [0, 3, 7] {
            assert_eq!(
                fit_metrics(&pages, PageDisplay::Continuous, current, 1.0, NORMAL).extent,
                row_metrics(&pages, PageDisplay::Continuous, current, 1.0, NORMAL).extent,
                "page {current}"
            );
        }
    }

    /// **Facing-continuous is different even on a uniform document, and it
    /// must be.**
    #[test]
    fn facing_continuous_fits_the_spread_not_the_cover() {
        let pages = vec![page(612.0, 792.0); 8];
        let fit = fit_metrics(&pages, PageDisplay::FacingContinuous, 0, 1.0, NORMAL);
        let cover = row_metrics(&pages, PageDisplay::FacingContinuous, 0, 1.0, NORMAL);
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
    #[test]
    fn a_paged_mode_still_fits_the_current_row() {
        let pages = vec![page(1190.0, 841.0), page(612.0, 792.0)];
        for display in [PageDisplay::Single, PageDisplay::Facing] {
            for current in 0..pages.len() {
                assert_eq!(
                    fit_metrics(&pages, display, current, 1.0, NORMAL).extent,
                    row_metrics(&pages, display, current, 1.0, NORMAL).extent,
                    "{display:?} page {current} must be unchanged"
                );
            }
        }
    }
}
