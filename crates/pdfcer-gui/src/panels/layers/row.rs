//! # `panels::layers::row` — one layer's row
//!
//! Contract: [`draw`] lays out a padlock (only when `/Locked`), the visibility
//! check box, the state word and the name, on a selection plate when the
//! canvas selection is on this layer. It returns the operator's tick and the
//! name's response, which senses click and drag so the tree can start a drag
//! from it. It emits the `layer-row` trace line every frame the
//! row is drawn, and scrolls the row into view on the frame the highlight
//! moves to it.

use std::collections::BTreeSet;

use pdfcer_core::layers::{Layer, Layers};
use pdfcer_core::object::ObjId;

use crate::text::panels as t;
use crate::text::panels::drawlayer as tdl;

/// Where the last-scrolled-to layer is remembered, so the list is scrolled
/// on the frame the selection changes and not on every frame after it.
// ui-text-exempt: a memory key, never displayed.
const SCROLL_MEMO: &str = "panel-layers-scrolled-to";

/// What every row in one frame shares.
pub(super) struct RowCtx<'a> {
    pub read: &'a Layers,
    /// The set the page is drawn from: the operator's override, else the
    /// document's own.
    pub hidden: &'a BTreeSet<ObjId>,
    /// The layer the canvas selection is on.
    pub highlighted: Option<ObjId>,
    /// Whether this mode authors layers (row menus).
    pub authoring: bool,
    /// The layer new content goes on (`OpenDoc::draw_layer`).
    pub drawing: Option<ObjId>,
    /// Set when a click on a registered layer's name sets or clears it.
    pub draw_pick: &'a std::cell::Cell<Option<Option<ObjId>>>,
}

/// What a drawn row reports.
pub(super) struct RowOut {
    /// The check box was clicked: the layer and the state asked for.
    pub toggled: Option<(ObjId, bool)>,
    /// The name, sensing click and drag.
    pub name: egui::Response,
}

/// Draw `l`'s row; `menu_extra` adds the tree's moves to its right-click menu.
pub(super) fn draw(
    ui: &mut egui::Ui,
    cx: &RowCtx<'_>,
    l: &Layer,
    menu_extra: impl FnOnce(&mut egui::Ui),
) -> RowOut {
    let name = super::row_name(l);
    let effective = !cx.hidden.contains(&l.id);
    let is_highlighted = cx.highlighted == Some(l.id);
    let row = ui.scope(|ui| {
        if is_highlighted {
            paint_plate(ui);
        }
        ui.horizontal(|ui| {
            let toggled = check_box(ui, l, effective);
            let current = cx.drawing == Some(l.id);
            if current {
                pencil(ui);
            }
            let response = ui.add(
                egui::Button::new(name.clone())
                    .frame(false)
                    .sense(egui::Sense::click_and_drag()),
            );
            let response = if l.in_default_config {
                if response.clicked() {
                    cx.draw_pick.set(Some((!current).then_some(l.id)));
                }
                response.on_hover_text(if current {
                    tdl::current_hover()
                } else {
                    tdl::choose_hover()
                })
            } else {
                response
            };
            if cx.authoring {
                super::authoring::row_menu(&response, cx.read, l, &name, menu_extra);
                super::authoring::publish_row(&name, response.rect);
            }
            let response = super::row_caveats(cx.read, l, effective)
                .into_iter()
                .fold(response, |r, note| r.on_hover_text(note));
            RowOut {
                toggled,
                name: response,
            }
        })
        .inner
    });
    if is_highlighted {
        scroll_once(ui, l.id, row.response.rect);
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "layer-row name={:?} visible={effective} default={} locked={} registered={} intent_view={} highlighted={is_highlighted} current={}",
            l.name,
            l.visible_by_default,
            l.locked,
            l.in_default_config,
            l.intent_view,
            u8::from(cx.drawing == Some(l.id))
        )
    });
    row.inner
}

/// The current layer's mark before its name: the pencil, drawn at the
/// height of a row.
fn pencil(ui: &mut egui::Ui) {
    let side = ui.spacing().interact_size.y;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    let color = ui.visuals().text_color();
    crate::icons::paint_icon(
        ui.painter(),
        crate::icons::Icon::EditText,
        rect.shrink(2.0),
        color,
        crate::icons::IconWeight::Regular,
    );
    response.on_hover_text(tdl::current_hover());
}

/// The highlight plate: a shape as well as a tint (never colour alone), in
/// the role the Objects panel's selected row uses. `strong()` text is
/// unusable here (`check-strong-text.sh`).
fn paint_plate(ui: &egui::Ui) {
    let plate = ui.available_rect_before_wrap();
    let plate = egui::Rect::from_min_size(
        plate.min,
        egui::vec2(plate.width(), ui.spacing().interact_size.y),
    );
    ui.painter().rect_filled(
        plate,
        ui.visuals().widgets.hovered.corner_radius,
        egui_shell::Theme::of(ui.ctx()).palette.selected_plate,
    );
}

/// Padlock, check box and state word. The padlock is drawn only when
/// `/Locked` is set, so the reason a box will not move shows before the
/// pointer arrives; the state is a word as well as a tick.
fn check_box(ui: &mut egui::Ui, l: &Layer, effective: bool) -> Option<(ObjId, bool)> {
    if l.locked {
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::splat(ui.spacing().icon_width),
            egui::Sense::hover(),
        );
        crate::icons::paint_icon(
            ui.painter(),
            crate::icons::Icon::Locked,
            rect,
            egui_shell::Theme::of(ui.ctx()).palette.text_muted,
            crate::icons::IconWeight::Regular,
        );
    }
    let mut want = effective;
    let cb = ui
        .add_enabled(!l.locked, egui::Checkbox::new(&mut want, ""))
        .on_hover_text(if l.locked {
            t::layer_locked_tooltip()
        } else {
            t::layer_toggle_tooltip()
        })
        .on_disabled_hover_text(t::layer_locked_tooltip());
    ui.label(if effective {
        t::layer_visible_marker()
    } else {
        t::layer_hidden_marker()
    });
    cb.changed().then_some((l.id, want))
}

/// Scroll `rect` into view on the frame the highlight moves to `id`, and not
/// after, so the operator can still scroll away from it.
fn scroll_once(ui: &egui::Ui, id: ObjId, rect: egui::Rect) {
    let key = egui::Id::new(SCROLL_MEMO);
    let last = ui.ctx().data(|d| d.get_temp::<ObjId>(key));
    if last != Some(id) {
        ui.ctx().data_mut(|d| d.insert_temp(key, id));
        ui.scroll_to_rect(rect, Some(egui::Align::Center));
    }
}
