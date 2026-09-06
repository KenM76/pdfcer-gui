//! # `canvas::modelneed::tests` — the tripwire the compiler cannot be
//!
//! ## ★★★ What these are actually guarding
//!
//! The gesture half is guarded by the **compiler**:
//! [`super::gesture_needs_model`] is an exhaustive `match` with no wildcard, so
//! a new `GestureOutcome` variant stops the build until somebody answers the
//! question for it. Nothing below can improve on that and nothing below should
//! try to.
//!
//! What the compiler cannot guard is the half that is not an enum at all: the
//! **keyboard** term. [`super::Need::delete_at_a_deeper_rung`] is a plain
//! conjunction, and deleting it — or narrowing it back to the Object rung —
//! compiles perfectly and restores the 2026-09-05 defect: Delete at the Part
//! and Node rungs declining `NoObjectModel` because nobody asked for the
//! decomposition, which is three shipped engine verbs reachable by nothing at
//! all, silently.
//!
//! So the tripwire is here, stated per rung, and it names its own subject in
//! the test name so a failure says what broke rather than which line moved.
//!
//! ## `#![cfg(test)]` at the top
//!
//! `check-ui-strings.sh` and `check-theme-colors.sh` both recognise the inner
//! attribute as meaning *"none of this is in the shipped binary"*. Without it
//! every `assert!` message below is reported as un-catalogued operator copy.
//! `check-file-size.sh` still counts these lines; this is the split R2 asks
//! for, not a way of hiding from it.

#![cfg(test)]

use super::{Need, gesture_needs_model};
use crate::canvas::gesture::{GestureOutcome, MarqueeIntent, Phase};
use crate::canvas::handles::Grip;
use crate::canvas::selection::{ClickHit, SelectionLevel, SelectionState};
use crate::canvas::target::TargetId;
use egui::{Pos2, Rect, Vec2};

/// The object the rungs below are built on. Any index; nothing resolves it.
const OBJECT: u64 = 7;

/// A selection sitting at the Object rung — one whole object picked.
fn at_object_rung() -> SelectionState {
    let mut selection = SelectionState::default();
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(OBJECT)),
            part: None,
            node: None,
        },
        false,
        false,
    );
    assert_eq!(selection.level(), SelectionLevel::Object);
    selection
}

/// A selection descended into one part of one object — the Part rung.
///
/// Built through `SelectionState::click` with `double`, exactly as
/// `canvas::deleting::tests` builds its own, so the rung is reached by the
/// ladder's real rule rather than by writing the field.
fn at_part_rung() -> SelectionState {
    let mut selection = SelectionState::default();
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(OBJECT)),
            part: Some(3),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(selection.level(), SelectionLevel::Part);
    selection
}

/// A selection descended to one anchor — the Node rung.
fn at_node_rung() -> SelectionState {
    let mut selection = at_part_rung();
    selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(OBJECT)),
            part: Some(3),
            node: Some(2),
        },
        false,
        true,
    );
    assert_eq!(selection.level(), SelectionLevel::Node);
    selection
}

/// The frame that does nothing at all: no gesture, no click, no key.
fn quiet<'a>(selection: &'a SelectionState, idle: &'a GestureOutcome) -> Need<'a> {
    Need {
        outcome: idle,
        secondary_clicked: false,
        measure_armed: false,
        delete_pressed: false,
        selection,
    }
}

/// ★★★ **THE TRIPWIRE.** Delete at a deeper rung must ask for the model.
///
/// This is the fourth recurrence of one defect, and the first that could not
/// have been fixed by adding a variant to a list of gesture outcomes — Delete
/// is a keystroke, so there was no variant to add. If this test goes red, the
/// symptom in the running window is:
///
/// ```text
/// canvas-delete-declined level=Part sel=1 reason=NoObjectModel asked=false
/// ```
///
/// …the operator presses Delete on a selected line or label and **nothing
/// happens**, exactly as it did before 2026-09-05.
#[test]
fn a_delete_at_a_deeper_rung_asks_for_the_object_model() {
    let idle = GestureOutcome::Idle;
    for (rung, selection) in [("Part", at_part_rung()), ("Node", at_node_rung())] {
        let need = Need {
            delete_pressed: true,
            ..quiet(&selection, &idle)
        };
        assert!(
            need.delete_at_a_deeper_rung(),
            "a Delete at the {rung} rung must ask for the decomposition: \
             `canvas::deleting::subject` needs it to tell a subpath from a show operator, \
             and without it declines `NoObjectModel` — which reads, from the operator's \
             chair, as a key that does nothing"
        );
        assert!(
            need.wanted(),
            "the {rung} rung's Delete term must reach `wanted()`, not merely exist"
        );
    }
}

