//! # `canvas::annotnodes` tests — the shell's half, and the engine's ruling
//!
//! ## ★★★ What these can and cannot prove, stated first
//!
//! **They cannot prove the operator can edit a node.** Every test here calls a
//! function directly, and the whole point of R1 is that a passing unit test is
//! not a report of working software: this project once had eight green tests
//! while the feature performed 1 of 14 steps. Six process boundaries stand
//! between [`super::resolved`] and a hand on a mouse — the tool arming, the
//! chord reaching dispatch, the modifier surviving winit, the press
//! classifying, the anchor being painted where the hit test looks, and the
//! engine accepting the write — and **not one of them is observable in
//! process**. `tools/ui-verify`'s `a_markup_shapes_nodes_can_be_edited` is the
//! instrument for that, and its own header enumerates the six.
//!
//! What these DO prove, and what makes them worth the second they cost:
//!
//! 1. **The shell's subtype table is right** — which shapes show anchors. That
//!    is a painting decision this shell owns, so nothing else can check it.
//! 2. **The engine's ruling is asked rather than restated.** Every count test
//!    below authors a real annotation into a real fixture and drives the real
//!    `reshape_annotation_preview`. A test that faked the annotation would get
//!    `AnnotationNotFound` for every case *while looking exactly like a test
//!    that passed for the right reason* — `dimdrag::tests` records that trap in
//!    the same words.
//! 3. **A refusal is a sentence.** Half of each refusal test asserts the
//!    decline was raised, because a build that merely dropped the gesture would
//!    pass the other half and would be the operator's original complaint.
//!
//! ## ★★ The shapes are chosen so the boundary is exercised
//!
//! A test on a five-sided polygon proves nothing about the floor. The shapes
//! here are:
//!
//! | shape | nodes | floor | what removing one does |
//! |---|---|---|---|
//! | `/Polygon` triangle | 3 | 3 | **refuses** — at the floor |
//! | `/Polygon` square | 4 | 3 | succeeds, leaving a triangle |
//! | `/PolyLine`, three points | 3 | 2 | succeeds, leaving a straight line |
//! | `/Line` | 2 | — | **refuses by name** — a Line is two ends by definition |
//!
//! The triangle and the three-point polyline are the same three points and give
//! **opposite** answers, which is what proves the refusal measures the shape's
//! own floor rather than blanket-refusing three-node shapes.

// ★★ The INNER attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and without it every assertion message here
// is reported as operator-facing copy.
#![cfg(test)]

use super::*;
use crate::canvas::selection::{AnnotSelection, AnnotTarget};
use pdfcer_core::annot_author::{Color, LineEnding, MarkupSpec};

/// A real document with one markup annotation authored into it, and a selection
/// naming it.
///
/// ★★ **Through `add_markup`, not through a hand-built dictionary.** The
/// preflight reads the annotation out of the session's own graph, so a fixture
/// that faked one would be asking the engine about a shape that does not exist.
fn authored(spec: &MarkupSpec) -> (crate::app::state::OpenDoc, ObjId) {
    let mut doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
    let session = std::sync::Arc::get_mut(&mut doc.session).expect("the fixture is sole owner");
    let id = session.add_markup(0, spec).expect("the markup is authored");
    (doc, id)
}

/// Select the annotation `authored` made, as the canvas would.
///
/// The outline is arbitrary — nothing here reads it — and the `subtype` and
/// `locked` fields are not, because [`super::geometry`] reads both.
fn select(doc: &crate::app::state::OpenDoc, id: ObjId, locked: bool) -> SelectionState {
    let subtype = pdfcer_core::annot::page_annotations(&doc.session.graph(), doc.pages[0].id)
        .into_iter()
        .find(|a| a.id == Some(id))
        .map(|a| String::from_utf8_lossy(&a.subtype).into_owned())
        .expect("the annotation is on page 1");
    let mut state = SelectionState::default();
    state.select_annot(AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id,
            kind: AnnotKind::Markup,
            subtype,
            locked,
        },
        outline: egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(40.0, 30.0)),
    });
    state
}

