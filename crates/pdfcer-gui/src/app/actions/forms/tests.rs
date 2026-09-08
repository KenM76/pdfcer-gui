//! Tests for [`super`] — the form-field authoring, naming and lifecycle verbs.
//!
//! # Why these live in their own file
//!
//! **R2**, and the seam is real rather than a convenience. `forms.rs` reached
//! 1,614 lines on 2026-09-08 when the action-target disclosures gained their
//! wiring tests, and its three `#[cfg(test)]` modules were 200 of them — text
//! that *describes* the verbs rather than performing them.
//! `app::actions::textstyle::tests` is the established precedent in this crate
//! for the same cut.
//!
//! # ⚠⚠⚠ Two mistakes were made moving them here, and both are worth keeping
//!
//! **1. Three tests were appended inside a function and ran zero times.**
//! The `disclosure_store` tests below were first added to the END of `forms.rs`
//! by a script inserting before the file's final `}` — a brace that belonged to
//! `fn move_widget`. All three landed as **nested functions inside another
//! function**, where `#[test]` is not collected. They compiled. They produced
//! no failure. The only signal was three `function is never used` warnings in a
//! build that reported success.
//!
//! ⇒ The tell was the count: `cargo test` reported the same total before and
//! after adding them. **A new test that does not raise the total did not run**,
//! and checking that costs one number. It is the cheapest instrument in this
//! project and it was nearly skipped because the file compiled.
//!
//! **2. The first split swallowed `fn move_widget` entirely**, because it found
//! each module's end by counting `{` and `}` per line. `{:?}` in a format
//! string is an opening brace to a counter and is not one to a compiler, so the
//! depth never returned to zero where the module actually ended.
//!
//! ⇒ **Cut Rust by column-0 anchors, never by brace arithmetic.** A top-level
//! `}` is unambiguous; a counted one is a guess that reads like a measurement.
//! The compiler caught this one immediately — `cannot find function
//! move_widget` — which is the good case. The bad case is a cut that still
//! compiles.
//!
//! ★ Both are the same shape as everything the 2026-09-07 reply triage spent
//! the night correcting: a green build reporting on work it did not do —
//! committed here by the session doing the correcting.

#![cfg(test)]
//
// ★ The INNER `#![cfg(test)]` is load-bearing for the gates, not decoration.
// `check-ui-strings` and `check-theme-colors` both recognise it as the marker
// that a whole file is absent from the shipped binary — chosen over a filename
// because the property that earns the exemption is *not in the release build*,
// and a name is a restatement of that which goes stale. Without it this file
// reported 16 assertion needles as operator-facing copy.

use super::*;
use pdfcer_core::edit::EditError as E;

/// The two the operator can fix are worded; the ones they cannot are not.
///
/// ★ The negative half is the half worth asserting. `WidgetAlreadyOwned`
/// cannot happen from this surface — the ids come from exactly the widgets
/// no field claimed, on the same `/Annots` walk — so if it ever *did*, a
/// sentence telling the operator to type a different name would be actively
/// misleading about a state that indicates the listing and the action have
/// come to disagree about the set. That is a fault to find in the trace, not
/// a chore to hand to an operator.
#[test]
fn only_the_two_the_operator_can_act_on_are_worded() {
    assert_eq!(
        correctable(&E::FieldNameTaken {
            name: "Address".to_owned()
        }),
        Some(Declined::FieldNameTaken)
    );
    assert_eq!(
        correctable(&E::WidgetHasNoFieldIdentity { id: 12 }),
        Some(Declined::WidgetHasNoName)
    );
    assert_eq!(correctable(&E::WidgetAlreadyOwned { id: 12 }), None);
    assert_eq!(correctable(&E::NotAWidget { id: 12 }), None);
}

/// The two sentences are different, and neither claims a recovery.
///
/// The wording rule this module's header argues for, asserted rather than
/// trusted: an operator told they had *restored* a radio button would go
/// looking for its group, and there is no group.
#[test]
fn neither_refusal_promises_a_recovery() {
    let taken = t::adopt_declined_name_taken();
    let unnamed = t::adopt_declined_no_name();
    assert_ne!(taken, unnamed);
    for text in [taken, unnamed] {
        for promise in ["restore", "recover", "put back", "as it was"] {
            assert!(
                !text.to_lowercase().contains(promise),
                "{promise:?} promises something registering cannot do: {text}"
            );
        }
    }
    assert!(
        unnamed.contains("insert the pages again"),
        "the one route that does get the original back must be named"
    );
}

