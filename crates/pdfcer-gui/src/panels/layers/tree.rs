//! # `panels::layers::tree` — the Layers list as the document arranges it
//!
//! Contract: walks [`layerorder::tree`] of the document's `/D /Order`. A
//! folder is a fold with a folder icon and its label and no check box (it has
//! no visibility of its own); a layer with sublayers is a fold whose header is
//! the layer's row; a leaf is the row; an unlabelled grouping indents what it
//! holds. Folds start open, and when the canvas selection moves to a layer
//! inside a closed fold, that fold opens so the highlight is seen.
//!
//! Every row's right-click menu carries the moves [`layerorder::offers`]
//! lists; a folder's adds Rename and Remove. Every row is a drag source (see
//! [`super::drag`]). Each node drawn emits `layer-node path= kind= name=
//! open=`, which is what a driven check reads the tree's shape from.
//!
//! [`layerorder::tree`]: pdfcer_gui_base::layerorder::tree
//! [`layerorder::offers`]: pdfcer_gui_base::layerorder::offers

use pdfcer_core::object::ObjId;
use pdfcer_gui_base::layeraction::{LayerAction, OrderAction};
use pdfcer_gui_base::layerorder::{self as order, Kind, Node, Offer, OutOf};

use super::authoring::publish_keyed;
use super::drag::{self, Drawn};
use super::row::{self, RowCtx};
use crate::app::actions::Action;
use crate::text::panels::layeredit as t;

// ui-text-exempt: trace region names and memory keys, never displayed.
const REGION_FOLDER: &str = "panel.layers.folder";
// ui-text-exempt: trace region name prefix, never displayed.
const REGION_FOLD: &str = "panel.layers.fold";
// ui-text-exempt: trace region name, never displayed.
const REGION_MOVE_UP: &str = "panel.layers.menu.move_up";
// ui-text-exempt: trace region name, never displayed.
const REGION_MOVE_DOWN: &str = "panel.layers.menu.move_down";
// ui-text-exempt: trace region name, never displayed.
const REGION_MOVE_OUT: &str = "panel.layers.menu.move_out";
// ui-text-exempt: trace region name, never displayed.
const REGION_MOVE_INTO: &str = "panel.layers.menu.move_into";
// ui-text-exempt: trace region name prefix, never displayed.
const REGION_INTO: &str = "panel.layers.menu.into";
// ui-text-exempt: trace region name, never displayed.
const REGION_RENAME: &str = "panel.layers.menu.rename_folder";
// ui-text-exempt: trace region name, never displayed.
const REGION_REMOVE: &str = "panel.layers.menu.remove_folder";
// ui-text-exempt: a memory key, never displayed.
const OPENED_FOR: &str = "panel-layers-opened-for";

/// What one frame of the tree produced.
#[derive(Default)]
pub(super) struct Walk {
    /// A check box the operator clicked.
    pub toggled: Option<(ObjId, bool)>,
    rows: Vec<Drawn>,
}

/// Draw the tree, then layers `/Order` does not list, flat after it.
pub(super) fn show(
    ui: &mut egui::Ui,
    cx: &RowCtx<'_>,
    tree: &[Node],
    actions: &mut Vec<Action>,
) -> Option<(ObjId, bool)> {
    open_to_highlight(ui.ctx(), cx, tree);
    let mut w = Walk::default();
    let strip = (ui.max_rect().left(), ui.max_rect().right());
    let walker = Walker { cx, tree, strip };
    walker.nodes(ui, tree, &mut Vec::new(), &mut w, actions);
    for l in cx.read.layers.iter().filter(|l| !l.in_order) {
        let out = row::draw(ui, cx, l, |_| {});
        w.toggled = w.toggled.or(out.toggled);
    }
    drag::finish(ui, tree, &w.rows, actions);
    w.toggled
}

/// When the highlight moves to a layer, open every fold above it.
fn open_to_highlight(ctx: &egui::Context, cx: &RowCtx<'_>, tree: &[Node]) {
    let key = egui::Id::new(OPENED_FOR);
    let last = ctx.data(|d| d.get_temp::<Option<ObjId>>(key)).flatten();
    if last == cx.highlighted {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(key, cx.highlighted));
    let Some(path) = cx.highlighted.and_then(|id| order::path_of_layer(tree, id)) else {
        return;
    };
    for k in 1..path.len() {
        let prefix = path.get(..k).unwrap_or_default();
        if let Some(id) = order::node_at(tree, prefix).and_then(|n| fold_id(prefix, n)) {
            let mut state =
                egui::collapsing_header::CollapsingState::load_with_default_open(ctx, id, true);
            state.set_open(true);
            state.store(ctx);
        }
    }
}

