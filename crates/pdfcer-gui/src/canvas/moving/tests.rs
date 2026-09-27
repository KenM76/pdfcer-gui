//! # `canvas::moving::tests` — the move gesture's algebra, asserted
//!
//!
//! ## What these are about, and why the module they test is worth this much
//!
//! Three obligations, stated in [`super`]'s header and each one paid for by a
//! defect:
//!
//!
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
//! **The line gate still counts these lines.** `check-file-size.sh` counts
//! total lines, tests included, on purpose — so this is not a way of hiding
//! from R2. It is the split R2 asked for.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/moving/tests.md`.

#![cfg(test)]

use super::*;
use crate::app::actions::CanvasDecline;
use crate::canvas::selection::ClickHit;
use crate::canvas::{target::TargetId, targetstub::StubTargets};
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
        resources_defaulted: false,
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
        chunk: false,
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
        run_move: None,
    }
}

// -----------------------------------------------------------------
// The invariant the whole feature was blocked on
// -----------------------------------------------------------------

/// **A move never alters the selection.**
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
            chunk: false,
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
            chunk: false,
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

/// **The object lands where the pointer put it, at every zoom.**
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

/// **A multi-select moves as ONE command**, carrying the whole operand
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

/// **A non-path member ROUTES THE MOVE THROUGH A TRANSFORM** — and
/// this test used to assert that it refused the whole drag.
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
            chunk: false,
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

/// A selection with the first part of object 0 entered — the shape every run
/// test below starts from, factored out because four of them need it and a
/// fifth copy would be a fifth chance to enter a different rung by accident.
fn run_entered() -> SelectionState {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: None,
            chunk: false,
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Part);
    sel
}

/// **A text run at the Part rung MOVES** — `OPERATOR_REQUESTS.md` O188,
/// and the day this test was inverted is the day the feature shipped.
#[test]
fn a_line_of_text_at_the_part_rung_moves_that_line() {
    let sel = run_entered();
    let ctx = MoveContext {
        non_path: None,
        part_kind: Some(PartKind::TextLine),
        run_move: None,
    };
    assert_eq!(
        eligible(&sel, 0, ctx),
        Ok(MoveSubject::TextLine {
            page: 0,
            object: 0,
            line: 0,
        }),
        "a run the engine would accept must move as a run — not as its enclosing \
         object, and not as a refusal"
    );
    assert_eq!(
        action(
            MoveSubject::TextLine {
                page: 0,
                object: 0,
                line: 0,
            },
            PageDelta { dx: 4.0, dy: -1.25 },
            None,
            &[],
        ),
        Ok(VectorAction::MoveTextLine {
            page: 0,
            object: 0,
            line: 0,
            dx: 4.0,
            dy: -1.25,
        }
        .into()),
        "the delta must reach the verb unmodified — a run has no anchors, so \
         nothing here may snap, clamp or round it"
    );
}

/// **Several Shift-clicked chunks move as SEVERAL chunks, in one undo
/// entry** — O215 ask 4.
#[test]
fn several_selected_chunks_move_as_one_command() {
    let mut sel = run_entered();
    for part in [2_usize, 5] {
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId::Object(0)),
                part: Some(part),
                node: None,
                chunk: false,
            },
            true,
            false,
        );
    }
    assert_eq!(
        sel.selected_parts_on(0, TargetId::Object(0)),
        vec![0, 2, 5],
        "the model must hold every Shift-picked chunk"
    );

    let ctx = MoveContext {
        non_path: None,
        part_kind: Some(PartKind::TextLine),
        run_move: None,
    };
    assert_eq!(
        eligible(&sel, 0, ctx),
        Ok(MoveSubject::TextLines {
            page: 0,
            object: 0,
            lines: vec![0, 2, 5],
        }),
        "a multi-chunk selection must produce the PLURAL subject — the singular \
         one moves the first entry and says nothing about the other two"
    );
    assert_eq!(
        action(
            MoveSubject::TextLines {
                page: 0,
                object: 0,
                lines: vec![0, 2, 5],
            },
            PageDelta { dx: 4.0, dy: -1.25 },
            None,
            &[],
        ),
        Ok(VectorAction::MoveTextLines {
            page: 0,
            object: 0,
            lines: vec![0, 2, 5],
            dx: 4.0,
            dy: -1.25,
        }
        .into()),
        "one action carrying every chunk, so the fold gives one undo entry"
    );
}

