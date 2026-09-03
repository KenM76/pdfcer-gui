//! # `canvas::geometry` — the pure arithmetic behind panning and zooming
//!
//! **Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\canvas.rs`** (Class
//! B, `SALVAGE.md`: *"the `CanvasTool` enum, dispatch, and the escape
//! ladder are sound concepts… this becomes several modules under
//! `canvas/`"*). This is the first of those modules: the two scroll-offset
//! solves, lifted with their documentation and their entire test suite.
//! The tool dispatch, the selection layer and the escape ladder stay behind
//! until the stages that need them (S4, S5).
//!
//! ## Why these are pure functions in their own file
//!
//! Both answer the same shape of question — *given where the view is and
//! what the operator just did, where should the scroll offset be?* — and
//! both are wrong in ways that are invisible in a screenshot and obvious in
//! use: a pan that rubber-bands, a zoom that slides the detail out from
//! under the pointer by an amount proportional to how far off-centre you
//! were pointing. Neither can be unit-tested through a `ScrollArea`; both
//! are trivially testable as arithmetic. So they are arithmetic, and the
//! widget code that calls them ([`super::show`]) is wiring.

/// The scroll offset a middle-drag pan should move to, clamped to what the
/// canvas can actually show.
///
/// # Why the clamp is not optional
///
/// The offset is subtracted, so the content follows the hand. Without a clamp
/// an unscrollable canvas — the page fitted inside the viewport, offset pinned
/// at zero — still accepts a negative target for one frame, so the page slides
/// with the pointer and then snaps back the instant the drag ends. Observed
/// exactly that on 2026-08-04: a 50 px slide and a 50 px jump back. Refusing to
/// move at all is the honest response to "there is nothing to pan to".
///
/// # Known limitation, deliberately left
///
/// This clamps to the PAGE, so the page edges cannot be dragged inward past the
/// viewport edge. The operator asked to "navigate beyond the page's edges",
/// which needs reserved space around the page rather than a different clamp —
/// a change to how the canvas reserves its content area, with a visible
/// consequence (scrollbars present at every zoom). That is a UX call, and this
/// function is the one place it would need to change.
#[must_use]
pub fn pan_offset(
    last: (f32, f32),
    pan: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(last: f32, pan: f32, d: f32, v: f32) -> f32 {
        if !(last.is_finite() && pan.is_finite() && d.is_finite() && v.is_finite()) {
            return last;
        }
        (last - pan).clamp(0.0, (content_extent(d, v) - v).max(0.0))
    }
    (
        axis(last.0, pan.0, display.0, viewport.0),
        axis(last.1, pan.1, display.1, viewport.1),
    )
}

/// The centring margin on one axis: half the slack when the page is smaller
/// than the viewport, zero once it is larger.
///
/// Lifted out of [`zoom_anchor_offset`] when [`anchor_screen_pos`] and
/// [`offset_holding_anchor_at`] were split out of it, so that all three read
/// the *same* margin. The margin is not a refinement — see
/// [`zoom_anchor_offset`]'s derivation — and two spellings of it would drift
/// apart in exactly the case that matters, the fit-page zoom an operator
/// starts from.
#[must_use]
fn margin(display: f32, viewport: f32) -> f32 {
    (display.max(viewport) - display) / 2.0
}

/// The pasteboard, as a multiple of the viewport. O23: half a viewport puts
/// a page corner at the screen's centre, a whole one puts it at the opposite
/// corner, and the operator asked for the second.
const PASTEBOARD_FRACTION: f32 = 1.0;

/// The pasteboard on one axis, in logical points. Zero for a degenerate
/// viewport so a frame measured before layout cannot produce a NaN extent.
#[must_use]
fn pasteboard(viewport: f32) -> f32 {
    if viewport.is_finite() && viewport > 0.0 {
        viewport * PASTEBOARD_FRACTION
    } else {
        0.0
    }
}

/// The **scroll content's** extent: the strip plus a pasteboard each side,
/// never smaller than the viewport. This is what `display.max(viewport)` used
/// to be at every call site, back when the strip and the content were the
/// same rectangle.
#[must_use]
pub fn content_extent(display: f32, viewport: f32) -> f32 {
    let out = display.max(viewport) + 2.0 * pasteboard(viewport);
    if out.is_finite() {
        out
    } else {
        display.max(viewport)
    }
}

