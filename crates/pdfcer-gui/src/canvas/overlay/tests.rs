//! Tests for [`super`] — the canvas overlay's rectangles, ghosts and grips.
//!
//! Split out on 2026-09-08 for R2 (no source file over 1,500 lines) when O154's
//! `ghost_box` and its two tests pushed `overlay.rs` past the limit. **Nothing
//! else moved**: the same `mod tests` block, de-indented, with its parent's
//! private items still reachable because a child module can see them.

#![cfg(test)]

use super::*;
use egui::{Pos2, pos2};

/// ★★★ **A SELECTED ANNOTATION HAS A GHOST BOX** — `OPERATOR_REQUESTS.md`
/// O154, and this is the assertion whose absence let the defect ship.
///
/// > *"the Markup Items don't have a live preview — the bounding box stays
/// > the same size when I drag the handles."*
///
/// [`grip_box`] reads `SelectionState::outlines`, which holds **page
/// content** entries; a markup annotation's box lives on `AnnotSelection`.
/// So `grip_box` answered `None` for every annotation, the `if let` in
/// `canvas::painting` never ran, and the resize ghost was unreachable for
/// exactly the selections he was dragging.
///
/// ⚠ **Both halves are asserted, and the second is the one that keeps this
/// honest.** A `ghost_box` that simply answered `Some` for everything
/// would satisfy the first; the second pins that it is the *annotation's*
/// rectangle and not some other box that happens to exist.
///
/// ★ And `grip_box`'s own behaviour is asserted **unchanged**, because the
/// tempting fix was to widen it instead — which would have altered what
/// `pressing::grabbable` measures and what the published `canvas-grip-box`
/// rect means to a driven check. Two callers wanting the content box is
/// why this is a second function rather than an edit to the first.
#[test]
fn an_annotations_ghost_box_is_its_own_rect_and_grip_box_is_left_alone() {
    use crate::canvas::selection::{AnnotKind, AnnotSelection, AnnotTarget, SelectionState};
    use pdfcer_core::object::ObjId;

    let map = crate::canvas::mapping::PageMapping::new(
        Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(600.0, 800.0)),
        (600.0, 800.0),
        1.0,
    );
    let outline = Rect::from_min_size(pos2(10.0, 20.0), egui::vec2(40.0, 30.0));

    let mut state = SelectionState::default();
    state.select_annot(AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id: ObjId::new(7, 0),
            kind: AnnotKind::Markup,
            // ui-text-exempt: a PDF /Subtype name in a test fixture.
            subtype: "Square".to_owned(),
            locked: false,
        },
        outline,
        oriented: None,
    });

    let ghost = ghost_box(&map, &state).expect(
        "★★★ an annotation must have a ghost box. Without one `canvas::painting`'s \
         `if let` never runs and a markup previews NOTHING while its grips are dragged \
         — which is O154, reported as \"the bounding box stays the same size\"",
    );
    let expected = visible_outline_rect(map.rect_to_screen(outline), MIN_OUTLINE_EXTENT_PX);
    assert!(
        (ghost.min - expected.min).length() < 0.01 && (ghost.max - expected.max).length() < 0.01,
        "★★ the ghost must be the ANNOTATION's rectangle. Got {ghost:?}, wanted {expected:?}"
    );

    assert_eq!(
        grip_box(&map, &state),
        None,
        "★ `grip_box` must be left alone: it answers for a CONTENT selection, and its two \
         callers — `pressing::grabbable`'s content fallthrough and the published \
         `canvas-grip-box` a driven check aims at — both mean that. Widening it was the \
         tempting fix and it would have changed what those two measure"
    );
}

/// ★★ **With nothing selected there is no ghost**, which is the half a
/// blanket `Some` would break.
///
/// A ghost box for an empty selection would put a preview rectangle on
/// screen for a gesture aimed at nothing — and, worse, it would make the
/// assertion above pass on a build that had learned nothing about
/// annotations.
#[test]
fn nothing_selected_has_no_ghost_box() {
    use crate::canvas::selection::SelectionState;
    let map = crate::canvas::mapping::PageMapping::new(
        Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(600.0, 800.0)),
        (600.0, 800.0),
        1.0,
    );
    assert_eq!(ghost_box(&map, &SelectionState::default()), None);
}

/// ★ A zero-height rule gets a visible band rather than nothing.
#[test]
fn a_degenerate_outline_is_grown_until_it_can_be_seen() {
    // The measured case: `100 200 m 300 200 l S`, projected to screen.
    let rule = Rect::from_min_max(pos2(100.0, 200.0), pos2(300.0, 200.0));
    let out = visible_outline_rect(rule, MIN_OUTLINE_EXTENT_PX);
    assert!(out.height() >= MIN_OUTLINE_EXTENT_PX);
    assert!(
        (out.width() - 200.0).abs() < f32::EPSILON,
        "the axis that was already visible must not be touched"
    );
    assert!(
        (out.center().y - 200.0).abs() < f32::EPSILON,
        "the band must straddle the rule, not sit to one side of it"
    );
}

