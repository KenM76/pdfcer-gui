//! # `canvas::fit` — **where the view goes when the viewport changes, or a fit
//! is pressed**
//!
//! ★★★ **The subject widened on 2026-08-31** (`OPERATOR_REQUESTS.md` O78) and
//! the old title — *"spending a fit command's request to place the view"* — is
//! kept above the new one because the widening is the finding.
//!
//! The operator:
//!
//! > *"when I change the size of the canvas window, whatever area was centered
//! > in the current canvas should stay centered, and unless I have manually
//! > changed the zoom after clicking one of the preset options, the pdf should
//! > maintain whichever option was selected."*
//!
//! ## ★★★ Preserving the centre SUBSUMES a fit's re-placement
//!
//!
//! On an axis a fit **pins**, the page is by construction no larger than the
//! viewport, so `margin = (v − d) / 2`, and holding the page's own centre at
//! the viewport centre gives
//! `(v − d)/2 + 0.5·d − v/2 = 0` — **exactly** what
//! [`crate::canvas::geometry::fit_placement_offset`] returns for a pinned
//! axis. So a fit-page document nobody has panned is re-centred by the general
//! rule for free, and `centring_agrees_with_the_pinned_fit_answer` pins that
//! equality so deleting the old path cannot silently change what Fit page
//! does.
//!
//!
//! ## The original subject, unchanged below this line
//!
//! # `canvas::fit` — spending a fit command's request to place the view
//!
//! ## The request
//!
//!
//! > *"If I press the Fit width or fit page button the view should center to
//! > the width as well or center the page."*
//!
//! ## ★★★ Why a fit is now a position as well as a scale
//!
//! Before O23's pasteboard a page no larger than the viewport had nowhere to
//! be except the middle, so *fit* and *centred* were the same act and the
//! button never had to choose between them. The pasteboard added a whole
//! viewport of slack on every side — deliberately, so any corner of the page
//! can be brought to any point of the screen — and with it the state the
//! operator is reporting: **the scale is right and the page is not on
//! screen.**
//!
//! ## The two-frame handshake, and why it is the same one the zoom anchor uses
//!
//! `Action::Fit` cannot place the view itself: the re-fitted zoom is computed
//! by `ViewState::apply_fit` from a viewport the action funnel cannot see, so
//! the page's new drawn size is not known until the canvas next runs. So the
//! action records the request on [`crate::app::state::OpenDoc::fit_placement`]
//! and this module spends it on the following frame, by which time
//! `apply_fit` has run near the top of `show_in` and `current_display` is the
//! page's **new** size.
//!
//! That is exactly the shape [`crate::canvas::zoom`]'s anchor uses, for
//! exactly the same reason, and the resemblance is not a coincidence worth
//! collapsing: an anchor says *"hold this page point where it is"* and a fit
//! says *"decide where the page goes"*, which on a pinned axis is the
//! statement that there is no previous position worth holding.
//!
//! ## Where the rules live, and why not here
//!
//! * *Which axes does this mode pin?* — [`crate::viewer::FitMode::pinned_axes`],
//!   beside `fit_scale`, because the two are one decision: an axis is pinned
//!   exactly when the fit has just decided its extent.
//! * *What offset does a pinned or unpinned axis get?* —
//!   [`crate::canvas::geometry::fit_placement_offset`], beside the other
//!   offset solves and the `margin` term whose definition makes the pinned
//!   answer a single constant.
//!
//! This file is the **frame plumbing** between them: read where the view is
//! now, ask the two rules, hand back an offset. Keeping it separate is what
//! stops either rule being restated in `canvas::show`.

use egui::{Rect, Vec2, vec2};

use crate::app::state::OpenDoc;
use crate::canvas::geometry;

/// How far the viewport must move, on either axis, before it counts as a
/// **resize** that re-places the view.
///
///
///
/// # Why half a point, and why the comparison is STRICTLY greater
///
/// The worst jitter measured is exactly 0.5 pt (`444.0` → `444.5` in the
/// driven trace), so `>=` would have let that one frame through — the first
/// test below was written with that value in its series and went red on `>=`
/// before the operator ever could. Smaller than any resize an operator can
/// make: a dock splitter moves in whole
/// points, a window edge in whole physical pixels, and a panel collapse by
/// its whole width. Nothing an operator does lands in the gap. It is a
/// **floor**, not a tolerance on the placement arithmetic — once the gate
/// opens the placement is exact.
const RESIZE_FLOOR_PT: f32 = 0.5;

