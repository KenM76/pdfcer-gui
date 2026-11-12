//! The figure's walk: a gait that advances with the ground the walker
//! covers, and the pose it gives each part, turned about its joint.
//!
//! The phase advances by distance, not time, so the feet keep pace with the
//! ground at any speed and step backwards walking backwards. The swing
//! eases in as walking starts and out as it stops. After posing, the figure
//! is lowered or raised so its lowest corner is on the floor: the foot
//! bearing the weight is planted and the hips rise and fall with the legs.

use super::figure::{Figure, Role, Side};

/// One full stride — two steps — as a share of the figure's height.
const STRIDE: f64 = 0.8;
/// How fast the swing eases in and out, per second.
const EASE: f64 = 5.0;
/// A walker covering less than this per second, metres, is standing.
const STILL: f64 = 0.05;
/// A jump in one frame further than this, metres, is a move, not a step.
const JUMP: f64 = 3.0;

/// The gait between frames.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Gait {
    /// Radians round the stride.
    pub phase: f64,
    /// How far into its swing, 0 standing to 1 walking.
    pub swing: f64,
    /// Walking speed now, metres a second.
    pub speed: f64,
}

impl Gait {
    /// Advance by `ahead` metres walked forward (negative backward) and
    /// `across` sideways, over `dt` seconds, for a figure `height` metres
    /// tall. `true` while the swing is still easing.
    pub fn step(&mut self, ahead: f64, across: f64, dt: f64, height: f64) -> bool {
        let moved = ahead.hypot(across);
        if moved > JUMP || dt <= 0.0 {
            return false;
        }
        self.speed = moved / dt;
        let walking = self.speed > STILL;
        if walking {
            let sign = if ahead < -0.25 * moved { -1.0 } else { 1.0 };
            self.phase = (self.phase
                + sign * moved / (STRIDE * height.max(0.1)) * std::f64::consts::TAU)
                .rem_euclid(std::f64::consts::TAU);
        }
        let target = if walking { 1.0 } else { 0.0 };
        let eased = self.swing + (target - self.swing) * (EASE * dt).min(1.0);
        self.swing = if (eased - target).abs() < 1e-3 {
            target
        } else {
            eased
        };
        (self.swing - target).abs() > 0.0
    }

    /// How wide the swing is: wider running.
    fn reach(&self) -> f64 {
        self.swing * (0.8 + 0.25 * (self.speed / 1.4).clamp(0.0, 2.0))
    }

    /// `role`'s turn about its joint: forward pitch (about x, a hanging limb
    /// swinging forward) and twist (about z), radians.
    fn turn(&self, role: Role) -> (f64, f64) {
        let a = self.reach();
        let leg = |side: Side| match side {
            Side::Left => self.phase,
            Side::Right => self.phase + std::f64::consts::PI,
        };
        match role {
            Role::Thigh(s) => (0.5 * a * leg(s).sin(), 0.0),
            Role::Shin(s) => (-a * (0.1 + 0.9 * leg(s).cos().max(0.0)), 0.0),
            Role::Foot(s) => {
                let shin = 0.5 * leg(s).sin() - (0.1 + 0.9 * leg(s).cos().max(0.0));
                (-0.7 * a * shin, 0.0)
            }
            Role::UpperArm(s) => (-0.4 * a * leg(s).sin(), 0.0),
            Role::ForeArm(_) => (0.15 + 0.3 * a, 0.0),
            Role::Torso => (-0.08 * a, 0.08 * a * self.phase.sin()),
            Role::Hips => (0.0, -0.06 * a * self.phase.sin()),
            Role::Head => (0.06 * a, -0.05 * a * self.phase.sin()),
            Role::Tail => (0.05 * a, 0.35 * a * self.phase.sin()),
            Role::Hand(_) | Role::Other => (0.0, 0.0),
        }
    }
}

type Matrix = [[f64; 3]; 3];

fn turned(pitch: f64, twist: f64) -> Matrix {
    let (sp, cp) = pitch.sin_cos();
    let (st, ct) = twist.sin_cos();
    // Rz(twist) · Rx(pitch)
    [
        [ct, -st * cp, st * sp],
        [st, ct * cp, -ct * sp],
        [0.0, sp, cp],
    ]
}