fn polygon(vertices: Vec<(f64, f64)>) -> MarkupSpec {
    MarkupSpec::Polygon {
        vertices,
        border: Some(Color::Rgb(1.0, 0.0, 0.0)),
        interior: None,
        width: 1.0,
    }
}

fn polyline(vertices: Vec<(f64, f64)>) -> MarkupSpec {
    MarkupSpec::PolyLine {
        vertices,
        color: Color::Rgb(1.0, 0.0, 0.0),
        width: 1.0,
    }
}

fn triangle() -> Vec<(f64, f64)> {
    vec![(100.0, 100.0), (300.0, 100.0), (200.0, 260.0)]
}

fn square() -> Vec<(f64, f64)> {
    vec![
        (100.0, 100.0),
        (300.0, 100.0),
        (300.0, 300.0),
        (100.0, 300.0),
    ]
}

/// Drive one frame of a node edit and report what it drew and what it raised.
fn run(
    doc: &crate::app::state::OpenDoc,
    id: ObjId,
    points: &[Point],
    closed: bool,
    intent: VertexIntent,
    index: usize,
    phase: Phase,
) -> (Option<Vec<(Point, Point)>>, Vec<Action>) {
    let mut actions = Vec::new();
    let drag = resolved(Resolve {
        session: &doc.session,
        id,
        points,
        closed,
        intent,
        index,
        old: points[index],
        target: Point::new(150.0, 150.0),
        phase,
        snap: None,
        actions: &mut actions,
    });
    (drag.segments, actions)
}

fn points_of(pairs: &[(f64, f64)]) -> Vec<Point> {
    pairs.iter().map(|&(x, y)| Point::new(x, y)).collect()
}

// ===========================================================================
// 1. Which shapes show anchors — the one table this shell owns
// ===========================================================================

/// ★★★ **A `/Polygon` closes, a `/PolyLine` does not, and a `/Line` has two
/// ends.**
///
/// The `closed` flag is what decides whether the preview draws the segment back
/// to the first node, so getting it backwards would draw an open triangle over
/// a closed one — a picture that is wrong in a way an operator would report as
/// *"it looks like it lost a side"* and that no count assertion would catch.
#[test]
fn the_three_shapes_with_nodes_report_their_geometry() {
    let (doc, id) = authored(&polygon(triangle()));
    let (_, points, closed) =
        geometry(&doc, &select(&doc, id, false)).expect("a polygon has nodes");
    assert_eq!(points.len(), 3);
    assert!(closed, "a /Polygon closes back to its first vertex");

    let (doc, id) = authored(&polyline(triangle()));
    let (_, points, closed) =
        geometry(&doc, &select(&doc, id, false)).expect("a polyline has nodes");
    assert_eq!(points.len(), 3);
    assert!(!closed, "a /PolyLine is an open path");

    let (doc, id) = authored(&MarkupSpec::Line {
        start: (10.0, 20.0),
        end: (110.0, 220.0),
        color: Color::Rgb(0.0, 0.0, 1.0),
        width: 1.0,
        endings: (LineEnding::None, LineEnding::None),
    });
    let (_, points, closed) =
        geometry(&doc, &select(&doc, id, false)).expect("a line has two ends");
    assert!(!closed);
    assert_eq!(
        points.len(),
        2,
        "a /Line's ends come from /L, not /Vertices"
    );
    // ★ Index 0 is the START and index 1 is the END, which is exactly how the
    // engine addresses them. A shell that read them the other way round would
    // move the wrong end and look, on screen, like a working gesture.
    assert!((points[0].x - 10.0).abs() < 1e-6, "{:?}", points[0]);
    assert!((points[1].x - 110.0).abs() < 1e-6, "{:?}", points[1]);
}

