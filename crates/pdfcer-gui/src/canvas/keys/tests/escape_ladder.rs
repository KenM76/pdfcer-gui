//! # The Escape ladder, enumerated
//!
//! [`super::super::escape`] — the function, not the file next door — is an
//! ordered list of `if let` arms, and the only way to hold an ordered list in
//! place is to enumerate it: every rung needs a case that **reaches** it and a
//! case that proves the rung above **did not swallow** it. That is why twelve
//! tests assert one short function, and why they are their own file rather
//! than a section of [`super`].
//!
//! ## The rungs, in the order Escape meets them
//!
//! | Rung | Reached by | Protected from above by |
//! |---|---|---|
//! | a live drag spends the press | `an_escape_spent_on_a_drag_leaves_the_rung_alone` | — it is the top |
//! | a markup tool is put down | `escape_retires_the_markup_tool_before_the_region_zoom` | `an_escape_spent_on_a_markup_drag_leaves_the_tool_armed` |
//! | an armed region zoom retires | `escape_retires_an_armed_region_zoom_before_it_touches_the_ladder` | `an_escape_spent_on_a_drag_leaves_the_armed_zoom_alone` |
//! | the selection ascends a rung | `escape_ascends_a_rung_and_raises_no_action` | `escape_reaches_the_ladder_again_once_nothing_is_armed` |
//!
//! The in-progress constructions — a guide drag, a circle fit, a vertex run —
//! each get a pair of their own, because each is abandoned *before* the rung
//! below it and a second press then reaches that rung.
//!
//! ## ★ Not to be confused with `canvas::escape`
//!
//! That module is the keyboard route **out of a canvas that drew nothing**,
//! and has no rungs. This one is the Escape key's precedence over the canvas's
//! claimants on a frame that drew normally. The two never interact; the shared
//! word is the key's name.
//!
//! ## What every case here passes, and why
//!
//! `targets: None` with `model_attempted: true` — no decomposition, and the
//! frame asked for one — which is what lets these run without opening a file.
//! `page: None`, which is honest: no assertion here presses an arrow, and
//! [`keys::Keys::page`] exists for the nudge alone. `PickFilter::all()`, which
//! is what a shell that has never touched the filter hands over. Those
//! statements are true **of this file**; [`super`]'s header no longer claims
//! them of the Delete side, where a Tab case and an arrow case both exist.

use super::*;

/// ★ **An Escape already spent cancelling a drag does not also ascend a
/// rung.** One press, one effect: an operator who abandons a move drag
/// must still be standing where they were, or cancelling costs them the
/// part they were working in as well as the drag.
#[test]
fn an_escape_spent_on_a_drag_leaves_the_rung_alone() {
    let mut selection = part_entered();
    let ctx = Context::default();
    let mut actions = Vec::new();
    let mut text_selection = None;
    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: true,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });
    assert_eq!(selection.level(), SelectionLevel::Part);
    assert!(actions.is_empty());
}

/// ★ **Escape retires an armed region zoom instead of ascending a rung —
/// and only one of the two happens.**
///
/// The rule this must not break is already in the file above: *"there is
/// already an Escape rule that must not both cancel a drag and ascend a
/// selection rung."* Phase 3.4 inserts a third claimant between them, so
/// the same discipline is asserted for the new pair: an operator who arms
/// a marquee zoom and changes their mind gets out of the tool **and keeps
/// the part they were working in**.
#[test]
fn escape_retires_an_armed_region_zoom_before_it_touches_the_ladder() {
    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    zoom::arm_region_zoom(&ctx);

    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });

    assert!(
        !zoom::region_zoom_armed(&ctx),
        "the zoom tool must be retired"
    );
    assert_eq!(
        selection.level(),
        SelectionLevel::Part,
        "and the rung must be left exactly where it was"
    );
    assert!(actions.is_empty());
}

