//! # `canvas::moving::nudge` tests — the step, the sign, and the guard that
//! must not be got wrong
//!
//! Split into a file of its own on the module's first day, following
//! [`super::super::tests`]' seam: **the rule and the enumeration of the rule are
//! different subjects**, and a nudge is enumerated the way a ladder is — one
//! case per refusal, plus a case per direction.
//!
//! ## ★★★ THE TEST THAT WOULD HAVE CAUGHT THE FOUNDING DEFECT
//!
//! [`a_real_focused_text_field_keeps_its_arrow_keys`] builds an actual
//! [`egui::TextEdit`], focuses it, and **asserts that
//! `Context::text_edit_focused()` is genuinely `true`** before asserting that no
//! nudge was raised.
//!
//! That order is the whole point and it is not decoration. `DEFECTS.md` D1 —
//! the Delete key silently dying after any canvas click — survived because *"the
//! only test of it built a bare `egui::Context` with no widgets, so the
//! condition that breaks the real application cannot occur in the harness"*. A
//! test that presses an arrow into a `Context` with nothing focused passes
//! whatever the guard says; it is not evidence, it is the shape of the original
//! defect's evidence.
//!
//! ⇒ So this file's guard tests all take the same form: **prove the harness
//! reached the state, then prove the code answered it.** The first assertion is
//! the one that makes the second mean anything.
//!
//! ## ★★ …and the SECOND claimant, which egui cannot see at all
//!
//! [`a_canvas_draft_keeps_its_arrow_keys`] stores a real
//! [`crate::canvas::textedit::Draft`] and asserts `text_edit_focused()` is
//! **false** while `composing()` is **true** — the exact gap that cost the
//! operator the space bar on 2026-08-20, when a guard asked egui alone. An
//! arrow key is where that gap costs most: it is how a caret is moved.

// ★★ The INNER attribute, not just the `mod tests;` in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a test file **from the file**
// rather than from its name; without it every `assert!` message below is
// reported as un-catalogued operator copy.
#![cfg(test)]

use super::*;
use crate::canvas::selection::{AnnotSelection, AnnotTarget, ClickHit};
use crate::canvas::target::{StubTargets, TargetId};
use egui::{Event, Key, Modifiers, RawInput};
use pdfcer_core::object::{Dict, ObjId};
use pdfcer_core::page_tree::Rect as PageRect;

/// The annotation every nudge case acts on.
const MARK: ObjId = ObjId::new(7, 0);

/// A minimal page fixture — the same one [`super::super::tests`] uses, because
/// [`super::super::page_delta`] reads exactly what those tests exercise:
/// `crop_box` and `rotate`.
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

/// A selection holding one annotation.
fn annot_selection(kind: AnnotKind, locked: bool) -> SelectionState {
    let mut state = SelectionState::default();
    state.select_annot(AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id: MARK,
            kind,
            // ui-text-exempt: a PDF /Subtype name in a test fixture.
            subtype: "Square".to_owned(),
            locked,
        },
        outline: egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(40.0, 30.0)),
    });
    state
}

/// A selection holding a **page-content** object — the case that is refused
/// with a sentence rather than in silence.
fn object_selection() -> SelectionState {
    let mut state = SelectionState::default();
    state.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            ..ClickHit::default()
        },
        false,
        false,
    );
    state.resolve(
        Some(&StubTargets::new(
            0,
            [egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(100.0, 100.0),
            )],
        )),
        0,
        0,
    );
    state
}

/// `RawInput` carrying `count` presses of one key.
///
/// ★ `count` rather than a `bool`, because a held arrow arrives as several
/// `Event::Key { pressed: true, repeat: true }` in one frame and *"one undo
/// entry per keypress"* is a claim about exactly that.
fn arrows(key: Key, modifiers: Modifiers, count: usize) -> RawInput {
    RawInput {
        events: (0..count)
            .map(|i| Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: i > 0,
                modifiers,
            })
            .collect(),
        modifiers,
        ..Default::default()
    }
}

/// Run one frame and return whatever the nudge raised in it.
fn nudged(
    ctx: &egui::Context,
    input: RawInput,
    page: Option<&Page>,
    caps: Capabilities,
    selection: &SelectionState,
) -> Vec<Action> {
    let mut out = Vec::new();
    let _ = ctx.run_ui(input, |ui| {
        keys(
            ui.ctx(),
            &Frame {
                page,
                caps,
                edit_epoch: 0,
            },
            selection,
            &mut out,
        );
    });
    out
}