/// ★★★ **R9: a shape with no editable nodes draws NOTHING** — not a greyed
/// anchor, not a ghost anchor.
///
/// The `/Ink` row is the one that matters, and it is the one a plausible
/// implementation gets wrong. `Annotation::ink_list` is **readable**, so a
/// shell that derived *"draggable"* from *"readable"* would put an anchor on
/// every point of every pen stroke and then refuse every drag from them — the
/// "visible control, silently inert" failure, at the density of a freehand
/// scribble.
#[test]
fn a_shape_with_no_editable_nodes_shows_no_anchors() {
    let rect = pdfcer_core::page_tree::Rect {
        llx: 100.0,
        lly: 100.0,
        urx: 200.0,
        ury: 200.0,
    };
    for spec in [
        MarkupSpec::Square {
            rect,
            border: Some(Color::Rgb(1.0, 0.0, 0.0)),
            interior: None,
            border_width: 1.0,
            border_effect: None,
        },
        MarkupSpec::Circle {
            rect,
            border: Some(Color::Rgb(1.0, 0.0, 0.0)),
            interior: None,
            border_width: 1.0,
        },
        MarkupSpec::Ink {
            strokes: vec![vec![(10.0, 10.0), (20.0, 30.0), (40.0, 25.0)]],
            color: Color::Rgb(1.0, 0.0, 0.0),
            width: 1.0,
        },
    ] {
        let (doc, id) = authored(&spec);
        let selection = select(&doc, id, false);
        assert!(
            geometry(&doc, &selection).is_none(),
            "this shape reported nodes it cannot edit: {spec:?}"
        );
        assert!(
            nodes(&doc, &selection).is_empty(),
            "and the painter would have drawn some: {spec:?}"
        );
    }
}

/// ★★ **A locked annotation offers no anchors either**, and the refusal is
/// honoured here rather than left to the engine.
///
/// §12.5.3 Table 165 bit 8 is the *file* saying the user interface may not
/// change this. An anchor drawn on it would be a promise the release cannot
/// keep — `annotdrag::eligible` states the identical rule for the whole-shape
/// drag, and this is that rule applied one level down.
#[test]
fn a_locked_shape_offers_no_anchors() {
    let (doc, id) = authored(&polygon(triangle()));
    assert!(geometry(&doc, &select(&doc, id, true)).is_none());
}

/// ★ **A ce dimension is not this module's business.**
///
/// The load-bearing half. A ce dimension is a `/Line` with
/// `/IT /LineDimension`, so it passes every *"is this markup?"* test; reshaping
/// one through `reshape_annotation` would move the drawn line and leave the
/// sidecar record — and therefore the printed number — describing geometry that
/// is no longer there. The engine refuses it by name as the backstop; this is
/// what stops the backstop being reached.
#[test]
fn a_ce_dimension_is_dimdrags_and_not_this_modules() {
    let (doc, id) = authored(&polygon(triangle()));
    let mut state = select(&doc, id, false);
    let mut annot = state.annot().expect("selected").clone();
    annot.target.kind = AnnotKind::CeDimension;
    state.select_annot(annot);
    assert!(geometry(&doc, &state).is_none());
}

// ===========================================================================
// 2. The preview is one derivation with the commit
// ===========================================================================

/// **A closed shape's preview carries the closing segment; an open one's does
/// not.**
///
/// Four nodes closed is four segments; four nodes open is three. Getting this
/// wrong draws a shape the release does not commit, which is the one thing the
/// honesty contract in this module's header forbids.
#[test]
fn a_closed_shape_previews_its_closing_segment() {
    let pts = points_of(&square());
    assert_eq!(preview_of(&pts, true).len(), 4);
    assert_eq!(preview_of(&pts, false).len(), 3);
}

