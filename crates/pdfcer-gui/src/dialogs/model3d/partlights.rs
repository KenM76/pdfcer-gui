//! Which of the model's lights are on: the lamps it carries
//! (`AssembledModel::lights`) and the parts whose material gives off light
//! (`MaterialFinish::emissive`).
//!
//! A part switched off is drawn with no emission and a lamp switched off
//! with no colour, written into the drawn model itself so the picture, its
//! shadows and every saved picture agree. A lamp stays in the model's list
//! because a model with no lights is lit by a headlight instead
//! (`LightingScheme::Artwork`). Nothing is written into the file.
//!
//! A lamp names no tree node, so the lamps are listed in a folder the viewer
//! makes up, beside every glowing part that is not already in a folder of the
//! model's that holds glowing parts only.

use std::collections::BTreeSet;

use egui::{Sense, Ui, Vec2};
use pdfcer_3d::{AssembledModel, MaterialFinish, ModelLight, ModelLightKind, ModelNode};

use super::ModelView;
use super::partpick::{parents, unhidden};
use super::parts::Visibility;
use crate::text::model3d as m;

/// One row's light switch; the suffix is the node's index in the list.
pub const REGION_GLOW_PREFIX: &str = "model3d.part.glow."; // ui-text-exempt: trace region name, never displayed
/// The made-up folder's switch for everything it lists.
pub const REGION_ALL: &str = "model3d.lights.all"; // ui-text-exempt: trace region name, never displayed
/// One lamp's switch; the suffix is its index in the model's lights.
pub const REGION_LAMP_PREFIX: &str = "model3d.lights.lamp."; // ui-text-exempt: trace region name, never displayed
/// One gathered part's switch; the suffix is the node's index.
pub const REGION_LOOSE_PREFIX: &str = "model3d.lights.part."; // ui-text-exempt: trace region name, never displayed

/// The model's lights as the file gives them, and which are switched off.
#[derive(Debug, Clone, Default)]
pub(super) struct Lights {
    /// The model's own lamps, all on.
    lamps: Vec<ModelLight>,
    /// The model's own finishes, every emission as stored.
    finishes: Vec<Option<MaterialFinish>>,
    /// Nodes that draw a glowing surface, folders included.
    glowing: BTreeSet<usize>,
    /// Glowing parts gathered into the made-up folder.
    loose: Vec<usize>,
    /// Nodes whose surfaces give off nothing, their descendants with them.
    dark: BTreeSet<usize>,
    /// Lamps switched off, by index in `lamps`.
    lamps_off: BTreeSet<usize>,
    /// Why the drawn model no longer matches, until it is rewritten.
    stale: Option<&'static str>,
}

impl Lights {
    /// The lights of `model`, every one on.
    pub(super) fn of(model: &AssembledModel) -> Self {
        let glow: BTreeSet<usize> = model
            .mesh_placements
            .iter()
            .zip(&model.finishes)
            .filter(|(_, f)| f.is_some_and(|f| f.is_emissive()))
            .map(|(&p, _)| p)
            .collect();
        let glowing: BTreeSet<usize> = model
            .tree
            .iter()
            .enumerate()
            .filter(|(_, n)| glow.iter().any(|p| n.placements.contains(p)))
            .map(|(i, _)| i)
            .collect();
        let lights = Self {
            lamps: model.lights.clone(),
            finishes: model.finishes.clone(),
            loose: loose(&parents(&model.tree), &glowing),
            glowing,
            ..Self::default()
        };
        lights.trace("open"); // ui-text-exempt: trace token
        lights
    }

    /// Whether any row carries a light switch.
    pub(super) fn has_parts(&self) -> bool {
        !self.glowing.is_empty()
    }

    /// Whether node `i` draws a glowing surface.
    pub(super) fn glows(&self, i: usize) -> bool {
        self.glowing.contains(&i)
    }

    /// Whether node `i` gives off its light: neither it nor an ancestor is
    /// switched off.
    pub(super) fn lit(&self, nodes: &[ModelNode], mut i: usize) -> bool {
        loop {
            if self.dark.contains(&i) {
                return false;
            }
            match nodes.get(i).and_then(|n| n.parent) {
                Some(parent) => i = parent,
                None => return true,
            }
        }
    }

