//! The viewer's part list and which parts it draws: the engine's model tree,
//! one row per product occurrence indented by depth, each with a box that
//! shows or hides its part and a name that highlights it.
//!
//! The viewer assembles every part, the stored-hidden ones too
//! (`StoredVisibility::DrawAll`), so it opens on the file's stored state and
//! the operator can show a part the file hides. A node's `placements` range
//! names the placements it draws; `AssembledModel::mesh_placements` names each
//! mesh's placement, so a hidden node's meshes are painted fully transparent,
//! which the renderer skips, and the highlighted node's in the selection
//! colour. Painting is lent to one render and restored after it. Pictures for
//! the page and for a file carry the hiding but never the highlight: the
//! highlight is the cursor, not content. Nothing is written into the file.

use std::collections::BTreeSet;

use egui::Ui;
use pdfcer_3d::{AssembledModel, ModelNode, NameSource};

use crate::text::model3d as m;
use crate::text::panels::models as t;

/// The part list's published region, for `ui-verify`.
pub const REGION_PARTS: &str = "model3d.parts"; // ui-text-exempt: trace region name, never displayed
/// One row's show box; the suffix is the node's index in the list.
pub const REGION_SHOW_PREFIX: &str = "model3d.part.show."; // ui-text-exempt: trace region name, never displayed
/// One row's name, which highlights the part; the suffix is the node's index.
pub const REGION_NAME_PREFIX: &str = "model3d.part.name."; // ui-text-exempt: trace region name, never displayed

/// Points each tree level is indented.
const INDENT: f32 = 12.0;

/// Which nodes are hidden and which one is highlighted, by index in the
/// list. `version` changes with either, so the picture is redrawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Visibility {
    hidden: BTreeSet<usize>,
    highlight: Option<usize>,
    pub version: u32,
}

impl Visibility {
    /// The file's stored state: every node it does not draw is hidden.
    pub fn stored(nodes: &[ModelNode]) -> Self {
        Self {
            hidden: nodes
                .iter()
                .enumerate()
                .filter(|(_, n)| !n.drawn)
                .map(|(i, _)| i)
                .collect(),
            highlight: None,
            version: 0,
        }
    }

    fn set(&mut self, hidden: BTreeSet<usize>, highlight: Option<usize>, why: &str) {
        if hidden == self.hidden && highlight == self.highlight {
            return;
        }
        self.hidden = hidden;
        self.highlight = highlight;
        self.version = self.version.wrapping_add(1);
        crate::diag::trace(|| {
            let hidden: Vec<String> = self.hidden.iter().map(ToString::to_string).collect();
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-parts-state why={why} hidden={} highlight={} version={}",
                if hidden.is_empty() {
                    "-".to_owned()
                } else {
                    hidden.join("|")
                },
                self.highlight
                    .map_or_else(|| "-".to_owned(), |i| i.to_string()),
                self.version
            )
        });
    }

    /// Whether node `i` is drawn: neither it nor an ancestor is hidden.
    fn shows(&self, nodes: &[ModelNode], mut i: usize) -> bool {
        loop {
            if self.hidden.contains(&i) {
                return false;
            }
            match nodes.get(i).and_then(|n| n.parent) {
                Some(parent) => i = parent,
                None => return true,
            }
        }
    }

    /// Whether placement `p` is drawn: no node drawing it is hidden.
    fn draws(&self, nodes: &[ModelNode], p: usize) -> bool {
        self.hidden
            .iter()
            .all(|&i| nodes.get(i).is_none_or(|n| !n.placements.contains(&p)))
    }

    /// Draw with `model` painted for this state, the highlight in `accent`
    /// when given, and restore the model's own paint afterwards.
    pub fn painted<R>(
        &self,
        model: &mut AssembledModel,
        accent: Option<[u8; 4]>,
        draw: impl FnOnce(&AssembledModel) -> R,
    ) -> R {
        let lit = accent.and_then(|a| Some((self.highlight?, a)));
        if model.mesh_placements.is_empty() || (self.hidden.is_empty() && lit.is_none()) {
            return draw(model);
        }
        let nodes = std::mem::take(&mut model.tree);
        let colours = model.colours.clone();
        let textures = model.mesh_textures.clone();
        model.colours.resize(model.meshes.len(), None);
        model.mesh_textures.resize(model.meshes.len(), None);
        for (mesh, &p) in model.mesh_placements.iter().enumerate() {
            let paint = if !self.draws(&nodes, p) {
                Some([0, 0, 0, 0])
            } else {
                lit.filter(|&(n, _)| nodes.get(n).is_some_and(|n| n.placements.contains(&p)))
                    .map(|(_, a)| a)
            };
            if let Some(paint) = paint {
                model.colours[mesh] = Some(paint);
                model.mesh_textures[mesh] = None;
            }
        }
        model.tree = nodes;
        let drawn = draw(model);
        model.colours = colours;
        model.mesh_textures = textures;
        drawn
    }
}

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
/// `steerable` is false when the meshes do not say which placement drew
/// them, and the rows then only list.
pub(super) fn show(
    ui: &mut Ui,
    nodes: &[ModelNode],
    error: Option<&str>,
    visibility: &mut Visibility,
    steerable: bool,
) {
    ui.label(t::parts_heading(nodes.len()));
    if let Some(why) = error {
        ui.small(t::parts_unreadable(why));
        return;
    }
    if steerable && !nodes.is_empty() {
        ui.small(m::parts_note());
    }
    let scroll = egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, node) in nodes.iter().enumerate() {
                row(ui, nodes, i, node, visibility, steerable);
            }
        });
    crate::diag::ui_rect_visible(REGION_PARTS, scroll.inner_rect, ui.clip_rect());
}

