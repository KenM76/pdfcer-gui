//! # `app::status::decline::tests` — the worded decline's rules, asserted
//! headlessly
//!
//! ★★ In its own file for R2 (no `.rs` over 1,500 lines) and for nothing else:
//! [`super`] is one subject — a store, a retirement rule and one line in the
//! status bar — and splitting the *code* along a seam it does not have would
//! have been churn. A test module is the one part of a single-subject file that
//! can leave without taking a seam with it, and `crate::app::actions::forms`
//! and its `forms/` directory are the same arrangement in this crate already.
//!
//! Everything here reads [`super`]'s private items through `use super::*`,
//! which is exactly what it did as an inline module; nothing changed about
//! what these tests can see.

// ★★ The INNER attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and without it every assertion message here
// is reported as operator-facing copy — twelve of them, on the first run of
// this split, and 28 the last time a test module was split under R2. The noise
// is the actual hazard: it trains people to ignore the report.
#![cfg(test)]

use super::*;
use crate::app::state::Status;
use crate::app::status::test_support::{opened, settled_bar_frame};
use egui::Context;

// =======================================================================
// The retirement rule — pure, so every property is pinned without a window
// =======================================================================

/// ★ **A decline is retired by the state that produced it stopping being
/// true**, and by nothing else.
///
/// The full matrix, both directions on every variant. The "still true"
/// direction is the one worth stating explicitly: a decline whose reason
/// still holds must survive, or the sentence would flicker off on the next
/// frame and the operator would never read it.
#[test]
fn a_decline_lives_exactly_as_long_as_its_reason() {
    // An empty command log — the state the two zoom declines were written
    // against, and the one every assertion below that is not about the
    // history is indifferent to.
    let empty = History::default();

    // Nothing to frame: retired the moment something is framable, and
    // indifferent to whether the canvas has drawn.
    assert!(Declined::NothingToFrame.still_true(false, true, empty, false));
    assert!(Declined::NothingToFrame.still_true(false, false, empty, false));
    assert!(!Declined::NothingToFrame.still_true(true, true, empty, false));
    assert!(!Declined::NothingToFrame.still_true(true, false, empty, false));

    // Canvas not drawn: retired the moment it has, and indifferent to the
    // selection — the remedy arrives without the operator doing anything.
    assert!(Declined::CanvasNotDrawn.still_true(false, false, empty, false));
    assert!(Declined::CanvasNotDrawn.still_true(true, false, empty, false));
    assert!(!Declined::CanvasNotDrawn.still_true(false, true, empty, false));
    assert!(!Declined::CanvasNotDrawn.still_true(true, true, empty, false));

    // ★ A failed save survives every combination of the two facts, because
    // neither is about it: a folder that could not be written to does not
    // become writable because the operator selected something or because a
    // page finished drawing. It is retired by `retire` — the operator's
    // next command — and by nothing else. Asserted over the whole matrix
    // rather than once, so a future edit that "tidied" this variant into
    // one of the two predicates fails here instead of making the sentence
    // vanish on the next raster.
    for has_bounds in [false, true] {
        for drawn in [false, true] {
            assert!(
                Declined::SaveFailed.still_true(has_bounds, drawn, empty, false),
                "a failed write does not repair itself ({has_bounds}, {drawn})"
            );
        }
    }

    // ★★★ An engine refusal survives every combination too — and for a
    // DIFFERENT reason from the one above, which is why it is asserted
    // separately rather than folded into the same loop.
    //
    // `SaveFailed` survives because its condition is stable. `EditRefused`
    // cannot claim that: its causes are unknown by construction and some of
    // them do change under the operator. What it has instead is the tense —
    // the sentence reports what happened when the operator pressed, so no
    // later frame can falsify it — and, decisively, **no predicate to
    // re-ask**. A build that gave it one would be guessing at the engine's
    // reason, and a wrong guess answering `false` would take a true sentence
    // off the screen while the operator was reading it. See the variant's
    // docs; this is the assertion that fails if somebody "improves" it.
    for has_bounds in [false, true] {
        for drawn in [false, true] {
            for in_form in [false, true] {
                assert!(
                    Declined::EditRefused.still_true(
                        has_bounds,
                        drawn,
                        History {
                            can_undo: true,
                            can_redo: true,
                        },
                        in_form
                    ),
                    "an unexplained refusal was retired by a fact that has nothing to do with \
                     it ({has_bounds}, {drawn}, {in_form})"
                );
            }
        }
    }
}

