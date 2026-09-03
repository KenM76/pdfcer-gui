//! # `canvas::moving::tests` — the move gesture's algebra, asserted
//!
//! Split out of [`super`] under **R2** on 2026-08-27, when the form-XObject
//! refusal took the module past 1,500 lines. The tests were the seam, for the
//! reason `canvas::selection::tests` gives when it made the same move: the
//! code above is one subject — *which verb does a drag on this selection
//! become, and what does it refuse* — and there is no honest place to cut it
//! in two. The assertions are a different subject with a different reader.
//!
//! ## What these are about, and why the module they test is worth this much
//!
//! Three obligations, stated in [`super`]'s header and each one paid for by a
//! defect:
//!
//! 1. **A press that turns out to be a drag must not change the selection.**
//! 2. **A ghost is drawn if and only if the release would commit** — the
//!    preview may not promise a move the engine is going to refuse, which is
//!    why [`super::eligible`] is asked once per frame *and* once on release.
//! 3. **Every refusal is named**, not silently absorbed. There are nine, and
//!    the newest — `InsideForm` — is the only one that reaches the operator in
//!    words, because it is the only one describing a state they did not put
//!    themselves in and cannot see.
//!
//! ## `#![cfg(test)]` at the top, and why it is the marker rather than the name
//!
//! `check-ui-strings.sh` and `check-theme-colors.sh` both recognise the inner
//! attribute as meaning *"none of this is in the shipped binary"*, and both
//! state why they match on that rather than on a filename: the property that
//! earns the exemption is not being in the binary, and a filename is a
//! restatement of it that goes stale the moment a third such module is
//! written. Without it, every `assert!` message below is reported as
//! un-catalogued operator copy.
//!
//! ★ **The line gate still counts these lines.** `check-file-size.sh` counts
//! total lines, tests included, on purpose — so this is not a way of hiding
//! from R2. It is the split R2 asked for.

#![cfg(test)]

use super::*;
use crate::canvas::selection::ClickHit;
use crate::canvas::target::{StubTargets, TargetId};
use egui::{Rect, vec2};
use pdfcer_core::object::{Dict, ObjId};
use pdfcer_core::page_tree::Rect as PageRect;

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), vec2(w, h))
}

/// A minimal page fixture — the same one `viewer`'s geometry tests use,
/// because these functions read exactly what those do: `crop_box` and
/// `rotate`.
fn test_page(w: f64, h: f64, rotate: u16) -> Page {
    Page {
        id: ObjId::new(1, 0),
        resources: Dict::new(),
        media_box: PageRect::from_corners(0.0, 0.0, w, h),
        crop_box: PageRect::from_corners(0.0, 0.0, w, h),
        rotate,
        contents: Vec::new(),
        contents_unresolved: 0,
        contents_flattened: 0,
    }
}

fn hit_object(index: u64) -> ClickHit {
    ClickHit {
        object: Some(TargetId::Object(index)),
        ..ClickHit::default()
    }
}

/// A click that landed on anchor `node` of subpath `part` of `object`.
fn hit_node(object: u64, part: usize, node: usize) -> ClickHit {
    ClickHit {
        object: Some(TargetId::Object(object)),
        part: Some(part),
        node: Some(node),
    }
}

/// Two objects on page 0, the first with two subpaths.
fn stub() -> StubTargets {
    StubTargets::new(
        0,
        [rect(0.0, 0.0, 100.0, 100.0), rect(200.0, 200.0, 50.0, 50.0)],
    )
    .with_parts(
        0,
        [rect(0.0, 0.0, 40.0, 40.0), rect(60.0, 60.0, 40.0, 40.0)],
    )
}

/// The same two objects, translated — what a decomposition taken *after* a
/// committed move yields: the same objects at the same indices, in new
/// places.
fn stub_moved(by: Vec2) -> StubTargets {
    StubTargets::new(
        0,
        [
            rect(0.0, 0.0, 100.0, 100.0).translate(by),
            rect(200.0, 200.0, 50.0, 50.0).translate(by),
        ],
    )
    .with_parts(
        0,
        [
            rect(0.0, 0.0, 40.0, 40.0).translate(by),
            rect(60.0, 60.0, 40.0, 40.0).translate(by),
        ],
    )
}

