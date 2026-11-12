//! Walking's mouse past the stick reader's left-button drag: what each
//! button's drag turns (seen from behind, always as a game's: the left
//! circles the camera round the figure, the right steers it), both buttons walking forward, and free look — the
//! picture holding the mouse so moving it turns the view with no button
//! down. Design: `DESIGNS.md` § O411 / O412.
//!
//! Free look confines the pointer to the window and hides it, re-asked
//! every frame it holds, so a missed frame cannot leave the pointer hidden
//! after it lets go. It lets go on Escape, on leaving walking, on opening
//! the options, when the window loses focus, and when the viewer closes.

use egui::{CursorGrab, PointerButton, Response, Ui, Vec2, ViewportCommand};

use super::ModelView;
use super::bindings::Drag;

/// Radians the look turns per point the mouse moves.
pub(super) const LOOK_RATE: f64 = 0.005;

impl ModelView {
    /// The mouse this frame, given the stick reader's left-button drag
    /// `left` and whether the drone sticks are offered: whether both buttons
    /// are down over the picture, which walks forward when so bound, and
    /// whether a held mouse moved.
    pub(super) fn walk_mouse(
        &mut self,
        ui: &Ui,
        response: &Response,
        left: Vec2,
        sticks: bool,
    ) -> (bool, bool) {
        let bind = self.nav.prefs.bind;
        if self.free_look(ui, response) {
            let moved = ui.input(|i| i.pointer.motion().map(|m| m / i.pixels_per_point()));
            let moved = moved.unwrap_or_else(|| ui.input(|i| i.pointer.delta()));
            self.drag_turn(Drag::Turn, moved, sticks);
            return (false, moved != Vec2::ZERO);
        }
        let right = response
            .dragged_by(PointerButton::Secondary)
            .then(|| response.drag_delta());
        if self.nav.prefs.behind {
            self.drag_behind(left, right);
        } else {
            self.drag_turn(bind.left_drag, left, sticks);
            if let Some(right) = right {
                self.drag_turn(bind.right_drag, right, sticks);
            }
        }
        let both = bind.both_forward
            && response.contains_pointer()
            && ui.input(|i| i.pointer.primary_down() && i.pointer.secondary_down());
        (both, false)
    }

    /// Turn the body or the camera by a mouse move of `delta` points.
    fn drag_turn(&mut self, drag: Drag, delta: Vec2, sticks: bool) {
        if delta == Vec2::ZERO {
            return;
        }
        let (yaw, pitch) = self.nav.prefs.bind.look(delta, LOOK_RATE);
        let Some(walk) = &mut self.nav.walk else {
            return;
        };
        match drag {
            Drag::Auto if sticks => walk.look(yaw, pitch),
            Drag::Auto | Drag::Turn => walk.turn(yaw, pitch),
            Drag::Look => walk.look(yaw, pitch),
            Drag::Nothing => {}
        }
    }

    /// Whether the picture holds the mouse this frame: taken by a click on
    /// the picture while free look is bound and the pointer is a mouse,
    /// given back when walking, the options or the window's focus end.
    fn free_look(&mut self, ui: &Ui, response: &Response) -> bool {
        let focused = ui.input(|i| i.viewport().focused != Some(false));
        let allowed = self.nav.prefs.bind.free_look
            && self.nav.walk.is_some()
            && !self.nav.options_open
            && !self.nav.sticks.touch()
            && focused;
        if !allowed {
            self.hold_mouse(ui.ctx(), false);
            return false;
        }
        if !self.nav.mouse_held && response.clicked() {
            self.hold_mouse(ui.ctx(), true);
        }
        if self.nav.mouse_held {
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            ui.ctx().request_repaint();
        }
        self.nav.mouse_held
    }

    /// Take or give back the mouse.
    pub(super) fn hold_mouse(&mut self, ctx: &egui::Context, hold: bool) {
        if self.nav.mouse_held == hold {
            return;
        }
        self.nav.mouse_held = hold;
        let grab = if hold {
            CursorGrab::Confined
        } else {
            CursorGrab::None
        };
        ctx.send_viewport_cmd(ViewportCommand::CursorGrab(grab));
        ctx.send_viewport_cmd(ViewportCommand::CursorVisible(!hold));
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-nav-mouse held={hold}"));
    }

    /// Escape while the picture holds the mouse gives it back and does
    /// nothing else: it neither leaves full screen nor closes the viewer.
    /// Run before anything else reads the frame's Escape.
    pub(super) fn mouse_escape(&mut self, ui: &mut Ui) {
        if !self.nav.mouse_held {
            return;
        }
        let escape = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if escape {
            self.hold_mouse(ui.ctx(), false);
            self.escape_left_full_screen = true;
        }
    }
}