/// ★ **Each history decline is retired by ITS OWN stack filling, and by
/// the other's it is not.**
///
/// The cross terms are the reason this is a separate test rather than four
/// more lines in the matrix above. A build whose two arms read the same
/// field — the mistake a two-field struct makes available, and the reason
/// [`History`]'s doc comment argues for it over two loose booleans — would
/// pass every same-stack assertion and fail only here.
///
/// The remedy arriving *without a command* is the whole point: authoring a
/// rectangle is a canvas gesture that reaches no dispatcher, so [`retire`]
/// never runs and only this filter can end the sentence. That is
/// [`Declined::NothingToFrame`]'s property, and it is why both of these
/// have a live predicate at all rather than [`Declined::SaveFailed`]'s
/// unconditional `true`.
#[test]
fn a_history_decline_is_retired_by_its_own_stack() {
    let empty = History::default();
    let undoable = History {
        can_undo: true,
        can_redo: false,
    };
    let redoable = History {
        can_undo: false,
        can_redo: true,
    };

    // Its own stack is what retires it…
    assert!(Declined::NothingToUndo.still_true(false, true, empty, false));
    assert!(!Declined::NothingToUndo.still_true(false, true, undoable, false));
    assert!(Declined::NothingToRedo.still_true(false, true, empty, false));
    assert!(!Declined::NothingToRedo.still_true(false, true, redoable, false));

    // …and the OTHER stack is not. An operator who authors something can
    // undo it and still has nothing to redo, so a "nothing to redo"
    // sentence that vanished when the undo stack filled would retire on a
    // state that has not changed for it.
    assert!(
        Declined::NothingToRedo.still_true(false, true, undoable, false),
        "an undoable change is not something to redo"
    );
    assert!(
        Declined::NothingToUndo.still_true(false, true, redoable, false),
        "a redoable change is not something to undo"
    );

    // Indifferent to the two zoom facts, in every combination: neither the
    // selection nor the raster has anything to do with a command log.
    for has_bounds in [false, true] {
        for drawn in [false, true] {
            assert!(Declined::NothingToUndo.still_true(has_bounds, drawn, empty, false));
            assert!(Declined::NothingToRedo.still_true(has_bounds, drawn, empty, false));
        }
    }
}

/// ★ **Undo's and redo's declines are two sentences, recorded by name.**
///
/// [`record_history_empty`] takes the value rather than a `bool`, and the
/// property that buys is asserted here: pressing `Ctrl+Y` with an empty
/// redo stack must not leave the bar saying the document has no changes.
/// The ordering half is [`Declined::SaveFailed`]'s, already pinned above —
/// both record in the apply phase, after the frame's `retire`.
#[test]
fn the_two_history_declines_do_not_share_a_slot_or_a_sentence() {
    retire();
    record_history_empty(Declined::NothingToUndo);
    assert_eq!(
        LAST.with_borrow(|slot| *slot),
        Some(Declined::NothingToUndo)
    );
    retire();
    record_history_empty(Declined::NothingToRedo);
    assert_eq!(
        LAST.with_borrow(|slot| *slot),
        Some(Declined::NothingToRedo)
    );
    assert_ne!(
        Declined::NothingToUndo.line(),
        Declined::NothingToRedo.line(),
        "one line reaches the operator; two states that need different \
         sentences must not share one"
    );
    retire();
}

