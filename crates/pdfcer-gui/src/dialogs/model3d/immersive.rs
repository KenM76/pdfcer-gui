//! The viewer filling the screen, as a game does: the picture edge to edge
//! and one row of glyph buttons over its top-left corner (leave, Walk or
//! Orbit, Ghost and Level while walking, the gimbal, Fit). Where the viewer
//! is an OS window, full screen is that window's; where it is drawn inside
//! the application (Android, a single-window host), it is an area over the
//! application's whole screen.

use egui::{Rect, Ui, UiBuilder, Vec2};
use pdfcer_gui_base::icons::{self, Icon};
use pdfcer_gui_base::windowshape;

use super::ModelView;
use super::nav::{REGION_GIMBAL, REGION_IMMERSIVE, REGION_WALK};
use crate::text::panels::modelnav as t;

/// The egui id of the area the viewer fills the screen with.
const AREA_ID: &str = "model3d-immersive"; // ui-text-exempt: an egui id, never displayed
/// The button row's gap from the screen's corner, in points.
const INSET: f32 = 8.0;

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

    /// The picture filling `rect`, with the button row over it.
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
        let bar = Rect::from_min_size(
            rect.min + Vec2::splat(INSET),
            rect.size() - Vec2::splat(INSET * 2.0),
        );
        let mut row = ui.new_child(
            UiBuilder::new()
                .max_rect(bar)
                .layout(egui::Layout::left_to_right(egui::Align::Min)),
        );
        self.hud(&mut row);
        self.nav.hud.push(row.min_rect().expand(4.0));
    }

    /// The button row.
    fn hud(&mut self, ui: &mut Ui) {
        let leave = glyph(ui, Icon::ExitFullscreen, t::immersive_leave(), false);
        crate::diag::ui_rect_visible(REGION_IMMERSIVE, leave.rect, ui.clip_rect());
        if leave.clicked() {
            self.leave_immersive(ui.ctx());
        }
        let walking = self.nav.walk.is_some();
        let (icon, label) = if walking {
            (Icon::Orbit, t::orbit())
        } else {
            (Icon::Walk, t::walk())
        };
        let walk = glyph(ui, icon, label, false);
        crate::diag::ui_rect_visible(REGION_WALK, walk.rect, ui.clip_rect());
        if walk.clicked() {
            self.set_walking(!walking);
        }
        if walking {
            self.walk_switches(ui, true);
        }
        let gimbal = glyph(ui, Icon::Gimbal, t::gimbal(), self.nav.prefs.gimbal);
        crate::diag::ui_rect_visible(REGION_GIMBAL, gimbal.rect, ui.clip_rect());
        if gimbal.clicked() {
            self.nav.prefs.gimbal = !self.nav.prefs.gimbal;
            self.switched(ui.ctx());
        }
        let home = glyph(ui, Icon::FitPage, t::home(), false);
        crate::diag::ui_rect_visible(super::REGION_FIT, home.rect, ui.clip_rect());
        if home.clicked() {
            self.home();
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

/// A glyph button with `label` as its tooltip, large enough for a thumb.
pub(super) fn glyph(ui: &mut Ui, icon: Icon, label: &str, on: bool) -> egui::Response {
    let image = if on {
        icons::selected_image(ui, icon)
    } else {
        icons::image(ui, icon)
    };
    ui.add(
        egui::Button::image(image.fit_to_exact_size(Vec2::splat(28.0)))
            .selected(on)
            .min_size(Vec2::splat(44.0)),
    )
    .on_hover_text(label)
}
