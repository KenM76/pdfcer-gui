//! # `canvas::destscroll` — a destination that named a POINT scrolls, it does not zoom
//!
//! `OPERATOR_REQUESTS.md` O200, `DEFECTS.md` D47:
//!
//! > *"…links in word documents saved as pdfs such as table of contents …
//! > should just jump the position on the page pointed to without changing the
//! > zoom. If the position jumped to is visible on the page with the current
//! > horizontal position of the page, the horizontal position shouldn't be
//! > changed."*
//!
//! ## Contract
//!
//! [`crate::canvas::destination::arrive`] parks a [`DestScroll`] on the document
//! when a `/XYZ`-family destination resolves; `crate::canvas::offset`'s ranked
//! chain spends it, once, on the first frame that is drawing the named page.
//! **Nothing here reads or writes the zoom.** A `/XYZ` that carried an explicit
//! magnification still gets one, because `app::actions::destination` raises a
//! separate `Action::ZoomTo` that lands a frame earlier — so the scroll is
//! solved against the size that zoom produced.
//!
//! Rectangle destinations (`/FitR`, and every SolidWorks drawing bookmark
//! measured in the operator's own packages) do not come here at all: they keep
//! travelling through `zoom::zoom_to_rect`, so O154's framing is untouched.
//!
//! ## Why a second solver, when `destination` argues against one
//!
//! Framing and scrolling answer two different questions — *"what magnification
//! shows this region?"* and *"where must the view sit for this point to be at
//! the top-left?"*. The risk the old argument named, two positions that drift,
//! is answered by construction instead: the position itself comes from
//! [`geometry::offset_holding_anchor_at`], shared with the zoom anchor and with
//! `find::reveal`. All this module decides is **which screen position to ask
//! for, per axis**.
//!
//! The parked-fraction shape is `find::reveal`'s, for its reason: a scroll that
//! changes no magnification cannot ride `zoom::AnchorStep`'s handshake, because
//! that handshake is gated on the page's drawn size changing.

use egui::Vec2;

use crate::app::state::OpenDoc;
use crate::canvas::geometry;

/// How many frames a parked [`DestScroll`] waits for its page before it is
/// abandoned. A scroll held indefinitely would be spent minutes later, on an
/// unrelated page change, as a view that lurches on its own.
pub const DEST_GRACE_FRAMES: u8 = 4;

/// Paper left between the destination point and the corner of the view. The
/// same constant a fit is framed with, so "against the edge" means one thing.
pub const DEST_EDGE_MARGIN: f32 = crate::canvas::CANVAS_MARGIN;

/// How clear of the viewport edge a point must be to count as already visible.
///
/// A point exactly on the boundary is technically visible and practically is
/// not — it is under the scroll bar, or half of it is. Deliberately the same
/// number as [`DEST_EDGE_MARGIN`]: "clear of the edge" and "inset from the
/// edge" are one idea, and a second constant would let them drift.
const VISIBLE_CLEARANCE: f32 = DEST_EDGE_MARGIN;

/// A destination that named a point, waiting for a frame that can solve it.
///
/// Spans frames for the same reason `find_reveal` and `zoom_anchor` do: the
/// page change is applied after the canvas has drawn, so the earliest frame on
/// which the target page's drawn size is known is a later one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DestScroll {
    /// The page the destination is on. Solved only on a frame showing it.
    pub page: usize,
    /// Where the point sits across the page's **canvas-space** width, as a
    /// fraction of the page extent — `None` for §12.3.2.2's null, *"leave this
    /// axis as it is"*.
    ///
    /// A canvas axis, not a PDF one, and the difference is `/Rotate`: on a
    /// 90°-rotated sheet a PDF *x* drives the canvas *y*, so a `/FitV` — which
    /// specifies a PDF left edge and nothing else — constrains the view
    /// vertically. [`fracs_for`] resolves that once, when the point is parked.
    pub frac_x: Option<f32>,
    /// See [`Self::frac_x`].
    pub frac_y: Option<f32>,
    /// The horizontal content offset the view had **before anything
    /// navigated** — see [`crate::app::state::OpenDoc::dest_origin_x`]. Both
    /// the visibility test and the hold-still answer are made against this
    /// rather than against the live offset, because on a cross-page
    /// destination the live offset is already the strip's horizontal centring
    /// of the new page.
    pub origin_x: f32,
    /// Frames spent waiting for [`Self::page`]. See [`DEST_GRACE_FRAMES`].
    pub waited: u8,
}

