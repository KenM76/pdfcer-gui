//! # `panels::layers::authoring` — New layer, and each row's Properties and Delete
//!
//! Drawn only when the mode edits content. Every control raises one
//! [`LayerAction`]; nothing here writes the document. The Properties window
//! and the Delete dialog hold their drafts in egui temp memory, so closing the
//! panel drops them.
//!
//! Print, export and purpose start at the value the file holds. A value
//! pdfcer cannot name (`None` from `read_layers`) shows as "Leave as it is"
//! and is written back only if the operator picks something else.

use pdfcer_core::edit::{LayerContentPolicy, LayerEdit, LayerIntent, LayerOutputState};
use pdfcer_core::layers::{Layer, Layers};
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::layeraction::LayerAction;

use crate::app::actions::Action;
use crate::text::panels::layeredit as t;

// ui-text-exempt: trace region names and memory keys, never displayed.
const REGION_NEW_NAME: &str = "panel.layers.new.name";
// ui-text-exempt: trace region name, never displayed.
const REGION_NEW: &str = "panel.layers.new";
// ui-text-exempt: trace region name, never displayed.
const REGION_MENU_PROPS: &str = "panel.layers.menu.properties";
// ui-text-exempt: trace region name, never displayed.
const REGION_MENU_DELETE: &str = "panel.layers.menu.delete";
// ui-text-exempt: trace region name, never displayed.
const REGION_PROPS_NAME: &str = "panel.layers.props.name";
// ui-text-exempt: trace region name, never displayed.
const REGION_PROPS_APPLY: &str = "panel.layers.props.apply";
// ui-text-exempt: trace region name, never displayed.
const REGION_DELETE_KEEP: &str = "panel.layers.delete.keep";
// ui-text-exempt: trace region name, never displayed.
const REGION_DELETE_REMOVE: &str = "panel.layers.delete.remove";
// ui-text-exempt: a memory key, never displayed.
const NEW_NAME_KEY: &str = "panel-layers-new-name";
// ui-text-exempt: a memory key, never displayed.
const PROPS_KEY: &str = "panel-layers-props";
// ui-text-exempt: a memory key, never displayed.
const DELETE_KEY: &str = "panel-layers-delete";

/// Whether this mode may author layers.
pub(super) fn enabled(ctx: &egui::Context) -> bool {
    crate::canvas::tool::capabilities(ctx).edit_content
}

/// The name field and New layer button.
pub(super) fn new_layer_row(ui: &mut egui::Ui, read: &Layers, actions: &mut Vec<Action>) {
    let key = egui::Id::new(NEW_NAME_KEY);
    let mut name: String = ui.ctx().data(|d| d.get_temp(key)).unwrap_or_default();
    ui.horizontal(|ui| {
        let field = ui.add(
            // escape-disposition: not-content — a layer name, stored as a text string.
            egui::TextEdit::singleline(&mut name)
                .hint_text(t::new_name_hint())
                .desired_width(140.0),
        );
        crate::diag::ui_rect(REGION_NEW_NAME, field.rect);
        let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let button = ui
            .button(t::new_layer())
            .on_hover_text(t::new_layer_tooltip());
        crate::diag::ui_rect(REGION_NEW, button.rect);
        if button.clicked() || enter {
            let typed = name.trim();
            let name_to_use = if typed.is_empty() {
                unused_default(read)
            } else {
                typed.to_owned()
            };
            actions.push(Action::Layer(LayerAction::Add { name: name_to_use }));
            name.clear();
        }
    });
    ui.ctx().data_mut(|d| d.insert_temp(key, name));
}

/// The first default name no layer already has.
fn unused_default(read: &Layers) -> String {
    (1..)
        .map(t::default_name)
        .find(|n| !read.layers.iter().any(|l| &l.name == n))
        .unwrap_or_default()
}

/// Publish a row's name label as `panel.layers.row.<name>`, every character
/// outside `[A-Za-z0-9]` written `_`, so a driven check can right-click it.
pub(super) fn publish_row(name: &str, rect: egui::Rect) {
    // ui-text-exempt: trace region name prefix, never displayed.
    publish_keyed("panel.layers.row", name, rect);
}

