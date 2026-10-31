//! Walking through the model: a first-person camera at a body's eye height.
//!
//! Solid (ghost mode off), the walker stops at what it meets, slides along
//! it, climbs steps up to [`STEP`] of its height, and falls onto the floor
//! under it; with nothing under it, it keeps its height. Ghosting, it flies
//! through everything. Moving is horizontal when solid or level, so it
//! never drifts up or down; a ghost that is not level flies where it looks.
//!
//! Scale: every distance is a share of the model's radius, so a building
//! and a bracket walk alike. Model units throughout.

use pdfcer_3d::{Camera, Hit, Projection, Ray};

use super::orbit::{Axes, PITCH_LIMIT, cross, dot, unit};

/// The walking field of view, degrees: a game's, wider than the orbit's.
const FOV: f64 = 60.0;
/// Eye height, as a share of the model's radius.
const BODY_SHARE: f64 = 0.08;
/// Walking speed at speed 1, in model radii per second.
const PACE: f64 = 0.25;
/// Running speed, as a multiple of walking.
const RUN: f64 = 3.0;
/// The highest step climbed without rising, as a share of eye height.
const STEP: f64 = 0.35;
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
}

/// The walker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Walk {
    pub eye: [f64; 3],
    /// Radians round the up axis, as [`Axes::look`] measures it.
    pub yaw: f64,
    /// Radians below the horizon.
    pub pitch: f64,
    axes: Axes,
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
        let (yaw, pitch) = axes.angles(look).unwrap_or((0.0, 0.0));
        Self {
            eye: camera.eye,
            yaw,
            pitch,
            axes,
            scale: radius,
            fall: 0.0,
        }
    }

    /// The eye's height above a floor, model units.
    pub fn body(&self) -> f64 {
        self.scale * BODY_SHARE
    }

    /// Whether it is still falling, so the picture must keep redrawing.
    pub fn falling(&self) -> bool {
        self.fall != 0.0
    }

    pub fn direction(&self) -> [f64; 3] {
        self.axes.look(self.yaw, self.pitch)
    }

    pub fn camera(&self) -> Camera {
        let d = self.direction();
        let ahead = self.body() * 4.0;
        Camera {
            eye: self.eye,
            target: std::array::from_fn(|i| self.eye[i] + d[i] * ahead),
            up: self.axes.up,
            projection: Projection::Perspective { fov_y: FOV },
        }
    }

    /// Turn by `yaw` and `pitch` radians; pitch stops short of straight up
    /// or down.
    pub fn turn(&mut self, yaw: f64, pitch: f64) {
        self.yaw += yaw;
        self.pitch = (self.pitch + pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Look along `direction`.
    pub fn face(&mut self, direction: [f64; 3]) {
        if let Some((yaw, pitch)) = self.axes.angles(direction) {
            (self.yaw, self.pitch) = (yaw, pitch);
        }
    }

    /// Move for `dt` seconds of `intent`. `world` is the model's nearest hit
    /// along a ray; it is asked only when solid.
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
        let rise = intent.rise * pace * dt;
        if rules.solid {
            self.settle(rise, dt, world);
        } else {
            self.eye = along(self.eye, self.axes.up, rise);
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
            self.direction()
        };
        let side = cross(flat, up);
        let mut motion: [f64; 3] = std::array::from_fn(|i| ahead[i] * forward + side[i] * right);
        if rules.solid {
            motion = self.slide(motion, world);
        }
        self.eye = std::array::from_fn(|i| self.eye[i] + motion[i]);
    }

    /// `motion` cut where it would bring the eye or the top of a step within
    /// reach of a wall: the part into the wall dropped, then stopped
    /// outright if the rest meets one too.
    fn slide(&self, motion: [f64; 3], world: &mut impl FnMut(&Ray) -> Option<Hit>) -> [f64; 3] {
        let Some(wall) = self.blocked(motion, world) else {
            return motion;
        };
        let into = dot(motion, wall);
        let along: [f64; 3] = std::array::from_fn(|i| motion[i] - wall[i] * into);
        if self.blocked(along, world).is_some() {
            [0.0; 3]
        } else {
            along
        }
    }

    /// The horizontal unit normal, facing the walker, of the nearest wall
    /// `motion` comes within reach of, at eye height or a step above the
    /// floor; `None` when the way is clear.
    fn blocked(
        &self,
        motion: [f64; 3],
        world: &mut impl FnMut(&Ray) -> Option<Hit>,
    ) -> Option<[f64; 3]> {
        let length = dot(motion, motion).sqrt();
        let direction = unit(motion)?;
        let reach = self.body() * REACH;
        let knee = along(self.eye, self.axes.up, -self.body() * (1.0 - STEP));
        [self.eye, knee]
            .into_iter()
            .filter_map(|origin| {
                world(&Ray {
                    origin,
                    direction,
                    min_distance: 0.0,
                })
            })
            .filter(|hit| hit.distance < length + reach)
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
            .map(|hit| {
                let up = self.axes.up;
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
    /// stepping up onto it from below; rise and sink by `rise`, a rise
    /// stopping under a ceiling. With no floor under it the walker keeps its
    /// height.
    fn settle(&mut self, rise: f64, dt: f64, world: &mut impl FnMut(&Ray) -> Option<Hit>) {
        let up = self.axes.up;
        let reach = self.body() * REACH;
        if rise > 0.0 {
            let ceiling = world(&Ray {
                origin: self.eye,
                direction: up,
                min_distance: 0.0,
            })
            .map_or(f64::INFINITY, |h| (h.distance - reach).max(0.0));
            self.eye = along(self.eye, up, rise.min(ceiling));
            self.fall = 0.0;
            return;
        }
        let floor = world(&Ray {
            origin: self.eye,
            direction: up.map(|c| -c),
            min_distance: 0.0,
        });
        let Some(floor) = floor else {
            self.eye = along(self.eye, up, rise);
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
        let drop = (self.fall * dt - rise).min(above);
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

fn along(point: [f64; 3], direction: [f64; 3], distance: f64) -> [f64; 3] {
    std::array::from_fn(|i| point[i] + direction[i] * distance)
}

#[cfg(test)]
#[path = "walk_tests.rs"]
mod tests;