/// ★★★ **Where the view should go, AND which layout unit that offset is
/// relative to** — `OPERATOR_REQUESTS.md` O177, second half.
///
/// The operator:
///
/// > *"fit page when in 2 pages side by side views should fit the two side by
/// > side pages onto the canvas - right now it snaps to fitting one."*
///
/// ## Why a returned offset now has to say what it is measured against
///
/// The fit's **scale** has been row-aware since facing modes shipped —
/// [`crate::viewer::strip::Strip::row_extent`] says so in its own doc,
/// *"fitting one page of a spread would leave the other half off screen"*. The
/// fit's **placement** was not: the offset came back as a *page*-local number
/// and `canvas::offset` converted it through the acting page's rect. So a
/// spread was scaled to fit two pages and then positioned as though it were
/// one, and half of it sat off the canvas.
///
/// The general lesson, which is the part worth carrying to the next feature:
/// **when a rule about SCALE learns about a new layout unit and the matching
/// rule about PLACEMENT does not, the symptom presents as the scale being
/// wrong.** The operator reported a fit that "snaps to fitting one page"; the
/// scale was already correct.
///
/// ## Why an enum rather than always returning row-local
///
/// Because the two arms below genuinely want different units, and collapsing
/// them would be a silent behaviour change on the arm that is not about O177:
///
/// * [`Self::Row`] — a **pressed fit** and a **page-display recentre**. Both
///   are the operator saying *"put the thing I am looking at in the middle"*,
///   and under a facing mode the thing they are looking at is the spread.
/// * [`Self::Page`] — the **resize** arm, which preserves whatever was in the
///   middle across a viewport change. It must stay page-based, and that is a
///   theorem rather than a preference: when the unit fits the viewport at both
///   ends, `margin = (v - d)/2`, so [`geometry::centred_frac`] reduces to
///   `u = off/d + 0.5` and [`geometry::offset_holding_anchor_at`] to
///   `off' = off*d'/d`. The gap between the row's centre and the page's centre
///   scales with the zoom by exactly the same factor, so row-centring is
///   preserved **exactly** by the page-based rule. Teaching this arm about rows
///   would buy nothing and would cost `CanvasFrame` a row field it has no other
///   use for — its reason to exist is the zoom anchor, which genuinely wants
///   the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Placed {
    /// Relative to the acting **page**'s rect within the strip.
    Page(Vec2),
    /// Relative to the acting page's **row** — the page itself under
    /// `PageDisplay::Single` and `PageDisplay::Continuous`, the whole facing
    /// spread under either facing mode.
    Row(Vec2),
}

