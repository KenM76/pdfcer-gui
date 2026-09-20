//! Tests for [`super`] — the canvas overlay's rectangles, ghosts and grips.
//!

#![cfg(test)]

use super::*;
use egui::{Pos2, pos2};

use super::raster::blit_of;

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

    let ghost = ghost_box(&map, &state, None).expect(
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
    assert_eq!(ghost_box(&map, &SelectionState::default(), None), None);
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

/// ★★★ **THE TRAVELLING COPY IS TAKEN FROM THE RECTANGLE THE TEXTURE IS A
/// PICTURE OF, AND IT KEEPS ITS SCALE** — `OPERATOR_REQUESTS.md` O215 ask 5.
///
/// Every number below is worked out from the definition of the projection
/// rather than read off [`blit_of`], because an expectation produced by the
/// function under test agrees with it however wrong both are.
///
/// ⚠ The middle case is the decisive one. A chunk half off the painted
/// raster must lose the same amount from BOTH rectangles: cropping the UV and
/// clamping the destination to the full width would stretch the lettering to
/// fill it, and a picture travelling at the wrong size looks like a rendering
/// quirk rather than like clipping.
#[test]
fn a_travelling_copy_samples_the_painted_rect_and_never_stretches() {
    // A raster of a REGION, not of the page: 200x200 points at (100, 100).
    let source = Rect::from_min_max(pos2(100.0, 100.0), pos2(300.0, 300.0));
    let shift = egui::vec2(40.0, 40.0);

    // Wholly inside. UV is the offset into `source` over `source`'s size.
    let inside = blit_of(
        Rect::from_min_max(pos2(150.0, 120.0), pos2(250.0, 140.0)),
        source,
        shift,
    )
    .expect("a chunk wholly over the raster has a blit");
    assert!(
        !inside.cropped,
        "nothing was cut, so nothing may be reported cut"
    );
    assert_eq!(
        inside.to,
        Rect::from_min_max(pos2(190.0, 160.0), pos2(290.0, 180.0)),
        "the piece lands at the chunk's own rect moved by the shift"
    );
    for (got, want, axis) in [
        (inside.uv.min.x, 0.25, "uv.min.x"),
        (inside.uv.min.y, 0.10, "uv.min.y"),
        (inside.uv.max.x, 0.75, "uv.max.x"),
        (inside.uv.max.y, 0.20, "uv.max.y"),
    ] {
        assert!(
            (got - want).abs() < 1e-6,
            "{axis}: {got} vs {want} — the UV is relative to the painted \
             rect, and reading it off the page rect samples the wrong pixels \
             wherever a region's raster is in flight"
        );
    }

    // Straddling the left edge: 40 of its 100 points hang off the raster.
    let straddling = blit_of(
        Rect::from_min_max(pos2(60.0, 120.0), pos2(160.0, 140.0)),
        source,
        shift,
    )
    .expect("a chunk partly over the raster has a blit of the part that is");
    assert!(
        straddling.cropped,
        "40 of 100 points were cut and that is disclosed"
    );
    assert!(
        (straddling.to.width() - 60.0).abs() < 1e-6,
        "the destination lost exactly what the source lost: {} wide, not 60. \
         Clamping one side alone stretches the lettering.",
        straddling.to.width()
    );
    assert_eq!(
        straddling.to,
        Rect::from_min_max(pos2(140.0, 160.0), pos2(200.0, 180.0)),
        "the surviving piece lands where that piece was, moved by the shift"
    );
    assert!(
        (straddling.uv.min.x - 0.0).abs() < 1e-6 && (straddling.uv.max.x - 0.30).abs() < 1e-6,
        "the UV starts at the raster's own edge: {:?}",
        straddling.uv
    );

    assert!(
        blit_of(
            Rect::from_min_max(pos2(400.0, 400.0), pos2(450.0, 420.0)),
            source,
            shift
        )
        .is_none(),
        "a chunk scrolled clear of the painted raster has no piece to copy, \
         which is a position rather than an error"
    );
    assert!(
        blit_of(
            Rect::from_min_max(pos2(150.0, 120.0), pos2(250.0, 140.0)),
            Rect::from_min_max(pos2(100.0, 100.0), pos2(100.0, 100.0)),
            shift
        )
        .is_none(),
        "a degenerate raster would divide the UV by zero"
    );
}

/// A shape preview carrying **real geometry** — the one input for which
/// withholding is correct.
///
/// Its subpath list is empty, because every predicate under test reads
/// `ShapePreview::is_empty`, which asks whether there is a SHAPE — one shape
/// is what makes the operator see anchors travel.
fn a_preview_with_geometry_in_it() -> crate::canvas::shapes::ShapePreview {
    crate::canvas::shapes::ShapePreview {
        shapes: vec![crate::canvas::shapes::PreviewShape {
            subpaths: Vec::new(),
            style: pdfcer_core::vector::PaintStyle {
                fill: None,
                stroke: true,
            },
            line_width: 1.0,
        }],
        erase: Vec::new(),
        capped: false,
    }
}

/// ★★★ **THE GHOST IS WITHHELD ONLY WHERE SOMETHING BETTER IS ON SCREEN** —
/// `OPERATOR_REQUESTS.md` O63 and O215 ask 5.
///
/// The gate [`ghost_is_owed`] replaced read *is this an inner rung*, which is
/// not the same question and had the same answer until a text chunk became
/// selectable. Then a chunk drag previewed **nothing at all**: the ghost was
/// withheld, `draw_selection` is gated on the same flag so no outline was
/// drawn, and the chunk boxes are painted where the text still is.
///
/// ⚠ **The middle row is the one that matters.** `shapes::transformed` returns
/// a preview with an EMPTY `shapes` vec for a text object, having no path to
/// transform. A gate that asked `is_some()` would read that as *the real
/// geometry is travelling* and withhold, which is the defect restated in a
/// different word.
#[test]
fn the_ghost_is_withheld_only_for_a_preview_that_has_something_in_it() {
    let empty = crate::canvas::shapes::ShapePreview {
        shapes: Vec::new(),
        erase: Vec::new(),
        capped: false,
    };
    assert!(
        ghost_is_owed(false, None),
        "an inner rung with no shape preview at all is a text chunk being \
         dragged, and withholding there leaves the gesture with no feedback"
    );
    assert!(
        ghost_is_owed(false, Some(&empty)),
        "a preview that exists and draws nothing shows the operator nothing, \
         so it may not stand in for the ghost"
    );
    assert!(
        ghost_is_owed(true, None),
        "the object rung is always owed a ghost"
    );
    assert!(
        ghost_is_owed(true, Some(&empty)),
        "the object rung is always owed a ghost, whatever the preview says"
    );

    // ★★ The FALSE case. Without it every assertion above is satisfied by a
    // function that returns `true` and reads nothing.
    let travelling = a_preview_with_geometry_in_it();
    assert!(
        !ghost_is_owed(false, Some(&travelling)),
        "the real anchors are already travelling at the inner rung, so a ghost \
         of the same thing draws it twice"
    );
    assert!(
        ghost_is_owed(true, Some(&travelling)),
        "the object rung states the SET, so its box is owed even while the \
         geometry inside it travels"
    );
}

/// ★★★ **THE TRAVELLING COPY IS WITHHELD ONLY WHERE THE GEOMETRY ITSELF
/// MOVES** — `OPERATOR_REQUESTS.md` O215 ask 5.
///
/// [`raster_ghost_is_owed`] decides whether a translucent copy of the page's
/// own pixels travels with the pointer. It withholds on exactly one ground:
/// the real geometry is already moving on screen.
///
/// ⚠ The empty preview is the trap. `shapes::transformed` returns a preview
/// that EXISTS and is EMPTY for a text object, so a predicate spelled
/// `already_travelling.is_none()` withholds the lettering while the outline
/// still draws — a box travelling with none of the operator's words in it,
/// which is the O215 defect in a spelling that reads as a fix.
#[test]
fn the_travelling_copy_is_withheld_only_when_the_geometry_itself_moves() {
    let empty = crate::canvas::shapes::ShapePreview::default();
    assert!(
        raster_ghost_is_owed(None),
        "a text chunk drag has no shape preview at all, and it is the gesture \
         this copy exists for"
    );
    assert!(
        raster_ghost_is_owed(Some(&empty)),
        "a preview that exists and paints nothing shows the operator nothing, \
         so it may not stand in for the travelling copy"
    );
    assert!(
        !raster_ghost_is_owed(Some(&a_preview_with_geometry_in_it())),
        "the anchors are already visibly travelling, and a blitted copy of the \
         same content doubles it"
    );
}
