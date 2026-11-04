//! Walking's input: the drone sticks and their buttons, the keys that stand
//! in for them (W A S D, the arrows or keypad, I J K L, U and O, Shift), a
//! drag to look, a pinch or Ctrl with the wheel to zoom, the wheel to step,
//! the gimbal's taps, and a double tap that goes to the surface tapped.
//!
//! The drone sticks are offered, while the controls are shown, on a touch
//! screen and whenever the viewer fills the screen. With them, a drag turns the camera apart from the
//! heading, as the camera stick does; without them (a mouse in a window) a
//! drag turns the body, as a desktop game's mouse-look does, so going
//! forward goes where the view faces.

use egui::{Key, Pos2, Rect, Response, Ui, Vec2};

use super::ModelView;
use super::drone::{self, CAMERA, Drone, LEFT, RIGHT, shaped};
use super::gimbal::{Gimbal, ZONES as GIMBAL_ZONES};
use super::stick::Bases;
use super::walk::Intent;

/// Radians the look turns per point dragged.
const LOOK_RATE: f64 = 0.005;
/// Eye heights a wheel notch's 50 points steps.
const WHEEL_STEP: f64 = 0.5;
/// Radians a fully tilted left stick turns the body per second.
const STICK_TURN: f64 = 1.6;
/// Radians a fully tilted camera stick turns the camera per second: left
/// and right, up and down.
const CAMERA_TURN: (f64, f64) = (1.6, 1.0);

impl ModelView {
    /// Move the walker from this frame's input on the picture.
    pub(super) fn walk_input(&mut self, ui: &Ui, response: &Response) {
        let rect = response.rect;
        let gimbal = self.gimbal(rect);
        let offered =
            self.nav.prefs.controls && (self.nav.sticks.touch() || self.filling_screen(ui.ctx()));
        let drone = offered.then(|| Drone::layout(rect));
        // The gimbal's areas first, a fixed count of them, so an area's index
        // means the same thing when the finger lifts as when it landed: the
        // first touch is what offers the drone and adds its buttons.
        let mut zones: Vec<Rect> = gimbal
            .as_ref()
            .map_or_else(|| vec![Rect::NOTHING; GIMBAL_ZONES], Gimbal::zones);
        let buttons_from = zones.len();
        if let Some(d) = &drone {
            zones.extend(d.buttons);
        }
        let (centres, radii) = (drone.map(|d| d.centres), drone.map(|d| d.radii));
        let bases = Bases {
            centres: centres.as_ref().map_or(&[], |c| c.as_slice()),
            radii: radii.as_ref().map_or(&[], |r| r.as_slice()),
        };
        let hud = std::mem::take(&mut self.nav.hud);
        let reading = self.nav.sticks.read(ui, rect, bases, &zones, &hud);
        self.nav.hud = hud;
        let (keyed, keys, wheel) = self.walk_keys(ui, response);
        let tilts = self.tilts(&reading.sticks, keys);
        self.nav.drone = drone.map(|d| (d, tilts));

        // Mode 2: left is throttle and yaw, right is pitch and roll.
        let intent = Intent {
            forward: shaped(tilts[RIGHT].y),
            right: shaped(tilts[RIGHT].x),
            rise: shaped(tilts[LEFT].y),
            run: ui.input(|i| i.modifiers.shift),
        };
        let dt = f64::from(ui.input(|i| i.stable_dt.min(0.1)));
        let pinch = if self.nav.sticks.touch() {
            reading.pinch
        } else {
            ui.input(egui::InputState::zoom_delta)
        };
        let before = self.nav.walk;
        if let Some(walk) = &mut self.nav.walk {
            let (drag_yaw, drag_pitch) = (
                -f64::from(reading.look.x) * LOOK_RATE,
                f64::from(reading.look.y) * LOOK_RATE,
            );
            if drone.is_some() {
                walk.look(drag_yaw, drag_pitch);
            } else {
                walk.turn(drag_yaw, drag_pitch);
            }
            walk.turn(-shaped(tilts[LEFT].x) * STICK_TURN * dt, 0.0);
            walk.look(
                -shaped(tilts[CAMERA].x) * CAMERA_TURN.0 * dt,
                -shaped(tilts[CAMERA].y) * CAMERA_TURN.1 * dt,
            );
            if pinch != 1.0 {
                walk.zoom_by(f64::from(pinch));
            }
        }
        for &z in &reading.tapped_zones {
            match (z >= buttons_from, &gimbal) {
                (true, _) => self.drone_button(ui.ctx(), z - buttons_from),
                (false, Some(g)) => self.gimbal_tap(g, z),
                (false, None) => {}
            }
        }
        self.walk_taps(ui, rect, &reading.taps);
        self.walk_step(intent, dt, wheel);
        let busy = self.nav.sticks.active() || keyed || wheel != 0.0 || pinch != 1.0;
        self.nav.moving = self.nav.walk != before && busy;
        self.walk_trace(busy);
        if busy || self.nav.walk.is_some_and(|w| w.falling()) {
            ui.ctx().request_repaint();
        }
    }

    /// Each stick's tilt from its finger and its keys, clamped to the base.
    /// Solid, the left stick only turns: its knob keeps to the level so the
    /// stick shows that up and down is off.
    fn tilts(&self, held: &[Vec2], keys: [Vec2; 3]) -> [Vec2; 3] {
        let solid = self.nav.prefs.rules().solid;
        std::array::from_fn(|i| {
            let finger = held.get(i).copied().unwrap_or_default();
            let tilt = (finger + keys[i]).clamp(Vec2::splat(-1.0), Vec2::splat(1.0));
            if solid && i == LEFT {
                Vec2::new(tilt.x, 0.0)
            } else {
                tilt
            }
        })
    }

