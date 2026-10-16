//! The Properties panel's delete buttons with several form fields selected:
//! one deletes every selected field whole, the other only the selected boxes,
//! as the Delete key does. Each is one undo step.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::app::state::SelectedField;
use crate::text::panels::fielddelete as t;

/// Published while the whole-fields button is drawn.
pub const REGION_DELETE_FIELDS: &str = "properties.form_field.delete_fields";
/// Published while the selected-boxes button is drawn.
pub const REGION_DELETE_BOXES: &str = "properties.form_field.delete_boxes";

/// Draw the two buttons when `boxes` holds more than one, and say whether it
/// did; the one-field row is the caller's otherwise. The boxes button is
/// drawn only when some selected field has a box that is not selected, since
/// otherwise both delete the same.
pub fn row(
    ui: &mut Ui,
    form: &pdfcer_core::forms::AcroForm,
    boxes: &[SelectedField],
    actions: &mut Vec<Action>,
) -> bool {
    if boxes.len() < 2 {
        return false;
    }
    let mut fields: Vec<&str> = boxes.iter().map(|b| b.field.as_str()).collect();
    fields.sort_unstable();
    fields.dedup();
    let total: usize = form
        .fields
        .iter()
        .filter(|f| fields.contains(&f.fully_qualified_name.as_str()))
        .map(|f| f.widgets.len())
        .sum();
    let page = boxes.last().map_or(0, |b| b.page);
    ui.horizontal(|ui| {
        let whole = ui
            .button(t::delete_fields(fields.len()))
            .on_hover_text(t::delete_fields_hover());
        crate::diag::ui_rect(REGION_DELETE_FIELDS, whole.rect);
        if whole.clicked() {
            actions.push(
                FieldAction::DeleteFields {
                    fields: fields.iter().map(|f| (*f).to_owned()).collect(),
                    page,
                }
                .into(),
            );
        }
        if total > boxes.len() {
            let some = ui
                .button(t::delete_boxes(boxes.len()))
                .on_hover_text(t::delete_boxes_hover());
            crate::diag::ui_rect(REGION_DELETE_BOXES, some.rect);
            if some.clicked()
                && let Some(action) = FieldAction::delete_selected(boxes.to_vec())
            {
                actions.push(action.into());
            }
        }
    });
    true
}
