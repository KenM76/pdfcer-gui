//! What a click selects in the viewer's part list and on its picture, and the
//! menu a right-click opens on either.
//!
//! The list follows the file-manager conventions of
//! `panels::objects::rowclick`: a plain click selects the row alone (again on
//! the only selected row clears it), Ctrl+click adds or removes one,
//! Shift+click selects the run from the anchor, Ctrl+Shift adds that run. On
//! the picture a click selects the deepest node that draws the surface under
//! the pointer, Ctrl toggles it, Shift adds it, and a click on nothing clears
//! the selection; the node's row is unfolded to and scrolled into view. A
//! right-click on something not selected selects it alone first, so the menu
//! always acts on the selection.
//!
//! While only the hidden parts are drawn, a click on one shows it again: it
//! leaves that view and is back in the regular one.

use std::collections::BTreeSet;

use egui::{Response, Ui};
use pdfcer_3d::{AssembledModel, ModelNode};

use super::ModelView;
use super::partlights::Lights;
use super::parts::Visibility;
use super::partstree::Shape;
use crate::text::model3d as m;

/// The switch that draws only the hidden parts.
pub const REGION_HIDDEN_ONLY: &str = "model3d.parts.hidden_only"; // ui-text-exempt: trace region name, never displayed
/// One menu item; the suffix is the item's key.
pub const REGION_MENU_PREFIX: &str = "model3d.menu."; // ui-text-exempt: trace region name, never displayed

/// One click, on a row or on the picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Click {
    /// The secondary button.
    pub secondary: bool,
    /// Ctrl (Cmd on macOS) held.
    pub command: bool,
    /// Shift held.
    pub shift: bool,
}

impl Click {
    /// The click `resp` carries, if any.
    pub fn read(resp: &Response) -> Option<Self> {
        let secondary = resp.secondary_clicked();
        if !resp.clicked() && !secondary {
            return None;
        }
        let mods = resp.ctx.input(|i| i.modifiers);
        Some(Self {
            secondary,
            command: mods.command,
            shift: mods.shift,
        })
    }

    /// Neither button modifier: the click that replaces the selection.
    fn plain(self) -> bool {
        !self.secondary && !self.command && !self.shift
    }
}

/// The deepest node that draws `mesh`: the part, not the folders it is in.
pub(super) fn node_of(model: &AssembledModel, mesh: usize) -> Option<usize> {
    let p = *model.mesh_placements.get(mesh)?;
    model
        .tree
        .iter()
        .enumerate()
        .filter(|(_, n)| n.placements.contains(&p))
        .max_by_key(|(_, n)| n.depth)
        .map(|(i, _)| i)
}

/// The indices of `i`'s ancestors, nearest first.
fn ancestors(parents: &[Option<usize>], i: usize) -> impl Iterator<Item = usize> + '_ {
    let up = |i: usize| parents.get(i).copied().flatten();
    std::iter::successors(up(i), move |&p| up(p))
}

/// Each node's parent, the shape the selection rules read.
pub(super) fn parents(nodes: &[ModelNode]) -> Vec<Option<usize>> {
    nodes.iter().map(|n| n.parent).collect()
}

/// `hidden` with `i` shown and nothing else: a hidden folder above it is
/// opened and its other children hidden in its place.
pub(super) fn unhidden(
    parents: &[Option<usize>],
    hidden: &BTreeSet<usize>,
    i: usize,
) -> BTreeSet<usize> {
    let mut out = hidden.clone();
    out.remove(&i);
    let mut below = i;
    for above in ancestors(parents, i) {
        if out.remove(&above) {
            let siblings = (0..parents.len()).filter(|&j| parents[j] == Some(above) && j != below);
            out.extend(siblings);
        }
        below = above;
    }
    out
}

impl Visibility {
    /// A click on the list row of node `i`; `rows` is the order shown.
    pub(super) fn click_row(&mut self, rows: &[usize], i: usize, click: Click) {
        let mut selected = self.selected.clone();
        if click.secondary {
            if selected.contains(&i) {
                return;
            }
            selected = BTreeSet::from([i]);
        } else if click.shift {
            let at = |n: usize| rows.iter().position(|&r| r == n);
            let run = match (self.anchor.and_then(at), at(i)) {
                (Some(a), Some(b)) => rows[a.min(b)..=a.max(b)].to_vec(),
                _ => vec![i],
            };
            if !click.command {
                selected.clear();
            }
            selected.extend(run);
        } else if click.command {
            if !selected.remove(&i) {
                selected.insert(i);
            }
        } else if selected.len() == 1 && selected.contains(&i) {
            selected.clear();
        } else {
            selected = BTreeSet::from([i]);
        }
        if !click.shift {
            self.anchor = Some(i);
        }
        self.select(selected, "highlight"); // ui-text-exempt: trace token
    }