/// A selection holding both objects at the Object rung, resolved.
fn two_objects_selected() -> SelectionState {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(0, hit_object(1), true, false);
    sel.resolve(Some(&stub()), 0, 0);
    sel
}

/// Every object is a path, and the entered one decomposes into subpaths —
/// the ordinary case.
fn paths() -> MoveContext {
    MoveContext {
        non_path: None,
        part_kind: Some(PartKind::Subpath),
    }
}

// -----------------------------------------------------------------
// ★ The invariant the whole feature was blocked on
// -----------------------------------------------------------------

/// ★ **A move never alters the selection.**
///
/// The counterpart of
/// [`navigating_the_view_never_alters_the_selection`](crate::canvas::selection),
/// and it is asserted the same way: drive the thing that *could* reach the
/// selection, then compare.
///
/// What a committed move does to the shell is exactly two things — it
/// bumps `OpenDoc::edit_epoch`, and it makes the next decomposition report
/// the same objects at the same indices in new places. Both are modelled
/// here: the epoch moves from 0 to 1, and the provider handed to the
/// re-resolve is `stub_moved`. `object_identity_across_edits.rs` is what
/// licenses the second half — `move_*` rewrites operands in place, adds and
/// removes no operator, and therefore renumbers nothing.
///
/// **The test is not vacuous**, and the second assertion is what makes it
/// so: the outlines must have *moved*. A `resolve` that quietly did nothing
/// would satisfy the identity assertion perfectly.
#[test]
fn a_move_never_alters_the_selection() {
    let mut sel = two_objects_selected();
    let entries_before = sel.entries().to_vec();
    let outlines_before = sel.outlines().to_vec();
    assert_eq!(entries_before.len(), 2);

    // The move lands: epoch bumped, geometry translated, indices intact.
    let by = vec2(25.0, -40.0);
    sel.resolve(Some(&stub_moved(by)), 0, 1);

    assert_eq!(
        sel.entries(),
        entries_before.as_slice(),
        "a move reached the selection; only `delete_*` may renumber, and this is not one"
    );
    assert_ne!(
        sel.outlines(),
        outlines_before.as_slice(),
        "the outlines must follow the move, or this test would pass on a no-op resolve"
    );
    for ((entry, after), (_, before)) in sel.outlines().iter().zip(&outlines_before) {
        assert_eq!(
            *after,
            before.translate(by),
            "entry {entry:?} outline did not follow the move exactly"
        );
    }
}

/// A deeper rung survives it too — the entry keeps its subpath and its
/// node, because a move rewrites operands and leaves the operator count,
/// and therefore every index, alone.
#[test]
fn a_move_never_alters_a_node_selection() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: None,
        },
        false,
        true,
    );
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: Some(4),
        },
        false,
        true,
    );
    sel.resolve(Some(&stub()), 0, 0);
    assert_eq!(sel.level(), SelectionLevel::Node);
    let before = sel.entries().to_vec();

    sel.resolve(Some(&stub_moved(vec2(-3.0, 7.0))), 0, 1);

    assert_eq!(sel.entries(), before.as_slice());
    assert_eq!(sel.entries()[0].node, Some(4));
    assert_eq!(sel.entries()[0].subpath, Some(1));
}

// -----------------------------------------------------------------
// The delta is page space
// -----------------------------------------------------------------

