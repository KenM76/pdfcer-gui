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

/// ★★★ **A rename that collides with an existing name says so, instead of the
/// funnel floor's generic shrug.**
///
/// # What this is really asserting, because it is not the engine's rule
///
/// That `rename_field` refuses a collision is the engine's business and is
/// asserted in the engine's own suite. What could only be asserted from here
/// is that the refusal **arrives somewhere an operator can read**, and until
/// 2026-09-12 it did not: `forms::rename` mapped only `Ok`, so every failure
/// reached `decline::floor` and came out as *"That change was refused"*.
///
/// ★★ That is the shape worth the paragraph. The bar was never blank. There
/// was always a sentence, it was always true, and it answered nothing — so no
/// report, no gate and no test had anything to catch. A missing answer dressed
/// as a present one is invisible in a way a missing sentence is not.
///
/// ★ And this is the refusal that matters, measured rather than guessed. The
/// Properties panel greys Rename on
/// `!typed.is_empty() && !typed.contains('.')`, which pre-empts every refusal
/// derivable from the typed string. A collision needs the field tree, so it is
/// the only one that can reach the engine from that control — and it is the
/// ordinary one: rename `Rev1` to `Rev2` on a form that has a `Rev2`.
///
/// It asserts [`Declined::FieldNameTaken`] — the **same** decline `adopt`
/// raises from a different engine variant — under *one fact, one wording*. If
/// a later reader splits those into two variants, this test is where the
/// reasoning is.
#[test]
fn renaming_onto_a_name_that_is_taken_says_so() {
    crate::app::status::decline::retire();
    let mut doc = crate::app::state::open_local_fixture("action-names-field.pdf");
    let before = doc.edit_epoch;

    super::rename(&mut doc, "Amount", "ResetIt");

    assert_eq!(
        doc.edit_epoch, before,
        "a refused rename may not have edited anything"
    );
    assert_eq!(
        crate::app::status::decline::recorded_for_test(),
        Some(crate::app::status::decline::Declined::FieldNameTaken),
        "the fixture holds both `Amount` and `ResetIt`, so renaming one onto the other collides.\
            `None` means the rename route still words nothing and the floor answered for it; a\
            different variant means the `RenameCollision` arm is missing from `correctable`"
    );
}

/// ★★ **And the dotted name — asserted even though the panel will not let it
/// through, which is the unusual part and needs its reason stated.**
///
/// `panels::properties::formfield` greys Rename while the typed text contains
/// a period, so an operator cannot provoke this from that control. The
/// assertion is here anyway, and NOT because more coverage is better:
///
/// ★★★ **the gate is a shell-side model of an engine rule, and this project
/// deleted another one of those the same day for drifting.**
/// `group_is_a_field` modelled a document-dependent rule and went wrong; this
/// gate models a pure string rule and cannot. But *"cannot drift"* is an
/// argument, not a guarantee — so the engine's refusal is wired behind the
/// gate, and this test is what says the wiring works. If a later reader
/// removes the gate, the sentence is already there and this test already
/// proves it.
///
/// ⇒ It calls `super::rename` directly, which is what the panel's action
/// dispatches to, minus the gate. That is the only way to reach the engine's
/// refusal from a test, and it is honest about being that.
#[test]
fn a_dotted_rename_is_worded_even_though_the_panel_greys_it() {
    crate::app::status::decline::retire();
    let mut doc = crate::app::state::open_local_fixture("action-names-field.pdf");

    super::rename(&mut doc, "Amount", "A.B");

    let recorded = crate::app::status::decline::recorded_for_test().expect(
        "`rename_field` refuses a dotted partial name unconditionally, so the `inspect_err` arm\
            must have recorded a decline on the way past",
    );
    assert_eq!(
        recorded,
        crate::app::status::decline::Declined::DottedPartialName("A.B".to_owned()),
        "the decline must name THIS refusal and carry the operator's own string. A different\
            payload means the name was re-derived somewhere instead of being read out of the\
            engine's error. Recorded: {recorded:?}"
    );
}

