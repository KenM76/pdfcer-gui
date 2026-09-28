//! # `canvas::keys` tests — the Delete ladder enumerated, and the two keys
//! # that only pass through
//!
//! ## The seam, and why the half left behind is the interesting one
//!
//! [`super`] is **two precedence ladders** — which claimant a Delete reaches,
//! and which a press of Escape does — and each is a short function whose whole
//! content is an ordered list of `if let` arms. The tests are bulky because a
//! ladder is exercised by enumerating it, not by three cases: every rung needs
//! a case that reaches it AND a case that proves the rung above did not swallow
//! it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/keys/tests.md`.

#![cfg(test)]

use super::*;
use crate::app::actions::VectorAction;
use crate::canvas::pick::PickFilter;
use crate::canvas::selection::{ClickHit, SelectionLevel};
use crate::canvas::target::TargetId;
use egui::{Context, Event, Modifiers, RawInput};

/// The Escape ladder's enumeration, which is the longer of the two and the
/// one with claimants outside this module (a drag, an armed region zoom, a
/// markup tool, a guide, a fit). It takes its fixtures from here through
/// `use super::*`; everything above is shared by both ladders.
mod escape_ladder;

/// A selection holding one whole object on page 0.
fn object_selected() -> SelectionState {
    let mut selection = SelectionState::default();
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(3)),
            ..ClickHit::default()
        },
        false,
        false,
    );
    selection
}

/// …and the same one, descended a rung into part 1.
fn part_entered() -> SelectionState {
    let mut selection = object_selected();
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(3)),
            part: Some(1),
            node: None,
            chunk: false,
        },
        false,
        true,
    );
    selection
}

/// `RawInput` carrying one unmodified key press.
fn key(key: Key) -> RawInput {
    RawInput {
        events: vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    }
}

/// Run [`canvas_keys`] for one frame against a real `egui::Context`.
fn keys_for(input: RawInput, selection: &mut SelectionState) -> Vec<Action> {
    let ctx = Context::default();
    let mut actions = Vec::new();
    let mut text_selection = None;
    let _ = ctx.run_ui(input, |ui| {
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
            selection,
            &mut text_selection,
            &mut actions,
        );
    });
    actions
}

/// **AN ARROW KEY REACHES THE NUDGE THROUGH THIS FUNCTION.**
#[test]
fn an_arrow_key_reaches_the_nudge_through_canvas_keys() {
    use crate::app::actions::annot::AnnotAction;
    use crate::canvas::selection::{AnnotKind, AnnotSelection, AnnotTarget};

    let page = pdfcer_core::page_tree::Page::with_boxes(
        pdfcer_core::object::ObjId::new(1, 0),
        pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
        pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
        0,
    );
    let mut selection = SelectionState::default();
    selection.select_annot(AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id: pdfcer_core::object::ObjId::new(7, 0),
            kind: AnnotKind::Markup,
            // ui-text-exempt: a PDF /Subtype name in a test fixture.
            subtype: "Square".to_owned(),
            locked: false,
        },
        outline: egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(40.0, 30.0)),
        oriented: None,
    });

    let ctx = Context::default();
    let mut actions = Vec::new();
    let mut text_selection = None;
    let _ = ctx.run_ui(key(Key::ArrowUp), |ui| {
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
                page: Some(&page),
                escape_consumed: false,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });

    let [Action::Annot(AnnotAction::Move { dx, dy, .. })] = actions.as_slice() else {
        panic!("an arrow key must reach the nudge through this function: {actions:?}");
    };
    assert!(dx.abs() < 1e-6, "one axis only: dx={dx}");
    assert!(*dy > 0.0, "Up is a positive dy in PDF user space: dy={dy}");
}