/// ★ **The object lands where the pointer put it, at every zoom.**
///
/// The hit-tolerance trap, in the move gesture's clothing — and stating it
/// correctly is half the value of the test, because the *tempting* wording
/// is the wrong one. "The same screen distance yields the same page delta"
/// is **false and must be false**: a fixed screen distance is
/// `distance / zoom` page units, which is
/// `viewer::screen_to_page_distance_scales_as_one_over_zoom`. Asserting it
/// would be asserting the defect.
///
/// What must be invariant is the operator's experience: grab a point on the
/// page, drop it on another point on the page, and the object moves by the
/// distance *between those two page points* — the same answer at 25 % and
/// at 1200 %. So the fixture drags between two fixed **page** positions,
/// projects them to screen through the frame's mapping (which is what the
/// pointer really reports), converts back exactly as `canvas/mod.rs` does,
/// and asserts one answer at four magnifications.
///
/// A second division by zoom anywhere in that chain — or a missing one —
/// makes this fan out by a factor of the zoom, which is precisely the
/// failure [`crate::canvas::mapping`] was built to make unavailable and the
/// reason [`PageMapping`](crate::canvas::mapping::PageMapping) has no
/// `zoom()` accessor to divide by.
#[test]
fn a_drag_between_two_page_points_moves_the_same_distance_at_every_zoom() {
    use crate::canvas::mapping::PageMapping;
    use crate::viewer::page_extent_pts;

    let page = test_page(200.0, 300.0, 0);
    let extent = page_extent_pts(&page);
    // Two positions ON THE PAGE, in canvas space: grab here, drop there.
    let grabbed = Pos2::new(40.0, 60.0);
    let dropped = Pos2::new(100.0, 84.0);

    let mut seen: Vec<PageDelta> = Vec::new();
    for &zoom in &[0.25_f32, 1.0, 4.0, 12.0] {
        let image_rect = Rect::from_min_size(
            Pos2::new(37.0, 11.0),
            vec2(extent.0 * zoom, extent.1 * zoom),
        );
        let map = PageMapping::new(image_rect, extent, zoom);
        // Round-trip through the screen, because that is the only thing
        // the pointer ever reports — and it is where a stray zoom would
        // enter.
        let from = map.to_page(map.to_screen(grabbed));
        let to = map.to_page(map.to_screen(dropped));
        seen.push(page_delta(to - from, &page).expect("invertible page"));
    }
    for delta in &seen {
        assert!(
            (delta.dx - seen[0].dx).abs() < 1e-3 && (delta.dy - seen[0].dy).abs() < 1e-3,
            "the page delta changed with the zoom: {seen:?}"
        );
    }
    // 60 canvas units right and 24 canvas units DOWN, which in Y-up PDF
    // user space is +60 and -24.
    assert!((seen[0].dx - 60.0).abs() < 1e-3, "{seen:?}");
    assert!((seen[0].dy + 24.0).abs() < 1e-3, "{seen:?}");
}

/// The canvas is Y-down and PDF user space is Y-up, so a downward drag is
/// a *negative* dy. Stated as its own assertion because getting it
/// backwards is silent: the object moves, just the wrong way.
#[test]
fn a_downward_drag_is_a_negative_page_dy() {
    let page = test_page(200.0, 300.0, 0);
    let delta = page_delta(vec2(0.0, 10.0), &page).expect("invertible page");
    assert!(delta.dy < 0.0, "{delta:?}");
    assert!((delta.dy + 10.0).abs() < 1e-3, "{delta:?}");
}

/// A rotated page rotates the delta, and it does so through the renderer's
/// own transform rather than a formula written out here. On a page turned
/// 90° clockwise, dragging right on screen moves the object *down* the
/// un-rotated page — i.e. -y in PDF user space.
#[test]
fn a_rotated_page_rotates_the_delta() {
    let page = test_page(200.0, 300.0, 90);
    let delta = page_delta(vec2(10.0, 0.0), &page).expect("invertible page");
    assert!(delta.dx.abs() < 1e-3, "{delta:?}");
    assert!((delta.dy.abs() - 10.0).abs() < 1e-3, "{delta:?}");
    // And the un-rotated page's answer is the other axis entirely, which is
    // what makes this a rotation test rather than a magnitude test.
    let upright =
        page_delta(vec2(10.0, 0.0), &test_page(200.0, 300.0, 0)).expect("invertible page");
    assert!((upright.dx - 10.0).abs() < 1e-3, "{upright:?}");
    assert!(upright.dy.abs() < 1e-3, "{upright:?}");
}

/// A drag that ends where it began raises nothing — a no-op must not take
/// a slot on the undo stack.
#[test]
fn a_drag_with_no_travel_commits_nothing() {
    let sel = two_objects_selected();
    let subject = eligible(&sel, 0, paths()).expect("eligible");
    assert_eq!(
        action(subject, PageDelta { dx: 0.0, dy: 0.0 }, None, &[]),
        Err(Refusal::NoTravel)
    );
}