/// One node: its show box, its name (or the unnamed word), what the file
/// stores about its visibility, its markups and views, and on hover where a
/// borrowed name came from.
fn row(
    ui: &mut Ui,
    nodes: &[ModelNode],
    i: usize,
    node: &ModelNode,
    visibility: &mut Visibility,
    steerable: bool,
) {
    ui.horizontal(|ui| {
        ui.add_space(INDENT * node.depth as f32);
        if steerable {
            show_box(ui, i, visibility);
        }
        let name = node
            .name
            .clone()
            .unwrap_or_else(|| t::part_unnamed().to_owned());
        let label = if !steerable {
            if node.drawn {
                ui.label(name)
            } else {
                ui.weak(name)
            }
        } else {
            let shown = visibility.shows(nodes, i);
            let text = if shown {
                egui::RichText::new(name)
            } else {
                egui::RichText::new(name).weak()
            };
            let lit = visibility.highlight == Some(i);
            let r = ui
                .selectable_label(lit, text)
                .on_hover_text(m::part_highlight_tooltip());
            // ui-text-exempt: trace region name, never displayed
            let region = format!("{REGION_NAME_PREFIX}{i}");
            crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
            if r.clicked() {
                let to = if lit { None } else { Some(i) };
                visibility.set(visibility.hidden.clone(), to, "highlight"); // ui-text-exempt: trace token
            }
            r.context_menu(|ui| menu(ui, nodes, i, visibility));
            r
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
        if node.markups > 0 || node.views > 0 {
            ui.weak(m::part_extras(node.markups, node.views));
        }
    });
}

fn show_box(ui: &mut Ui, i: usize, visibility: &mut Visibility) {
    let mut on = !visibility.hidden.contains(&i);
    let r = ui
        .checkbox(&mut on, "")
        .on_hover_text(m::part_show_tooltip());
    // ui-text-exempt: trace region name, never displayed
    let region = format!("{REGION_SHOW_PREFIX}{i}");
    crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
    if r.changed() {
        let mut hidden = visibility.hidden.clone();
        if on {
            hidden.remove(&i);
        } else {
            hidden.insert(i);
        }
        visibility.set(hidden, visibility.highlight, "box"); // ui-text-exempt: trace token
    }
}

/// A row's menu: show only this part, show every part, or back to the file's
/// stored state.
fn menu(ui: &mut Ui, nodes: &[ModelNode], i: usize, visibility: &mut Visibility) {
    if ui.button(m::part_isolate()).clicked() {
        let keep = |j: usize| {
            j == i || ancestors(nodes, i).any(|a| a == j) || ancestors(nodes, j).any(|a| a == i)
        };
        let hidden = (0..nodes.len()).filter(|&j| !keep(j)).collect();
        visibility.set(hidden, visibility.highlight, "isolate"); // ui-text-exempt: trace token
        ui.close();
    }
    if ui.button(m::parts_show_all()).clicked() {
        visibility.set(BTreeSet::new(), visibility.highlight, "all"); // ui-text-exempt: trace token
        ui.close();
    }
    if ui.button(m::parts_as_stored()).clicked() {
        let stored = Visibility::stored(nodes).hidden;
        visibility.set(stored, visibility.highlight, "stored"); // ui-text-exempt: trace token
        ui.close();
    }
}

/// The indices of `i`'s ancestors, nearest first.
fn ancestors(nodes: &[ModelNode], i: usize) -> impl Iterator<Item = usize> + '_ {
    std::iter::successors(nodes.get(i).and_then(|n| n.parent), |&p| {
        nodes.get(p).and_then(|n| n.parent)
    })
}
