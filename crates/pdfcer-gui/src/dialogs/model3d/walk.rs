//! Walking through the model: a first-person camera at a body's eye height.
//!
//! Solid (ghost mode off), the walker stops at what it meets, slides along
//! it, and falls onto the floor under it; with nothing under it, it keeps its
//! height. It cannot rise or sink by itself: stair climbing, when on, is the
//! only way up, over anything lower than [`CLIMB`] of its eye height; off,
//! only a floor seam under [`BUMP`]. Ghosting, it flies through everything
//! and rises and sinks as asked. Moving is horizontal when solid or level, so it
//! never drifts up or down; a ghost that is not level flies where it looks.
//!
//! The body and the camera turn apart, as a drone and its gimbal do: `yaw`
//! is the body's heading, which moving follows, and `look_yaw` the camera's
//! turn from it. `pitch` is the camera's alone; the body never tilts.
//!
//! Scale: every distance is a share of the model's radius, so a building
//! and a bracket walk alike, except the eye height where the operator set
//! one ([`Walk::height`]). Model units throughout.

use pdfcer_3d::{Camera, Hit, Ray};

use super::lens::Lens;
use super::orbit::{Axes, PITCH_LIMIT, cross, dot, unit};

/// The walking field of view, degrees, vertically: wider than the orbit's,
/// narrower than a game's so a wide window's edges stay unstretched.
const FOV: f64 = 50.0;
/// The widest aspect the vertical field holds for. A wider window keeps a
/// 16:9 window's sideways field and narrows the vertical one instead, so
/// stretching the window never stretches the picture's edges.
const WIDEST: f64 = 16.0 / 9.0;
/// Eye height, as a share of the model's radius.
const BODY_SHARE: f64 = 0.08;
/// Walking speed at speed 1, in model radii per second.
const PACE: f64 = 0.25;
/// Running speed, as a multiple of walking.
const RUN: f64 = 3.0;
/// The highest obstacle climbed with stair climbing on, as a share of eye
/// height: the operator's "a third of the camera's height".
pub(super) const CLIMB: f64 = 1.0 / 3.0;
/// The highest step climbed with it off: a seam or a threshold, never an
/// object.
const BUMP: f64 = 0.05;
/// The widest vertical field zooming out reaches through a flat lens,
/// degrees: about 130 degrees sideways at 16:9. A flat (rectilinear)
/// picture this wide stretches its edges by design; the operator chooses
/// it, the default stays at [`FOV`]. A fisheye reaches [`Lens::widest`].
#[cfg(test)]
const WIDEST_FOV: f64 = super::lens::FLAT_WIDEST;
/// The camera's narrowest zoom-in, as a multiple of [`FOV`]'s view; the
/// widest zoom-out is the lens's.
const ZOOM_IN: f64 = 6.0;
/// A hit whose normal is this close to the up axis is a floor or a ramp,
/// stood on rather than walked into.
const WALKABLE: f64 = 0.7;
/// How close a wall stops the eye, as a share of eye height.
const REACH: f64 = 0.25;
/// Gravity, in eye heights per second squared: about Earth's for a
/// 1.6 m eye.
const GRAVITY: f64 = 6.0;

/// What the operator is asking for this frame. `forward` and `right` run
/// from -1 to 1 (a stick's tilt, or a held key's 1); `rise` likewise.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct Intent {
    pub forward: f64,
    pub right: f64,
    pub rise: f64,
    pub run: bool,
}

impl Intent {
    pub fn idle(&self) -> bool {
        self.forward == 0.0 && self.right == 0.0 && self.rise == 0.0
    }
}

/// How walking treats the model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Rules {
    /// Walls stop and floors hold the walker (ghost mode off).
    pub solid: bool,
    /// Move horizontally whatever the look's pitch.
    pub level: bool,
    /// A multiple of the walking pace.
    pub speed: f64,
    /// Climb obstacles lower than [`CLIMB`] of the eye height.
    pub stairs: bool,
}