/// A `/XYZ`-family destination's PDF point as a per-canvas-axis page fraction,
/// or `None` for a page whose device transform will not invert.
///
/// # Why the axes are probed rather than assumed
///
/// `left` and `top` are PDF user space; the canvas is the page as drawn, with
/// `/Rotate` applied. The probe transforms a second point displaced along PDF
/// *x*: whether that displacement comes out mostly along canvas *x* or mostly
/// along canvas *y* is the answer, and it is right for every rotation without
/// this module reading `/Rotate` at all.
///
/// The value substituted for a null axis never reaches the result — that axis's
/// fraction is dropped by the `map` below.
#[must_use]
pub fn fracs_for(
    left: Option<f64>,
    top: Option<f64>,
    page: &pdfcer_core::page_tree::Page,
) -> Option<(Option<f32>, Option<f32>)> {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "page coordinates are bounded by the media box; f32 is the canvas's own precision" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let (l, t) = (left.unwrap_or(0.0) as f32, top.unwrap_or(0.0) as f32);
    let at = crate::viewer::pdf_space_to_canvas(egui::pos2(l, t), page)?;
    // Ten points: large enough not to decide on rounding noise, small enough to
    // stay inside any real media box.
    let along_x = crate::viewer::pdf_space_to_canvas(egui::pos2(l + 10.0, t), page)?;
    let pdf_x_drives_canvas_x = (along_x.x - at.x).abs() >= (along_x.y - at.y).abs();

    // Against the page EXTENT rather than its drawn size, which is what makes
    // the value independent of the zoom and so recordable now, spendable later.
    let frac = crate::canvas::zoom::frac_of(at, crate::viewer::page_extent_pts(page));

    Some(if pdf_x_drives_canvas_x {
        (left.map(|_| frac.0), top.map(|_| frac.1))
    } else {
        (top.map(|_| frac.0), left.map(|_| frac.1))
    })
}

/// O200's second clause: does this axis already show the point?
///
/// `point` and `current` are both content-space — the position of the
/// destination along the scroll content, and the offset the view sits at.
///
/// # Why it is applied to the horizontal and not to both
///
/// A link is overwhelmingly a request for a vertical position: a heading, a
/// sheet, a paragraph. Applying this test vertically would make a link to a
/// heading half a screen below the current one do nothing at all, which reads
/// as a broken link. Horizontally there is no such expectation, and moving
/// sideways for a point already on screen is the lurch being reported.
#[must_use]
pub fn axis_stays_where_it_is(point: f32, current: f32, viewport: f32) -> bool {
    if !point.is_finite() || !current.is_finite() || !viewport.is_finite() {
        // Nothing can be asserted from a meaningless geometry, and not moving
        // is the answer that cannot be wrong.
        return true;
    }
    point >= current + VISIBLE_CLEARANCE && point <= current + viewport - VISIBLE_CLEARANCE
}

/// What [`solve`] decided, per axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solved {
    /// The content-space offset the scroll area should be forced to.
    pub offset: Vec2,
    /// The horizontal axis was held rather than moved.
    pub keep_x: bool,
    /// The vertical axis was held rather than moved.
    pub keep_y: bool,
}

