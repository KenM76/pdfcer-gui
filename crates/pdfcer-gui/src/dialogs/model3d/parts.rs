//! The viewer's part list and which parts it draws: the engine's model tree,
//! one row per product occurrence under guide lines that fold by branch (see
//! [`super::partstree`]), each with a box that shows or hides its part and a
//! name that selects it. [`super::partpick`] holds what a click selects and
//! the menu both the list and the picture open.
//!
//! The viewer assembles every part, the stored-hidden ones too
//! (`StoredVisibility::DrawAll`), so it opens on the file's stored state and
//! the operator can show a part the file hides. A node's `placements` range
//! names the placements it draws; `AssembledModel::mesh_placements` names each
//! mesh's placement, so a hidden node's meshes are painted fully transparent,
//! which the renderer skips, and the selected nodes' in the selection
//! colour. Painting is lent to one render and restored after it. Pictures for
//! the page and for a file carry the hiding but never the selection: the
//! selection is the cursor, not content. Nothing is written into the file.

use std::collections::BTreeSet;

use egui::Ui;
use pdfcer_3d::{AssembledModel, ModelNode, NameSource};

use super::partpick::Click;
use super::partstree::{self, Shape};
use crate::text::model3d as m;
use crate::text::panels::models as t;

/// The part list's published region, for `ui-verify`.
pub const REGION_PARTS: &str = "model3d.parts"; // ui-text-exempt: trace region name, never displayed
/// One row's show box; the suffix is the node's index in the list.
pub const REGION_SHOW_PREFIX: &str = "model3d.part.show."; // ui-text-exempt: trace region name, never displayed
/// One row's name, which selects the part; the suffix is the node's index.
pub const REGION_NAME_PREFIX: &str = "model3d.part.name."; // ui-text-exempt: trace region name, never displayed

