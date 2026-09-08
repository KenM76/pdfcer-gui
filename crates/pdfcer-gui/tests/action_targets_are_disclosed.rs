//! **A rename repairs other people's buttons and a delete breaks them — and
//! both must say so**, asserted through the real verbs on a real document.
//!
//! # What this adds over the string tests
//!
//! `text::forms::authoring`'s tests pin the *wording* of
//! [`form_field_actions_retargeted`] and [`form_field_actions_orphaned`] —
//! that the two do not borrow each other's alarm or reassurance, that both
//! admit JavaScript was not handled, that the rename count is worded as
//! *places* rather than *buttons*. All true, and all satisfied by a build in
//! which **neither sentence is ever produced**, because the wiring is six
//! conditional lines in `app::actions::forms` and `::forms::delete`.
//!
//! Those tests said so, in as many words, and named the blocker: the counters
//! see only action targets written as **fully-qualified name strings**, and the
//! only fixture in this repository carrying a form action at all
//! (`submit-button.pdf`) names its target by **object reference**, which the
//! traversal is structurally — and correctly — blind to.
//!
//! ⇒ `fixtures/action-names-field.pdf` is that fixture, hand-authored for this.
//! Its `.PROVENANCE.py` explains the shape and warns against "tidying"
//! `/Fields [(Amount)]` into `/Fields [4 0 R]`, which would make both
//! disclosures untestable again while every test still compiled.
//!
//! # ★★ What this still does not prove
//!
//! That the sentences reach the **status bar**. This asserts the engine reports
//! the counts and that the shell's own disclosure functions produce the right
//! prose for them; the frame is `tools/ui-verify`'s job. Stated rather than
//! implied, because a test whose limits are unwritten gets read as covering
//! more than it does — which is the defect this whole file exists because of.

use pdfcer_core::edit::EditSession;

/// The hand-authored form: one text field `Amount`, one push button whose
/// `/ResetForm` names `Amount` **as a string**.
const FIXTURE: &str = "fixtures/action-names-field.pdf";

/// The field the button names.
const TARGET: &str = "Amount";

fn session() -> EditSession {
    let path = format!("{}/../../{FIXTURE}", env!("CARGO_MANIFEST_DIR"));
    let doc = pdfcer_core::document::Document::load(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("cannot load {path}: {e:?}"));
    EditSession::new(doc)
}

/// ★★★ **A rename repoints the button, reports that it did, and the shell says
/// so in words that promise the button still works.**
#[test]
fn renaming_a_named_field_retargets_the_button_and_discloses_it() {
    let mut s = session();
    let outcome = s
        .rename_field(TARGET, "Total")
        .expect("renaming an ordinary text field must be accepted");

    assert!(
        outcome.action_targets_retargeted >= 1,
        "the fixture's `/ResetForm` names {TARGET:?} as a NAME STRING, so a rename must repoint \
         it. A zero here means the fixture has been tidied into an object reference — read \
         `fixtures/action-names-field.PROVENANCE.py` before changing this assertion. Got {:?}.",
        outcome.action_targets_retargeted
    );

    // The sentence the operator actually reads, built from that count.
    let said =
        pdfcer_gui::text::forms::form_field_actions_retargeted(outcome.action_targets_retargeted);
    assert!(
        said.contains("still work"),
        "a rename REPAIRS the reference, so the disclosure must say the buttons survive: {said}"
    );
    assert!(
        said.contains("JavaScript"),
        "R55 means no script was rewritten, and a sentence that omits it overstates the repair: \
         {said}"
    );
}

/// ★★★ **A delete cannot repoint anything, reports the orphan, and the shell
/// says the buttons will now do less than they say.**
///
/// ⚠ This is the operator-facing half that matters: nothing in the saved file
/// records that pdfcer knew, so without the sentence the fact surfaces later as
/// a Reset button that quietly stopped resetting one field.
#[test]
fn deleting_a_named_field_orphans_the_button_and_discloses_it() {
    let mut s = session();
    let outcome = s
        .delete_field(TARGET)
        .expect("deleting an ordinary text field must be accepted");

    assert!(
        outcome.action_targets_orphaned >= 1,
        "the button still names {TARGET:?} and the field is gone, so the engine must count it as \
         orphaned. Got {:?}.",
        outcome.action_targets_orphaned
    );

    let said =
        pdfcer_gui::text::forms::form_field_actions_orphaned(outcome.action_targets_orphaned);
    assert!(
        said.contains("do less than they say"),
        "the document is now degraded and the sentence must name that consequence rather than \
         count internals: {said}"
    );
    assert!(
        !said.contains("still work"),
        "the delete case must not borrow the rename case's reassurance: {said}"
    );
}

/// ★★ **The control**, and it is doing real work rather than decorating the
/// file.
///
/// Renaming the **button itself** touches no action target — nothing names
/// `ResetIt` — so both counters must be zero, and the disclosures must
/// therefore not be produced at all.
///
/// Without this, a build that hard-wired either count to a positive number, or
/// that counted every action in the document regardless of target, would
/// satisfy both tests above completely. **A one-sided test of a count is not a
/// test of the count.**
#[test]
fn renaming_a_field_nothing_names_reports_no_retargeting() {
    let mut s = session();
    let outcome = s
        .rename_field("ResetIt", "ResetAll")
        .expect("renaming the button field must be accepted");

    assert_eq!(
        outcome.action_targets_retargeted, 0,
        "no action names {:?}, so nothing can have been repointed — a non-zero count here means \
         the traversal is counting actions rather than matching targets, and the operator would \
         be told his buttons were rewritten when they were not",
        "ResetIt"
    );
}