/// The fold state's id: a folder by place and label, a layer by identity.
fn fold_id(path: &[usize], n: &Node) -> Option<egui::Id> {
    match &n.kind {
        Kind::Folder(label) => Some(egui::Id::new(("layer-folder", path, label))),
        Kind::Layer(id) => Some(egui::Id::new(("layer-fold", *id))),
        Kind::Grouping => None,
    }
}

/// What every node in one walk shares.
struct Walker<'a> {
    cx: &'a RowCtx<'a>,
    tree: &'a [Node],
    /// The list's left and right edges, for each row's full-width strip.
    strip: (f32, f32),
}

impl Walker<'_> {
    fn nodes(
        &self,
        ui: &mut egui::Ui,
        nodes: &[Node],
        path: &mut Vec<usize>,
        w: &mut Walk,
        actions: &mut Vec<Action>,
    ) {
        for (i, n) in nodes.iter().enumerate() {
            path.push(i);
            self.node(ui, n, path, w, actions);
            path.pop();
        }
    }

    fn node(
        &self,
        ui: &mut egui::Ui,
        n: &Node,
        path: &mut Vec<usize>,
        w: &mut Walk,
        actions: &mut Vec<Action>,
    ) {
        let layer = match &n.kind {
            Kind::Layer(id) => self.cx.read.layers.iter().find(|l| l.id == *id),
            _ => None,
        };
        match (&n.kind, layer) {
            (Kind::Folder(_), _) | (Kind::Layer(_), Some(_)) if !n.children.is_empty() => {
                self.fold(ui, n, layer, path, w, actions);
            }
            (Kind::Folder(_), _) => self.fold(ui, n, None, path, w, actions),
            (Kind::Layer(_), Some(l)) => {
                let top = ui.cursor().top();
                ui.horizontal(|ui| {
                    ui.add_space(ui.spacing().indent);
                    let out = row::draw(ui, self.cx, l, |ui| self.moves(ui, path, actions));
                    w.toggled = w.toggled.or(out.toggled);
                    drag::begin(ui, &out.name, path);
                    self.record(w, path, top, ui.min_rect().bottom(), out.name.rect.left());
                });
                trace(path, "layer", &l.name, true);
            }
            _ => {
                trace(path, "group", "", true);
                let salt = egui::Id::new(("layer-grouping", path.clone()));
                ui.indent(salt, |ui| {
                    self.nodes(ui, &n.children, path, w, actions);
                });
            }
        }
    }

    /// A folder, or a layer with sublayers: a fold whose header is the
    /// folder's label or the layer's row.
    fn fold(
        &self,
        ui: &mut egui::Ui,
        n: &Node,
        layer: Option<&pdfcer_core::layers::Layer>,
        path: &mut Vec<usize>,
        w: &mut Walk,
        actions: &mut Vec<Action>,
    ) {
        let Some(id) = fold_id(path, n) else {
            return;
        };
        let state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
        let open = state.is_open();
        let slot = w.rows.len();
        let header = state.show_header(ui, |ui| match layer {
            Some(l) => {
                let out = row::draw(ui, self.cx, l, |ui| self.moves(ui, path, actions));
                w.toggled = w.toggled.or(out.toggled);
                out.name
            }
            None => self.folder_header(ui, n, path, actions),
        });
        let (toggle, head, _) = header.body(|ui| self.nodes(ui, &n.children, path, w, actions));
        drag::begin(ui, &head.inner, path);
        let name = layer.map_or_else(
            || n.folder_label().unwrap_or_default().to_owned(),
            |l| l.name.clone(),
        );
        publish_keyed(REGION_FOLD, &name, toggle.rect);
        let rect = head.response.rect;
        let row = self.drawn(path, rect.top(), rect.bottom(), head.inner.rect.left());
        w.rows.insert(slot, row);
        trace(
            path,
            if layer.is_some() { "layer" } else { "folder" },
            &name,
            open,
        );
    }

    fn folder_header(
        &self,
        ui: &mut egui::Ui,
        n: &Node,
        path: &[usize],
        actions: &mut Vec<Action>,
    ) -> egui::Response {
        let label = n.folder_label().unwrap_or_default();
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::splat(ui.spacing().icon_width),
            egui::Sense::hover(),
        );
        crate::icons::paint_icon(
            ui.painter(),
            crate::icons::Icon::FontFolders,
            rect,
            egui_shell::Theme::of(ui.ctx()).palette.text_muted,
            crate::icons::IconWeight::Regular,
        );
        let name = ui
            .add(
                egui::Button::new(label)
                    .frame(false)
                    .sense(egui::Sense::click_and_drag()),
            )
            .on_hover_text(t::folder_tooltip());
        publish_keyed(REGION_FOLDER, label, name.rect);
        if self.cx.authoring {
            name.context_menu(|ui| {
                let rename = ui.button(t::menu_rename_folder());
                crate::diag::ui_rect(REGION_RENAME, rename.rect);
                if rename.clicked() {
                    super::folders::open_rename(ui.ctx(), path, label);
                    ui.close();
                }
                let remove = ui.button(t::menu_remove_folder());
                crate::diag::ui_rect(REGION_REMOVE, remove.rect);
                if remove.clicked() {
                    let at = path.to_vec();
                    actions.push(Action::Layer(LayerAction::Order(
                        OrderAction::DeleteFolder { at },
                    )));
                    ui.close();
                }
                self.moves(ui, path, actions);
            });
        }
        name
    }

    /// The move items of a row's menu, under a separator.
    fn moves(&self, ui: &mut egui::Ui, path: &[usize], actions: &mut Vec<Action>) {
        let offers = order::offers(self.tree, path);
        if offers.is_empty() {
            return;
        }
        ui.separator();
        let mut into = Vec::new();
        let mut picked = None;
        for offer in offers {
            let (words, region, m) = match offer {
                Offer::Up(m) => (t::menu_move_up().to_owned(), REGION_MOVE_UP, m),
                Offer::Down(m) => (t::menu_move_down().to_owned(), REGION_MOVE_DOWN, m),
                Offer::Out(m, of) => (self.out_words(&of), REGION_MOVE_OUT, m),
                Offer::Into(m, label) => {
                    into.push((m, label));
                    continue;
                }
            };
            let b = ui.button(words);
            crate::diag::ui_rect(region, b.rect);
            if b.clicked() {
                picked = Some(m);
            }
        }
        if !into.is_empty() {
            let sub = ui.menu_button(t::menu_move_into(), |ui| {
                for (m, label) in into {
                    let b = ui.button(&label);
                    publish_keyed(REGION_INTO, &label, b.rect);
                    if b.clicked() {
                        picked = Some(m);
                    }
                }
            });
            crate::diag::ui_rect(REGION_MOVE_INTO, sub.response.rect);
        }
        if let Some(m) = picked {
            actions.push(Action::Layer(LayerAction::Order(OrderAction::Move(m))));
            ui.close();
        }
    }

    fn out_words(&self, of: &OutOf) -> String {
        match of {
            OutOf::Folder(label) => t::menu_move_out(label),
            OutOf::Layer(id) => {
                t::menu_move_out(&super::layer_name_for(self.cx.read, *id).unwrap_or_default())
            }
            OutOf::Grouping => t::menu_move_out_of_group().to_owned(),
        }
    }

    fn drawn(&self, path: &[usize], top: f32, bottom: f32, indent_left: f32) -> Drawn {
        Drawn {
            path: path.to_vec(),
            rect: egui::Rect::from_min_max(
                egui::pos2(self.strip.0, top),
                egui::pos2(self.strip.1.max(self.strip.0), bottom),
            ),
            indent_left,
        }
    }

    fn record(&self, w: &mut Walk, path: &[usize], top: f32, bottom: f32, indent_left: f32) {
        w.rows.push(self.drawn(path, top, bottom, indent_left));
    }
}

fn trace(path: &[usize], kind: &str, name: &str, open: bool) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "layer-node path={} kind={kind} name={name:?} open={open}",
            order::dotted(path)
        )
    });
}