/// How far the **strip's** origin sits from the **content's**: the centring
/// margin plus the pasteboard.
///
/// ★★ Not the same function as [`margin`], and must not become it. Two
/// offset spaces exist and only one is padded — the scroll offset egui is
/// given is measured from the content's origin, the page-local offset the
/// view stores is measured from the page's. [`strip_offset`] and
/// [`page_local_offset`] therefore call **one of each**; using the same
/// margin for both makes the pad cancel and a stored offset of zero scrolls
/// to blank paper.
///
/// `anchor_screen_pos` and `offset_holding_anchor_at` look like scroll-space
/// functions and are **page-local** — `canvas::mod` converts before building
/// the `CanvasFrame` — so they keep [`margin`]. Padding them doubles the pad.
///
/// ★ `pub` since O26g, because `canvas::show` must place the strip from it
/// **symbolically** rather than by subtracting two large rectangles. See
/// [`strip_origin_offset`].
#[must_use]
pub fn strip_margin(display: f32, viewport: f32) -> f32 {
    margin(display, viewport) + pasteboard(viewport)
}

/// **How far the strip's top-left sits from the scroll content's, on one
/// axis** — the number `canvas::show` adds to `outer_rect.min` to place the
/// strip.
///
/// # ★★★ Why this is not `(outer − display) / 2`
///
/// It *is* that, algebraically. Evaluated that way in `f32` it is a
/// catastrophic cancellation, and at deep zoom in a continuous mode it is the
/// **dominant source of error in the whole canvas**.
///
/// The strip's height is `pages × page_height × zoom`. On the operator's
/// 36-page drawing set at 1,045,114 % that is 4.6 × 10⁸ logical points, where
/// an `f32`'s representable step is **32 points**. `Rect::from_center_size`
/// forms `content_centre − strip/2` — two numbers near 2.3 × 10⁸ whose
/// difference is about 619 — so the strip's origin, and therefore every page
/// rect derived from it, and therefore the zoom anchor's `frac`, the raster
/// region and the pointer mapping, were all quantised to 32 points.
///
/// ★★ Measured, and the arithmetic predicts the measurement: an anchored zoom
/// notch slid the view 10 points at 292,415 % (strip 1.3 × 10⁸, step 8) and
/// 16 points at 1,045,114 % (strip 4.6 × 10⁸, step 32). That is why zooming
/// deep in a *multi-page* document creeps while the same zoom on a single page
/// does not: `viewer::deep_position_needed` measures the **page's** magnitude,
/// and it is the **strip's** that overflows `f32`'s exact range — earlier by
/// exactly the page count.
///
/// # The symbolic form
///
/// `outer` is `content_extent(display, viewport).max(avail)` per axis, so
///
/// * when the content wins — every case that matters, because a strip taller
///   than the window is what "scroll" means — the difference is
///   [`strip_margin`]: a centring margin that is **exactly zero** once the
///   display exceeds the viewport, plus a pasteboard that is one viewport.
///   Both are small, both are exact, and no large intermediate is formed at
///   all;
/// * when `avail` wins — a document smaller than the window, where every
///   magnitude is a few hundred points — the plain expression is used, and its
///   precision is not in question there.
///
/// The two agree to the last bit wherever the first branch is taken, which
/// [`tests::the_strip_origin_is_the_plain_expression_wherever_that_expression_is_exact`]
/// asserts.
#[must_use]
pub fn strip_origin_offset(display: f32, viewport: f32, avail: f32) -> f32 {
    let content = content_extent(display, viewport);
    let out = if content >= avail {
        strip_margin(display, viewport)
    } else {
        (avail - display) / 2.0
    };
    if out.is_finite() { out } else { 0.0 }
}

/// Convert a **scroll offset** back into **strip space** — the inverse of
/// [`strip_to_scroll`], and the one every consumer that thinks in strip
/// coordinates needs.
///
/// ★★★ Its absence was O23's whole failure, through three attempts. The
/// canvas builds its visible-region rect from `last_scroll_offset`, which is a
/// **content-space** offset, and then intersects it with the strip's own
/// layout. Before the pasteboard those were the same space and the omission
/// was invisible. With one, the rect lands a whole pasteboard past the end of
/// the strip, `layout.visible()` returns nothing, and the application draws
/// **no canvas at all** — it says so itself, as
/// `canvas-unavailable reason=nothing-visible`.
///
/// Every symptom chased for three attempts followed from that one line: no
/// pointer input, because there was no canvas to point at; a page rect that
/// looked correct, because it was published before the region went; and
/// `drawn=0`, because nothing was visible to raster.
#[must_use]
pub fn scroll_to_strip(scroll: f32, strip: f32, viewport: f32) -> f32 {
    let out = scroll - strip_margin(strip, viewport);
    if out.is_finite() { out } else { 0.0 }
}

