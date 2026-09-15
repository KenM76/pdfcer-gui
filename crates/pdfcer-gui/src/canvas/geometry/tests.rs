//! # `canvas::geometry` tests — the arithmetic, pinned
//!
//!
//! ★ The seam is the one this crate has taken four times already
//! (`app/state`, `app/prefs`, `canvas/interact`): the parent answers *"what is
//! the arithmetic?"* and this answers *"is it still right?"*. Nothing moved
//! but its address — every test below is byte-identical to the one that was in
//! the parent, and the `#[allow(clippy::float_cmp)]` moved with them because
//! the reason it exists is theirs: exact `f32` arithmetic on exact literals.

// ★ The INNER attribute, and it is load-bearing for more than the compiler.
//
// `tools/gates/check-ui-strings.sh` stops scanning a file at `#![cfg(test)]`,
// because a test assertion message is read by whoever is staring at a failing
// test and is not operator copy. Without this line the split would have turned
// thirty-two perfectly good assertion messages into gate violations — which is
// exactly what it did on the first attempt, and is why the same line sits at
// the top of `canvas/selection/tests.rs`, the split this one follows.
#![cfg(test)]
#![allow(clippy::float_cmp, reason = "exact f32 arithmetic on exact literals")] // ui-text-exempt: clippy lint justification, never displayed

use super::*;

// ---- middle-drag pan ----------------------------------------------

#[test]
fn panning_moves_the_content_opposite_the_offset_so_the_page_follows_the_hand() {
    // Page twice the viewport, so there is room to move.
    let out = pan_offset(
        (500.0, 500.0),
        (30.0, -20.0),
        (1600.0, 1600.0),
        (800.0, 800.0),
        (0.0, 0.0),
    );
    assert_eq!(
        out,
        (470.0, 520.0),
        "dragging right must DECREASE the offset, or the page moves against the hand"
    );
}

#[test]
fn an_unscrollable_canvas_refuses_to_pan_rather_than_rubber_banding() {
    // The fit-page case: page smaller than the viewport, offset pinned.
    // Before the clamp this returned -50 and the page visibly slid, then
    // snapped back when the drag ended.
    let out = pan_offset(
        (0.0, 0.0),
        (50.0, 50.0),
        (600.0, 600.0),
        (800.0, 800.0),
        (0.0, 0.0),
    );
    assert_eq!(out, (0.0, 0.0));
}

/// ★★ **The far edge is now a whole viewport PAST the page**, which is
/// `OPERATOR_REQUESTS.md` O23 stated as a number.
///
///
/// > *"I should also be able to move the view of the corner of the page to
/// > the center of the screen, or even all the way vertically to the
/// > opposite corner if I want to."*
///
/// With a one-viewport pasteboard the content is `1000 + 2×800 = 2600`
/// wide, so the last offset that still shows anything is `2600 − 800 =
/// 1800`. The pan asks for `700 + 500 = 1200`, which is now inside the
/// range and is therefore granted in full.
#[test]
fn panning_stops_a_whole_viewport_past_the_page_edge() {
    let out = pan_offset(
        (700.0, 0.0),
        (-500.0, 0.0),
        (1000.0, 1000.0),
        (800.0, 800.0),
        (0.0, 0.0),
    );
    assert_eq!(
        out.0, 1200.0,
        "the pasteboard makes this pan reachable; it used to clamp at 200"
    );

    // …and the clamp still exists, one viewport further out.
    let far = pan_offset(
        (1800.0, 0.0),
        (-500.0, 0.0),
        (1000.0, 1000.0),
        (800.0, 800.0),
        (0.0, 0.0),
    );
    assert_eq!(
        far.0,
        content_extent(1000.0, 800.0, 0.0) - 800.0,
        "there is still an end; it is the end of the PASTEBOARD, not of the page"
    );
}

/// ★★★ **O23, asserted as the operator's own two sentences.**
///
/// The pasteboard's size is not a taste; it is whatever makes these two
/// true. If `PASTEBOARD_FRACTION` is ever reduced, this fails and says
/// which sentence stopped holding.
#[test]
fn any_page_corner_can_be_brought_to_the_centre_and_to_the_opposite_corner() {
    // A page smaller than the window — the hard case, because there is no
    // scrolling to be had from the page's own size.
    let (d, v) = (200.0_f32, 800.0_f32);
    let range = (content_extent(d, v, 0.0) - v).max(0.0);

    // Where the strip's own origin sits inside the content.
    let origin = strip_margin(d, v, 0.0);

    // "the corner of the page to the center of the screen": the offset that
    // puts the strip's top-left half a viewport in from the view's left.
    let to_centre = origin - v / 2.0;
    assert!(
        (0.0..=range).contains(&to_centre),
        "a page corner must reach the centre of the screen: {to_centre} not in 0..={range}"
    );

    // "even all the way … to the opposite corner": the strip's top-left
    // pushed to the far edge of the view.
    let to_far_corner = origin - v;
    assert!(
        (0.0..=range).contains(&to_far_corner),
        "a page corner must reach the opposite corner: {to_far_corner} not in 0..={range}"
    );

    // And the mirror: the page's BOTTOM-RIGHT corner brought back to the
    // view's top-left, which is the same freedom in the other direction.
    let bottom_right_to_origin = origin + d;
    assert!(
        (0.0..=range).contains(&bottom_right_to_origin),
        "the far corner must reach the near one: {bottom_right_to_origin} not in 0..={range}"
    );
}

/// **The pasteboard rule, restated rather than called.**
///
/// Four tests below need to know how much slack a viewport gets, and every one
/// of them needs to know it *independently of [`pasteboard`]* — a test that
/// calls the function under test agrees with it by construction, including when
/// it is wrong. Before O186 they each spelled it `v * PASTEBOARD_FRACTION`
/// inline, which was right while the rule was one term.
///
/// ★ It is now two, so the restatement lives here once: a whole viewport, less
/// the band [`MIN_SHEET_ON_SCREEN`] keeps on screen. Still a restatement — it
/// reads the two constants and does the arithmetic itself, so a change to
/// `pasteboard`'s *code* cannot be ratified by these tests, only a change to
/// its *constants*. That is the property worth having, and four copies of it
/// drifting apart is not.
fn pasteboard_rule(viewport: f32) -> f32 {
    viewport * PASTEBOARD_FRACTION - MIN_SHEET_ON_SCREEN.min(viewport / 2.0)
}

/// **The overhang at which the overhang branch overtakes the fixed slack**, by
/// the same restatement rule as [`pasteboard_rule`].
///
/// `pasteboard` offers the larger of `viewport × FRACTION − sliver` and
/// `overhang + viewport / 2`, so the second wins once the overhang passes
/// `viewport / 2 − sliver`. Worth naming because the number is not obvious and
/// two tests need it: it is **zero** on a viewport narrower than twice
/// [`MIN_SHEET_ON_SCREEN`], and a few hundred points on a real canvas.
fn viewport_half_less_sliver(viewport: f32) -> f32 {
    viewport / 2.0 - MIN_SHEET_ON_SCREEN.min(viewport / 2.0)
}

// ---- zoom to cursor -----------------------------------------------

/// The whole point, stated as the invariant rather than as an offset:
/// re-derive where the anchored page point lands on screen after the step
/// and assert it has not moved.
///
/// Screen position is `margin + frac * display - offset`, which is the same
/// expression the doc comment solves — so this checks the solve, not merely
/// that the code agrees with itself about arithmetic (assert the outcome,
/// not the intent).
fn anchored_screen_x(off: f32, d: f32, v: f32, u: f32) -> f32 {
    (d.max(v) - d) / 2.0 + u * d - off
}

