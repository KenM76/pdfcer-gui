//! # `canvas::keys` tests — the Delete ladder and the Escape ladder, enumerated
//!
//! Split out of [`super`] under **R2** on 2026-08-28, when the form-field
//! Delete took that file past 1,500 lines.
//!
//! ## ★★ The seam, and why the half left behind is the interesting one
//!
//! [`super`] is **two precedence ladders** — which claimant a Delete reaches,
//! and which a press of Escape does — and each is a short function whose whole
//! content is an ordered list of `if let` arms. The tests are bulky because a
//! ladder is exercised by enumerating it, not by three cases: every rung needs
//! a case that reaches it AND a case that proves the rung above did not swallow
//! it.
//!
//! ⇒ So this is a split between **the ladder** and **the enumeration of the
//! ladder**, a subject boundary rather than a size-driven cut — the same one
//! `gesture::meaning` took three commits earlier.

// ★★ The INNER attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and without it every assertion message here
// is reported as operator-facing copy — 28 of them, the last time a test module
// was split under R2.
#![cfg(test)]

use super::*;
use crate::canvas::selection::ClickHit;
use crate::canvas::target::TargetId;
use egui::{Context, Event, Modifiers, RawInput};

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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
                escape_consumed: false,
            },
            selection,
            &mut text_selection,
            &mut actions,
        );
    });
    actions
}