/// …and the *next* Escape, with nothing armed, ascends exactly as it
/// always did. Without this the test above would pass on a build where
/// Escape had stopped reaching the ladder altogether.
#[test]
fn escape_reaches_the_ladder_again_once_nothing_is_armed() {
    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    assert!(!zoom::region_zoom_armed(&ctx));

    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });

    assert_eq!(selection.level(), SelectionLevel::Object);
    assert_eq!(selection.len(), 1, "leaving a rung does not clear");
}

/// ★ **An Escape already spent cancelling a drag leaves the armed zoom
/// alone too.**
///
/// The one-press-one-effect rule runs in both directions: a cancelled
/// zoom-marquee drag must not *also* disarm the tool, or an operator who
/// mis-drags a zoom box has to re-arm it before they can try again.
#[test]
fn an_escape_spent_on_a_drag_leaves_the_armed_zoom_alone() {
    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    zoom::arm_region_zoom(&ctx);

    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: true,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });

    assert!(
        zoom::region_zoom_armed(&ctx),
        "the drag consumed the key; the arming must survive for the retry"
    );
    assert_eq!(selection.level(), SelectionLevel::Part);
}

/// Escape ascends one rung and raises no action — the ladder is canvas
/// state, not a document change.
#[test]
fn escape_ascends_a_rung_and_raises_no_action() {
    let mut selection = part_entered();
    assert!(keys_for(key(Key::Escape), &mut selection).is_empty());
    assert_eq!(selection.level(), SelectionLevel::Object);
    assert_eq!(selection.len(), 1, "leaving a rung does not clear");

    assert!(keys_for(key(Key::Escape), &mut selection).is_empty());
    assert!(selection.is_empty(), "the next press clears");
}

/// ★ **Escape retires an armed markup tool before it touches the region
/// zoom or the ladder — and retires exactly one thing.**
///
/// Both are armed at once deliberately, for the reason the guide-versus-zoom
/// test below states: asserting the markup tool is retired would pass on a
/// build that retired everything. Asserting the zoom **survives** is what
/// makes it a precedence test.
#[test]
fn escape_retires_the_markup_tool_before_the_region_zoom() {
    use crate::canvas::markup::MarkupKind;
    use crate::canvas::tool;

    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    zoom::arm_region_zoom(&ctx);
    tool::arm_markup(&ctx, MarkupKind::Rectangle);

    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });

    assert_eq!(
        tool::selected(&ctx),
        tool::CanvasTool::Select,
        "the pen must be put down"
    );
    assert!(
        zoom::region_zoom_armed(&ctx),
        "the armed zoom must SURVIVE: one press, one effect"
    );
    assert_eq!(
        selection.level(),
        SelectionLevel::Part,
        "and the selection rung must be untouched"
    );
    assert!(actions.is_empty());
}

/// ★ **An Escape already spent abandoning a markup band does NOT also put
/// the pen down.**
///
/// The sharpest form of one-press-one-effect for this feature: an operator
/// who mis-drags a rectangle and cancels it is still holding the rectangle
/// tool, so their next drag draws a rectangle. Retiring the tool as well
/// would make every abandoned drag cost a trip back to the ribbon.
#[test]
fn an_escape_spent_on_a_markup_drag_leaves_the_tool_armed() {
    use crate::canvas::markup::MarkupKind;
    use crate::canvas::tool;

    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    tool::arm_markup(&ctx, MarkupKind::Ellipse);

    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        // `true`: the gesture machine already spent the key cancelling the
        // band, exactly as `canvas::interact` reports it.
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: true,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });

    assert_eq!(
        tool::selected(&ctx),
        tool::CanvasTool::Markup(MarkupKind::Ellipse),
        "the drag consumed the key; the tool must survive for the retry"
    );
    assert_eq!(selection.level(), SelectionLevel::Part);
}