/// **Delete removes a selected FORM FIELD, and it outranks the other
/// two claimants.**
#[test]
fn delete_removes_a_selected_form_field_and_nothing_else() {
    use crate::app::actions::forms::FieldAction;
    use crate::app::state::SelectedField;

    let field = SelectedField {
        field: "Check1".to_owned(),
        widget: 0,
        page: 0,
    };
    // A CONTENT selection is live at the same time, which is the state
    // that makes the precedence testable: without it the content branch
    // would raise nothing anyway and the test would pass on a build with no
    // precedence at all.
    let mut selection = object_selected();
    let mut text_selection = None;
    let mut actions = Vec::new();
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(key(Key::Delete), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: Some(&field),
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
    assert_eq!(actions.len(), 1, "{actions:?}");
    assert!(
        matches!(
            &actions[0],
            Action::Field(FieldAction::DeleteWidget { field, widget })
                if field == "Check1" && *widget == 0
        ),
        "Delete did not remove the selected form field: {actions:?}"
    );
}

/// **Delete does NOT act on a form field the engine would refuse, and it
/// does not fall through to the content rung either.**
#[test]
fn delete_does_not_act_on_a_form_field_whose_deletion_would_be_refused() {
    use crate::app::state::SelectedField;

    let field = SelectedField {
        field: "Check1".to_owned(),
        widget: 0,
        page: 0,
    };
    let mut selection = object_selected();
    let mut text_selection = None;
    let mut actions = Vec::new();
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(key(Key::Delete), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: Some(&field),
                annot_delete_refused: false,
                field_delete_refused: true,
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
        actions.is_empty(),
        "the ladder must stop at the form-field rung rather than fall through to \
         the content rung: a refused widget delete is not a licence to delete the \
         page objects underneath it — {actions:?}"
    );
    assert!(
        !selection.is_empty(),
        "nothing may clear a selection here. The Properties panel's sentence \
         explaining the refusal is drawn from `doc.selected_field`, and losing a \
         selection to a delete that did not happen is how the refusal became a \
         silence in the first place"
    );
}

/// **The same press with the field gate open raises the delete**, which is
/// what makes the test above evidence rather than a tautology.
#[test]
fn delete_acts_on_a_form_field_when_the_gate_is_open() {
    use crate::app::actions::forms::FieldAction;
    use crate::app::state::SelectedField;

    let field = SelectedField {
        field: "Check1".to_owned(),
        widget: 2,
        page: 0,
    };
    let mut selection = object_selected();
    let mut text_selection = None;
    let mut actions = Vec::new();
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(key(Key::Delete), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: Some(&field),
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

    assert_eq!(actions.len(), 1, "{actions:?}");
    assert!(
        matches!(
            &actions[0],
            Action::Field(FieldAction::DeleteWidget { field, widget })
                if field == "Check1" && *widget == 2
        ),
        "an open gate must still delete THIS box, named by its widget index: {actions:?}"
    );
}

/// **Click, then Delete — the sequence `DEFECTS.md` D1 is about.**
#[test]
fn delete_with_an_object_selected_raises_the_delete_action() {
    let mut selection = object_selected();
    assert_eq!(
        keys_for(key(Key::Delete), &mut selection),
        vec![
            VectorAction::DeleteSelection {
                page: 0,
                objects: vec![3],
            }
            .into()
        ]
    );

    // Backspace is bound too — a laptop without a Delete key is the
    // common case.
    let mut selection = object_selected();
    assert_eq!(keys_for(key(Key::Backspace), &mut selection).len(), 1);
}

/// With nothing selected, Delete raises nothing rather than an empty
/// batch the engine would have to refuse.
#[test]
fn delete_with_nothing_selected_raises_nothing() {
    let mut selection = SelectionState::default();
    assert!(keys_for(key(Key::Delete), &mut selection).is_empty());
}

/// **Delete inside an object NEVER borrows the Object rung's verb.**
#[test]
fn delete_inside_an_object_never_borrows_the_object_rungs_verb() {
    let mut selection = part_entered();
    assert_eq!(selection.level(), SelectionLevel::Part);
    let raised = keys_for(key(Key::Delete), &mut selection);
    assert!(
        !raised
            .iter()
            .any(|a| matches!(a, Action::Vector(VectorAction::DeleteSelection { .. }))),
        "the Part rung must never raise the whole-object delete"
    );
    assert!(
        raised.is_empty(),
        "and with no object model it raises nothing at all, because it cannot \
         tell a subpath from a label"
    );
    assert_eq!(selection.len(), 1, "and the selection is left alone");
}

/// **A focused text field keeps its Delete key** — the guard D1 is
/// about, asserted in the direction that matters for correctness.
#[test]
fn a_focused_text_field_keeps_delete_for_itself() {
    let ctx = Context::default();
    let mut buffer = String::from("x");
    let mut selection = object_selected();
    let mut actions = Vec::new();
    let mut text_selection = None;

    // Frame 1: build the field and take focus.
    let _ = ctx.run_ui(RawInput::default(), |ui| {
        // escape-disposition: not-a-surface — a field this test builds to put egui
        // into a known focus state. It is never drawn for an operator.
        ui.add(egui::TextEdit::singleline(&mut buffer))
            .request_focus();
    });
    // Frame 2: the field holds focus; Delete belongs to it.
    let mut typing = false;
    let _ = ctx.run_ui(key(Key::Delete), |ui| {
        // escape-disposition: not-a-surface — a field this test builds to put egui
        // into a known focus state. It is never drawn for an operator.
        ui.add(egui::TextEdit::singleline(&mut buffer));
        // typing-guard-exempt: a TEST asserting the harness actually reached
        // the focused state. Reading the raw egui answer is the point - a
        // test that asked `composing()` could not tell a focused widget from
        // a canvas draft, and the thing being proved is that the widget half
        // is reachable at all. D1 shipped because its test could not reach it.
        typing = ui.ctx().text_edit_focused();
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
        typing,
        "the test is vacuous unless a TEXT field really holds focus"
    );
    assert!(
        actions.is_empty(),
        "a focused text field must keep Delete for itself"
    );
    assert_eq!(selection.len(), 1);
}

/// **Delete does NOT act on an annotation the engine would refuse**, and
/// nothing at all is raised.
#[test]
fn delete_does_not_act_on_an_annotation_whose_deletion_would_be_refused() {
    let mut selection = SelectionState::default();
    selection.select_annot(crate::canvas::selection::annot::AnnotSelection {
        target: crate::canvas::selection::annot::AnnotTarget {
            page: 0,
            id: pdfcer_core::object::ObjId::new(12, 0),
            kind: crate::canvas::selection::annot::AnnotKind::Markup,
            subtype: "Square".to_owned(),
            locked: false,
        },
        outline: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(10.0, 10.0)),
        oriented: None,
    });

    let ctx = Context::default();
    let mut actions = Vec::new();
    let mut text_selection = None;
    let _ = ctx.run_ui(key(Key::Delete), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                pick: PickFilter::all(),
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: true,
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
        actions.is_empty(),
        "the ladder must stop at the annotation rung rather than fall through to \
         the content rung: a refused annotation delete is not a licence to delete \
         the page objects underneath it"
    );
    assert!(
        selection.annot().is_some(),
        "nothing may clear the selection here — the Properties panel's sentence \
         explaining the refusal is drawn from it, and losing it is how the \
         refusal became a silence in the first place"
    );
}

/// **The same press with the gate open raises the delete**, which is what
/// makes the test above evidence rather than a tautology.
#[test]
fn delete_acts_on_an_annotation_when_the_gate_is_open() {
    let mut selection = SelectionState::default();
    let id = pdfcer_core::object::ObjId::new(12, 0);
    selection.select_annot(crate::canvas::selection::annot::AnnotSelection {
        target: crate::canvas::selection::annot::AnnotTarget {
            page: 0,
            id,
            kind: crate::canvas::selection::annot::AnnotKind::Markup,
            subtype: "Square".to_owned(),
            locked: false,
        },
        outline: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(10.0, 10.0)),
        oriented: None,
    });

    let actions = keys_for(key(Key::Delete), &mut selection);
    assert_eq!(
        actions,
        vec![Action::Annot(
            crate::app::actions::annot::AnnotAction::Delete { page: 0, id }
        )],
        "one action, naming the annotation by object id — `delete_annotation` \
         finds it wherever it lives, so the page travels for the message rather \
         than for the verb"
    );
}

/// **THE TRIPWIRE FIRES.** A Delete declined `NoObjectModel` on a frame
/// that never ASKED for the decomposition panics, loudly, naming the class.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never ASKED for the")]
fn a_delete_declined_for_want_of_asking_is_not_allowed_to_be_quiet() {
    let ctx = Context::default();
    let mut selection = part_entered();
    let mut actions = Vec::new();
    let mut text_selection = None;
    let _ = ctx.run_ui(key(Key::Delete), |ui| {
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
                // The whole point of the case: the frame did not ask.
                model_attempted: false,
                escape_consumed: false,
                page: None,
            },
            &mut selection,
            &mut text_selection,
            &mut actions,
        );
    });
}

