//! The viewer's part list: the engine's model tree (`PrcFile::model_tree`),
//! one row per product occurrence, indented by depth, with the visibility
//! the file stores. Read-only: an assembled mesh does not say which node drew
//! it (G134), so a row cannot hide or highlight its part.

use egui::Ui;
use pdfcer_3d::{ModelNode, NameSource};

use crate::text::panels::models as t;

/// The part list's published region, for `ui-verify`.
pub const REGION_PARTS: &str = "model3d.parts"; // ui-text-exempt: trace region name, never displayed

/// Points each tree level is indented.
const INDENT: f32 = 12.0;

/// The `model-view-parts` trace fields for `nodes`: counts, the depths in
/// list order, then the names, each `|`-joined, an unnamed node as `-`.
pub(super) fn trace_fields(nodes: &[ModelNode]) -> String {
    let hidden = nodes.iter().filter(|n| n.hidden).count();
    let suppressed = nodes.iter().filter(|n| n.suppressed).count();
    let borrowed = nodes
        .iter()
        .filter(|n| matches!(n.name_from, NameSource::Prototype | NameSource::Part))
        .count();
    let names: Vec<&str> = nodes
        .iter()
        .map(|n| n.name.as_deref().unwrap_or("-"))
        .collect();
    let depths: Vec<String> = nodes.iter().map(|n| n.depth.to_string()).collect();
    format!(
        // ui-text-exempt: diagnostic trace fields, never displayed
        "nodes={} hidden={hidden} suppressed={suppressed} borrowed={borrowed} depths={} names={}",
        nodes.len(),
        depths.join("|"),
        names.join("|")
    )
}

/// Draw the list into `ui`, or `error` when the tree could not be read.
pub(super) fn show(ui: &mut Ui, nodes: &[ModelNode], error: Option<&str>) {
    ui.strong(t::parts_heading(nodes.len()));
    if let Some(why) = error {
        ui.small(t::parts_unreadable(why));
        return;
    }
    let scroll = egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for node in nodes {
                row(ui, node);
            }
        });
    crate::diag::ui_rect_visible(REGION_PARTS, scroll.inner_rect, ui.clip_rect());
}

/// One node: its name (or the unnamed word), what the file stores about its
/// visibility, and on hover where a borrowed name came from.
fn row(ui: &mut Ui, node: &ModelNode) {
    ui.horizontal(|ui| {
        ui.add_space(INDENT * node.depth as f32);
        let name = node
            .name
            .clone()
            .unwrap_or_else(|| t::part_unnamed().to_owned());
        let label = if node.drawn {
            ui.label(name)
        } else {
            ui.weak(name)
        };
        match node.name_from {
            NameSource::Prototype => {
                label.on_hover_text(t::part_named_from_prototype());
            }
            NameSource::Part => {
                label.on_hover_text(t::part_named_from_part());
            }
            _ => {}
        }
        if node.hidden {
            ui.weak(t::part_hidden());
        }
        if node.suppressed {
            ui.weak(t::part_suppressed());
        }
        if !node.drawn && !node.hidden && !node.suppressed {
            ui.weak(t::part_not_drawn());
        }
    });
}
