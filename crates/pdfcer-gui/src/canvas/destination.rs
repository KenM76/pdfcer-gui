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
//! ## ★★★ And it must not frame until the canvas is drawing the right page
//!
//! The half added on 2026-09-06, for `DEFECTS.md` **D23**. The scroll offset
//! moves on the frame the page turns; the strip's *visible set* does not — it
//! was chosen from the offset the frame inherited. So on the frame a
//! destination arrives, the canvas is scrolled to the new page and still
//! drawing the old one, and `zoom::frame_rect` plans the framing against a
//! [`crate::canvas::zoom::CanvasFrame`] that is about **another sheet**. The
//! anchor it builds carries that sheet's index, `consume_anchor` solves it
//! exactly, and the operator is put back on the page they left, magnified.
//!
//! [`arrive_step`] is the gate, and it carries the whole argument — including
//! why the repair is a bounded wait on the *destination* path rather than a
//! change to the framing every zoom in the product shares.
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

impl PendingDestination {
    /// **The page this destination is about** — the one field both variants
    /// share, and the one [`arrive_step`] is a decision about.
    #[must_use]
    pub const fn page(&self) -> usize {
        match self {
            Self::Point { page, .. } | Self::Rect { page, .. } => *page,
        }
    }
}

/// The margin a destination is framed with — the same constant the zoom
/// marquee uses, from the same place, so the two cannot drift apart.
use super::CANVAS_MARGIN;

/// `egui::Memory` key for how many frames a parked destination has already
/// waited for the page it names to be laid out. See [`arrive_step`].
const WAITED_MEMORY_KEY: &str = "pdfcer-canvas-destination-waited"; // ui-text-exempt: internal memory id, never displayed

/// **How many frames a destination may wait for its own page.** See
/// [`arrive_step`] for the whole argument; the number itself is measured, not
/// chosen: the driven trace of `a_link_goes_to_the_page_it_names` shows the
/// strip laying the destination page out on the **first** frame after the
/// scroll moves, so one would do. Four is that one plus room for the two
/// one-shots that can each own a frame ahead of it — a fit's placement (rank 3
/// in [`crate::canvas::offset`]) and a page command's scroll (rank 6) — and it
/// is still only about 65 ms, far short of a spring-back the operator could
/// mistake for their own gesture being undone.
pub const MAX_WAIT_FRAMES: u32 = 4;

/// What [`arrive`] should do with a parked destination on this frame.
///
/// The mirror of [`crate::canvas::zoom::AnchorStep`], and deliberately so: that
/// one is the *output* side of the same two-frame problem — the display not
/// having settled when the anchor is solved — and until 2026-09-06 the **input**
/// side had no equivalent at all. See [`arrive_step`] for what that cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArriveStep {
    /// The canvas's frame record describes the destination's own page, so the
    /// framing can be planned against geometry that is actually about it.
    Frame,
    /// The page turn has landed in the *view* but not yet in the *layout*.
    /// Leave the destination parked and try again next frame.
    Hold,
    /// Give up: the view has moved to another sheet, or the page never
    /// appeared. Consume the destination without moving anything.
    Drop,
}