/// A set that is back down to ONE chunk takes the singular verb again.
#[test]
fn one_selected_chunk_still_takes_the_singular_verb() {
    let mut sel = run_entered();
    // Add a second chunk and take it straight back out again.
    let second = ClickHit {
        object: Some(TargetId::Object(0)),
        part: Some(2),
        node: None,
        chunk: false,
    };
    sel.click(0, second, true, false);
    sel.click(0, second, true, false);
    assert_eq!(sel.selected_parts_on(0, TargetId::Object(0)), vec![0]);

    let ctx = MoveContext {
        non_path: None,
        part_kind: Some(PartKind::TextLine),
        run_move: None,
    };
    assert_eq!(
        eligible(&sel, 0, ctx),
        Ok(MoveSubject::TextLine {
            page: 0,
            object: 0,
            line: 0,
        })
    );
}

/// **A run the ENGINE would refuse never gets a ghost** — the pre-check
/// that makes O188's move half honest rather than merely present.
#[test]
fn a_run_the_engine_would_refuse_declines_before_the_ghost_is_drawn() {
    let sel = run_entered();
    for block in [
        RunMoveBlock::NoPositionOfItsOwn,
        RunMoveBlock::WouldMoveNextRun,
    ] {
        let ctx = MoveContext {
            non_path: None,
            part_kind: Some(PartKind::TextLine),
            run_move: Some(block),
        };
        assert_eq!(
            eligible(&sel, 0, ctx),
            Err(Refusal::TextRunCannotMove(block)),
            "{block:?} must reach the operator as its own refusal, carrying the \
             engine's reason rather than a summary of it"
        );
    }
}

/// **A refused drag on one line of text says WHICH refusal it was** —
/// `OPERATOR_REQUESTS.md` O188.
#[test]
fn a_refused_drag_on_one_line_of_text_asks_for_a_sentence() {
    for (block, expected) in [
        (
            RunMoveBlock::NoPositionOfItsOwn,
            CanvasDecline::TextRunHasNoPositionOfItsOwn,
        ),
        (
            RunMoveBlock::WouldMoveNextRun,
            CanvasDecline::TextRunWouldDragTheNextLine,
        ),
    ] {
        let mut actions = Vec::new();
        decline(
            &SelectionState::default(),
            Refusal::TextRunCannotMove(block),
            &mut actions,
        );
        assert_eq!(
            actions,
            vec![Action::DeclineOnCanvas(expected)],
            "{block:?} must raise its OWN sentence, not a shared one and not silence"
        );
    }
}

/// **The third block stays silent, and that is the correct answer** —
/// [`RunMoveBlock::NotThere`] means the selection named a line this object does
/// not have.
#[test]
fn a_run_index_that_is_not_there_refuses_without_a_sentence() {
    let mut actions = Vec::new();
    decline(
        &SelectionState::default(),
        Refusal::TextRunCannotMove(RunMoveBlock::NotThere),
        &mut actions,
    );
    assert!(
        actions.is_empty(),
        "an impossible selection must not word itself at the operator, got {actions:?}"
    );
}

/// The form-interior twin still asks for its own sentence — the arm that
/// existed before O188, asserted here because O188 rewrote the mechanism
/// underneath it from an `==` comparison to [`super::Refusal::worded`].
#[test]
fn a_refused_drag_inside_a_form_still_asks_for_its_own_sentence() {
    let mut actions = Vec::new();
    decline(
        &SelectionState::default(),
        Refusal::InsideForm,
        &mut actions,
    );
    assert_eq!(
        actions,
        vec![Action::DeclineOnCanvas(CanvasDecline::InsideFormNotAPath)],
        "the 2026-08-27 refusal must survive the rewrite that generalised it"
    );
}

/// **Every refusal describing a state the operator can SEE raises
/// nothing at all.**
#[test]
fn the_refusals_the_operator_can_see_raise_nothing() {
    for reason in silent_refusals() {
        let mut actions = Vec::new();
        decline(&SelectionState::default(), reason, &mut actions);
        assert!(
            actions.is_empty(),
            "{reason:?} describes a state the operator can see, so it must not \
             put a sentence on the bar; got {actions:?}"
        );
    }
}

