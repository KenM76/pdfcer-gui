//! # `app::status::decline::tests` — the worded decline's rules, asserted
//! headlessly
//!
//! In its own file because [`super`] is one subject — a store, a retirement
//! rule and one line in the status bar — and splitting the *code* along a seam
//! it does not have would be churn. A test module is the one part of a
//! single-subject file that can leave without taking a seam with it, and
//! `crate::app::actions::forms` and its `forms/` directory are the same
//! arrangement in this crate already.
//!
//! Everything here reads [`super`]'s private items through `use super::*`,
//! exactly as an inline module does; the split changes nothing about what
//! these tests can see.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline/tests.md`.

// The INNER attribute, not just the `mod tests;` declaration in the parent.
// `check-ui-strings.sh`'s exclusion 2b recognises a whole test file **from the
// file** rather than from its name, and without it every assertion message here
// is reported as operator-facing copy — dozens of lines of it. The noise is the
// actual hazard: it trains people to ignore the report.
#![cfg(test)]

use super::*;
use crate::app::state::Status;
use crate::app::status::test_support::{opened, settled_bar_frame};
use egui::Context;

// =======================================================================
// The retirement rule — pure, so every property is pinned without a window
// =======================================================================

/// **A decline is retired by the state that produced it stopping being
/// true**, and by nothing else.
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

    // A failed save survives every combination of the two facts, because
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

    // An engine refusal survives every combination too — and for a
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

/// **Each history decline is retired by ITS OWN stack filling, and by
/// the other's it is not.**
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

