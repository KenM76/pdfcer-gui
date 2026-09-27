//! # `panels::layers::combine` — Merge a layer into another, and Flatten all
//!
//! Drawn only when the mode edits content, like `authoring`. Each window
//! raises one [`LayerAction`] and holds its draft in egui temp memory.

use pdfcer_core::edit::HiddenLayerPolicy;
use pdfcer_core::layers::{Layer, Layers};
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::layeraction::LayerAction;

use crate::app::actions::Action;
use crate::text::panels::layeredit as t;

// ui-text-exempt: trace region name, never displayed.
const REGION_MENU_MERGE: &str = "panel.layers.menu.merge";
// ui-text-exempt: trace region name, never displayed.
const REGION_MERGE_TARGET: &str = "panel.layers.merge.target";
// ui-text-exempt: trace region name prefix, never displayed.
const REGION_MERGE_OPTION: &str = "panel.layers.merge.option";
// ui-text-exempt: trace region name, never displayed.
const REGION_MERGE_GO: &str = "panel.layers.merge.go";
// ui-text-exempt: trace region name, never displayed.
const REGION_FLATTEN: &str = "panel.layers.flatten";
// ui-text-exempt: trace region name, never displayed.
const REGION_FLATTEN_GO: &str = "panel.layers.flatten.go";
// ui-text-exempt: trace region name, never displayed.
const REGION_FLATTEN_REMOVE: &str = "panel.layers.flatten.remove";
// ui-text-exempt: trace region name, never displayed.
const REGION_FLATTEN_SHOW: &str = "panel.layers.flatten.show";
// ui-text-exempt: a memory key, never displayed.
const MERGE_KEY: &str = "panel-layers-merge";
// ui-text-exempt: a memory key, never displayed.
const FLATTEN_KEY: &str = "panel-layers-flatten";

/// The Merge entry in a row's right-click menu, offered only when there is
/// another layer to merge into.
pub(super) fn menu_item(ui: &mut egui::Ui, read: &Layers, l: &Layer, name: &str) {
    if read.layers.len() < 2 {
        return;
    }
    let merge = ui.button(t::menu_merge());
    crate::diag::ui_rect(REGION_MENU_MERGE, merge.rect);
    if merge.clicked() {
        let draft = MergeDraft {
            layer: l.id,
            name: name.to_owned(),
            target: None,
        };
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(MERGE_KEY), draft));
        ui.close();
    }
}

/// The Flatten button, below the New layer row.
pub(super) fn flatten_button(ui: &mut egui::Ui, read: &Layers) {
    if read.layers.is_empty() {
        return;
    }
    let button = ui
        .button(t::flatten_button())
        .on_hover_text(t::flatten_tooltip());
    crate::diag::ui_rect(REGION_FLATTEN, button.rect);
    if button.clicked() {
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(FLATTEN_KEY), true));
    }
}

#[derive(Clone)]
struct MergeDraft {
    layer: ObjId,
    name: String,
    target: Option<ObjId>,
}

/// Draw whichever of the two windows is open.
pub(super) fn windows(ctx: &egui::Context, read: &Layers, actions: &mut Vec<Action>) {
    merge_window(ctx, read, actions);
    flatten_window(ctx, read, actions);
}

/// A layer's display name; an unnamed group shows its object id.
fn shown(l: &Layer) -> String {
    if l.name.is_empty() {
        l.id.to_string()
    } else {
        l.name.clone()
    }
}

fn merge_window(ctx: &egui::Context, read: &Layers, actions: &mut Vec<Action>) {
    let key = egui::Id::new(MERGE_KEY);
    let Some(mut draft) = ctx.data(|d| d.get_temp::<MergeDraft>(key)) else {
        return;
    };
    let mut close = false;
    egui::Window::new(t::merge_title(&draft.name))
        .id(egui::Id::new("layer-merge-window"))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(t::merge_into());
                let picked = draft
                    .target
                    .and_then(|id| read.layers.iter().find(|l| l.id == id))
                    .map_or_else(|| t::merge_pick().to_owned(), shown);
                let combo = egui::ComboBox::from_id_salt("layer-merge-target")
                    .selected_text(picked)
                    .show_ui(ui, |ui| {
                        for l in read.layers.iter().filter(|l| l.id != draft.layer) {
                            let option =
                                ui.selectable_value(&mut draft.target, Some(l.id), shown(l));
                            super::authoring::publish_keyed(
                                REGION_MERGE_OPTION,
                                &shown(l),
                                option.rect,
                            );
                        }
                    });
                crate::diag::ui_rect(REGION_MERGE_TARGET, combo.response.rect);
            });
            ui.label(egui::RichText::new(t::merge_explained()).small().weak());
            ui.horizontal(|ui| {
                let go = ui.add_enabled(draft.target.is_some(), egui::Button::new(t::merge()));
                crate::diag::ui_rect(REGION_MERGE_GO, go.rect);
                if go.clicked()
                    && let Some(target) = draft.target
                {
                    actions.push(Action::Layer(LayerAction::Merge {
                        target,
                        merged: vec![draft.layer],
                    }));
                    close = true;
                }
                if ui.button(t::cancel()).clicked() {
                    close = true;
                }
            });
        });
    ctx.data_mut(|d| {
        if close {
            d.remove::<MergeDraft>(key);
        } else {
            d.insert_temp(key, draft);
        }
    });
}

fn flatten_window(ctx: &egui::Context, read: &Layers, actions: &mut Vec<Action>) {
    let key = egui::Id::new(FLATTEN_KEY);
    if ctx.data(|d| d.get_temp::<bool>(key)) != Some(true) {
        return;
    }
    let hidden = read.layers.iter().filter(|l| !l.visible_by_default).count();
    let mut chosen = None;
    let mut close = false;
    egui::Window::new(t::flatten_title())
        .id(egui::Id::new("layer-flatten-window"))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(t::flatten_question(read.layers.len(), hidden));
            ui.horizontal(|ui| {
                if hidden == 0 {
                    let go = ui.button(t::flatten_go());
                    crate::diag::ui_rect(REGION_FLATTEN_GO, go.rect);
                    if go.clicked() {
                        chosen = Some(HiddenLayerPolicy::Refuse);
                    }
                } else {
                    let remove = ui
                        .button(t::flatten_remove_hidden())
                        .on_hover_text(t::flatten_remove_hidden_tooltip());
                    crate::diag::ui_rect(REGION_FLATTEN_REMOVE, remove.rect);
                    if remove.clicked() {
                        chosen = Some(HiddenLayerPolicy::Remove);
                    }
                    let show = ui
                        .button(t::flatten_show_hidden())
                        .on_hover_text(t::flatten_show_hidden_tooltip());
                    crate::diag::ui_rect(REGION_FLATTEN_SHOW, show.rect);
                    if show.clicked() {
                        chosen = Some(HiddenLayerPolicy::Show);
                    }
                }
                if ui.button(t::cancel()).clicked() {
                    close = true;
                }
            });
        });
    if let Some(hidden) = chosen {
        actions.push(Action::Layer(LayerAction::Flatten { hidden }));
        close = true;
    }
    if close {
        ctx.data_mut(|d| d.remove::<bool>(key));
    }
}