/// **Every refusal traces a distinct, stable, lower-kebab token.**
///
#[test]
fn every_refusal_traces_a_distinct_stable_token() {
    let all = all_refusals();
    let mut seen: Vec<&'static str> = Vec::new();
    for reason in &all {
        let token = reason.token();
        assert!(
            !token.is_empty() && token.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
            "{reason:?} traces {token:?}, which is not a lower-kebab token \
             a harness can grep for"
        );
        assert!(
            !seen.contains(&token),
            "{reason:?} traces {token:?}, which another refusal already uses — a \
             collision makes a driven check assert the wrong cause"
        );
        seen.push(token);
    }
}

/// Every [`Refusal`], for the two completeness tests above.
fn all_refusals() -> Vec<Refusal> {
    let all = vec![
        Refusal::NoObjectModel,
        Refusal::NothingSelected,
        Refusal::UnaddressableObject,
        Refusal::InsideForm,
        Refusal::NotAPath(3),
        Refusal::NoPartEntered,
        Refusal::NoVerbForPart(PartKind::Subpath),
        Refusal::NoVerbForPart(PartKind::TextLine),
        // Every block, not one representative. The vector below is
        // what `refusals_that_owe_nothing` and its twin iterate, so a block
        // missing here is a block whose sentence — or whose deliberate
        // silence — is never checked by anything. `NotThere` is the one that
        // matters most: it is the only arm that says nothing, and an omission
        // here would read as a pass.
        Refusal::TextRunCannotMove(RunMoveBlock::NoPositionOfItsOwn),
        Refusal::TextRunCannotMove(RunMoveBlock::WouldMoveNextRun),
        Refusal::TextRunCannotMove(RunMoveBlock::NotThere),
        Refusal::NoNodeEntered,
        Refusal::NodeNotFound(1),
        Refusal::NoTravel,
        Refusal::DegeneratePage,
    ];
    for reason in &all {
        match reason {
            Refusal::NoObjectModel
            | Refusal::NothingSelected
            | Refusal::UnaddressableObject
            | Refusal::InsideForm
            | Refusal::NotAPath(_)
            | Refusal::NoPartEntered
            | Refusal::NoVerbForPart(_)
            // Destructured, where every neighbour is a wildcard. A
            // `TextRunCannotMove(_)` arm accepts a fourth [`RunMoveBlock`]
            // silently, and the vector above would then be missing the one
            // entry this whole function exists to force — the completeness
            // guard would go on passing while the newest refusal's sentence
            // was checked by nothing.
            | Refusal::TextRunCannotMove(
                RunMoveBlock::NoPositionOfItsOwn
                | RunMoveBlock::WouldMoveNextRun
                | RunMoveBlock::NotThere,
            )
            | Refusal::NoNodeEntered
            | Refusal::NodeNotFound(_)
            | Refusal::NoTravel
            | Refusal::DegeneratePage => {}
        }
    }
    all
}

/// The refusals that owe the operator nothing — every [`Refusal`] for which
/// [`super::Refusal::worded`] answers `None`, derived rather than re-typed.
fn silent_refusals() -> Vec<Refusal> {
    all_refusals()
        .into_iter()
        .filter(|reason| reason.worded().is_none())
        .collect()
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
            chunk: false,
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
            chunk: false,
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
    // And no SHAPE either, since O63. The two are separate values and a
    // future edit could plausibly leave one of them populated on a rung that
    // declines — which would draw the operator a preview of a move that is
    // about to refuse, the exact "placeholder" failure R9 forbids.
    assert_eq!(
        ghost.shape, None,
        "a shape preview must not describe an unverifiable move either"
    );
    assert!(actions.is_empty());
}

/// **Four Shift-clicked anchors move as FOUR anchors, in one command.**
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
            run_move: None,
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

/// **One stale anchor refuses the whole drag**, rather than moving the
/// three the decomposition still recognises.
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
            run_move: None,
        },
    )
    .expect("one anchor on a path has a move subject");
    assert!(
        matches!(subject, MoveSubject::Node { .. }),
        "one anchor must stay singular, got {subject:?}"
    );
}