/// ★★ **A node moves to where the pointer resolved, and its neighbours do
/// not.**
///
/// Asserted on the resulting geometry rather than on the action's `dx`/`dy`
/// alone, because a build that raised the right delta and previewed the wrong
/// node would pass an argument check and show the operator the wrong picture.
#[test]
fn moving_a_node_moves_that_node_and_no_other() {
    let (doc, id) = authored(&polygon(square()));
    let pts = points_of(&square());
    let (segments, _) = run(&doc, id, &pts, true, VertexIntent::Move, 1, Phase::InFlight);
    let segments = segments.expect("a frame in flight draws something");
    assert_eq!(segments.len(), 4, "a closed square is four segments");
    // Segment 0 runs node 0 -> node 1, so its far end is the moved node and its
    // near end is the untouched neighbour.
    let (a, b) = segments[0];
    assert!(
        (a.x - 100.0).abs() < 1e-9 && (a.y - 100.0).abs() < 1e-9,
        "{a:?}"
    );
    assert!(
        (b.x - 150.0).abs() < 1e-9 && (b.y - 150.0).abs() < 1e-9,
        "{b:?}"
    );
}

/// **A frame in flight raises nothing, and the release raises exactly one
/// action.**
///
/// `drag-moves` D4: one gesture is one `Ctrl+Z`. A build that raised per frame
/// would fill the undo stack with sixty entries a second and would look
/// perfectly correct on screen.
#[test]
fn only_the_release_raises_an_action_and_it_raises_one() {
    let (doc, id) = authored(&polygon(square()));
    let pts = points_of(&square());
    let (_, in_flight) = run(&doc, id, &pts, true, VertexIntent::Move, 1, Phase::InFlight);
    assert!(in_flight.is_empty(), "{in_flight:?}");

    let (segments, released) = run(&doc, id, &pts, true, VertexIntent::Move, 1, Phase::Complete);
    assert_eq!(released.len(), 1, "{released:?}");
    assert!(
        matches!(
            released[0],
            Action::Annot(AnnotAction::MoveNode { index: 1, .. })
        ),
        "{released:?}"
    );
    assert!(
        segments.is_none(),
        "the committing frame drew a preview over the shape it is about to redraw"
    );
}

/// ★ **A move sends a DELTA measured from the node, not from the pointer.**
///
/// The node at `(300, 100)` dragged to `(150, 150)` is `dx = -150, dy = +50`.
/// A build that sent the absolute target would move the shape by the target's
/// distance from the origin — a very large jump, and one that looks like a
/// coordinate-space bug rather than an arithmetic one.
#[test]
fn a_move_sends_the_displacement_of_the_node() {
    let (doc, id) = authored(&polygon(square()));
    let pts = points_of(&square());
    let (_, actions) = run(&doc, id, &pts, true, VertexIntent::Move, 1, Phase::Complete);
    let Some(Action::Annot(AnnotAction::MoveNode { dx, dy, .. })) = actions.first() else {
        panic!("no move was raised: {actions:?}");
    };
    assert!((dx + 150.0).abs() < 1e-9, "dx was {dx}");
    assert!((dy - 50.0).abs() < 1e-9, "dy was {dy}");
}

// ===========================================================================
// 3. The engine's ruling, asked rather than restated
// ===========================================================================

/// ★★★ **A three-node polygon refuses to lose one, and SAYS SO.**
///
/// The floor is the engine's (`/Polygon` keeps three) and this test drives the
/// real `reshape_annotation_preview` against a real annotation, so it fails the
/// day the engine's ruling changes — which is the property that makes it worth
/// more than an assertion about a constant in this crate.
///
/// Both halves are asserted deliberately. A build that simply dropped the
/// gesture would pass the first and fail the second, and **it is the second
/// that is the operator's actual complaint**: a node drag that is refused with
/// nothing said anywhere is the founding defect of this project.
#[test]
fn a_triangle_refuses_to_lose_a_node_and_says_why() {
    let (doc, id) = authored(&polygon(triangle()));
    let pts = points_of(&triangle());
    let (_, actions) = run(
        &doc,
        id,
        &pts,
        true,
        VertexIntent::Remove,
        1,
        Phase::Complete,
    );
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, Action::Annot(AnnotAction::RemoveNode { .. }))),
        "a removal below the floor reached the engine: {actions:?}"
    );
    assert!(
        actions.iter().any(|a| matches!(
            a,
            Action::Annot(AnnotAction::DeclineNodeEdit {
                why: crate::text::markup::NodeEditRefusal::WouldLeaveTooFew
            })
        )),
        "the refusal was silent, which is the defect this surface answers: {actions:?}"
    );
}

