//! # `app::actions::apply` tests — the funnel's own assertions
//!
//! Split out of `apply.rs` on 2026-08-26 under R2, when the placed-object
//! selection and the save-epoch work took that file past 1,500 lines. Nothing
//! moved but the tests, and they moved whole — the gate's own header is
//! explicit that the right response to it firing is to split the module, not to
//! shrink the prose.
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

#![cfg(test)]

use super::*;
use crate::app::actions::last_edit_disclosure;
use crate::app::actions::{EditDisclosure, record_edit_disclosure};
use crate::app::state::{FOUR_PAGES, open_fixture};

/// ★ **An undo is an edit, and moves the epoch like one — while an undo
/// with nothing to undo moves nothing at all.**
///
/// # The two failures this pins, and why neither is visible anywhere else
///
/// 1. **The epoch.** A build whose history arm called `EditSession::undo`
///    directly — `Arc::get_mut(&mut doc.session).map(EditSession::undo)`,
///    which is the obvious three-line version — would restore the bytes and
///    leave `edit_epoch` where it was. Every count anybody could read from
///    the engine would then be correct, and the decomposition, the
///    page-text cache, the font inventory and the canvas selection would all
///    go on describing the revision the operator just left. That is the
///    build `tools/ui-verify`'s `undo_redo_round_trip` catches from outside
///    the process; this is the half that can be caught from inside it.
/// 2. **The empty stack.** The decline must cost nothing: no epoch bump, no
///    dropped texture, no cancelled raster. A bump here would dissolve the
///    operator's selection and discard several caches to record that
///    *nothing happened* — which is `crate::app::save` §3.1's argument
///    about a save, arriving at the same answer from the other direction.
///
/// # Why `is_modified` is asserted as well as the epoch
///
/// Because the epoch alone cannot tell an undo from any other edit: it
/// counts revisions, and it only ever goes up. `EditSession::is_modified`
/// asks the **dirty set**, which is the same question a save asks, and its
/// own doc comment says in as many words that *"an edit-then-undo reports
/// `false`"*. So it is the one available proof that the document really
/// went back rather than merely forward again — and the pair of them
/// together is the whole claim: *the document is where it started, and the
/// shell knows the revision changed.*
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

    // --- ★ the undo ----------------------------------------------------
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

/// ★ **A disclosure a verb returns is live for the revision that verb
/// produced** — the wiring, driven rather than planted.
///
/// [`plant_edit_disclosure_for_test`] proves the status bar can *draw* a
/// disclosure. It cannot prove [`vector_edit`] ever *records* one, and it
/// cannot prove the stamp is right — which is the failure this test
/// exists for, because that failure is silent in both directions:
///
/// - Stamp the epoch the edit ran **against** (the pre-bump value) and the
///   sentence is invisible from the moment it is written. Nothing errors,
///   no test that plants its own value notices, and the operator simply
///   never learns their rectangle became four lines.
/// - Fail to record at all and the same thing happens, with the trace
///   still cheerfully printing `disclosures=…` — which is exactly the
///   "recorded, not disclosed" state this work was written to end.
///
/// So the edit closure here returns a disclosure list the way a real
/// `move_node` over an `re` rectangle does, and the assertion is made
/// against the epoch the *document* ends up on, read back through the
/// public accessor the bar uses.
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

/// ★ **A disclosure is shown only while it describes the revision on
/// screen.**
///
/// The staleness rule, and the whole reason nothing anywhere has to
/// remember to clear this sentence: an undo bumps the epoch, the epoch no
/// longer matches, and the bar stops drawing it. The comparison IS the
/// mechanism, so it is pinned rather than trusted — the same test, for the
/// same reason, as
/// `crate::panels::forms::edit::tests::a_disclosure_is_hidden_once_the_document_moves_past_it`.
///
/// Both directions matter and both are asserted. A *later* revision must
/// not show a note about an earlier one (the undo case, and the ordinary
/// "they carried on editing" case). An *earlier* one must not either —
/// that pairing is unreachable through `vector_edit`, which only ever
/// stamps the epoch it just produced, and it is asserted anyway because
/// the filter is what makes it unreachable.
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

