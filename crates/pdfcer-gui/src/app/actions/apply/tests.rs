//! # `app::actions::apply` tests — the funnel's own assertions
//!
//! Held here rather than in `apply.rs` so that the funnel stays inside R2's
//! 1,500-line ceiling. The tests moved whole: the file-size gate's own header
//! is explicit that the right response to it firing is to split the module,
//! not to shrink the prose.
//!
//! ## What these guard
//!
//! `apply` is the single funnel every document change passes through, so its
//! tests are mostly about the **protocol** rather than about any one verb: that
//! an edit bumps the epoch, that the caches it invalidates are the ones it
//! should, that a refusal is traced rather than swallowed, and that the four
//! steps happen in the one order that makes an edit undoable.
//!
//! A verb-specific assertion belongs with its verb; what belongs here is
//! anything that would still be true if every verb were replaced.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/apply/tests.md`.

#![cfg(test)]

use super::*;
use crate::app::actions::last_edit_disclosure;
use crate::app::actions::{EditDisclosure, record_edit_disclosure};
use crate::app::state::{FOUR_PAGES, open_fixture};

/// **An undo is an edit, and moves the epoch like one — while an undo
/// with nothing to undo moves nothing at all.**
#[test]
fn an_undo_is_an_edit_and_moves_the_epoch_like_one() {
    use crate::canvas::markup::{Geometry, MarkupKind};

    let mut doc = open_fixture(FOUR_PAGES);
    let opened_at = doc.edit_epoch;

    // --- an empty log costs nothing ------------------------------------
    assert!(!doc.session.can_undo(), "the fixture opens with no history");
    crate::app::actions::history::history_step(
        &mut doc,
        crate::app::actions::history::Direction::Undo,
    );
    crate::app::actions::history::history_step(
        &mut doc,
        crate::app::actions::history::Direction::Redo,
    );
    assert_eq!(
        doc.edit_epoch, opened_at,
        "a history step with an empty stack must not bump the epoch — it would discard the \
         decomposition, the page-text cache and the operator's selection to record that \
         nothing happened"
    );
    assert!(!doc.session.is_modified(), "and must change no bytes");

    // --- one real edit, through the funnel every gesture uses -----------
    let spec = crate::canvas::markup::spec_default_pen(
        MarkupKind::Rectangle,
        &Geometry::Band {
            start: (100.0, 100.0),
            end: (200.0, 160.0),
        },
    )
    .expect("a band is the Rectangle kind's own geometry"); // ui-text-exempt: test panic
    vector_edit(&mut doc, "add-markup", 0, 1, |session| {
        session.add_markup(0, &spec).map(|_| Vec::new())
    });
    let authored_at = doc.edit_epoch;
    assert_ne!(
        authored_at, opened_at,
        "the fixture edit did not take, so nothing below is testing what it says"
    );
    assert!(doc.session.is_modified(), "the document now differs");
    assert!(doc.session.can_undo());
    assert!(
        !doc.session.can_redo(),
        "authoring something is not a reason to offer a redo"
    );

    // --- the undo ------------------------------------------------------
    crate::app::actions::history::history_step(
        &mut doc,
        crate::app::actions::history::Direction::Undo,
    );
    assert_ne!(
        doc.edit_epoch, authored_at,
        "★ THE UNDO DID NOT BUMP THE EPOCH. The annotation is off the session and every \
         epoch-keyed cache still describes the revision that had it — so the canvas would go \
         on drawing the rectangle that was just taken back. See `vector_edit` step 3"
    );
    assert!(
        !doc.session.is_modified(),
        "★ the undo did not restore the document: the dirty set a save would write is still \
         non-empty"
    );
    assert!(!doc.session.can_undo(), "the log's only entry was consumed");
    assert!(doc.session.can_redo(), "…and is now redoable");

    // --- and back again ------------------------------------------------
    let undone_at = doc.edit_epoch;
    crate::app::actions::history::history_step(
        &mut doc,
        crate::app::actions::history::Direction::Redo,
    );
    assert_ne!(doc.edit_epoch, undone_at, "a redo is an edit too");
    assert!(
        doc.session.is_modified(),
        "the redo did not re-apply the annotation"
    );
    assert!(doc.session.can_undo());
    assert!(!doc.session.can_redo());
}