/// ★ **A failed save is recorded, survives a frame, and is retired by the
/// operator's next command — so two failed saves are two events.**
///
/// The store half of [`Declined::SaveFailed`], and the ordering is the
/// interesting part: [`retire`] runs at the top of `dispatch_command` while
/// [`record_save_failure`] runs in the **apply** phase of the same frame,
/// which is later. A sentence recorded by a save therefore survives the
/// dispatch that raised it, and is cleared by the *next* command — which is
/// what makes a second `Ctrl+S` record a second sentence rather than
/// re-showing the first.
///
/// Reversing those two would be silent: the bar would simply never draw the
/// line, and a reader of the trace would still see `save-copy-failed`.
#[test]
fn a_failed_save_is_recorded_and_retired_by_the_next_command() {
    retire();
    record_save_failure();
    assert_eq!(
        LAST.with_borrow(|slot| *slot),
        Some(Declined::SaveFailed),
        "the failure must reach the store, or the bar has nothing to draw"
    );

    // The frame's own dispatch already ran before the apply that recorded
    // this, so the sentence is still there on the next frame.
    assert!(Declined::SaveFailed.still_true(true, true, History::default(), false));

    // …and the operator's next command ends it.
    retire();
    assert_eq!(LAST.with_borrow(|slot| *slot), None);

    // Two failures in a row are two events: the second press retires the
    // first sentence through `retire` and then records its own.
    record_save_failure();
    retire();
    record_save_failure();
    assert_eq!(LAST.with_borrow(|slot| *slot), Some(Declined::SaveFailed));
    retire();
}

/// ★ **A clamped framing zoom is not a decline.**
///
/// The one case this module is deliberately blind to. A region zoom past
/// the page's raster ceiling still zooms, still centres what was asked
/// for, and raises `Action::ZoomTo` carrying the clamped scale — so the
/// bar's own zoom readout states the truth on the same frame. Wording it
/// would word a non-event.
///
/// Asserted for the clamped case *and* the exact one, because a store that
/// happened to reject only the exact case would pass a test written the
/// obvious way and still ship the sentence nobody wants.
#[test]
fn a_partial_grant_is_not_a_decline() {
    let clamped = ZoomOutcome::Zoomed {
        requested: 40.0,
        applied: crate::viewer::MAX_ZOOM,
    };
    assert!(
        clamped.ceiling_changed_the_answer(),
        "the fixture must really be the clamped case, or this proves nothing"
    );
    assert_eq!(
        Declined::of(clamped),
        None,
        "the ceiling reports itself through the zoom readout; a second \
         report in words would fire when nothing was declined"
    );
    assert_eq!(
        Declined::of(ZoomOutcome::Zoomed {
            requested: 2.0,
            applied: 2.0
        }),
        None
    );

    // …and both genuine declines are carried.
    assert_eq!(
        Declined::of(ZoomOutcome::NoBounds),
        Some(Declined::NothingToFrame)
    );
    assert_eq!(
        Declined::of(ZoomOutcome::NoCanvas),
        Some(Declined::CanvasNotDrawn)
    );
}

/// Each decline says its own thing, from the catalog.
///
/// Three now rather than two, and asserted pairwise: the operator gets one
/// line, and "nothing is selected", "the page is still drawing" and "the
/// copy was not written" have three different remedies. A shared sentence
/// would be a decline that does not say which command declined.
#[test]
fn no_two_declines_share_a_sentence() {
    let all = [
        Declined::NothingToFrame,
        Declined::CanvasNotDrawn,
        Declined::SaveFailed,
        // ★ Four now. The un-categorised engine refusal is the one most at
        // risk of being written as a paraphrase of a neighbour, because it is
        // the one with the least to say — and a decline that reads like
        // another decline tells the operator the wrong thing happened.
        Declined::EditRefused,
    ];
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a.line(), b.line(), "{a:?} and {b:?} read the same");
        }
    }
}

// =======================================================================
// The store — recorded, retired, and repeatable
// =======================================================================

