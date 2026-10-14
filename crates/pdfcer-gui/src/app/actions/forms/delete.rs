//! # `app::actions::forms::delete` — the structural delete verbs, and the
//! gate that is the last door to them
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/forms/delete.md`.

use crate::app::state::OpenDoc;

/// **Delete a whole field, with every widget it draws.**
pub(in crate::app::actions) fn field(doc: &mut OpenDoc, field: &str) {
    if refused(doc, "delete-field", field) {
        return;
    }
    let before = doc.edit_epoch;
    crate::app::actions::apply::vector_edit(doc, "delete-field", 0, 1, |session| {
        session.delete_field(field).map(|outcome| {
            let mut lines = vec![crate::text::forms::form_field_deleted(
                outcome.widgets_removed,
            )];
            // Rule 4, and the sharper half of it: pdfcer knows it just
            // broke buttons elsewhere and CANNOT repair them.
            //
            // A rename can repoint an action, because the field still exists
            // under a known name. A deletion cannot — there is no name left to
            // point at — so the engine counts the references and repairs
            // nothing. Its own words: *"each one is a button that will do less
            // than it says when pressed."*
            //
            // Nothing in the saved file records that pdfcer knew. Without
            // this sentence the operator discovers it when a Reset button
            // quietly stops resetting one field, which is not a thing anybody
            // notices until it matters.
            if outcome.action_targets_orphaned > 0 {
                lines.push(crate::text::forms::form_field_actions_orphaned(
                    outcome.action_targets_orphaned,
                ));
            }
            lines
        })
    });
    clear_selection_if_edited(doc, before);
}

/// **Delete one widget, leaving the field.**
pub(in crate::app::actions) fn widget(doc: &mut OpenDoc, field: &str, widget: usize) {
    if refused(doc, "delete-widget", field) {
        return;
    }
    let before = doc.edit_epoch;
    crate::app::actions::apply::vector_edit(doc, "delete-widget", 0, 1, |session| {
        session.delete_widget(field, widget).map(|outcome| {
            vec![if outcome.field_removed {
                crate::text::forms::form_widget_deleted_last()
            } else {
                crate::text::forms::form_widget_deleted()
            }]
        })
    });
    clear_selection_if_edited(doc, before);
}

/// **Delete several boxes as one undo step** — the selected fields of a
/// multi-field selection. The engine deletes one widget per command (G192),
/// so each goes by `delete_widget` and the commands pushed are folded by
/// `coalesce_last`.
///
/// A field's later boxes are renumbered when an earlier one goes, so the
/// boxes are deleted highest index first within each field.
pub(in crate::app::actions) fn widgets(
    doc: &mut OpenDoc,
    widgets: &[crate::app::state::SelectedField],
) {
    let Some(first) = widgets.first() else {
        return;
    };
    if refused(doc, "delete-widgets", &first.field) {
        return;
    }
    let mut order: Vec<(&str, usize)> = widgets
        .iter()
        .map(|w| (w.field.as_str(), w.widget))
        .collect();
    order.sort_by(|a, b| a.0.cmp(b.0).then(b.1.cmp(&a.1)));
    order.dedup();
    let before = doc.edit_epoch;
    let mut report = (0_usize, false);
    crate::app::actions::apply::vector_edit(
        doc,
        "delete-widgets",
        first.page,
        order.len(),
        |session| commit(session, &order, &mut report),
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "delete-widgets-applied deleted={}/{} one_step={}",
            report.0,
            order.len(),
            report.1
        )
    });
    clear_selection_if_edited(doc, before);
}

/// Delete each box in `order`, then fold. `report` is (deleted, folded), set
/// even when a verb refuses part-way, since what was deleted stays deleted.
fn commit(
    session: &mut pdfcer_core::edit::EditSession,
    order: &[(&str, usize)],
    report: &mut (usize, bool),
) -> Result<Vec<String>, pdfcer_core::edit::EditError> {
    use pdfcer_core::edit::{CommandKind, EditError, MAX_UNDO_DEPTH};
    if order.len() > MAX_UNDO_DEPTH {
        return Err(EditError::SelectionTooLargeForOneUndo {
            targets: order.len(),
            limit: MAX_UNDO_DEPTH,
        });
    }
    let (mut pushed, mut fields_removed, mut failure) = (0_usize, 0_usize, None);
    for &(field, widget) in order {
        match session.delete_widget(field, widget) {
            Ok(gone) => {
                pushed += 1;
                fields_removed += usize::from(gone.field_removed);
            }
            Err(e) => {
                failure = Some(e);
                break;
            }
        }
    }
    let folded = session.coalesce_last(pushed, CommandKind::DeleteFormField);
    *report = (pushed, folded);
    let mut said = vec![crate::text::forms::form_widgets_deleted(
        pushed,
        fields_removed,
    )];
    if let Some(e) = failure {
        if pushed == 0 {
            return Err(e);
        }
        said.push(crate::text::forms::form_widgets_partly_deleted(
            pushed,
            order.len(),
            &e.to_string(),
        ));
    }
    if !folded {
        said.push(crate::text::markup::deleted_not_one_undo().to_owned());
    }
    Ok(said)
}

/// **Decline a structural form delete in WORDS, and keep the selection.**
fn refused(doc: &OpenDoc, label: &str, field: &str) -> bool {
    if !crate::panels::properties::formfield::document_refuses_delete(doc) {
        return false;
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // `-declined`, NOT the bare `{label}`: `tools/gates/check-trace-names.py`
            // forbids a module's own line from sharing its first token with a
            // `vector_edit` funnel label, and both labels passed here are such
            // labels. A harness asking `last("delete-widget")` would otherwise
            // read whichever of the two lines came last.
            "{label}-declined field={field} reason=structural-form-refusal"
        )
    });
    crate::app::status::decline::record_field_delete_refused();
    true
}

/// **Clear the field selection only if the edit actually landed.**
fn clear_selection_if_edited(doc: &mut OpenDoc, epoch_before: u64) {
    if doc.edit_epoch != epoch_before {
        doc.selected_field = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{SelectedField, open_local_fixture};
    use crate::app::status::decline::{Declined, recorded_for_test};

    /// The `/Sig` field both certified fixtures carry, merged with its widget
    /// on page 1. See `tools/gen-certified-fixture.py`.
    fn certifier() -> SelectedField {
        SelectedField {
            field: "Certifier".to_owned(),
            widget: 0,
            page: 0,
        }
    }

    /// **The one fixture that is certified AND nested**, built by
    /// `tools/gen-certified-nested-fixture.py`. See
    /// [`the_certified_nested_fixture_is_both_certified_and_nested`] for what it
    /// has to be, and that script's header for why nothing already on disk was
    /// it.
    const CERTIFIED_NESTED: &str = "certified-nested-form.pdf";

    /// **The fixture contract for `certified-nested-form.pdf`, asserted
    /// with the engine rather than by eye.**
    #[test]
    fn the_certified_nested_fixture_is_both_certified_and_nested() {
        // 1 — it loads. `open_local_fixture` asserts the file is on disk, calls
        //     `Document::load` and parses the page tree; any of the three
        //     failing panics here rather than further down.
        let doc = open_local_fixture(CERTIFIED_NESTED);

        // 2 — the certification refuses restructuring.
        assert!(
            doc.session.deletion_refusal().is_some(),
            "the fixture is not certified, or its `/Perms /DocMDP` is not enforced: \
             `forbids_structural_change()` is `perms_enforced && signatures > 0`, so a \
             missing catalog `/Perms` entry OR a signature dictionary the census cannot \
             see leaves every structural gate open and phase F with nothing to withhold"
        );

        // 3 — and the field-name tree has an interior for it to withhold
        //     controls over. THE half every previous attempt got wrong.
        let view = doc.session.view();
        let form = pdfcer_core::forms::parse_acroform(&view)
            .expect("the fixture carries an `/AcroForm` with fields");
        let groups: Vec<&str> = form
            .groups
            .iter()
            .map(|node| node.fully_qualified_name.as_str())
            .collect();
        assert_eq!(
            groups,
            ["Personal.Address", "Personal"],
            "★★★ `AcroForm::groups` must be non-empty AND two levels deep. Empty is the \
             failure that produced this fixture: `panels::forms::groups::section` returns \
             before drawing anything when it is, so the driven check finds no arm control \
             and passes having tested nothing. `walk_field` records a node only at its \
             `!child_fields.is_empty() && widget_kids.is_empty()` early return, so a bare \
             widget added to `Personal` or to `Address` would empty this list without \
             changing anything a reader would notice. Order is post-order, deepest first, \
             per core's own note on the field"
        );
        // The cascade the two-level shape exists for: three terminals hang off
        // the root, and one of them is a level shallower than the other two.
        let terminals: Vec<&str> = form
            .fields
            .iter()
            .map(|f| f.fully_qualified_name.as_str())
            .filter(|name| name.starts_with("Personal."))
            .collect();
        assert_eq!(
            terminals,
            [
                "Personal.Address.Zip",
                "Personal.Address.City",
                "Personal.Name"
            ],
            "deleting `Personal` must be a cascade the operator cannot predict — three \
             terminals and a second grouping node nobody named. `Personal.Name` sits one \
             level shallower on purpose: a walk with a single shared depth counter gets \
             exactly one of the two depths wrong, and a uniform tree would let that through"
        );

        // 4 — while FILLING is still permitted, which is what `/P 2` buys and
        //     what makes 2 and 3 a withholding rather than a dead panel.
        assert!(
            doc.session.fill_refusal().is_none(),
            "★ filling is refused too, so the two gates no longer disagree and this fixture \
             cannot tell them apart. `/P 1` is the value that does this — \
             `check_certification_for_fill` refuses only at permission 1 — and a fixture at \
             `/P 1` passes phase F whether or not the shell distinguishes a structural \
             refusal from a total one"
        );
    }

    /// **A refused delete keeps the selection AND says something.**
    #[test]
    fn a_refused_widget_delete_keeps_the_selection_and_words_itself() {
        let mut doc = open_local_fixture("certified-comments.pdf");
        doc.selected_field = Some(certifier());
        let before = doc.edit_epoch;

        widget(&mut doc, "Certifier", 0);

        assert_eq!(doc.edit_epoch, before, "nothing may have been edited");
        assert_eq!(
            doc.selected_field,
            Some(certifier()),
            "the selection was cleared by a delete that did not happen — the \
             Properties panel's sentence explaining the refusal is drawn from it, \
             so this is the silence destroying its own explanation"
        );
        assert_eq!(
            recorded_for_test(),
            Some(Declined::FieldDeleteRefused),
            "a refusal must be a sentence, never a silence: `vector_edit`'s Err \
             arm writes one trace line and says nothing to the operator"
        );
    }

    /// **[`field`] has the identical shape, and it was checked rather
    /// than assumed.**
    #[test]
    fn a_refused_field_delete_keeps_the_selection_and_words_itself() {
        let mut doc = open_local_fixture("certified-comments.pdf");
        doc.selected_field = Some(certifier());
        let before = doc.edit_epoch;

        field(&mut doc, "Certifier");

        assert_eq!(doc.edit_epoch, before, "nothing may have been edited");
        assert_eq!(doc.selected_field, Some(certifier()));
        assert_eq!(recorded_for_test(), Some(Declined::FieldDeleteRefused));
    }

    /// **The uncertified twin still deletes**, which is what makes the two
    /// tests above evidence rather than a tautology.
    #[test]
    fn an_uncertified_document_still_deletes_and_then_clears_the_selection() {
        let mut doc = open_local_fixture("threaded-comments.pdf");
        doc.selected_field = Some(certifier());
        let before = doc.edit_epoch;

        widget(&mut doc, "Certifier", 0);

        assert_ne!(
            doc.edit_epoch, before,
            "the gate refused on the uncertified twin — an approval signature is \
             not an enforced certification, and a build that refuses here \
             withholds Delete from every signed document"
        );
        assert!(
            doc.selected_field.is_none(),
            "on success the selection must go: the widget it names no longer exists"
        );
    }

    /// Both boxes of one radio group plus a text field go in one undo step:
    /// the second radio box is found after the first renumbers it, and one
    /// Undo brings all three back.
    #[test]
    fn several_boxes_delete_as_one_undo_step() {
        let mut doc = open_local_fixture("all-field-kinds.pdf");
        let pick = |field: &str, widget| SelectedField {
            field: field.to_owned(),
            widget,
            page: 0,
        };
        let names = |doc: &OpenDoc| -> Vec<String> {
            pdfcer_core::forms::parse_acroform(&doc.session.view())
                .expect("a form")
                .fields
                .iter()
                .map(|f| f.fully_qualified_name.clone())
                .collect()
        };
        let before = names(&doc);
        widgets(
            &mut doc,
            &[
                pick("RadioGroup", 0),
                pick("TextOne", 0),
                pick("RadioGroup", 1),
            ],
        );
        let after = names(&doc);
        assert_eq!(after.len() + 2, before.len(), "{after:?}");
        assert!(!after.iter().any(|n| n == "RadioGroup" || n == "TextOne"));
        std::sync::Arc::get_mut(&mut doc.session)
            .expect("sole owner in a test")
            .undo()
            .expect("one undo");
        assert_eq!(
            names(&doc),
            before,
            "one Undo did not restore all three boxes"
        );
    }
}