/// **Spend a pending fit request and return where the view should go**, as an
/// offset plus the unit it is measured against, or `None` on the overwhelming
/// majority of frames where nothing is pending.
///
/// ★ The request is **taken** whatever happens — including on a frame at the
/// deep-zoom tier, and on one where something else wins the scroll offset. A
/// request left pending would fire on whatever frame the caller's chain next
/// reached it, which the operator experiences as the view jumping for a button
/// they pressed some seconds ago. That is the failure mode `zoom::consume_anchor`'s
/// own `Drop` step exists to prevent, and it is prevented here the same way.
///
/// # Arguments
///
///
pub(super) fn placement(
    doc: &mut OpenDoc,
    current_display: (f32, f32),
    row_rect: Rect,
    display_size: Vec2,
    vp: Vec2,
    // ★ The PREVIOUS frame's geometry — `zoom::last_frame` — which is the
    // whole "before" state the centre measurement needs: the page-local
    // offset it settled on, the size the page was drawn at, and the viewport
    // it was measured against. `None` on the first frame of a document, which
    // this function declines rather than guesses at. O78.
    before: Option<crate::canvas::zoom::CanvasFrame>,
    // The page being acted on, so a `before` describing a different document
    // can be declined. See the check below.
    page_index: usize,
) -> Option<Placed> {
    let row_display = (row_rect.width(), row_rect.height());
    //
    // > *"if the canvas window is resized the pdf should resize to match"*
    //
    // ## What was here before, and the exact half it was missing
    //
    // This read `doc.fit_placement.take()?` alone — a **one-shot**, set by
    // `Action::Fit` and spent on the following frame. So the page was placed
    // when the operator pressed the button, and never again.
    //
    // `ViewState::apply_fit` meanwhile recomputes the **zoom** from the
    // viewport on *every* frame a fit mode is active, which is why a resize
    // already re-scaled correctly. Nothing re-placed it. The page therefore
    // grew or shrank about whatever offset it happened to be sitting at, and
    // drifted off centre — the scale right, the position stale, which is the
    // same pair O28 was about arriving through a different door.
    //
    // ⇒ **A fit is a MODE, so its placement is a mode too.** Recomputing the
    // scale every frame and the position once is the inconsistency the
    // operator was looking at.
    //
    // ## ★★ Why the one-shot survives rather than being replaced
    //
    // Because it is still the thing that fires on a frame where the mode was
    // *already* active — pressing **Fit page** while already fitted to page
    // must still recentre a view the operator has panned away, and the mode
    // alone cannot distinguish that frame from the sixty before it.
    //
    // ★ It is `take`n whatever happens, for the reason this function's own
    // note gives: a request left pending fires on some later frame and reads
    // as the view jumping for a button pressed seconds ago.
    let pending = doc.fit_placement.take();
    // ★★★ **The other one-shot: the operator changed the page arrangement** —
    // `OPERATOR_REQUESTS.md` O177, first half.
    //
    // > *"when switching the view from scroll pages to show one page at a time
    // > or show two pages side by side the page or pages view should snap back
    // > to center of the canvas."*
    //
    // Set by `Action::SetPageDisplay` and spent here, for the same two-frame
    // reason the fit request has: the arrangement changes during the action
    // funnel, and the new layout's drawn size is not known until the canvas
    // next lays the strip out.
    //
    // ★ Taken **unconditionally**, on the same argument as `pending` above and
    // in the same breath so the two cannot drift: a request left pending fires
    // on whatever frame the chain next reaches it, which the operator
    // experiences as the view jumping for a button pressed seconds ago.
    let recentre = std::mem::take(&mut doc.recentre);
    // ★★★ **A RESIZE, NOT A FRAME**, and the difference is a regression that
    // was written, run and caught the same hour.
    //
    // Re-placing on **every** frame while a fit is active is the obvious
    // reading of *"a fit is a mode, so its placement is a mode"*, and it is
    // wrong: under **Fit page** both axes are pinned, so the placement returns
    // the page's origin on every frame and **the wheel cannot scroll at all**.
    // In a continuous display that makes the document unnavigable, because the
    // wheel is how the next page is reached.
    //
    // ⇒ Caught by `a_fit_command_puts_the_page_on_screen`'s own precondition,
    // which scrolls into the pasteboard and **asserts it got there** before
    // pressing anything. It reported *"the pan did not move the page"* and
    // SKIPPED — a setup step refusing to proceed rather than a subject
    // failing, which is the shape a precondition is supposed to have and the
    // reason that check was written to establish its own.
    //
    // ★ The operator's sentence says it exactly: *"if the canvas window is
    // **resized** the pdf should resize to match"*. Resized, not redrawn.
    //
    //
    // ★★★ Somebody has, and it was measured rather than argued: the central
    // panel's width oscillates by **0.1–0.5 pt from frame to frame** with no
    // dock, ribbon or window change (`central-panel rect max.x` 732.0 / 732.3
    // / 732.4 … in a driven trace; 1072.0 / 1072.2 in an off-screen smoke
    // launch). Its source is not yet run down. Its effect was that this gate
    // stood OPEN on every frame — so the frame after the canvas's scroll bars
    // became solid, when the measure and place halves of this rule briefly
    // disagreed about the viewport's width, the page walked off the screen at
    // 7.4 px per frame. See `RESIZE_FLOOR_PT`. That is R128's rule from the
    // other side: a guard against repetition is not a guard against creep, and
    // an exact comparison against a jittering input is no guard at all.
    //
    // ⇒ The reference is only re-recorded when it moves by at least the floor,
    // so a jitter smaller than the floor compares against a FIXED value and
    // can never accumulate into a resize; a real resize — a dock drag, a
    // window edge, a panel collapsing — is tens to hundreds of points and
    // clears the floor on its first frame.
    //
    //
    // That was worse than "the scroll offset is kept in pixels". On a single
    // page `page_local_offset` reduces to `page_local = scroll − viewport`,
    // because the pasteboard is exactly one viewport — so holding the scroll
    // offset while the viewport grows by Δ slides the page across the screen
    // by the **whole** of Δ. Widening a dock threw the operator's position
    // away, and that is the report.
    let changed = doc.view_viewport.is_none_or(|(x, y)| {
        (x - vp.x).abs() > RESIZE_FLOOR_PT || (y - vp.y).abs() > RESIZE_FLOOR_PT
    });
    // Recorded on EVERY frame that clears the floor, whatever this function
    // goes on to decide.
    //
    // ★ Including the frames it declines — no previous frame, a different
    // document, a degenerate viewport. A frame that declined without recording
    // would leave the NEXT frame reading as a resize and moving the view for
    // nothing, which is the one way this can produce a jump the operator did
    // not cause.
    //
    // ★ And NOT on a frame under the floor: re-recording a jittered value
    // would let a 0.3 pt wobble walk the reference one step per frame, which
    // is exactly the creep the floor exists to stop.
    if changed {
        doc.view_viewport = Some((vp.x, vp.y));
    }

    // ---- 1. a pressed fit outranks everything --------------------------
    //
    // Pressing **Fit page** while already fitted to page must recentre a view
    // the operator has panned away from, and the mode alone cannot distinguish
    // that frame from the sixty before it. This is still the only path that
    // deliberately discards the operator's position, because pressing the
    // button is them asking for it.
    //
    // ★ Solved against the **row** as of O177, not the page. Under every
    // non-facing mode the row IS the page and this is the arithmetic it always
    // was; under a facing mode it is the fix — the scale was already fitting
    // two pages, and this is the half that puts both of them on screen.
    if let Some(mode) = pending
        && let Some(pinned) = mode.pinned_axes()
    {
        // Where the view is now, expressed the way the solve expects. The
        // PREVIOUS frame's settled offset, which is the only one available
        // before this frame's scroll area is built — and the correct one,
        // because nothing has moved the view since.
        let now = geometry::page_local_offset(
            (
                doc.frame.last_scroll_offset.x,
                doc.frame.last_scroll_offset.y,
            ),
            (row_rect.min.x, row_rect.min.y),
            (display_size.x, display_size.y),
            row_display,
            (vp.x, vp.y),
            (doc.pasteboard_overhang.x, doc.pasteboard_overhang.y),
        );
        let (x, y) = geometry::fit_placement_offset(pinned, now, row_display, (vp.x, vp.y));
        return Some(Placed::Row(vec2(x, y)));
    }

    // ---- 2. a page-display switch snaps back to the middle -------------
    //
    // `OPERATOR_REQUESTS.md` O177, first half. Below a pressed fit, because a
    // fit is a later and more specific instruction if both land on one frame,
    // and above the resize arm, because the resize arm's whole job is to
    // preserve a centre that the arrangement change has just invalidated: the
    // `before` frame it measures describes the OLD layout.
    //
    // ## Why this is needed at all, given the continuous strip is scrollable
    //
    //
    // ## Why the ROW and not the page
    //
    // Because *"show two pages side by side ... should snap back to center"* is
    // a sentence about the spread. See [`Placed`].
    if recentre {
        let (x, y) = geometry::offset_holding_anchor_at(
            (0.5, 0.5),
            (vp.x / 2.0, vp.y / 2.0),
            row_display,
            (vp.x, vp.y),
        );
        return Some(Placed::Row(vec2(x, y)));
    }

    // ---- 3. a resize keeps what was in the middle, in the middle -------
    if !changed {
        return None;
    }
    //
    // What stood here said: *"No previous frame is no centre to preserve, so
    // the very first frame of a document declines by construction and
    // `canvas::offset`'s seed arm does the placing."* Every clause of that is
    // still true about the **intent**. None of it was a guard. It leaned on
    // `before` — `zoom::last_frame` — happening to be `None` for the first two
    // frames of a document, which is true only while those frames draw
    // nothing: `canvas::present` returns early and publishes
    // `canvas-unavailable reason=nothing-visible` before it ever calls
    // `zoom::remember_frame`.
    //
    // O186's fix narrowed the pasteboard by `MIN_SHEET_ON_SCREEN`, which is
    // precisely a change to *how much sheet is on screen at an extreme
    // placement* — so frame 0 of a freshly opened document started drawing a
    // sliver instead of nothing. `remember_frame` then ran, `before` became
    // `Some`, and on frame 1 this arm outranked the seed and preserved the
    // "centre" of a frame whose scroll offset was egui's own default `(0,0)`:
    // the top-left corner of the pasteboard, one whole viewport above and left
    // of the page. `geometry::strip_offset`'s lower clamp turned that into a
    // request for `(0.0, 0.0)`, which is what the area was already at, so the
    // symptom was a document that opened with the page off the bottom-right
    // corner — and the seed arm, a one-shot keyed on a single frame index,
    // never fired at all. Measured by driving the binary; see
    // `canvas::trace::placed`, whose `src=` field is what made the two arms
    // distinguishable.
    //
    // ⇒ **The predicate is "has the view been placed", not "is there a
    // previous frame".** They are not the same question and the difference is
    // exactly one un-seeded frame. `SEED_FRAME` is shared with the arm that
    // does the seeding so the two cannot drift apart.
    //
    // ⚠ This is the LAST arm, deliberately. A pressed fit (arm 1) and a
    // page-display recentre (arm 2) do not read `before` at all — they place
    // from the viewport and the row — so neither needs the guard and neither
    // is weakened by it. Only a centre-*preservation* needs a centre that
    // somebody chose.
    if doc.canvas_frames <= crate::canvas::offset::SEED_FRAME {
        return None;
    }
    // A previous frame is still required on top of that: a document can reach
    // this arm with the seed long spent and still have no frame to measure —
    // the first frame after a tab switch writes the other document's. Kept
    // as its own decline rather than folded into the guard above, because it
    // is a different fact with a different lifetime.
    let before = before?;
    // ★★ …and a previous frame describing a DIFFERENT PAGE is declined rather
    // than corrected. `zoom::remember_frame` writes one global `egui::Id`, so
    // on the first frame after a document-tab switch it still describes the
    // other document. A tab switch alone does not change the viewport, so this
    // normally cannot fire — but a switch that coincides with a dock drag
    // would otherwise re-centre this document from that one's geometry.
    // Declining loses one frame of centre-preservation; correcting would mean
    // inventing a before-state, which is worse.
    if before.page != page_index {
        return None;
    }
    // Measure, then place. The two are exact inverses of each other and are
    // each other's tested pair; see `centred_frac`.
    let frac = geometry::centred_frac(before.offset, before.display, before.outer);
    let (x, y) = geometry::offset_holding_anchor_at(
        frac,
        (vp.x / 2.0, vp.y / 2.0),
        current_display,
        (vp.x, vp.y),
    );
    Some(Placed::Page(vec2(x, y)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::mapping::PageMapping;
    use crate::canvas::zoom::CanvasFrame;
    use egui::{Pos2, Rect, vec2};

    const FIXTURE: &str = "annots-with-everything.pdf";

    /// The offset out of a [`Placed`], for the tests that only care about the
    /// number. Which arm produced it is asserted separately where it matters.
    fn offset_of(placed: Placed) -> Vec2 {
        match placed {
            Placed::Page(v) | Placed::Row(v) => v,
        }
    }

    /// **Move the document past the open-seed frame.**
    ///
    /// Every resize test below is about a document that has been on screen for
    /// a while and is *then* resized — a dock drag, a window edge, a panel
    /// collapsing. That world has a canvas-frame count well past
    /// [`crate::canvas::offset::SEED_FRAME`], and the resize arm declines
    /// outright below it (see the guard).
    ///
    /// ★ Spelled as its own call in each test rather than folded into
    /// [`place_at`], deliberately. Folding it in would make every test in this
    /// module unable to observe the guard at all, and
    /// [`the_resize_arm_declines_until_the_seed_has_placed_the_view`] — the
    /// test that exists *because* the guard was missing — would be testing a
    /// world the helper had already made impossible.
    fn settled(doc: &mut OpenDoc) {
        doc.canvas_frames = crate::canvas::offset::SEED_FRAME + 4;
    }

    /// A settled single-page frame: the page drawn at `zoom`, centred in a
    /// `viewport`-sized area. The world every placement test needs and none
    /// of them cares about beyond "there was a previous frame on this page".
    fn frame(zoom: f32, viewport: (f32, f32)) -> CanvasFrame {
        let extent = (612.0_f32, 792.0_f32);
        let display = (extent.0 * zoom, extent.1 * zoom);
        let image_rect = Rect::from_min_size(Pos2::new(40.0, 20.0), vec2(display.0, display.1));
        CanvasFrame {
            map: PageMapping::new(image_rect, extent, zoom),
            extent,
            display,
            viewport,
            outer: viewport,
            viewport_rect: Rect::from_min_size(Pos2::new(0.0, 0.0), vec2(viewport.0, viewport.1)),
            offset: (0.0, 0.0),
            page: 0,
        }
    }

    fn place(doc: &mut OpenDoc, vp: (f32, f32), before: Option<CanvasFrame>) -> Option<Placed> {
        place_at(doc, 0.5, vp, before)
    }

    /// ★ A **single** page, so the row and the page are the same rect — these
    /// tests are about the resize gate, not about O177's layout unit, and
    /// passing the page rect twice is the truthful modelling of Single rather
    /// than a convenience. A facing spread is exercised by driving the real
    /// binary; see `ui-verify`'s
    /// `switching_the_page_display_recentres_and_a_facing_fit_fits_the_spread`.
    fn place_at(
        doc: &mut OpenDoc,
        zoom: f32,
        vp: (f32, f32),
        before: Option<CanvasFrame>,
    ) -> Option<Placed> {
        let display = vec2(612.0 * zoom, 792.0 * zoom);
        let rect = Rect::from_min_size(Pos2::ZERO, display);
        placement(
            doc,
            (display.x, display.y),
            rect,
            display,
            vec2(vp.0, vp.1),
            before,
            0,
        )
    }

    /// ★★★ **The jitter that fed R128.** A viewport that wobbles by less than
    /// the floor from frame to frame is not a resize, and must not re-place
    /// the view on every frame — that is the gate that stood open while the
    /// page crept 7.4 px per frame on 2026-09-08.
    ///
    /// The wobble is the measured one: ±0.2–0.4 pt around a fixed width.
    #[test]
    fn a_sub_pixel_wobble_is_not_a_resize() {
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        // ★ Past the seed, or every `None` below would be the seed guard's
        // answer rather than the wobble gate's and this test would pass
        // without ever exercising its subject.
        settled(&mut doc);
        let before = frame(0.5, (444.0, 592.0));
        // First frame: no reference yet, so it records and declines to place
        // against a `before` — this is the seed arm's frame.
        assert_eq!(place(&mut doc, (444.0, 592.0), None), None);
        assert_eq!(doc.view_viewport, Some((444.0, 592.0)));

        for wobble in [444.4, 444.0, 444.1, 444.5, 444.0, 443.7] {
            assert_eq!(
                place(&mut doc, (wobble, 592.0), Some(before)),
                None,
                "★ a viewport {wobble} against a reference of 444.0 is a wobble, not a resize"
            );
            assert_eq!(
                doc.view_viewport,
                Some((444.0, 592.0)),
                "★ the reference must NOT follow the wobble — re-recording it is how a 0.3 pt \
                 jitter walks the view one step per frame"
            );
        }
    }

    /// A real resize — the smallest a dock splitter can make — clears the
    /// floor on its first frame, re-places, and moves the reference.
    #[test]
    fn a_one_point_resize_still_re_places() {
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        settled(&mut doc);
        let before = frame(0.5, (444.0, 592.0));
        assert_eq!(place(&mut doc, (444.0, 592.0), None), None);

        let placed = place(&mut doc, (445.0, 592.0), Some(before));
        assert!(
            placed.is_some(),
            "★ a whole-point resize must re-place the view"
        );
        assert_eq!(doc.view_viewport, Some((445.0, 592.0)));
    }

    /// The floor is a floor on the GATE, not a tolerance on the arithmetic:
    /// two placements for viewports that differ by exactly the floor differ
    /// by exactly half of it, as the centre rule says they must.
    #[test]
    fn once_the_gate_opens_the_placement_is_exact() {
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        settled(&mut doc);
        let before = frame(1.0, (444.0, 592.0));
        assert_eq!(place_at(&mut doc, 1.0, (444.0, 592.0), None), None);
        let a = offset_of(place_at(&mut doc, 1.0, (446.0, 592.0), Some(before)).expect("placed"));
        doc.view_viewport = Some((444.0, 592.0));
        let b = offset_of(place_at(&mut doc, 1.0, (448.0, 592.0), Some(before)).expect("placed"));
        assert!(
            ((a.x - b.x).abs() - 1.0).abs() < 1e-4,
            "★ half the viewport delta (2 pt) on the x offset: got {} vs {}",
            a.x,
            b.x
        );
    }

    /// ★★★ **The regression of 2026-09-13, in the smallest world that has it.**
    ///
    /// A freshly opened document reaches its second canvas frame with a
    /// `before` frame available — `zoom::remember_frame` ran on frame 0 the
    /// moment O186's narrower pasteboard let a sliver of sheet be drawn — and
    /// with a viewport that differs from frame 0's, because the canvas's
    /// scroll bars have just become solid. Both of the resize arm's original
    /// preconditions are therefore met on the one frame the open-seed arm owns.
    ///
    /// The arm must decline anyway. The "centre" it would preserve is
    /// `before.offset`, which on an unplaced frame is the `ScrollArea`'s own
    /// default `(0, 0)` — the top-left corner of the pasteboard, a whole
    /// viewport above and left of the page. Preserving it parks the document
    /// off the bottom-right corner of the canvas, and because the seed is a
    /// one-shot keyed on a single frame index, nothing ever places it again.
    ///
    /// ⚠ The two asserts are deliberately a pair. The first is the guard; the
    /// second is what makes the guard non-vacuous, by showing that the very
    /// same call with the document one frame further on *does* place. Without
    /// it a future change that made this arm decline for an unrelated reason
    /// would leave a green test asserting nothing.
    #[test]
    fn the_resize_arm_declines_until_the_seed_has_placed_the_view() {
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let before = frame(1.0, (444.0, 592.0));

        // Frame 0: records the viewport reference, declines for want of a
        // `before` — exactly as it always did.
        doc.canvas_frames = 0;
        assert_eq!(place_at(&mut doc, 1.0, (444.0, 592.0), None), None);

        doc.canvas_frames = crate::canvas::offset::SEED_FRAME;
        assert_eq!(
            place_at(&mut doc, 1.0, (446.0, 592.0), Some(before)),
            None,
            "★ the resize arm must not preserve a centre from a frame the seed has not placed \
             yet — that is the document opening off the bottom-right corner"
        );

        // One frame later the same call places, which is what proves the
        // assert above is about the seed and not about something else.
        doc.canvas_frames = crate::canvas::offset::SEED_FRAME + 1;
        doc.view_viewport = Some((444.0, 592.0));
        assert!(
            place_at(&mut doc, 1.0, (446.0, 592.0), Some(before)).is_some(),
            "★ once the view has been seeded the identical resize must place, or the guard above \
             is passing for a reason this test cannot see"
        );
    }
}