/// …and with no markup armed, Escape reaches the zoom and then the ladder
/// exactly as it did before this claimant existed. Without this, the two
/// tests above would pass on a build where the markup claimant swallowed
/// every Escape.
#[test]
fn escape_still_reaches_the_zoom_and_the_ladder_with_no_markup_armed() {
    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    zoom::arm_region_zoom(&ctx);

    for _ in 0..2 {
        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                Keys {
                    ctx: ui.ctx(),
                    page_index: 0,
                    pick: PickFilter::all(),
                    caps: Capabilities::FULL,
                    selected_field: None,
                    annot_delete_refused: false,
                    field_delete_refused: false,
                    targets: None,
                    edit_epoch: 0,
                    model_attempted: true,
                    page: None,
                    escape_consumed: false,
                },
                &mut selection,
                &mut text_selection,
                &mut actions,
            );
        });
    }

    assert!(
        !zoom::region_zoom_armed(&ctx),
        "the first press took the zoom"
    );
    assert_eq!(
        selection.level(),
        SelectionLevel::Object,
        "and the second reached the ladder"
    );
}

/// ★ **A guide drag outranks an armed region zoom, and only one of the
/// two is retired.**
///
/// The tie-break the precedence table states: retire the most transient
/// thing first. Both are "in flight", and the guide is the one following
/// the pointer *this frame* while the zoom is waiting for a drag that has
/// not started.
///
/// Both are armed at once deliberately. Asserting the guide is cancelled
/// would pass on a build that cancelled everything; asserting the zoom
/// SURVIVES is what makes it a precedence test rather than a "something
/// happened" test.
#[test]
fn escape_abandons_a_guide_drag_before_it_touches_the_region_zoom() {
    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    zoom::arm_region_zoom(&ctx);
    crate::canvas::guides::plant_drag_for_test(&ctx);

    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });

    assert!(
        zoom::region_zoom_armed(&ctx),
        "the armed zoom must SURVIVE: one press, one effect"
    );
    assert_eq!(
        selection.level(),
        SelectionLevel::Part,
        "and the selection rung must be untouched"
    );
    assert!(
        actions.is_empty(),
        "an abandoned drag holds a proposal, so it raises no action"
    );
}

/// ★ **A circular pick set is abandoned by the FIRST Escape and the tool
/// by the second** — the two rungs, over the one tool that most needs
/// them.
///
/// The radius/diameter gesture has no natural end, so a pick set can sit
/// there for as long as the operator keeps toggling arcs into it. That
/// makes the two-rung rule load-bearing rather than tidy: an operator who
/// has picked four arcs and catches a fifth by mistake presses Escape to
/// correct it and must find themselves still holding the tool, with the
/// set cleared — not back in the select tool with everything gone.
///
/// A region zoom is armed throughout, and asserting it **survives** both
/// presses is what makes this a precedence test rather than a "something
/// happened" test: a build that retired everything on the first press would
/// pass the first two assertions.
#[test]
fn escape_abandons_a_circle_fit_before_it_puts_the_measure_tool_down() {
    use crate::canvas::measure::{self, MeasureKind};
    use crate::canvas::tool;

    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    zoom::arm_region_zoom(&ctx);
    tool::arm_measure(&ctx, MeasureKind::Circular);
    measure::circular::plant_pick_for_test(&ctx, 0);
    assert!(
        measure::finishable(&ctx),
        "the fixture must be a real, finishable pick set"
    );

    // Press 1: the pick set, and nothing else.
    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });
    assert_eq!(
        tool::selected(&ctx),
        tool::CanvasTool::Measure(MeasureKind::Circular),
        "the tool must survive: one press corrects a mis-picked arc"
    );
    assert!(
        !measure::finishable(&ctx),
        "and the pick set is the thing that went"
    );
    assert!(zoom::region_zoom_armed(&ctx), "one press, one effect");
    assert_eq!(selection.level(), SelectionLevel::Part);

    // Press 2: the tool.
    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });
    assert_eq!(
        tool::selected(&ctx),
        tool::CanvasTool::Select,
        "the second press puts the tool down"
    );
    assert!(zoom::region_zoom_armed(&ctx), "and still not the zoom");
    assert_eq!(
        selection.level(),
        SelectionLevel::Part,
        "two presses, two effects — and neither of them the ladder"
    );
    assert!(
        actions.is_empty(),
        "abandoning a pick authors nothing: the dimension only exists once \
         one of the two endings raises it"
    );
}

