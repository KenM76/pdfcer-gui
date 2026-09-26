//! Tests for [`super`] — the form-field authoring, naming and lifecycle verbs.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/forms/tests.md`.

#![cfg(test)]
//
// The INNER `#![cfg(test)]` is load-bearing for the gates, not decoration.
// `check-ui-strings` and `check-theme-colors` both recognise it as the marker
// that a whole file is absent from the shipped binary — chosen over a filename
// because the property that earns the exemption is *not in the release build*,
// and a name is a restatement of that which goes stale. Without it this file
// reported 16 assertion needles as operator-facing copy.

use super::*;
use pdfcer_core::edit::EditError as E;

/// The two the operator can fix are worded; the ones they cannot are not.
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
#[test]
fn a_document_gaining_its_first_form_is_told() {
    assert!(t::adopted("A", true, true).contains("had no interactive form"));
    assert!(!t::adopted("A", true, false).contains("had no interactive form"));
}

// ===========================================================================
// The action-target disclosures REACH THE STORE — the six conditional lines
// that neither the string tests nor the engine tests could see
// ===========================================================================

/// **A rename records the retargeting sentence where the status bar reads
/// it.**
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

    // And the ordinary sentence is still first. The retargeting line is
    // additional, not a replacement: an operator who renamed a field wants to
    // be told the rename happened before being told what else moved.
    assert!(
        recorded.notes[0].contains("Renamed to"),
        "the rename's own receipt must lead: {:?}",
        recorded.notes
    );
}

/// **A rename that collides with an existing name says so, instead of the
/// funnel floor's generic shrug.**
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

/// **And the dotted name — asserted even though the panel will not let it
/// through, which is the unusual part and needs its reason stated.**
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

/// **The dotted name is refused with a sentence, on the one surface that
/// can provoke it.**
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

/// **A delete records the orphan warning**, which is the half that matters
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

/// **The control, and it is the assertion that makes the two above mean
/// something.**
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
/// **The dotted-name refusal is the ENGINE's, and this module asserts that
/// the shell reads it rather than re-deriving it.**
mod dotted_names {
    /// **`author` reads the engine's refusal, and does not model it.**
    fn code_only(src: &str) -> String {
        src.lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn author_words_the_engines_refusal_rather_than_pre_empting_it() {
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
    /// **Field authoring is NOT blocked, and this is the test that settled
    /// it.**
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