/// …and the preview does not lie about it either.
///
/// The frame before the release draws the shape it would commit, and here that
/// is the shape already on the page. A build that drew the node vanishing and
/// then refused would be showing an edit that never happens — which looks like
/// it worked until the next repaint.
#[test]
fn a_refused_removal_previews_the_shape_that_is_already_there() {
    let (doc, id) = authored(&polygon(triangle()));
    let pts = points_of(&triangle());
    let (segments, actions) = run(
        &doc,
        id,
        &pts,
        true,
        VertexIntent::Remove,
        1,
        Phase::InFlight,
    );
    assert_eq!(
        segments.expect("a frame in flight draws something").len(),
        3,
        "the preview showed a shape with a node missing"
    );
    assert!(actions.is_empty(), "{actions:?}");
}

/// ★★★ **The same three points, OPEN, and the answer is the opposite one.**
///
/// A `/PolyLine` keeps two, so this removal is legal and what it leaves is a
/// straight line: one segment, from the first node to the last. This is what
/// proves the test above measures the shape's own floor rather than a blanket
/// refusal on three-node shapes.
#[test]
fn an_open_three_point_path_may_lose_a_node_and_becomes_a_line() {
    let (doc, id) = authored(&polyline(triangle()));
    let pts = points_of(&triangle());
    let (segments, _) = run(
        &doc,
        id,
        &pts,
        false,
        VertexIntent::Remove,
        1,
        Phase::InFlight,
    );
    let segments = segments.expect("a frame in flight draws something");
    assert_eq!(segments.len(), 1, "two nodes are one segment");
    let (a, b) = segments[0];
    assert!(
        (a.x - 100.0).abs() < 1e-9 && (b.x - 200.0).abs() < 1e-9,
        "the MIDDLE node should be the one that went: {a:?} -> {b:?}"
    );

    let (_, actions) = run(
        &doc,
        id,
        &pts,
        false,
        VertexIntent::Remove,
        1,
        Phase::Complete,
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, Action::Annot(AnnotAction::RemoveNode { index: 1, .. }))),
        "the release reached no removal verb: {actions:?}"
    );
}

/// ★★ **A node is added AFTER the one grabbed**, which is what makes the
/// gesture mean *"put a point on this segment"* rather than *"put a point
/// somewhere on this shape"*.
///
/// Asserted on the geometry rather than on the action's `after` field alone,
/// because a build that raised `after: 1` and previewed the point at index 0
/// would pass an argument check and show the operator the wrong thing.
#[test]
fn a_node_is_added_after_the_one_that_was_grabbed() {
    let (doc, id) = authored(&polygon(square()));
    let pts = points_of(&square());
    let (segments, actions) = run(
        &doc,
        id,
        &pts,
        true,
        VertexIntent::Insert,
        1,
        Phase::InFlight,
    );
    let segments = segments.expect("a frame in flight draws something");
    assert_eq!(segments.len(), 5, "four nodes plus one, closed");
    // Segment 1 runs the grabbed node -> the new one.
    let (a, b) = segments[1];
    assert!(
        (a.x - 300.0).abs() < 1e-9 && (a.y - 100.0).abs() < 1e-9,
        "{a:?}"
    );
    assert!(
        (b.x - 150.0).abs() < 1e-9 && (b.y - 150.0).abs() < 1e-9,
        "{b:?}"
    );
    assert!(actions.is_empty());

    let (_, actions) = run(
        &doc,
        id,
        &pts,
        true,
        VertexIntent::Insert,
        1,
        Phase::Complete,
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, Action::Annot(AnnotAction::InsertNode { after: 1, .. }))),
        "{actions:?}"
    );
}