/// ★ **Two presses are two events**, and the second one registers.
///
/// This is the property an edit-epoch key **cannot** express, and the
/// reason this module has a store of its own: a decline changes no
/// document, so an epoch-keyed sentence would be identical on both presses
/// and would never retire in between. Here the sequence
/// *decline → the operator does something else → decline again* puts the
/// sentence back, which is what makes the second press an answer rather
/// than a swallowed keystroke.
#[test]
fn a_decline_can_be_raised_again_after_the_operator_moves_on() {
    let ctx = Context::default();
    let status = opened();
    let Status::Open(doc) = &status else {
        unreachable!("`opened()` returns an open document")
    };

    record(ZoomOutcome::NoBounds);
    assert_eq!(live(&ctx, doc), Some(Declined::NothingToFrame));

    // The operator's next act — any command at all.
    retire();
    assert_eq!(live(&ctx, doc), None, "the next command ends the sentence");

    // …and pressing the chord again is a second event, not a repeat of a
    // sentence that was never taken down.
    record(ZoomOutcome::NoBounds);
    assert_eq!(
        live(&ctx, doc),
        Some(Declined::NothingToFrame),
        "the second press must register, or the operator has pressed a \
         chord and been told nothing"
    );
}

/// A framing zoom that *worked* silences a decline on the spot, rather
/// than leaving it to be retired by whatever comes next.
#[test]
fn a_successful_zoom_takes_the_sentence_down_itself() {
    let ctx = Context::default();
    let status = opened();
    let Status::Open(doc) = &status else {
        unreachable!("`opened()` returns an open document")
    };

    record(ZoomOutcome::NoBounds);
    assert!(live(&ctx, doc).is_some());
    record(ZoomOutcome::Zoomed {
        requested: 2.0,
        applied: 2.0,
    });
    assert_eq!(live(&ctx, doc), None);
}

// =======================================================================
// The wiring — through the real dispatcher
// =======================================================================

/// ★ **The dispatcher words the decline, and the next command retires
/// it.**
///
/// Driven through `PdfcerApp::dispatch_command`, which is the same entry
/// point a ribbon click, a quick-access click and a keyboard chord all
/// reach — so what is asserted is the real routing rather than a
/// hand-assembled approximation of it.
///
/// Three steps, and the middle one is the point of the whole module:
///
/// 1. `view.zoom_selection` on a freshly-opened document declines. Nothing
///    is selected, so `zoom_to_selection` returns `NoBounds` before it ever
///    looks for a canvas frame — which is why the expected variant is
///    `NothingToFrame` and not `CanvasNotDrawn` even though the canvas has
///    also never drawn here.
/// 2. The sentence is **live**, which is the thing that used to be
///    missing: the outcome reached the bar instead of the floor.
/// 3. Any other command retires it. Asserted with `view.zoom_actual` — an
///    ordinary, unrelated verb — because the rule is "the operator's next
///    act", not "an act about zooming".
///
///    ★ It was `view.zoom_in` until 2026-08-15, when that arm was deleted
///    as one of the four `shell::commands::reach::UNREACHED_ARMS` — an arm
///    for an id no token names. The assertion would still have passed,
///    because `retire()` runs *above* the `match` and an unimplemented id
///    reaches the catch-all — which is exactly why it was changed: a test
///    whose subject is "any other **command**" must name one that exists,
///    or it is quietly asserting something weaker than it says.
#[test]
fn the_dispatcher_words_a_decline_and_the_next_command_retires_it() {
    let ctx = Context::default();
    let mut app = crate::app::tests::opened();
    retire();

    app.dispatch_command(&ctx, "view.zoom_selection", &mut Vec::new());
    {
        let Status::Open(doc) = &app.status else {
            unreachable!("the fixture is open")
        };
        assert_eq!(
            live(&ctx, doc),
            Some(Declined::NothingToFrame),
            "the outcome `zoom_to_selection` returned reached the bar; \
             before this row was built it was dropped on the floor"
        );
    }

    app.dispatch_command(&ctx, "view.zoom_actual", &mut Vec::new());
    let Status::Open(doc) = &app.status else {
        unreachable!("the fixture is open")
    };
    assert_eq!(
        live(&ctx, doc),
        None,
        "a sentence about a gesture must not outlive the gesture after it \
         — that is the failure an edit-epoch key would have shipped"
    );
}

