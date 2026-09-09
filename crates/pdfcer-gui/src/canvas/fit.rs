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
//! This module used to have two jobs — spend a pending fit request, and
//! re-place the view when the viewport changed **while a fit was active**. The
//! second is now a special case of a general rule, and it is a theorem rather
//! than a convenience.
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
//! ⇒ A fit is now purely a rule about **zoom** — `ViewState::apply_fit`, run
//! every frame — and this is the one rule about **position**. They used to be
//! entangled, and the entanglement is why a pan had to leave the fit: a
//! re-placement would have thrown the operator's position away, so the only
//! defence available was to stop being in a fit. With position defended in its
//! own right that defence is unnecessary, which is why
//! [`crate::canvas::offset`]'s pan arm no longer calls `set_fit(FitMode::None)`.
//!
//! ## The original subject, unchanged below this line
//!
//! # `canvas::fit` — spending a fit command's request to place the view
//!
//! ## The request
//!
//! `OPERATOR_REQUESTS.md` O28, 2026-08-24:
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
/// # ★★★ Why a floor exists, measured on 2026-09-08
///
/// The central panel's width oscillated by 0.1-0.5 pt between consecutive
/// frames with nothing on screen changing. Run down on 2026-09-09: a solid
/// scroll bar fading in inside a dock body overshoots its pane by a
/// rounding residue, and `egui::Panel` answered by sliding the whole side
/// inward by that residue (`egui_shell::dock::overflow_probe`). The dock
/// now keeps a body's union out of its frame, so that source is gone; the
/// floor stays because it is the right contract regardless of source - a
/// sub-pixel change of viewport is not a resize an operator made - and a
/// future source would otherwise reach the view again. An exact
/// comparison reported a resize on **every
/// frame**, which was harmless only for as long as the measure and place
/// halves of the centre rule agreed exactly about the viewport's width. The
/// frame they disagreed — the canvas's scroll bars took real width and one
/// half was still reading the outer size — the page moved 7.4 px per frame
/// until it left the screen.
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

/// **Spend a pending fit request and return where the view should go**, as a
/// page-local offset, or `None` on the overwhelming majority of frames where
/// no fit is pending.
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
/// * `current_rect` — the acting page's rect **within the strip**, for its
///   origin. Under `PageDisplay::Single` that origin is `(0, 0)` and every
///   conversion below is the identity it always was.
/// * `current_display` — the acting page's drawn size, already re-fitted this
///   frame.
/// * `display_size` — the whole strip's drawn size.
/// * `vp` — the viewport measured before the scroll area was built, the same
///   measurement every margin term in [`geometry`] is derived against.
pub(super) fn placement(
    doc: &mut OpenDoc,
    current_rect: Rect,
    current_display: (f32, f32),
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
) -> Option<Vec2> {
    // ★★★ **A pending request OR a live fit mode**, and the second half is
    // `OPERATOR_REQUESTS.md` **O55**, 2026-08-28:
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
    // ★ Compared exactly rather than with a tolerance — UNTIL 2026-09-08. The
    // paragraph that stood here said a viewport that has not changed produces
    // bit-identical floats, and that a tolerance "would only decide how much
    // of a resize is allowed to be ignored, which is a question nobody has".
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
    // ★★★ **The comparison is now made on EVERY frame, whatever the fit** —
    // O78. It used to read
    // `doc.view.fit != FitMode::None && doc.fit_viewport != Some(...)`, so a
    // document that was not in a fit was never told the viewport had changed
    // and got no resize handling at all.
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
    // that frame from the sixty before it. Unchanged, byte for byte, from
    // before O78 — this is still the only path that deliberately discards the
    // operator's position, because pressing the button is them asking for it.
    if let Some(mode) = pending
        && let Some(pinned) = mode.pinned_axes()
    {
        // Where the view is now, expressed the way a single-page solve
        // expects. The PREVIOUS frame's settled offset, which is the only one
        // available before this frame's scroll area is built — and the correct
        // one, because nothing has moved the view since.
        let now = geometry::page_local_offset(
            (doc.last_scroll_offset.x, doc.last_scroll_offset.y),
            (current_rect.min.x, current_rect.min.y),
            (display_size.x, display_size.y),
            current_display,
            (vp.x, vp.y),
        );
        let (x, y) = geometry::fit_placement_offset(pinned, now, current_display, (vp.x, vp.y));
        return Some(vec2(x, y));
    }

    // ---- 2. a resize keeps what was in the middle, in the middle -------
    if !changed {
        return None;
    }
    // ★ No previous frame is no centre to preserve, so the very first frame of
    // a document declines by construction and `canvas::offset`'s seed arm does
    // the placing. That is also what keeps O23's first-frame hazard shut: this
    // path never runs against a layout that has not settled once.
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
    Some(vec2(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::mapping::PageMapping;
    use crate::canvas::zoom::CanvasFrame;
    use egui::{Pos2, Rect, vec2};

    const FIXTURE: &str = "annots-with-everything.pdf";

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

    fn place(doc: &mut OpenDoc, vp: (f32, f32), before: Option<CanvasFrame>) -> Option<Vec2> {
        place_at(doc, 0.5, vp, before)
    }

    fn place_at(
        doc: &mut OpenDoc,
        zoom: f32,
        vp: (f32, f32),
        before: Option<CanvasFrame>,
    ) -> Option<Vec2> {
        let display = vec2(612.0 * zoom, 792.0 * zoom);
        placement(
            doc,
            Rect::from_min_size(Pos2::ZERO, display),
            (display.x, display.y),
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
        // ★ Zoom 1.0, so the page (612 × 792) is LARGER than the viewport on
        // both axes. A page smaller than the viewport is centred by its
        // margin and its page-local offset is 0 whatever the viewport does —
        // the first draft of this test used zoom 0.5 and compared 0 with 0.
        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let before = frame(1.0, (444.0, 592.0));
        assert_eq!(place_at(&mut doc, 1.0, (444.0, 592.0), None), None);
        let a = place_at(&mut doc, 1.0, (446.0, 592.0), Some(before)).expect("placed");
        doc.view_viewport = Some((444.0, 592.0));
        let b = place_at(&mut doc, 1.0, (448.0, 592.0), Some(before)).expect("placed");
        assert!(
            ((a.x - b.x).abs() - 1.0).abs() < 1e-4,
            "★ half the viewport delta (2 pt) on the x offset: got {} vs {}",
            a.x,
            b.x
        );
    }
}