/// **A disclosure a verb returns is live for the revision that verb
/// produced** — the wiring, driven rather than planted.
#[test]
fn a_verbs_disclosure_is_live_for_the_revision_the_edit_produced() {
    record_edit_disclosure(None);
    let mut doc = open_fixture(FOUR_PAGES);
    let before = doc.edit_epoch;

    vector_edit(&mut doc, "move-node", 0, 1, |_session| {
        // The turbofish is `vector_edit`'s generic error type, named as the
        // engine's own for the reason the undo caller's is — see there.
        Ok::<_, pdfcer_core::edit::EditError>(vec![
            "This shape was stored as a rectangle.".to_owned(),
        ])
    });

    assert_ne!(
        doc.edit_epoch, before,
        "the edit did not bump the epoch, so nothing below is testing what it says"
    );
    let live = last_edit_disclosure(doc.edit_epoch);
    assert!(
        live.is_some(),
        "the verb's disclosure is not live for the revision now on screen \
         (epoch {before} → {}); the bar would draw nothing and the operator \
         would learn about the rewrite from a diff",
        doc.edit_epoch
    );
    assert_eq!(
        live.expect("asserted live one line above").notes,
        vec!["This shape was stored as a rectangle.".to_owned()],
        "core's sentence must reach the store unaltered"
    );
    assert!(
        last_edit_disclosure(before).is_none(),
        "the disclosure was stamped with the revision the edit ran AGAINST rather \
         than the one it produced, which makes it invisible from the moment it is \
         written"
    );

    // A second edit that discloses nothing retires the first sentence —
    // both by the epoch and by clearing the slot outright.
    vector_edit(&mut doc, "move-node", 0, 1, |_session| {
        Ok::<_, pdfcer_core::edit::EditError>(Vec::new())
    });
    assert!(
        last_edit_disclosure(doc.edit_epoch).is_none(),
        "an edit with nothing to disclose must leave no sentence behind"
    );
    record_edit_disclosure(None);
}

/// **A disclosure is shown only while it describes the revision on
/// screen.**
#[test]
fn a_disclosure_is_hidden_once_the_document_moves_past_it() {
    record_edit_disclosure(Some(EditDisclosure {
        epoch: 7,
        notes: vec!["This shape was stored as a rectangle.".to_owned()],
    }));
    assert!(last_edit_disclosure(7).is_some());
    assert!(
        last_edit_disclosure(8).is_none(),
        "a later revision must not show a note about an earlier one"
    );
    assert!(last_edit_disclosure(6).is_none());

    // An edit that disclosed nothing draws no sentence, at any epoch.
    // `vector_edit` records `None` for this case rather than an empty
    // list, so the filter here is belt and braces — and it is exactly the
    // belt that stops an empty line appearing under every drag, which
    // would train the operator to ignore the ones that matter.
    record_edit_disclosure(Some(EditDisclosure {
        epoch: 7,
        notes: Vec::new(),
    }));
    assert!(
        last_edit_disclosure(7).is_none(),
        "an empty disclosure must draw no sentence"
    );
    record_edit_disclosure(None);
}

/// **A row click and a canvas click write the same thing.**
#[test]
fn selecting_from_the_objects_panel_produces_an_ordinary_canvas_selection() {
    use crate::canvas::target::TargetId;

    let mut app = crate::app::PdfcerApp::new();
    app.open_path(crate::panels::objects::test_support::engine_fixture(
        FOUR_PAGES,
    ));
    let Status::Open(_) = &app.status else {
        panic!("the fixture opens");
    };

    app.apply_actions(
        vec![Action::Selection(
            crate::app::actions::selecting::SelectionAction::SelectObject {
                page: 0,
                object: Some(TargetId::Object(1)),
            },
        )],
        1.0,
    );

    let Status::Open(doc) = &app.status else {
        unreachable!("still open")
    };
    assert_eq!(
        doc.selection.object_indices_on(0),
        vec![1],
        "a row click must produce a selection the canvas and every panel can read — not a \
         second, private notion of what is being worked on"
    );

    // And clicking the selected row again clears it, which is what clicking a
    // selected item does in every list in every application. The panel decides
    // WHICH of the two it is asking for; the action does what it is told.
    app.apply_actions(
        vec![Action::Selection(
            crate::app::actions::selecting::SelectionAction::SelectObject {
                page: 0,
                object: None,
            },
        )],
        1.0,
    );
    let Status::Open(doc) = &app.status else {
        unreachable!("still open")
    };
    assert!(
        doc.selection.is_empty(),
        "clicking the already-selected row must deselect"
    );
}

// =======================================================================
// The worded decline — `OPERATOR_REQUESTS.md` O116
// =======================================================================

