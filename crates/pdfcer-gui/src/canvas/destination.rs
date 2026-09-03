//! # `canvas::destination` — **arriving where a bookmark points**
//!
//! Operator report, 2026-09-01: *"in Acrobat clicking on the nested bookmarks
//! in the drawing package takes you to a zoomed in area of the page … when we
//! click on ours it just jumps us to the correct page, but doesn't send us to
//! the spot on the page the bookmark actually points to."*
//!
//! Two halves, in two places, and this is the second:
//!
//! | | where |
//! |---|---|
//! | what a destination MEANS — the five `/XYZ`-family views | `app::actions::destination` |
//! | where the view actually LANDS — viewport, margin, zoom ceiling | here |
//!
//! ## ★★★ Why the landing cannot happen in the apply phase
//!
//! Arriving needs the canvas rectangle, the page's drawn extent and the scroll
//! offset. None of those exists where actions are applied, so the action parks
//! a [`crate::app::state::PendingDestination`] and this drains it on the next
//! frame — `OpenDoc::fit_placement`'s own pattern, for its own reason.
//!
//! ## ★★ It frames through `zoom::zoom_to_rect`, which is the zoom marquee's
//! ## own code
//!
//! Deliberately, and it is what makes the fix trustworthy rather than merely
//! close: a bookmark and a rubber band drawn over the same region arrive
//! **identically**. Two framings that agreed approximately would drift, and the
//! drift would present as a bookmark that lands *nearly* right — harder to
//! diagnose than one that does not move at all.
//!
//! ## ★ A point is framed as a region, not scrolled to
//!
//! `/XYZ` names a single coordinate, and framing a point has no answer — a zoom
//! onto zero area is either everything or nothing. `zoom_to_rect` already
//! solves *"put this on screen"* against the viewport, the margin and the
//! ceiling; adding a second "scroll to a point" solver would be two answers to
//! one question, and they would disagree at the edges of the page where it
//! matters most.

use egui::Context;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::zoom;

/// **A place on a page a bookmark asked the view to arrive at.**
///
/// Two shapes, because §12.3.2.2's five destination views reduce to exactly two
/// things a viewport can do: put a point at the top-left, or frame a rectangle.
/// The fits — `/Fit`, `/FitH`, `/FitV` — travel as an ordinary `Action::Fit`
/// beside one of these rather than as more variants here, so this type stays
/// about POSITION and the shell's fit vocabulary stays in one place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PendingDestination {
    /// Put this PDF-space point at the view's top-left.
    ///
    /// `None` on an axis is §12.3.2.2's null — *"leave this one as it is"*. A
    /// literal `0.0` is a real coordinate and is NOT that: Table 151 states the
    /// zero-means-null equivalence for `zoom` alone.
    Point {
        /// The page the coordinates belong to.
        page: usize,
        /// PDF-space left edge.
        left: Option<f64>,
        /// PDF-space top edge.
        top: Option<f64>,
    },
    /// Frame this PDF-space rectangle, `(left, bottom, right, top)`.
    Rect {
        /// The page the rectangle belongs to.
        page: usize,
        /// The rectangle.
        rect: (f64, f64, f64, f64),
    },
}

/// The margin a destination is framed with — the same constant the zoom
/// marquee uses, from the same place, so the two cannot drift apart.
use super::CANVAS_MARGIN;

/// Drain a parked destination, if there is one, and land on it.
///
/// ★★ A ONE-SHOT — `take()`, not a read. A destination that survived its frame
/// would fight every subsequent pan, and the operator would find the view
/// springing back to a bookmark they clicked a minute ago.
pub(crate) fn arrive(
    ctx: &Context,
    doc: &mut OpenDoc,
    max_zoom_percent: f32,
    actions: &mut Vec<Action>,
) {
    let Some(pending) = doc.pending_destination.take() else {
        return;
    };
    let page_index = doc.view.page_index;
    // ★ The page is CHECKED rather than trusted. The action queue turns the
    // page and parks the destination in the same drain, so both land before
    // this frame — but a destination for another sheet would frame this one's
    // coordinates, which is the mirrored-landing class of defect this canvas
    // has produced before from a different cause.
    let region = match pending {
        PendingDestination::Rect { page, rect } if page == page_index => doc
            .pages
            .get(page)
            .and_then(|p| crate::canvas::geometry::pdf_rect_to_canvas(rect, p)),
        PendingDestination::Point { page, left, top } if page == page_index => doc
            .pages
            .get(page)
            .and_then(|p| crate::canvas::geometry::pdf_point_to_canvas_region(left, top, p)),
        // A destination for a page this frame is not showing. Dropped rather
        // than deferred: a mismatch means something reordered the queue, and
        // framing the wrong sheet's coordinates is worse than not moving.
        _ => None,
    };
    // ★★ Traced whether or not it lands. A destination that was parked and then
    // dropped — wrong page, un-invertible geometry — is indistinguishable from
    // one that was never raised, and the two send a reader to opposite places.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "destination-arrive page={page_index} framed={} pending={pending:?}",
            region.is_some()
        )
    });
    if let Some(region) = region {
        let _ = zoom::zoom_to_rect(ctx, doc, region, CANVAS_MARGIN, max_zoom_percent, actions);
    }
}
