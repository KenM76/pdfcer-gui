//! The viewer's options behind one glyph at the picture's top-left corner,
//! in a window, on a phone and full screen alike, on four tabs: View (the
//! views, projection, lens, axes, mesh fit and full screen), Look (lighting,
//! effects, render mode), Moving (the controls, navigation and eye height)
//! and Picture (what *Save picture…* draws at, and, full screen where the
//! footer is gone, the picture and save buttons). Each tab shows a glyph
//! with its name. Controls the glyph row over the picture already shows are
//! left out while it is shown. The menu resizes from its corner, keeps its
//! size, opens on the tab last shown, stays open while its switches are used
//! and closes on a click outside it or Escape.

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
/// The menu's width when first opened, in points.
const WIDTH: f32 = 330.0;
/// The smallest the menu is resized to, in points.
const SMALLEST: [f32; 2] = [240.0, 160.0];
/// The egui id the menu's size is kept under.
const SIZE_ID: &str = "model3d-options-size"; // ui-text-exempt: an egui id, never displayed
/// Each tab's glyph, in [`TAB_WORDS`]' order.
const TAB_ICONS: [Icon; 4] = [Icon::Model3d, Icon::Lighting, Icon::Walk, Icon::Snapshot];
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
    ///
    /// It opens no lower than the picture's bottom, so the footer's buttons
    /// below the picture stay in reach while it is open; dragged from its
    /// corner it grows to the window's bottom.
    pub(super) fn options_button(&mut self, ui: &mut Ui, rect: Rect) {
        let side = self.glyph_side(ui.ctx());
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
        let (opens_at, room) = heights(
            rect.bottom() - at.bottom() - BELOW,
            (ui.ctx().content_rect().bottom() - at.bottom() - BELOW).max(side * 4.0),
        );
        let shown = egui::Popup::menu(&button)
            .open_bool(&mut open)
            .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
            .show(|ui| {
                egui::Resize::default()
                    .id_salt(SIZE_ID)
                    .default_size([WIDTH, opens_at])
                    .min_size(SMALLEST)
                    .max_height(room)
                    .show(ui, |ui| {
                        let shown = self.nav.options_tab;
                        self.tab_row(ui);
                        ui.separator();
                        let fresh = opening || shown != self.nav.options_tab;
                        let mut scroll = egui::ScrollArea::vertical()
                            .auto_shrink(false)
                            .max_height(ui.available_height().max(TABS_HIGH));
                        if fresh {
                            scroll = scroll.vertical_scroll_offset(0.0);
                        }
                        scroll.show(ui, |ui| self.options(ui));
                    });
            });
        if let Some(menu) = shown {
            crate::diag::ui_rect_visible(REGION_MENU, menu.response.rect, ui.clip_rect());
        }
        self.nav.options_open = open;
    }

    /// The tabs, each a glyph and its name, the one shown marked; they wrap
    /// in a narrow menu.
    fn tab_row(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            let names = s::tabs().into_iter().zip(TAB_WORDS).zip(TAB_ICONS);
            for (i, ((name, word), icon)) in names.enumerate() {
                let on = self.nav.options_tab == i;
                let image = if on {
                    icons::selected_image(ui, icon)
                } else {
                    icons::image(ui, icon)
                };
                let tab = ui.add(
                    egui::Button::image_and_text(image.fit_to_exact_size(Vec2::splat(20.0)), name)
                        .selected(on)
                        .min_size(Vec2::new(0.0, 30.0)),
                );
                if tab.clicked() {
                    self.nav.options_tab = i;
                }
                // ui-text-exempt: trace region name, never displayed
                let region = format!("{REGION_TAB}{word}");
                crate::diag::ui_rect_visible(&region, tab.rect, ui.clip_rect());
            }
        });
    }

    /// The tab shown's rows.
    fn options(&mut self, ui: &mut Ui) {
        let full = self.filling_screen(ui.ctx());
        let row = self.nav.prefs.controls;
        match self.nav.options_tab {
            0 => self.view_tab(ui, row),
            1 => {
                self.lighting_control(ui);
                self.effects_control(ui);
                self.looks_control(ui);
            }
            2 => self.moving_tab(ui, row),
            _ => {
                self.shot_control(ui);
                if full {
                    ui.separator();
                    ui.horizontal_wrapped(|ui| self.buttons(ui));
                }
            }
        }
    }

    /// The View tab: the views, projection, axes, mesh fit and full screen;
    /// Fit only where the glyph `row` is not shown.
    fn view_tab(&mut self, ui: &mut Ui, row: bool) {
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
            if !row {
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
        self.lens_control(ui);
        ui.horizontal_wrapped(|ui| self.axes_control(ui));
        ui.horizontal_wrapped(|ui| self.mesh_fit_control(ui));
        ui.horizontal_wrapped(|ui| self.full_screen_control(ui));
    }

    /// The Moving tab: the controls, navigation and eye height; the
    /// navigation switches only where the glyph `row` is not shown.
    fn moving_tab(&mut self, ui: &mut Ui, row: bool) {
        self.controls_switch(ui);
        ui.separator();
        if !row {
            ui.horizontal_wrapped(|ui| self.nav_controls(ui));
        } else if self.nav.walk.is_some() {
            self.speed_slider(ui);
        }
        ui.horizontal_wrapped(|ui| self.eye_height_control(ui));
    }
}

/// The menu's opening height and the most it may grow to: `wanted` and
/// `room`, each at least the smallest the menu is, the opening height at most
/// the room; a value not finite counts as the smallest. A window too small
/// for the menu gets the smallest menu, cut by its edge.
fn heights(wanted: f32, room: f32) -> (f32, f32) {
    let least = |v: f32| {
        if v.is_finite() {
            v.max(SMALLEST[1])
        } else {
            SMALLEST[1]
        }
    };
    let room = least(room);
    (least(wanted).min(room), room)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The heights a 1x1 and a zero-size window give, and a size not yet
    /// known, lay the menu out without a panic.
    #[test]
    fn a_window_too_small_for_the_menu_lays_it_out() {
        for (wanted, room) in [
            (1.0 - BELOW, 4.0),
            (-BELOW, 0.0),
            (f32::NAN, f32::NAN),
            (f32::INFINITY, f32::NEG_INFINITY),
        ] {
            let (opens, most) = heights(wanted, room);
            assert!(opens.is_finite() && opens >= SMALLEST[1] && opens <= most);
            let _ = opens.clamp(SMALLEST[1], most);
        }
        assert_eq!(heights(300.0, 500.0), (300.0, 500.0));
        assert_eq!(heights(900.0, 500.0), (500.0, 500.0));
    }
}