/// **O200's rule, and the whole of it**, as arithmetic on four already-solved
/// positions.
///
/// Separated from [`take_dest_scroll_offset`] so that the rule has an enforcer
/// that does not need a window: everything above this line is geometry shared
/// with the zoom anchor and the find reveal, and everything the operator
/// actually asked for is the choice made here.
///
/// # Arguments
///
/// * `point` — where the destination sits along the content: the offset that
///   would put it at screen position zero.
/// * `want` — the offset that puts it [`DEST_EDGE_MARGIN`] in from the
///   top-left corner. Used only on an axis that moves.
/// * `current` — the offset the view is sitting at this frame, after whatever
///   outranked this in `canvas::offset` has had its say.
///
/// # The two answers to "hold this axis still"
///
/// * A **null** axis is §12.3.2.2's *"leave this one as it is"*, and "as it
///   is" means whatever won this frame's ranking — a fit-width's placement,
///   say. Held at `current`, so this cannot undo it.
/// * A **specified** horizontal axis already on screen is the operator's own
///   position, and that is [`DestScroll::origin_x`]: the offset before the page
///   turn, not the strip's horizontal centring of the page it turned to.
#[must_use]
pub fn solve(
    pending: &DestScroll,
    point: Vec2,
    want: Vec2,
    current: Vec2,
    viewport: (f32, f32),
) -> Solved {
    let keep_x =
        pending.frac_x.is_none() || axis_stays_where_it_is(point.x, pending.origin_x, viewport.0);
    // No visibility test on the vertical — see `axis_stays_where_it_is`.
    let keep_y = pending.frac_y.is_none();
    let hold_x = if pending.frac_x.is_none() {
        current.x
    } else {
        pending.origin_x
    };
    Solved {
        offset: Vec2::new(
            if keep_x { hold_x } else { want.x },
            if keep_y { current.y } else { want.y },
        ),
        keep_x,
        keep_y,
    }
}

/// One axis's fraction for the trace, spelling a null axis with §12.3.2.2's own
/// word rather than a Rust `Option`'s Debug form.
fn frac_field(frac: Option<f32>) -> String {
    // ui-text-exempt: diagnostic trace field, never displayed in the UI
    frac.map_or_else(|| "null".to_owned(), |f| format!("{f:.4}"))
}

