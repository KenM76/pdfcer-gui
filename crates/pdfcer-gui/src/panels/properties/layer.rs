//! # `panels::properties::layer` — the selection's layer, and the Move to layer window
//!
//! Contract: in a mode that edits content, with at least one layer in the
//! document, the Properties panel shows a Layer combo for a movable
//! selection (whole objects on one page, an annotation, or a form field) and
//! the layer it is on; choosing another raises `LayerAction::Assign`. A
//! selection of parts or across pages shows the combo greyed with the reason.
//! `format.move_to_layer` opens [`window`], which raises the same act.

use pdfcer_core::layers::Layer;
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::layeraction::LayerAction;

use crate::app::actions::Action;
use crate::app::actions::layerassign::{self, Operand};
use crate::app::state::OpenDoc;
use crate::panels::layers::authoring::publish_keyed;
use crate::panels::layers::highlight::{self, Membership};
use crate::text::panels::layerassign as t;
use crate::text::panels::layeredit::LayerRefusal;

// ui-text-exempt: trace region name, never displayed.
const REGION_COMBO: &str = "properties.layer.combo";
// ui-text-exempt: trace region name prefix, never displayed.
const REGION_OPTION: &str = "properties.layer.option";
// ui-text-exempt: trace region name, never displayed.
const REGION_WINDOW_COMBO: &str = "layer-assign.window.combo";
// ui-text-exempt: trace region name prefix, never displayed.
const REGION_WINDOW_OPTION: &str = "layer-assign.window.option";
// ui-text-exempt: trace region name, never displayed.
const REGION_WINDOW_GO: &str = "layer-assign.window.go";
// ui-text-exempt: a memory key, never displayed.
const WINDOW_KEY: &str = "layer-assign-window";

/// A layer's display name; an unnamed group shows its object id.
fn shown(l: &Layer) -> String {
    if l.name.is_empty() {
        l.id.to_string()
    } else {
        l.name.clone()
    }
}

/// The document's layers, or `None` when it has none.
fn layers(doc: &OpenDoc) -> Option<Vec<Layer>> {
    let read = pdfcer_core::layers::read_layers(&doc.session.view()).layers;
    (!read.is_empty()).then_some(read)
}

/// What the operand is on: `Some(Some(id))` for one layer, `Some(None)` for
/// none, `None` when it is several or cannot be read.
fn current(doc: &OpenDoc, operand: Option<&Operand>) -> (Option<Option<ObjId>>, Membership) {
    let membership = match operand {
        Some(Operand::Annotation { page, id }) => highlight::on_annotation(doc, *page, *id),
        _ => highlight::resolve(doc),
    };
    let value = match membership {
        Membership::Group(id) => Some(Some(id)),
        Membership::None => Some(None),
        _ => None,
    };
    (value, membership)
}

/// The combo's text for `value`.
fn label(read: &[Layer], value: Option<Option<ObjId>>, membership: Membership) -> String {
    match value {
        Some(None) => t::no_layer().to_owned(),
        Some(Some(id)) => read
            .iter()
            .find(|l| l.id == id)
            .map_or_else(|| t::other().to_owned(), shown),
        None if membership == Membership::Mixed => t::mixed().to_owned(),
        None => t::other().to_owned(),
    }
}

/// The choices: No layer, then every layer. `value` is what is picked.
fn choices(ui: &mut egui::Ui, read: &[Layer], value: &mut Option<Option<ObjId>>, region: &str) {
    let none = ui.selectable_value(value, Some(None), t::no_layer());
    publish_keyed(region, t::no_layer(), none.rect);
    for l in read {
        let option = ui.selectable_value(value, Some(Some(l.id)), shown(l));
        publish_keyed(region, &shown(l), option.rect);
    }
}

/// Draw the Layer row. `true` if anything was drawn.
pub fn section(ui: &mut egui::Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    if !crate::canvas::tool::capabilities(ui.ctx()).edit_content {
        return false;
    }
    let Some(read) = layers(doc) else {
        return false;
    };
    let operand = match layerassign::operand(doc) {
        Ok(operand) => Ok(operand),
        Err(LayerRefusal::AssignParts) => Err(t::parts_tooltip()),
        Err(LayerRefusal::AssignSeveralPages) => Err(t::pages_tooltip()),
        Err(_) => return false,
    };
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(t::heading());
        let (was, membership) = current(doc, operand.as_ref().ok());
        match &operand {
            Ok(_) => {
                let mut value = was;
                let combo = egui::ComboBox::from_id_salt("properties-layer")
                    .selected_text(label(&read, was, membership))
                    .show_ui(ui, |ui| choices(ui, &read, &mut value, REGION_OPTION));
                crate::diag::ui_rect(REGION_COMBO, combo.response.rect);
                combo.response.on_hover_text(t::tooltip());
                if value != was
                    && let Some(layer) = value
                {
                    actions.push(Action::Layer(LayerAction::Assign { layer }));
                }
            }
            Err(why) => {
                let shown = label(&read, was, membership);
                ui.add_enabled(false, egui::Button::new(shown))
                    .on_disabled_hover_text(*why);
            }
        }
    });
    true
}

/// Open the Move to layer window (`format.move_to_layer`).
pub fn open_window(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp::<Option<Option<ObjId>>>(egui::Id::new(WINDOW_KEY), None));
}

/// Draw the Move to layer window when it is open.
pub fn window(ctx: &egui::Context, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let key = egui::Id::new(WINDOW_KEY);
    let Some(mut draft) = ctx.data(|d| d.get_temp::<Option<Option<ObjId>>>(key)) else {
        return;
    };
    let read = layers(doc).unwrap_or_default();
    let mut close = false;
    egui::Window::new(t::window_title())
        .id(egui::Id::new("layer-assign-window"))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(t::window_to());
                let text = draft.map_or_else(
                    || t::window_pick().to_owned(),
                    |v| label(&read, Some(v), Membership::NothingSelected),
                );
                let combo = egui::ComboBox::from_id_salt("layer-assign-target")
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        choices(ui, &read, &mut draft, REGION_WINDOW_OPTION)
                    });
                crate::diag::ui_rect(REGION_WINDOW_COMBO, combo.response.rect);
            });
            ui.horizontal(|ui| {
                let go = ui.add_enabled(draft.is_some(), egui::Button::new(t::window_go()));
                crate::diag::ui_rect(REGION_WINDOW_GO, go.rect);
                if go.clicked()
                    && let Some(layer) = draft
                {
                    actions.push(Action::Layer(LayerAction::Assign { layer }));
                    close = true;
                }
                if ui.button(t::window_cancel()).clicked() {
                    close = true;
                }
            });
        });
    ctx.data_mut(|d| {
        if close {
            d.remove::<Option<Option<ObjId>>>(key);
        } else {
            d.insert_temp(key, draft);
        }
    });
}