    /// A click on the picture over `node`'s surface, or over nothing.
    fn pick(&mut self, parents: &[Option<usize>], node: Option<usize>, click: Click) {
        let Some(n) = node else {
            if click.plain() {
                self.select(BTreeSet::new(), "pick"); // ui-text-exempt: trace token
            }
            return;
        };
        if self.hidden_only && !click.secondary {
            self.hide(unhidden(parents, &self.hidden, n), "restore"); // ui-text-exempt: trace token
            return;
        }
        let mut selected = self.selected.clone();
        if click.secondary {
            if !selected.contains(&n) {
                selected = BTreeSet::from([n]);
            }
        } else if click.command {
            if !selected.remove(&n) {
                selected.insert(n);
            }
        } else if click.shift {
            selected.insert(n);
        } else {
            selected = BTreeSet::from([n]);
        }
        self.anchor = Some(n);
        self.select(selected, "pick"); // ui-text-exempt: trace token
        self.show_row(parents, n);
    }

    /// Unfold the list down to node `i` and scroll its row into view.
    pub(super) fn show_row(&mut self, parents: &[Option<usize>], i: usize) {
        let mut folded = self.folded.clone();
        let mut opened = false;
        for above in ancestors(parents, i) {
            opened |= folded.remove(&above);
        }
        if opened {
            self.fold(folded, "reveal"); // ui-text-exempt: trace token
        }
        self.reveal = Some(i);
    }
}

/// The switch above the list that draws only the hidden parts.
pub(super) fn hidden_only_box(ui: &mut Ui, visibility: &mut Visibility) {
    let mut on = visibility.hidden_only;
    let r = ui
        .checkbox(&mut on, m::parts_hidden_only())
        .on_hover_text(m::parts_hidden_only_tooltip());
    crate::diag::ui_rect_visible(REGION_HIDDEN_ONLY, r.rect, ui.clip_rect());
    if r.changed() {
        visibility.view(on);
    }
}

/// One menu item; `true` the frame it is chosen, and the menu closes.
fn item(ui: &mut Ui, key: &str, text: &str) -> bool {
    let r = ui.button(text);
    let region = format!("{REGION_MENU_PREFIX}{key}");
    crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
    if r.clicked() {
        ui.close();
    }
    r.clicked()
}

/// The items that switch the selected parts' own light off and on; none when
/// no selected part glows.
// ui-text-exempt: the item keys are trace tokens, never displayed
fn light_items(ui: &mut Ui, nodes: &[ModelNode], chosen: &[usize], lights: &mut Lights) {
    let glow = |on: bool| {
        chosen
            .iter()
            .any(|&i| lights.glows(i) && lights.lit(nodes, i) == on)
    };
    let (lit, dark) = (glow(true), glow(false));
    if lit && item(ui, "light_off", m::parts_light_off()) {
        lights.switch(nodes, chosen, false, "menu");
    }
    if dark && item(ui, "light_on", m::parts_light_on()) {
        lights.switch(nodes, chosen, true, "menu");
    }
    if lit || dark {
        ui.separator();
    }
}

/// The menu of a row and of the picture: what can be done with the selected
/// parts, then the views of the whole model, then the list's folding.
// ui-text-exempt: the item keys are trace tokens, never displayed
pub(super) fn menu(ui: &mut Ui, nodes: &[ModelNode], v: &mut Visibility, lights: &mut Lights) {
    let chosen: Vec<usize> = v.selected.iter().copied().collect();
    let parents = parents(nodes);
    light_items(ui, nodes, &chosen, lights);
    if chosen.iter().any(|&i| v.shows(nodes, i)) && item(ui, "hide", m::parts_hide()) {
        let mut hidden = v.hidden.clone();
        hidden.extend(chosen.iter().copied());
        v.hide(hidden, "hide");
    }
    if chosen.iter().any(|&i| !v.shows(nodes, i)) && item(ui, "show", m::parts_show()) {
        let hidden = chosen
            .iter()
            .fold(v.hidden.clone(), |h, &i| unhidden(&parents, &h, i));
        v.hide(hidden, "show");
    }
    if !chosen.is_empty() {
        if item(ui, "isolate", m::part_isolate()) {
            let kin = |i: usize, j: usize| ancestors(&parents, i).any(|a| a == j);
            let keep = |j: usize| chosen.iter().any(|&i| j == i || kin(i, j) || kin(j, i));
            let hidden = (0..nodes.len()).filter(|&j| !keep(j)).collect();
            v.hide(hidden, "isolate");
        }
        ui.separator();
    }
    let view = if v.hidden_only {
        m::parts_back_to_shown()
    } else {
        m::parts_hidden_only()
    };
    if item(ui, "hidden_only", view) {
        v.view(!v.hidden_only);
    }
    if item(ui, "all", m::parts_show_all()) {
        v.hide(BTreeSet::new(), "all");
    }
    if item(ui, "stored", m::parts_as_stored()) {
        v.hide(Visibility::stored(nodes).hidden, "stored");
    }
    ui.separator();
    if item(ui, "fold", m::parts_fold_all()) {
        let shape = Shape::of(parents);
        v.fold(shape.branches(), "all");
    }
    if item(ui, "unfold", m::parts_unfold_all()) {
        v.fold(BTreeSet::new(), "none");
    }
}