/// Convert a position in **strip space** into the **scroll offset** that puts
/// it at the viewport's top-left, clamped to what can be reached.
#[must_use]
pub fn strip_to_scroll(in_strip: f32, strip: f32, viewport: f32) -> f32 {
    let out = in_strip + strip_margin(strip, viewport);
    if out.is_finite() {
        out.clamp(0.0, (content_extent(strip, viewport) - viewport).max(0.0))
    } else {
        0.0
    }
}

/// Where the page point at `anchor_frac` currently sits **relative to the
/// viewport's top-left**, in logical points.
///
/// The forward half of the pair this module's zoom solves are built from:
///
/// ```text
///     screen = margin(display, viewport) + anchor_frac * display - offset
/// ```
///
/// Not clamped and not guarded, deliberately. It is a *measurement* of where
/// something is, and a value outside `0 ..= viewport` is the true answer for a
/// point that has been scrolled off the edge of the view — clamping it would
/// silently claim the anchor was visible when it was not.
#[must_use]
pub fn anchor_screen_pos(
    anchor_frac: (f32, f32),
    offset: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(u: f32, off: f32, d: f32, v: f32) -> f32 {
        margin(d, v) + u * d - off
    }
    (
        axis(anchor_frac.0, offset.0, display.0, viewport.0),
        axis(anchor_frac.1, offset.1, display.1, viewport.1),
    )
}

/// ★★★ **Which page point is in the middle of the view** —
/// `OPERATOR_REQUESTS.md` O78.
///
/// The operator: *"when I change the size of the canvas window, whatever area
/// was centered in the current canvas should stay centered."*
///
/// The third member of the family [`anchor_screen_pos`] and
/// [`offset_holding_anchor_at`] already form. Those two answer *"where is this
/// fraction on screen?"* and *"what offset puts it there?"*; this answers
/// *"which fraction is at the middle?"* — it is `anchor_screen_pos` solved for
/// `anchor_frac` with `target = viewport / 2`.
///
/// With those three, preserving the centred point across a resize is
/// measure-then-place and needs nothing else:
///
/// ```text
/// let frac = centred_frac(before.offset, before.display, before.viewport);
/// let off  = offset_holding_anchor_at(frac, (v.x / 2.0, v.y / 2.0), display, v);
/// ```
///
/// # ★★★ Why this SUBSUMES a fit's re-placement, which is the whole design
///
/// It is a theorem rather than a hope, and it is pinned by
/// `centring_agrees_with_the_pinned_fit_answer` in this module's tests.
///
/// On an axis a fit **pins**, the page is by construction no larger than the
/// viewport, so `display ≤ viewport` and `margin = (v − d) / 2`. Substituting
/// `frac = 0.5` and `target = v / 2` into [`offset_holding_anchor_at`]:
///
/// ```text
///   off = (v − d)/2 + 0.5·d − v/2
///       = v/2 − d/2 + d/2 − v/2
///       = 0
/// ```
///
/// — which is **exactly** what [`fit_placement_offset`] returns for a pinned
/// axis. So a fit-page document nobody has panned has its page centre at the
/// viewport centre, and restoring the centred point re-centres it *for free*.
///
/// ⇒ That is what lets `canvas::fit` stop having a separate resize path. A fit
/// becomes purely a rule about **zoom** (`ViewState::apply_fit`, every frame)
/// and this becomes the one rule about **position**. Two rules that used to be
/// entangled, and the entanglement was what made a pan have to leave the fit.
///
/// # A non-finite axis yields `0.5` — the middle of the page
///
/// [`offset_holding_anchor_at`]'s guard fails to `0.0` because that is the one
/// value guaranteed to be a legal *offset*. This is a *fraction*, and the
/// harmless value for a fraction is the middle — the same choice
/// [`crate::canvas::zoom::frac_of`] makes, for the same reason: a degenerate
/// extent must not put a NaN into a scroll offset, and "the middle of the
/// page" is the answer that looks deliberate rather than broken.
///
/// # Unclamped, like both of its siblings
///
/// A fraction outside `0 ..= 1` is the true answer for a view scrolled into
/// the pasteboard, and it must stay true: clamping here would silently claim
/// the operator was looking at the page when they were looking past it, and
/// the next resize would then *move* the page to make that claim come out
/// right.
#[must_use]
pub fn centred_frac(offset: (f32, f32), display: (f32, f32), viewport: (f32, f32)) -> (f32, f32) {
    fn axis(off: f32, d: f32, v: f32) -> f32 {
        let u = (off + v / 2.0 - margin(d, v)) / d;
        if u.is_finite() { u } else { 0.5 }
    }
    (
        axis(offset.0, display.0, viewport.0),
        axis(offset.1, display.1, viewport.1),
    )
}

