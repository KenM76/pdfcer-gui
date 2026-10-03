//! # `canvas::dimdrag` tests — the placement arithmetic and the corner verbs
//!
//!
//! ## Two kinds of test live here and they cost very different things
//!
//! The **placement** tests below are pure — [`super::placed`] takes a
//! `DimensionKind` and two `f64`s — so they are four lines each and assert
//! arithmetic.
//!
//! The **corner-count** tests are not, and deliberately so. Whether a corner
//! may be removed is `pdfcer-core`'s ruling, evaluated by
//! `EditSession::vertex_edit_preview` against a real sidecar record, and a
//! shell-side re-implementation of the minimum-count rule is exactly the *"two
//! things that must agree and eventually will not"* the engine's own doc
//! comment argues against by name. So these open a fixture, author a real ce
//! dimension into it, and drive [`super::count_edit`] against the engine — the
//! only shape of test that can fail when the engine's ruling changes, which is
//! the property that makes it worth the second or two it costs.
//!
//! ## The vacuous shape this module was written to avoid
//!
//! A vertex test on a shape with **three** corners where insert and remove
//! cannot produce a degenerate case proves nothing. Both of the shapes here are
//! chosen so that removing a corner reaches the boundary: a **closed** triangle
//! is at the minimum (three) and must refuse, and an **open** three-point path
//! is one above its minimum (two) and must succeed, leaving a straight line.
//! The same shape, closed or not, gives opposite answers — which is the whole
//! of the rule, and a test on a square would have exercised neither side of it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/dimdrag/tests.md`.

// The INNER attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and without it every assertion message here
// is reported as operator-facing copy.
#![cfg(test)]

use super::*;
use crate::canvas::tool::CanvasTool;
use pdfcer_core::vector::AxisConstraint;

fn horizontal() -> DimensionKind {
    DimensionKind::Linear {
        a: Point::new(100.0, 200.0),
        b: Point::new(300.0, 200.0),
        constraint: AxisConstraint::Horizontal,
        offset: 0.0,
        text_along: 0.0,
        extension_gap: [None; 2],
    }
}

/// A horizontal dimension's frame is `u = +x`, `n = +y`. So a drag straight
/// up is pure standoff and a drag sideways is pure text slide — the two
/// components do not contaminate each other.
#[test]
fn a_delta_splits_into_standoff_and_slide_along_the_axis() {
    let (_, offset, along) = placed(&horizontal(), 0.0, 30.0).expect("linear places");
    assert!((offset - 30.0).abs() < 1e-9, "straight up is all standoff");
    assert!(along.abs() < 1e-9, "and none of it slides the text");

    let (_, offset, along) = placed(&horizontal(), 25.0, 0.0).expect("linear places");
    assert!(offset.abs() < 1e-9, "sideways changes no standoff");
    assert!(
        (along - 25.0).abs() < 1e-9,
        "and slides the text by the drag"
    );
}

/// The property the whole design rests on: **placement never touches what
/// is measured.** Whatever the drag, `a` and `b` come out unchanged, so the
/// printed number cannot move.
#[test]
fn no_drag_can_move_the_measured_points() {
    let before = horizontal();
    for (dx, dy) in [(0.0, 0.0), (500.0, -900.0), (-12.5, 7.25), (1e6, 1e6)] {
        let (after, _, _) = placed(&before, dx, dy).expect("linear places");
        let (DimensionKind::Linear { a, b, .. }, DimensionKind::Linear { a: a0, b: b0, .. }) =
            (&after, &before)
        else {
            panic!("both are linear");
        };
        assert!(
            (a.x - a0.x).abs() < 1e-9 && (a.y - a0.y).abs() < 1e-9,
            "point a moved on a drag of {dx},{dy}"
        );
        assert!(
            (b.x - b0.x).abs() < 1e-9 && (b.y - b0.y).abs() < 1e-9,
            "point b moved on a drag of {dx},{dy}"
        );
    }
}

/// Placement accumulates: two drags of ten leave the dimension where one
/// drag of twenty would. The delta form is what makes this true — an
/// absolute `placement_from_point` would put it wherever the pointer last
/// was, which is a different gesture.
#[test]
fn two_drags_compose_into_one() {
    let (once, _, _) = placed(&horizontal(), 0.0, 10.0).expect("places");
    let (twice, offset, _) = placed(&once, 0.0, 10.0).expect("places");
    let (_, direct, _) = placed(&horizontal(), 0.0, 20.0).expect("places");
    assert!((offset - direct).abs() < 1e-9);
    let DimensionKind::Linear { offset: o, .. } = twice else {
        panic!("linear")
    };
    assert!((o - 20.0).abs() < 1e-9);
}