/// The `(dx, dy)` of the single move an action list holds.
#[track_caller]
fn one_move(actions: &[Action]) -> (f64, f64) {
    assert_eq!(actions.len(), 1, "expected exactly one move: {actions:?}");
    match &actions[0] {
        Action::Annot(AnnotAction::Move { id, dx, dy }) => {
            assert_eq!(*id, MARK, "the move must name the selected mark");
            (*dx, *dy)
        }
        other => panic!("expected a markup move, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// The Y sign
// ---------------------------------------------------------------------------

/// ★★★ **Up moves the mark UP the page.**
///
/// PDF user space has y increasing upward from the bottom-left (§8.3.2.3) and
/// canvas space has it increasing downward, so this asserts a positive `dy` for
/// the key labelled Up — the one assertion whose failure would be invisible in
/// review and obvious in one second of use.
#[test]
fn the_up_arrow_moves_the_mark_up_the_page() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        Some(&page),
        Capabilities::FULL,
        &selection,
    );
    let (dx, dy) = one_move(&actions);
    assert!(
        (dx - 0.0).abs() < 1e-6,
        "an Up arrow moves along one axis only: dx={dx}"
    );
    assert!(
        (dy - f64::from(STEP_PT)).abs() < 1e-6,
        "y is UP in PDF user space, so Up is a POSITIVE dy: dy={dy}"
    );
}

/// The other three directions, so the sign table is enumerated rather than
/// inferred from one case.
#[test]
fn every_arrow_moves_one_point_in_its_own_direction() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);
    let step = f64::from(STEP_PT);
    for (key, want) in [
        (Key::ArrowUp, (0.0, step)),
        (Key::ArrowDown, (0.0, -step)),
        (Key::ArrowLeft, (-step, 0.0)),
        (Key::ArrowRight, (step, 0.0)),
    ] {
        let actions = nudged(
            &ctx,
            arrows(key, Modifiers::NONE, 1),
            Some(&page),
            Capabilities::FULL,
            &selection,
        );
        let got = one_move(&actions);
        assert!(
            (got.0 - want.0).abs() < 1e-6 && (got.1 - want.1).abs() < 1e-6,
            "{key:?} should move {want:?}, moved {got:?}"
        );
    }
}

/// ★★★ **A quarter-turned page nudges in the direction the operator pressed.**
///
/// The claim that makes routing through [`super::super::page_delta`] worth the
/// parameter: on a page carrying `/Rotate 90`, screen-up is page-**left**, so an
/// Up arrow must produce a displacement along **x** and not along y. A nudge
/// that hard-coded `dy = +1` passes every test above and moves a mark sideways
/// on the first landscape drawing it meets.
#[test]
fn a_quarter_turned_page_nudges_along_the_axis_the_operator_sees() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 90);
    let selection = annot_selection(AnnotKind::Markup, false);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        Some(&page),
        Capabilities::FULL,
        &selection,
    );
    let (dx, dy) = one_move(&actions);
    assert!(
        dx.abs() > 0.9 && dy.abs() < 1e-6,
        "on a /Rotate 90 page, screen-up is a page-space X displacement: dx={dx} dy={dy}"
    );
    // …and it is still exactly one point of travel, whichever axis it landed on.
    assert!(
        (dx.hypot(dy) - f64::from(STEP_PT)).abs() < 1e-6,
        "the step is a distance, not a coordinate: {dx},{dy}"
    );
}

// ---------------------------------------------------------------------------
// The step and the modifiers
// ---------------------------------------------------------------------------

/// **Ctrl gives the smaller step** — Acrobat's convention, and the whole of the
/// modifier vocabulary this gesture claims.
#[test]
fn ctrl_gives_the_finer_step() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::COMMAND, 1),
        Some(&page),
        Capabilities::FULL,
        &selection,
    );
    let (_, dy) = one_move(&actions);
    assert!(
        (dy - f64::from(FINE_STEP_PT)).abs() < 1e-6,
        "Ctrl is the FINE step, not a multiplier: dy={dy}"
    );
    // ★ A `const` assertion, which is what clippy asks for and what this claim
    // actually is: the direction of the modifier is a decision baked into two
    // constants, not a run-time property. Acrobat's modifier makes the step
    // SMALLER; a bigger one would be Illustrator's Shift, which this canvas has
    // already spent on "constrain to one axis". Reversing the two constants is a
    // compile failure rather than a red test.
    const {
        assert!(FINE_STEP_PT < STEP_PT);
    }
}

