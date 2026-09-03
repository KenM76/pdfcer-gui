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
    // ★ Compared exactly rather than with a tolerance. A viewport that has not
    // changed produces bit-identical floats — it is the same measurement of the
    // same layout — and a tolerance would only decide how much of a resize is
    // allowed to be ignored, which is a question nobody has.
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
    let changed = doc.view_viewport != Some((vp.x, vp.y));
    // Recorded on EVERY frame, whatever this function goes on to decide.
    //
    // ★ Including the frames it declines — no previous frame, a different
    // document, a degenerate viewport. A frame that declined without recording
    // would leave the NEXT frame reading as a resize and moving the view for
    // nothing, which is the one way this can produce a jump the operator did
    // not cause.
    doc.view_viewport = Some((vp.x, vp.y));

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