/// An aligned dimension whose two picks coincide has no axis, so there is
/// nothing to resolve a delta against. Refused rather than fabricated — see
/// `axis_frame`, which makes the same call one level down.
#[test]
fn a_degenerate_dimension_has_no_frame_and_is_refused() {
    let degenerate = DimensionKind::Linear {
        a: Point::new(50.0, 50.0),
        b: Point::new(50.0, 50.0),
        constraint: AxisConstraint::Aligned,
        offset: 0.0,
        text_along: 0.0,
        extension_gap: [None; 2],
    };
    assert!(placed(&degenerate, 5.0, 5.0).is_none());
}
fn square_perimeter() -> DimensionKind {
    DimensionKind::Perimeter {
        points: vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ],
        closed: true,
        area: false,
        offset: 0.0,
        text_along: 0.0,
    }
}

/// **A perimeter's label goes where it is dropped**, in both axes.
#[test]
fn a_perimeter_label_takes_the_delta_in_both_axes() {
    let (_, offset, along) = placed(&square_perimeter(), 25.0, -40.0).expect("places");
    assert!((along - 25.0).abs() < 1e-9, "x goes to text_along");
    assert!((offset + 40.0).abs() < 1e-9, "y goes to offset");
}

/// And the same guarantee the linear case has, which is the whole reason
/// this gesture is safe to be the default: **the measured shape is never
/// touched**, so the number cannot change no matter where the label lands.
#[test]
fn no_drag_moves_a_perimeter_vertex() {
    let before = square_perimeter();
    let (after, _, _) = placed(&before, 900.0, -700.0).expect("places");
    let (
        DimensionKind::Perimeter { points, closed, .. },
        DimensionKind::Perimeter {
            points: p0,
            closed: c0,
            ..
        },
    ) = (&after, &before)
    else {
        panic!("both are perimeters");
    };
    assert_eq!(points.len(), p0.len());
    for (a, b) in points.iter().zip(p0) {
        assert!((a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9);
    }
    assert_eq!(closed, c0, "and the ring is still a ring");
}

/// A real document with one perimeter ce dimension authored into it.
fn authored(
    points: Vec<Point>,
    closed: bool,
) -> (crate::app::state::OpenDoc, DimensionId, Vec<Point>) {
    let mut doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
    let session = std::sync::Arc::get_mut(&mut doc.session).expect("the fixture is sole owner");
    let group = session
        .add_dimension_group("corners", pdfcer_core::dimension::Unit::Millimeter)
        .expect("a group is created");
    let (_, id) = session
        .add_dimension(
            0,
            group,
            DimensionKind::Perimeter {
                points: points.clone(),
                closed,
                area: false,
                offset: 0.0,
                text_along: 0.0,
            },
        )
        .expect("the ce dimension is authored");
    (doc, id, points)
}

/// Drive one frame of a count edit and report what it drew and what it raised.
fn count(
    doc: &crate::app::state::OpenDoc,
    id: DimensionId,
    points: &[Point],
    closed: bool,
    intent: VertexIntent,
    index: usize,
    phase: Phase,
) -> (Option<Vec<(Point, Point)>>, Vec<Action>) {
    let mut actions = Vec::new();
    let drag = count_edit(CountEdit {
        id,
        intent,
        index,
        target: Point::new(150.0, 150.0),
        points,
        closed,
        phase,
        session: &doc.session,
        snap: None,
        actions: &mut actions,
    });
    (drag.and_then(|d| d.segments), actions)
}

/// A closed triangle, the fewest corners a ring may have.
fn closed_triangle() -> (crate::app::state::OpenDoc, DimensionId, Vec<Point>) {
    authored(
        vec![
            Point::new(100.0, 100.0),
            Point::new(300.0, 100.0),
            Point::new(200.0, 260.0),
        ],
        true,
    )
}

/// The same three corners, **open** — one above the minimum of two.
fn open_three() -> (crate::app::state::OpenDoc, DimensionId, Vec<Point>) {
    authored(
        vec![
            Point::new(100.0, 100.0),
            Point::new(300.0, 100.0),
            Point::new(200.0, 260.0),
        ],
        false,
    )
}

/// **The boundary, and the whole reason this pair of tests exists.**
#[test]
fn a_closed_triangle_refuses_to_lose_a_corner_and_says_why() {
    let (doc, id, points) = closed_triangle();
    let (_, actions) = count(
        &doc,
        id,
        &points,
        true,
        VertexIntent::Remove,
        1,
        Phase::Complete,
    );
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, Action::Dimension(DimensionAction::RemoveVertex { .. }))),
        "a removal that would leave a two-point ring reached the engine: {actions:?}"
    );
    assert!(
        actions.iter().any(|a| matches!(
            a,
            Action::Dimension(DimensionAction::DeclineVertexEdit {
                why: crate::text::measure::VertexEditRefusal::WouldLeaveTooFew
            })
        )),
        "the refusal was silent, which is the defect this whole surface answers: {actions:?}"
    );
}

