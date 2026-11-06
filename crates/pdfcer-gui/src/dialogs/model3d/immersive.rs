//! The viewer filling the screen, as a game does: the picture edge to edge.
//! Its row of glyph buttons over the picture's top-left corner (full screen
//! or leave, Walk or Orbit, Ghost, Level and Stairs while walking, the
//! gimbal, Fit) is drawn in every layout while the controls are shown,
//! thumb-sized where a thumb uses it and smaller in a desktop window. Where the viewer
//! is an OS window, full screen is that window's; where it is drawn inside
//! the application (Android, a single-window host), it is an area over the
//! application's whole screen.

use egui::{Rect, Ui, UiBuilder, Vec2};
use pdfcer_gui_base::icons::{self, Icon};
use pdfcer_gui_base::windowshape;

use super::ModelView;
use super::nav::{REGION_GIMBAL, REGION_IMMERSIVE, REGION_WALK};
use crate::text::model3dmeasure as tm;
use crate::text::panels::modelnav as t;

/// The egui id of the area the viewer fills the screen with.
const AREA_ID: &str = "model3d-immersive"; // ui-text-exempt: an egui id, never displayed
/// The button row's gap from the picture's corner, in points.
const INSET: f32 = 8.0;
/// A glyph button's side where a thumb uses it, in points.
const THUMB: f32 = 44.0;
/// A glyph button's side in a desktop window, in points.
const DESK: f32 = 30.0;