/// The exact inverse of [`anchor_screen_pos`]: the scroll offset that would
/// put the page point at `anchor_frac` at viewport-relative position
/// `target`.
///
/// **Unclamped, and that is the point.** Two callers need the raw solve for
/// two different reasons and a clamp inside here would spoil both:
///
/// * [`zoom_anchor_offset`] applies the scrollable-range clamp *itself*, after
///   composing this with [`anchor_screen_pos`], because the clamp belongs to
///   the offset that is actually handed to a `ScrollArea` and not to an
///   intermediate;
/// * [`crate::canvas::zoom`] uses it to *fabricate a before-state* — "the
///   offset at which the anchor would have been sitting where we want it to
///   end up" — which is a hypothetical, not a scroll position, and clamping a
///   hypothetical into the current page's range would quietly change the
///   framing it describes.
///
/// A non-finite axis yields `0.0` on that axis. There is no honest answer to
/// "where would the offset have been" when one of the inputs is NaN, and `0.0`
/// is the one value guaranteed to be a legal scroll offset for any page — the
/// same "fail to a finite, harmless value" discipline `viewer` applies to a
/// degenerate zoom.
#[must_use]
pub fn offset_holding_anchor_at(
    anchor_frac: (f32, f32),
    target: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(u: f32, target: f32, d: f32, v: f32) -> f32 {
        let off = margin(d, v) + u * d - target;
        if off.is_finite() { off } else { 0.0 }
    }
    (
        axis(anchor_frac.0, target.0, display.0, viewport.0),
        axis(anchor_frac.1, target.1, display.1, viewport.1),
    )
}

// ---------------------------------------------------------------------------
// The strip ⟷ page-local bridge  (Phase 4)
// ---------------------------------------------------------------------------
//
// ★ **Why this pair exists, and what it buys.**
//
// Before Phase 4 the scroll area's content was one page, so a scroll offset and
// a page-relative offset were the same number. Under a continuous mode the
// content is a *strip* of pages and they are not — which threatens two solves
// that are deliberately owned elsewhere and must not be reimplemented here:
//
// * [`crate::canvas::zoom`] anchors every zoom against `ZoomAnchor`, whose
//   fields are a page fraction, a "before" offset and a "before" drawn size;
// * `crate::find::reveal::take_reveal_offset` scrolls a search hit into the
//   middle of the viewport from a page fraction and a page drawn size.
//
// Neither module is this work's to edit, and neither should be: the anchor
// rule and the reveal handshake are correct and are each asserted by their own
// suite. What they need is for the world to keep looking the way they expect —
// **one page, at the origin of the scroll content** — and that is exactly what
// these two functions provide. The canvas converts the real strip offset into
// the offset those solves would see if the current page were the only thing in
// the scroll area, hands it over, and converts the answer back.
//
// The conversion is exact, not an approximation, and
// [`tests::the_strip_bridge_preserves_where_a_page_point_lands_on_screen`]
// proves it the only way that matters: by asserting that a page point lands at
// the same screen position measured either way.
//
// One consequence is worth naming rather than discovering. `zoom_anchor_offset`
// clamps its answer to *the page's own* scroll range before the conversion
// back, so under a continuous mode an anchored zoom cannot scroll further than
// the current page's own extent in a single step. That is the same behaviour
// single-page mode has always had — the clamp is what stops an anchor near an
// edge from scrolling blank space into view — and applying it per page keeps
// a zoom about the cursor from throwing the operator onto a different sheet.