fn times(a: &Matrix, b: &Matrix) -> Matrix {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| a[i][k] * b[k][j]).sum()))
}

fn apply(m: &Matrix, v: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}

/// A triangle of the posed figure, metres in the figure's frame.
pub(super) type Posed = ([[f64; 3]; 3], egui::Color32);

/// `figure` posed by `gait`, its lowest corner on z = 0.
pub(super) fn posed(figure: &Figure, gait: &Gait) -> Vec<Posed> {
    let mut frames: Vec<(Matrix, [f64; 3])> = Vec::with_capacity(figure.parts.len());
    let mut out = Vec::new();
    for part in &figure.parts {
        let (pitch, twist) = gait.turn(part.role);
        let local = turned(pitch, twist);
        let frame = match part.parent {
            Some(up) => {
                let (r, at) = frames[up];
                let offset = std::array::from_fn(|a| part.pivot[a] - figure.parts[up].pivot[a]);
                let joint = apply(&r, offset);
                (times(&r, &local), std::array::from_fn(|a| at[a] + joint[a]))
            }
            None => (local, part.pivot),
        };
        frames.push(frame);
        let corner = |i: usize| {
            let v = apply(&frame.0, part.verts[i]);
            std::array::from_fn(|a| frame.1[a] + v[a])
        };
        out.extend(
            part.faces
                .iter()
                .map(|(f, ink)| ([corner(f[0]), corner(f[1]), corner(f[2])], *ink)),
        );
    }
    let low = out
        .iter()
        .flat_map(|(f, _)| f.iter().map(|p| p[2]))
        .fold(f64::INFINITY, f64::min);
    if low.is_finite() {
        for (f, _) in &mut out {
            for p in f {
                p[2] -= low;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::figure::{Kind, Measures, builtin, fitted};
    use super::*;

    fn lowest_foot(posed: &[Posed]) -> f64 {
        posed
            .iter()
            .flat_map(|(f, _)| f.iter().map(|p| p[2]))
            .fold(f64::INFINITY, f64::min)
    }

    #[test]
    fn standing_the_pose_is_as_made() {
        let fig = fitted(builtin(Kind::Human), &Measures::default());
        let still = posed(&fig, &Gait::default());
        let triangles: usize = fig.parts.iter().map(|p| p.faces.len()).sum();
        assert_eq!(still.len(), triangles);
        assert!(lowest_foot(&still).abs() < 1e-9);
    }

    #[test]
    fn walking_swings_the_legs_in_opposition_and_keeps_a_foot_down() {
        let mut gait = Gait::default();
        for _ in 0..60 {
            gait.step(0.02, 0.0, 1.0 / 60.0, 1.8);
        }
        assert!(gait.swing > 0.9, "{gait:?}");
        let (l, _) = gait.turn(Role::Thigh(Side::Left));
        let (r, _) = gait.turn(Role::Thigh(Side::Right));
        assert!(l * r <= 0.0, "{l} {r}");
        let fig = fitted(builtin(Kind::Werewolf), &Measures::default());
        assert!(lowest_foot(&posed(&fig, &gait)).abs() < 1e-9);
    }

    #[test]
    fn the_stride_keeps_pace_with_the_ground_and_reverses_backwards() {
        let mut gait = Gait::default();
        gait.step(0.4, 0.0, 0.1, 1.0);
        let ahead = gait.phase;
        assert!((ahead - 0.4 / STRIDE * std::f64::consts::TAU).abs() < 1e-9);
        gait.step(-0.4, 0.0, 0.1, 1.0);
        assert!(gait.phase.abs() < 1e-9 || (gait.phase - std::f64::consts::TAU).abs() < 1e-9);
    }

    #[test]
    fn stopping_eases_the_swing_out_then_settles() {
        let mut gait = Gait {
            swing: 1.0,
            ..Gait::default()
        };
        let mut frames = 0;
        while gait.step(0.0, 0.0, 1.0 / 60.0, 1.8) {
            frames += 1;
            assert!(frames < 200);
        }
        assert!(gait.swing == 0.0 && frames > 10, "{frames}");
    }
}