/// Publish `rect` as `<prefix>.<name>`, every character of `name` outside
/// `[A-Za-z0-9]` written `_`.
pub(super) fn publish_keyed(prefix: &str, name: &str, rect: egui::Rect) {
    let key: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    crate::diag::ui_rect(&format!("{prefix}.{key}"), rect);
}

/// Attach the row's right-click menu to `response`, the row's name label.
pub(super) fn row_menu(response: &egui::Response, read: &Layers, l: &Layer, name: &str) {
    response.context_menu(|ui| {
        let props = ui.button(t::menu_properties());
        crate::diag::ui_rect(REGION_MENU_PROPS, props.rect);
        if props.clicked() {
            let draft = PropsDraft::of(l, name);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "layer-props-opened print={} export={} intent={}",
                    output_word(draft.print),
                    output_word(draft.export),
                    intent_word(draft.intent)
                )
            });
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::new(PROPS_KEY), draft));
            ui.close();
        }
        let delete = ui.button(t::menu_delete());
        crate::diag::ui_rect(REGION_MENU_DELETE, delete.rect);
        if delete.clicked() {
            ui.ctx().data_mut(|d| {
                d.insert_temp(egui::Id::new(DELETE_KEY), (l.id, name.to_owned()));
            });
            ui.close();
        }
        super::combine::menu_item(ui, read, l, name);
    });
}

/// A trace word for a read print or export state.
fn output_word(s: Option<LayerOutputState>) -> &'static str {
    // ui-text-exempt: diagnostic trace words, never displayed.
    match s {
        Some(LayerOutputState::WhenVisible) => "when_visible",
        Some(LayerOutputState::Always) => "always",
        Some(LayerOutputState::Never) => "never",
        _ => "unnamed",
    }
}

/// A trace word for a read intent.
fn intent_word(i: Option<LayerIntent>) -> &'static str {
    // ui-text-exempt: diagnostic trace words, never displayed.
    match i {
        Some(LayerIntent::View) => "view",
        Some(LayerIntent::Design) => "design",
        Some(LayerIntent::Both) => "both",
        _ => "unnamed",
    }
}

/// The Properties window's draft.
#[derive(Clone)]
struct PropsDraft {
    layer: ObjId,
    was_name: String,
    name: String,
    was_visible: bool,
    visible: bool,
    was_locked: bool,
    locked: bool,
    was_print: Option<LayerOutputState>,
    print: Option<LayerOutputState>,
    was_export: Option<LayerOutputState>,
    export: Option<LayerOutputState>,
    was_intent: Option<LayerIntent>,
    intent: Option<LayerIntent>,
}

impl PropsDraft {
    fn of(l: &Layer, name: &str) -> Self {
        Self {
            layer: l.id,
            was_name: name.to_owned(),
            name: name.to_owned(),
            was_visible: l.visible_by_default,
            visible: l.visible_by_default,
            was_locked: l.locked,
            locked: l.locked,
            was_print: l.print,
            print: l.print,
            was_export: l.export,
            export: l.export,
            was_intent: l.intent_kind,
            intent: l.intent_kind,
        }
    }

    /// Only what the operator changed.
    fn edit(&self) -> LayerEdit {
        let mut e = LayerEdit::new();
        if self.name.trim() != self.was_name {
            e = e.name(self.name.trim());
        }
        if self.visible != self.was_visible {
            e = e.visible_by_default(self.visible);
        }
        if self.locked != self.was_locked {
            e = e.locked(self.locked);
        }
        if let Some(p) = self.print.filter(|_| self.print != self.was_print) {
            e = e.print(p);
        }
        if let Some(x) = self.export.filter(|_| self.export != self.was_export) {
            e = e.export(x);
        }
        if let Some(i) = self.intent.filter(|_| self.intent != self.was_intent) {
            e = e.intent(i);
        }
        e
    }
}

/// Draw whichever of the two windows is open.
pub(super) fn windows(ctx: &egui::Context, actions: &mut Vec<Action>) {
    properties_window(ctx, actions);
    delete_window(ctx, actions);
}

