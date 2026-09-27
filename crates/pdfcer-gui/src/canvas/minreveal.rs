//! # `canvas::minreveal` — bring a rectangle into view with the least movement
//!
//! `OPERATOR_REQUESTS.md` O204. A Tab press must be able to reach a form field
//! or a canvas object that is below the fold, and it must do so without
//! throwing the view around: the operator is reading a page, and the cost of
//! each Tab should be the smallest scroll that makes the next stop visible.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/minreveal.md`.

use egui::Vec2;

use crate::app::state::OpenDoc;
use crate::canvas::geometry;

/// How many frames a parked [`MinReveal`] waits for its page before it is
/// abandoned.
pub const REVEAL_GRACE_FRAMES: u8 = crate::canvas::destscroll::DEST_GRACE_FRAMES;

/// Paper left between the revealed rectangle and the edge of the view.
pub const REVEAL_MARGIN: f32 = crate::canvas::CANVAS_MARGIN;

pub use pdfcer_gui_base::scrolltarget::MinReveal;

/// Express a **canvas-space** rectangle on `page` as the fraction pair
/// [`MinReveal`] carries.
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
pub fn park(doc: &mut OpenDoc, reveal: MinReveal) {
    doc.min_reveal = Some(reveal);
}

/// What one axis should do about a rectangle that spans `[lo, hi]` along the
/// scroll content while the view sits at `current` and is `viewport` long.
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