impl Rules {
    /// The highest step climbed, as a share of eye height.
    fn step(self) -> f64 {
        if self.stairs { CLIMB } else { BUMP }
    }
}

/// The walker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Walk {
    pub eye: [f64; 3],
    /// The body's heading: radians round the up axis, as [`Axes::look`]
    /// measures it.
    pub yaw: f64,
    /// The camera's turn from the heading, radians, the same way.
    pub look_yaw: f64,
    /// The camera's radians below the horizon.
    pub pitch: f64,
    /// The camera's zoom, a multiple of the [`FOV`] view.
    pub zoom: f64,
    axes: Axes,
    /// The eye height the operator set, model units; `None` for the
    /// radius's share.
    pub height: Option<f64>,
    /// The model's radius, which every distance is a share of.
    scale: f64,
    /// Falling speed, model units per second; 0 on a floor.
    fall: f64,
}

impl Walk {
    /// A walker standing where `camera` is, looking where it looks, turning
    /// about `axes.up`.
    pub fn from_camera(camera: &Camera, axes: Axes, radius: f64) -> Self {
        let look: [f64; 3] = std::array::from_fn(|i| camera.target[i] - camera.eye[i]);
        Self::standing(camera.eye, look, axes, radius)
    }

    /// A walker standing at `eye`, looking along `look`, turning about
    /// `axes.up`; `radius` is the model's.
    pub fn standing(eye: [f64; 3], look: [f64; 3], axes: Axes, radius: f64) -> Self {
        let (yaw, pitch) = axes.angles(look).unwrap_or((0.0, 0.0));
        Self {
            eye,
            yaw,
            look_yaw: 0.0,
            pitch,
            zoom: 1.0,
            axes,
            height: None,
            scale: radius,
            fall: 0.0,
        }
    }

    /// The eye's height above a floor, model units.
    pub fn body(&self) -> f64 {
        self.height
            .filter(|h| h.is_finite() && *h > 0.0)
            .unwrap_or(self.scale * BODY_SHARE)
    }

    /// Whether it is still falling, so the picture must keep redrawing.
    pub fn falling(&self) -> bool {
        self.fall != 0.0
    }

    /// Where the camera looks.
    pub fn direction(&self) -> [f64; 3] {
        self.axes.look(self.yaw + self.look_yaw, self.pitch)
    }

    /// The camera for a picture of `aspect`, width / height, through `lens`.
    /// The field is the zoom's, no wider than the lens reaches.
    pub fn camera(&self, aspect: f64, lens: &Lens) -> Camera {
        let d = self.direction();
        let ahead = self.body() * 4.0;
        let natural = (FOV / self.zoom).min(lens.widest());
        let fisheye = lens.fisheye_at(natural);
        Camera {
            eye: self.eye,
            target: std::array::from_fn(|i| self.eye[i] + d[i] * ahead),
            up: self.axes.up,
            projection: lens.projection(fov_y(natural, aspect, fisheye)),
        }
    }

    /// Turn the body by `yaw` and the camera's pitch by `pitch` radians;
    /// pitch stops short of straight up or down.
    pub fn turn(&mut self, yaw: f64, pitch: f64) {
        self.yaw += yaw;
        self.tilt(pitch);
    }

    /// Turn the camera alone by `yaw` and `pitch` radians, the heading kept.
    pub fn look(&mut self, yaw: f64, pitch: f64) {
        self.look_yaw = wrap(self.look_yaw + yaw);
        self.tilt(pitch);
    }