/// **A CLAIMED TAB REACHES THE OBJECT RING THROUGH THIS FUNCTION.**
#[test]
fn a_claimed_tab_reaches_the_object_ring_through_canvas_keys() {
    use crate::canvas::tabnav::{self, Scope};

    let ctx = Context::default();
    let id = egui::Id::new("objring-wiring-test");

    // The page's stand-in: one keyboard-only widget that asks for focus and
    // publishes itself, which is exactly what `canvas::pagefocus` does.
    let seat = |ui: &mut egui::Ui| {
        let r = ui.interact(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)),
            id,
            egui::Sense::focusable_noninteractive(),
        );
        r.request_focus();
        tabnav::publish(ui.ctx(), Scope::Object, id);
    };

    // Frame 1 seats the focus. `claim` runs before a frame begins and reads
    // the focus the PREVIOUS frame left, so there has to be a previous one.
    let _ = ctx.run_ui(RawInput::default(), |ui| seat(ui));

    // Frame 2: the hook takes the press off the raw events, then the frame
    // runs and `canvas_keys` is the only thing in it that could spend one.
    let mut raw = key(Key::Tab);
    tabnav::claim(&ctx, &mut raw);
    assert!(
        raw.events.is_empty(),
        "the hook claims the press, or this test is measuring nothing"
    );

    let mut selection = SelectionState::default();
    let mut actions = Vec::new();
    let mut text_selection = None;
    let _ = ctx.run_ui(raw, |ui| {
        seat(ui);
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
        tabnav::take(&ctx, Scope::Object).is_none(),
        "the claimed press was still sitting in the store after the frame that \
         should have spent it — `canvas_keys` is not reaching `objring::advance`"
    );
}
