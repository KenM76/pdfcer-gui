//! # `panels::dimension_groups::like` — a new group set up like an existing one
//!
//! Two routes to `DimensionAction::AddGroupLike`: the group row's right-click
//! menu, and a button beside *Add group* naming the selected group. Both use
//! the name typed in the new-group field, or the first unused numbered copy of
//! the template's name.

use egui::Ui;
use pdfcer_core::dimension::{DimensionModel, Group};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::text::dimension_groups as t;

/// The row menu's command.
pub const REGION_MENU: &str = "dimension-groups.row-menu.like"; // ui-text-exempt: trace region name, never displayed
/// The button beside *Add group*.
pub const REGION_BUTTON: &str = "dimension-groups.add-like"; // ui-text-exempt: trace region name, never displayed

/// The new group's name: the typed one, else `<name> 2`, `<name> 3`, ...
/// whichever no group already has.
fn name_for(model: &DimensionModel, template: &Group, typed: &str) -> String {
    let typed = typed.trim();
    if !typed.is_empty() {
        return typed.to_owned();
    }
    (2..)
        .map(|n| t::like_name(&template.name, n))
        .find(|candidate| model.groups().iter().all(|g| &g.name != candidate))
        .unwrap_or_else(|| template.name.clone())
}

/// Queue the action, clearing the typed name it consumed.
fn raise(model: &DimensionModel, template: &Group, typed: &mut String, actions: &mut Vec<Action>) {
    let name = name_for(model, template, typed);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "dimension-group-add-like template={} chars={}",
            template.id.0,
            name.len()
        )
    });
    actions.push(Action::Dimension(DimensionAction::AddGroupLike {
        name,
        template: template.id,
    }));
    typed.clear();
}

/// The row's right-click menu.
pub(super) fn row_menu(
    row: &egui::Response,
    model: &DimensionModel,
    group: &Group,
    typed: &mut String,
    actions: &mut Vec<Action>,
) {
    row.context_menu(|ui| {
        let like = ui.button(t::like_menu()).on_hover_text(t::like_hover());
        crate::diag::ui_rect(REGION_MENU, like.rect);
        if like.clicked() {
            raise(model, group, typed, actions);
            ui.close();
        }
    });
}

/// The button beside *Add group*, for the group the panel is showing.
pub(super) fn button(
    ui: &mut Ui,
    model: &DimensionModel,
    selected: Option<&Group>,
    typed: &mut String,
    actions: &mut Vec<Action>,
) {
    let Some(group) = selected else {
        return;
    };
    let response = ui
        .button(t::like_button(&group.name))
        .on_hover_text(t::like_hover());
    crate::diag::ui_rect(REGION_BUTTON, response.rect);
    if response.clicked() {
        raise(model, group, typed, actions);
    }
}