impl ModelView {
    /// Select from this frame's click on the picture, and open the menu on a
    /// right-click. Orbiting only: walking keeps the pointer for looking.
    pub(super) fn pick_input(&mut self, response: &Response) {
        if self.model.drawn.tree.is_empty() || self.nav.walk.is_some() {
            return;
        }
        let rect = response.rect;
        if let Some(click) = Click::read(response)
            && !self.measure.on
            && !self.nav.options_open
            && let Some(at) = response.interact_pointer_pos()
            && !self.nav.hud.iter().any(|r| r.contains(at))
            && !self
                .gimbal(rect)
                .is_some_and(|g| g.zones().iter().any(|z| z.contains(at)))
        {
            let node = self
                .hit_at(rect, at)
                .and_then(|hit| node_of(&self.model.drawn, hit.mesh));
            self.parts
                .pick(&parents(&self.model.drawn.tree), node, click);
        }
        let (nodes, parts, lights) = (&self.model.drawn.tree, &mut self.parts, &mut self.lights);
        response.context_menu(|ui| menu(ui, nodes, parts, lights));
        self.lights_sync();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: Click = Click {
        secondary: false,
        command: false,
        shift: false,
    };

    /// **Showing one part of a hidden folder hides its siblings instead.**
    #[test]
    fn one_part_out_of_a_hidden_folder() {
        // 0 root; 1 folder under it; 2, 3, 4 parts in the folder.
        let nodes = [None, Some(0), Some(1), Some(1), Some(1)];
        let hidden = BTreeSet::from([1]);
        assert_eq!(unhidden(&nodes, &hidden, 3), BTreeSet::from([2, 4]));
        assert_eq!(unhidden(&nodes, &BTreeSet::from([3]), 3), BTreeSet::new());
    }

    /// **Shift selects the run from the anchor in the order shown; Ctrl
    /// toggles one; a right-click inside the selection leaves it.**
    #[test]
    fn the_list_follows_the_file_manager_rules() {
        let rows = [0, 1, 2, 3];
        let mut v = Visibility::stored(&[]);
        v.click_row(&rows, 1, PLAIN);
        v.click_row(
            &rows,
            3,
            Click {
                shift: true,
                ..PLAIN
            },
        );
        assert_eq!(v.selected, BTreeSet::from([1, 2, 3]));
        v.click_row(
            &rows,
            2,
            Click {
                command: true,
                ..PLAIN
            },
        );
        assert_eq!(v.selected, BTreeSet::from([1, 3]));
        v.click_row(
            &rows,
            3,
            Click {
                secondary: true,
                ..PLAIN
            },
        );
        assert_eq!(v.selected, BTreeSet::from([1, 3]));
        v.click_row(
            &rows,
            2,
            Click {
                secondary: true,
                ..PLAIN
            },
        );
        assert_eq!(v.selected, BTreeSet::from([2]));
    }

    /// **A click on a hidden part in the hidden-only view shows it, and
    /// selects nothing.**
    #[test]
    fn a_click_in_the_hidden_view_restores() {
        let nodes = [None, Some(0), Some(0)];
        let mut v = Visibility::stored(&[]);
        v.hidden = BTreeSet::from([1, 2]);
        v.view(true);
        v.pick(&nodes, Some(2), PLAIN);
        assert_eq!(v.hidden, BTreeSet::from([1]));
        assert!(v.selected.is_empty());
    }
}