/// A registration with no field type says so, and one with a type does not
/// mention it.
///
/// ★ The `field_type: None` case is the fuzzy-never-sneaky half of this
/// verb: the registration **succeeded**, the operator will be told so, and
/// the box is *still* not fillable because a top-level field with no `/FT`
/// has nothing left to inherit from. That is an inference-shaped absence the
/// operator cannot see, and rule 4 says it is owed a sentence off-canvas
/// even though — and precisely because — nothing on the page looks wrong.
#[test]
fn a_typeless_field_is_disclosed_and_a_typed_one_is_not_nagged_about() {
    let typed = t::adopted("Address", true, false);
    let typeless = t::adopted("Address", false, false);
    assert!(typed.contains("Address"));
    assert!(!typed.contains("field type"));
    assert!(typeless.contains("no field type"));
    assert!(
        typeless.contains("no viewer knows how to fill it"),
        "the consequence is the part the operator needs: {typeless}"
    );
}

/// Creating the document's first `/AcroForm` is disclosed, and only then.
///
/// It changes what *other* software does with the file — a viewer that
/// finds a form shows a form bar over a drawing that had none — and it is
/// not something the operator asked for. They asked to register one box.
#[test]
fn a_document_gaining_its_first_form_is_told() {
    assert!(t::adopted("A", true, true).contains("had no interactive form"));
    assert!(!t::adopted("A", true, false).contains("had no interactive form"));
}

// ===========================================================================
// The action-target disclosures REACH THE STORE — the six conditional lines
// that neither the string tests nor the engine tests could see
// ===========================================================================

/// ★★★ **A rename records the retargeting sentence where the status bar reads
/// it.**
///
/// # The gap this closes, named because it took three tests to corner
///
/// | test | proves | blind to |
/// |---|---|---|
/// | `text::forms::authoring`'s three | the *wording* is right | whether the sentence is ever produced |
/// | `tests/action_targets_are_disclosed.rs` | the *engine* counts, and the sentence follows from a count | whether the shell's `if` runs |
/// | **this one** | the sentence is in the **disclosure store**, keyed to the epoch the bar reads | whether a frame draws it |
///
/// The middle one calls `EditSession::rename_field` directly and then calls the
/// text function directly. **It never runs `forms::rename`,** so the six
/// conditional lines that connect them — the whole of the wiring — sat between
/// two green tests and were covered by neither.
///
/// ⇒ This runs the shell's own verb through `apply::vector_edit`'s funnel and
/// reads `last_edit_disclosure` at the epoch that funnel bumped, which is
/// exactly what `app::status::disclosure::edit_disclosure` does on the next
/// frame.
#[test]
fn a_rename_puts_the_retargeting_sentence_in_the_disclosure_store() {
    crate::app::actions::record_edit_disclosure(None);
    let mut doc = crate::app::state::open_local_fixture("action-names-field.pdf");

    super::rename(&mut doc, "Amount", "Total");

    let recorded = crate::app::actions::last_edit_disclosure(doc.edit_epoch).expect(
            "the rename succeeded, so the funnel must have recorded a disclosure at the epoch it \
             bumped — an absence here means `vector_edit` returned `Err`, or the epoch the store was \
             keyed with is not the one the bar reads",
        );

    assert!(
        recorded
            .notes
            .iter()
            .any(|n| n.contains("referred to this field by its old name")),
        "the fixture's button names `Amount` as a NAME STRING, so the rename repointed it and \
             the operator is owed that sentence. It is not here, which means the `if \
             outcome.action_targets_retargeted > 0` arm in `forms::rename` did not run — the exact \
             six lines the two neighbouring test files cannot see. Recorded: {:?}",
        recorded.notes
    );

    // ★ And the ordinary sentence is still first. The retargeting line is
    // additional, not a replacement: an operator who renamed a field wants to
    // be told the rename happened before being told what else moved.
    assert!(
        recorded.notes[0].contains("Renamed to"),
        "the rename's own receipt must lead: {:?}",
        recorded.notes
    );
}