/// …and the preview does not lie about it either.
#[test]
fn a_refused_removal_previews_the_shape_that_is_already_there() {
    let (doc, id, points) = closed_triangle();
    let (segments, actions) = count(
        &doc,
        id,
        &points,
        true,
        VertexIntent::Remove,
        1,
        Phase::InFlight,
    );
    let segments = segments.expect("a frame in flight draws something");
    assert_eq!(
        segments.len(),
        3,
        "a closed triangle is three segments and the preview showed {}",
        segments.len()
    );
    assert!(
        actions.is_empty(),
        "a frame in flight committed something: {actions:?}"
    );
}

/// **The same three corners, open, and the answer is the opposite one.**
#[test]
fn an_open_three_point_path_may_lose_a_corner_and_becomes_a_line() {
    let (doc, id, points) = open_three();
    let (segments, _) = count(
        &doc,
        id,
        &points,
        false,
        VertexIntent::Remove,
        1,
        Phase::InFlight,
    );
    let segments = segments.expect("a frame in flight draws something");
    assert_eq!(
        segments.len(),
        1,
        "two corners are one segment; the preview drew {}",
        segments.len()
    );
    let (a, b) = segments[0];
    assert!(
        (a.x - 100.0).abs() < 1e-9 && (b.x - 200.0).abs() < 1e-9,
        "the MIDDLE corner should be the one that went: {a:?} -> {b:?}"
    );

    let (_, actions) = count(
        &doc,
        id,
        &points,
        false,
        VertexIntent::Remove,
        1,
        Phase::Complete,
    );
    assert!(
        actions.iter().any(|a| matches!(
            a,
            Action::Dimension(DimensionAction::RemoveVertex { index: 1, .. })
        )),
        "the release reached no removal verb: {actions:?}"
    );
}

/// **A corner is added AFTER the one grabbed**, which is what makes the
/// gesture mean "put a point on this segment" rather than "put a point
/// somewhere on this shape".
#[test]
fn a_corner_is_added_after_the_one_that_was_grabbed() {
    let (doc, id, points) = closed_triangle();
    let (segments, _) = count(
        &doc,
        id,
        &points,
        true,
        VertexIntent::Insert,
        1,
        Phase::InFlight,
    );
    let segments = segments.expect("a frame in flight draws something");
    assert_eq!(
        segments.len(),
        4,
        "four corners closed is four segments; the preview drew {}",
        segments.len()
    );
    // Segment 1 runs from the grabbed corner to the new one.
    let (from, to) = segments[1];
    assert!(
        (from.x - 300.0).abs() < 1e-9 && (from.y - 100.0).abs() < 1e-9,
        "the new corner does not follow corner 1: {from:?}"
    );
    assert!(
        (to.x - 150.0).abs() < 1e-9 && (to.y - 150.0).abs() < 1e-9,
        "the new corner is not where the pointer was: {to:?}"
    );

    let (_, actions) = count(
        &doc,
        id,
        &points,
        true,
        VertexIntent::Insert,
        1,
        Phase::Complete,
    );
    assert!(
        actions.iter().any(|a| matches!(
            a,
            Action::Dimension(DimensionAction::InsertVertex { after: 1, .. })
        )),
        "the release reached no insert verb, or named the wrong segment: {actions:?}"
    );
}

/// **`after == len - 1` is the CLOSING segment, not an out-of-range index.**
#[test]
fn the_closing_segment_can_take_a_corner_too() {
    let (doc, id, points) = closed_triangle();
    let (segments, actions) = count(
        &doc,
        id,
        &points,
        true,
        VertexIntent::Insert,
        2,
        Phase::Complete,
    );
    assert!(segments.is_none(), "the committing frame previews nothing");
    assert!(
        actions.iter().any(|a| matches!(
            a,
            Action::Dimension(DimensionAction::InsertVertex { after: 2, .. })
        )),
        "a point on the closing segment was refused or renamed: {actions:?}"
    );
}