/// The Object rung answers from the selection alone and must NOT pay for a
/// decomposition.
///
/// ★ The other half of the tripwire above, and it is what stops a future
/// session "fixing" the first one by asking unconditionally. On the operator's
/// benchmark drawing a decomposition is **531 ms / 129,758 objects**, and
/// `canvas::deleting`'s Object arm never reads it.
#[test]
fn a_delete_at_the_object_rung_pays_for_nothing() {
    let idle = GestureOutcome::Idle;
    let selection = at_object_rung();
    let need = Need {
        delete_pressed: true,
        ..quiet(&selection, &idle)
    };
    assert!(!need.delete_at_a_deeper_rung());
    assert!(
        !need.wanted(),
        "a Delete on whole objects must not decompose the page: the operand list is a \
         filter over four integers, and a page that will not decompose must stay deletable \
         at the rung where deletion needs no decomposition at all"
    );
}

/// A deeper rung on its own — with no key pressed — asks for nothing.
///
/// ★ This is the choice the measurement decided. *"Ask whenever a deeper rung
/// is selected"* would be correct and would pay 531 ms, after each content
/// edit, on frames nothing reads the model — while the operator merely holds
/// the selection. See the module header.
#[test]
fn merely_standing_at_a_deeper_rung_asks_for_nothing() {
    let idle = GestureOutcome::Idle;
    let selection = at_part_rung();
    assert!(!quiet(&selection, &idle).wanted());
}

/// The two non-gesture terms that were already there, restated so a refactor
/// that drops one is red rather than quiet.
#[test]
fn the_pre_existing_non_gesture_terms_still_ask() {
    let idle = GestureOutcome::Idle;
    let selection = at_object_rung();

    let right_click = Need {
        secondary_clicked: true,
        ..quiet(&selection, &idle)
    };
    assert!(
        right_click.wanted(),
        "a right-click must know what is under the pointer, or the menu is about the wrong \
         object — which is worse than no menu"
    );

    let measuring = Need {
        measure_armed: true,
        ..quiet(&selection, &idle)
    };
    assert!(
        measuring.wanted(),
        "an armed measure tool needs the model on EVERY frame, not only on the frame of a \
         click: the snap indicator has to appear while the operator is still deciding where \
         to click"
    );
}

/// The gesture answers, per variant, including the three whose absence each
/// cost a driving session.
#[test]
fn the_gestures_that_read_the_page_ask_for_it() {
    let asks = [
        GestureOutcome::Click {
            point: Pos2::ZERO,
            shift: false,
            double: false,
            triple: false,
        },
        GestureOutcome::Move {
            delta: Vec2::ZERO,
            phase: Phase::Complete,
        },
        GestureOutcome::Resize {
            grip: Grip::SouthEast,
            delta: Vec2::ZERO,
            phase: Phase::Complete,
        },
        GestureOutcome::Handle {
            node: 0,
            handle: pdfcer_core::vector::Handle::Outgoing,
            at: Pos2::ZERO,
            phase: Phase::Complete,
        },
        GestureOutcome::DimensionVertex {
            index: 0,
            from: Pos2::ZERO,
            at: Pos2::ZERO,
            phase: Phase::Complete,
        },
        GestureOutcome::Rotate {
            from: Pos2::ZERO,
            at: Pos2::ZERO,
            phase: Phase::Complete,
        },
    ];
    for outcome in &asks {
        assert!(
            gesture_needs_model(outcome),
            "{outcome:?} reads the page's object model and must ask for it. Three of these \
             shipped without asking, and each presented as `the gesture does nothing`"
        );
    }
}

/// A **zoom** marquee decomposes nothing, and an in-flight select marquee has
/// resolved nothing yet.
///
/// ★ The zoom half is the concrete payoff for carrying the intent on the
/// outcome: a region zoom over a 129,758-object drawing costs one scroll
/// offset, not 531 ms.
#[test]
fn a_zoom_marquee_decomposes_nothing() {
    let rect = Rect::from_two_pos(Pos2::ZERO, Pos2::new(10.0, 10.0));
    let marquee = |intent, phase| GestureOutcome::Marquee {
        rect,
        crossing: false,
        shift: false,
        intent,
        phase,
    };
    assert!(!gesture_needs_model(&marquee(
        MarqueeIntent::Zoom,
        Phase::Complete
    )));
    assert!(!gesture_needs_model(&marquee(
        MarqueeIntent::Select,
        Phase::InFlight
    )));
    assert!(
        gesture_needs_model(&marquee(MarqueeIntent::Select, Phase::Complete)),
        "a released SELECT marquee resolves what it enclosed and needs the model to do it"
    );
}