/// **The control: adopting with an ordinary name succeeds.**
///
/// First, and not for coverage. A refusal test alone cannot tell *"the dotted
/// guard fired"* from *"adopt refuses this fixture for some other reason"* —
/// `adopt_plan` has five refusals before it looks at the name, and a fixture
/// that tripped any of them would make the refusal test pass while proving
/// nothing. ⇒ *a uniform failure at every rung of a sweep is about the probe;
/// the baseline rung is the control.*
///
/// It also pins what the fixture IS, in a place a failure message will quote:
/// an unowned merged field-widget that adopts losslessly and recovers its own
/// `/T`. See [`crate::app::state::ORPHAN_WIDGET`].
#[test]
fn an_unowned_widget_adopts_under_an_ordinary_name() {
    crate::app::status::decline::retire();
    let mut doc = crate::app::state::open_local_fixture(crate::app::state::ORPHAN_WIDGET);
    let widget = only_widget(&doc);

    super::adopt(&mut doc, 0, widget, Some("Claimed".to_owned()));

    assert!(
        crate::app::status::decline::recorded_for_test().is_none(),
        "adopting an unowned widget under an undotted name must not decline. A decline here means one of `adopt_plan`'s five earlier refusals fired, and the dotted test below would then be passing for a reason that has nothing to do with the period"
    );
    assert!(
        super::field_names(&doc).iter().any(|n| n == "Claimed"),
        "the registration must produce a field under the supplied name; names now: {:?}",
        super::field_names(&doc)
    );
}

/// ★★★ **The dotted name is refused with a sentence, on the one surface that
/// can provoke it.**
///
/// This is what consuming the 2026-09-12 delivery means. The engine shipped
/// the guard because this shell asked for it; the shell already had the
/// `correctable` arm; so the delivery is consumed when the **chain** is proven,
/// not when the arm exists.
///
/// ★★ And it is a different test from the rename one above, which is green and
/// proves nothing about this. That drives `rename_field`, which has refused
/// dotted names for weeks, from a surface whose button is greyed while a period
/// is typed. `adopt_widget` is reached from
/// `panels::forms::tab_order::register`'s per-widget name box — **free text,
/// gated only on non-empty** — the one route in `correctable`'s reachability
/// table marked *yes*, and it had no test at all.
/// ⇒ *a green test naming the variant is not evidence about the route that can
/// actually raise it.*
///
/// What is asserted is the operator's own string coming back, not merely the
/// variant: a payload that is not `Text.2` means the name was re-derived
/// somewhere instead of being read out of the engine's error, and a re-derived
/// name is the defect this project deleted `group_is_a_field` for.
///
/// ★★★ **The `expect` is deliberately not the assertion that carries this, and
/// an `is_some()` in its place would be a check that cannot fail.** Measured,
/// by neutering `correctable`'s `DottedPartialName` arm and re-running: the
/// `expect` passed anyway and the equality went red with `left: EditRefused`.
/// [`crate::app::status::decline::before_the_verb`]'s floor records
/// `Declined::EditRefused` for **any** refused vector edit unless the verb
/// recorded something better — so on this route *something* is always recorded
/// when the engine refuses, whatever the mapping does. The equality is the
/// evidence; the `expect` only separates *"refused"* from *"authored it"*.
#[test]
fn adopting_under_a_dotted_name_is_refused_and_worded() {
    crate::app::status::decline::retire();
    let mut doc = crate::app::state::open_local_fixture(crate::app::state::ORPHAN_WIDGET);
    let widget = only_widget(&doc);

    super::adopt(&mut doc, 0, widget, Some("Text.2".to_owned()));

    let recorded = crate::app::status::decline::recorded_for_test()
        .expect("nothing AT ALL recorded means the verb did not refuse — `adopt_widget` authored `Text.2` — which is what an engine pin older than the 2026-09-12 guard does. It is not what a lost `correctable` arm does: the decline floor records `Declined::EditRefused` for any refused vector edit, and that case goes red on the equality below, not here");
    assert_eq!(
        recorded,
        crate::app::status::decline::Declined::DottedPartialName("Text.2".to_owned()),
        "the decline must name THIS refusal and carry the operator's own string. Recorded: {recorded:?}"
    );
    assert!(
        super::field_names(&doc).is_empty(),
        "the refusal must leave the document unchanged — no field, dotted or otherwise. Names now: {:?}",
        super::field_names(&doc)
    );
}