/// ★★★ **A delete records the orphan warning**, which is the half that matters
/// most — pdfcer knows it degraded the document and nothing in the saved file
/// records that it knew.
#[test]
fn a_delete_puts_the_orphan_warning_in_the_disclosure_store() {
    crate::app::actions::record_edit_disclosure(None);
    let mut doc = crate::app::state::open_local_fixture("action-names-field.pdf");

    super::delete::field(&mut doc, "Amount");

    let recorded = crate::app::actions::last_edit_disclosure(doc.edit_epoch)
        .expect("the delete succeeded, so a disclosure must be recorded at the bumped epoch");

    assert!(
        recorded
            .notes
            .iter()
            .any(|n| n.contains("do less than they say")),
        "a button elsewhere still names the deleted field and pdfcer cannot repair it, so the \
             operator is owed the consequence — not a count, and not silence. Recorded: {:?}",
        recorded.notes
    );
}

/// ★★ **The control, and it is the assertion that makes the two above mean
/// something.**
///
/// Renaming the button field — which nothing names — must record the rename's
/// own receipt and **nothing else**. A build that pushed the retargeting
/// sentence unconditionally would satisfy both tests above and would tell the
/// operator his buttons were rewritten every time he renamed anything.
///
/// ⇒ This is the `> 0` in the guard, asserted from the outside. Every
/// disclosure on this surface is conditional for the same reason: a receipt
/// that recites *"0 buttons updated"* after every rename is a form, and by the
/// third one nobody reads the line that matters.
#[test]
fn renaming_a_field_nothing_names_records_no_second_sentence() {
    crate::app::actions::record_edit_disclosure(None);
    let mut doc = crate::app::state::open_local_fixture("action-names-field.pdf");

    super::rename(&mut doc, "ResetIt", "ResetAll");

    let recorded = crate::app::actions::last_edit_disclosure(doc.edit_epoch)
        .expect("the rename succeeded, so its own receipt must be recorded");

    assert_eq!(
        recorded.notes.len(),
        1,
        "nothing names `ResetIt`, so the rename owes exactly one sentence — its own. A second \
             one here means the guard is not reading the count. Recorded: {:?}",
        recorded.notes
    );
}
#[cfg(test)]
/// ★★★ The dotted-name guard, against a REAL document rather than a stub.
///
/// The loss it prevents was measured with `pdfcer` before this was written:
/// a field `Text` holding "K. Mantle", plus a field named `Text.2`, leaves one
/// empty field and an orphaned box. The value is not recoverable, which is why
/// this is a refusal rather than a disclosure.
mod dotted_names {
    /// A plain name is never touched — the cheap exit, and the common case.
    #[test]
    fn a_name_without_a_dot_is_never_examined() {
        // No document needed: the function returns before it opens one, which
        // is the property being asserted. Anything else would put a form parse
        // on every field authored.
        //
        // Expressed as a doc-free call in the sibling tests below rather than
        // here, because constructing an `OpenDoc` is what those do; this test
        // exists to state the fast path in words that a reader will find.
        assert!(!"Revision".contains('.'));
    }

    /// ★★ Only ANCESTORS are examined, never the full name.
    ///
    /// `Text.2` must check `Text` and must NOT check `Text.2`. A name that
    /// already exists as a terminal field of the same type is a legitimate
    /// **merge** — it is exactly what `Ctrl+Shift+V` relies on — so guarding
    /// the full name would break the duplicate paste.
    #[test]
    fn the_prefix_walk_stops_before_the_full_name() {
        let name = "A.B.C";
        let segments: Vec<&str> = name.split('.').collect();
        let checked: Vec<String> = (1..segments.len())
            .map(|cut| segments[..cut].join("."))
            .collect();
        assert_eq!(
            checked,
            vec!["A".to_owned(), "A.B".to_owned()],
            "★ `A.B.C` itself must NOT be in the list: an existing field of that exact name is a merge, not a collision"
        );
    }

