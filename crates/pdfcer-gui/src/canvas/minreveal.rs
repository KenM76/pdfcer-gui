//! # `canvas::minreveal` — bring a rectangle into view with the least movement
//!
//! `OPERATOR_REQUESTS.md` O204. A Tab press must be able to reach a form field
//! or a canvas object that is below the fold, and it must do so without
//! throwing the view around: the operator is reading a page, and the cost of
//! each Tab should be the smallest scroll that makes the next stop visible.
//!
//! ## Contract
//!
//! A caller parks a [`MinReveal`] on the open document — one shot, exactly as
//! `find::reveal` and [`crate::canvas::destscroll`] do — naming a page and a
//! rectangle expressed as **fractions of that page's canvas extent**.
//! `crate::canvas::offset`'s ranked chain spends it on the first frame that is
//! drawing the named page, and [`solve_axis`] decides, per axis, between
//! *stay exactly where you are* and *move by the minimum*.
//!
//! **Nothing here reads or writes the zoom**, and that is the difference
//! between this and `zoom::zoom_to_rect`: a Tab is a change of focus, not a
//! change of framing. A ring that re-framed the page on every press would be
//! unusable on a form whose fields differ in size.
//!
//! ## Why fractions, and why they span frames
//!
//! Both for `destscroll`'s reasons. A fraction of the page extent is
//! independent of the zoom, so it can be recorded on the frame the ring
//! advances and spent on a later one; and the page a cross-page Tab lands on is
//! not laid out until after `Action::GoToPage` has been applied, so the
//! earliest frame that can solve the scroll is not the frame that asked for it.
//!
//! ## Why a separate solver from `destscroll`
//!
//! `destscroll` moves its named axis **unconditionally** — a link is a request
//! for a vertical position, and refusing to move because the target happened to
//! be on screen reads as a broken link. Its own header carries that argument.
//! A Tab is the opposite: the overwhelmingly common case is the next field on
//! the same screen, and moving at all would be the defect. The two policies
//! cannot live in one function without a flag, and a flag would be a caller
//! choosing between two meanings of the same call.
//!
//! What *is* shared is the arithmetic: the position of a page point along the
//! scroll content comes from [`geometry::offset_holding_anchor_at`], the same
//! function the zoom anchor, the find reveal and the destination scroll all
//! use, so no second opinion about where a page point sits can exist.

use egui::Vec2;

use crate::app::state::OpenDoc;
use crate::canvas::geometry;

/// How many frames a parked [`MinReveal`] waits for its page before it is
/// abandoned.
///
/// Deliberately [`crate::canvas::destscroll::DEST_GRACE_FRAMES`]: both are
/// "wait for the page turn the action beside me raised", and a second number
/// would be two answers to one question. A reveal held indefinitely would be
/// spent minutes later, on an unrelated page change, as a view that lurches on
/// its own.
pub const REVEAL_GRACE_FRAMES: u8 = crate::canvas::destscroll::DEST_GRACE_FRAMES;

/// Paper left between the revealed rectangle and the edge of the view.
///
/// The same constant a fit is framed with, so "clear of the edge" means one
/// thing across the canvas. It is applied on both the near and the far side,
/// which is what stops a field that is technically visible but sitting under
/// the scroll bar from counting as reached.
pub const REVEAL_MARGIN: f32 = crate::canvas::CANVAS_MARGIN;

/// A rectangle waiting for a frame that can scroll to it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinReveal {
    /// The page the rectangle is on. Solved only on a frame showing it.
    pub page: usize,
    /// The rectangle's top-left, as a fraction of the page's **canvas** extent.
    ///
    /// Canvas space, not PDF space: the page's `/Rotate` is already folded in
    /// by `viewer::pdf_space_to_canvas`, so a widget on a rotated sheet arrives
    /// here at the position the operator sees it. See
    /// [`fracs_for_canvas_rect`].
    pub min: (f32, f32),
    /// The rectangle's bottom-right, in the same units as [`Self::min`].
    pub max: (f32, f32),
    /// How many frames this has waited for its page.
    pub waited: u8,
    /// Who asked, for the trace. Never displayed.
    pub why: &'static str,
}

