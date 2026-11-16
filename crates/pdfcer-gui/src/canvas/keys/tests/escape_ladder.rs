//! # The Escape ladder, enumerated
//!
//! [`super::super::escape`] — the function, not the file next door — is an
//! ordered list of `if let` arms, and the only way to hold an ordered list in
//! place is to enumerate it: every rung needs a case that **reaches** it and a
//! case that proves the rung above **did not swallow** it. That is why twelve
//! tests assert one short function, and why they are their own file rather
//! than a section of [`super`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/keys/tests/escape_ladder.md`.

use super::*;

/// **An Escape already spent cancelling a drag does not also ascend a
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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

/// **Escape retires an armed region zoom instead of ascending a rung —
/// and only one of the two happens.**
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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

/// **An Escape already spent cancelling a drag leaves the armed zoom
/// alone too.**
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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

/// **Escape retires an armed markup tool before it touches the region
/// zoom or the ladder — and retires exactly one thing.**
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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

/// **An Escape already spent abandoning a markup band does NOT also put
/// the pen down.**
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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
                    field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                    annot_delete_refused: false,
                    field_delete_refused: false,
                    targets: None,
                    edit_epoch: 0,
                    model_attempted: true,
                    page: None,
                    field_page: None,
                    pen: Default::default(),
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

/// **A guide drag outranks an armed region zoom, and only one of the
/// two is retired.**
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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

/// **A circular pick set is committed by the FIRST Escape and the tool put
/// down by the second** — the two rungs, over an open-ended measure tool.
#[test]
fn escape_commits_a_circle_fit_before_it_puts_the_measure_tool_down() {
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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
        "the tool must survive: one press, one effect"
    );
    assert!(
        !measure::finishable(&ctx),
        "the pick set is spent, and not left waiting for a placing click"
    );
    assert!(
        matches!(
            actions.as_slice(),
            [Action::Dimension(
                crate::app::actions::dimensions::DimensionAction::Commit { .. }
            )]
        ),
        "the fit is committed as its ce dimension, not discarded"
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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
    assert_eq!(
        actions.len(),
        1,
        "putting the tool down authors nothing more"
    );
}

/// **A markup vertex run is finished by the FIRST Escape and the pen put down
/// by the second** — rung 3a's second occupant, asserted the same way its
/// first is.
#[test]
fn escape_finishes_a_vertex_run_before_it_puts_the_markup_tool_down() {
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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
        "the pen must survive: one press, one effect"
    );
    assert!(!vertex::finishable(&ctx), "the run is spent");
    assert!(
        matches!(
            actions.as_slice(),
            [Action::CommitMarkup {
                kind: MarkupKind::Polygon,
                ..
            }]
        ),
        "the run is committed as its polygon, not discarded"
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
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
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
    assert_eq!(
        actions.len(),
        1,
        "putting the pen down authors nothing more"
    );
}

/// One Escape through the whole ladder, with nothing selected around it.
fn press_escape(ctx: &Context, actions: &mut Vec<Action>) {
    let mut selection = part_entered();
    let mut text_selection = None;
    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            actions,
        );
    });
}

/// **A run too short for its kind is abandoned, not authored** — and the pen
/// stays up, so the press still has exactly one effect.
#[test]
fn escape_abandons_a_vertex_run_too_short_to_finish() {
    use crate::canvas::markup::{MarkupKind, vertex};
    use crate::canvas::tool;

    let ctx = Context::default();
    let mut actions = Vec::new();
    tool::arm_markup(&ctx, MarkupKind::Polygon);
    vertex::plant_short_run_for_test(&ctx, 0, MarkupKind::Polygon);
    assert!(!vertex::finishable(&ctx), "the fixture must be too short");

    press_escape(&ctx, &mut actions);
    assert!(actions.is_empty(), "two corners are not a polygon");
    assert_eq!(
        tool::selected(&ctx),
        tool::CanvasTool::Markup(MarkupKind::Polygon),
        "the press went on the run, not on the pen"
    );
    press_escape(&ctx, &mut actions);
    assert_eq!(tool::selected(&ctx), tool::CanvasTool::Select);
}

/// **A Linear pick is still discarded**: only the open-ended tools finish on
/// Escape, because a half-taken two-point pick has no ce dimension to author.
#[test]
fn escape_still_discards_a_linear_pick() {
    use crate::canvas::measure::{self, MeasureKind};
    use crate::canvas::tool;

    let ctx = Context::default();
    let mut actions = Vec::new();
    tool::arm_measure(&ctx, MeasureKind::Linear);
    measure::plant_linear_first_for_test(&ctx, 0);

    press_escape(&ctx, &mut actions);
    assert!(actions.is_empty(), "a single point authors nothing");
    assert!(
        measure::read(&ctx).is_none_or(|s| !s.gesture_in_progress()),
        "and the point is gone"
    );
    assert_eq!(
        tool::selected(&ctx),
        tool::CanvasTool::Measure(MeasureKind::Linear),
        "one press, one effect"
    );
}

/// …and a second Escape then retires the zoom, so nothing is stranded.
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
                    field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                    annot_delete_refused: false,
                    field_delete_refused: false,
                    targets: None,
                    edit_epoch: 0,
                    model_attempted: true,
                    page: None,
                    field_page: None,
                    pen: Default::default(),
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

/// An Escape that closes a popup (a colour picker, a menu) leaves the
/// selection where it was.
#[test]
fn an_escape_closing_a_popup_leaves_the_selection_alone() {
    let mut selection = part_entered();
    let ctx = Context::default();
    let mut actions = Vec::new();
    let mut text_selection = None;
    egui::Popup::open_id(&ctx, egui::Id::new("picker"));
    let _ = ctx.run_ui(key(Key::Escape), |ui| {
        crate::canvas::keys::note_popup_at_frame_start(ui.ctx());
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                field_set: &pdfcer_gui_base::fieldset::FieldSet::default(),
                annot_delete_refused: false,
                field_delete_refused: false,
                targets: None,
                edit_epoch: 0,
                model_attempted: true,
                page: None,
                field_page: None,
                pen: Default::default(),
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });
    assert_eq!(selection.level(), SelectionLevel::Part);
}
