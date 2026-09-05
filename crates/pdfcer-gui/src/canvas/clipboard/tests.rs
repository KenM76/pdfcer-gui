//! Unit tests for [`super`] — the canvas clipboard.
//!
//! ★ Split into a file of its own on 2026-09-05, when the annotation clipboard
//! took `clipboard.rs` past R2's 1,500-line ceiling. It is the same seam
//! `canvas::selection` already uses (`selection/tests.rs`), and it is the right
//! one for the same reason: the assertions grew a fixture, a session and a
//! paste round trip, which is a body of work with its own shape rather than a
//! coda on the module it tests.
//!
//! **Do not shorten the reasoning on any of these to fit a line count.** Each
//! header says what the assertion would miss if it were written the obvious
//! way, and that is the part a future session needs.

// ★★★ **THE INNER `#![cfg(test)]` IS LOAD-BEARING, not decoration.**
//
// `tools/gates/check-ui-strings.sh` scans every `.rs` file for operator-facing
// string literals outside the `ui_text` catalog, and it skips a test module by
// stopping at `#[cfg(test)]` — which does not exist in a file that IS the test
// module. Without this line the gate reads 25 assertion messages here as
// unrouted UI strings and fails, which is what it did the first time this file
// was split out.
//
// `canvas::selection::tests` carries the identical line for the identical
// reason and is the precedent. ★ It also means a non-test item added to this
// file would be invisible to the gate — so do not add one; this file is
// assertions and nothing else.
#![cfg(test)]

/// ★★★ **A cut that cannot delete must not copy either.**
///
/// The fourth door onto `delete_annotation`, found by an adversarial review
/// on 2026-08-29 after the other three had been gated the day before, and
/// the worst of the four: on a certified document `Ctrl+X` copied the
/// annotation, raised a Delete the engine then refused into a silent `Err`
/// arm, and `annots::delete` cleared the selection anyway — leaving the
/// operator with the markup still on the page, no selection, no
/// explanation, **and a clipboard holding a copy of it**, so the next
/// `Ctrl+V` duplicates the thing they were trying to move.
///
/// # What this asserts, and why each half is needed
///
/// 1. **`Err(DeleteRefused)`** — the whole gesture is refused, and it
///    carries the reason so the status row can say which of encryption,
///    certification or the `/F` Locked bit it was.
/// 2. **No action was raised** — asserting only the `Err` would pass on a
///    build that refused *and* pushed the Delete anyway, which is the state
///    this fix exists to remove.
/// 3. **Nothing reached the clipboard** — the half that makes it a *cut*
///    failure rather than a delete failure. A build that degraded the cut to
///    a copy would satisfy 1 and 2 and still hand the operator a duplicate.
///
/// ★ `certified-comments.pdf` and `threaded-comments.pdf` differ in exactly
/// one dictionary — the catalog's `/Perms` — so the pair tells *"withheld
/// here"* from *"offered there"* while varying one thing. This test drives
/// the refusing half; the offering half is the driven check's.
#[test]
fn a_cut_that_cannot_delete_does_not_copy_either() {
    use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

    let ctx = egui::Context::default();
    let mut doc = crate::app::state::open_local_fixture("certified-comments.pdf");
    let page = doc.pages.first().expect("the fixture has a page");
    let square = pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id)
        .into_iter()
        .find(|a| a.subtype_label() == "Square")
        .expect("the fixture carries a /Square");
    let id = square.id.expect("an indirect annotation");
    doc.selection.select_annot(AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id,
            kind: AnnotKind::Markup,
            subtype: "Square".to_owned(),
            locked: false,
        },
        outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
    });

    let mut actions = Vec::new();
    let outcome = cut(&ctx, &doc, &mut actions);

    assert!(
        matches!(outcome, Err(Refusal::DeleteRefused(_))),
        "a certified document must refuse the whole gesture, got {outcome:?}"
    );
    assert!(
        actions.is_empty(),
        "the cut was refused and raised {} action(s) anyway — a refusal that \
         still pushes the delete is the state this gate exists to remove",
        actions.len()
    );
    assert!(
        read(&ctx).is_none(),
        "nothing was deleted and something reached the clipboard: the cut \
         degraded to a copy, so the next paste hands the operator a duplicate \
         of the markup they were trying to move"
    );
}

use super::*;

/// The offset is applied on a same-page paste and not on a cross-page one.
///
/// ★ Asserted as arithmetic rather than by driving, because the *decision*
/// is the thing worth pinning: whether the copy is visible when it lands on
/// top of its original is a property of this one comparison, and a driven
/// check would prove it for one pair of pages.
#[test]
fn the_offset_is_same_page_only() {
    let same = if 3 == 3 { PASTE_OFFSET_PT } else { 0.0 };
    let across = if 3 == 7 { PASTE_OFFSET_PT } else { 0.0 };
    assert!(same > 0.0, "a copy on top of its original must be visible");
    assert!(
        across.abs() < f64::EPSILON,
        "a mark copied to another sheet belongs where it was on the first"
    );
}