/// A comfortable rect is returned unchanged — the growth is a repair, not
/// a permanent inflation that would misreport every object's extent.
#[test]
fn a_healthy_outline_is_left_alone() {
    let r = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 80.0));
    assert_eq!(visible_outline_rect(r, MIN_OUTLINE_EXTENT_PX), r);
}

/// An inside-out rect normalises before it is grown, so a projection that
/// swapped the corners still paints.
#[test]
fn an_inside_out_rect_normalises_before_growing() {
    let backwards = Rect::from_min_max(pos2(300.0, 240.0), pos2(100.0, 200.0));
    let out = visible_outline_rect(backwards, MIN_OUTLINE_EXTENT_PX);
    assert!(out.width() > 0.0 && out.height() > 0.0);
    assert!(out.contains(pos2(200.0, 220.0)));
}

/// A non-finite rect is left exactly as it arrived: there is no
/// meaningful centre to grow about, and repairing it here would hide a
/// bug that belongs upstream.
#[test]
fn a_non_finite_rect_is_returned_unchanged() {
    let nan = Rect::from_min_max(pos2(f32::NAN, 0.0), pos2(10.0, 10.0));
    let out = visible_outline_rect(nan, MIN_OUTLINE_EXTENT_PX);
    assert!(out.min.x.is_nan());
    // And a nonsense minimum is refused rather than shrinking the rect.
    let r = Rect::from_min_max(Pos2::ZERO, pos2(10.0, 10.0));
    assert_eq!(visible_outline_rect(r, -1.0), r);
    assert_eq!(visible_outline_rect(r, f32::NAN), r);
}

/// The wash keeps its hue and drops its alpha, so the content under a
/// rubber-band stays readable.
///
/// Asserted through `to_srgba_unmultiplied` rather than through `.r()`,
/// and **approximately**. Both halves of that are the point:
///
/// - [`Color32`] stores **premultiplied** components, so a translucent
///   blue reads back as `(11, 23, 38)` from the plain accessors and looks
///   as though the hue was lost. It was not, and "fixing" that by dropping
///   the alpha would be the wrong repair.
/// - Premultiplying at alpha 48 and dividing back out is lossy — 60
///   returns as 58 — so exact equality would be asserting the precision of
///   egui's colour storage rather than the property this function has.
#[test]
fn the_marquee_wash_is_translucent_and_keeps_the_themes_hue() {
    // NOT A THEME COLOUR: a test fixture standing in for whatever the
    // theme supplies; the assertion is that the hue survives, so the
    // exact input has to be a known literal.
    let base = Color32::from_rgb(60, 120, 200);
    let [r, g, b, a] = wash(base).to_srgba_unmultiplied();
    for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
        assert!(
            got.abs_diff(want) <= 4,
            "the wash drifted off the theme's hue: {got} vs {want}"
        );
    }
    assert!(a < 64, "a rubber-band must not hide what it encloses");
}

/// The ghost keeps the theme's hue and is translucent — visibly a *copy*
/// of the outline rather than a second, competing selection.
///
/// Asserted through `to_srgba_unmultiplied` for the reason [`ghost`]'s own
/// docs give, and approximately because premultiplying and dividing back
/// out is lossy.
#[test]
fn the_move_ghost_is_translucent_and_keeps_the_themes_hue() {
    // NOT A THEME COLOUR: test fixture, as above.
    let base = Color32::from_rgb(60, 120, 200);
    let [r, g, b, a] = ghost(base).to_srgba_unmultiplied();
    for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
        assert!(
            got.abs_diff(want) <= 4,
            "the ghost drifted: {got} vs {want}"
        );
    }
    assert_eq!(a, GHOST_ALPHA);
    assert!(
        a > 64,
        "the ghost must be readable over dense linework, unlike the marquee wash"
    );
}