/// **The page-local offset, MEASURED from where the page was actually drawn.**
///
/// # ★★★ Why this exists beside [`page_local_offset`], which computes the same
/// number
///
/// `page_local_offset` *reconstructs* the offset from the scroll area's own
/// offset. That is correct exactly while the scroll offset is where the view
/// is — and above the deep-position threshold it is **not**. There the content
/// is taken down to the viewport, the scroll offset is forced to `(0, 0)` so
/// egui has nothing to round, and the position is held by
/// [`crate::viewer::deep::DeepAnchor`] in `f64`. Reconstructing from a forced
/// zero yields "the page is centred in the pasteboard", which is a statement
/// about a page nobody is looking at.
///
/// ★★★ **That lie was `OPERATOR_REQUESTS.md` O26e.** `CanvasFrame::offset` is
/// the `offset_before` of the next zoom, so every frame spent at deep zoom
/// recorded a fictitious "before". Nothing went wrong while the tier held —
/// the deep branch does not consult it — but the moment a zoom-out crossed
/// back, [`zoom_anchor_offset`] solved against it and put the page's **origin**
/// under the pointer. Driven, 2026-08-24: descending through the boundary at
/// 1,185,799 % moved the page point under the viewport centre from
/// (791.93, 1152.34) to **(−0.02, −0.03)** — the corner of the sheet, with
/// twelve million pixels of drawing off screen. The operator's report was
/// *"zoom out … repositions the page so that it is off screen in the far
/// bottom left corner … from around 2 million %"*.
///
/// # The measurement
///
/// ```text
///     page_top_left_on_screen = viewport_origin + margin(display, viewport) - offset
/// ```
///
/// which is [`anchor_screen_pos`] at `anchor_frac = 0`, rearranged. So the
/// offset is `margin − (page_min − viewport_min)`, and every term is a rect
/// this frame really drew. **It cannot disagree with the pixels, because it is
/// derived from them.**
///
/// ★ It is not an approximation of [`page_local_offset`] and not a second
/// spelling of it: on the shallow tier the two are *algebraically identical*,
/// which [`tests::measuring_the_offset_from_the_drawn_rect_matches_the_solved_one`]
/// asserts against the same inputs rather than trusting this paragraph. What
/// it buys is that the identity survives the tier change, because it never
/// mentions the scroll offset at all.
///
/// # Arguments
///
/// * `page_min` — the current page's top-left **on screen** (`image_rect.min`).
/// * `viewport_min` — the scroll viewport's top-left on screen
///   (`inner_rect.min`), which is where a viewport-relative position is
///   measured from.
/// * `display` — the page's drawn size, the same one the solve is handed.
/// * `viewport` — the viewport's size, the same measurement the margin term is
///   derived against.
///
/// Non-finite inputs yield `(0.0, 0.0)`: a `NaN` here would propagate into the
/// next zoom's `offset_before` and blank the canvas, and "centred" is the only
/// safe fiction when the true answer is unrepresentable.
#[must_use]
pub fn offset_from_drawn(
    page_min: (f32, f32),
    viewport_min: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(page_min: f32, viewport_min: f32, display: f32, viewport: f32) -> f32 {
        let out = margin(display, viewport) - (page_min - viewport_min);
        if out.is_finite() { out } else { 0.0 }
    }
    (
        axis(page_min.0, viewport_min.0, display.0, viewport.0),
        axis(page_min.1, viewport_min.1, display.1, viewport.1),
    )
}