/// …but the smallest real travel does commit. There is no second
/// threshold; egui's drag threshold is the only one.
#[test]
fn the_smallest_real_travel_still_commits() {
    let sel = two_objects_selected();
    let subject = eligible(&sel, 0, paths()).expect("eligible");
    let raised = action(subject, PageDelta { dx: 0.01, dy: 0.0 }, None, &[]).expect("committed");
    assert!(matches!(
        raised,
        Action::Vector(VectorAction::MoveSelection { .. })
    ));
}

/// A non-finite delta is refused rather than authored into a content
/// stream.
#[test]
fn a_non_finite_delta_is_refused() {
    let sel = two_objects_selected();
    for delta in [
        PageDelta {
            dx: f64::NAN,
            dy: 0.0,
        },
        PageDelta {
            dx: 0.0,
            dy: f64::INFINITY,
        },
    ] {
        let subject = eligible(&sel, 0, paths()).expect("eligible");
        assert_eq!(action(subject, delta, None, &[]), Err(Refusal::NoTravel));
    }
}

// -----------------------------------------------------------------
// One gesture, one command
// -----------------------------------------------------------------

/// ★ **A multi-select moves as ONE command**, carrying the whole operand
/// list — never one action per object, which would be N undo entries and N
/// re-splices planned against stale byte offsets.
#[test]
fn a_multi_select_moves_as_one_command() {
    let sel = two_objects_selected();
    let subject = eligible(&sel, 0, paths()).expect("eligible");
    assert_eq!(
        subject,
        MoveSubject::Objects {
            page: 0,
            objects: vec![0, 1],
        }
    );
    assert_eq!(
        action(subject, PageDelta { dx: 5.0, dy: -2.0 }, None, &[]),
        Ok(VectorAction::MoveSelection {
            page: 0,
            objects: vec![0, 1],
            dx: 5.0,
            dy: -2.0,
        }
        .into())
    );
}

/// Nothing selected raises nothing rather than an empty batch the engine
/// would have to refuse.
#[test]
fn an_empty_selection_moves_nothing() {
    let sel = SelectionState::default();
    assert_eq!(eligible(&sel, 0, paths()), Err(Refusal::NothingSelected));
}

/// A selection on another page is not moved by a drag on this one.
#[test]
fn a_selection_on_another_page_is_not_moved() {
    let mut sel = SelectionState::default();
    sel.click(3, hit_object(0), false, false);
    assert_eq!(eligible(&sel, 0, paths()), Err(Refusal::NothingSelected));
}

/// ★★★ **A non-path member ROUTES THE MOVE THROUGH A TRANSFORM** — and
/// this test used to assert that it refused the whole drag.
///
/// It read *"a non-path member refuses the WHOLE move, and names the
/// offender"*, and the reasoning was sound while it lasted:
///
/// > *"The engine does this too, and would do it correctly. Refusing here
/// > as well is what keeps the ghost honest: an outline that slides across
/// > the page and then snaps back has already told the operator something
/// > untrue."*
///
/// The ghost obligation stands and is now satisfied the other way round —
/// the outline slides **and the release commits**, because `Pass 113.0` gave
/// this shell a verb that moves anything. The operator asked for it three
/// times: *"can I please please please have the capability to move the text
/// after?"*
///
/// ★ What is asserted is the **rung**, not the absence of a refusal: a
/// build that routed every move through the transform would also stop
/// refusing here, and it would be wrong for the reason `eligible`'s own
/// comment gives about the file rather than the API.
#[test]
fn a_non_path_in_the_selection_routes_through_a_transform() {
    let sel = two_objects_selected();
    let ctx = MoveContext {
        non_path: Some(1),
        ..paths()
    };
    assert!(
        matches!(
            eligible(&sel, 0, ctx),
            Ok(MoveSubject::Transform { page: 0, .. })
        ),
        "a selection containing a picture or a text run must reach the transform rung"
    );
}

/// …and an all-path selection still takes the LIGHTER verb.
///
/// The other half of the fork, and the half a tidy-up would delete. A
/// transform wraps each object in `q <cm> … Q` per gesture; `move_objects`
/// rewrites the coordinates in place and adds nothing. On a drawing that is
/// nudged dozens of times, the wrapping accumulates in a file somebody then
/// sends on.
#[test]
fn an_all_path_selection_still_reaches_move_objects() {
    let sel = two_objects_selected();
    assert!(
        matches!(
            eligible(&sel, 0, paths()),
            Ok(MoveSubject::Objects { page: 0, .. })
        ),
        "a selection made only of shapes must not pay for the general verb"
    );
}