/// ★ **The current find hit is distinguished by emphasis, not by hue.**
///
/// Both halves are asserted because both are the design:
///
/// - the two alphas differ by enough to read at a glance, so a page of
///   hits shows *which one* the readout is counting;
/// - the hue is the theme's, unchanged, in both — a find highlight that
///   borrowed `warn_fg_color` would say *warning* about something that is
///   not a warning, and would break the first time somebody restyled the
///   warning colour for warnings. `tools/gates/check-theme-colors.sh`
///   enforces the general rule; this asserts the specific consequence.
///
/// The second signal — the stroke on the current hit — is structural
/// rather than a colour and is asserted by reading [`draw_find_hits`],
/// which strokes if and only if `current`.
#[test]
fn the_current_find_hit_differs_by_emphasis_and_keeps_the_themes_hue() {
    // NOT A THEME COLOUR: a test fixture standing in for whatever the
    // theme supplies; the assertion is that the hue survives, so the exact
    // input has to be a known literal.
    let base = Color32::from_rgb(60, 120, 200);
    let ordinary = at_alpha(base, HIT_ALPHA);
    let current = at_alpha(base, CURRENT_ALPHA);

    for colour in [ordinary, current] {
        let [r, g, b, _] = colour.to_srgba_unmultiplied();
        for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
            assert!(
                got.abs_diff(want) <= 6,
                "a find highlight drifted off the theme's hue: {got} vs {want}"
            );
        }
    }

    // ★ The three relations between the two alphas are checked at
    // COMPILE time rather than here.
    //
    // They are properties of two constants, so a run-time assertion would
    // only re-discover what the compiler can refuse outright — the same
    // argument `crate::app::status`'s `HEIGHT_PTS > ROW_HEIGHT_PTS`
    // makes. They live inside this test rather than beside the constants
    // so the whole colour argument is readable in one place.
    const _: () = assert!(
        CURRENT_ALPHA > HIT_ALPHA * 2,
        // ui-text-exempt: compile-error text, never displayed in the UI
        "the current hit must be obviously different from its neighbours; alpha is one \
         of the two signals and it must not be a subtle one"
    );
    const _: () = assert!(
        HIT_ALPHA < 96,
        // ui-text-exempt: compile-error text, never displayed in the UI
        "a highlight that hides the text it is highlighting defeats its own purpose"
    );
    // ★ The bound that came from a screenshot rather than from reasoning.
    // At 168 the current hit was a solid block over its own word; see
    // `CURRENT_ALPHA`'s docs. 112 is the ceiling that keeps ordinary black
    // text legible through the theme's selection blue in both presets.
    const _: () = assert!(
        CURRENT_ALPHA <= 112,
        // ui-text-exempt: compile-error text, never displayed in the UI
        "the operator's next act after finding a hit is to READ it; a wash this \
         opaque covers the word it is marking"
    );
}

/// ★ **The text-selection wash is readable through** — the bound the
/// current-hit defect established, applied to the surface that needs it
/// most.
///
/// A find highlight marks one of several candidate answers; a text
/// selection marks *the characters that are about to be copied*, and the
/// operator's only way to check them is to read them. So the ceiling is
/// asserted at compile time against the same value
/// [`CURRENT_ALPHA`]'s own screenshot-derived bound uses, and the hue is
/// asserted to be the theme's — a selection wash that borrowed a named
/// palette entry would break the first time somebody restyled it for its
/// real purpose. `tools/gates/check-theme-colors.sh` enforces the general
/// rule; this asserts the specific consequence.
#[test]
fn the_text_selection_wash_is_readable_through_and_keeps_the_themes_hue() {
    // NOT A THEME COLOUR: a test fixture standing in for whatever the theme
    // supplies; the assertion is that the hue survives, so the exact input
    // has to be a known literal.
    let base = Color32::from_rgb(60, 120, 200);
    let [r, g, b, a] = at_alpha(base, TEXT_SELECTION_ALPHA).to_srgba_unmultiplied();
    for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
        assert!(
            got.abs_diff(want) <= 6,
            "the selection wash drifted off the theme's hue: {got} vs {want}"
        );
    }
    assert_eq!(a, TEXT_SELECTION_ALPHA);

    const _: () = assert!(
        TEXT_SELECTION_ALPHA <= CURRENT_ALPHA,
        // ui-text-exempt: compile-error text, never displayed in the UI
        "a text selection is what the operator is about to COPY, and the only way to \
         check it is to read it — it must never be more opaque than the find hit whose \
         opacity was already measured down from a solid block"
    );
    const _: () = assert!(
        TEXT_SELECTION_ALPHA > 0,
        // ui-text-exempt: compile-error text, never displayed in the UI
        "a selection nobody can see is a selection nobody can aim"
    );
}

/// A translucent source colour does not get darkened twice — the failure
/// the accessor choice in [`ghost`] guards against.
#[test]
fn a_translucent_theme_colour_keeps_its_hue_through_the_ghost() {
    // NOT A THEME COLOUR: test fixture — a deliberately translucent
    // source, which is the input this test exists to exercise.
    let translucent = Color32::from_rgba_unmultiplied(60, 120, 200, 90);
    let [r, g, b, _] = ghost(translucent).to_srgba_unmultiplied();
    for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
        assert!(
            got.abs_diff(want) <= 6,
            "premultiplied components were re-premultiplied: {got} vs {want}"
        );
    }
}
