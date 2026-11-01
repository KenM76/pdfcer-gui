//! The viewer's OS window: Minimize, and full screen from its button or F11,
//! the conventional viewer key.

use egui::Ui;

use super::{ModelView, REGION_FULL_SCREEN, REGION_MINIMIZE};
use crate::text::panels::models as t;

impl ModelView {
    /// F11 into and out of full screen, and the minimised state traced:
    /// every frame, whether or not the options holding the buttons are open.
    /// Escape out of full screen is the full-screen body's. Only on a real
    /// OS window; an embedded fallback has no screen of its own to fill.
    pub(super) fn window_keys(&mut self, ui: &Ui) {
        if !self.native {
            return;
        }
        self.trace_minimized(ui);
        if ui.input(|i| i.key_pressed(egui::Key::F11)) {
            let full = pdfcer_gui_base::windowshape::fullscreen_believed(ui.ctx());
            Self::ask_full_screen(ui.ctx(), !full);
        }
    }

    /// Minimize, since full screen has no title bar, and the full-screen
    /// switch.
    pub(super) fn full_screen_control(&mut self, ui: &mut Ui) {
        if !self.native {
            return;
        }
        let minimize = ui
            .button(t::view_minimize())
            .on_hover_text(t::view_minimize_tooltip());
        crate::diag::ui_rect_visible(REGION_MINIMIZE, minimize.rect, ui.clip_rect());
        if minimize.clicked() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        let full = pdfcer_gui_base::windowshape::fullscreen_believed(ui.ctx());
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
        if button.clicked() {
            Self::ask_full_screen(ui.ctx(), !full);
        }
    }

    fn ask_full_screen(ctx: &egui::Context, on: bool) {
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-view-full-screen asked={on} escape=false"));
        pdfcer_gui_base::windowshape::set_fullscreen(ctx, on);
    }

    /// The window's minimised state as the OS reports it, with the camera,
    /// traced when it changes.
    fn trace_minimized(&mut self, ui: &Ui) {
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