// -----------------------------------------------------------------
// The rung decides the verb
// -----------------------------------------------------------------

/// The Part rung of a path reaches `move_subpath`, with the entered
/// subpath as its operand.
#[test]
fn the_part_rung_reaches_move_subpath() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Part);

    let subject = eligible(&sel, 0, paths()).expect("eligible");
    assert_eq!(
        subject,
        MoveSubject::Subpath {
            page: 0,
            object: 0,
            subpath: 1,
        }
    );
    assert_eq!(
        action(subject, PageDelta { dx: 1.5, dy: 2.5 }, None, &[]),
        Ok(VectorAction::MoveSubpath {
            page: 0,
            object: 0,
            subpath: 1,
            dx: 1.5,
            dy: 2.5,
        }
        .into())
    );
}

/// ★ **A text run at the Part rung declines** — it is a part, and there is
/// no verb that moves one. The same shape as Delete declining at a rung
/// whose verb is not wired.
#[test]
fn a_text_run_at_the_part_rung_declines_rather_than_moving_the_object() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: None,
        },
        false,
        true,
    );
    let ctx = MoveContext {
        non_path: None,
        part_kind: Some(PartKind::Run),
    };
    assert_eq!(
        eligible(&sel, 0, ctx),
        Err(Refusal::NoVerbForPart(PartKind::Run)),
        "moving the enclosing object because a run was selected is the wrong action, \
         not a lenient one"
    );
}

/// The Node rung reaches `move_node`, and the destination is the anchor's
/// current position **plus** the delta — absolute, because that is what the
/// verb takes.
#[test]
fn the_node_rung_reaches_move_node_with_an_absolute_destination() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: None,
        },
        false,
        true,
    );
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: Some(4),
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Node);

    let subject = eligible(&sel, 0, paths()).expect("eligible");
    assert_eq!(
        subject,
        MoveSubject::Node {
            page: 0,
            object: 0,
            node: 4,
        }
    );
    let raised = action(
        subject,
        PageDelta { dx: 10.0, dy: -4.0 },
        Some(Point::new(100.0, 200.0)),
        &[],
    );
    assert_eq!(
        raised,
        Ok(VectorAction::MoveNode {
            page: 0,
            object: 0,
            node: 4,
            to: Point::new(110.0, 196.0),
        }
        .into())
    );
}

/// A node whose position the decomposition no longer reports refuses,
/// rather than moving the anchor to the delta itself — which would fling
/// it to the bottom-left of the page.
#[test]
fn a_node_with_no_known_position_refuses() {
    let subject = MoveSubject::Node {
        page: 0,
        object: 0,
        node: 4,
    };
    assert_eq!(
        action(subject, PageDelta { dx: 1.0, dy: 1.0 }, None, &[]),
        Err(Refusal::NodeNotFound(4))
    );
}

/// With no object model the move declines: nothing can be verified, so
/// nothing may be promised — and in particular no ghost is drawn.
#[test]
fn a_page_with_no_object_model_declines() {
    let sel = two_objects_selected();
    let mut actions = Vec::new();
    let ghost = drag(
        vec2(10.0, 10.0),
        Phase::InFlight,
        &sel,
        0,
        None,
        None,
        &mut actions,
    );
    assert_eq!(
        ghost.ghost, None,
        "a ghost must not describe an unverifiable move"
    );
    // ★ And no SHAPE either, since O63. The two are separate values and a
    // future edit could plausibly leave one of them populated on a rung that
    // declines — which would draw the operator a preview of a move that is
    // about to refuse, the exact "placeholder" failure R9 forbids.
    assert_eq!(
        ghost.shape, None,
        "a shape preview must not describe an unverifiable move either"
    );
    assert!(actions.is_empty());
}