/// **Undo's and redo's declines are two sentences, recorded by name.**
#[test]
fn the_two_history_declines_do_not_share_a_slot_or_a_sentence() {
    retire();
    record_history_empty(Declined::NothingToUndo);
    assert_eq!(
        LAST.with_borrow(Clone::clone),
        Some(Declined::NothingToUndo)
    );
    retire();
    record_history_empty(Declined::NothingToRedo);
    assert_eq!(
        LAST.with_borrow(Clone::clone),
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

/// **A failed save is recorded, survives a frame, and is retired by the
/// operator's next command — so two failed saves are two events.**
#[test]
fn a_failed_save_is_recorded_and_retired_by_the_next_command() {
    retire();
    record_save_failure();
    assert_eq!(
        LAST.with_borrow(Clone::clone),
        Some(Declined::SaveFailed),
        "the failure must reach the store, or the bar has nothing to draw"
    );

    // The frame's own dispatch already ran before the apply that recorded
    // this, so the sentence is still there on the next frame.
    assert!(Declined::SaveFailed.still_true(true, true, History::default(), false));

    // …and the operator's next command ends it.
    retire();
    assert_eq!(LAST.with_borrow(Clone::clone), None);

    // Two failures in a row are two events: the second press retires the
    // first sentence through `retire` and then records its own.
    record_save_failure();
    retire();
    record_save_failure();
    assert_eq!(LAST.with_borrow(Clone::clone), Some(Declined::SaveFailed));
    retire();
}

/// **A clamped framing zoom is not a decline.**
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
#[test]
fn no_two_declines_share_a_sentence() {
    let all = [
        Declined::NothingToFrame,
        Declined::CanvasNotDrawn,
        Declined::SaveFailed,
        // The un-categorised engine refusal is the one most at risk of
        // being written as a paraphrase of a neighbour, because it is the one
        // with the least to say — and a decline that reads like another
        // decline tells the operator the wrong thing happened.
        Declined::EditRefused,
        // The fifth and sixth entries (`OPERATOR_REQUESTS.md` O188). Not a
        // formality for these: their sentences are fetched by calls into
        // `text::arrange`, two modules away from every other entry here, so
        // nothing but this loop would notice one paraphrasing a neighbour. And
        // the neighbour they are nearest to is `EditRefused` directly above —
        // all three answer *the thing you just tried did not happen*, and only
        // two of them are allowed to say why.
        //
        // **The pair is nearest of all to EACH OTHER**, and that is the
        // comparison this loop was extended for. They are two refusals of one
        // gesture, differing only in which line the file failed to state a
        // position for, and they end with the identical remedy clause. A later
        // edit that collapsed them into one wording would leave the program
        // telling the operator *there is nothing to change* about a line whose
        // position is perfectly well stated — true-sounding, and wrong about
        // the only fact he needs.
        Declined::TextRunHasNoPositionOfItsOwn,
        Declined::TextRunWouldDragTheNextLine,
        Declined::OcrLayer(crate::text::ocr::OcrLayerRefusal::AlreadyPresent),
        Declined::OcrLayer(crate::text::ocr::OcrLayerRefusal::LayerGone),
        Declined::OcrLayer(crate::text::ocr::OcrLayerRefusal::NoneFound),
        Declined::RunMerge(crate::text::runmerge::RunMergeRefusal::StylesDiffer),
        Declined::RunMerge(crate::text::runmerge::RunMergeRefusal::WouldMoveNextRun),
        Declined::RunMerge(crate::text::runmerge::RunMergeRefusal::Other),
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

/// **Two presses are two events**, and the second one registers.
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

/// **The dispatcher words the decline, and the next command retires
/// it.**
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

/// **A worded decline does not change the bar's height** — R128 for the
/// sentence a refused command puts there.
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

// =======================================================================
// The clipboard's mode refusal
// =======================================================================

/// **A cut or a paste the mode does not do reaches the `⊗` slot**, and
/// the operator's next command takes it down.
#[test]
fn a_clipboard_verb_the_mode_refuses_is_worded_and_then_retired() {
    use crate::text::clipboard::ModeRefusal;

    record_mode_refusal(ModeRefusal::PasteContent);
    assert_eq!(
        LAST.with_borrow(Clone::clone),
        Some(Declined::ClipboardMode(ModeRefusal::PasteContent)),
        "a paste the mode does not do must be recorded, not merely traced"
    );

    // The store is a slot, not a queue: the operator who presses again
    // before moving the selector must get the second press's sentence, and
    // an operand that changed under them must change the sentence with it.
    record_mode_refusal(ModeRefusal::PasteMarkup);
    assert_eq!(
        LAST.with_borrow(Clone::clone),
        Some(Declined::ClipboardMode(ModeRefusal::PasteMarkup))
    );

    // …and it is in the `retire`-only class, so the next command clears it.
    // See the variant's docs for why the tense argument is the only one
    // available here: the condition it reports is the one the sentence asks
    // the operator to change.
    retire();
    assert_eq!(LAST.with_borrow(Clone::clone), None);
}

/// **The six mode refusals are six sentences, and none of them is any
/// other decline's.**
#[test]
fn a_mode_refusal_reads_like_no_other_decline() {
    use crate::text::clipboard::ModeRefusal;

    let mine = [
        ModeRefusal::PasteContent,
        ModeRefusal::PasteMarkup,
        ModeRefusal::PasteField,
        ModeRefusal::CutContent,
        ModeRefusal::CutMarkup,
        ModeRefusal::CutField,
    ]
    .map(Declined::ClipboardMode);
    let others = [
        Declined::NothingToFrame,
        Declined::CanvasNotDrawn,
        Declined::SaveFailed,
        Declined::EditRefused,
    ];
    for a in &mine {
        for b in &others {
            assert_ne!(a.line(), b.line(), "{a:?} and {b:?} read the same");
        }
        // …and each still describes the application after the frame that
        // raised it: a report of a past press cannot become false.
        assert!(
            a.still_true(false, false, History::default(), false),
            "{a:?} must survive to be read"
        );
    }
}

/// **The paste's mode gate reaches the bar, through the real
/// dispatcher** — the call site, not the recorder.
#[test]
fn a_paste_the_mode_refuses_reaches_the_bar_through_the_dispatcher() {
    use crate::text::clipboard::ModeRefusal;

    let ctx = Context::default();
    let mut app = crate::app::tests::opened();
    app.dispatch_command(&ctx, "mode.read", &mut Vec::new());
    retire();

    app.dispatch_command(&ctx, "edit.paste", &mut Vec::new());
    let Status::Open(doc) = &app.status else {
        unreachable!("the fixture is open")
    };
    assert_eq!(
        live(&ctx, doc),
        Some(Declined::ClipboardMode(ModeRefusal::PasteMarkup)),
        "Read authors no markup, so the paste is refused — and a refusal that \
         reaches only the trace is a keystroke that does nothing and says nothing"
    );
}

// =======================================================================
// O172 — the operator's own stamps, and the gap this file had all along
// =======================================================================

/// **A compile-time tripwire: a new `Declined` variant cannot be added
/// without somebody reading this file.**
///
/// # The gap it closes, which is this project's most-repeated defect shape
///
/// Two tests above — [`no_two_declines_share_a_sentence`] and
/// [`a_mode_refusal_reads_like_no_other_decline`] — assert that declines do not
/// read alike, and both do it against a **hand-written list**. Between them
/// they name **six** of the enum's **thirty-three** variants. The other
/// twenty-seven are invisible to the checks built to find exactly this, and
/// the count still adds up: the tests pass, the suite grows, and a new decline
/// that paraphrases an old one ships without a single thing going red.
///
/// This function has no assertions and never runs. Its `match` has **no `_`
/// arm**, so the compiler refuses the build the moment a variant is added, and
/// the author who added it is standing in the file that says what to do.
///
/// # What to do when this stops compiling
///
/// 1. Add the new variant to [`no_two_declines_share_a_sentence`]'s list, with
///    a representative payload if it carries one.
/// 2. Add its arm here.
///
/// # What this is NOT
///
/// It is not the census. The honest state, written down rather than implied:
/// **six of thirty-three variants are compared for a distinct sentence**, plus
/// the six clipboard-mode refusals against those six. Building the full
/// pairwise census needs one representative of each of the eight payload enums
/// and would very likely surface a genuine collision or two — which is worth
/// doing, and is not done here because a collision is a **wording decision**
/// and this file is not where wording decisions are made.
///
/// # The counts above are a measurement, and measurements drift
///
/// Prose has no compiler, and the tripwire below guards the **match**, not the
/// sentence describing the match — so a count in this paragraph can be wrong
/// with nothing going red. Re-measure instead of trusting it:
///
/// ```text
/// awk '/^pub\(crate\) enum Declined \{/,/^\}/' app/status/decline.rs \
///   | grep -cE '^    [A-Z][A-Za-z]*(\(|,| \{)'
/// ```
#[allow(dead_code)]
fn a_new_decline_cannot_be_added_unnoticed(declined: Declined) {
    match declined {
        Declined::NothingToFrame
        | Declined::CanvasNotDrawn
        | Declined::SaveFailed
        | Declined::InsideForm(_)
        | Declined::TextStyle(_)
        | Declined::Rotate(_)
        | Declined::Unshare(_)
        | Declined::RunMerge(_)
        | Declined::SettingsNotSaved
        | Declined::FieldNameTaken
        | Declined::FieldPathCrossesTerminal(_)
        | Declined::DottedPartialName(_)
        | Declined::WidgetHasNoName
        | Declined::FlattenCertified
        | Declined::FieldDeleteRefused
        | Declined::NodeToolNeedsEditMode
        | Declined::VertexEditRefused(_)
        | Declined::MarkupNodeRefused(_)
        | Declined::FieldGroupPreviewRefused
        | Declined::FieldGroupDeleteRefused
        | Declined::CustomStampUnavailable(_)
        | Declined::BookmarkMoveIntoOwnSubtree
        | Declined::BookmarkMoveRefused
        | Declined::NothingToUndo
        | Declined::NothingToRedo
        | Declined::EditRefused
        | Declined::Reflow(_)
        | Declined::EnterCannotSplit
        | Declined::ClipboardMode(_)
        | Declined::EditText(_)
        // These two are STRUCT variants, and they are why a compiler check
        // beats a generated one: a regular expression over the enum that
        // matches `Name(` and `Name,` misses both of them, and the compiler
        // does not. A hand-written list is wrong the day it is written, not
        // later.
        | Declined::ResizeNotRebuildable { .. }
        | Declined::ResizeFixedSizeMarker { .. }
        // A variant added in another file breaks the build HERE, and here is
        // the only place that makes the author add it to the uniqueness loop
        // above as well. Nothing else in the suite goes red for it, and the
        // count still adds up — which is also what happens when a variant is
        // REPLACED rather than added. A rename the compiler waves through is
        // how a completeness test quietly stops being complete.
        | Declined::TextRunHasNoPositionOfItsOwn
        | Declined::TextRunWouldDragTheNextLine
        | Declined::OcrLayer(_) => {}
    }
}

/// **The two custom-stamp declines are two sentences, and neither is the
/// engine floor's** — `OPERATOR_REQUESTS.md` O172.
#[test]
fn the_custom_stamp_declines_say_two_different_things() {
    use crate::text::stamps::CustomStampUnavailable;

    let unreadable = Declined::CustomStampUnavailable(CustomStampUnavailable::Unreadable);
    let gone = Declined::CustomStampUnavailable(CustomStampUnavailable::PageGone);

    assert_ne!(
        unreadable.line(),
        gone.line(),
        "a missing collection file and a rewritten one are different findings"
    );
    for mine in [unreadable, gone] {
        assert_ne!(
            mine.line(),
            Declined::EditRefused.line(),
            "{mine:?} fell back to the floor sentence it exists to replace"
        );
    }
}

/// **Neither custom-stamp decline claims anything was edited.**
#[test]
fn a_refused_stamp_is_not_reported_as_an_edit() {
    use crate::text::stamps::CustomStampUnavailable;

    retire();
    record_custom_stamp_unavailable(CustomStampUnavailable::PageGone);
    assert_eq!(
        recorded_for_test(),
        Some(Declined::CustomStampUnavailable(
            CustomStampUnavailable::PageGone
        )),
        "the decline channel is the one that means `nothing happened`"
    );
    // …and it is still true on the next frame, because no edit moved past it.
    // Every argument is the one that would retire some OTHER decline: a
    // framable selection, a drawn canvas, a full history, a selection inside a
    // form. None of them touches a stamps folder, and that is the assertion.
    assert!(
        Declined::CustomStampUnavailable(CustomStampUnavailable::PageGone).still_true(
            true,
            true,
            History {
                can_undo: true,
                can_redo: true,
            },
            true,
        ),
        "a decline about a stamps folder has no live predicate to go stale on"
    );
    retire();
}