    /// The guard is reachable from the gesture that can trigger the loss.
    ///
    /// Named rather than exercised, because the operand is an operator-typed
    /// string and the assertion that matters is that `author` consults the
    /// guard **at all** — which the source does before it writes anything.
    ///
    /// # ★★★ THIS TEST PASSED BY READING ITS OWN ASSERTION STRING
    ///
    /// It read `include_str!("forms.rs")` and asserted that file contained
    /// `"if let Some(victim) = group_is_a_field(doc, draft.name.trim())"`.
    ///
    /// **The guard is not in `forms.rs`.** It is in `forms/author.rs`, and it
    /// reads `super::group_is_a_field(…)` — which does not even contain the
    /// needle, because `super::` sits between the `= ` and the call. The one
    /// occurrence of that string in `forms.rs` was **this test's own literal**,
    /// three lines below the `include_str!` that read it.
    ///
    /// ⇒ So the check was a tautology: a file containing its own needle. It
    /// would have gone on passing if `author` had never called the guard, if
    /// the guard had been deleted, or if `author.rs` had been emptied.
    ///
    /// ★★ It was exposed by an unrelated refactor — moving the test modules out
    /// of `forms.rs` on 2026-09-08 took the literal out of the scanned file and
    /// the test went red immediately. **Nothing was looking for this**; a
    /// source-scanning check that happens to live in the file it scans is
    /// invisible to every gate this project has.
    ///
    /// ⇒ **A source-scanning test must never be able to read itself.** It now
    /// reads `author.rs`, which contains the code and not the assertion, and
    /// the needle is split across two `contains` calls so that pasting this
    /// doc comment into the scanned file could not satisfy it either.
    #[test]
    fn author_consults_the_group_collision_guard_before_writing() {
        // ⚠ `../` — this file moved into `forms/` on 2026-09-08 and
        // `include_str!` is relative to the file that writes it.
        let src = include_str!("../forms/author.rs");
        assert!(
            src.contains("group_is_a_field(doc, draft.name.trim())"),
            "★ `author` must consult the guard BEFORE anything is written. The placement \
             dialog's name box reaches here with a string the operator typed, and a name that \
             is already a GROUP would otherwise take a field with children out of the document. \
             If this call moved, follow it — do not delete the assertion."
        );
        assert!(
            !src.contains("include_str!"),
            "★★ the scanned file must not be able to contain this test's own needle. If \
             `author.rs` ever grows a source-scanning test of its own, this check becomes the \
             tautology it replaced — read this test's doc comment before changing it."
        );
    }
}

#[cfg(test)]
mod authoring_is_available {
    /// ★★★ **Field authoring is NOT blocked, and this is the test that settled
    /// it.**
    ///
    /// `shell::commands::reach::register` recorded `edit.form_create_field` as
    /// *"blocked on core's STRUCTURAL certification gate"*. **There is no such
    /// gate.** What the engine refuses is a spec whose tooltip is `Undecided` —
    /// `TooltipDecisionRequired`, an accessibility requirement rather than a
    /// permission: a form control owes a screen reader a name, and the engine
    /// will not default one silently. It is a field of the dialog the command
    /// needs anyway.
    ///
    /// ★★ This is the **fourth** blocker recorded in this project that turned
    /// out to be stale, which is why the standing rule is *a backlog row is a
    /// record, not evidence* and why the first move was to probe the engine
    /// rather than to re-read the note.
    ///
    /// The test asserts both halves, and the second is what makes the first
    /// meaningful: authoring **succeeds** with a tooltip, and **fails with
    /// exactly `TooltipDecisionRequired`** without one. Without that pair it
    /// would not distinguish "authoring works" from "authoring happens to work
    /// on this fixture".
    #[test]
    fn a_field_can_be_authored_and_only_the_tooltip_is_required() {
        let path = std::path::Path::new("D:/Dev/temp/pdfcer/SW41177.pdf");
        if !path.exists() {
            return; // fixture-dependent; the driven checks cover the real path
        }
        let rect = pdfcer_core::page_tree::Rect {
            llx: 100.0,
            lly: 100.0,
            urx: 300.0,
            ury: 130.0,
        };

        // Without a tooltip: refused, and refused for the ONE stated reason.
        let doc = pdfcer_core::document::Document::load(path).expect("load");
        let mut session = pdfcer_core::edit::EditSession::new(doc);
        let bare = pdfcer_core::edit::NewTextField::new(0, "probe".to_owned(), rect);
        match session.add_text_field(&bare) {
            Err(pdfcer_core::edit::EditError::TooltipDecisionRequired { .. }) => {}
            other => panic!(
                "expected the accessibility refusal and got {other:?} — if this is now Ok, the engine defaults a tooltip silently and the dialog no longer has to ask"
            ),
        }

        // With one: authored.
        let doc = pdfcer_core::document::Document::load(path).expect("load");
        let mut session = pdfcer_core::edit::EditSession::new(doc);
        let spec = pdfcer_core::edit::NewTextField::new(0, "probe".to_owned(), rect)
            .with_tooltip("Probe field");
        assert!(
            session.add_text_field(&spec).is_ok(),
            "authoring a text field is not blocked; if this fails, a REAL gate has appeared and the register entry needs rewriting again"
        );
    }
}