#[test]
fn the_point_under_the_cursor_stays_under_the_cursor() {
    // A page larger than the viewport, pointer three quarters across —
    // i.e. far from centre, where the old centre-anchored behaviour was
    // most visibly wrong.
    let (v, u) = (800.0_f32, 0.75_f32);
    let (d0, d1) = (1200.0_f32, 1800.0_f32); // a 1.5x zoom in
    let off0 = 300.0_f32;
    let before = anchored_screen_x(off0, d0, v, u);

    let off1 = zoom_anchor_offset((off0, 0.0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
    let after = anchored_screen_x(off1, d1, v, u);

    assert!(
        (after - before).abs() < 0.01,
        "the anchored point moved {} px across the zoom (before {before}, after {after})",
        after - before
    );
}

#[test]
fn zooming_in_from_fit_page_moves_the_view_even_though_the_offset_starts_pinned() {
    // The case the margin term exists for: at "fit page" the page is
    // SMALLER than the viewport, so offset is 0 and cannot go lower.
    // Zooming past the viewport must start scrolling toward the anchor.
    let (v, u) = (800.0_f32, 0.9_f32); // pointer near the right edge
    let (d0, d1) = (600.0_f32, 2000.0_f32);
    let off1 = zoom_anchor_offset((0.0, 0.0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
    assert!(
        off1 > 0.0,
        "zooming in past the viewport with the pointer off-centre must scroll toward it, \
             got {off1}"
    );
    let before = anchored_screen_x(0.0, d0, v, u);
    let after = anchored_screen_x(off1, d1, v, u);
    assert!(
        (after - before).abs() < 0.01,
        "the anchored point moved {} px",
        after - before
    );
}

/// ★★ **The offset handed to the scroll area never leaves its range** —
/// and after O24e that range is the pasteboard's, not the page's.
///
#[test]
fn the_offset_never_leaves_the_scrollable_range() {
    // Zooming OUT far enough that the page no longer fills the viewport.
    // The solve itself is handed back raw…
    let out = zoom_anchor_offset(
        (900.0, 900.0),
        (2000.0, 2000.0),
        (400.0, 400.0),
        (800.0, 800.0),
        (0.1, 0.1),
    );
    // …and whatever it says, the value that reaches the widget is inside
    // the content. Both ends checked, because a negative offset is the
    // failure the original test was written for and it must still be
    // impossible.
    let (v, d) = (800.0_f32, 400.0_f32);
    let range = content_extent(d, v, 0.0) - v;
    for probe in [out.0, -5000.0, 0.0, 5000.0] {
        let reached = strip_offset(
            (probe, probe),
            (0.0, 0.0),
            (d, d),
            (d, d),
            (v, v),
            (0.0, 0.0),
        );
        assert!(
            reached.0 >= 0.0 && reached.0 <= range,
            "{probe} reached {reached:?}, outside [0, {range}]"
        );
        assert!(reached.1 >= 0.0 && reached.1 <= range);
    }

    // And never past the far edge, however extreme the anchor fraction.
    // ★ Against the CONTENT's range, not the page's — that substitution is
    // the whole of O24e.
    let (v2, d2) = (800.0_f32, 1000.0_f32);
    let solved = zoom_anchor_offset((900.0, 0.0), (500.0, 500.0), (d2, d2), (v2, v2), (5.0, 0.0)).0;
    let reached = strip_offset(
        (solved, 0.0),
        (0.0, 0.0),
        (d2, d2),
        (d2, d2),
        (v2, v2),
        (0.0, 0.0),
    )
    .0;
    let range2 = content_extent(d2, v2, 0.0) - v2;
    assert!(
        reached <= range2 + 0.01,
        "offset {reached} exceeds the maximum scroll {range2}"
    );
}

/// ★★★ **The anchor solve is unclamped; the SCROLL OFFSET is clamped** —
/// and the two are different values in different spaces.
///
///
/// ★ The behaviour the old test was protecting is real and still wanted —
/// an anchor near an edge must saturate rather than scroll into nothing.
/// It just belongs to the value that reaches the widget. So the assertion
/// moved to [`strip_offset`], which clamps against `content_extent`, the
/// pasteboard included.
#[test]
fn the_scroll_offset_saturates_at_the_pasteboard_edge_not_at_the_page_edge() {
    let (v, u) = (800.0_f32, 0.9_f32);
    let (d0, d1) = (600.0_f32, 1000.0_f32);

    // 1. The raw solve is handed back whole, over-range and all.
    let solved = zoom_anchor_offset((0.0, 0.0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
    let unclamped = 0.9 * (d1 - d0) - 100.0; // 260
    assert!(
        unclamped > d1 - v,
        "this case must actually be over-range for the page to test anything"
    );
    assert_eq!(
        solved, unclamped,
        "the anchor solve must not clamp: it does not know the real range"
    );

    // 2. And 260 is REACHABLE, because the pasteboard extends the range
    //    well past the page's own 200. This is the whole point: the old
    //    clamp was discarding positions the operator can legitimately be
    //    at.
    // ★ Compared against the PAGE's range, not against `unclamped`:
    // `strip_offset` also applies the strip↔page-local conversion, so the
    // number it returns is in a different space and is not expected to
    // equal the solve. What matters is that it was not truncated to the
    // page's 200 — the position the operator panned to is still reachable.
    let reached = strip_offset(
        (solved, 0.0),
        (0.0, 0.0),
        (d1, d1),
        (d1, d1),
        (v, v),
        (0.0, 0.0),
    )
    .0;
    assert!(
        reached > d1 - v,
        "reached {reached}, which is inside the page's own range of {} — the pasteboard \
             position was discarded",
        d1 - v
    );

    // 3. The saturation itself still happens — at the pasteboard's edge.
    let far = content_extent(d1, v, 0.0) * 4.0;
    let limit = strip_offset(
        (far, 0.0),
        (0.0, 0.0),
        (d1, d1),
        (d1, d1),
        (v, v),
        (0.0, 0.0),
    )
    .0;
    assert_eq!(
        limit,
        content_extent(d1, v, 0.0) - v,
        "an absurd offset must saturate at the end of the scrollable content"
    );
}

/// ★ **The split solve is the closed form it replaced**, checked against
/// the original expression rather than against itself.
///
#[test]
fn the_split_solve_is_the_closed_form_it_replaced() {
    fn closed_form(off0: f32, d0: f32, d1: f32, v: f32, u: f32) -> f32 {
        let margin = |d: f32| (d.max(v) - d) / 2.0;
        // ★ No clamp: O24e moved it to `strip_offset`, which is the only
        // caller that knows the pasteboard-extended range. See
        // `zoom_anchor_offset`.
        off0 + u * (d1 - d0) + (margin(d1) - margin(d0))
    }
    for &(off0, d0, d1, v) in &[
        (300.0_f32, 1200.0_f32, 1800.0_f32, 800.0_f32),
        (0.0, 600.0, 2000.0, 800.0),   // starts smaller than the viewport
        (900.0, 2000.0, 400.0, 800.0), // ends smaller: clamps to 0
        (0.0, 600.0, 1000.0, 800.0),   // anchors past the edge: saturates
        (120.0, 1000.0, 1000.0, 1000.0), // no zoom change at all
    ] {
        for &u in &[0.0_f32, 0.1, 0.5, 0.9, 1.0, 5.0, -2.0] {
            let via_split = zoom_anchor_offset((off0, off0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
            let via_closed = closed_form(off0, d0, d1, v, u);
            assert!(
                (via_split - via_closed).abs() < 1e-3,
                "u={u} off0={off0} d0={d0} d1={d1} v={v}: {via_split} vs {via_closed}"
            );
        }
    }
}

/// ★ **`offset_holding_anchor_at` really is the inverse of
/// `anchor_screen_pos`** — the property zoom-to-region's framing rests on.
///
/// Framing a rect is "put this page point at the viewport centre", which
/// is the second function with a chosen target. If the pair ever stopped
/// being an exact inverse, a marquee zoom would land the region *near* the
/// centre — an error small enough to look like a rounding artefact and
/// large enough to be the wrong answer.
#[test]
fn placing_an_anchor_and_measuring_it_are_exact_inverses() {
    let v = (800.0_f32, 600.0_f32);
    for &d in &[(400.0_f32, 300.0_f32), (1600.0, 2400.0), (800.0, 600.0)] {
        for &u in &[(0.0_f32, 0.0_f32), (0.5, 0.5), (0.25, 0.9), (1.0, 0.0)] {
            for &target in &[(400.0_f32, 300.0_f32), (0.0, 0.0), (-120.0, 55.0)] {
                let off = offset_holding_anchor_at(u, target, d, v);
                let back = anchor_screen_pos(u, off, d, v);
                assert!(
                    (back.0 - target.0).abs() < 1e-3 && (back.1 - target.1).abs() < 1e-3,
                    "u={u:?} d={d:?} target={target:?} came back as {back:?}"
                );
            }
        }
    }
}

/// A non-finite request for a *hypothetical* offset yields the origin
/// rather than a NaN that would propagate into a scroll offset and blank
/// the canvas. The sibling guard to
/// `a_non_finite_input_refuses_to_move_rather_than_blanking_the_canvas`,
/// which cannot apply here because this function has no "before" state to
/// refuse to move from.
#[test]
fn a_non_finite_placement_falls_back_to_the_origin() {
    assert_eq!(
        offset_holding_anchor_at((f32::NAN, 0.5), (10.0, 10.0), (100.0, 100.0), (80.0, 80.0)),
        // y: margin(100,80) = 0, so 0 + 0.5*100 - 10 = 40.
        //
        // ★ UNCHANGED by O23's pasteboard, and that is itself the assertion:
        // this function works in PAGE-LOCAL space, where there is no
        // pasteboard. If a future edit makes this 120, it has padded a
        // page-local function — see `strip_margin`'s note.
        (0.0, 40.0)
    );
}

// ---- the strip ⟷ page-local bridge --------------------------------

/// ★ **Under single page the bridge is the identity.**
///
/// The mechanical form of "continuous is an option, not a replacement":
/// the default path must not merely *behave* the same, it must compute the
/// same number. With one page in the strip its origin is `(0,0)` and the
/// strip's size is the page's, so both terms cancel — at every zoom, and
/// with the page both larger and smaller than the viewport (the latter is
/// where the centring margin is non-zero and a sloppy conversion would
/// show).
#[test]
fn the_strip_bridge_is_a_pure_pasteboard_shift_for_a_single_page() {
    let v = (800.0_f32, 600.0_f32);
    for &page in &[(400.0_f32, 300.0_f32), (1600.0, 2400.0), (800.0, 600.0)] {
        let pad = (pasteboard(v.0, 0.0), pasteboard(v.1, 0.0));
        let range = (
            (content_extent(page.0, v.0, 0.0) - v.0).max(0.0),
            (content_extent(page.1, v.1, 0.0) - v.1).max(0.0),
        );
        for &off in &[(0.0_f32, 0.0_f32), (120.0, 55.0), (900.0, 1800.0)] {
            // ★ Going OUT: the page-local offset gains exactly one
            // pasteboard and nothing else. With one page the strip IS the
            // page, so the two centring margins are equal and cancel — the
            // only surviving term is the pad, which is the whole of what
            // O23 added. Anything else here would mean the centring margin
            // had leaked into a space that does not have one.
            let expected = (
                (off.0 + pad.0).clamp(0.0, range.0),
                (off.1 + pad.1).clamp(0.0, range.1),
            );
            assert_eq!(
                strip_offset(off, (0.0, 0.0), page, page, v, (0.0, 0.0)),
                expected,
                "page {page:?} offset {off:?}"
            );

            // …and coming BACK the pad is removed again, so a round trip
            // through both legs is the identity wherever the clamp did not
            // bite. **This is the property that matters** — the pad must
            // not accumulate, or every frame would drift one viewport
            // further into blank paper.
            let back = page_local_offset(expected, (0.0, 0.0), page, page, v, (0.0, 0.0));
            if expected.0 > 0.0 && expected.0 < range.0 {
                assert!(
                    (back.0 - off.0).abs() < 0.001,
                    "x round trip: {off:?} -> {expected:?} -> {back:?}"
                );
            }
            if expected.1 > 0.0 && expected.1 < range.1 {
                assert!(
                    (back.1 - off.1).abs() < 0.001,
                    "y round trip: {off:?} -> {expected:?} -> {back:?}"
                );
            }
        }
    }
}

/// ★ **The bridge preserves where a page point lands on screen.**
///
/// The property the whole pair exists for, asserted as an *outcome*: take
/// a fraction of the current page, work out where it appears on screen
/// from the real strip geometry, then work it out again through the
/// page-local view the zoom and reveal solves are handed — and require the
/// two to agree. A conversion that dropped either margin term would pass
/// every algebraic check and fail this one at exactly the zoom an operator
/// starts from.
#[test]
fn the_strip_bridge_preserves_where_a_page_point_lands_on_screen() {
    let v = (800.0_f32, 600.0_f32);
    for &strip in &[(612.0_f32, 4000.0_f32), (1200.0, 500.0), (300.0, 200.0)] {
        for &page in &[(612.0_f32, 792.0_f32), (300.0, 200.0)] {
            for &origin in &[(0.0_f32, 0.0_f32), (0.0, 1200.0), (294.0, 2400.0)] {
                for &off in &[(0.0_f32, 0.0_f32), (100.0, 900.0)] {
                    for &frac in &[(0.0_f32, 0.0_f32), (0.5, 0.5), (1.0, 0.25)] {
                        // Where it really is: the strip's own margin, plus
                        // the page's origin in the strip, plus the point
                        // inside the page, less the scroll offset.
                        // ★ The strip's origin inside the CONTENT — its
                        // centring margin plus the pasteboard. Spelled out
                        // rather than calling `strip_margin`, because a
                        // test that reuses the function under test agrees
                        // with it by construction, including when wrong.
                        let truth = (
                            (strip.0.max(v.0) - strip.0) / 2.0
                                + pasteboard_rule(v.0)
                                + origin.0
                                + frac.0 * page.0
                                - off.0,
                            (strip.1.max(v.1) - strip.1) / 2.0
                                + pasteboard_rule(v.1)
                                + origin.1
                                + frac.1 * page.1
                                - off.1,
                        );
                        // Where the single-page solves think it is.
                        let local = page_local_offset(off, origin, strip, page, v, (0.0, 0.0));
                        let via_bridge = anchor_screen_pos(frac, local, page, v);
                        assert!(
                            (via_bridge.0 - truth.0).abs() < 1e-2
                                && (via_bridge.1 - truth.1).abs() < 1e-2,
                            "strip={strip:?} page={page:?} origin={origin:?} off={off:?} \
                                 frac={frac:?}: {via_bridge:?} vs {truth:?}"
                        );
                    }
                }
            }
        }
    }
}

/// ★★★ **[`offset_from_drawn`] is [`page_local_offset`], on the shallow
/// tier — the same number, from the pixels instead of from the offset.**
///
/// This is the claim O26e's fix rests on, and it is the claim that makes
/// the change safe: `canvas::show` swapped one for the other on **every**
/// frame, not only deep ones, so if they disagreed anywhere below the
/// threshold the fix would have traded a rare catastrophe for a constant
/// small one.
///
/// The shallow tier's geometry is reconstructed here exactly as `show`
/// builds it — content origin, strip centring margin, pasteboard, the
/// page's place inside the strip, the scroll offset — and then the page's
/// screen rect and the viewport's screen rect are handed to
/// `offset_from_drawn` the way `show` hands it `image_rect.min` and
/// `inner_rect.min`. Spelled out rather than calling `strip_margin`,
/// for the reason the sibling test above states: a test that reuses the
/// function under test agrees with it by construction, including when
/// both are wrong.
///
/// ★ What this does **not** claim, deliberately: that they agree at the
/// deep tier. They do not, and that is the whole point — there the scroll
/// offset is forced to zero and `page_local_offset` describes a page
/// nobody is looking at, while `offset_from_drawn` describes the one on
/// screen. There is no assertion to write for "one of these is a lie",
/// only a driven check: `zooming_back_out_keeps_the_view`.
#[test]
fn measuring_the_offset_from_the_drawn_rect_matches_the_solved_one() {
    let v = (800.0_f32, 600.0_f32);
    // Somewhere non-zero, so an implementation that forgot the viewport
    // origin cannot pass by it being (0, 0).
    let viewport_min = (137.0_f32, 91.0_f32);
    for &strip in &[(612.0_f32, 4000.0_f32), (1200.0, 500.0), (300.0, 200.0)] {
        for &page in &[(612.0_f32, 792.0_f32), (300.0, 200.0)] {
            for &origin in &[(0.0_f32, 0.0_f32), (0.0, 1200.0), (294.0, 2400.0)] {
                for &off in &[(0.0_f32, 0.0_f32), (100.0, 900.0)] {
                    // The content's origin on screen: the viewport's, less
                    // however far the area has been scrolled.
                    let content_min = (viewport_min.0 - off.0, viewport_min.1 - off.1);
                    // The page's top-left on screen: the content's origin,
                    // plus the strip's centring margin and the pasteboard,
                    // plus the page's own place inside the strip.
                    let page_min = (
                        content_min.0
                            + (strip.0.max(v.0) - strip.0) / 2.0
                            + pasteboard_rule(v.0)
                            + origin.0,
                        content_min.1
                            + (strip.1.max(v.1) - strip.1) / 2.0
                            + pasteboard_rule(v.1)
                            + origin.1,
                    );
                    let solved = page_local_offset(off, origin, strip, page, v, (0.0, 0.0));
                    let measured = offset_from_drawn(page_min, viewport_min, page, v);
                    assert!(
                        (measured.0 - solved.0).abs() < 1e-2
                            && (measured.1 - solved.1).abs() < 1e-2,
                        "strip={strip:?} page={page:?} origin={origin:?} off={off:?}: measured \
                             {measured:?} vs solved {solved:?}"
                    );
                }
            }
        }
    }
}

/// A non-finite rect cannot poison the next zoom's `offset_before`.
///
/// ★ Zero rather than the previous value, because this function has no
/// previous value to return — it is a measurement, not a step. "Centred"
/// is the safe fiction; a `NaN` propagates into `zoom_anchor_offset` and
/// blanks the canvas, which is the one outcome worse than a wrong offset.
#[test]
fn a_non_finite_drawn_rect_measures_as_centred_rather_than_as_nan() {
    let out = offset_from_drawn(
        (f32::NAN, f32::INFINITY),
        (0.0, 0.0),
        (612.0, 792.0),
        (800.0, 600.0),
    );
    assert_eq!(
        out,
        (0.0, 0.0),
        "a non-finite rect must not yield a NaN offset"
    );
}

/// [`strip_origin_offset`] is `(outer − display) / 2` wherever that
/// expression can still be evaluated exactly — which is the claim that
/// makes replacing one with the other safe.
///
/// ★ The magnitudes here are deliberately ordinary. The whole point of the
/// symbolic form is that it agrees with the plain one where the plain one
/// is trustworthy and continues to be right where it is not, and only the
/// first half of that is assertable in `f32` arithmetic — the second half
/// is what `zooming_back_out_keeps_the_view` drives.
#[test]
fn the_strip_origin_is_the_plain_expression_wherever_that_expression_is_exact() {
    for &vp in &[600.0_f32, 619.0, 1000.0] {
        for &display in &[100.0_f32, 599.0, 600.0, 1200.0, 40_000.0] {
            for &avail in &[400.0_f32, 600.0, 5_000.0] {
                let outer = content_extent(display, vp, 0.0).max(avail);
                let plain = (outer - display) / 2.0;
                let symbolic = strip_origin_offset(display, vp, avail, 0.0);
                assert!(
                    (plain - symbolic).abs() < 1e-3,
                    "display={display} vp={vp} avail={avail}: plain {plain} vs symbolic \
                         {symbolic}"
                );
            }
        }
    }
}

// ---- the fit placement (O28) ---------------------------------------

/// ★★★ **A pinned axis lands centred when the page fits, and flush when it
/// does not** — the property the whole of O28 rests on, and the reason the
/// pinned answer can be the single constant `0.0`.
///
/// Asserted through [`anchor_screen_pos`] rather than by re-stating the
/// arithmetic: what matters is not that the function returns zero, it is
/// **where the page's top-left ends up on screen** when it does. A test
/// that checked for zero would keep passing if [`margin`] were changed
/// underneath it, which is exactly the coupling this pins.
#[test]
fn a_pinned_axis_centres_a_page_that_fits_and_sits_flush_with_one_that_does_not() {
    let viewport = (800.0_f32, 600.0_f32);
    // Smaller than the viewport on both axes: fit-page's own case.
    let small = (500.0_f32, 400.0_f32);
    let placed = fit_placement_offset((true, true), (123.0, 456.0), small, viewport);
    let corner = anchor_screen_pos((0.0, 0.0), placed, small, viewport);
    assert!(
        (corner.0 - (viewport.0 - small.0) / 2.0).abs() < 1e-3
            && (corner.1 - (viewport.1 - small.1) / 2.0).abs() < 1e-3,
        "a page that fits must be centred, not merely at offset zero: {corner:?}"
    );

    // Exactly the viewport's width, which is what fit-width produces.
    let wide = (800.0_f32, 2000.0_f32);
    let placed = fit_placement_offset((true, false), (600.0, 300.0), wide, viewport);
    let corner = anchor_screen_pos((0.0, 0.0), placed, wide, viewport);
    assert!(
        corner.0.abs() < 1e-3,
        "a page exactly as wide as the viewport must sit flush to its left edge, so the full width shows and no pasteboard does: {corner:?}"
    );
}

/// An unpinned axis keeps where the operator was — and cannot keep them in
/// the pasteboard.
///
/// ★ Both halves in one test, because they are one rule. Keeping the
/// position is what stops "Fit width" throwing the operator back to the
/// top of a long sheet; clamping it is what stops "kept" meaning "still
/// looking at nothing".
#[test]
fn an_unpinned_axis_is_kept_but_clamped_to_the_page() {
    let viewport = (800.0_f32, 600.0_f32);
    let tall = (800.0_f32, 2000.0_f32);
    assert_eq!(
        fit_placement_offset((true, false), (0.0, 900.0), tall, viewport).1,
        900.0,
        "a position inside the page must survive a fit untouched"
    );
    assert_eq!(
        fit_placement_offset((true, false), (0.0, 5000.0), tall, viewport).1,
        tall.1 - viewport.1,
        "a position out in the pasteboard must be pulled back onto the page"
    );
    assert_eq!(
        fit_placement_offset((true, false), (0.0, -900.0), tall, viewport).1,
        0.0,
        "and so must one above the page's top, which the pasteboard also allows"
    );
}

/// A page shorter than the viewport has a clamp range of `[0, 0]`, so the
/// "kept" rule and the "centred" rule agree rather than fighting.
///
/// This is the landscape-sheet-under-fit-width case, and the one where a
/// `max(0.0)` on the wrong side would leave the page pinned to the top of
/// the window with a gap underneath.
#[test]
fn an_unpinned_axis_on_a_page_smaller_than_the_viewport_still_centres() {
    let viewport = (800.0_f32, 600.0_f32);
    let short = (800.0_f32, 300.0_f32);
    let placed = fit_placement_offset((true, false), (0.0, 250.0), short, viewport);
    let corner = anchor_screen_pos((0.0, 0.0), placed, short, viewport);
    assert!(
        (corner.1 - (viewport.1 - short.1) / 2.0).abs() < 1e-3,
        "a page shorter than the viewport must be centred on the free axis too: {corner:?}"
    );
}

/// A non-finite position cannot survive a fit.
#[test]
fn a_non_finite_position_falls_back_to_the_pinned_answer() {
    let out = fit_placement_offset(
        (false, false),
        (f32::NAN, f32::INFINITY),
        (800.0, 2000.0),
        (800.0, 600.0),
    );
    assert_eq!(out, (0.0, 0.0));
}

/// The two directions round-trip, so an offset handed to a single-page
/// solve and brought back is the offset it started as — within the strip's
/// scrollable range, which the return leg clamps to.
#[test]
fn the_strip_bridge_round_trips_within_the_scroll_range() {
    let v = (800.0_f32, 600.0_f32);
    let strip = (612.0_f32, 4000.0_f32);
    let page = (612.0_f32, 792.0_f32);
    let origin = (0.0_f32, 1200.0_f32);
    for &off in &[(0.0_f32, 0.0_f32), (0.0, 1500.0), (0.0, 3400.0)] {
        let local = page_local_offset(off, origin, strip, page, v, (0.0, 0.0));
        let back = strip_offset(local, origin, strip, page, v, (0.0, 0.0));
        assert!(
            (back.0 - off.0).abs() < 1e-2 && (back.1 - off.1).abs() < 1e-2,
            "{off:?} round-tripped to {back:?}"
        );
    }
}

/// The return leg never hands a `ScrollArea` an offset outside its range,
/// and a non-finite input yields the origin rather than a NaN that would
/// blank the canvas.
#[test]
fn the_return_leg_clamps_and_survives_a_nan() {
    let v = (800.0_f32, 600.0_f32);
    let strip = (612.0_f32, 4000.0_f32);
    let page = (612.0_f32, 792.0_f32);
    let out = strip_offset((99_000.0, 99_000.0), (0.0, 0.0), strip, page, v, (0.0, 0.0));
    assert_eq!(
        out,
        (
            content_extent(strip.0, v.0, 0.0) - v.0,
            content_extent(strip.1, v.1, 0.0) - v.1
        )
    );
    assert!(out.0 > 0.0, "a narrow page must still be pannable sideways");
    let out = strip_offset((-9_000.0, -9_000.0), (0.0, 0.0), strip, page, v, (0.0, 0.0));
    assert_eq!(out, (0.0, 0.0));
    assert_eq!(
        strip_offset((f32::NAN, 100.0), (0.0, 0.0), strip, page, v, (0.0, 0.0)).0,
        0.0
    );
    assert_eq!(
        page_local_offset((f32::NAN, 100.0), (0.0, 0.0), strip, page, v, (0.0, 0.0)).0,
        0.0
    );
}

#[test]
fn a_non_finite_input_refuses_to_move_rather_than_blanking_the_canvas() {
    // `anchor_frac` divides by the drawn page size, which is zero for one
    // frame after an open — so NaN really can reach here.
    let off0 = (120.0, 45.0);
    assert_eq!(
        zoom_anchor_offset(
            off0,
            (0.0, 0.0),
            (100.0, 100.0),
            (800.0, 800.0),
            (f32::NAN, 0.5)
        ),
        off0
    );
}

// ---- O78: a resize keeps what was centred, centred ---------------------

/// ★★★ **THE THEOREM THE WHOLE OF O78 RESTS ON.**
///
/// On an axis a fit **pins**, holding the page's own centre at the viewport
/// centre gives exactly the offset [`fit_placement_offset`] gives. So
/// preserving the centred point across a resize **subsumes** the fit's
/// re-placement, and `canvas::fit` can stop having a separate resize path.
///
/// If this ever fails, deleting that path changed what Fit page does — which
/// is why it is asserted over a range of sizes rather than at one, and why it
/// asserts the two functions agree rather than asserting each is zero.
#[test]
fn centring_agrees_with_the_pinned_fit_answer() {
    // A pinned axis is by construction one where the page fits, so every pair
    // here has display <= viewport.
    for &(d, v) in &[
        (100.0_f32, 100.0_f32),
        (100.0, 400.0),
        (399.9, 400.0),
        (1.0, 1000.0),
        (0.0, 400.0),
    ] {
        let centred = offset_holding_anchor_at((0.5, 0.5), (v / 2.0, v / 2.0), (d, d), (v, v));
        let fitted = fit_placement_offset((true, true), (17.0, -23.0), (d, d), (v, v));
        assert_eq!(
            centred, fitted,
            "d={d} v={v}: centring and the pinned fit answer must be the same number, or \
             deleting the resize path in `canvas::fit` changes what Fit page does"
        );
        assert_eq!(
            centred,
            (0.0, 0.0),
            "d={d} v={v}: and both are the page's origin"
        );
    }
}

/// Measuring the centred point and placing it back are exact inverses.
///
/// The sibling of `placing_an_anchor_and_measuring_it_are_exact_inverses`, and
/// written in the same shape: at an unchanged viewport the round trip must
/// return the offset it started from, or a frame with no resize would still
/// move the view.
#[test]
fn measuring_the_centred_point_and_placing_it_are_exact_inverses() {
    let display = (1600.0, 1200.0);
    let viewport = (900.0, 700.0);
    for &off in &[
        (0.0, 0.0),
        (250.0, -80.0),
        (1500.0, 1100.0),
        (-320.0, 640.0),
    ] {
        let frac = centred_frac(off, display, viewport);
        let back = offset_holding_anchor_at(
            frac,
            (viewport.0 / 2.0, viewport.1 / 2.0),
            display,
            viewport,
        );
        assert!(
            (back.0 - off.0).abs() < 1e-3 && (back.1 - off.1).abs() < 1e-3,
            "offset {off:?} did not survive the round trip: got {back:?}"
        );
    }
}

/// ★★★ **The operator's sentence, as arithmetic**: a wider viewport keeps the
/// same page point in the middle.
///
/// > *"when I change the size of the canvas window, whatever area was centered
/// > in the current canvas should stay centered."*
///
/// Asserted by measuring where the preserved fraction lands **after** the
/// resize, rather than by comparing offsets — an offset that happened to be
/// preserved for the wrong reason would pass a comparison of offsets and fail
/// this.
#[test]
fn a_resize_keeps_the_same_page_point_in_the_middle() {
    let display = (1600.0, 1200.0);
    let before = (900.0, 700.0);
    let off = (420.0, 310.0);
    let frac = centred_frac(off, display, before);

    for &after in &[
        (1200.0, 700.0),
        (600.0, 700.0),
        (900.0, 1000.0),
        (1400.0, 200.0),
    ] {
        let placed = offset_holding_anchor_at(frac, (after.0 / 2.0, after.1 / 2.0), display, after);
        let landed = anchor_screen_pos(frac, placed, display, after);
        assert!(
            (landed.0 - after.0 / 2.0).abs() < 1e-2 && (landed.1 - after.1 / 2.0).abs() < 1e-2,
            "viewport {after:?}: the preserved point landed at {landed:?}, not the middle"
        );
    }
}

/// A degenerate extent yields the middle of the page, not a NaN.
///
/// `offset_holding_anchor_at` fails to `0.0` because that is a legal *offset*;
/// this is a *fraction*, so the harmless value is `0.5`. Asserted because the
/// two guards look alike and choosing the wrong constant would put a NaN into
/// a scroll offset one call later.
#[test]
fn a_degenerate_extent_centres_rather_than_producing_a_nan() {
    let frac = centred_frac((0.0, 0.0), (0.0, 0.0), (400.0, 400.0));
    assert_eq!(frac, (0.5, 0.5));
    assert!(frac.0.is_finite() && frac.1.is_finite());
}

/// ★★ **The opening seed**: a page larger than the viewport starts at its own
/// centre, and one that fits starts exactly where it always did.
///
/// `OPERATOR_REQUESTS.md` O78: *"when starting the view should be centered on
/// the canvas when a pdf is first opened."*
///
/// The second half is what makes the change safe to ship: under the shipped
/// default (Fit page, so the page always fits) the new seed expression is
/// **exactly** the old literal `(0.0, 0.0)`, so nothing about the common path
/// moves.
#[test]
fn the_opening_seed_centres_a_large_page_and_is_a_no_op_for_a_small_one() {
    let v = (900.0, 700.0);
    let centre = |d: (f32, f32)| offset_holding_anchor_at((0.5, 0.5), (v.0 / 2.0, v.1 / 2.0), d, v);

    // Larger than the viewport: half the overflow, so the middle of the sheet
    // is in the middle of the window rather than its top-left corner.
    let big = (2000.0, 1500.0);
    let placed = centre(big);
    assert!((placed.0 - (big.0 - v.0) / 2.0).abs() < 1e-3, "{placed:?}");
    assert!((placed.1 - (big.1 - v.1) / 2.0).abs() < 1e-3, "{placed:?}");

    // Fits: identical to the literal the seed used before O78.
    assert_eq!(centre((400.0, 300.0)), (0.0, 0.0));
    assert_eq!(centre(v), (0.0, 0.0));
}

// ---- the pasteboard's overhang term — O23's second half ------------
//
//
// Every number below is the one measured on the real canvas: a 470 px-wide
// viewport, a 200 pt sheet, and an object 100 pt off its left edge.

/// The arithmetic of the old defect, stated so it cannot come back silently.
///
/// With a fixed one-viewport pasteboard, the slack is a count of **screen**
/// pixels, so the slice of the **drawing** it covers is `viewport / zoom` and
/// shrinks with every notch. Centring a point 100 pt off the sheet needs
/// `pasteboard ≥ 100 × zoom + viewport / 2`; with `pasteboard = viewport`
/// that is `zoom ≤ 235 / 100`, i.e. **235 %**. Above it the object walks off
/// the screen while the operator zooms toward it.
///
/// ★ O186 moved the crossover to **203 %** — `(470 − 32) − 235`, over 100 —
/// because [`MIN_SHEET_ON_SCREEN`] comes out of the fixed slack. The table
/// below is therefore re-measured rather than re-tuned, and the ceiling is
/// derived from the rule instead of written as a literal, so the next change to
/// either constant moves it without a second edit. The number itself does not
/// matter to anybody: this test pins the *shape* of the old defect — a ceiling
/// that exists at all — and the whole point of the overhang term is that the
/// ceiling does not apply when there is content out there to reach.
#[test]
fn a_fixed_pasteboard_stops_reaching_off_page_content_at_a_calculable_zoom() {
    const V: f32 = 470.0;
    const OFF_PTS: f32 = 100.0;
    let ceiling = (pasteboard_rule(V) - V / 2.0) / OFF_PTS; // 2.03
    for (zoom, reachable) in [(1.5_f32, true), (2.0, true), (2.1, false), (10.0, false)] {
        let over_px = OFF_PTS * zoom;
        // The OLD rule: no overhang term at all.
        let got = pasteboard(V, 0.0) >= over_px + V / 2.0;
        assert_eq!(
            got, reachable,
            "at {zoom}x (ceiling {ceiling}) a fixed pasteboard reaching={got}"
        );
    }
}

/// ★★★ **The fix: the off-page object is centreable at every zoom.**
///
/// `strip_offset` is the one function whose answer actually reaches the
/// `ScrollArea`, and it is the one that clamps — so a solve that is thrown
/// away by the clamp is indistinguishable, to the operator, from a solve that
/// was never made. This drives the whole chain: the offset that would put the
/// off-page point at the viewport's centre must survive the clamp unchanged.
///
/// Single page, so `page_origin` is `(0,0)` and `strip == page_display`; that
/// is the geometry the driven check `an_object_off_the_page_is_actually_drawn`
/// exercises, and the same shape as `off-page-object.pdf`.
#[test]
fn an_object_off_the_page_can_be_centred_at_every_zoom() {
    const V: f32 = 470.0;
    const PAGE_PTS: f32 = 200.0;
    const OFF_PTS: f32 = 100.0;
    for zoom in [0.5_f32, 1.0, 2.35, 5.0, 20.0, 100.0, 800.0] {
        let d = PAGE_PTS * zoom;
        let over = (OFF_PTS * zoom, 0.0);
        // The point is OFF the page, so its page-local x is negative; the
        // offset that centres it is that, less half a viewport.
        let want_local = (-OFF_PTS * zoom - V / 2.0, 0.0);
        let got = strip_offset(want_local, (0.0, 0.0), (d, d), (d, d), (V, V), over);
        // Round-tripping through the inverse is the honest check: it says the
        // clamp did not eat the answer, in the space the answer was asked in.
        let back = page_local_offset(got, (0.0, 0.0), (d, d), (d, d), (V, V), over);
        assert!(
            (back.0 - want_local.0).abs() < 0.01,
            "at {zoom}x the clamp moved the centring offset: wanted {}, got {}",
            want_local.0,
            back.0
        );
    }
}

/// The `max` in [`pasteboard`] keeps the operator's fixed slack whenever it is
/// the larger, so **every document with nothing off the sheet takes the
/// fraction branch and nothing else**. This is the regression guard for the
/// 99 % case.
///
#[test]
fn a_page_with_nothing_off_it_takes_the_fixed_pasteboard_branch() {
    for v in [1.0_f32, 470.0, 578.3, 2000.0] {
        assert_eq!(pasteboard(v, 0.0), pasteboard_rule(v));
        // ★★ Where the overhang term takes over, **stated rather than
        // assumed**: it offers `overhang + viewport / 2`, so it wins once
        // `overhang > viewport / 2 − sliver`. On any real canvas that is a
        // reach of hundreds of points and the quarter-viewport overhang below
        // is nowhere near it.
        //
        // ★ On a viewport of 1.0 pt it is **zero** — `sheet_sliver` is half the
        // viewport there, so the fixed slack has no advantage left and *any*
        // overhang at all wins. Harmless (the overhang branch is the more
        // generous one, and a 1 pt canvas shows nothing either way) but it is
        // why this assertion is guarded rather than unconditional. An earlier
        // draft asserted it for every viewport and failed on exactly that row
        // — which is the row that exists to be awkward.
        let crossover = viewport_half_less_sliver(v);
        if v / 4.0 < crossover {
            assert_eq!(pasteboard(v, v / 4.0), pasteboard_rule(v));
        }
        // And past the crossover the overhang branch really does take over, so
        // the `if` above is a guard on a reachable case rather than a way of
        // skipping the assertion everywhere.
        assert!(pasteboard(v, crossover + v) > pasteboard_rule(v));
        // ★ And the sliver is real: the slack is strictly less than a whole
        // viewport, which is the one property that removes the blank frame.
        // Stated as an inequality rather than a difference so it holds on the
        // 1.0 pt viewport too, where `sheet_sliver` is half the viewport
        // rather than the constant.
        assert!(
            pasteboard(v, 0.0) < v * PASTEBOARD_FRACTION,
            "viewport {v}: the pasteboard reaches a whole viewport, so its \
             extreme is the zero-overlap placement O186 reported"
        );
    }
}

/// A non-finite or negative overhang is ignored rather than propagated, on the
/// same rule every other input in this module follows: a frame measured before
/// layout must not be able to produce a NaN scroll content.
#[test]
fn nonsense_overhang_falls_back_to_the_fixed_pasteboard() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, -1e30] {
        assert_eq!(pasteboard(470.0, bad), pasteboard_rule(470.0));
        assert!(content_extent(1000.0, 470.0, bad).is_finite());
    }
}

/// ★★ The bound. The overhang term is multiplied by the zoom, so left alone it
/// would push the scroll content past the point where an `f32` offset stops
/// addressing every screen pixel — while the strip, which is what the `f64`
/// deep tier keys its hand-over on, was still below it. The cap is a quarter
/// of that constant, so the content extent stays inside one and a half times
/// the strip's own ceiling however far the content reaches.
#[test]
fn the_pasteboard_is_bounded_by_the_tier_the_deep_model_hands_over_at() {
    let cap = crate::viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT / 4.0;
    for over in [1e6_f32, 1e9, 1e30] {
        assert!(
            pasteboard(470.0, over) <= cap,
            "an overhang of {over} produced a pasteboard of {}",
            pasteboard(470.0, over)
        );
    }
    // And the cap is not a floor: an overhang under it is honoured exactly.
    let modest = cap / 2.0;
    assert_eq!(pasteboard(470.0, modest), modest + 470.0 / 2.0);
}

// ---- O186: the reachable range at the deep tier ---------------------

/// ★★★ **The theorem the whole function rests on: its two ends are the two
/// scroll offsets a `ScrollArea` can actually be at.**
///
/// `OPERATOR_REQUESTS.md` O186 stage one. Stated in the direction that is
/// *exact* in `f32` — take the two reachable scroll offsets, `0` and
/// `content_extent − viewport`, convert each into strip space with
/// [`scroll_to_strip`], and you get `lo` and `hi` on the nose. Both sides are
/// `x − strip_margin(...)` evaluated on the same arguments, so there is no
/// rounding to argue about.
///
/// ★ The reverse direction is asserted as *behaviour* rather than as an
/// identity, and the nudge is `64.0` rather than `1.0` on purpose: a round trip
/// through `hi + pad` is two roundings, and at the strip magnitudes this tier
/// reaches — nine hundred thousand points and up, where an `f32`'s step is a
/// sixteenth of a point — `1.0` is close enough to the boundary to be arguing
/// with the last bit rather than with the clamp. 64 points is unambiguous at
/// every magnitude tested and is still a fraction of a viewport.
#[test]
fn the_ends_of_the_range_are_the_two_scroll_offsets_the_area_can_reach() {
    for (strip, viewport, over) in [
        (1600.0_f32, 800.0_f32, 0.0_f32),
        (3200.0, 800.0, 0.0),
        (12800.0, 800.0, 0.0),
        (1600.0, 800.0, 2000.0),
        (600.0, 800.0, 0.0),
        (909_000.0, 700.0, 0.0),
    ] {
        let (lo, hi) = visible_origin_range(strip, viewport, over);
        let top = (content_extent(strip, viewport, over) - viewport).max(0.0);

        // Exact, both ends: the range IS the scroll clamp, seen from strip space.
        assert_eq!(
            scroll_to_strip(0.0, strip, viewport, over),
            lo,
            "strip {strip} viewport {viewport} overhang {over}: the bottom of the \
             range is not the offset the area sits at when fully scrolled back"
        );
        assert_eq!(
            scroll_to_strip(top, strip, viewport, over),
            hi,
            "strip {strip} viewport {viewport} overhang {over}: the top of the \
             range is not the offset the area sits at when fully scrolled on"
        );

        // And going the other way, a value outside the range reaches the bound
        // and no further — which is the property `deep::confine` relies on.
        assert_eq!(strip_to_scroll(lo - 64.0, strip, viewport, over), 0.0);
        assert_eq!(strip_to_scroll(hi + 64.0, strip, viewport, over), top);

        // A value strictly inside is not clamped at either end, so the range is
        // not degenerate for any case above.
        let mid = (lo + hi) * 0.5;
        let got = strip_to_scroll(mid, strip, viewport, over);
        assert!(
            got > 0.0 && got < top,
            "strip {strip} viewport {viewport} overhang {over}: the midpoint \
             {mid} mapped to {got}, which is outside (0, {top})"
        );
    }
}

/// ★★ **With no overhang the top of the range is the strip's own length, less
/// the sliver** — and the bottom is a whole viewport below zero, plus it.
///
/// A corollary rather than a separate rule, and worth pinning because it is the
/// shape every number in O186's trace block is read against:
/// `pasteboard = viewport × PASTEBOARD_FRACTION − sliver`, so
/// `hi = strip + pasteboard − viewport` collapses to `strip − sliver`. If
/// someone later changes either constant this test fails immediately and says
/// which identity the trace block's arithmetic was relying on, rather than
/// leaving a reader to wonder why a measured `hi` of 1684.27 nearly equalled a
/// measured sheet height.
///
/// ★★★ **The `− sliver` is the whole of O186's second half.** Without it `hi`
/// is `strip` exactly, which places the viewport's top-left on the strip's last
/// point: zero overlap, `canvas-unavailable reason=nothing-visible`, and a
/// clamp to that endpoint is a clamp onto a blank frame. Both ends are
/// asserted, because both ends were blank.
#[test]
fn without_an_overhang_the_top_of_the_range_is_the_strip_itself() {
    for (strip, viewport) in [
        (1600.0_f32, 800.0_f32),
        (12800.0, 800.0),
        (909_000.0, 700.0),
        (4.6e8, 1380.0),
    ] {
        let (lo, hi) = visible_origin_range(strip, viewport, 0.0);
        let sliver = MIN_SHEET_ON_SCREEN.min(viewport / 2.0);
        assert_eq!(hi, strip - sliver, "strip {strip} viewport {viewport}");
        // ★ Both endpoints leave the sliver on screen. At `hi` the strip's far
        // end is `sliver` inside the viewport's near edge; at `lo` its start is
        // `sliver` inside the far one. Asserted as the overlap itself — what
        // the operator can see — rather than as the two offsets, because the
        // offsets are what was already green while the canvas was blank.
        for origin in [lo, hi] {
            let overlap = (origin + viewport).min(strip) - origin.max(0.0);
            assert!(
                overlap >= sliver - 1.0e-2,
                "strip {strip} viewport {viewport} origin {origin}: only \
                 {overlap} pt of sheet on screen, wanted {sliver}"
            );
        }
        assert_eq!(lo, sliver - viewport, "strip {strip} viewport {viewport}");
    }
}

/// ★★★ **The interval is never inverted, for any input** — the property that
/// lets [`crate::canvas::deep::confine`] clamp with `max` then `min` and lets
/// a caller who used `f64::clamp` not panic.
///
/// `hi − lo` is `content_extent − viewport` by construction, which is
/// `2 × pasteboard` plus `display.max(viewport) − viewport ≥ 0`. The grid below
/// covers the cases a formula written out longhand gets wrong: a strip shorter
/// than the viewport, a strip of exactly zero, a strip equal to its viewport, a
/// degenerate viewport, and an overhang large enough to hit the pasteboard cap.
#[test]
fn the_range_is_never_inverted_however_short_the_strip() {
    for strip in [0.0_f32, 1.0, 400.0, 800.0, 1600.0, 1.0e6, 1.0e9] {
        for viewport in [0.0_f32, 1.0, 470.0, 800.0, 1.0e4] {
            for over in [0.0_f32, 1.0, 1.0e4, 1.0e9] {
                let (lo, hi) = visible_origin_range(strip, viewport, over);
                assert!(
                    lo.is_finite() && hi.is_finite(),
                    "strip {strip} viewport {viewport} overhang {over} gave \
                     ({lo}, {hi})"
                );
                assert!(
                    hi >= lo,
                    "strip {strip} viewport {viewport} overhang {over} inverted \
                     the interval: ({lo}, {hi})"
                );
            }
        }
    }
}

/// **A strip shorter than its viewport is free to sit anywhere in its
/// pasteboard window, and sits centred when nothing has moved it** — the
/// range's midpoint is exactly `-margin(strip, viewport)`.
///
/// Unreachable at the deep tier, where a strip is `pages × page × zoom` and the
/// tier engages only once `longest_page_pt × zoom` exceeds
/// [`crate::viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT`], and asserted anyway
/// because the function is ordinary geometry and a reader of the shallow tier
/// will hit it. The answer must be the *shallow* tier's answer: centring a display
/// smaller than its viewport is what [`margin`] and [`fit_placement_offset`]
/// already do, and a range whose midpoint was anything else would mean the two
/// tiers disagree about where a small document rests.
#[test]
fn a_short_strips_range_is_centred_on_the_centring_margin() {
    for (strip, viewport) in [
        (600.0_f32, 800.0_f32),
        (100.0, 470.0),
        (0.0, 800.0),
        (792.0, 1380.0),
    ] {
        let (lo, hi) = visible_origin_range(strip, viewport, 0.0);
        assert_eq!(
            (lo + hi) * 0.5,
            -margin(strip, viewport),
            "strip {strip} viewport {viewport}: range ({lo}, {hi})"
        );
    }
}

/// ★★★ **O186 itself, in the numbers the trace printed** — the measured case
/// this function was written for, clamped the way
/// [`crate::canvas::deep::confine`] clamps it, in **both** directions.
///
///
/// ★★ The two things this pins that a one-sided test would not:
///
/// * **the magnitude is the whole defect.** 0.05 pt is nothing; `0.05 × 539.7`
///   is 27 logical points of strip, which against a 700-point viewport is
///   enough to carry the entire strip off it. The assertion on the recovered
///   page coordinate is what keeps that factor honest — a future change that
///   clamped to the right side of the wrong bound would still satisfy
///   `clamped == hi`.
/// * **both signs.** A suite that only tries one sign is not testing the value;
///   the anchor in the trace had *also* been seen at `-0.54` pt, above the
///   sheet rather than below it, which is the same defect reached from the other
///   end of the pasteboard.
#[test]
fn the_o186_anchor_is_pulled_back_to_the_end_of_the_sheet_from_either_side() {
    let zoom = 539.7_f32;
    let sheet = 1684.27_f32; // the A1 sheet's height, in page points
    let strip = sheet * zoom;
    let viewport = 700.0_f32;

    let (lo, hi) = visible_origin_range(strip, viewport, 0.0);

    // Where the anchor had walked to: a hair past the end of the sheet.
    let want = 1684.32_f32 * zoom;
    assert!(
        want > hi,
        "the measured anchor {want} is not outside the range ({lo}, {hi}), so \
         this test no longer reproduces O186"
    );

    // Clamped exactly as `deep::confine` clamps it.
    let clamped = want.max(lo).min(hi);
    assert_eq!(clamped, hi);

    // ★★ And the correction, converted back into page points the way `confine`
    // converts it, is the 0.05 pt of overshoot the trace shows **plus the
    // sliver**, because the range now stops short of the sheet's end by
    // [`MIN_SHEET_ON_SCREEN`]. Stated as the sum of the two terms rather than as
    // a re-measured literal: the interesting fact is the second term's size in
    // paper, and it is the argument for why the sliver costs the operator
    // nothing where it matters most.
    let sliver_in_paper = MIN_SHEET_ON_SCREEN.min(viewport / 2.0) / zoom;
    assert!(
        sliver_in_paper < 0.06,
        "at {zoom}x a 32 pt sliver should be a twentieth of a point of paper, \
         not {sliver_in_paper}"
    );
    let correction = (want - clamped) / zoom;
    assert!(
        (correction - (0.05 + sliver_in_paper)).abs() < 1.0e-3,
        "the correction came back as {correction} pt of paper, not \
         0.05 + {sliver_in_paper}"
    );

    // The other sign: the same anchor was seen 0.54 pt above the sheet.
    let above = lo - 0.54 * zoom;
    assert!(above < lo);
    assert_eq!(above.max(lo).min(hi), lo);
}

/// **A frame measured before layout clamps everything to the strip's own
/// origin** — `(0.0, 0.0)`, the answer every other function in this module
/// gives for a non-finite input.
///
/// Not a tolerance and not a fallback to the last good value: a degenerate pair
/// makes [`crate::canvas::deep::confine`] pin the anchor at the strip's origin
/// for that one frame, which is the one placement that needs no history and
/// cannot be wrong about anything.
#[test]
fn a_nonsense_frame_collapses_the_range_to_the_strips_origin() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(visible_origin_range(bad, 800.0, 0.0), (0.0, 0.0));
        assert_eq!(visible_origin_range(1600.0, bad, 0.0), (0.0, 0.0));
    }
    // ★ An overhang is the exception, because `pasteboard` already absorbs a
    // nonsense one rather than propagating it — see
    // [`nonsense_overhang_falls_back_to_the_fixed_pasteboard`]. So the range
    // stays usable, which is the behaviour the rest of the module promises.
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, -1.0e30] {
        let (lo, hi) = visible_origin_range(1600.0, 800.0, bad);
        assert!(lo.is_finite() && hi.is_finite() && hi >= lo);
        assert_eq!((lo, hi), visible_origin_range(1600.0, 800.0, 0.0));
    }
}

/// ★★★ **The draft formula this function replaced, measured** — and it was
/// wrong in two ways, neither of which was the one its own guard was written
/// for.
///
/// The draft was `(-pb, (strip + pb - viewport).max(-pb))`. See
/// [`visible_origin_range`]'s doc for the argument; this is the measurement
/// behind it, and it exists because the doc originally claimed the `.max` was
/// *needed*. It is not. Writing the test is what said so.
///
/// ⇒ Two lessons, both general: **a guard whose precondition is decided by a
/// constant elsewhere in the module cannot be read locally** — `.max(-pb)`
/// looks essential and is dead — and **a fresh formula for a range some
/// existing function already clamps to re-derives the range and invents its own
/// faults**, which here meant a window narrower than the scroll area's by
/// exactly the centring margin.
#[test]
fn the_draft_longhand_range_was_wrong_in_the_two_ways_measuring_it_found() {
    // 1. The inversion the `.max` guarded cannot happen at these constants.
    //    It needs `strip < viewport - 2 * pasteboard`, and with the pasteboard
    //    a whole viewport less the sliver that is `2 * sliver - viewport`,
    //    which no strip reaches because no strip is negative.
    //
    // ★★★ And **`sheet_sliver`'s `min` is exactly what keeps that true.** The
    // bound is `2 × sliver − viewport`, so it is at most zero precisely while
    // `sliver ≤ viewport / 2` — which is the clamp `sheet_sliver` applies, put
    // there for a quite different reason (a non-negative pasteboard on a
    // degenerate canvas). Raise `MIN_SHEET_ON_SCREEN` past half a viewport
    // without that `min` and the draft's dead case comes back to life.
    //
    for viewport in [1.0_f32, 470.0, 800.0, 1.0e4] {
        let pb = pasteboard(viewport, 0.0);
        assert_eq!(pb, pasteboard_rule(viewport));
        assert!(
            viewport - 2.0 * pb <= 0.0,
            "viewport {viewport}: the draft's inverted-interval case is \
             reachable after all, at strip < {}",
            viewport - 2.0 * pb
        );
    }

    // 2. The fault that was real: for a strip shorter than its viewport the
    //    draft's top is exactly `margin` below the reachable one, so a short
    //    strip could not be pushed as far down its window as egui allows.
    for (strip, viewport) in [(600.0_f32, 800.0_f32), (0.0, 800.0), (100.0, 470.0)] {
        let pb = pasteboard(viewport, 0.0);
        let draft_hi = (strip + pb - viewport).max(-pb);
        let (_, hi) = visible_origin_range(strip, viewport, 0.0);
        assert_eq!(
            hi - draft_hi,
            margin(strip, viewport),
            "strip {strip} viewport {viewport}: the draft top was {draft_hi}, \
             the reachable one is {hi}"
        );
    }

    // And where the strip exceeds its viewport — every case the deep tier can
    // actually be in — the two agree exactly, which is why the draft survived
    // reading and only failed measuring.
    for (strip, viewport) in [(1600.0_f32, 800.0_f32), (909_000.0, 700.0)] {
        let pb = pasteboard(viewport, 0.0);
        let draft_hi = (strip + pb - viewport).max(-pb);
        let (_, hi) = visible_origin_range(strip, viewport, 0.0);
        assert_eq!(hi, draft_hi, "strip {strip} viewport {viewport}");
    }
}