/// Express a **canvas-space** rectangle on `page` as the fraction pair
/// [`MinReveal`] carries.
///
/// Measured against the page extent rather than its drawn size, which is what
/// makes the value independent of the zoom and so recordable now, spendable
/// later.
#[must_use]
pub fn fracs_for_canvas_rect(
    rect: egui::Rect,
    page: &pdfcer_core::page_tree::Page,
) -> ((f32, f32), (f32, f32)) {
    let extent = crate::viewer::page_extent_pts(page);
    (
        crate::canvas::zoom::frac_of(rect.min, extent),
        crate::canvas::zoom::frac_of(rect.max, extent),
    )
}

/// Park a reveal, replacing any reveal already waiting.
///
/// Replacing rather than queueing is the correct rule for a key the operator
/// holds down: three fast Tab presses are a request to be at the third stop,
/// and a queue would walk the view through the first two.
pub fn park(doc: &mut OpenDoc, reveal: MinReveal) {
    doc.min_reveal = Some(reveal);
}

/// What one axis should do about a rectangle that spans `[lo, hi]` along the
/// scroll content while the view sits at `current` and is `viewport` long.
///
/// `lo` and `hi` are content-space positions — the offsets at which the
/// rectangle's near and far edges would sit exactly at the start of the view.
/// The margin is applied outside both, so the answer is the offset at which the
/// rectangle *and its paper* are inside the view.
///
/// Returns `None` for *"this axis already shows it — do not move"*, which is
/// the answer the whole module exists to be able to give.
///
/// # The rectangle larger than the viewport
///
/// When `hi - lo + 2 * margin` exceeds `viewport` the two constraints conflict
/// and no offset satisfies both. The near edge wins: a field taller than the
/// screen is read from its start, and aligning to the far edge would put the
/// caret off the top of the view on the frame the operator began typing.
#[must_use]
pub fn solve_axis(lo: f32, hi: f32, current: f32, viewport: f32, margin: f32) -> Option<f32> {
    if !lo.is_finite() || !hi.is_finite() || !current.is_finite() || !viewport.is_finite() {
        // Nothing can be asserted from a meaningless geometry, and not moving
        // is the answer that cannot be wrong.
        return None;
    }
    let near = lo - margin;
    let far = hi + margin;
    if current > near {
        // The rectangle starts above, or left of, the view.
        Some(near)
    } else if current < far - viewport {
        // Its far edge is past the end of the view. Checked second so the
        // oversized case above resolves to the near edge.
        Some(far - viewport)
    } else {
        None
    }
}