/// The scroll offset that puts a parked point destination at the top-left of
/// the view, or `None` to leave the scroll area alone.
///
/// # Arguments that are not obvious
///
/// * `current` — the offset the view is sitting at this frame, in content
///   space. What a NULL axis is held at; see [`solve`] for why a specified
///   horizontal axis is held at [`DestScroll::origin_x`] instead.
/// * `to_strip` — the caller's page-local ⇒ content-space conversion. Passed in
///   because a continuous strip's conversion needs the strip layout, the
///   pasteboard overhang and the row rect, none of which belong here.
///
/// # The probe for "where is the point?", and what its clamp costs
///
/// `to_strip` clamps to the scrollable range, so near either end of the content
/// the probe reports the clamp rather than the point. That degrades safely: in
/// every clamped case the probe lands at the limit of the range, the visibility
/// test therefore answers *not visible*, and the offset the axis is then moved
/// to is that same limit — so the view does not move.
pub fn take_dest_scroll_offset(
    doc: &mut OpenDoc,
    display: (f32, f32),
    viewport: (f32, f32),
    current: Vec2,
    to_strip: &dyn Fn((f32, f32)) -> Vec2,
) -> Option<Vec2> {
    let pending = doc.dest_scroll?;
    if pending.page != doc.view.page_index {
        if pending.waited >= DEST_GRACE_FRAMES {
            doc.dest_scroll = None;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "dest-scroll-dropped page={} showing={}",
                    pending.page, doc.view.page_index
                )
            });
        } else {
            doc.dest_scroll = Some(DestScroll {
                waited: pending.waited + 1,
                ..pending
            });
        }
        return None;
    }
    doc.dest_scroll = None;

    // A null axis contributes no constraint and its fraction is never read —
    // the decision below takes `current` on that axis regardless.
    let frac = (pending.frac_x.unwrap_or(0.0), pending.frac_y.unwrap_or(0.0));
    let want = to_strip(geometry::offset_holding_anchor_at(
        frac,
        (DEST_EDGE_MARGIN, DEST_EDGE_MARGIN),
        display,
        viewport,
    ));
    // Where the point is along the content: the offset that would put it at
    // screen position zero. See the note on the clamp above.
    let point = to_strip(geometry::offset_holding_anchor_at(
        frac,
        (0.0, 0.0),
        display,
        viewport,
    ));

    let Solved {
        offset: solved,
        keep_x,
        keep_y,
    } = solve(&pending, point, want, current, viewport);
    crate::diag::trace(|| {
        // One `key=value` per field, never a `{:?}` tuple: a harness reads
        // `keep_x` here as the oracle for O200's horizontal clause, and a
        // Debug-formatted pair is not a field any trace reader can ask for.
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "dest-scroll-solved page={} frac_x={} frac_y={} point_x={:.1} point_y={:.1} \
             origin_x={:.1} keep_x={keep_x} keep_y={keep_y} off_x={:.1} off_y={:.1}",
            pending.page,
            frac_field(pending.frac_x),
            frac_field(pending.frac_y),
            point.x,
            point.y,
            pending.origin_x,
            solved.x,
            solved.y
        )
    });
    Some(solved)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O200's second clause. A point comfortably inside the visible span leaves
    /// the horizontal scroll exactly where the operator left it.
    #[test]
    fn a_point_already_on_screen_does_not_move_its_axis() {
        assert!(axis_stays_where_it_is(500.0, 100.0, 800.0));
        assert!(axis_stays_where_it_is(120.0, 100.0, 800.0));
        assert!(axis_stays_where_it_is(880.0, 100.0, 800.0));
    }

    /// A point off either end of the visible span moves its axis.
    #[test]
    fn a_point_off_the_screen_moves_its_axis() {
        assert!(!axis_stays_where_it_is(50.0, 100.0, 800.0));
        assert!(!axis_stays_where_it_is(2_000.0, 100.0, 800.0));
    }

    /// A point technically inside the viewport but pressed against its edge is
    /// under the scroll bar or clipped in half, and an operator who followed a
    /// link to it would say the link had not worked.
    #[test]
    fn a_point_pressed_against_the_edge_is_not_counted_as_visible() {
        assert!(!axis_stays_where_it_is(100.0, 100.0, 800.0));
        assert!(!axis_stays_where_it_is(900.0, 100.0, 800.0));
        assert!(!axis_stays_where_it_is(
            100.0 + VISIBLE_CLEARANCE - 0.1,
            100.0,
            800.0
        ));
        assert!(axis_stays_where_it_is(
            100.0 + VISIBLE_CLEARANCE,
            100.0,
            800.0
        ));
    }

    /// A destination at `frac`, with the operator's horizontal position at
    /// `origin_x`.
    fn parked(frac_x: Option<f32>, frac_y: Option<f32>, origin_x: f32) -> DestScroll {
        DestScroll {
            page: 2,
            frac_x,
            frac_y,
            origin_x,
            waited: 0,
        }
    }

    /// The viewport, and four positions chosen so that every axis's two
    /// outcomes are distinguishable from each other and from the inputs.
    const VP: (f32, f32) = (800.0, 600.0);
    /// Where the view is sitting this frame — on a cross-page destination this
    /// is already the strip's horizontal centring of the new page, which is
    /// what makes it the WRONG number to hold the horizontal at.
    const CURRENT: Vec2 = Vec2::new(300.0, 900.0);
    /// Where the operator was looking when they clicked.
    const ORIGIN_X: f32 = 120.0;
    /// The offset that would put the destination at the top-left corner.
    const POINT: Vec2 = Vec2::new(400.0, 5_000.0);
    /// The offset that puts it one margin in from that corner.
    const WANT: Vec2 = Vec2::new(384.0, 4_984.0);

    /// **D47 / O200's first clause, in the only place it can be asserted
    /// without a window.** A point destination produces an OFFSET and nothing
    /// else — there is no magnification anywhere in [`Solved`], and none of the
    /// four positions it chooses between is a scale.
    ///
    /// The mechanism this replaced grew the point into a 150 pt square and
    /// handed it to the framing solver, which answered with a zoom. That
    /// cannot be expressed here, which is the point: the type is the enforcer.
    #[test]
    fn a_point_destination_can_only_answer_with_a_position() {
        let solved = solve(
            &parked(Some(0.5), Some(0.5), ORIGIN_X),
            POINT,
            WANT,
            CURRENT,
            VP,
        );
        assert!(
            solved.offset.x == WANT.x || solved.offset.x == ORIGIN_X,
            "{solved:?}"
        );
        assert!(
            solved.offset.y == WANT.y || solved.offset.y == CURRENT.y,
            "{solved:?}"
        );
    }

    /// **O200's second clause.** The point is on screen from where the operator
    /// was, so the horizontal holds at THEIR position — not at `current.x`,
    /// which on a cross-page destination is the strip's centring of the page
    /// that was just turned to.
    ///
    /// The two numbers differ here deliberately: an assertion that both
    /// satisfy would not say which one shipped.
    #[test]
    fn a_visible_point_holds_the_horizontal_where_the_operator_left_it() {
        // POINT.x is 400; from origin 120 the visible span is 136..904.
        let solved = solve(
            &parked(Some(0.5), Some(0.5), ORIGIN_X),
            POINT,
            WANT,
            CURRENT,
            VP,
        );
        assert!(solved.keep_x, "{solved:?}");
        assert_eq!(solved.offset.x, ORIGIN_X, "{solved:?}");
        assert_ne!(solved.offset.x, CURRENT.x, "{solved:?}");
    }

    /// The witness for the clause above: a point the operator could NOT see
    /// moves the horizontal, so the hold is a decision rather than a
    /// do-nothing.
    #[test]
    fn a_point_the_operator_could_not_see_moves_the_horizontal() {
        // Origin 120, viewport 800: the visible span is 136..904, and 2,000 is
        // far off the right of it.
        let solved = solve(
            &parked(Some(0.5), Some(0.5), ORIGIN_X),
            Vec2::new(2_000.0, POINT.y),
            WANT,
            CURRENT,
            VP,
        );
        assert!(!solved.keep_x, "{solved:?}");
        assert_eq!(solved.offset.x, WANT.x, "{solved:?}");
    }

    /// **A null axis is left as it is — and "as it is" is THIS frame's
    /// offset**, not the pre-navigation one.
    ///
    /// `/FitH` names no left edge and raises `Action::Fit(Width)` beside the
    /// scroll; that fit outranks this in `canvas::offset` and has already
    /// placed the horizontal. Holding at `origin_x` here would undo it one
    /// frame later, which is D47's own failure shape wearing a different hat.
    #[test]
    fn a_null_horizontal_defers_to_whatever_placed_the_view_this_frame() {
        let solved = solve(&parked(None, Some(0.5), ORIGIN_X), POINT, WANT, CURRENT, VP);
        assert!(solved.keep_x, "{solved:?}");
        assert_eq!(solved.offset.x, CURRENT.x, "{solved:?}");
    }

    /// A null vertical axis — `/FitV`, and `/XYZ` with a null `top` — leaves
    /// the vertical alone, and a specified one always moves it. There is no
    /// visibility test on this axis: a link to a heading half a screen below
    /// the current one must still go there.
    #[test]
    fn the_vertical_moves_whenever_it_is_named_and_never_when_it_is_not() {
        let named = solve(
            &parked(Some(0.5), Some(0.5), ORIGIN_X),
            POINT,
            WANT,
            CURRENT,
            VP,
        );
        assert!(!named.keep_y, "{named:?}");
        assert_eq!(named.offset.y, WANT.y, "{named:?}");

        // Vertically on screen from the current offset — 5,000 is nowhere near
        // 900..1,500 — and it still moves, because only the horizontal asks.
        let near = solve(
            &parked(Some(0.5), Some(0.5), ORIGIN_X),
            Vec2::new(POINT.x, 1_000.0),
            WANT,
            CURRENT,
            VP,
        );
        assert!(!near.keep_y, "{near:?}");

        let null = solve(&parked(Some(0.5), None, ORIGIN_X), POINT, WANT, CURRENT, VP);
        assert!(null.keep_y, "{null:?}");
        assert_eq!(null.offset.y, CURRENT.y, "{null:?}");
    }

    /// A geometry that means nothing must not move the view.
    #[test]
    fn a_non_finite_geometry_holds_the_axis_still() {
        assert!(axis_stays_where_it_is(f32::NAN, 100.0, 800.0));
        assert!(axis_stays_where_it_is(500.0, f32::NAN, 800.0));
        assert!(axis_stays_where_it_is(500.0, 100.0, f32::INFINITY));
    }
}