/// ★★★ **Shift and Alt raise nothing, and Alt is the one that would have been a
/// live defect.**
///
/// `Alt+Up` and `Alt+Down` are bound in the built-in keymap to
/// `pages.move_up` / `pages.move_down`. `egui`'s `consume_key` matches with
/// `Modifiers::matches_logically`, which **ignores extra Shift and Alt** — so a
/// nudge written the obvious way would have reordered the page *and* moved the
/// mark from one press, and no test injecting a bare arrow could have seen it.
///
/// Shift is refused for a different reason and the same effect: it already means
/// *constrain to one axis* on this canvas, three gestures over.
///
/// ★ Falsified: deleting the `shift || alt` line in [`super::step_for`] turns
/// both halves of this red.
#[test]
fn shift_and_alt_are_not_this_gestures_to_take() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);
    for modifiers in [
        Modifiers::SHIFT,
        Modifiers::ALT,
        Modifiers::COMMAND.plus(Modifiers::SHIFT),
        Modifiers::COMMAND.plus(Modifiers::ALT),
    ] {
        let actions = nudged(
            &ctx,
            arrows(Key::ArrowUp, modifiers, 1),
            Some(&page),
            Capabilities::FULL,
            &selection,
        );
        assert!(
            actions.is_empty(),
            "{modifiers:?}+Up is not this gesture's chord — Alt belongs to pages.move_up and \
             Shift means 'one axis' everywhere else on this canvas"
        );
    }
}

/// **One undo entry per keypress, auto-repeat included.**
///
/// Three press events in one frame raise three moves of one point, not one move
/// of three. That is what Illustrator, InDesign and Acrobat all do, and the
/// alternative needs a notion of *gesture end* a keyboard does not offer.
#[test]
fn a_held_arrow_raises_one_move_per_repeat() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 3),
        Some(&page),
        Capabilities::FULL,
        &selection,
    );
    assert_eq!(actions.len(), 3, "one entry per press: {actions:?}");
    for action in &actions {
        match action {
            Action::Annot(AnnotAction::Move { dy, .. }) => assert!(
                (dy - f64::from(STEP_PT)).abs() < 1e-6,
                "each repeat is one whole step, not a share of one"
            ),
            other => panic!("expected a markup move, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// ★★★ The guard — both claimants, each proved reachable first
// ---------------------------------------------------------------------------

/// ★★★ **A REAL focused text field keeps its arrow keys.**
///
/// This is the test `DEFECTS.md` D1 did not have. It builds an actual
/// [`egui::TextEdit`], focuses it, and asserts
/// `Context::text_edit_focused()` is **genuinely true** before asserting that
/// nothing was nudged — because a test that presses an arrow into a bare
/// `Context` with no widgets passes whatever the guard says, which is precisely
/// how the founding defect shipped.
///
/// ★ Falsified: replacing [`crate::canvas::textedit::composing`] with a
/// constant `false` in [`super::keys`] turns this red on the *second* assertion
/// while the first stays green — which is the proof that the first assertion is
/// doing its job.
#[test]
fn a_real_focused_text_field_keeps_its_arrow_keys() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);
    let mut buffer = String::from("x"); // ui-text-exempt: a test fixture's edit buffer.
    let field = egui::Id::new("a-real-text-edit");

    // Frame 1: draw a real `TextEdit` and give it focus. Drawing it is what
    // stores the `TextEditState` that `text_edit_focused` looks for — a bare
    // `request_focus` on an id would leave the predicate false and the test
    // vacuous.
    let _ = ctx.run_ui(RawInput::default(), |ui| {
        let response = ui.add(egui::TextEdit::singleline(&mut buffer).id(field));
        response.request_focus();
    });

    // Frame 2: the field still has focus, and an arrow arrives.
    let mut text_focused = false;
    let mut composing_now = false;
    let mut actions = Vec::new();
    let _ = ctx.run_ui(arrows(Key::ArrowUp, Modifiers::NONE, 1), |ui| {
        ui.add(egui::TextEdit::singleline(&mut buffer).id(field));
        // typing-guard-exempt: a TEST asserting the harness actually reached the
        // focused state. Reading egui's own answer is the point — a test that
        // asked `composing()` here could not tell a focused widget from a canvas
        // draft, and what is being proved is that the WIDGET half is reachable
        // at all. D1 shipped because its test could not reach it.
        text_focused = ui.ctx().text_edit_focused();
        composing_now = crate::canvas::textedit::composing(ui.ctx());
        keys(
            ui.ctx(),
            &Frame {
                page: Some(&page),
                caps: Capabilities::FULL,
                edit_epoch: 0,
            },
            &selection,
            &mut actions,
        );
    });

    assert!(
        text_focused,
        "the test is vacuous unless a REAL text field really holds focus — this is the exact \
         condition D1's guard was written for and its own test could not produce"
    );
    assert!(
        composing_now,
        "the wide predicate must agree with egui when egui is the claimant"
    );
    assert!(
        actions.is_empty(),
        "an arrow key belongs to the caret when a field has focus: {actions:?}"
    );
}

/// ★★★ **A canvas draft keeps its arrow keys, and egui cannot see it.**
///
/// The second claimant, and the one `text_edit_focused()` answers `false` for.
/// The caret this shell paints sits in PDF space at the glyphs' own scale, so it
/// is deliberately not an `egui::TextEdit` — which is why the predicate has to
/// be [`crate::canvas::textedit::composing`] and not egui's own.
///
/// Asserting `!text_focused && composing` is the whole content of that gap: the
/// second instance of D1 cost the operator the space bar because a guard asked
/// egui alone, and an arrow key is where the same gap costs a caret its
/// movement.
#[test]
fn a_canvas_draft_keeps_its_arrow_keys() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);

    // Frame 1: begin composing on the canvas — no widget, no focus.
    let _ = ctx.run_ui(RawInput::default(), |ui| {
        crate::canvas::textedit::store(
            ui.ctx(),
            crate::canvas::textedit::Draft {
                page: 0,
                kind: crate::canvas::textedit::TextEditKind::Add,
                anchor: crate::canvas::textedit::Anchor::Origin { x: 10.0, y: 10.0 },
                // ui-text-exempt: a test fixture's half-typed word.
                text: "SHEE".to_owned(),
                caret: 4,
                mark: None,
                seeded: true,
            },
        );
    });

    let mut text_focused = true;
    let mut composing_now = false;
    let mut actions = Vec::new();
    let _ = ctx.run_ui(arrows(Key::ArrowUp, Modifiers::NONE, 1), |ui| {
        // typing-guard-exempt: a TEST asserting the harness reached the state
        // egui CANNOT see. The assertion below is that this is false while the
        // operator is visibly mid-word, which is the whole of the gap.
        text_focused = ui.ctx().text_edit_focused();
        composing_now = crate::canvas::textedit::composing(ui.ctx());
        keys(
            ui.ctx(),
            &Frame {
                page: Some(&page),
                caps: Capabilities::FULL,
                edit_epoch: 0,
            },
            &selection,
            &mut actions,
        );
    });

    assert!(
        !text_focused,
        "the canvas caret is not a widget — if egui can see it, this test has stopped \
         exercising the gap it exists for"
    );
    assert!(composing_now, "a draft in flight IS the operator typing");
    assert!(
        actions.is_empty(),
        "an arrow moves the caret, not the mark, while a draft is in flight: {actions:?}"
    );
}