/// **Where a fit command puts the view** — `OPERATOR_REQUESTS.md` O28.
///
/// # The report, and why a fit is now a position as well as a scale
///
/// > *"If I press the Fit width or fit page button the view should center to
/// > the width as well or center the page."*
///
/// Before O23's pasteboard a page no larger than the viewport had nowhere to
/// be except the middle, so *fit* and *centred* were the same act and the
/// button never had to choose. The pasteboard added a whole viewport of slack
/// on every side, and with it the state the operator is describing: the scale
/// is right and the page is not on screen.
///
/// # The rule, per axis
///
/// * **Pinned** — the fit has just decided this axis's extent, so there is one
///   honest position for it and the answer is **zero**. Zero is not an
///   arbitrary choice: [`anchor_screen_pos`] at `frac = 0` places the page's
///   top-left at `margin - offset` from the viewport's, and [`margin`] is
///   *half the slack when the page is smaller than the viewport and exactly
///   zero once it is larger*. So a page-local offset of zero means **centred
///   if it fits, flush if it does not** — which is fit-page's answer on both
///   axes and fit-width's on the horizontal, without a special case for
///   either.
/// * **Unpinned** — the operator is still navigating this axis, so their
///   position is *kept*, clamped to the page's own range `0 ..= display -
///   viewport`. Keeping it is why "Fit width" on page twelve of a drawing set
///   does not throw them back to the top of the sheet; clamping it is what
///   stops "kept" meaning "still looking at pasteboard".
///
/// ★ The clamp collapses to `[0, 0]` whenever the page is no larger than the
/// viewport on that axis — the fit-page case, and the landscape-sheet case —
/// and `0` is centred there, so the two rules agree at the boundary rather
/// than fighting over it.
///
/// `current` is the page-local offset the view is at now. Non-finite input
/// yields the pinned answer, because a `NaN` position is not one worth
/// preserving.
#[must_use]
pub fn fit_placement_offset(
    pinned: (bool, bool),
    current: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(pinned: bool, current: f32, display: f32, viewport: f32) -> f32 {
        if pinned || !current.is_finite() {
            return 0.0;
        }
        current.clamp(0.0, (display - viewport).max(0.0))
    }
    (
        axis(pinned.0, current.0, display.0, viewport.0),
        axis(pinned.1, current.1, display.1, viewport.1),
    )
}

/// **Strip offset → the offset a single-page solve expects.**
///
/// `page_origin` is the current page's top-left in strip space (from
/// [`crate::viewer::strip::Strip::rect_of`]); `strip` is the strip's whole
/// drawn size; `page_display` is the current page's drawn size. Under
/// [`crate::viewer::PageDisplay::Single`] the origin is `(0,0)` and `strip`
/// equals `page_display`, so this is the identity — which is the mechanical
/// form of "the single-page path is untouched", asserted by
/// [`tests::the_strip_bridge_is_the_identity_for_a_single_page`].
///
/// Not clamped, deliberately: the result is fed to solves that do their own
/// clamping ([`zoom_anchor_offset`]) or that are measuring a hypothetical
/// (`offset_holding_anchor_at`), and a clamp here would quietly change what
/// they were asked.
#[must_use]
pub fn page_local_offset(
    strip_offset: (f32, f32),
    page_origin: (f32, f32),
    strip: (f32, f32),
    page_display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(off: f32, origin: f32, strip: f32, page: f32, v: f32) -> f32 {
        let out = off - origin - strip_margin(strip, v) + margin(page, v);
        if out.is_finite() { out } else { 0.0 }
    }
    (
        axis(
            strip_offset.0,
            page_origin.0,
            strip.0,
            page_display.0,
            viewport.0,
        ),
        axis(
            strip_offset.1,
            page_origin.1,
            strip.1,
            page_display.1,
            viewport.1,
        ),
    )
}

/// **The exact inverse of [`page_local_offset`]: back to a strip offset.**
///
/// Clamped to the strip's scrollable range, because *this* is the value that
/// is handed to a `ScrollArea` — the same division of labour
/// [`offset_holding_anchor_at`] and [`zoom_anchor_offset`] already observe
/// between them, where the raw solve is unclamped and the offset that actually
/// reaches the widget is not.
#[must_use]
pub fn strip_offset(
    page_local: (f32, f32),
    page_origin: (f32, f32),
    strip: (f32, f32),
    page_display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(off: f32, origin: f32, strip: f32, page: f32, v: f32) -> f32 {
        let out = off + origin + strip_margin(strip, v) - margin(page, v);
        if out.is_finite() {
            out.clamp(0.0, (content_extent(strip, v) - v).max(0.0))
        } else {
            0.0
        }
    }
    (
        axis(
            page_local.0,
            page_origin.0,
            strip.0,
            page_display.0,
            viewport.0,
        ),
        axis(
            page_local.1,
            page_origin.1,
            strip.1,
            page_display.1,
            viewport.1,
        ),
    )
}