/// ★★★ **The gate that stops a destination being framed against another page's
/// geometry** — `DEFECTS.md` D23, and the fourth reading of it, which is the
/// one the trace forces.
///
/// # What actually happened, in the numbers the trace prints
///
/// Clicking a `/GoTo … /FitH` link that names page 4 landed the operator back
/// on page 1, magnified 312 %. Three readings of that were wrong before this
/// one — *"the destination defaulted to index 0"*, *"the `Fit(Width)`
/// re-places the view"*, and *"`last_frame` is the PREVIOUS frame"* — and the
/// line that settles it is a single self-contradictory `canvas` trace:
///
/// ```text
/// destination-arrive page=3 framed=true pending=Point { page: 3, … }
/// canvas-zoom to=rect requested=3.1200 applied=3.1200 clamped=false
/// canvas rect=[[296.0 -1678.7] - [764.0 -1073.0]] zoom=0.7647 page=3 … off=[484.0 2436.8] visible=1
/// ```
///
/// `page=` on that line is `view.page_index` and `rect=` is the **acting
/// page's** rectangle. The view says page 3; the rectangle is 1,844 px above
/// the viewport, which at 0.7647× is exactly `3 × (792 + 12)` — *page 0's*
/// rectangle, seen from page 3's scroll position. `visible=1`: one page was
/// laid out that frame, and it was the wrong one.
///
/// ⇒ **The scroll offset moves on the frame the page turns; the strip's
/// visible set does not.** [`crate::canvas::strip`] chooses which pages to lay
/// out from the offset the frame *inherited*, so on the frame a destination
/// arrives the canvas is scrolled to the new page while still drawing the old
/// one. [`crate::canvas::zoom::CanvasFrame`] is written from that layout — its
/// `page`, `extent`, `display` and `map` are all about page 0 — and
/// `zoom::frame_rect` plans the framing against it. `place_centred` stamps
/// `page: 0` into the anchor, `consume_anchor` solves it faithfully a frame
/// later, and `offset::decide` converts the answer through **page 0's** origin
/// in the strip. The view lands on page 0 at 3.12×, which is precisely what
/// the operator sees.
///
/// ★ Note what is NOT wrong, because two readings died on each: the
/// destination resolves, the page turn happens, the arrival matches, the
/// anchor arithmetic is exact, and `last_frame` is this frame's record (
/// `zoom::remember_frame` runs in `canvas::present` *before* `interact`, which
/// is where `arrive` is called from). Every part works. They are handed a
/// frame that is about a different sheet.
///
/// # Why the repair is a HOLD and not a re-read of the geometry
///
/// D23 names two candidate repairs and warns they are not equivalent. The
/// other one — give `frame_rect` fresher geometry — is the dangerous one:
/// `frame_rect` is shared by the zoom marquee, `view.zoom_selection` and every
/// bookmark, and the only "fresher" geometry available inside the draw is
/// geometry the draw is still deciding. Feeding a layout back into a
/// fit-to-viewport zoom is `PROJECT_PLAN.md`'s **R128** exactly, and this
/// canvas has already recorded that loop twice. It would also be a change to
/// three surfaces in order to fix one.
///
/// This gate touches only the destination path: it decides *whether* to call
/// the shared framing at all, on a condition — "is the frame record about my
/// page?" — that is true on every frame for every other caller, because a
/// marquee and a selection are by construction on the page being drawn.
///
/// # The three answers
///
/// | condition | step | why |
/// |---|---|---|
/// | the view is on another sheet | [`ArriveStep::Drop`] | the previous behaviour, unchanged: something reordered the queue, and framing the wrong sheet's coordinates is worse than not moving |
/// | the frame record is about the destination's page | [`ArriveStep::Frame`] | the ordinary case, and the *only* case that existed before — a bookmark whose page is already on screen takes this arm on its first frame, so nothing about bookmarks changes |
/// | it is about some other page, or there is no record yet | [`ArriveStep::Hold`], until `waited` reaches [`MAX_WAIT_FRAMES`], then [`ArriveStep::Drop`] | the page turn is in flight; one more frame and the strip will have laid it out |
///
/// ★★ **Bounded, like `AnchorStep`, and for the same reason.** A destination
/// held indefinitely would be spent much later on an unrelated layout change —
/// a window resize, a mode switch — as a view that springs to a bookmark
/// clicked a minute ago. That is worse than not arriving.
#[must_use]
pub fn arrive_step(
    pending_page: usize,
    view_page: usize,
    frame_page: Option<usize>,
    waited: u32,
) -> ArriveStep {
    if pending_page != view_page {
        return ArriveStep::Drop;
    }
    match frame_page {
        Some(page) if page == pending_page => ArriveStep::Frame,
        _ if waited < MAX_WAIT_FRAMES => ArriveStep::Hold,
        _ => ArriveStep::Drop,
    }
}