// ---------------------------------------------------------------------------
// The refusals
// ---------------------------------------------------------------------------

/// **A locked mark refuses, and says so on the status row.**
///
/// §12.5.3 Table 165 bit 8. The sentence matters as much as the refusal: the
/// Properties panel's standing sentence for a locked mark is about its
/// *appearance*, so an operator pressing an arrow has been told nothing relevant
/// and silence would be the standing cross-cutting defect.
#[test]
fn a_locked_mark_refuses_and_says_why() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, true);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        Some(&page),
        Capabilities::FULL,
        &selection,
    );
    assert!(actions.is_empty(), "the document forbids it: {actions:?}");
    let said = crate::app::actions::last_edit_disclosure(0).expect("a refusal owes a sentence");
    assert!(
        said.notes
            .iter()
            .any(|n| n == crate::text::arrange::locked_cannot_move()),
        "the status row must carry the lock sentence: {:?}",
        said.notes
    );
}

/// **A ce dimension refuses** — rule 15, guarded by the `AnnotKind` match rather
/// than by a `/Subtype` string.
///
/// Moving one has to re-measure it, which `move_annotation` does not do and
/// refuses by name. The pointer makes the same fork one module over
/// (`dimdrag` claims it, `annotdrag` does not), so this is that fork restated
/// for the keyboard.
#[test]
fn a_ce_dimension_is_not_nudged_by_this_verb() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::CeDimension, false);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowRight, Modifiers::NONE, 1),
        Some(&page),
        Capabilities::FULL,
        &selection,
    );
    assert!(
        actions.is_empty(),
        "a measurement moves through its own verb: {actions:?}"
    );
    let said = crate::app::actions::last_edit_disclosure(0).expect("a refusal owes a sentence");
    assert!(
        said.notes
            .iter()
            .any(|n| n == crate::text::arrange::dimension_use_the_pointer()),
        "the sentence must name the re-measure, not the verb: {:?}",
        said.notes
    );
}

