//! # `canvas::strip` — wiring the laid-out strip to the frame
//!
//! The canvas's half of Phase 4, kept out of [`super`] because it answers a
//! different question and because that file is against rule R2's 1,500-line
//! ceiling.
//!
//! Three modules now carry the word *strip*, and the split between them is the
//! project's standing one — the unit-testable decision on one side, the wiring
//! on the other:
//!
//! | module | subject |
//! |---|---|
//! | [`crate::viewer::strip`] | **where** every page sits, as pure geometry |
//! | [`crate::render::strip`] | **what** each page's picture is, or says instead |
//! | this module | **which** page the frame is about, and in what order the rest should be drawn |
//!
//! Everything here is a decision the frame has to make once the scroll area has
//! settled and before the gesture layer runs, and every one of them is a pure
//! function of numbers the frame already has — which is exactly why they are
//! here rather than inline in [`super::show`], where a `Response` in the way
//! would make them untestable.

use egui::{PointerButton, Pos2, Rect, Vec2, vec2};

use crate::canvas::geometry;
use crate::canvas::mapping::PageMapping;

use crate::app::state::OpenDoc;
use crate::viewer;

/// One page the canvas drew this frame, and the widget that senses it.
///
/// Every visible page gets a `click_and_drag` response, not only the current
/// one, because pressing on a page is how the operator moves to it under a
/// continuous mode — see [`show`]. The response of exactly one of them becomes
/// the frame's interaction response.
pub(crate) struct DrawnPage {
    /// The 0-based page index.
    pub(crate) page: usize,
    /// Its rect on screen, in window logical points.
    pub(crate) rect: Rect,
    /// Its own sensing widget.
    pub(crate) response: egui::Response,
    /// Whether a raster was painted into it, as opposed to a state sentence.
    ///
    /// Recorded here rather than re-derived for the trace, because the caches
    /// are asked once — during the draw — and a second reading afterwards
    /// could disagree with what the operator is looking at.
    pub(super) has_raster: bool,
    /// Where its raster was actually PAINTED, which is not [`Self::rect`]
    /// once the region tier is engaged.
    ///
    /// ★★ Recorded so the trace can report it, and the trace reports it
    /// because `OPERATOR_REQUESTS.md` O24c — the page lurching backwards
    /// mid-pan — is a defect in the PAINT rectangle and is invisible in every
    /// other field. `rect=` is the page's own rect and moves smoothly all the
    /// way through the fault; only this jumps.
    ///
    /// Equal to [`Self::rect`] whenever the raster is a whole-page one, which
    /// is every zoom below the pixmap ceiling.
    pub(super) paint_rect: Rect,
}

/// One page and its screen ⟷ canvas map.
///
/// The Find wash's unit of work. Separate from [`DrawnPage`] because it
/// outlives the scroll-area closure and holds no `Response`, and because
/// [`Frame`] is `Copy` while a `Response` is not.
#[derive(Debug, Clone, Copy)]
pub(super) struct PageView {
    /// The 0-based page index.
    pub(super) page: usize,
    /// Its screen ⟷ canvas map.
    pub(super) map: PageMapping,
}

/// What the **current** page has instead of a raster, if it has none.
///
/// The strip cache answers this for every other page ([`OpenDoc::strip_page_state`]);
/// the current page's answer comes from its own two fields, because its raster
/// lives in its own slot. See [`crate::render::strip`]'s header on why the
/// split exists.
///
/// `None` is unreachable in practice from the one call site — it is only asked
/// when there is no texture — and is answered as "waiting" rather than by
/// panicking, because the honest thing to draw on a page with no picture and
/// no stated reason is that it has not been drawn.
pub(super) fn current_page_state(doc: &OpenDoc) -> Option<crate::render::strip::PageState> {
    use crate::render::strip::PageState;
    if let Some(detail) = &doc.render_error {
        return Some(PageState::Refused(detail.clone()));
    }
    if doc
        .render_worker
        .rendering_key()
        .map(|k| k.page())
        .is_some_and(|p| p == doc.view.page_index)
    {
        return Some(PageState::Drawing);
    }
    Some(PageState::Waiting)
}

