//! The viewer's options behind one glyph at the picture's top-left corner,
//! in a window, on a phone and full screen alike, on four tabs: View (the
//! views, projection, axes, mesh fit and full screen), Look (lighting,
//! effects, render mode), Moving (the controls, navigation and eye height)
//! and Picture (what *Save picture…* draws at, and, full screen where the
//! footer is gone, the picture and save buttons). The menu opens on the tab
//! last shown, stays open while its switches are used and closes on a click
//! outside it or Escape.

use egui::{Rect, Ui, UiBuilder, Vec2};
use pdfcer_gui_base::icons::{self, Icon};

use super::{ModelView, REGION_FIT, REGION_VIEW_PREFIX, phone};

/// The Perspective switch, greyed while walking.
pub const REGION_PERSPECTIVE: &str = "model3d.perspective"; // ui-text-exempt: trace region name, never displayed
use crate::text::model3dshot as s;
use crate::text::panels::modelnav as tn;
use crate::text::panels::models as t;

/// The open menu's frame.
pub const REGION_MENU: &str = "model3d.options.menu"; // ui-text-exempt: trace region name, never displayed
/// The options glyph's published region.
pub const REGION_OPTIONS: &str = "model3d.options"; // ui-text-exempt: trace region name, never displayed
/// The glyph's gap from the picture's corner, in points.
const INSET: f32 = 8.0;
/// The glyph's side in a desktop window, and where a thumb uses it.
const SIDE: (f32, f32) = (28.0, 44.0);
/// The width the glyph takes from the picture's left edge where a thumb
/// uses it, so a row beside it starts clear of it.
pub(super) const THUMB_CLEAR: f32 = INSET + SIDE.1;
/// The menu's width, in points.
const WIDTH: f32 = 300.0;
/// Room kept below the menu, in points: its frame's margin and shadow.
const BELOW: f32 = 24.0;
/// Room the tab row and its rule take above the rows, in points.
const TABS_HIGH: f32 = 40.0;
/// Each tab's published region, after [`REGION_TAB`].
const TAB_WORDS: [&str; 4] = ["view", "look", "moving", "picture"]; // ui-text-exempt: trace region names, never displayed
/// The tabs' region prefix: `model3d.options.tab.view` and so on.
pub const REGION_TAB: &str = "model3d.options.tab."; // ui-text-exempt: trace region name, never displayed

impl ModelView {
    /// The options glyph over `rect`'s top-left corner and its menu.
    ///
    /// The menu's open state is the viewer's own, not egui's remembered
    /// popup: egui remembers one popup per viewport, so a drop-down opened
    /// inside the menu would close it. For the same reason a click closes
    /// the menu only when it lands outside every popup layer.
    ///
    /// The menu scrolls once it is taller than the room below the glyph:
    /// egui moves a popup that does not fit up the window, over the glyph,
    /// and the click meant to close it then lands in the menu instead. It
    /// opens scrolled to the top, as a menu does.
    pub(super) fn options_button(&mut self, ui: &mut Ui, rect: Rect) {
        let thumb = self.filling_screen(ui.ctx()) || egui_shell::ribbon::phone::is_phone(ui.ctx());
        let side = if thumb { SIDE.1 } else { SIDE.0 };
        let at = Rect::from_min_size(
            egui::pos2(rect.left() + INSET, rect.top() + INSET),
            Vec2::splat(side),
        );
        let mut child = ui.new_child(UiBuilder::new().max_rect(at));
        let image = icons::image(&child, Icon::Settings).fit_to_exact_size(Vec2::splat(side * 0.6));
        let button = child
            .add(egui::Button::image(image).min_size(Vec2::splat(side)))
            .on_hover_text(tn::options_tooltip());
        crate::diag::ui_rect_visible(REGION_OPTIONS, button.rect, ui.clip_rect());
        self.nav.hud.push(at.expand(4.0));
        let mut open = self.nav.options_open;
        if button.clicked() {
            open = !open;
        } else if open && clicked_outside_popups(ui.ctx(), ui.layer_id()) {
            open = false;
        }
        let opening = open && !self.nav.options_open;
        let room = (ui.ctx().content_rect().bottom() - at.bottom() - BELOW).max(side * 4.0);
        let shown = egui::Popup::menu(&button)
            .open_bool(&mut open)
            .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
            .width(WIDTH)
            .show(|ui| {
                let shown = self.nav.options_tab;
                self.tab_row(ui);
                ui.separator();
                let fresh = opening || shown != self.nav.options_tab;
                let mut scroll = egui::ScrollArea::vertical().max_height(room - TABS_HIGH);
                if fresh {
                    scroll = scroll.vertical_scroll_offset(0.0);
                }
                scroll.show(ui, |ui| self.options(ui));
            });
        if let Some(menu) = shown {
            crate::diag::ui_rect_visible(REGION_MENU, menu.response.rect, ui.clip_rect());
        }
        self.nav.options_open = open;
    }