impl ModelView {
    /// The viewer over the application's whole screen, in place of its
    /// dialog window. Escape, or the leave button, goes back to the window.
    pub(super) fn show_immersive(&mut self, ctx: &egui::Context) {
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new(AREA_ID))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                ui.set_min_size(screen.size());
                ui.set_max_size(screen.size());
                ui.painter()
                    .rect_filled(screen, 0.0, ui.visuals().extreme_bg_color);
                self.immersive_body(ui, screen);
            });
    }

    /// The picture filling `rect`, with the button row over it while the
    /// controls are shown.
    pub(super) fn immersive_body(&mut self, ui: &mut Ui, rect: Rect) {
        let (escape, closing) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.viewport().close_requested(),
            )
        });
        // Escape with the options open closes them only.
        if escape
            && !closing
            && !crate::canvas::textedit::composing(ui.ctx())
            && !self.nav.options_open
            && !egui::Popup::is_any_open(ui.ctx())
        {
            self.leave_immersive(ui.ctx());
        }
        self.picture(ui, rect.size());
    }

    /// The glyph row over `rect`, the picture, while the controls are shown;
    /// it wraps onto further rows in a narrow picture.
    pub(super) fn hud_row(&mut self, ui: &mut Ui, rect: Rect) {
        if !self.nav.prefs.controls {
            return;
        }
        let side = self.glyph_side(ui.ctx());
        // The row starts right of the options glyph in the corner.
        let clear = INSET + side;
        let bar = Rect::from_min_size(
            rect.min + egui::vec2(clear + INSET, INSET),
            (rect.size() - egui::vec2(clear + INSET * 2.0, INSET * 2.0)).max(Vec2::splat(side)),
        );
        let mut row = ui.new_child(
            UiBuilder::new()
                .max_rect(bar)
                .layout(egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true)),
        );
        self.hud(&mut row, side);
        self.nav.hud.push(row.min_rect().expand(4.0));
    }

    /// The side of a glyph button over the picture: thumb-sized full screen
    /// and on a phone, smaller in a desktop window.
    pub(super) fn glyph_side(&self, ctx: &egui::Context) -> f32 {
        if self.filling_screen(ctx) || egui_shell::ribbon::phone::is_phone(ctx) {
            THUMB
        } else {
            DESK
        }
    }

    /// The button row, of glyphs `side` points square.
    fn hud(&mut self, ui: &mut Ui, side: f32) {
        self.screen_glyph(ui, side);
        let walking = self.nav.walk.is_some();
        let (icon, label) = if walking {
            (Icon::Orbit, t::orbit())
        } else {
            (Icon::Walk, t::walk())
        };
        let walk = glyph(ui, icon, label, false, side);
        crate::diag::ui_rect_visible(REGION_WALK, walk.rect, ui.clip_rect());
        if walk.clicked() {
            self.set_walking(!walking);
        }
        if walking {
            self.walk_switches(ui, Some(side));
        }
        let gimbal = glyph(ui, Icon::Gimbal, t::gimbal(), self.nav.prefs.gimbal, side);
        crate::diag::ui_rect_visible(REGION_GIMBAL, gimbal.rect, ui.clip_rect());
        if gimbal.clicked() {
            self.nav.prefs.gimbal = !self.nav.prefs.gimbal;
            self.switched(ui.ctx());
        }
        let on = self.measure.on;
        let measure = glyph(ui, Icon::Measure, tm::MEASURE, on, side);
        crate::diag::ui_rect_visible(
            super::measure_tab::REGION_GLYPH,
            measure.rect,
            ui.clip_rect(),
        );
        if measure.clicked() {
            self.measure.switch(!on);
        }
        let home = glyph(ui, Icon::FitPage, t::home(), false, side);
        crate::diag::ui_rect_visible(super::REGION_FIT, home.rect, ui.clip_rect());
        if home.clicked() {
            self.home();
        }
    }

    /// Leave full screen when filling it; otherwise fill it, the OS window's
    /// own full screen where the viewer is one, else the area.
    fn screen_glyph(&mut self, ui: &mut Ui, side: f32) {
        if self.filling_screen(ui.ctx()) {
            let leave = glyph(ui, Icon::ExitFullscreen, t::immersive_leave(), false, side);
            crate::diag::ui_rect_visible(REGION_IMMERSIVE, leave.rect, ui.clip_rect());
            if leave.clicked() {
                self.leave_immersive(ui.ctx());
            }
            return;
        }
        let enter = glyph(ui, Icon::Fullscreen, t::immersive(), false, side)
            .on_hover_text(t::immersive_tooltip());
        let region = if self.native {
            super::REGION_FULL_SCREEN
        } else {
            REGION_IMMERSIVE
        };
        crate::diag::ui_rect_visible(region, enter.rect, ui.clip_rect());
        if enter.clicked() {
            if self.native {
                Self::ask_full_screen(ui.ctx(), true);
            } else {
                self.set_immersive(true);
            }
        }
    }

    /// The whole model in view: orbiting, fitted.
    pub(super) fn home(&mut self) {
        if self.nav.walk.is_some() {
            self.set_walking(false);
        }
        self.orbit.refit();
    }

    /// Back to the viewer's window: the OS window out of full screen, or the
    /// area given up.
    fn leave_immersive(&mut self, ctx: &egui::Context) {
        if self.nav.immersive {
            self.set_immersive(false);
        } else if self.native && windowshape::fullscreen_believed(ctx) {
            self.escape_left_full_screen = true;
            windowshape::set_fullscreen(ctx, false);
        }
    }

    /// Whether the viewer fills the screen now.
    pub(super) fn filling_screen(&self, ctx: &egui::Context) -> bool {
        self.nav.immersive || (self.native && windowshape::fullscreen_believed(ctx))
    }
}

/// A glyph button `side` points square with `label` as its tooltip.
pub(super) fn glyph(ui: &mut Ui, icon: Icon, label: &str, on: bool, side: f32) -> egui::Response {
    let image = if on {
        icons::selected_image(ui, icon)
    } else {
        icons::image(ui, icon)
    };
    ui.add(
        egui::Button::image(image.fit_to_exact_size(Vec2::splat(side * 0.64)))
            .selected(on)
            .min_size(Vec2::splat(side)),
    )
    .on_hover_text(label)
}