/// ★★★ **Delete removes a selected FORM FIELD, and it outranks the other
/// two claimants.**
///
/// `OPERATOR_REQUESTS.md` **O53**. Delete did not reach a form field at all:
/// this ladder never had `doc.selected_field` in front of it, because a
/// widget is deliberately not an annotation selection and the form surface
/// owns those presses.
///
/// ★★ The assertion is a **comparison**, not a presence check. A build that
/// raised the field deletion *and* fell through to the content one would
/// satisfy "did it raise the field action?" and would delete the operator's
/// page content as well — silently, because the field deletion they asked
/// for did happen. Exactly one action, and it is the right one.
#[test]
fn delete_removes_a_selected_form_field_and_nothing_else() {
    use crate::app::actions::forms::FieldAction;
    use crate::app::state::SelectedField;

    let field = SelectedField {
        field: "Check1".to_owned(),
        widget: 0,
        page: 0,
    };
    // ★ A CONTENT selection is live at the same time, which is the state
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
                caps: Capabilities::FULL,
                selected_field: Some(&field),
                annot_delete_refused: false,
                field_delete_refused: false,
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

/// ★★★ **Delete does NOT act on a form field the engine would refuse, and it
/// does not fall through to the content rung either.**
///
/// # What this pins, and why the shape of the failure is the point
///
/// Rung 0 above was added on 2026-08-28 with **no gate at all** — it pushed
/// `DeleteWidget` on `caps.edit_content && selected_field` and returned six
/// lines above the annotation branch that does ask one. So the R83 pass that
/// closed the annotation rung on 2026-08-29 walked past this rung without
/// seeing it, because a rung that asks nothing looks like a rung with nothing
/// to ask.
///
/// The consequence was not a harmless no-op. `actions::forms::delete_widget`
/// cleared `doc.selected_field` **before** calling the engine, so on an
/// ordinary certified fillable form the press produced:
///
/// 1. an action raised for a verb that would refuse,
/// 2. a refusal into `actions::apply::vector_edit`'s `Err` arm — a trace line
///    and, by that arm's own recorded decision, nothing to the operator,
/// 3. **and the selection cleared anyway**, which blanked the Properties
///    panel's `formfield` section — the one surface that was correctly saying
///    *"This document does not allow form fields to be removed."*
///
/// ⇒ A silence that also destroys the sentence explaining it. `actions` being
/// empty pins step 1, which is the only one of the three this ladder can
/// prevent — and preventing it prevents all three.
///
/// ★★ A **content** selection is live at the same time, deliberately, and that
/// is what makes the second assertion evidence rather than decoration: a build
/// that declined the field rung by *falling through* instead of returning would
/// delete the page objects underneath the widget. Refusing one verb is never a
/// licence to run a different one.
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
                caps: Capabilities::FULL,
                selected_field: Some(&field),
                annot_delete_refused: false,
                field_delete_refused: true,
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

/// ★★ **The same press with the field gate open raises the delete**, which is
/// what makes the test above evidence rather than a tautology.
///
/// A rung that declined unconditionally would satisfy every assertion above
/// perfectly, and would be a strictly worse defect than the one being fixed: a
/// control withheld where it would have worked leaves the operator no gesture
/// that reports it. This is the other half every rung in this file is required
/// to carry.
///
/// ★ It is `delete_removes_a_selected_form_field_and_nothing_else` above with
/// one field flipped, and it is written separately rather than folded into it
/// because that test is about **precedence** and this one is about the
/// **gate**. Two questions, two failures worth telling apart.
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
                caps: Capabilities::FULL,
                selected_field: Some(&field),
                annot_delete_refused: false,
                field_delete_refused: false,
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

/// ★ **Click, then Delete — the sequence `DEFECTS.md` D1 is about.**
///
/// D1's own words: *"I can't even click on an object and delete it by
/// hitting the delete key."* `app::keyboard` proves the key survives a
/// canvas click; this proves the key now reaches a verb.
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

/// ★ **Delete inside an object deletes NOTHING, rather than the object.**
///
/// The destructive wrong action this stage must not ship: the selection
/// names a subpath, the only wired verb removes whole objects, and one
/// measured CAD export holds an entire drawing view as a single path
/// object with 1,194 subpaths. "They can undo it" is not an answer to a
/// keypress that removes a drawing.
#[test]
fn delete_inside_an_object_refuses_rather_than_deleting_the_object() {
    let mut selection = part_entered();
    assert_eq!(selection.level(), SelectionLevel::Part);
    assert!(
        keys_for(key(Key::Delete), &mut selection).is_empty(),
        "the Part rung has no delete verb wired, and must not borrow the Object rung's"
    );
    assert_eq!(selection.len(), 1, "and the selection is left alone");
}

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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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

/// ★ **A focused text field keeps its Delete key** — the guard D1 is
/// about, asserted in the direction that matters for correctness.
///
/// `app::keyboard`'s regression test proves the *other* direction: a
/// focused NON-text widget must not suppress the key. Both are needed.
/// This one builds a real `TextEdit` and focuses it, because
/// `text_edit_focused()` resolves the focused id and looks for a
/// `TextEditState` under it — a hand-requested focus on a bare id would
/// pass vacuously.
#[test]
fn a_focused_text_field_keeps_delete_for_itself() {
    let ctx = Context::default();
    let mut buffer = String::from("x");
    let mut selection = object_selected();
    let mut actions = Vec::new();
    let mut text_selection = None;

    // Frame 1: build the field and take focus.
    let _ = ctx.run_ui(RawInput::default(), |ui| {
        ui.add(egui::TextEdit::singleline(&mut buffer))
            .request_focus();
    });
    // Frame 2: the field holds focus; Delete belongs to it.
    let mut typing = false;
    let _ = ctx.run_ui(key(Key::Delete), |ui| {
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                    caps: Capabilities::FULL,
                    selected_field: None,
                    annot_delete_refused: false,
                    field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: false,
                field_delete_refused: false,
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
                    caps: Capabilities::FULL,
                    selected_field: None,
                    annot_delete_refused: false,
                    field_delete_refused: false,
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

/// ★★★ **Delete does NOT act on an annotation the engine would refuse**, and
/// nothing at all is raised.
///
/// # What this pins, and why the shape of the failure matters more than the rung
///
/// This rung read `if annot.target.locked` and nothing else until 2026-08-29 —
/// one of the **three** things that refuse an annotation delete. `/Encrypt` and
/// an enforced certification signature were not asked, because
/// `EditSession::annotation_deletion_refusal` — a pure query whose own doc
/// comment names the call site by rule number — was called by nothing in this
/// shell.
///
/// The consequence was not a harmless no-op. `actions::annots::delete` clears
/// the annotation selection **after** the funnel rather than on success, so on a
/// certified drawing the press produced:
///
/// 1. an action raised for a verb that would refuse,
/// 2. a refusal into `actions::apply::vector_edit`'s `Err` arm — a trace line
///    and, by that arm's own recorded decision, nothing to the operator,
/// 3. **and the selection cleared anyway**, taking the Properties panel's
///    explanation off the screen with it.
///
/// ⇒ A silence that also destroys the sentence explaining it. Asserting
/// `actions.is_empty()` pins step 1, which is the only one of the three this
/// ladder can prevent — and preventing it prevents all three.
///
/// ★ Asserted with the annotation **unlocked**, deliberately. A locked
/// annotation would be refused by the older half of the gate, so the test would
/// pass on the code this fixes and prove nothing.
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
    });

    let ctx = Context::default();
    let mut actions = Vec::new();
    let mut text_selection = None;
    let _ = ctx.run_ui(key(Key::Delete), |ui| {
        canvas_keys(
            Keys {
                ctx: ui.ctx(),
                page_index: 0,
                caps: Capabilities::FULL,
                selected_field: None,
                annot_delete_refused: true,
                field_delete_refused: false,
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

/// ★★ **The same press with the gate open raises the delete**, which is what
/// makes the test above evidence rather than a tautology.
///
/// A rung that declined unconditionally would satisfy the assertion above
/// perfectly. This is the other half every rung in this file is required to
/// carry: a case that reaches it, beside the case that proves it was not
/// swallowed.
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