// =======================================================================
// R128 — the height that must not move
// =======================================================================

/// ★ **A worded decline does not change the bar's height** — R128 for the
/// sentence a refused command puts there.
///
/// # Why this needs its own test beside the edit-disclosure one
///
/// Same rule, different arrival, and this arrival is the awkward one. The
/// edit disclosure follows a drag; this follows a **keyboard chord**, and
/// a chord is precisely the gesture where the operator is looking at the
/// page rather than at their hands. If this line grew the bar, an active
/// `FitMode` would recompute its zoom from a viewport one row smaller on
/// the next frame, and the page would visibly shrink in response to a
/// command that **did nothing at all**. R128's measured symptom is *"the
/// page jumped when I clicked an object"*; this variant would read as
/// *"the page moved when the command was refused"*, and it would be
/// investigated in the zoom code, where nothing is wrong.
///
/// # The three assertions, and why none of them is the obvious one
///
/// 1. **A measurement happened at all** (`Some(_)`, never `None`) —
///    `HANDOFF.md` §10's rule. `cargo test -p egui-shell` and `cargo test
///    --workspace` compile `egui` with different features (no fonts vs
///    `default_fonts`), so a layout assertion can be entirely vacuous
///    under one of the two commands a developer runs.
/// 2. **The sentence reached the painter** — more shapes with the decline
///    live than without it. Without this, assertion 3 is satisfied just as
///    well by a [`show`] that returned early and drew nothing, which is
///    true and proves nothing.
/// 3. **The height did not move.** Asserted as `Some(true)` rather than
///    with a bare `assert!`, so a run in which either frame failed to
///    measure reads as `None` and fails, rather than reading as agreement.
///
/// [`Declined::NothingToFrame`] is the case tested because it is the one
/// an operator will actually reach, and because its sentence is the longer
/// of the two — the defence against a long sentence is eliding inside a
/// bounded sub-region with the whole text on hover, never wrapping,
/// because wrapping is how a one-row bar becomes a two-row bar.
#[test]
fn a_worded_decline_does_not_change_the_bar_height() {
    let ctx = Context::default();
    let status = opened();
    let Status::Open(doc) = &status else {
        unreachable!("`opened()` returns an open document")
    };

    retire();
    let absent = settled_bar_frame(&ctx, &status);

    record(ZoomOutcome::NoBounds);
    // The precondition, asserted rather than assumed: without it every
    // comparison below measures that an absent line did not change the
    // height, which is true and worthless.
    assert!(
        live(&ctx, doc).is_some(),
        "the recorded decline is not live for this document, so the bar \
         drew no line and everything below proves nothing"
    );

    let present = settled_bar_frame(&ctx, &status);

    let drew = match (absent, present) {
        (Some((_, before)), Some((_, after))) => Some(after > before),
        _ => None,
    };
    assert_eq!(
        drew,
        Some(true),
        "the bar painted no more shapes with a live decline ({present:?}) \
         than without one ({absent:?}); the sentence never reached the \
         painter, so the height comparison would be vacuous. `None` here \
         means a frame did not measure at all, which is the other failure \
         and is not a pass"
    );

    let same_height = match (absent, present) {
        (Some((before, _)), Some((after, _))) => Some((after - before).abs() < 0.01),
        _ => None,
    };
    assert_eq!(
        same_height,
        Some(true),
        "a worded decline changed the bar's height ({absent:?} → \
         {present:?}); that re-fits the page on the frame a command \
         refused to do anything, which is the one gesture that must \
         provably move nothing"
    );

    retire();
}