/// ★★★ **A row click and a canvas click now write the same thing.**
///
/// The operator, 2026-08-26: *"when I have an object selected like text the
/// Tool tab doesn't switch to giving me the editable stuff for that object."*
///
/// The cause was three parallel notions of *"the thing I am working on"* — the
/// armed tool, a panel-local `focus` written only by the Objects panel, and the
/// canvas selection — with no bridge between them. The Properties panel read
/// the second; he had just created the third.
///
/// This asserts the binding that replaced it: the Objects panel raises
/// `Action::SelectObject`, and what it produces is an **ordinary canvas
/// selection**, indistinguishable from one made by clicking the page. That is
/// the property that makes the Properties panel, the row highlight, the
/// handles, Delete and every Format verb agree about the same object without
/// any of them being told twice.
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
        vec![Action::SelectObject {
            page: 0,
            object: Some(TargetId::Object(1)),
        }],
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

    // ★ And clicking the selected row again clears it, which is what clicking a
    // selected item does in every list in every application. The panel decides
    // WHICH of the two it is asking for; the action does what it is told.
    app.apply_actions(
        vec![Action::SelectObject {
            page: 0,
            object: None,
        }],
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
///
/// ★ A real `EditError`'s `Display` is what these two tests are defending
/// against, and paraphrasing it would have made them defend a paraphrase.
/// `vector_edit`'s error bound is `Display` and nothing more (see
/// [`super::vector_edit`]'s ★ section on why), so a bespoke type carrying the
/// engine's exact sentence is a faithful stand-in for the value the funnel
/// really meets — and it keeps the test independent of
/// `pdfcer_core::edit::EditError`, which is `#[non_exhaustive]` and whose
/// variants this crate is deliberately not permitted to name.
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

/// ★★★ **A refused edit is a sentence, never a silence** —
/// `OPERATOR_REQUESTS.md` O116.
///
/// The founding defect class of this project, pinned at the one place every
/// document change passes through. Before 2026-09-04 this arm wrote a line to
/// `PDFCER_DIAG` and stopped, so an operator who armed Edit ▸ Edit text on a
/// CAD drawing, placed a caret, typed and committed was told **nothing at
/// all** — and the engine's refusal was correct, which is what makes the
/// silence indefensible rather than merely unhelpful.
///
/// # The four properties asserted, and why each would fail invisibly
///
/// 1. **A decline is recorded.** Break the wiring and nothing errors, no other
///    test notices, and the symptom is exactly the state this test exists to
///    end — which is why it cannot be left to review.
/// 2. **The document did not move.** The sentence says *"the document is
///    unchanged"*, and that has to be true by construction rather than by
///    intention: no epoch bump, so no cache invalidation and no undo entry.
/// 3. **The verb's own sentence wins.** Six recorders fire from inside the
///    closure, and an unconditional write in the error arm would replace a
///    sentence naming a one-click remedy with one naming nothing. This is the
///    assertion that stops a future "simplification" of `BeforeTheVerb` into a
///    bare `record`.
/// 4. **Two presses are two events.** The second commit on the same
///    unsupported text has to register; see `decline::BeforeTheVerb`'s
///    repeatability section for why the take is what delivers it.
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

/// ★★★ **The sentence names no cause and carries none of the engine's own
/// words.**
///
/// Two rules that look like one and are not.
///
/// **No cause**, because there is no honest way to obtain one:
/// `pdfcer_core::edit::EditError` exposes no coarse discriminant a front end
/// may switch on, matching on its variants would be a second copy of its
/// taxonomy that drifts and then tells the operator the *wrong* reason, and
/// parsing its prose is greping a diagnostic that is theirs to reword. The
/// engine has been asked for the discriminant; until it lands, one
/// un-categorised sentence is the fallback the request itself specifies.
///
/// **No borrowed words**, because `check-ui-strings.sh`'s exclusion 3 says in
/// as many words that being a `Display` impl *"is not permission to route UI
/// text through an error type"*. That gate cannot see a `format!("{error}")`
/// that reaches a label at runtime; this can.
///
/// # ★ How the second half is asserted, and why it is not a keyword list
///
/// Every word of the engine's prose is checked against every word of the
/// sentence, and a collision fails — except for a short, explicitly-named set
/// of ordinary English that any two sentences about the same event will share.
/// A keyword list would only catch the words whoever wrote it thought of; this
/// catches **any** leak, including the one that matters most — somebody
/// appending `format!(": {error}")` to make the message "more helpful".
///
/// ★ `refused` is on the allow-list and is the interesting entry: it is the
/// plain English verb for what happened, not part of the engine's diagnostic
/// vocabulary, and both sentences are entitled to it.
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
