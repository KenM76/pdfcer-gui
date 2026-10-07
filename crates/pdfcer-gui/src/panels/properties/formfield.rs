//! # `panels::properties::formfield` — the properties of a form field clicked
//! on the page
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/formfield.md`.

use crate::app::actions::forms::FieldAction;
use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::formfield as t;
use pdfcer_core::object::Object;

/// The section's rect, for `ui-verify`.
const REGION: &str = "properties.form_field";
/// The **Rename** control's rect, published only when the control is drawn.
const REGION_RENAME: &str = "properties.form_field.rename";
/// The **Delete field** control's rect, published only when the control is
/// drawn. See [`REGION_RENAME`].
const REGION_DELETE: &str = "properties.form_field.delete";
/// The per-frame census of what the two structural gates answered.
///
/// Written whether or not either control is drawn, which is what makes the
/// regions above readable as evidence rather than as noise.
const TRACE_GATES: &str = "form-field-gates";
/// The region the refusal sentence publishes, and **only** when it is drawn.
const REGION_DELETE_REFUSED: &str = "properties.form_field.delete_refused";

/// **Would deleting the selected form field be refused right now?**
#[must_use]
pub fn refuses_delete(doc: &OpenDoc) -> bool {
    doc.selected_field.is_some() && document_refuses_delete(doc)
}

/// **Would deleting ANY form field of this document be refused?** —
/// [`refuses_delete`] with the selection question taken out of it.
#[must_use]
pub fn document_refuses_delete(doc: &OpenDoc) -> bool {
    doc.session.deletion_refusal().is_some()
}

/// Draw the selected form field's properties, if one is selected.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    state: &mut PanelsState,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(selected) = doc.selected_field.clone() else {
        return false;
    };
    let view = doc.session.view();
    let Some(form) = pdfcer_core::forms::parse_acroform(&view) else {
        return false;
    };
    let Some(field) = form
        .fields
        .iter()
        .find(|f| f.fully_qualified_name == selected.field)
    else {
        return false;
    };

    let epoch = doc.edit_epoch;

    // R83 — ASKED HERE, ONCE, BEFORE EITHER CONTROL IS DRAWN, AND EACH
    // CONTROL ASKS ITS OWN QUESTION.
    //
    // Both are **pure queries**: they read the signature census and the trailer
    // and mutate nothing, so they are safe to call every frame from a UI, and
    // core says so in as many words.
    //
    // # Why two calls and not one, when the two answers are identical today
    //
    // They are. `rename_refusal` and `deletion_refusal` both delegate to
    // `structural_form_refusal`, and core's doc comment says outright that a
    // shell *"could call that one and be correct"* — and then says it should
    // not, in terms this file is the exact instance of:
    //
    // > the two gates *happen* to be computable together and are answers to
    // > different questions, and a call site that asks the wrong question is
    // > correct only until the answers diverge — at which point it is wrong
    // > silently, in a control that stays enabled while its verb refuses.
    //
    // > A GUI disabling a Rename button through a method named
    // > `deletion_refusal` is that hazard with the name spelled out at the call
    // > site.
    //
    // The coupling is explicit on core's side precisely so that if a future
    // spec nuance separates renaming from deletion, the split happens THERE,
    // once, and every caller keeps asking its own question. Two lines here is
    // the entire price of that.
    //
    // The same panel already carries the measured version of this argument the
    // other way round: `crate::panels::forms` gates its Flatten control on
    // `flatten_refusal` after a period of borrowing `deletion_refusal`, because
    // flatten additionally creates page content and carries a guard deletion
    // does not — two checks of three, which works until it does not, on
    // documents that are not exotic.
    //
    // The delete half is asked through [`refuses_delete`] rather than
    // through `doc.session.deletion_refusal()` inline, and that indirection is
    // the fix rather than a tidy-up. It was inline here, and the three other
    // doors to the same verb — the condition behind `format.delete`'s
    // `visible_when`, the `canvas.field` menu, and the Delete key's rung 0 —
    // asked either the WRONG query or none at all. A sentence derived here
    // while a control is derived elsewhere is how a panel comes to explain a
    // refusal beside a live button that performs it.
    let rename_refusal = doc.session.rename_refusal();
    let delete_refused = refuses_delete(doc);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        // Written EVERY frame this section draws, refused or not — see
        // `REGION_RENAME` for why the regions are only readable as evidence
        // when this line is unconditional.
        format!(
            "{TRACE_GATES} rename_refused={} delete_refused={}",
            u8::from(rename_refusal.is_some()),
            u8::from(delete_refused),
        )
    });

    // No `.strong()` — R84 / DEFECTS.md D11: no theme this project ships
    // renders it legibly on a panel.
    ui.label(t::heading());
    ui.add_space(4.0);

    facts(ui, field, &selected);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    rename_row(ui, state, &selected, rename_refusal.is_some(), actions);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    //
    // `field.clone()` is deliberately NOT taken: the section reads the field
    // it is handed and raises actions, so the borrow ends with the frame.
    super::fieldedit::section(
        ui,
        field,
        states_own_quadding(doc, field),
        &selected.field,
        state,
        epoch,
        actions,
    );
    ui.add_space(6.0);
    // The WIDGET half, directly under the field half, in the engine's own
    // scope order: what belongs to the field, then what belongs to this one
    // box. `widget_scope_note` explains the distinction in the one state where
    // it is visible — a field drawn in more than one place.
    super::widgetedit::section(
        ui,
        field,
        &selected,
        widget_dash(field, selected.widget),
        state,
        epoch,
        actions,
    );
    ui.add_space(6.0);
    super::fieldscripts::section(ui, doc, &form, field, actions);
    ui.add_space(6.0);
    delete_row(ui, field, &selected, delete_refused, actions);
    ui.add_space(6.0);
    // What is left out of reach, and it is now the WIDGET half rather than
    // the field half. See `text::panels::formfield::not_editable_note` for the
    // sentence this replaced and why it was worse than a gap.
    ui.small(t::not_editable_note());
    ui.add_space(6.0);
    ui.separator();
    //
    // `max_rect` is the space a `Ui` is ALLOWED to use, not the space it took.
    // Published before anything is drawn, it reported
    // `[[786, 465] - [1086, 647]]` on the operator's own layout while this
    // section's own controls were at y = 735 — **a rect naming a different
    // panel entirely**, because the Properties dock's slot begins below the
    // Objects panel and `max_rect` had not been narrowed to it yet.
    //
    // Nothing failed. The region was declared, so every check asking *"did the
    // section draw?"* answered yes and was right. What broke was the second
    // thing a section rect is for: `ui-verify` scrolls **at** it, and a wheel
    // event aimed at that centre landed in the **Objects list** and scrolled
    // that instead — so a check hunting for controls below the fold scrolled
    // six times, moved nothing, and reported the controls missing.
    //
    // ⇒ **A region must name where the thing IS, not where it could have
    // been.** `min_rect` after drawing is the occupied space, which is the only
    // rect that is true of what an operator can see and point at.
    crate::diag::ui_rect(REGION, ui.min_rect());
    true
}

/// The selected widget's border dash, from the engine's `Widget::border_dash`.
fn widget_dash(
    field: &pdfcer_core::forms::Field,
    widget: usize,
) -> crate::canvas::markup::linestyle::DashReading {
    use crate::canvas::markup::linestyle::{DashReading, of_widget_dash};
    let Some(w) = field.widgets.get(widget) else {
        return DashReading::Solid;
    };
    let dashed = w
        .border
        .as_ref()
        .is_some_and(|b| b.style == pdfcer_core::edit::BorderStyle::Dashed);
    of_widget_dash(w.border_dash.as_ref(), dashed)
}

/// The read-only facts: what this field is, where it is, and what it holds.
fn facts(
    ui: &mut Ui,
    field: &pdfcer_core::forms::Field,
    selected: &crate::app::state::SelectedField,
) {
    row(ui, &t::label_name(), &field.fully_qualified_name);
    row(ui, &t::label_type(), &t::field_type(field));
    // 1-based, because a page number in a UI is what the operator reads off the
    // page strip and every other surface in this shell states it that way.
    row(
        ui,
        &t::label_page(),
        &t::page_number(selected.page.saturating_add(1)),
    );
    // Only when there is more than one, because "1 box" is noise on the
    // overwhelming majority of fields and the number is only interesting as a
    // warning: a field drawn in three places is one an operator can change from
    // three pages without realising.
    let widgets = field.widgets.len();
    if widgets > 1 {
        row(
            ui,
            &t::label_boxes(),
            &t::box_count(widgets, selected.widget),
        );
    }
    if let Some(value) = t::field_value(field) {
        row(ui, &t::label_value(), &value);
    }
    // The flags, as a single line naming only the ones that are set. A grid of
    // greyed checkboxes would look editable and is not, which is the one
    // reading this panel must not invite.
    if let Some(flags) = t::field_flags(field) {
        row(ui, &t::label_flags(), &flags);
    }
}

/// One label-and-value line.
fn row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::Label::new(value).truncate())
            .on_hover_text(value);
    });
}

/// The rename draft and its button.
fn rename_row(
    ui: &mut Ui,
    state: &mut PanelsState,
    selected: &crate::app::state::SelectedField,
    refused: bool,
    actions: &mut Vec<Action>,
) {
    if refused {
        ui.label(t::rename_refused());
        return;
    }
    ui.label(t::rename_label());
    // The draft is seeded from the selection and re-seeded when the selection
    // changes, so clicking a second field does not leave the first field's name
    // sitting in the box waiting to be applied to the wrong one. That is the
    // failure this two-field state exists to prevent, and it is why the key is
    // stored beside the draft rather than inferred.
    let draft = state.field_rename_mut(&selected.field);
    let response = ui.add(
        // escape-disposition: keeps-draft — typed straight into the panel's own
        // rename draft, which is re-seeded only when the selection changes.
        egui::TextEdit::singleline(draft)
            .desired_width(f32::INFINITY)
            .char_limit(crate::canvas::formfield::draft::NAME_MAX),
    );
    // The partial name, NOT the qualified one. `rename_field` takes a
    // partial name and rebuilds the qualified one from the parent chain, so a
    // dotted string typed here would author a `/T` containing a dot — a field
    // no reader, including pdfcer, can address again.
    let typed = draft.trim().to_owned();
    //
    // The argument was sound. It was also **exactly the argument the deleted
    // shim's author would have written**, which is why it went to the engine as
    // a decision-058 workaround report rather than being filed as settled. The
    // reply shipped `validate_partial_name` within the hour and reported that
    // making the rule askable had immediately exposed a live divergence:
    // `adopt_widget` and `sign` accepted `a..b` where `rename_field` refused it.
    // Three enforcement sites, two behaviours, unreported and unreportable
    // while the rule could only be enforced and never asked.
    //
    // What this buys beyond being correct today is that the hover can name
    // the case that actually applies. The old gate could only report that one
    // of its two clauses had failed, so an operator looking at an EMPTY box —
    // the state this panel opens in — was told to type a name with no dots in
    // it.
    //
    // ⚠ The engine's refusal is still wired BEHIND this gate:
    // `FormAuthorError::DottedPartialName` -> `actions::forms::correctable` ->
    // `Declined::DottedPartialName` -> `fieldclip::name_is_a_path`. Removing the
    // gate would move the disclosure from hover to status bar, not silence the
    // rule. Said here because the next reader's question is *is the refusal
    // handled if I delete this?* and the answer is yes.
    //
    // Greying is R9-legal: *you have not typed a usable name yet* is temporary
    // and operator-fixable, and it is explained on hover a few lines down.
    let refusal = pdfcer_core::forms_author::validate_partial_name(&typed).err();
    let ready = refusal.is_none();
    let commit = ui.add_enabled(ready, egui::Button::new(t::rename_button()));
    // Published only on the path where the control exists — see
    // `REGION_RENAME`. Greying is still correct HERE: "you have not typed a
    // usable name yet" is exactly the temporary, operator-fixable condition R9
    // reserves greying for, and it is explained on hover two lines down.
    crate::diag::ui_rect(REGION_RENAME, commit.rect);
    let pressed = commit.clicked();
    // `if let` rather than `if !ready`: the two are the same condition, and
    // binding the refusal is what makes it impossible to word a hover for a
    // state the engine did not report.
    if let Some(refusal) = &refusal {
        commit.on_disabled_hover_text(t::rename_disabled(refusal));
    }
    // Enter in the box commits, because a single-field form with a button
    // beside it is the one place an operator always tries Enter first.
    let entered = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if ready && (pressed || entered) {
        actions.push(
            FieldAction::Rename {
                from: selected.field.clone(),
                to: typed,
            }
            .into(),
        );
    }
}

/// The two delete controls.
fn delete_row(
    ui: &mut Ui,
    field: &pdfcer_core::forms::Field,
    selected: &crate::app::state::SelectedField,
    refused: bool,
    actions: &mut Vec<Action>,
) {
    if refused {
        // A SENTENCE, never a silence — and it is published as a named
        // region so the withholding is *provable* from outside the process.
        //
        // `REGION_DELETE` above is declared only when the button is drawn and
        // this one only when it is not, so exactly one of the pair appears on
        // any frame this section runs. That is what makes the harness's
        // absence assertion admissible (`crate::checks`' rule 4): "no delete
        // button" and "the panel never opened" are otherwise the same trace,
        // and `TRACE_GATES` — written unconditionally a few lines up — is the
        // third fact that tells them apart.
        let sentence = ui.label(t::delete_refused());
        crate::diag::ui_rect(REGION_DELETE_REFUSED, sentence.rect);
        return;
    }
    ui.horizontal(|ui| {
        let remove = ui
            .button(t::delete_field())
            .on_hover_text(t::delete_field_hover(field.widgets.len()));
        crate::diag::ui_rect(REGION_DELETE, remove.rect);
        if remove.clicked() {
            actions.push(
                FieldAction::DeleteField {
                    field: selected.field.clone(),
                }
                .into(),
            );
        }
        if field.widgets.len() > 1
            && ui
                .button(t::delete_box())
                .on_hover_text(t::delete_box_hover())
                .clicked()
        {
            actions.push(
                FieldAction::DeleteWidget {
                    field: selected.field.clone(),
                    widget: selected.widget,
                }
                .into(),
            );
        }
    });
}

/// Whether the field's own dictionary carries `/Q`. `pdfcer_core::forms::Field`
/// exposes only the resolved quadding; an unreadable dictionary counts as
/// stating none.
fn states_own_quadding(doc: &OpenDoc, field: &pdfcer_core::forms::Field) -> bool {
    matches!(doc.session.value(field.id), Some(Object::Dict(d)) if d.get(b"Q").is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{SelectedField, open_local_fixture};

    /// The `/Sig` field both fixtures carry, merged with its widget on page 1.
    fn certifier() -> SelectedField {
        SelectedField {
            field: "Certifier".to_owned(),
            widget: 0,
            page: 0,
        }
    }

    /// **A certified document refuses to delete its form fields, and the
    /// derivation says so.**
    #[test]
    fn a_certified_document_refuses_to_delete_a_selected_field() {
        let mut doc = open_local_fixture("certified-comments.pdf");
        doc.selected_field = Some(certifier());
        assert!(
            refuses_delete(&doc),
            "the delete gate is open on a document carrying an enforced \
             certification — either `deletion_refusal` is not being asked or its \
             answer is being dropped, which is exactly the state that left four \
             live Delete controls on a form nothing could change"
        );
    }

    /// **The uncertified twin permits it**, which is what makes the test
    /// above evidence rather than a tautology.
    #[test]
    fn an_uncertified_document_permits_deleting_a_selected_field() {
        let mut doc = open_local_fixture("threaded-comments.pdf");
        doc.selected_field = Some(certifier());
        assert!(
            !refuses_delete(&doc),
            "the gate refused on the uncertified twin, which differs from the \
             certified fixture only in the catalog's /Perms entry — so this build \
             withholds Delete from every signed document"
        );
    }

    /// **`false` when nothing is selected, on the document that refuses.**
    #[test]
    fn no_field_selected_is_not_a_refusal() {
        let doc = open_local_fixture("certified-comments.pdf");
        assert!(doc.selected_field.is_none());
        assert!(
            !refuses_delete(&doc),
            "an empty field selection is not a refusal; it is nothing to refuse"
        );
    }
}
