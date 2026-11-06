//! Orbiting: the pointer and fingers on the picture (one finger or the
//! primary button turns, two fingers or the other buttons pan, a pinch or
//! the wheel zooms at the pointer), the arrow and zoom keys, a double tap
//! that centres on the surface tapped, a single tap that shows or hides the
//! controls, and the gimbal's taps.

use egui::{Key, Response, Ui, Vec2};
use pdfcer_3d_nav::Orbit;

use super::{ModelView, ORBIT_RATE, PITCH_LIMIT};

/// Radians a held arrow key turns per second.
const KEY_TURN: f64 = 1.5;
/// The zoom factor per second a held + or - applies.
const KEY_ZOOM: f64 = 2.5;
/// Pan `orbit` by a drag of `delta` points over a picture `height` points
/// high, so the content follows the pointer at the target's depth. Every
/// drag pans here, and [`Orbit::pan_by`] holds the pan in range.
fn drag_pan(orbit: &mut Orbit, delta: Vec2, height: f32) {
    let per_point = 2.0 / f64::from(height.max(1.0)) / orbit.zoom;
    orbit.pan_by([
        -f64::from(delta.x) * per_point,
        f64::from(delta.y) * per_point,
    ]);
}

impl ModelView {
    /// Steer the orbit from this frame's input on the picture.
    pub(super) fn orbit_input(&mut self, ui: &Ui, response: &Response) {
        let before = self.orbit;
        let rect = response.rect;
        let gimbal = self.gimbal(rect);
        let on_gimbal = |p: egui::Pos2| {
            gimbal
                .as_ref()
                .and_then(|g| g.zones().iter().position(|z| z.contains(p)))
        };
        let pressed_on_gimbal = ui
            .input(|i| i.pointer.press_origin())
            .is_some_and(|p| on_gimbal(p).is_some());
        if response.clicked()
            && let (Some(g), Some(zone)) = (
                gimbal.as_ref(),
                response.interact_pointer_pos().and_then(on_gimbal),
            )
        {
            self.gimbal_tap(g, zone);
        } else if !pressed_on_gimbal {
            self.steer(ui, response);
        }
        if response.clicked()
            && let Some(at) = response.interact_pointer_pos()
            && on_gimbal(at).is_none()
            && !self.nav.hud.iter().any(|r| r.contains(at))
        {
            self.tap_picture(ui.input(|i| i.time), at);
        }
        if response.double_clicked()
            && !self.measure.on
            && let Some(at) = response.interact_pointer_pos()
            && on_gimbal(at).is_none()
            && let Some(hit) = self.hit_at(rect, at)
        {
            self.orbit.centre_on(&self.bounds, hit.point);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-view-centred at={:.4},{:.4},{:.4}",
                    hit.point[0], hit.point[1], hit.point[2]
                )
            });
        }
        let keyed = self.orbit_keys(ui, response);
        let held = ui.input(|i| i.pointer.any_down() || i.multi_touch().is_some());
        self.nav.moving = self.orbit != before && (held || keyed);
    }

    /// The arrows turn and + and - zoom while the pointer is over the picture
    /// or it fills the screen, and no text box has the keys. Whether a key
    /// moved it.
    fn orbit_keys(&mut self, ui: &Ui, response: &Response) -> bool {
        if !(response.hovered() || self.nav.immersive)
            || crate::canvas::textedit::composing(ui.ctx())
        {
            return false;
        }
        let (dt, keys) = ui.input(|i| {
            let down = |k: Key| f64::from(u8::from(i.key_down(k)));
            (
                f64::from(i.stable_dt.min(0.1)),
                [
                    down(Key::ArrowRight) - down(Key::ArrowLeft),
                    down(Key::ArrowDown) - down(Key::ArrowUp),
                    down(Key::Plus) + down(Key::Equals) - down(Key::Minus),
                ],
            )
        });
        if keys == [0.0; 3] {
            return false;
        }
        self.orbit.yaw -= keys[0] * KEY_TURN * dt;
        self.orbit.pitch =
            (self.orbit.pitch + keys[1] * KEY_TURN * dt).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        if keys[2] != 0.0 {
            let rect = response.rect;
            let aspect = f64::from(rect.width().max(1.0) / rect.height().max(1.0));
            let factor = (keys[2] * KEY_ZOOM.ln() * dt).exp();
            self.orbit.zoom_at(
                &self.bounds,
                self.reach,
                aspect,
                factor,
                [0.0, 0.0],
                self.frame(aspect),
            );
        }
        ui.ctx().request_repaint();
        true
    }

    /// Pick the point this turn is about, held until the pointer or every
    /// finger lifts: the mouse turns about the model point it pressed on, as
    /// SolidWorks does, and a finger about the nearest model point under the
    /// picture's centre, which a thumb would hide. Either falls back to the
    /// centre's point, then to the point before.
    fn pivot(&mut self, ui: &Ui, rect: egui::Rect) {
        let aspect = f64::from(rect.width().max(1.0) / rect.height().max(1.0));
        let Ok(camera) = self.view_camera(aspect) else {
            return;
        };
        let pressed = ui
            .input(|i| {
                (!i.any_touches())
                    .then(|| i.pointer.press_origin())
                    .flatten()
            })
            .filter(|p| rect.contains(*p));
        let hit = pressed
            .and_then(|at| self.hit_at(rect, at))
            .or_else(|| self.hit_at(rect, rect.center()));
        let Some(hit) = hit else {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| "model-orbit-pivot at=none".to_owned());
            return;
        };
        self.orbit.anchor_at(&camera, hit.point);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-orbit-pivot at={:.4},{:.4},{:.4}",
                hit.point[0], hit.point[1], hit.point[2]
            )
        });
    }

    /// Turn the camera from this frame's pointer on the picture. On a touch
    /// screen one finger orbits, and two fingers pinch to zoom and drag to pan;
    /// a two-finger gesture never orbits, though its first finger is also the
    /// pointer.
    pub(super) fn steer(&mut self, ui: &Ui, response: &egui::Response) {
        let delta = response.drag_delta();
        let touch = response
            .hover_pos()
            .and_then(|_| ui.input(egui::InputState::multi_touch));
        if let Some(touch) = touch {
            drag_pan(
                &mut self.orbit,
                touch.translation_delta,
                response.rect.height(),
            );
        }
        if touch.is_none() && response.drag_started_by(egui::PointerButton::Primary) {
            self.pivot(ui, response.rect);
        }
        if touch.is_none()
            && response.dragged_by(egui::PointerButton::Primary)
            && delta != egui::Vec2::ZERO
        {
            self.orbit.yaw -= f64::from(delta.x) * ORBIT_RATE;
            self.orbit.pitch = (self.orbit.pitch + f64::from(delta.y) * ORBIT_RATE)
                .clamp(-PITCH_LIMIT, PITCH_LIMIT);
        }
        if (response.dragged_by(egui::PointerButton::Secondary)
            || response.dragged_by(egui::PointerButton::Middle))
            && delta != egui::Vec2::ZERO
        {
            drag_pan(&mut self.orbit, delta, response.rect.height());
        }
        if let Some(at) = response.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            // A pinch, or Ctrl with the wheel, as a factor; 1.0 when neither.
            let pinch = ui.input(egui::InputState::zoom_delta);
            if scroll != 0.0 || pinch != 1.0 {
                let rect = response.rect;
                let height = f64::from(rect.height().max(1.0));
                let offset = [
                    f64::from(at.x - rect.center().x) / height,
                    f64::from(rect.center().y - at.y) / height,
                ];
                let aspect = f64::from(rect.width().max(1.0)) / height;
                let factor = f64::from(scroll / 200.0).exp() * f64::from(pinch);
                self.orbit.zoom_at(
                    &self.bounds,
                    self.reach,
                    aspect,
                    factor,
                    offset,
                    self.frame(aspect),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_3d::Bounds;
    use pdfcer_3d_nav::Upright;

    #[test]
    fn a_pan_that_is_not_finite_is_ignored_and_a_huge_one_is_held() {
        let mut orbit = super::super::fileview::named(0, true, Upright::default());
        // 100 points over a 200-point picture at zoom 1 pans one radius.
        drag_pan(&mut orbit, Vec2::new(-25.0, -50.0), 200.0);
        assert_eq!(orbit.pan, [0.25, -0.5]);
        drag_pan(&mut orbit, Vec2::new(f32::NAN, 0.0), 200.0);
        drag_pan(&mut orbit, Vec2::new(0.0, f32::INFINITY), 200.0);
        assert_eq!(orbit.pan, [0.25, -0.5]);
        // A drag that would pan a billion radii.
        drag_pan(&mut orbit, Vec2::new(-1e11, 1e11), 200.0);
        assert_eq!(orbit.pan, [pdfcer_3d_nav::PAN_LIMIT; 2]);
        let bounds = Bounds {
            min: [-1.0; 3],
            max: [1.0; 3],
        };
        for zoom in [0.05, 1.0, 50.0] {
            orbit.zoom = zoom;
            let camera = orbit.camera(&bounds, 2.0, 1.0, None).unwrap();
            let shift = (0..3).map(|i| camera.target[i].abs()).fold(0.0, f64::max);
            assert!(shift > 100.0 && shift.is_finite(), "{shift}");
            assert!(camera.eye.iter().all(|c| c.is_finite()));
        }
    }
}