    /// Switch the glowing nodes among `which` on or off.
    pub(super) fn switch(
        &mut self,
        nodes: &[ModelNode],
        which: &[usize],
        on: bool,
        why: &'static str,
    ) {
        let parents = parents(nodes);
        let mut dark = self.dark.clone();
        for &i in which.iter().filter(|&&i| self.glows(i)) {
            if on {
                dark = unhidden(&parents, &dark, i);
            } else {
                dark.insert(i);
            }
        }
        if dark != self.dark {
            self.dark = dark;
            self.stale = Some(why);
        }
    }

    /// Switch lamp `i` on or off.
    fn lamp(&mut self, i: usize, on: bool) {
        let changed = if on {
            self.lamps_off.remove(&i)
        } else {
            i < self.lamps.len() && self.lamps_off.insert(i)
        };
        if changed {
            self.stale = Some("lamp"); // ui-text-exempt: trace token
        }
    }

    /// Whether everything the made-up folder lists is on.
    fn all_on(&self, nodes: &[ModelNode]) -> bool {
        self.lamps_off.is_empty() && self.loose.iter().all(|&i| self.lit(nodes, i))
    }

    /// Switch everything the made-up folder lists.
    fn all(&mut self, nodes: &[ModelNode], on: bool) {
        for i in 0..self.lamps.len() {
            self.lamp(i, on);
        }
        let loose = self.loose.clone();
        self.switch(nodes, &loose, on, "folder"); // ui-text-exempt: trace token
        if self.stale.is_some() {
            self.stale = Some("folder"); // ui-text-exempt: trace token
        }
    }

    /// Rewrite `model`'s lamps and emission for what is switched off;
    /// whether anything was rewritten, so the picture is out of date.
    fn apply(&mut self, model: &mut AssembledModel) -> bool {
        let Some(why) = self.stale.take() else {
            return false;
        };
        model.finishes = self.finishes.clone();
        for (mesh, p) in model.mesh_placements.iter().enumerate() {
            let covers = |&i: &usize| model.tree.get(i).is_some_and(|n| n.placements.contains(p));
            if self.dark.iter().any(covers)
                && let Some(Some(finish)) = model.finishes.get_mut(mesh)
            {
                finish.emissive = [0.0; 3];
            }
        }
        model.lights = self.lamps.clone();
        for &i in &self.lamps_off {
            if let Some(lamp) = model.lights.get_mut(i) {
                lamp.colour = [0.0; 3];
                lamp.ambient = [0.0; 3];
                lamp.specular = Some([0.0; 3]);
                lamp.intensity = 0.0;
            }
        }
        self.trace(why);
        true
    }

    fn trace(&self, why: &str) {
        crate::diag::trace(|| {
            let list = |all: &mut dyn Iterator<Item = &usize>| {
                let all: Vec<String> = all.map(ToString::to_string).collect();
                if all.is_empty() {
                    "-".to_owned()
                } else {
                    all.join("|")
                }
            };
            format!(
                // ui-text-exempt: diagnostic trace, never displayed
                "model-view-lights why={why} lamps={} lamps-off={} glowing={} gathered={} dark={}",
                self.lamps.len(),
                list(&mut self.lamps_off.iter()),
                list(&mut self.glowing.iter()),
                list(&mut self.loose.iter()),
                list(&mut self.dark.iter()),
            )
        });
    }
}

/// The glowing parts to gather: those with no glowing node below them whose
/// folder also holds something that does not glow.
fn loose(parents: &[Option<usize>], glowing: &BTreeSet<usize>) -> Vec<usize> {
    let parent = |i: usize| parents.get(i).copied().flatten();
    let above: BTreeSet<usize> = glowing.iter().filter_map(|&i| parent(i)).collect();
    let mixed: BTreeSet<Option<usize>> = (0..parents.len())
        .filter(|i| !glowing.contains(i))
        .map(parent)
        .collect();
    glowing
        .iter()
        .copied()
        .filter(|&i| !above.contains(&i) && mixed.contains(&parent(i)))
        .collect()
}

/// A row's light switch when node `i` glows, else the room one takes, so
/// the names stay in line. Draws nothing when no part of the model glows.
pub(super) fn glow_box(ui: &mut Ui, nodes: &[ModelNode], i: usize, lights: &mut Lights) {
    if !lights.has_parts() {
        return;
    }
    if !lights.glows(i) {
        let size = Vec2::new(ui.spacing().icon_width, ui.spacing().interact_size.y);
        ui.allocate_exact_size(size, Sense::hover());
        return;
    }
    let mut on = lights.lit(nodes, i);
    let r = ui
        .checkbox(&mut on, "")
        .on_hover_text(m::part_light_tooltip());
    // ui-text-exempt: trace region name, never displayed
    let region = format!("{REGION_GLOW_PREFIX}{i}");
    crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
    if r.changed() {
        lights.switch(nodes, &[i], on, "box"); // ui-text-exempt: trace token
    }
}

