//! The viewer's OS window: Minimize, and full screen from its button or F11
//! with Escape out of it, the conventional viewer keys.

use egui::Ui;

use super::{ModelView, REGION_FULL_SCREEN, REGION_MINIMIZE};
use crate::text::panels::models as t;

impl ModelView {
    /// Full screen from the button or F11, and Escape out of it: the
    /// conventional viewer keys. Only on a real OS window; an embedded
    /// fallback has no screen of its own to fill.
    pub(super) fn full_screen_control(&mut self, ui: &mut Ui) {
        use pdfcer_gui_base::windowshape;
        if !self.native {
            return;
        }
        let ctx = ui.ctx().clone();
        self.minimize_control(ui);
        let full = windowshape::fullscreen_believed(&ctx);
        let (label, tip) = if full {
            (
                t::view_full_screen_leave(),
                t::view_full_screen_leave_tooltip(),
            )
        } else {
            (t::view_full_screen(), t::view_full_screen_tooltip())
        };
        let button = ui.button(label).on_hover_text(tip);
        crate::diag::ui_rect_visible(REGION_FULL_SCREEN, button.rect, ui.clip_rect());
        let (f11, escape, closing) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::F11),
                i.key_pressed(egui::Key::Escape),
                i.viewport().close_requested(),
            )
        });
        let wanted = if button.clicked() || f11 {
            Some(!full)
        } else if full && escape && !closing {
            self.escape_left_full_screen = true;
            Some(false)
        } else {
            None
        };
        if let Some(on) = wanted {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-view-full-screen asked={on} escape={}",
                    full && escape
                )
            });
            windowshape::set_fullscreen(&ctx, on);
        }
    }

    /// A Minimize button, since full screen has no title bar, and a trace of
    /// the window's minimised state as the OS reports it, with the camera.
    fn minimize_control(&mut self, ui: &mut Ui) {
        let button = ui
            .button(t::view_minimize())
            .on_hover_text(t::view_minimize_tooltip());
        crate::diag::ui_rect_visible(REGION_MINIMIZE, button.rect, ui.clip_rect());
        if button.clicked() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        let minimized = ui.input(|i| i.viewport().minimized);
        if minimized.is_some() && minimized != self.minimized {
            self.minimized = minimized;
            let o = &self.orbit;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-view-window minimized={} yaw={:.3} pitch={:.3} zoom={:.3} pan={:.4},{:.4}",
                    minimized == Some(true),
                    o.yaw,
                    o.pitch,
                    o.zoom,
                    o.pan[0],
                    o.pan[1]
                )
            });
        }
    }
}