/// Where the canvas must be scrolled to so the page point under the pointer
/// stays under the pointer across a zoom step — "zoom to cursor".
///
/// # Why this exists
///
/// Ctrl+wheel previously called `zoom_by` and nothing else. The scroll offset
/// was left alone, so the *viewport centre* was the fixed point of the zoom and
/// whatever the operator was pointing at slid away — worse the further from
/// centre they were pointing, which is exactly where a person zooms in on a
/// drawing detail. Every other application that zooms a canvas (browsers, CAD,
/// Inkscape, Office) anchors on the cursor, and the operator reported the old
/// behaviour as "jarring" on 2026-08-04 for that reason.
///
/// # The geometry
///
/// The page is drawn at `display` pixels inside a scroll-area content box of
/// `outer = max(display, viewport)` — the `max` is what lets the area still
/// scroll when the page is bigger AND centre the page when it is smaller (see
/// the reservation comment in [`super::show`]). So the page's top-left sits at
/// `margin = (outer - display) / 2` in content coordinates, and a point at
/// fraction `anchor_frac` of the page appears on screen at
///
/// ```text
///     screen = viewport_origin + margin + anchor_frac * display - offset
/// ```
///
/// Holding `screen` fixed across the step and solving for the new offset gives
///
/// ```text
///     offset₁ = offset₀ + anchor_frac * (display₁ - display₀) + (margin₁ - margin₀)
/// ```
///
/// which needs no knowledge of where the viewport is on screen — only sizes.
/// The margin term is not a refinement: while the page is smaller than the
/// viewport the offset is pinned at zero and *all* of the movement is the
/// margin shrinking, so dropping it would make zoom-to-cursor do nothing at
/// precisely the "fit page" zoom an operator starts from.
///
/// # Contract
///
/// - `anchor_frac` is the pointer's position as a fraction of the page's drawn
///   size, `(pointer - page_top_left) / display₀`. Values outside `0..=1` are
///   meaningful (the pointer may be in the centring margin) and are not clamped.
/// - The result is clamped to the scrollable range `0 ..= max(0, display₁ -
///   viewport)`, so a caller may hand it straight to `ScrollArea::scroll_offset`
///   without producing an offset the area would fight back against.
/// - Non-finite inputs yield `offset_before` unchanged: refusing to move is the
///   only safe answer, since a NaN offset would blank the canvas.
///
/// # ★ Expressed as "measure, then re-place", and why that is not a
/// refactor for its own sake
///
/// The body below is literally *"find where the anchor is on screen
/// ([`anchor_screen_pos`]), then find the offset that puts it back there at
/// the new size ([`offset_holding_anchor_at`])"*, and the composition is
/// algebraically identical to the closed form in the derivation above —
/// [`tests::the_split_solve_is_the_closed_form_it_replaced`] asserts that
/// against the original expression rather than trusting the algebra.
///
/// It is written this way because **zoom-to-region and zoom-to-selection need
/// the same solve with a different target**: not "back where it was" but "at
/// the centre of the viewport". With the two halves named, framing a rect is
/// the *same arithmetic with one substitution* rather than a second solve
/// living beside this one — and two independently-maintained scroll solves is
/// how the discrete zoom commands ended up anchoring the page's top-left while
/// the wheel anchored the cursor, which is the defect Phase 3.1 exists to fix.
#[must_use]
pub fn zoom_anchor_offset(
    offset_before: (f32, f32),
    display_before: (f32, f32),
    display_after: (f32, f32),
    viewport: (f32, f32),
    anchor_frac: (f32, f32),
) -> (f32, f32) {
    let finite = [
        offset_before.0,
        offset_before.1,
        display_before.0,
        display_before.1,
        display_after.0,
        display_after.1,
        viewport.0,
        viewport.1,
        anchor_frac.0,
        anchor_frac.1,
    ]
    .iter()
    .all(|f| f.is_finite());
    if !finite {
        return offset_before;
    }

    let held = anchor_screen_pos(anchor_frac, offset_before, display_before, viewport);
    // ★★★ RETURNED UNCLAMPED — `OPERATOR_REQUESTS.md` O24e.
    //
    // This used to clamp to `display_after - viewport`: the range a page has
    // when the scroll content is the page and nothing else. **The pasteboard
    // made that false.** `content_extent` now adds a viewport of slack on
    // every side (O23, so an object off the page is still reachable), so the
    // real range is `content_extent(strip, viewport) - viewport` and the page
    // is only part of it.
    //
    // The damage was worst exactly where the operator found it. At a fit-page
    // zoom the page is no LARGER than the viewport, so `display_after -
    // viewport` is zero or negative, the clamp range collapses to `[0, 0]`,
    // and every zoom forced the offset to zero — which after
    // `strip_offset`'s conversion is the centred position. His report,
    // 2026-08-22:
    //
    // > *"if I am zoomed out to about page size, pan the cells to the center
    // > of the screen, then start to zoom, the page snaps back to near the
    // > center position."*
    //
    // Not "near" by accident: it is the centre, and it is the centre because
    // zero page-local offset means the page sits centred in the pasteboard.
    //
    // ★ The clamp is not gone, it has moved to the one place that can do it
    // correctly. [`strip_offset`] already clamps to
    // `content_extent(strip, v) - v` — the true range, pasteboard included —
    // and it is the value actually handed to the `ScrollArea`. That is the
    // division of labour this module's header states: *the raw solve is
    // unclamped and the offset that reaches the widget is not*. Clamping
    // here as well was a second clamp in the wrong space against the wrong
    // extent, and the two were not equivalent the moment the pasteboard
    // existed.
    offset_holding_anchor_at(anchor_frac, held, display_after, viewport)
}