/// ★★★ **A `/Line` moves either end and cannot gain or lose one — and the
/// refusal NAMES the shape.**
///
/// This is the brief's own case: *a refusal that names a shape type is a real
/// refusal and should be shown as a sentence, not a grey anchor.* The anchors
/// are drawn (a Line's two ends are draggable), the count edit is refused, and
/// what the operator gets is a sentence about lines rather than silence.
#[test]
fn a_line_moves_its_ends_and_refuses_to_gain_one_by_name() {
    let (doc, id) = authored(&MarkupSpec::Line {
        start: (100.0, 100.0),
        end: (300.0, 100.0),
        color: Color::Rgb(0.0, 0.0, 1.0),
        width: 1.0,
        endings: (LineEnding::None, LineEnding::None),
    });
    let pts = points_of(&[(100.0, 100.0), (300.0, 100.0)]);

    // Moving the second end is allowed and reaches the verb.
    let (_, actions) = run(
        &doc,
        id,
        &pts,
        false,
        VertexIntent::Move,
        1,
        Phase::Complete,
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, Action::Annot(AnnotAction::MoveNode { index: 1, .. }))),
        "a /Line's end could not be moved: {actions:?}"
    );

    // Adding one is refused, by name, with a sentence about lines.
    let (segments, actions) = run(
        &doc,
        id,
        &pts,
        false,
        VertexIntent::Insert,
        0,
        Phase::Complete,
    );
    assert!(segments.is_none());
    assert!(
        actions.iter().any(|a| matches!(
            a,
            Action::Annot(AnnotAction::DeclineNodeEdit {
                why: crate::text::markup::NodeEditRefusal::ShapeHasNoNodes {
                    subtype: crate::text::markup::ShapeWord::Line
                }
            })
        )),
        "the refusal did not name the shape: {actions:?}"
    );
}

// ===========================================================================
// 4. The wording, and the one coupling this feature depends on
// ===========================================================================

/// ★★ **Every refusal sentence is about a shape, never about a measurement.**
///
/// R8b rule 15. `crate::text::measure::VertexEditRefusal` says *"measurement"*
/// because its subject is a **ce dimension**; this enum's subject is a comment
/// somebody drew, and reusing those words would tell an operator their polygon
/// was measuring something. This is the assertion that stops a future tidy-up
/// merging the two enums.
#[test]
fn no_node_refusal_calls_a_markup_shape_a_measurement() {
    use crate::text::markup::{NodeEditRefusal as R, ShapeWord as W};
    let all = [
        R::WouldLeaveTooFew,
        R::Unplaceable,
        R::Locked,
        R::Refused,
        R::ShapeHasNoNodes { subtype: W::Ink },
        R::ShapeHasNoNodes {
            subtype: W::Rectangle,
        },
        R::ShapeHasNoNodes {
            subtype: W::Ellipse,
        },
        R::ShapeHasNoNodes { subtype: W::Line },
        R::ShapeHasNoNodes {
            subtype: W::TextMarkup,
        },
        R::ShapeHasNoNodes { subtype: W::Other },
    ];
    for why in all {
        let line = why.line();
        assert!(!line.is_empty(), "{why:?} has no sentence");
        let lower = line.to_lowercase();
        assert!(!lower.contains("measurement"), "{why:?}: {line}");
        assert!(!lower.contains("dimension"), "{why:?}: {line}");
        // ★ And no engine vocabulary: a sentence naming a PDF key or an engine
        // verb is diagnostic prose that has escaped into the UI, which is
        // exactly what `check-ui-strings`' exclusion 3 exists to keep out.
        //
        // ★★ **"polyline" is deliberately NOT on this list**, and it was on it
        // for one run. It looks like a PDF name — `/PolyLine` is one — and it
        // is also the operator's own word: `markup.polyline`'s ribbon label is
        // literally "Polyline", so the Line sentence's *"draw a polyline for
        // that"* names a button they can see. ⇒ The test is for **file-format
        // and API vocabulary**, not for words that happen to appear in both
        // vocabularies. `/Vertices`, `/InkList` and `/QuadPoints` appear on no
        // control anywhere; a leading slash and a trailing `_annotation` are
        // the tells that matter.
        for forbidden in ["/vertices", "/inklist", "/l ", "quadpoints", "_annotation"] {
            assert!(
                !lower.contains(forbidden),
                "{why:?} leaks `{forbidden}`: {line}"
            );
        }
    }
}