/// The pages drawn this frame, **nearest the viewport centre first**.
///
/// The order [`crate::render::settle`] fills the strip in. Nearest-first rather
/// than top-down because the operator is looking at the middle of the viewport:
/// filling from the top means the page they are reading is the last one to
/// arrive whenever they have scrolled to a boundary, which is exactly the
/// moment a continuous mode is being used.
///
/// `pages` is `(page index, the page's vertical centre on screen)` and
/// `centre_y` is the viewport's own vertical centre, both in **screen**
/// coordinates — one space, so there is no conversion here to get wrong. The
/// pair is passed rather than the [`DrawnPage`] slice so the ordering rule is a
/// pure function with a test ([`tests::the_render_order_starts_at_the_middle_of_the_viewport`]);
/// a `Response` cannot be constructed headlessly, and an ordering rule that
/// could only be checked by running a window is an ordering rule nobody checks.
pub(super) fn nearest_first(pages: &[(usize, f32)], centre_y: f32) -> Vec<usize> {
    let mut order: Vec<(f32, usize)> = pages
        .iter()
        .map(|&(page, y)| ((y - centre_y).abs(), page))
        .collect();
    // `total_cmp` rather than `partial_cmp().unwrap()`: a NaN distance is
    // reachable from a degenerate rect, and a comparator that panics on one
    // would take the whole frame down over a sort order.
    order.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    order.into_iter().map(|(_, page)| page).collect()
}

/// ★ **The scroll offset that brings a navigated-to page onto the strip**, or
/// `None` when nothing navigated.
///
/// The third source of a forced scroll offset, and the one Phase 4 adds. Under
/// a continuous mode a page **command** — Next page, the status bar's page box,
/// a bookmark, a Find hit that landed with no geometry — changes
/// `view.page_index` and nothing else, so without this the operator would press
/// "Next page" and watch nothing happen.
///
/// # The gate is `page_index != tracked_page`, and nothing weaker works
///
/// The canvas writes `page_index` itself on every frame from the scroll
/// position, so "the index changed" is true of every frame of every scroll and
/// cannot be the test. `tracked_page` records what the *canvas* last derived;
/// a difference therefore means something else wrote the field, which is
/// precisely the definition of a navigation. See
/// [`crate::app::state::OpenDoc::tracked_page`].
///
/// # Where the page is put, and why it is the top rather than the centre
///
/// The page's top edge goes to the top of the viewport, less the strip's
/// row gap so the sheet does not sit flush against the edge. Not centred:
/// "Next page" means *show me that page*, and a reader expects to arrive at
/// the top of it and read downwards. Centring a page shorter than the viewport
/// would also scroll the previous page's foot into view above it, which reads
/// as having overshot.
///
/// Returns `None` in the paged modes, where a page command changes what is
/// laid out rather than where it is — there is nothing to scroll to.
pub(super) fn page_scroll_offset(
    doc: &mut OpenDoc,
    strip: &viewer::strip::Strip,
    viewport: (f32, f32),
) -> Option<Vec2> {
    if !doc.view.display.is_continuous() || doc.view.page_index == doc.tracked_page {
        return None;
    }
    let rect = strip.rect_of(doc.view.page_index)?;
    doc.tracked_page = doc.view.page_index;
    let size = strip.size();
    // ★ The rect is STRIP space and the answer is a SCROLL offset; since O23
    // they differ by the pasteboard. `strip_to_scroll` is the one conversion.
    let x = crate::canvas::geometry::strip_to_scroll(
        rect.center().x - viewport.0 / 2.0,
        size.x,
        viewport.0,
    );
    let y = crate::canvas::geometry::strip_to_scroll(
        rect.min.y - viewer::strip::ROW_GAP,
        size.y,
        viewport.1,
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "canvas-page-scroll page={} offset=({x:.1},{y:.1})",
            doc.view.page_index
        )
    });
    Some(vec2(x, y))
}

