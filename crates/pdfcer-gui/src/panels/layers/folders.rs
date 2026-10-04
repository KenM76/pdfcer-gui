//! # `panels::layers::folders` — New folder, and the Rename folder window
//!
//! Contract: New folder adds a folder at the end of the top level, named by
//! the box or, when the box is empty, by the first "New folder N" no folder
//! has. Rename holds its draft (the folder's path and the text) in egui temp
//! memory, so closing the panel drops it. Both raise one
//! [`OrderAction`]; nothing here writes the document.

use pdfcer_core::layers::Layers;
use pdfcer_gui_base::layeraction::{LayerAction, OrderAction};
use pdfcer_gui_base::layerorder as order;

use crate::app::actions::Action;
use crate::text::panels::layeredit as t;

// ui-text-exempt: trace region names and memory keys, never displayed.
const REGION_NEW_NAME: &str = "panel.layers.new_folder.name";
// ui-text-exempt: trace region name, never displayed.
const REGION_NEW: &str = "panel.layers.new_folder";
// ui-text-exempt: trace region name, never displayed.
const REGION_RENAME_NAME: &str = "panel.layers.rename.name";
// ui-text-exempt: trace region name, never displayed.
const REGION_RENAME_APPLY: &str = "panel.layers.rename.apply";
// ui-text-exempt: a memory key, never displayed.
const NEW_NAME_KEY: &str = "panel-layers-new-folder-name";
// ui-text-exempt: a memory key, never displayed.
const RENAME_KEY: &str = "panel-layers-rename-folder";

/// The Rename window's draft: the folder's path, its label, the new text.
#[derive(Clone)]
struct RenameDraft {
    at: Vec<usize>,
    was: String,
    label: String,
}

/// The name field and New folder button.
pub(super) fn new_folder_row(ui: &mut egui::Ui, read: &Layers, actions: &mut Vec<Action>) {
    let words = super::authoring::NameRow {
        key: NEW_NAME_KEY,
        region_name: REGION_NEW_NAME,
        region_button: REGION_NEW,
        hint: t::new_folder_hint(),
        button: t::new_folder(),
        tooltip: t::new_folder_tooltip(),
    };
    let Some(typed) = super::authoring::name_row(ui, &words) else {
        return;
    };
    let tree = order::tree(&read.order);
    let label = if typed.is_empty() {
        unused_folder_name(&tree)
    } else {
        typed
    };
    actions.push(Action::Layer(LayerAction::Order(OrderAction::AddFolder {
        parent: Vec::new(),
        index: tree.len(),
        label,
    })));
}

/// The first default folder name no folder already has.
fn unused_folder_name(tree: &[order::Node]) -> String {
    let taken: Vec<String> = order::folders(tree).into_iter().map(|(_, l)| l).collect();
    (1..)
        .map(t::default_folder_name)
        .find(|n| !taken.contains(n))
        .unwrap_or_default()
}

/// Open the Rename window for the folder at `at`.
pub(super) fn open_rename(ctx: &egui::Context, at: &[usize], label: &str) {
    let draft = RenameDraft {
        at: at.to_vec(),
        was: label.to_owned(),
        label: label.to_owned(),
    };
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(RENAME_KEY), draft));
}

/// Draw the Rename window when it is open.
pub(super) fn rename_window(ctx: &egui::Context, actions: &mut Vec<Action>) {
    let key = egui::Id::new(RENAME_KEY);
    let Some(mut draft) = ctx.data(|d| d.get_temp::<RenameDraft>(key)) else {
        return;
    };
    let mut close = false;
    egui::Window::new(t::rename_folder_title(&draft.was))
        .id(egui::Id::new("layer-rename-folder-window"))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            let field = ui.add(
                // escape-disposition: not-content — a folder label, stored as a text string.
                egui::TextEdit::singleline(&mut draft.label).desired_width(200.0),
            );
            crate::diag::ui_rect(REGION_RENAME_NAME, field.rect);
            ui.horizontal(|ui| {
                let apply = ui.button(t::apply());
                crate::diag::ui_rect(REGION_RENAME_APPLY, apply.rect);
                if apply.clicked() {
                    actions.push(Action::Layer(LayerAction::Order(
                        OrderAction::RenameFolder {
                            at: draft.at.clone(),
                            label: draft.label.trim().to_owned(),
                        },
                    )));
                    close = true;
                }
                if ui.button(t::cancel()).clicked() {
                    close = true;
                }
            });
        });
    ctx.data_mut(|d| {
        if close {
            d.remove::<RenameDraft>(key);
        } else {
            d.insert_temp(key, draft);
        }
    });
}