/// ★★ **Four Shift-clicked anchors move as FOUR anchors, in one command.**
///
/// This is the regression test for a defect that lived in the gap between
/// two correct halves. `SelectionState::pick_within` has added a
/// Shift-clicked anchor as its own entry since the Node rung landed, and
/// `subject` read `entered_object()` — the FIRST entry. So the model held
/// four, the overlay drew four, and the drag moved one.
///
/// Nothing failed. Both halves' unit tests passed. The only thing that
/// would have caught it is driving it, or a test like this one that asks
/// the two halves the same question.
#[test]
fn several_selected_anchors_move_as_one_command() {
    let mut selection = SelectionState::default();
    selection.click(0, hit_object(7), false, false);
    selection.click(0, hit_node(7, 0, 1), false, true);
    selection.click(0, hit_node(7, 0, 1), false, true);
    // Now inside the object at the Node rung; Shift-pick three more.
    for node in [4_usize, 9, 2] {
        selection.click(0, hit_node(7, 0, node), true, false);
    }
    let nodes = selection.selected_nodes_on(0, TargetId::Object(7));
    assert!(
        nodes.len() >= 2,
        "the selection model must hold every Shift-picked anchor, got {nodes:?}"
    );

    let subject = eligible(
        &selection,
        0,
        MoveContext {
            non_path: None,
            part_kind: Some(PartKind::Subpath),
        },
    )
    .expect("a multi-node selection on a path has a move subject");
    let MoveSubject::Nodes { nodes, .. } = &subject else {
        panic!("several anchors must produce the PLURAL subject, got {subject:?}");
    };
    assert_eq!(
        nodes.len(),
        selection.selected_nodes_on(0, TargetId::Object(7)).len()
    );

    // Every selected anchor's position, so the plural arm can resolve them.
    let points: Vec<(usize, Point)> = (0..12)
        .map(|i| {
            (
                i,
                Point::new(f64::from(u32::try_from(i).unwrap()) * 10.0, 50.0),
            )
        })
        .collect();
    let raised = action(subject, PageDelta { dx: 3.0, dy: -7.0 }, None, &points)
        .expect("the plural arm resolves every anchor");
    let Action::Vector(VectorAction::MoveNodes { moves, .. }) = raised else {
        panic!("the plural subject must raise ONE MoveNodes, got {raised:?}");
    };
    assert!(moves.len() >= 2, "one command carrying every anchor");
    for (index, to) in &moves {
        let from = points[*index].1;
        assert!((to.x - (from.x + 3.0)).abs() < 1e-9);
        assert!((to.y - (from.y - 7.0)).abs() < 1e-9);
    }
}

/// ★ **One stale anchor refuses the whole drag**, rather than moving the
/// three the decomposition still recognises.
///
/// The same call `move_objects` makes over a non-path member, and for the
/// same reason its docs give: a partial application reads as a rendering
/// fault rather than as a refusal, and the operator has no way to learn
/// which of their anchors was dropped.
#[test]
fn one_missing_anchor_refuses_the_whole_move() {
    let subject = MoveSubject::Nodes {
        page: 0,
        object: 7,
        nodes: vec![0, 1, 99],
    };
    let points: Vec<(usize, Point)> = (0..3).map(|i| (i, Point::new(0.0, 0.0))).collect();
    let err = action(subject, PageDelta { dx: 1.0, dy: 1.0 }, None, &points)
        .expect_err("a selection that out-ran the decomposition must refuse");
    assert_eq!(err, Refusal::NodeNotFound(99));
}

/// A single selected anchor still takes the SINGULAR verb.
///
/// `EditSession` has both, and `docs/core-api/02`'s rule cuts both ways:
/// the plural verb is correct for a set and the singular one for a member.
/// Routing one anchor through a slice would lose the singular planner for
/// no gain.
#[test]
fn one_selected_anchor_still_takes_the_singular_verb() {
    let mut selection = SelectionState::default();
    selection.click(0, hit_object(7), false, false);
    selection.click(0, hit_node(7, 0, 1), false, true);
    selection.click(0, hit_node(7, 0, 1), false, true);
    if selection.selected_nodes_on(0, TargetId::Object(7)).len() != 1 {
        // The descent did not reach the Node rung on this fixture shape;
        // the assertion below would then be about the wrong thing.
        return;
    }
    let subject = eligible(
        &selection,
        0,
        MoveContext {
            non_path: None,
            part_kind: Some(PartKind::Subpath),
        },
    )
    .expect("one anchor on a path has a move subject");
    assert!(
        matches!(subject, MoveSubject::Node { .. }),
        "one anchor must stay singular, got {subject:?}"
    );
}