/// The made-up folder: one switch for all it lists, then each lamp and each
/// gathered part with its own. Draws nothing when it would list nothing.
pub(super) fn folder(
    ui: &mut Ui,
    nodes: &[ModelNode],
    lights: &mut Lights,
    visibility: &mut Visibility,
) {
    let count = lights.lamps.len() + lights.loose.len();
    if count == 0 {
        return;
    }
    let id = ui.make_persistent_id("model3d-lights-folder"); // ui-text-exempt: widget id, never displayed
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            let mut on = lights.all_on(nodes);
            let r = ui
                .checkbox(&mut on, m::lights_folder(count))
                .on_hover_text(m::lights_folder_tooltip());
            crate::diag::ui_rect_visible(REGION_ALL, r.rect, ui.clip_rect());
            if r.changed() {
                lights.all(nodes, on);
            }
        })
        .body(|ui| {
            egui::ScrollArea::vertical()
                .id_salt("model3d-lights-rows") // ui-text-exempt: widget id, never displayed
                .max_height(ui.spacing().interact_size.y * 6.0)
                .show(ui, |ui| rows(ui, nodes, lights, visibility));
        });
    ui.separator();
}

/// The made-up folder's rows.
fn rows(ui: &mut Ui, nodes: &[ModelNode], lights: &mut Lights, visibility: &mut Visibility) {
    for i in 0..lights.lamps.len() {
        let mut on = !lights.lamps_off.contains(&i);
        let name = lights.lamps.get(i).and_then(|l| l.name.as_deref());
        let kind = lights.lamps.get(i).map_or(0, |l| match l.kind {
            ModelLightKind::Point { .. } => 1,
            ModelLightKind::Spot { .. } => 2,
            ModelLightKind::Directional { .. } => 3,
            _ => 0,
        });
        let r = ui
            .checkbox(&mut on, m::lamp_row(name, i + 1, kind))
            .on_hover_text(m::part_light_tooltip());
        // ui-text-exempt: trace region name, never displayed
        let region = format!("{REGION_LAMP_PREFIX}{i}");
        crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
        if r.changed() {
            lights.lamp(i, on);
        }
    }
    for i in lights.loose.clone() {
        ui.horizontal(|ui| {
            let mut on = lights.lit(nodes, i);
            let r = ui
                .checkbox(&mut on, "")
                .on_hover_text(m::part_light_tooltip());
            // ui-text-exempt: trace region name, never displayed
            let region = format!("{REGION_LOOSE_PREFIX}{i}");
            crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
            if r.changed() {
                lights.switch(nodes, &[i], on, "folder-part"); // ui-text-exempt: trace token
            }
            let name = nodes.get(i).and_then(|n| n.name.clone());
            let name =
                name.unwrap_or_else(|| crate::text::panels::models::part_unnamed().to_owned());
            let chosen = visibility.selected.contains(&i);
            if ui
                .selectable_label(chosen, name)
                .on_hover_text(m::part_highlight_tooltip())
                .clicked()
            {
                visibility.select(BTreeSet::from([i]), "lights"); // ui-text-exempt: trace token
                visibility.show_row(&parents(nodes), i);
            }
        });
    }
}

impl ModelView {
    /// Write the light switches into the drawn model when they changed, and
    /// have the picture redrawn.
    pub(super) fn lights_sync(&mut self) {
        if self.lights.apply(&mut self.model.drawn) {
            self.parts.changed("lights"); // ui-text-exempt: trace token
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A glowing part beside a plain one is gathered; parts alone in a
    /// folder of glowing parts are not.**
    #[test]
    fn only_parts_outside_a_folder_of_their_own_are_gathered() {
        // 0 root; 1 plain; 2 glowing; 3 folder of 4 and 5, both glowing.
        let tree = [None, Some(0), Some(0), Some(0), Some(3), Some(3)];
        let glowing = BTreeSet::from([0, 2, 3, 4, 5]);
        assert_eq!(loose(&tree, &glowing), vec![2]);
    }
}