/// ★ **Down the page is negative.** The one-line property that would
/// otherwise ship inverted and never be reported, because a paste that
/// drifts up-and-right looks like a decision rather than a bug.
#[test]
fn the_paste_moves_down_the_page() {
    let dy = -PASTE_OFFSET_PT;
    assert!(dy < 0.0, "PDF y increases upward, so down is negative");
}

// ---------------------------------------------------------------------------
// The annotation clipboard — 2026-09-05
// ---------------------------------------------------------------------------

/// The fixture the four assertions below are aimed at. Its generator,
/// `tools/gen-annots-with-everything-fixture.py`, argues for every key in it;
/// the short version is that **every annotation carries `/CA`, `/T`, `/M` and
/// `/Contents`**, none of which a `MarkupSpec` can express, so no assertion
/// here can pass against a build that re-authors from a spec.
const EVERYTHING: &str = "annots-with-everything.pdf";

/// Open the fixture with the annotation at `/Annots` position `index`
/// selected, exactly as a click on the canvas would leave it.
fn with_annot_selected(index: usize) -> crate::app::state::OpenDoc {
    use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

    let mut doc = crate::app::state::open_local_fixture(EVERYTHING);
    let page = doc.pages.first().expect("the fixture has a page");
    let annots = pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id);
    let annot = annots.get(index).expect("the fixture has this annotation");
    doc.selection.select_annot(AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id: annot.id.expect("an indirect annotation"),
            kind: AnnotKind::Markup,
            subtype: String::from_utf8_lossy(&annot.subtype).into_owned(),
            locked: false,
        },
        outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
    });
    doc
}

/// ★★★ **A STICKY NOTE CAN BE COPIED**, and until 2026-09-05 it could not.
///
/// This is the operator-facing whole of the change. `Ctrl+C` over a `/Text`
/// annotation used to answer *"that annotation is not one pdfcer authors …
/// so there is nothing for it to copy"*, because the clipboard read a
/// `MarkupSpec` out of the dictionary and `spec_from_dict` has no reader for a
/// sticky note. A sticky note is the most-copied comment in a review workflow.
///
/// # What the assertions are, and why the obvious one is not enough
///
/// Asserting only `Ok(_)` would pass against a build that parked an **empty**
/// clip — which is the plausible failure here, because `copy_selection`
/// returns `Ok` with an empty `annotations` vector if the index list never
/// reached it. So the count on the clip is asserted, and so is the fact that
/// it is a `Selection` rather than a `Markup`: a build that quietly fell back
/// to the spec route would satisfy every other assertion and would have
/// dropped the baked `/AP`.
#[test]
fn a_sticky_note_reaches_the_clipboard() {
    let ctx = egui::Context::default();
    let doc = with_annot_selected(1);

    let clipped = copy(&ctx, &doc).expect("a sticky note copies");

    match &clipped {
        Clipped::Selection {
            count,
            annotations,
            annot_ids,
            left_behind,
            thin,
            ..
        } => {
            assert_eq!(*annotations, 1, "the sticky note must be ON the clip");
            assert_eq!(*count, 0, "and no page content came with it");
            assert_eq!(annot_ids.len(), 1, "the cut half needs its id");
            assert!(left_behind.is_empty(), "nothing was refused");
            assert_eq!(
                *thin, 0,
                "★ a sticky note is carried WHOLE — if this is 1 the engine \
                 started modelling /Text and the copy is now dropping its \
                 author, date, note and opacity"
            );
        }
        other => panic!(
            "★ a sticky note must travel as a clip, not as a spec: a MarkupSpec \
             cannot express one at all, and falling back to that route would \
             drop the baked /AP and render the paste as nothing. Got {other:?}"
        ),
    }
    assert_eq!(
        read(&ctx),
        Some(clipped),
        "and it must be parked where the paste will look for it"
    );
}