    /// The walking keys as stick tilts, while the keyboard is the viewer's
    /// (its own window, full screen, or the pointer over the picture) and no
    /// text box has it: whether any was held, the three tilts, and the wheel.
    /// W A S D are the left stick, the arrows or the keypad's 8 4 2 6 the
    /// right, I J K L the camera's; U and O tap the two centring buttons.
    /// Shift runs.
    fn walk_keys(&mut self, ui: &Ui, response: &Response) -> (bool, [Vec2; 3], f64) {
        let wheel = if response.hovered() {
            f64::from(ui.input(|i| i.smooth_scroll_delta.y))
        } else {
            0.0
        };
        let ours = response.hovered() || self.nav.immersive || self.native;
        if !ours || crate::canvas::textedit::composing(ui.ctx()) {
            return (false, [Vec2::ZERO; 3], wheel);
        }
        let (tilts, centre) = ui.input(|i| {
            let down = |ks: &[Key]| f32::from(u8::from(ks.iter().any(|k| i.key_down(*k))));
            let stick = |up: &[Key], left: &[Key], down_: &[Key], right: &[Key]| {
                Vec2::new(down(right) - down(left), down(up) - down(down_))
            };
            (
                [
                    stick(&[Key::W], &[Key::A], &[Key::S], &[Key::D]),
                    stick(
                        &[Key::ArrowUp, Key::Num8],
                        &[Key::ArrowLeft, Key::Num4],
                        &[Key::ArrowDown, Key::Num2],
                        &[Key::ArrowRight, Key::Num6],
                    ),
                    stick(&[Key::I], &[Key::J], &[Key::K], &[Key::L]),
                ],
                [i.key_pressed(Key::U), i.key_pressed(Key::O)],
            )
        });
        if centre[0] {
            self.drone_button(ui.ctx(), drone::CENTRE_YAW);
        }
        if centre[1] {
            self.drone_button(ui.ctx(), drone::CENTRE_PITCH);
        }
        let held = tilts.iter().any(|t| *t != Vec2::ZERO);
        (held || centre.contains(&true), tilts, wheel)
    }

    /// Step the walker, timing what it meets.
    fn walk_step(&mut self, intent: Intent, dt: f64, wheel: f64) {
        let Some(mut walk) = self.nav.walk else {
            return;
        };
        walk.height = self.eye_height();
        let rules = self.nav.prefs.rules();
        let started = std::time::Instant::now();
        let mut world = |ray: &pdfcer_3d::Ray| self.hit(ray);
        if wheel != 0.0 {
            walk.travel(
                [wheel / 50.0 * WHEEL_STEP * walk.body(), 0.0],
                rules,
                &mut world,
            );
        }
        walk.step(intent, dt, rules, &mut world);
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        self.nav.walk = Some(walk);
        if rules.solid && !intent.idle() {
            self.nav.walked.1 = self.nav.walked.1.max(ms);
        }
    }

    /// A drone button tapped: centre the camera left and right, or up and
    /// down, or switch stair climbing.
    fn drone_button(&mut self, ctx: &egui::Context, button: usize) {
        match (button, &mut self.nav.walk) {
            (drone::CENTRE_YAW, Some(walk)) => walk.centre_look(),
            (drone::CENTRE_PITCH, Some(walk)) => walk.level(),
            (drone::STAIRS, _) => self.toggle_stairs(ctx),
            _ => {}
        }
        let (look_yaw, pitch) = self.nav.walk.map_or((0.0, 0.0), |w| (w.look_yaw, w.pitch));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("model-drone-button n={button} look-yaw={look_yaw:.3} pitch={pitch:.3}")
        });
    }

    /// A double tap on the picture goes to the surface under it; a single one
    /// shows or hides the controls.
    fn walk_taps(&mut self, ui: &Ui, rect: Rect, taps: &[Pos2]) {
        let now = ui.input(|i| i.time);
        for &at in taps {
            if !self.tap_picture(now, at) {
                continue;
            }
            let Some(hit) = self.hit_at(rect, at) else {
                continue;
            };
            if let Some(walk) = &mut self.nav.walk {
                walk.go_to(&hit, walk.eye);
                let eye = walk.eye;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "model-walk-went eye={:.4},{:.4},{:.4}",
                        eye[0], eye[1], eye[2]
                    )
                });
            }
        }
    }

    /// Count this walk's frames while it moves, and trace it when it stops:
    /// where it ended and its slowest collision.
    fn walk_trace(&mut self, busy: bool) {
        if busy {
            self.nav.walked.0 += 1;
            return;
        }
        let (frames, slowest) = std::mem::take(&mut self.nav.walked);
        let Some(walk) = self.nav.walk.filter(|_| frames > 0) else {
            return;
        };
        self.nav.slow = (slowest > super::nav::SLOW_MS).then_some(slowest);
        let e = walk.eye;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-walk-moved eye={:.4},{:.4},{:.4} yaw={:.3} look-yaw={:.3} pitch={:.3} zoom={:.3} frames={frames} slowest-ms={slowest:.2}",
                e[0], e[1], e[2], walk.yaw, walk.look_yaw, walk.pitch, walk.zoom
            )
        });
    }
}