/// The engine's own prose for the refusal that produced **O116**, kept
/// verbatim.
struct SymbolicFontRefusal;

impl std::fmt::Display for SymbolicFontRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "R-INV-2: font 'AAAAAA+JetBrainsMono-Regular' is symbolic with a built-in/custom \
             cmap and no usable /Encoding (§9.6.6.4 Branch B ignores /Encoding); its code-glyph \
             relation lives inside the embedded program, which pdfcer-core does not parse (R21). \
             Editing is refused.",
        )
    }
}

/// **A refused edit is a sentence, never a silence** —
/// `OPERATOR_REQUESTS.md` O116.
#[test]
fn a_refused_edit_is_a_sentence_rather_than_a_silence() {
    use crate::app::status::decline;

    decline::retire();
    let mut doc = open_fixture(FOUR_PAGES);
    let before = doc.edit_epoch;

    vector_edit(&mut doc, "edit-text", 0, 1, |_session| {
        Err::<Vec<String>, _>(SymbolicFontRefusal)
    });

    assert_eq!(
        decline::recorded_for_test(),
        Some(decline::Declined::EditRefused),
        "the engine refused and the operator was told nothing — the founding defect class, \
         reachable from the one funnel every edit passes through"
    );
    assert_eq!(
        doc.edit_epoch, before,
        "a refusal must cost nothing: the sentence claims the document is unchanged, and that \
         claim is only true while the error arm bumps no epoch"
    );

    // 3 — the verb spoke for itself, so the floor yields.
    decline::retire();
    vector_edit(&mut doc, "resize-annotation", 0, 1, |_session| {
        decline::record_resize_not_rebuildable(true);
        Err::<Vec<String>, _>(SymbolicFontRefusal)
    });
    assert_eq!(
        decline::recorded_for_test(),
        Some(decline::Declined::ResizeNotRebuildable { uniform: true }),
        "the funnel overwrote a sentence that names a one-click remedy with one that names \
         nothing at all"
    );

    // 4 — and pressing commit again on the same unsupported text is a second
    // event, with no dispatcher in between to retire the first.
    vector_edit(&mut doc, "edit-text", 0, 1, |_session| {
        Err::<Vec<String>, _>(SymbolicFontRefusal)
    });
    assert_eq!(
        decline::recorded_for_test(),
        Some(decline::Declined::EditRefused),
        "the second commit was swallowed, or answered with the previous gesture's sentence"
    );

    decline::retire();
}

/// **The sentence names no cause and carries none of the engine's own
/// words.**
#[test]
fn the_sentence_names_no_cause_and_borrows_none_of_the_engines_words() {
    let sentence = crate::text::status::edit_declined_by_engine();
    let lowered = sentence.to_lowercase();

    // --- no cause -------------------------------------------------------
    //
    // The four buckets the engine was asked for — unsupported font, structure
    // frozen, not found, other — plus the diagnostic apparatus of the refusal
    // that produced O116. None of it may appear, because this shell does not
    // know which of them is true.
    for forbidden in [
        "font",
        "encoding",
        "cmap",
        "glyph",
        "symbolic",
        "signed",
        "certified",
        "encrypted",
        "structure",
        "not found",
        "r-inv",
        "r21",
    ] {
        assert!(
            !lowered.contains(forbidden),
            "the sentence names a cause (`{forbidden}`) this shell cannot know: {sentence}"
        );
    }

    // --- no borrowed words ----------------------------------------------
    //
    // Ordinary English two sentences about one event will share. Deliberately
    // short: every addition here weakens the assertion, so an entry earns its
    // place by being a word no diagnostic vocabulary owns.
    const ORDINARY: &[&str] = &[
        "a", "an", "and", "does", "in", "is", "it", "its", "no", "not", "refused", "the", "was",
        "which", "with",
    ];
    let strip = |w: &str| {
        w.trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase()
    };
    let ours: Vec<String> = sentence.split_whitespace().map(strip).collect();
    for theirs in SymbolicFontRefusal.to_string().split_whitespace() {
        let word = strip(theirs);
        if word.is_empty() || ORDINARY.contains(&word.as_str()) {
            continue;
        }
        assert!(
            !ours.contains(&word),
            "the operator's sentence carries the engine's own word `{word}` — diagnostic prose \
             has reached a label, which `check-ui-strings.sh`'s exclusion 3 forbids by name: \
             {sentence}"
        );
    }
}