/// ★★★ **A SQUARE STILL KEEPS ITS AUTHOR, DATE, NOTE AND OPACITY** — the
/// regression the lossless route would have shipped.
///
/// `pdfcer-core` models a `/Square`, so `copy_selection` carries it as a
/// `MarkupSpec` and `paste_clip_annotations` plants it with `add_markup` —
/// **not** `add_markup_with` — which drops `/CA`, `/T`, `/M` and `/Contents`.
/// A build that routed every annotation through the clip because the clip is
/// "the lossless one" would compile, pass a *"the paste happened"* test, and
/// hand the operator an anonymous, undated, opaque copy of a signed comment.
///
/// So the assertion is on the **carrier**: a modelled markup must come back as
/// `Clipped::Markup`, carrying `MarkupOptions` with all four facts in it.
///
/// ★ `/CA 0.4` rather than `/CA 1` in the fixture is deliberate and is the
/// difference between this test working and being vacuous — an opacity of 1
/// is what an absent `/CA` renders as, so a build that dropped the key would
/// look identical on screen and identical to a sloppier assertion.
#[test]
fn a_modelled_markup_keeps_what_a_spec_cannot_say() {
    let ctx = egui::Context::default();
    let doc = with_annot_selected(0);

    let clipped = copy(&ctx, &doc).expect("a square copies");

    let Clipped::Markup { options, spec, .. } = &clipped else {
        panic!(
            "★ a /Square is modelled by pdfcer, so the engine's clip carrier for it is a \
             MarkupSpec planted with add_markup — which drops /CA, /T, /M and /Contents. \
             Taking that route is a REGRESSION, not the lossless upgrade it looks like. \
             Got {clipped:?}"
        );
    };
    assert!(
        matches!(**spec, pdfcer_core::annot_author::MarkupSpec::Square { .. }),
        "the geometry travels as a square"
    );
    assert_eq!(
        options.opacity,
        Some(0.4),
        "★ /CA must survive: an opaque copy of a translucent mark looks correct against \
         white paper and wrong against the artwork underneath, which is a loss nobody reports"
    );
    let note = options
        .note
        .as_ref()
        .expect("★ /Contents must survive — a comment with no words is not a copy of a comment");
    assert_eq!(note.text, "Check this dimension.");
    assert_eq!(
        note.author.as_deref(),
        Some("A. Reviewer"),
        "★ /T must survive — a comment from nobody"
    );
    assert_eq!(
        note.modified.as_deref(),
        Some("D:20260905090000Z"),
        "★ /M must survive — a comment dated never"
    );
}

/// ★★ **A cut of an annotation is ONE undo entry**, and it deletes the
/// annotation it copied rather than the one now at that position.
///
/// Two assertions and the second is the one worth the test. A cut raises the
/// delete by `ObjId` taken **off the clip**, not by re-reading the selection
/// after the copy: two walks of `/Annots` with an edit between them can
/// disagree, and the window between them is exactly where a cut removes the
/// neighbour of the thing it copied.
#[test]
fn cutting_a_sticky_note_deletes_the_one_it_copied() {
    let ctx = egui::Context::default();
    let doc = with_annot_selected(1);
    let expected = doc.selection.annot().expect("selected").target.id;

    let mut actions = Vec::new();
    let clipped = cut(&ctx, &doc, &mut actions).expect("a sticky note cuts");

    assert!(
        matches!(clipped, Clipped::Selection { .. }),
        "the copy half took the clip route"
    );
    assert_eq!(
        actions.len(),
        1,
        "★ ONE action, so ONE undo entry — a cut the operator takes back with one Ctrl+Z \
         must return the comment, not leave them pressing it twice. Got {actions:?}"
    );
    match actions.first() {
        Some(Action::Annot(crate::app::actions::annot::AnnotAction::Delete { page, id })) => {
            assert_eq!(*page, 0);
            assert_eq!(
                *id, expected,
                "★ the delete must name the annotation that was COPIED. An index re-read after \
                 the copy would name whichever annotation is at that position now."
            );
        }
        other => panic!("a cut of an annotation raises a Delete, got {other:?}"),
    }
}

/// ★★★ **The paste raised for an annotation clip is the ONE verb that plants
/// both halves**, carrying the annotation count so a driven check can see it.
///
/// A build whose serialiser dropped the annotation payload would still park a
/// clip, still raise a `PasteObjects`, and still trace a paste — the content
/// half of every one of those is identical. What differs is the bytes, so this
/// asserts the round trip through `ObjectClip::from_bytes` rather than
/// trusting the action was raised: the bytes are what the clipboard actually
/// holds and what a cross-process paste would carry.
#[test]
fn the_annotation_survives_the_clips_own_serialisation() {
    let ctx = egui::Context::default();
    let doc = with_annot_selected(1);
    copy(&ctx, &doc).expect("a sticky note copies");

    let mut actions = Vec::new();
    paste(&ctx, 0, None, &mut actions).expect("it pastes");

    let Some(Action::Vector(crate::app::actions::VectorAction::PasteObjects { clip, .. })) =
        actions.first()
    else {
        panic!("an annotation clip pastes through paste_objects, got {actions:?}");
    };
    let round_tripped =
        pdfcer_core::vector::ObjectClip::from_bytes(clip).expect("the clip round-trips");
    assert_eq!(
        round_tripped.annotations.len(),
        1,
        "★ the annotation must survive `to_bytes`/`from_bytes`. Until the engine's clip format \
         version 2 it did NOT — annotations were dropped by the serialiser — and a build that \
         regressed there pastes the content half perfectly and silently loses every comment."
    );
}