fn properties_window(ctx: &egui::Context, actions: &mut Vec<Action>) {
    let key = egui::Id::new(PROPS_KEY);
    let Some(mut draft) = ctx.data(|d| d.get_temp::<PropsDraft>(key)) else {
        return;
    };
    let mut close = false;
    egui::Window::new(t::properties_title(&draft.was_name))
        .id(egui::Id::new("layer-properties-window"))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            egui::Grid::new("layer-properties-grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(t::field_name());
                    let field = ui.add(
                        // escape-disposition: not-content — a layer name, stored as a text string.
                        egui::TextEdit::singleline(&mut draft.name).desired_width(180.0),
                    );
                    crate::diag::ui_rect(REGION_PROPS_NAME, field.rect);
                    ui.end_row();
                    ui.label("");
                    ui.checkbox(&mut draft.visible, t::field_visible());
                    ui.end_row();
                    ui.label("");
                    ui.checkbox(&mut draft.locked, t::field_locked());
                    ui.end_row();
                    choice(
                        ui,
                        "layer-print",
                        t::field_print(),
                        &mut draft.print,
                        draft.was_print,
                        &t::OUTPUT_CHOICES,
                    );
                    choice(
                        ui,
                        "layer-export",
                        t::field_export(),
                        &mut draft.export,
                        draft.was_export,
                        &t::OUTPUT_CHOICES,
                    );
                    choice(
                        ui,
                        "layer-intent",
                        t::field_intent(),
                        &mut draft.intent,
                        draft.was_intent,
                        &t::INTENT_CHOICES,
                    );
                });
            ui.horizontal(|ui| {
                let apply = ui.button(t::apply());
                crate::diag::ui_rect(REGION_PROPS_APPLY, apply.rect);
                if apply.clicked() {
                    actions.push(Action::Layer(LayerAction::Edit {
                        layer: draft.layer,
                        edit: draft.edit(),
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
            d.remove::<PropsDraft>(key);
        } else {
            d.insert_temp(key, draft);
        }
    });
}

/// One labelled drop-down. "Leave as it is" is offered only when the file's
/// value is one pdfcer cannot name.
fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    salt: &str,
    label: &str,
    value: &mut Option<T>,
    was: Option<T>,
    choices: &[(T, &str)],
) {
    ui.label(label);
    let shown = value
        .and_then(|v| choices.iter().find(|(c, _)| *c == v).map(|(_, w)| *w))
        .unwrap_or(t::unchanged());
    let combo = egui::ComboBox::from_id_salt(salt)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            if was.is_none() {
                ui.selectable_value(value, None, t::unchanged());
            }
            for (c, words) in choices {
                ui.selectable_value(value, Some(*c), *words);
            }
        })
        .response;
    if was.is_none() {
        combo.on_hover_text(t::unread_settings_tooltip());
    }
    ui.end_row();
}

fn delete_window(ctx: &egui::Context, actions: &mut Vec<Action>) {
    let key = egui::Id::new(DELETE_KEY);
    let Some((layer, name)) = ctx.data(|d| d.get_temp::<(ObjId, String)>(key)) else {
        return;
    };
    let mut close = false;
    egui::Window::new(t::menu_delete())
        .id(egui::Id::new("layer-delete-window"))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(t::delete_question(&name));
            ui.horizontal(|ui| {
                let keep = ui
                    .button(t::delete_keep())
                    .on_hover_text(t::delete_keep_tooltip());
                crate::diag::ui_rect(REGION_DELETE_KEEP, keep.rect);
                let remove = ui
                    .button(t::delete_remove())
                    .on_hover_text(t::delete_remove_tooltip());
                crate::diag::ui_rect(REGION_DELETE_REMOVE, remove.rect);
                let policy = if keep.clicked() {
                    Some(LayerContentPolicy::KeepUnlayered)
                } else if remove.clicked() {
                    Some(LayerContentPolicy::RemoveContent)
                } else {
                    None
                };
                if let Some(policy) = policy {
                    actions.push(Action::Layer(LayerAction::Delete { layer, policy }));
                    close = true;
                }
                if ui.button(t::cancel()).clicked() {
                    close = true;
                }
            });
        });
    if close {
        ctx.data_mut(|d| d.remove::<(ObjId, String)>(key));
    }
}