/// See `canvas/geometry/tests.rs` — moved out under R2 on 2026-08-31.
#[cfg(test)]
mod tests;

/// **A PDF-space rectangle as a canvas-space one**, for a `/FitR` destination.
///
/// ★ Both corners go through [`crate::viewer::pdf_space_to_canvas`], the one
/// bridge between the two spaces, rather than through a local flip. PDF is
/// y-up and canvas is y-down, so a hand-rolled conversion mirrors the
/// rectangle about the page centre when it is wrong — and a mirrored
/// destination lands plausibly on the wrong half of the sheet, which reads as
/// "the bookmark is broken" rather than as an arithmetic error.
///
/// `None` on a page whose device geometry will not invert, which is the same
/// condition every other coordinate hop here declines on.
#[must_use]
pub fn pdf_rect_to_canvas(
    rect: (f64, f64, f64, f64),
    page: &pdfcer_core::page_tree::Page,
) -> Option<egui::Rect> {
    let (left, bottom, right, top) = rect;
    let a = crate::viewer::pdf_space_to_canvas(egui::pos2(left as f32, bottom as f32), page)?;
    let b = crate::viewer::pdf_space_to_canvas(egui::pos2(right as f32, top as f32), page)?;
    // `from_two_pos` rather than `from_min_max`: the y flip means the corner
    // that was the bottom is now the larger y, and a rect built from a min that
    // is not minimal is empty rather than wrong-looking.
    Some(egui::Rect::from_two_pos(a, b))
}

/// **How much paper to show around a point destination**, in PDF points.
///
/// ★★ A `/XYZ` destination names a single coordinate, and framing a point has
/// no answer — a zoom onto zero area is either everything or nothing. Acrobat
/// puts the point at the top-left and keeps the current magnification when the
/// destination's zoom is null; this shell frames a region around it instead,
/// because its one framing solver takes a rectangle and adding a second
/// "scroll to a point" solver would be two answers to one question.
///
/// 150 pt is about 5 cm of paper each way — enough that a detail on a drawing
/// arrives with its surroundings rather than magnified onto a coordinate, and
/// small enough that it is recognisably a destination rather than a page fit.
pub const DESTINATION_CONTEXT_PT: f64 = 150.0;

/// **A PDF-space point as the canvas-space region to frame around it.**
///
/// A missing axis falls back to the page's own extent on that axis, which is
/// §12.3.2.2's *"leave this one as it is"* expressed as a framing: the axis
/// that was not specified ends up showing the whole page rather than an
/// invented position.
#[must_use]
pub fn pdf_point_to_canvas_region(
    left: Option<f64>,
    top: Option<f64>,
    page: &pdfcer_core::page_tree::Page,
) -> Option<egui::Rect> {
    let (w, h) = crate::viewer::page_extent_pts(page);
    let l = left.unwrap_or(0.0);
    let t = top.unwrap_or(f64::from(h));
    let r = (l + DESTINATION_CONTEXT_PT).min(f64::from(w));
    let b = (t - DESTINATION_CONTEXT_PT).max(0.0);
    pdf_rect_to_canvas((l, b, r, t), page)
}