/// ★ **A markup vertex run is abandoned by the FIRST Escape and the pen by
/// the second** — rung 3a's second occupant, asserted the same way its first
/// is.
///
/// Written as a near-copy of the circle-fit test above **on purpose**, and
/// the copy is the point rather than duplication: the two gestures have the
/// same problem (a run of clicks with no natural end), were given the same
/// answer (two endings, one commit path), and now share a rung — so a build
/// that got the precedence right for one and wrong for the other is exactly
/// what a near-copy catches and a shared helper would hide.
///
/// The operator's case is concrete: someone clicking out a polygon round a
/// detail catches a seventh corner by mistake, presses Escape to correct it,
/// and must find themselves **still holding the pen** with the run cleared —
/// not back in the select tool with everything gone and the tool to re-arm.
///
/// A region zoom is armed throughout, and asserting it **survives both
/// presses** is what makes this a precedence test rather than a "something
/// happened" test: a build that retired everything on the first press would
/// pass the first two assertions.
#[test]
fn escape_abandons_a_vertex_run_before_it_puts_the_markup_tool_down() {
    use crate::canvas::markup::{MarkupKind, vertex};
    use crate::canvas::tool;

    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    zoom::arm_region_zoom(&ctx);
    tool::arm_markup(&ctx, MarkupKind::Polygon);
    vertex::plant_run_for_test(&ctx, 0, MarkupKind::Polygon);
    assert!(
        vertex::finishable(&ctx),
        "the fixture must be a real, finishable run"
    );

    // Press 1: the run, and nothing else.
    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });
    assert_eq!(
        tool::selected(&ctx),
        tool::CanvasTool::Markup(MarkupKind::Polygon),
        "the pen must survive: one press corrects a mis-clicked corner"
    );
    assert!(
        !vertex::finishable(&ctx),
        "and the run is the thing that went"
    );
    assert!(zoom::region_zoom_armed(&ctx), "one press, one effect");
    assert_eq!(selection.level(), SelectionLevel::Part);

    // Press 2: the pen.
    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });
    assert_eq!(
        tool::selected(&ctx),
        tool::CanvasTool::Select,
        "the second press puts the pen down"
    );
    assert!(zoom::region_zoom_armed(&ctx), "and still not the zoom");
    assert_eq!(
        selection.level(),
        SelectionLevel::Part,
        "two presses, two effects — and neither of them the ladder"
    );
    assert!(
        actions.is_empty(),
        "abandoning a run authors nothing: the annotation only exists once \
         one of the two endings raises it"
    );
}

/// …and a second Escape then retires the zoom, so nothing is stranded.
///
/// Without this, the test above would pass on a build where a guide drag
/// permanently swallowed Escape — which is a worse bug than the one being
/// fixed, because it would leave the operator unable to leave any tool.
#[test]
fn a_second_escape_retires_the_zoom_the_guide_drag_protected() {
    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    zoom::arm_region_zoom(&ctx);
    crate::canvas::guides::plant_drag_for_test(&ctx);

    for _ in 0..2 {
        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                Keys {
                    ctx: ui.ctx(),
                    page_index: 0,
                    pick: PickFilter::all(),
                    caps: Capabilities::FULL,
                    selected_field: None,
                    annot_delete_refused: false,
                    field_delete_refused: false,
                    targets: None,
                    edit_epoch: 0,
                    model_attempted: true,
                    page: None,
                    escape_consumed: false,
                },
                &mut selection,
                &mut text_selection,
                &mut actions,
            );
        });
    }

    assert!(
        !zoom::region_zoom_armed(&ctx),
        "the second press must reach the zoom"
    );
    assert_eq!(
        selection.level(),
        SelectionLevel::Part,
        "two presses, two effects — and neither of them the ladder"
    );
}