    /// The tabs, the one shown marked.
    fn tab_row(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for (i, (name, word)) in s::tabs().into_iter().zip(TAB_WORDS).enumerate() {
                let tab = ui.selectable_value(&mut self.nav.options_tab, i, name);
                // ui-text-exempt: trace region name, never displayed
                let region = format!("{REGION_TAB}{word}");
                crate::diag::ui_rect_visible(&region, tab.rect, ui.clip_rect());
            }
        });
    }

    /// The tab shown's rows.
    fn options(&mut self, ui: &mut Ui) {
        let full = self.filling_screen(ui.ctx());
        match self.nav.options_tab {
            0 => self.view_tab(ui, full),
            1 => {
                self.lighting_control(ui);
                self.effects_control(ui);
                self.looks_control(ui);
            }
            2 => self.moving_tab(ui, full),
            _ => {
                self.shot_control(ui);
                if full {
                    ui.separator();
                    ui.horizontal_wrapped(|ui| self.buttons(ui));
                }
            }
        }
    }

    /// The View tab: the views, projection, axes, mesh fit and full screen.
    fn view_tab(&mut self, ui: &mut Ui, full: bool) {
        ui.horizontal_wrapped(|ui| {
            self.file_view_control(ui);
            self.saved_views_control(ui);
            for (i, name) in t::view_names().into_iter().enumerate() {
                let button = ui.button(name);
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{REGION_VIEW_PREFIX}{i}"),
                    button.rect,
                    ui.clip_rect(),
                );
                if button.clicked() {
                    self.named = i;
                    self.orbiting();
                    self.orbit = super::Orbit::named(i, self.orbit.perspective, self.upright);
                }
            }
            // Full screen has Fit on its own row of glyphs.
            if !full {
                let fit = phone::glyph_button(
                    ui,
                    Icon::FitPage,
                    t::view_reset(),
                    t::view_reset_tooltip(),
                );
                crate::diag::ui_rect_visible(REGION_FIT, fit.rect, ui.clip_rect());
                if fit.clicked() {
                    self.home();
                }
            }
        });
        // Walking is always in perspective; the switch is the orbit's.
        let orbiting = self.nav.walk.is_none();
        let switch = ui
            .add_enabled(
                orbiting,
                egui::Checkbox::new(&mut self.orbit.perspective, t::view_perspective()),
            )
            .on_hover_text(t::view_perspective_tooltip())
            .on_disabled_hover_text(t::view_perspective_walking_tooltip());
        crate::diag::ui_rect_visible(REGION_PERSPECTIVE, switch.rect, ui.clip_rect());
        ui.horizontal_wrapped(|ui| self.axes_control(ui));
        ui.horizontal_wrapped(|ui| self.mesh_fit_control(ui));
        ui.horizontal_wrapped(|ui| self.full_screen_control(ui));
    }

    /// The Moving tab: the controls, navigation and eye height.
    fn moving_tab(&mut self, ui: &mut Ui, full: bool) {
        self.controls_switch(ui);
        // Full screen has the navigation switches on its own row of glyphs.
        if !full {
            ui.separator();
            ui.horizontal_wrapped(|ui| self.nav_controls(ui));
        }
        ui.horizontal_wrapped(|ui| self.eye_height_control(ui));
    }
}

/// A click this frame on `home`, the viewer's own layer, or on any layer
/// below the popups: one that is neither the menu nor a drop-down in it.
fn clicked_outside_popups(ctx: &egui::Context, home: egui::LayerId) -> bool {
    ctx.input(|i| {
        i.pointer
            .any_click()
            .then(|| i.pointer.interact_pos())
            .flatten()
    })
    .is_some_and(|pos| {
        ctx.layer_id_at(pos)
            .is_none_or(|l| l == home || l.order < egui::Order::Foreground)
    })
}