/// ★ **The subtype word is the operator's, not the file's.**
///
/// `/Square` is what pdfcer's own Rectangle tool authors, and telling an
/// operator "Square" for the thing they drew with the Rectangle button is the
/// surface disagreeing with itself about what it just did.
#[test]
fn a_square_is_called_a_rectangle_and_a_circle_an_ellipse() {
    use crate::text::markup::ShapeWord as W;
    assert_eq!(shape_word("Square"), W::Rectangle);
    assert_eq!(shape_word("Circle"), W::Ellipse);
    assert_eq!(shape_word("Ink"), W::Ink);
    assert_eq!(shape_word("Highlight"), W::TextMarkup);
    assert_eq!(shape_word("Stamp"), W::Other);
    assert!(
        W::Rectangle
            .no_nodes_line()
            .to_lowercase()
            .contains("rectangle")
    );
    assert!(
        W::Ellipse
            .no_nodes_line()
            .to_lowercase()
            .contains("ellipse")
    );
}

/// ★★★ **THE COUPLING THIS FEATURE HANGS ON, asserted rather than trusted.**
///
/// The count gestures require the **Points tool** to be armed — `Ctrl` alone
/// with the Select tool still moves the node, which is the safety that stops a
/// mis-held modifier destroying a node during an ordinary nudge. That tool's
/// arming predicate is `edit_content || author_measure`
/// (`canvas::tool::arm::retire_forbidden`, and an identical copy in
/// `app::dispatch::navigate`), and **it does not name `author_markup`**.
///
/// Markup is authored in **Review**, so if Review ever lost `author_measure`
/// the Points tool would stop arming there and adding or removing a node of a
/// comment shape would become unreachable — silently, with the anchors still
/// drawn and still draggable for a plain move. This test is the tripwire for
/// that, and it is here rather than in a paragraph because a paragraph cannot
/// go red.
///
/// ★ Why the predicate was not simply widened: the two copies of it must stay
/// identical, and a disagreement shows as a tool that arms and is retired on
/// the next frame — a flicker with no sentence attached. One of the two copies
/// lives in `app::dispatch`, which this session did not own. Reported rather
/// than half-changed.
#[test]
fn the_points_tool_arms_wherever_a_markup_shape_can_be_authored() {
    let shell = crate::shell::manifest::built_in();
    for mode in shell.modes() {
        let caps = crate::app::modes::Capabilities::for_mode(Some(&shell), Some(&mode.id));
        if !caps.author_markup {
            continue;
        }
        let mode = &mode.id;
        assert!(
            caps.edit_content || caps.author_measure,
            "{mode:?} may author markup and cannot arm the Points tool, so adding or removing a \
             node of a comment shape is unreachable there. Widen BOTH copies of the predicate — \
             `canvas::tool::arm::retire_forbidden`'s Node arm AND \
             `app::dispatch::navigate`'s `view.tool_node` arm — or they will disagree and the \
             tool will arm and retire on consecutive frames."
        );
    }
}
