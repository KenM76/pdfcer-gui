//! # `panels::properties::radiogroup` — which group a radio button is in
//!
//! Drawn for a radio button only, under the rename row: the group's name,
//! a list of the document's other radio groups, and **Move to group**, which
//! moves this one button (not its whole group) into the named group, or into
//! a new group when no field has that name. Rename's own row routes a radio
//! renamed to an existing group's name here too, which is how Acrobat joins
//! radio buttons; the engine's `rename_field` would refuse that name.

use egui::Ui;
use pdfcer_core::forms::{AcroForm, ButtonKind, Field, FieldType};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::app::state::SelectedField;
use crate::panels::PanelsState;
use crate::text::radiogroup as t;

/// The group-name box's rect, published when the row is drawn.
pub const REGION_NAME: &str = "properties.form_field.radio_group.name";
/// The **Move to group** button's rect, published when the row is drawn.
pub const REGION_MOVE: &str = "properties.form_field.radio_group.move";

/// Whether `field` is a radio-button field.
#[must_use]
pub fn is_radio(field: &Field) -> bool {
    field.field_type == Some(FieldType::Button) && field.button_kind == Some(ButtonKind::Radio)
}

/// The document's radio groups, by fully-qualified name, in `/Fields` order.
#[must_use]
pub fn groups(form: &AcroForm) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for f in form.fields.iter().filter(|f| is_radio(f)) {
        if !names.contains(&f.fully_qualified_name) {
            names.push(f.fully_qualified_name.clone());
        }
    }
    names
}

/// The action a Rename of the radio `selected` to `typed` should become, when
/// `typed` names another radio group: a move of this button into it.
#[must_use]
pub fn rename_joins(form: &AcroForm, selected: &SelectedField, typed: &str) -> Option<FieldAction> {
    let typed = typed.trim();
    let own = form.field_by_name(&selected.field)?;
    (is_radio(own) && typed != selected.field && groups(form).iter().any(|g| g == typed))
        .then(|| join(selected, typed))
}

fn join(selected: &SelectedField, to: &str) -> FieldAction {
    FieldAction::JoinRadioGroup {
        field: selected.field.clone(),
        widget: selected.widget,
        page: selected.page,
        to: to.to_owned(),
    }
}

/// Draw the row, for a radio button; nothing for any other field.
pub fn row(
    ui: &mut Ui,
    form: &AcroForm,
    field: &Field,
    selected: &SelectedField,
    state: &mut PanelsState,
    actions: &mut Vec<Action>,
) {
    if !is_radio(field) {
        return;
    }
    let others: Vec<String> = groups(form)
        .into_iter()
        .filter(|g| *g != selected.field)
        .collect();
    let key = format!("{}#{}", selected.field, selected.widget);
    let draft = state.radio_group_mut(&key, &selected.field);
    ui.label(t::LABEL);
    let mut picked = None;
    let box_response = ui
        .horizontal(|ui| {
            let response = ui
                .add(
                    // escape-disposition: keeps-draft — the panel's own group
                    // draft, re-seeded only when the selection changes.
                    egui::TextEdit::singleline(draft)
                        .desired_width(ui.available_width() - 28.0)
                        .char_limit(crate::canvas::formfield::draft::NAME_MAX),
                )
                .on_hover_text(t::NAME_HOVER);
            if !others.is_empty() {
                egui::ComboBox::from_id_salt("properties.radio_group.pick")
                    .selected_text("")
                    .width(20.0)
                    .show_ui(ui, |ui| {
                        for g in &others {
                            if ui.selectable_label(false, g).clicked() {
                                picked = Some(g.clone());
                            }
                        }
                    });
            }
            response
        })
        .inner;
    crate::diag::ui_rect(REGION_NAME, box_response.rect);
    if let Some(g) = picked {
        *draft = g;
    }
    let typed = draft.trim().to_owned();
    let blocked = blocked(form, selected, &typed);
    let button = ui.add_enabled(blocked.is_none(), egui::Button::new(t::MOVE));
    crate::diag::ui_rect(REGION_MOVE, button.rect);
    let pressed = button.clicked();
    match &blocked {
        Some(why) => drop(button.on_disabled_hover_text(why.as_str())),
        None => drop(button.on_hover_text(t::MOVE_HOVER)),
    }
    let entered = box_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if blocked.is_none() && (pressed || entered) {
        actions.push(join(selected, &typed).into());
    }
}

/// Why the move cannot be made with `typed`, or `None` when it can.
fn blocked(form: &AcroForm, selected: &SelectedField, typed: &str) -> Option<String> {
    if typed.is_empty() {
        return Some(t::EMPTY_NAME.to_owned());
    }
    if typed == selected.field {
        return Some(t::SAME_GROUP.to_owned());
    }
    match form.field_by_name(typed) {
        Some(f) if !is_radio(f) => Some(t::not_a_group(typed)),
        _ => None,
    }
}