/// **A mode that may not author markup nudges nothing.**
///
/// `author_markup`, not `edit_content` — Review has the second `false` and the
/// first `true`, and it is the mode an operator is in *because* they are working
/// on comments. The Delete rung learned this the hard way.
#[test]
fn a_mode_that_cannot_author_markup_nudges_nothing() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        Some(&page),
        Capabilities::NONE,
        &selection,
    );
    assert!(actions.is_empty(), "{actions:?}");

    // …and Review, which is the case that matters, nudges fine.
    let review = Capabilities {
        edit_content: false,
        author_markup: true,
        author_measure: false,
    };
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        Some(&page),
        review,
        &selection,
    );
    assert_eq!(
        actions.len(),
        1,
        "Review is the markup stance and must be able to nudge a comment"
    );
}

/// **Nothing selected is silent; page content selected is told.**
///
/// The split the refusal catalog argues for. Arrow keys are pressed constantly
/// for reasons that are not about a selection, so a bar that narrated every
/// stray press would stop being read — but an operator who picked a line out of
/// the drawing and pressed a key every drawing program binds is asking a
/// question.
#[test]
fn an_empty_selection_says_nothing_and_page_content_says_something() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);

    crate::app::actions::record_notes(0, Vec::new());
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        Some(&page),
        Capabilities::FULL,
        &SelectionState::default(),
    );
    assert!(actions.is_empty());
    assert!(
        crate::app::actions::last_edit_disclosure(0).is_none_or(|d| d.notes.is_empty()),
        "a stray arrow with nothing selected must not narrate"
    );

    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        Some(&page),
        Capabilities::FULL,
        &object_selection(),
    );
    assert!(actions.is_empty(), "{actions:?}");
    let said = crate::app::actions::last_edit_disclosure(0)
        .expect("a selected object and a pressed arrow is a question");
    assert!(
        said.notes
            .iter()
            .any(|n| n == crate::text::arrange::not_a_markup()),
        "it must say what DOES work — dragging: {:?}",
        said.notes
    );
}

/// **A page with no geometry declines rather than fabricating a delta.**
///
/// `page: None` is what a frame with no page on screen genuinely hands over.
/// The alternative — assuming an unrotated letter page — would author a move in
/// units nothing on screen agrees with.
#[test]
fn a_frame_with_no_page_declines_with_a_sentence() {
    let ctx = egui::Context::default();
    let selection = annot_selection(AnnotKind::Markup, false);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        None,
        Capabilities::FULL,
        &selection,
    );
    assert!(actions.is_empty(), "{actions:?}");
    let said = crate::app::actions::last_edit_disclosure(0).expect("a refusal owes a sentence");
    assert!(
        said.notes
            .iter()
            .any(|n| n == crate::text::arrange::degenerate_page()),
        "{:?}",
        said.notes
    );
}

/// **A held arrow that refuses reports once, not forty times.**
///
/// A refusal is a property of the selection and the mode, so every repeat of a
/// held key would refuse identically. Writing the same sentence per repeat would
/// make the status row flicker and would put forty identical lines in a trace a
/// harness has to read.
#[test]
fn a_held_arrow_that_refuses_reports_once() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, true);
    let actions = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 5),
        Some(&page),
        Capabilities::FULL,
        &selection,
    );
    assert!(actions.is_empty(), "{actions:?}");
    let said = crate::app::actions::last_edit_disclosure(0).expect("a refusal owes a sentence");
    assert_eq!(
        said.notes.len(),
        1,
        "one sentence for a held key, not one per repeat: {:?}",
        said.notes
    );
}

/// **A nudge never alters the selection.**
///
/// [`super::super::tests`]' first invariant, restated for the keyboard. The mark
/// stays selected so a second press nudges it again — an operator correcting a
/// position presses four times, and a gesture that dropped the selection after
/// the first would make that impossible.
#[test]
fn a_nudge_never_alters_the_selection() {
    let ctx = egui::Context::default();
    let page = test_page(612.0, 792.0, 0);
    let selection = annot_selection(AnnotKind::Markup, false);
    let before = selection.clone();
    let _ = nudged(
        &ctx,
        arrows(Key::ArrowUp, Modifiers::NONE, 1),
        Some(&page),
        Capabilities::FULL,
        &selection,
    );
    assert_eq!(
        selection.annot().map(|a| a.target.id),
        before.annot().map(|a| a.target.id)
    );
}