/// Spend a parked reveal, if this frame is drawing the page it names.
///
/// `display` is the page's drawn size and `viewport` the visible extent, both
/// in the units `geometry` works in; `current` is the offset the view is
/// sitting at, in strip space; `to_strip` is `canvas::offset`'s own page-local
/// → strip conversion, handed in rather than reimplemented so the visibility
/// test is made in the same space as the answer.
///
/// Returns the offset the `ScrollArea` should be forced to, or `None` for
/// *"nothing to do"* — which covers both "no reveal parked" and "parked, and
/// both axes already show it". In the second case the reveal is still consumed:
/// it has been satisfied.
pub fn take_reveal_offset(
    doc: &mut OpenDoc,
    display: (f32, f32),
    viewport: (f32, f32),
    current: Vec2,
    to_strip: &dyn Fn((f32, f32)) -> Vec2,
) -> Option<Vec2> {
    let pending = doc.min_reveal?;
    if pending.page != doc.view.page_index {
        if pending.waited >= REVEAL_GRACE_FRAMES {
            doc.min_reveal = None;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "min-reveal-dropped page={} showing={} why={}",
                    pending.page, doc.view.page_index, pending.why
                )
            });
        } else {
            doc.min_reveal = Some(MinReveal {
                waited: pending.waited + 1,
                ..pending
            });
        }
        return None;
    }
    doc.min_reveal = None;

    let corner = |frac: (f32, f32)| {
        to_strip(geometry::offset_holding_anchor_at(
            frac,
            (0.0, 0.0),
            display,
            viewport,
        ))
    };
    let lo = corner(pending.min);
    let hi = corner(pending.max);

    let x = solve_axis(lo.x, hi.x, current.x, viewport.0, REVEAL_MARGIN);
    let y = solve_axis(lo.y, hi.y, current.y, viewport.1, REVEAL_MARGIN);
    crate::diag::trace(|| {
        // One `key=value` per field, never a `{:?}` tuple: a harness reads
        // `moved_x` / `moved_y` as the oracle for O204's "the least movement"
        // clause, and a Debug-formatted pair is not a field any trace reader
        // can ask for.
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "min-reveal-solved page={} why={} lo_x={:.1} lo_y={:.1} hi_x={:.1} hi_y={:.1} \
             moved_x={} moved_y={} off_x={:.1} off_y={:.1}",
            pending.page,
            pending.why,
            lo.x,
            lo.y,
            hi.x,
            hi.y,
            x.is_some(),
            y.is_some(),
            x.unwrap_or(current.x),
            y.unwrap_or(current.y),
        )
    });
    if x.is_none() && y.is_none() {
        return None;
    }
    Some(egui::vec2(x.unwrap_or(current.x), y.unwrap_or(current.y)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A viewport 800 long sitting at 100 therefore shows content 100..=900.
    const VIEWPORT: f32 = 800.0;
    const AT: f32 = 100.0;
    const MARGIN: f32 = 20.0;

    /// O204's whole point: the common case is the next field on the same
    /// screen, and the answer there is *do not move*.
    #[test]
    fn a_rectangle_already_clear_of_both_edges_does_not_move_its_axis() {
        assert_eq!(solve_axis(400.0, 500.0, AT, VIEWPORT, MARGIN), None);
        // Exactly one margin clear of each edge still counts as shown.
        assert_eq!(solve_axis(120.0, 880.0, AT, VIEWPORT, MARGIN), None);
    }

    /// The reason the margin is applied on both sides: a field flush with the
    /// bottom edge is under the scroll bar, not reached.
    #[test]
    fn a_rectangle_inside_the_margin_is_not_counted_as_shown() {
        // Far edge at 895, which is inside the view but only 5 clear of it.
        assert_eq!(
            solve_axis(850.0, 895.0, AT, VIEWPORT, MARGIN),
            Some(895.0 + MARGIN - VIEWPORT)
        );
        // Near edge at 110, likewise.
        assert_eq!(
            solve_axis(110.0, 300.0, AT, VIEWPORT, MARGIN),
            Some(110.0 - MARGIN)
        );
    }

    /// Below the fold: the view moves down by the least that puts the far edge
    /// and its paper inside, which leaves the rest of the page where it was.
    #[test]
    fn a_rectangle_below_the_fold_moves_by_the_minimum() {
        let moved = solve_axis(1000.0, 1040.0, AT, VIEWPORT, MARGIN).expect("must move");
        assert!((moved - (1040.0 + MARGIN - VIEWPORT)).abs() < f32::EPSILON);
        // And having moved there, the same rectangle is now satisfied.
        assert_eq!(solve_axis(1000.0, 1040.0, moved, VIEWPORT, MARGIN), None);
    }

    /// Above the view — Shift+Tab's direction — aligns the near edge.
    #[test]
    fn a_rectangle_above_the_view_aligns_its_near_edge() {
        assert_eq!(
            solve_axis(10.0, 60.0, AT, VIEWPORT, MARGIN),
            Some(10.0 - MARGIN)
        );
    }

    /// The conflicting-constraints case stated in [`solve_axis`]'s contract.
    #[test]
    fn a_rectangle_larger_than_the_viewport_aligns_its_near_edge() {
        // Spans 0..=2000 against an 800 viewport; no offset satisfies both.
        assert_eq!(
            solve_axis(0.0, 2000.0, AT, VIEWPORT, MARGIN),
            Some(0.0 - MARGIN)
        );
    }

    /// Not moving is the answer that cannot be wrong.
    #[test]
    fn a_meaningless_geometry_moves_nothing() {
        assert_eq!(solve_axis(f32::NAN, 60.0, AT, VIEWPORT, MARGIN), None);
        assert_eq!(solve_axis(10.0, f32::INFINITY, AT, VIEWPORT, MARGIN), None);
        assert_eq!(solve_axis(10.0, 60.0, f32::NAN, VIEWPORT, MARGIN), None);
        assert_eq!(solve_axis(10.0, 60.0, AT, f32::NAN, MARGIN), None);
    }
}