/// Drain a parked destination, if there is one, and land on it.
///
/// ★★ A ONE-SHOT — consumed on the frame it is acted on, never left standing.
/// A destination that survived its frame would fight every subsequent pan, and
/// the operator would find the view springing back to a bookmark they clicked a
/// minute ago. The one exception is [`ArriveStep::Hold`], which is bounded by
/// [`MAX_WAIT_FRAMES`] precisely so that it cannot become that.
pub(crate) fn arrive(
    ctx: &Context,
    doc: &mut OpenDoc,
    max_zoom_percent: f32,
    actions: &mut Vec<Action>,
) {
    let waited_id = egui::Id::new(WAITED_MEMORY_KEY);
    // ★ READ, not `take()`. Whether this destination is spent is
    // [`arrive_step`]'s answer, and taking it first would make `Hold`
    // unexpressible — the frame that decided to wait would already have thrown
    // away the thing it was waiting for.
    let Some(pending) = doc.pending_destination else {
        // Nothing parked: the counter belongs to no destination, so it is
        // reset here rather than left for the next one to inherit.
        ctx.data_mut(|d| d.insert_temp(waited_id, 0u32));
        return;
    };
    let page_index = doc.view.page_index;
    let waited = ctx.data(|d| d.get_temp::<u32>(waited_id).unwrap_or(0));
    // ★★★ **The page the canvas's frame record is actually about** — the whole
    // of D23's fourth reading in one line. `zoom::frame_rect` will plan against
    // this record; if it describes another sheet, everything downstream of it
    // is arithmetic about the wrong page. See [`arrive_step`].
    let frame_page = zoom::last_frame(ctx).map(|frame| frame.page);
    let step = arrive_step(pending.page(), page_index, frame_page, waited);
    let region = match step {
        ArriveStep::Frame => match pending {
            PendingDestination::Rect { page, rect } => doc
                .pages
                .get(page)
                .and_then(|p| crate::canvas::geometry::pdf_rect_to_canvas(rect, p)),
            PendingDestination::Point { page, left, top } => doc
                .pages
                .get(page)
                .and_then(|p| crate::canvas::geometry::pdf_point_to_canvas_region(left, top, p)),
        },
        // A destination still in flight, or one being given up on: nothing to
        // frame this frame.
        ArriveStep::Hold | ArriveStep::Drop => None,
    };
    // ★★ Traced on EVERY step, whether or not it lands. A destination that was
    // parked and then dropped — wrong page, un-invertible geometry, a page that
    // never appeared — is indistinguishable from one that was never raised, and
    // the two send a reader to opposite places. `step=` and `frame_page=` are
    // here because D23 cost three wrong readings for want of exactly them: the
    // old line said `page=3 framed=true` on a frame whose geometry was page 0's,
    // and read as a success.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "destination-arrive page={page_index} step={step:?} frame_page={frame_page:?} \
             waited={waited} framed={} pending={pending:?}",
            region.is_some()
        )
    });
    match step {
        ArriveStep::Hold => {
            ctx.data_mut(|d| d.insert_temp(waited_id, waited + 1));
            // ★ The next frame is REQUESTED, not assumed. A reactive shell that
            // idles because nothing moved would hold the destination until the
            // operator jogged the mouse, and it would then land — correctly, and
            // seconds after the click, which reads as the view lurching on its
            // own.
            ctx.request_repaint();
        }
        ArriveStep::Frame | ArriveStep::Drop => {
            doc.pending_destination = None;
            ctx.data_mut(|d| d.insert_temp(waited_id, 0u32));
            if let Some(region) = region {
                // ★ The outcome is reported by `zoom::zoom_to_rect` itself, on
                // the same channel, as `canvas-zoom to=rect …` — so it is not
                // lost by being discarded here. The one variant this call site
                // could have acted on is `NoCanvas`, and it is now unreachable:
                // `arrive_step` returns `Frame` only when `last_frame` produced
                // a record, which is the same condition `frame_rect` declines
                // on.
                let _ =
                    zoom::zoom_to_rect(ctx, doc, region, CANVAS_MARGIN, max_zoom_percent, actions);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **D23 in one assertion.** The view has turned to page 3 and the
    /// canvas is still drawing page 0; framing now would plan a zoom against
    /// the wrong sheet's geometry and land the operator back where they were.
    #[test]
    fn a_destination_whose_page_is_not_yet_drawn_waits_for_it() {
        assert_eq!(arrive_step(3, 3, Some(0), 0), ArriveStep::Hold);
    }

    /// The ordinary case, and the only one that existed before: the page the
    /// destination names is the page on screen. A bookmark on the current
    /// sheet — which is every bookmark in a drawing package — takes this arm on
    /// its first frame, so nothing about bookmarks changed.
    #[test]
    fn a_destination_on_the_drawn_page_frames_immediately() {
        assert_eq!(arrive_step(3, 3, Some(3), 0), ArriveStep::Frame);
        assert_eq!(arrive_step(0, 0, Some(0), 0), ArriveStep::Frame);
    }

    /// ★★ The wait is BOUNDED. A destination held for ever would be spent on
    /// some later layout change as a view springing to a bookmark clicked a
    /// minute ago.
    #[test]
    fn a_page_that_never_appears_is_dropped_rather_than_held_for_ever() {
        assert_eq!(
            arrive_step(3, 3, Some(0), MAX_WAIT_FRAMES - 1),
            ArriveStep::Hold
        );
        assert_eq!(
            arrive_step(3, 3, Some(0), MAX_WAIT_FRAMES),
            ArriveStep::Drop
        );
    }

    /// The pre-existing guard, unchanged: a destination for a sheet the view is
    /// no longer on is dropped at once, not waited for. Waiting would let a
    /// destination survive the operator navigating away from it.
    #[test]
    fn a_destination_for_another_sheet_is_dropped_at_once() {
        assert_eq!(arrive_step(3, 1, Some(1), 0), ArriveStep::Drop);
        assert_eq!(arrive_step(3, 1, Some(3), 0), ArriveStep::Drop);
    }

    /// Before the canvas has ever drawn, there is no record to plan against —
    /// the same `None` every entry point in [`crate::canvas::zoom`] declines
    /// on. Held, not dropped: the first frame is exactly when a document opened
    /// straight onto a destination would arrive.
    #[test]
    fn no_canvas_record_yet_is_a_wait_not_a_drop() {
        assert_eq!(arrive_step(0, 0, None, 0), ArriveStep::Hold);
        assert_eq!(arrive_step(0, 0, None, MAX_WAIT_FRAMES), ArriveStep::Drop);
    }

    /// Both variants answer the question [`arrive_step`] asks them.
    #[test]
    fn both_destination_shapes_name_their_page() {
        assert_eq!(
            PendingDestination::Point {
                page: 7,
                left: None,
                top: Some(1.0)
            }
            .page(),
            7
        );
        assert_eq!(
            PendingDestination::Rect {
                page: 2,
                rect: (0.0, 0.0, 1.0, 1.0)
            }
            .page(),
            2
        );
    }
}