/// **Which page this frame's input is about**, written into `view.page_index`
/// and `tracked_page`.
///
/// One of the four items of per-frame view bookkeeping the canvas is permitted
/// to write directly (see `canvas`'s module header): a scroll position cannot
/// be deferred into an `Action`, because the action would be applied after the
/// frame that has already drawn from it.
///
/// Lives here rather than in `canvas::show` because it is a question about the
/// **strip** — where the viewport falls across a column of pages — and because
/// R2's line limit is a prompt to find the seam rather than to raise the
/// limit. Its scroll-space conversion is the same one every other consumer of
/// `Strip` needs, and having it beside them is what makes the omission that
/// caused O26 visible next time.
pub(super) fn track_current_page(
    doc: &mut OpenDoc,
    layout: &viewer::strip::Strip,
    drawn: &[DrawnPage],
    scroll_offset: (f32, f32),
    display_size: egui::Vec2,
    viewport_size: egui::Vec2,
    deep: bool,
) {
    // ★ **Which page this frame's input is about**, and the two ways it is
    // decided. Both write `view.page_index`, which is the fourth item of
    // per-frame view bookkeeping the canvas is permitted to write (see the
    // module header): a scroll position cannot be deferred into an `Action`,
    // because the action would be applied after the frame that has already
    // drawn from it.
    //
    // 1. **the scroll**, under a continuous mode: the page with the greatest
    //    visible area, per `Strip::page_at_view`. This is `GUI_ROADMAP.md`
    //    Phase 4.3's scroll-driven current-page tracking, and it is what makes
    //    the status bar's page box, the Objects panel and the `objects n=`
    //    trace describe the sheet the operator is actually reading.
    // 2. **a press**, in any mode: pressing on a page makes it current, so a
    //    click on the page below acts on the page below rather than missing.
    //    A press outranks the scroll because it is deliberate, and it is read
    //    from the pages' own responses so it costs nothing on a frame with no
    //    input.
    //
    // ★★★ CONVERTED OUT OF CONTENT SPACE FIRST — `OPERATOR_REQUESTS.md` O26,
    // and the SECOND site of the omission O23 was four attempts long.
    //
    // `page_at_view` takes a **strip-space** rect; `scroll_offset` is measured
    // from the **content's** origin, which since O23's pasteboard sits a whole
    // viewport above and to the left of the strip's. Feeding the raw offset in
    // asks *"which page is a viewport-sized box one pasteboard past where I am
    // looking?"*, and the answer is a page several pitches too far down — or,
    // far more often, **no page at all**, because the horizontal error is a
    // whole viewport and the strip is only as wide as its widest page.
    //
    // ★★ Both failure modes are damaging and the silent one is worse.
    //
    // * *No page* means `page_at_view` returns `None` on nearly every frame,
    //   the branch does not run, and scroll-driven current-page tracking —
    //   Phase 4.3, the whole reason this block exists — has been **inert since
    //   the pasteboard landed**. Nothing said so: the page number simply
    //   stopped following the scroll, which reads as a feature nobody
    //   finished.
    // * *The wrong page* happens whenever the strip grows wide enough for the
    //   displaced box to clip its right-hand edge — which is a function of the
    //   zoom, so it arrives at one particular magnification and not the ones
    //   either side of it. That is the operator's *"seems to happen at other
    //   junctions too"*.
    //
    // ★★★ And the wrong page is not a cosmetic mis-report, because
    // `current_origin` — the frame of reference every single-page solve in
    // `canvas::zoom` and `find::reveal` is handed — is *this page's* origin in
    // the strip. Set it to page 7 and the next anchored zoom converts its
    // answer back through page 7's origin, so the view moves by seven page
    // pitches in one wheel notch. Measured on `SW41177.pdf`: one Ctrl+wheel
    // notch at 30 % took the view from page 1 to page 8, and the status bar
    // said so.
    //
    // ★ Skipped entirely at the deep tier, for the reason `visible_rect` is:
    // above the threshold the scroll offset is **forced to zero** and says
    // nothing about where the view is — the `f64` anchor holds that. Asking
    // this question there would answer about the strip's origin rather than
    // about the operator, and snap the current page to the first one in the
    // document from wherever they had zoomed into. At that magnification one
    // page fills the screen many times over and it is the acting page by
    // construction, so leaving `page_index` alone is both correct and cheap.
    let view_rect = Rect::from_min_size(
        Pos2::new(
            geometry::scroll_to_strip(scroll_offset.0, display_size.x, viewport_size.x),
            geometry::scroll_to_strip(scroll_offset.1, display_size.y, viewport_size.y),
        ),
        viewport_size,
    );
    if !deep
        && doc.view.display.is_continuous()
        && let Some(page) = layout.page_at_view(view_rect)
    {
        doc.view.page_index = page;
        doc.tracked_page = page;
    }
    if let Some(page) = drawn
        .iter()
        .find(|d| {
            d.response.drag_started_by(PointerButton::Primary)
                || d.response.clicked_by(PointerButton::Primary)
                || d.response.dragged_by(PointerButton::Primary)
                || d.response.secondary_clicked()
        })
        .map(|d| d.page)
    {
        doc.view.page_index = page;
        doc.tracked_page = page;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use crate::viewer::PageDisplay;
    use crate::viewer::strip::Strip;

    /// ★ **The renderer is pointed at the middle of the viewport first.**
    ///
    /// `render::settle` starts one render per frame and takes the first entry
    /// of this list that has no raster, so the order **is** the fill order. Top
    /// down would mean that whenever the operator has scrolled to a page
    /// boundary — which is when a continuous mode is being used — the page they
    /// are reading is the last one to arrive.
    #[test]
    fn the_render_order_starts_at_the_middle_of_the_viewport() {
        // Pages 3,4,5 on screen, the viewport's centre level with page 4.
        let pages = [(3usize, 100.0_f32), (4, 400.0), (5, 700.0)];
        assert_eq!(nearest_first(&pages, 400.0), vec![4, 3, 5]);
        // Scrolled so the centre sits between 4 and 5.
        assert_eq!(nearest_first(&pages, 560.0), vec![5, 4, 3]);
        // A single page is its own answer.
        assert_eq!(nearest_first(&[(7, 0.0)], 999.0), vec![7]);
        assert!(nearest_first(&[], 0.0).is_empty());
    }

    /// An exact tie takes the lower page index, so the order does not
    /// oscillate between two answers on alternate frames while the operator
    /// sits still.
    #[test]
    fn a_tie_in_the_render_order_is_broken_by_the_page_index() {
        let pages = [(9usize, 300.0_f32), (2, 100.0), (5, 100.0)];
        assert_eq!(nearest_first(&pages, 200.0), vec![2, 5, 9]);
    }

    /// A degenerate centre does not panic the frame over a sort order.
    #[test]
    fn a_non_finite_centre_still_produces_an_order() {
        let pages = [(0usize, 0.0_f32), (1, f32::NAN)];
        assert_eq!(nearest_first(&pages, 0.0).len(), 2);
    }

    /// ★ **A page command scrolls the strip; a scroll does not.**
    ///
    /// The gate this function exists for, from both sides. Without the first
    /// half, "Next page" in a continuous document does nothing at all; without
    /// the second, every frame of every scroll would fight the operator by
    /// snapping back to the page the last frame derived.
    #[test]
    fn a_page_command_scrolls_the_strip_and_a_scroll_does_not() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.view.display = PageDisplay::Continuous;
        let viewport = (612.0_f32, 400.0_f32);
        let strip = Strip::new(&doc.pages, doc.view.display, 0, 1.0);

        // Nothing has navigated: page_index == tracked_page.
        assert_eq!(page_scroll_offset(&mut doc, &strip, viewport), None);

        // A page command — `Action::NextPage` writes `page_index` and nothing
        // else, exactly as this simulates.
        doc.view.page_index = 2;
        let offset = page_scroll_offset(&mut doc, &strip, viewport)
            .expect("a navigation must move the strip");
        let rect = strip.rect_of(2).expect("page 2 is laid out");
        // ★ The expected value gained a `strip_to_scroll` since O23: the rect
        // is in STRIP space and the answer is a SCROLL offset, and the two now
        // differ by the pasteboard. The property is unchanged — the page's top
        // lands at the top of the viewport — only the space the number is
        // expressed in has moved.
        let want = crate::canvas::geometry::strip_to_scroll(
            rect.min.y - crate::viewer::strip::ROW_GAP,
            strip.size().y,
            viewport.1,
        );
        assert!(
            (offset.y - want).abs() < 0.01,
            "the page's top must land at the top of the viewport: {offset:?} vs {rect:?}"
        );
        assert_eq!(doc.tracked_page, 2, "the scroll is spent once");

        // …and it is spent: asking again on the next frame moves nothing.
        assert_eq!(page_scroll_offset(&mut doc, &strip, viewport), None);
    }

    /// A paged mode has nothing to scroll to: the page command changes what is
    /// laid out, not where it is.
    #[test]
    fn a_paged_mode_never_scrolls_to_a_page() {
        let mut doc = open_fixture(FOUR_PAGES);
        let viewport = (612.0_f32, 400.0_f32);
        for &display in &[PageDisplay::Single, PageDisplay::Facing] {
            doc.view.display = display;
            doc.view.page_index = 3;
            doc.tracked_page = 0;
            let strip = Strip::new(&doc.pages, display, 3, 1.0);
            assert_eq!(page_scroll_offset(&mut doc, &strip, viewport), None);
        }
    }

    /// ★★ **The offset stays inside the scrollable range — which is now the
    /// CONTENT's range, not the strip's.**
    ///
    /// This asserted `Vec2::ZERO` until 2026-08-21, on the reasoning that a
    /// viewport taller than the whole strip has nowhere to scroll. O23 made
    /// that false on purpose: there is a pasteboard of one viewport on every
    /// side, so even a document that fits entirely on screen can be moved
    /// around — which is the operator's *"move the view of the corner of the
    /// page to the center of the screen"* for a small document.
    ///
    /// ★ So the assertion is the INVARIANT rather than the number. Pinning the
    /// new number would say nothing about whether it is reachable, and this
    /// function's whole job is that its answer is inside the range egui will
    /// accept — an offset beyond it is silently clamped, and the page then
    /// does not appear where the navigation promised.
    #[test]
    fn the_forced_offset_stays_inside_the_scroll_range() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.view.display = PageDisplay::Continuous;
        let strip = Strip::new(&doc.pages, doc.view.display, 0, 1.0);
        let viewport = (10_000.0_f32, 10_000.0_f32);
        doc.view.page_index = 3;
        let offset = page_scroll_offset(&mut doc, &strip, viewport)
            .expect("a navigation must move the strip");
        let size = strip.size();
        for (got, extent, v) in [
            (offset.x, size.x, viewport.0),
            (offset.y, size.y, viewport.1),
        ] {
            let range = (crate::canvas::geometry::content_extent(extent, v) - v).max(0.0);
            assert!(
                (0.0..=range).contains(&got),
                "{got} is outside the scrollable range 0..={range}"
            );
        }
    }
}