/// The fixture's single unclaimed `/Widget`, by object id.
///
/// ★★ Read through [`crate::panels::forms::tab_order::model::collect`] rather
/// than as a typed `ObjId`, and that is not fastidiousness about magic numbers:
/// **it is the derivation the panel row the operator presses uses**, so the two
/// tests above drive the id the surface would hand to
/// `FieldAction::Adopt`, not one a test author chose. A number typed into a test
/// is a claim about bytes nobody re-reads, and this project has already had a
/// harness report defects that did not exist from exactly that.
///
/// `form` is `None` for this fixture — the catalog has no `/AcroForm`, which is
/// the whole point of it — and `collect` is documented to put every widget in
/// `unclaimed` in that case. It asserts there is **one**, so a fixture that
/// grew a second widget fails here with a sentence instead of silently testing
/// whichever one came first in `/Annots`.
fn only_widget(doc: &OpenDoc) -> ObjId {
    let view = doc.session.view();
    let slots = doc.session.page_slots().expect("the fixture's page tree walks — it is five objects and its xref offsets are asserted by its own generator");
    let form = pdfcer_core::forms::parse_acroform(&view);
    let listing = crate::panels::forms::tab_order::model::collect(&view, &slots, form.as_ref());
    let found = &listing.pages[0].unclaimed;
    assert_eq!(
        found.len(),
        1,
        "`orphan-widget.pdf` holds exactly one unclaimed widget on its one page; see `fixtures/orphan-widget.PROVENANCE.py`. Found: {found:?}"
    );
    found[0].id
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
/// ★★★ **The dotted-name refusal is the ENGINE's, and this module asserts that
/// the shell reads it rather than re-deriving it.**
///
/// The loss that started all this was measured with `pdfcer` before any of it
/// was written: at engine `3ac9dd7`, a field `Text` holding "K. Mantle" plus a
/// new field named `Text.2` left one empty field and an orphaned box, with no
/// recovery. That is why it became a refusal rather than a disclosure.
///
/// It is no longer a loss. Since 2026-08-30 the engine refuses at
/// `place_new_field_deferred` — `FormAuthorError::FieldPathCrossesTerminal` —
/// before a byte is staged. On 2026-09-11 the shell's own pre-check was
/// deleted and `actions::forms::correctable` gained an arm that reads the
/// engine's variant and carries the field name the engine resolved.
///
/// # ★★★ TWO OF THE THREE TESTS THAT STOOD HERE COULD NOT HAVE FAILED
///
/// They are gone with the function, and the reason they are worth a paragraph
/// is that **the deletion is not what exposed them** — nothing did, and nothing
/// would have. Both would have gone on passing after the code they were named
/// for ceased to exist.
///
/// - `a_name_without_a_dot_is_never_examined` asserted
///   `!"Revision".contains('.')`. That is a claim about a string literal. It
///   never named the function, never opened a document, and its own comment
///   said so — *"Expressed as a doc-free call in the sibling tests below rather
///   than here"* — which is an admission that the test body asserts nothing and
///   the title is doing the work.
/// - `the_prefix_walk_stops_before_the_full_name` built the prefix list **in
///   the test** with `(1..segments.len())` and asserted the list it had just
///   built equalled the list it expected. The product's walk was a second copy
///   of the same three lines twenty feet away. Changing the product could not
///   turn it red.
///
/// ⇒ Recorded because it is the same shape as the tautology the surviving test
/// below documents, arrived at from the other direction: that one read its own
/// assertion string, these two read their own arithmetic. **A test that never
/// names the thing it is about is not testing it**, and a module title is not
/// a citation.
///
/// What replaces them is nothing, deliberately. The behaviour they gestured at
/// is the engine's and is asserted in the engine's own suite
/// (`tests/form_field_merge.rs:1241`); re-asserting it here would rebuild the
/// duplicate model whose deletion this module now documents.
mod dotted_names {
    /// **`author` reads the engine's refusal, and does not model it.**
    ///
    /// Named rather than exercised, because the operand is an operator-typed
    /// string and the assertion that matters is structural: that the refusal
    /// reaching the operator is the **engine's**, carrying the name the engine
    /// resolved, rather than a second predicate this shell evaluates first.
    ///
    /// ★★ Rewritten 2026-09-11, and the old subject is worth one line because
    /// the assertion inverted. It used to require that `author` consult a
    /// shell-side guard **before writing anything**. It now requires that it
    /// consult **nothing** before calling the verb, and word what comes back.
    /// The pre-check it defended had, by the time it was read, become wrong in
    /// the way a duplicate model always does: it refused on any prefix present
    /// in `AcroForm::fields`, where the engine refuses only on a **terminal**,
    /// so it falsely refused the mixed node. A test guarding a guard is only as
    /// right as the guard.
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
    /// Everything in `src` that is not a full-line comment.
    ///
    /// ★★★ Why a source-scanning assertion needs this, added 2026-09-12.
    ///
    /// The check below went red because `author`'s doc comment — moved into
    /// `author.rs` that day, see `tools/gates/check-orphan-docs.py` — *names*
    /// the pre-check this test forbids, in a sentence explaining that it was
    /// deleted. A flat `contains` over the whole file reads that mention as a
    /// resurrection.
    ///
    /// ⇒ *a test keyed on a name cannot tell code from commentary.* The shape
    /// is already recorded in the other direction, where prose DISCHARGED a
    /// coverage gate: twenty-five engine verbs once scored "consumed" on doc
    /// comments alone. A false red is the more confusing half, because it names
    /// a real file and a real symbol and sends the reader hunting for something
    /// that was correctly removed.
    ///
    /// ⚠ Full-line comments only. A trailing `// …` after code, and a `/* */`
    /// block, are both left in — neither occurs in this crate's style, and a
    /// stripper that tried to handle them would need to respect string literals
    /// and would become the second parser this project maintains. If that ever
    /// changes, the honest move is a real lexer, not a cleverer regex.
    fn code_only(src: &str) -> String {
        src.lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn author_words_the_engines_refusal_rather_than_pre_empting_it() {
        // ⚠ `../` — this file moved into `forms/` on 2026-09-08 and
        // `include_str!` is relative to the file that writes it.
        let src = code_only(include_str!("../forms/author.rs"));
        assert!(
            src.contains("super::correctable(error)"),
            "★ `author` must read the ENGINE's refusal and word it. Until 2026-09-11 it \
             pre-empted that refusal with a shell-side model of the rule, and by the time the \
             engine shipped the real one the model had become WRONG in the permissive \
             direction's opposite — it refused the mixed node, which the engine allows. If this \
             call moved, follow it; do not delete the assertion, and do not replace it with a \
             second model."
        );
        assert!(
            !src.contains("group_is_a_field"),
            "★★ the deleted pre-check must not come back. Its replacement is not a better \
             predicate, it is NO predicate: `correctable` reads the variant the engine raised \
             and carries the field name the engine itself resolved, which is the only answer \
             that cannot drift from the engine's. ⚠ This reads CODE only — the file's doc \
             comment names the deletion in prose, deliberately, and that is not a resurrection."
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