// ===========================================================================
// Who is allowed to change how many corners there are
// ===========================================================================

/// One frame's [`intent`] for a given armed tool and modifier set.
fn intent_with(tool: CanvasTool, modifiers: egui::Modifiers) -> VertexIntent {
    let ctx = egui::Context::default();
    crate::canvas::tool::select(&ctx, tool);
    let mut seen = VertexIntent::Move;
    let _ = ctx.run_ui(
        egui::RawInput {
            modifiers,
            ..Default::default()
        },
        |ui| seen = intent(ui.ctx()),
    );
    seen
}

const CTRL: egui::Modifiers = egui::Modifiers {
    command: true,
    ctrl: true,
    alt: false,
    shift: false,
    mac_cmd: false,
};

const CTRL_SHIFT: egui::Modifiers = egui::Modifiers {
    command: true,
    ctrl: true,
    alt: false,
    shift: true,
    mac_cmd: false,
};

/// **The safety, and the assertion worth more than the two below it.**
#[test]
fn ctrl_alone_cannot_change_how_many_corners_a_shape_has() {
    assert_eq!(intent_with(CanvasTool::Select, CTRL), VertexIntent::Move);
    assert_eq!(
        intent_with(CanvasTool::Select, CTRL_SHIFT),
        VertexIntent::Move
    );
    assert_eq!(intent_with(CanvasTool::Hand, CTRL), VertexIntent::Move);
}

/// With the Points tool armed, the two chords mean what the tool's own sentence
/// says they mean — and an unmodified drag still moves the corner.
#[test]
fn the_points_tool_gives_ctrl_and_ctrl_shift_their_meanings() {
    assert_eq!(
        intent_with(CanvasTool::Node, egui::Modifiers::NONE),
        VertexIntent::Move,
        "an unmodified drag must still reshape"
    );
    assert_eq!(intent_with(CanvasTool::Node, CTRL), VertexIntent::Insert);
    assert_eq!(
        intent_with(CanvasTool::Node, CTRL_SHIFT),
        VertexIntent::Remove
    );
}

/// **The mode gate, entered EXPLICITLY.**
#[test]
fn the_points_tool_survives_review_and_still_retires_in_read() {
    use crate::app::modes::capability::Capabilities;
    let review = Capabilities {
        edit_content: false,
        author_markup: true,
        author_measure: true,
    };
    for (caps, retired, mode) in [
        (Capabilities::FULL, false, "Edit"),
        (review, false, "Review"),
        (Capabilities::NONE, true, "Read"),
    ] {
        let ctx = egui::Context::default();
        crate::canvas::tool::store_capabilities(&ctx, caps);
        crate::canvas::tool::select(&ctx, CanvasTool::Node);
        assert_eq!(
            crate::canvas::tool::retire_forbidden(&ctx, caps),
            retired,
            "the Points tool was handled wrongly in {mode}"
        );
        assert_eq!(
            crate::canvas::tool::selected(&ctx).is_node(),
            !retired,
            "the armed tool after the mode change is wrong in {mode}"
        );
    }
}

/// **A circular ce dimension's label lands where it is dropped**: the engine's
/// polar placement round-trips, so the label anchor after the move is the drop
/// point, and the leader turns to face it.
#[test]
fn a_circular_label_drag_lands_where_it_is_dropped() {
    use pdfcer_core::dimension::FitCircle;
    use pdfcer_core::vector::Point;
    let fit = FitCircle {
        center: Point::new(100.0, 100.0),
        radius: 20.0,
        residual: 0.0,
    };
    let kind = DimensionKind::Circular {
        fit,
        show_diameter: false,
        leader_angle: 0.0,
        text_distance: None,
    };
    let from = kind.label_anchor().expect("an anchor");
    let (dx, dy) = (0.0 - (from.x - 100.0), 40.0 - (from.y - 100.0));
    let (moved, _, angle) = placed(&kind, dx, dy).expect("a circular drag is admitted");
    let to = moved.label_anchor().expect("an anchor after the move");
    assert!(
        (to.x - (from.x + dx)).abs() < 1e-6 && (to.y - (from.y + dy)).abs() < 1e-6,
        "dropped at ({}, {}), landed at {to:?}",
        from.x + dx,
        from.y + dy
    );
    assert!(
        (angle - 90.0).abs() < 1e-6,
        "the leader faces the drop: {angle}"
    );
}