/// Which nodes are hidden, which are selected and which are folded, by index
/// in the list. `version` changes with what the picture draws, so it is
/// redrawn; folding only changes the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Visibility {
    pub(super) hidden: BTreeSet<usize>,
    /// Painted in the selection colour; what the menu acts on.
    pub(super) selected: BTreeSet<usize>,
    pub(super) folded: BTreeSet<usize>,
    /// Draw only what the regular view hides.
    pub(super) hidden_only: bool,
    /// Where a Shift+click's run starts.
    pub(super) anchor: Option<usize>,
    /// A row to scroll into view on the next frame.
    pub(super) reveal: Option<usize>,
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
            selected: BTreeSet::new(),
            folded: BTreeSet::new(),
            hidden_only: false,
            anchor: None,
            reveal: None,
            version: 0,
        }
    }

    /// Fold these nodes and unfold the rest.
    pub(super) fn fold(&mut self, folded: BTreeSet<usize>, why: &str) {
        self.folded = folded;
        let shown: Vec<String> = self.folded.iter().map(ToString::to_string).collect();
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| {
            format!(
                "model-view-parts-fold why={why} folded={}",
                if shown.is_empty() {
                    "-".to_owned()
                } else {
                    shown.join("|")
                }
            )
        });
    }

    /// Hide and select these nodes, redrawing if either changed.
    pub(super) fn set(&mut self, hidden: BTreeSet<usize>, selected: BTreeSet<usize>, why: &str) {
        if hidden == self.hidden && selected == self.selected {
            return;
        }
        self.hidden = hidden;
        self.selected = selected;
        self.changed(why);
    }

    /// Hide these nodes and keep the selection.
    pub(super) fn hide(&mut self, hidden: BTreeSet<usize>, why: &str) {
        self.set(hidden, self.selected.clone(), why);
    }

    /// Select these nodes and keep what is hidden.
    pub(super) fn select(&mut self, selected: BTreeSet<usize>, why: &str) {
        self.set(self.hidden.clone(), selected, why);
    }

    /// Draw only the hidden parts, or go back to the shown ones.
    pub(super) fn view(&mut self, hidden_only: bool) {
        if hidden_only != self.hidden_only {
            self.hidden_only = hidden_only;
            self.changed("view"); // ui-text-exempt: trace token
        }
    }

    /// The picture is out of date: count it and say what it now draws.
    fn changed(&mut self, why: &str) {
        self.version = self.version.wrapping_add(1);
        crate::diag::trace(|| {
            let list = |set: &BTreeSet<usize>| {
                if set.is_empty() {
                    "-".to_owned()
                } else {
                    let all: Vec<String> = set.iter().map(ToString::to_string).collect();
                    all.join("|")
                }
            };
            format!(
                "model-view-parts-state why={why} hidden={} highlight={} version={} view={}",
                list(&self.hidden),
                list(&self.selected),
                self.version,
                if self.hidden_only { "hidden" } else { "shown" }
            )
        });
    }

    /// Whether node `i` is drawn: neither it nor an ancestor is hidden.
    pub(super) fn shows(&self, nodes: &[ModelNode], mut i: usize) -> bool {
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

    /// Which of `model`'s meshes are drawn, or `None` when every one is.
    pub fn shown_meshes(&self, model: &AssembledModel) -> Option<Vec<bool>> {
        if model.mesh_placements.is_empty() || (self.hidden.is_empty() && !self.hidden_only) {
            return None;
        }
        Some(
            model
                .mesh_placements
                .iter()
                .map(|&p| self.draws(&model.tree, p) != self.hidden_only)
                .collect(),
        )
    }

    /// Draw with `model` painted for this state, the highlight in `accent`
    /// when given, and restore the model's own paint afterwards.
    pub fn painted<R>(
        &self,
        model: &mut AssembledModel,
        accent: Option<[u8; 4]>,
        draw: impl FnOnce(&AssembledModel) -> R,
    ) -> R {
        let lit = accent.filter(|_| !self.selected.is_empty());
        if model.mesh_placements.is_empty()
            || (self.hidden.is_empty() && lit.is_none() && !self.hidden_only)
        {
            return draw(model);
        }
        let nodes = std::mem::take(&mut model.tree);
        let colours = model.colours.clone();
        let textures = model.mesh_textures.clone();
        model.colours.resize(model.meshes.len(), None);
        model.mesh_textures.resize(model.meshes.len(), None);
        for (mesh, &p) in model.mesh_placements.iter().enumerate() {
            let paint = if self.draws(&nodes, p) == self.hidden_only {
                Some([0, 0, 0, 0])
            } else {
                let selected = |&n: &usize| nodes.get(n).is_some_and(|n| n.placements.contains(&p));
                lit.filter(|_| self.selected.iter().any(selected))
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
        super::partpick::hidden_only_box(ui, visibility);
    }
    // Only the rows in sight are laid out: a model of a thousand parts
    // otherwise spends most of every frame on rows scrolled out of view.
    let height = ui.spacing().interact_size.y;
    let shape = Shape::of(nodes.iter().map(|n| n.parent).collect());
    let rows = shape.visible(&visibility.folded);
    let mut area = egui::ScrollArea::both().auto_shrink([false, false]);
    let wanted = visibility.reveal.take();
    if let Some(at) = wanted.and_then(|i| rows.iter().position(|&r| r == i)) {
        // Centred, so the rows around the one asked for are in view too.
        let pitch = height + ui.spacing().item_spacing.y;
        let top = at as f32 * pitch - (ui.available_height() - pitch) / 2.0;
        area = area.vertical_scroll_offset(top.max(0.0));
    }
    let scroll = area.show_rows(ui, height, rows.len(), |ui, range| {
        for &i in rows.get(range).unwrap_or_default() {
            if let Some(node) = nodes.get(i) {
                let tree = Tree {
                    nodes,
                    shape: &shape,
                    rows: &rows,
                };
                row(ui, &tree, i, node, visibility, steerable);
            }
        }
    });
    crate::diag::ui_rect_visible(REGION_PARTS, scroll.inner_rect, ui.clip_rect());
}

/// The list a row is drawn in: its nodes, their shape and the unfolded rows
/// in the order shown.
struct Tree<'a> {
    nodes: &'a [ModelNode],
    shape: &'a Shape,
    rows: &'a [usize],
}

/// One node: its show box, its name (or the unnamed word), what the file
/// stores about its visibility, its markups and views, and on hover where a
/// borrowed name came from.
fn row(
    ui: &mut Ui,
    tree: &Tree<'_>,
    i: usize,
    node: &ModelNode,
    visibility: &mut Visibility,
    steerable: bool,
) {
    ui.horizontal(|ui| {
        let (nodes, shape) = (tree.nodes, tree.shape);
        if partstree::guides(ui, shape, i, visibility.folded.contains(&i)) {
            let mut folded = visibility.folded.clone();
            if !folded.remove(&i) {
                folded.insert(i);
            }
            visibility.fold(folded, "box"); // ui-text-exempt: trace token
        }
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
            let lit = visibility.selected.contains(&i);
            let r = ui
                .selectable_label(lit, text)
                .on_hover_text(m::part_highlight_tooltip());
            // ui-text-exempt: trace region name, never displayed
            let region = format!("{REGION_NAME_PREFIX}{i}");
            crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
            if let Some(click) = Click::read(&r) {
                visibility.click_row(tree.rows, i, click);
            }
            r.context_menu(|ui| super::partpick::menu(ui, nodes, visibility));
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
        visibility.hide(hidden, "box"); // ui-text-exempt: trace token
    }
}