    fn tilt(&mut self, pitch: f64) {
        self.pitch = (self.pitch + pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Turn the camera back to the heading.
    pub fn centre_look(&mut self) {
        self.look_yaw = 0.0;
    }

    /// Multiply the camera's zoom by `factor`, out no wider than `lens`
    /// reaches and in no further than [`ZOOM_IN`].
    pub fn zoom_by(&mut self, factor: f64, lens: &Lens) {
        self.zoom = (self.zoom * factor).clamp(FOV / lens.widest(), ZOOM_IN);
    }

    /// Look and head along `direction`.
    pub fn face(&mut self, direction: [f64; 3]) {
        if let Some((yaw, pitch)) = self.axes.angles(direction) {
            (self.yaw, self.look_yaw, self.pitch) = (yaw, 0.0, pitch);
        }
    }

    /// Move for `dt` seconds of `intent`. `world` is the model's nearest hit
    /// along a ray; it is asked only when solid. A solid walker ignores
    /// `intent.rise`, so it can never lift itself over what it would meet.
    pub fn step(
        &mut self,
        intent: Intent,
        dt: f64,
        rules: Rules,
        world: &mut impl FnMut(&Ray) -> Option<Hit>,
    ) {
        let pace = self.scale * PACE * rules.speed * if intent.run { RUN } else { 1.0 };
        self.travel(
            [intent.forward * pace * dt, intent.right * pace * dt],
            rules,
            world,
        );
        if rules.solid {
            self.settle(dt, world);
        } else {
            self.eye = along(self.eye, self.axes.up, intent.rise * pace * dt);
        }
    }

    /// Move `distance` forward and right, sliding along any wall met.
    pub fn travel(
        &mut self,
        [forward, right]: [f64; 2],
        rules: Rules,
        world: &mut impl FnMut(&Ray) -> Option<Hit>,
    ) {
        let up = self.axes.up;
        let flat = self.axes.look(self.yaw, 0.0);
        let ahead = if rules.solid || rules.level {
            flat
        } else {
            self.axes.look(self.yaw, self.pitch)
        };
        let side = cross(flat, up);
        let mut motion: [f64; 3] = std::array::from_fn(|i| ahead[i] * forward + side[i] * right);
        if rules.solid {
            motion = self.slide(motion, rules.step(), world);
            let to = std::array::from_fn(|i| self.eye[i] + motion[i]);
            if self.lifts(to, rules.step(), world) {
                motion = [0.0; 3];
            }
        }
        self.eye = std::array::from_fn(|i| self.eye[i] + motion[i]);
    }

    /// Whether standing at `to` would raise the eye by more than `step` of
    /// its height, and above where standing here puts it. The wall rays see
    /// only two heights from the eye's line, so a slab between them, a
    /// rounded front or a wall behind a ramp's lip passes them; the floor
    /// under the destination does not.
    fn lifts(&self, to: [f64; 3], step: f64, world: &mut impl FnMut(&Ray) -> Option<Hit>) -> bool {
        let mut down = |origin: [f64; 3]| {
            world(&Ray {
                origin,
                direction: self.axes.up.map(|c| -c),
                min_distance: 0.0,
            })
            .map(|hit| hit.distance)
        };
        let here = down(self.eye);
        let Some(there) = down(to) else {
            return false;
        };
        let rise = self.body() - there;
        rise > step * self.body() + 1e-9 && here.is_none_or(|h| there < h - 1e-9)
    }

    /// `motion` cut where it would bring the eye or the top of a step within
    /// reach of a wall: the part into the wall dropped, then stopped
    /// outright if the rest meets one too.
    fn slide(
        &self,
        motion: [f64; 3],
        step: f64,
        world: &mut impl FnMut(&Ray) -> Option<Hit>,
    ) -> [f64; 3] {
        let Some(wall) = self.blocked(motion, step, world) else {
            return motion;
        };
        let into = dot(motion, wall);
        let along: [f64; 3] = std::array::from_fn(|i| motion[i] - wall[i] * into);
        if self.blocked(along, step, world).is_some() {
            [0.0; 3]
        } else {
            along
        }
    }

    /// The horizontal unit normal, facing the walker, of the nearest wall
    /// `motion` comes within reach of, at eye height or `step` of it above
    /// the floor; `None` when the way is clear. A floor or ramp met is no
    /// wall: [`Walk::settle`] stands on it.
    fn blocked(
        &self,
        motion: [f64; 3],
        step: f64,
        world: &mut impl FnMut(&Ray) -> Option<Hit>,
    ) -> Option<[f64; 3]> {
        let length = dot(motion, motion).sqrt();
        let direction = unit(motion)?;
        let reach = self.body() * REACH;
        let up = self.axes.up;
        let knee = along(self.eye, up, -self.body() * (1.0 - step));
        [self.eye, knee]
            .into_iter()
            .filter_map(|origin| {
                world(&Ray {
                    origin,
                    direction,
                    min_distance: 0.0,
                })
            })
            .filter(|hit| hit.distance < length + reach && dot(hit.normal, up).abs() < WALKABLE)
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
            .map(|hit| {
                let lift = dot(hit.normal, up);
                let flat: [f64; 3] = std::array::from_fn(|i| hit.normal[i] - up[i] * lift);
                let n = unit(flat).unwrap_or(direction.map(|c| -c));
                if dot(n, direction) > 0.0 {
                    n.map(|c| -c)
                } else {
                    n
                }
            })
    }

    /// Stand on the floor under the eye, falling onto it from above it and
    /// stepping up onto it from below. With no floor under it the walker
    /// keeps its height.
    fn settle(&mut self, dt: f64, world: &mut impl FnMut(&Ray) -> Option<Hit>) {
        let up = self.axes.up;
        let floor = world(&Ray {
            origin: self.eye,
            direction: up.map(|c| -c),
            min_distance: 0.0,
        });
        let Some(floor) = floor else {
            self.fall = 0.0;
            return;
        };
        let above = floor.distance - self.body();
        if above <= 0.0 {
            self.eye = along(self.eye, up, -above);
            self.fall = 0.0;
            return;
        }
        self.fall += GRAVITY * self.body() * dt;
        let drop = (self.fall * dt).min(above);
        self.eye = along(self.eye, up, -drop);
        if drop >= above {
            self.fall = 0.0;
        }
    }

    /// Go to `hit`, the surface under a tapped point: standing on it when it
    /// faces up, else a step back from it.
    pub fn go_to(&mut self, hit: &Hit, from: [f64; 3]) {
        let up = self.axes.up;
        let back = unit(std::array::from_fn(|i| from[i] - hit.point[i])).unwrap_or(up);
        let facing = if dot(hit.normal, back) < 0.0 {
            hit.normal.map(|c| -c)
        } else {
            hit.normal
        };
        self.eye = if dot(facing, up) > 0.7 {
            along(hit.point, up, self.body())
        } else {
            along(hit.point, back, self.body())
        };
        self.fall = 0.0;
    }

    /// Level the look.
    pub fn level(&mut self) {
        self.pitch = 0.0;
    }
}

/// `angle` in [-π, π).
fn wrap(angle: f64) -> f64 {
    let turn = std::f64::consts::TAU;
    angle - turn * ((angle + std::f64::consts::PI) / turn).floor()
}

fn along(point: [f64; 3], direction: [f64; 3], distance: f64) -> [f64; 3] {
    std::array::from_fn(|i| point[i] + direction[i] * distance)
}

#[cfg(test)]
#[path = "walk_tests.rs"]
mod tests;

/// `natural`'s vertical field, narrowed past [`WIDEST`] so the sideways
/// field stops growing with the window's width. A fisheye's sideways field
/// is its vertical one times the aspect, so it narrows in proportion; a
/// flat one's tangent does.
fn fov_y(natural: f64, aspect: f64, fisheye: bool) -> f64 {
    if aspect <= WIDEST {
        return natural;
    }
    if fisheye {
        return natural * WIDEST / aspect;
    }
    let half = (natural.to_radians() / 2.0).tan() * WIDEST / aspect;
    2.0 * half.atan().to_degrees()
}
