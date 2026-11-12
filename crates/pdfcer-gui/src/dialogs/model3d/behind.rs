//! Walking seen from behind, as a game's third-person view: the camera hangs
//! behind and above the walker's head, looking where the walker looks, and a
//! figure stands where the walker is, turned to its heading.
//!
//! The walker itself is unchanged: its eye, collisions and steps are the
//! engine's `Walk`, and only the camera the picture is drawn from moves. A
//! wall or floor behind the head pulls the camera in, so it never draws from
//! inside the model.
//!
//! The figure is the viewer's overlay, like the gimbal and the measurements:
//! drawn over the picture with the painter, nearest faces last, and not in a
//! picture written out. Because it is drawn over the model, a part between
//! the camera and the figure does not hide it; the tooltip says so.

use egui::{Color32, Mesh, Pos2, Rect, Shape, Ui};
use pdfcer_3d::vec3::{add, dot, normalize, scale, sub, unit_cross};
use pdfcer_3d::{Camera, Ray};
use pdfcer_3d_nav::{Lens, Walk};

use super::ModelView;
use super::measure::{aspect_of, whole};

/// The figure's bounds on the picture, while it is drawn.
pub const REGION_FIGURE: &str = "model3d.figure"; // ui-text-exempt: trace region name, never displayed

/// The figure's height, a multiple of the eye height: eyes sit at 93% of a
/// standing person's height.
const HEIGHT_PER_EYE: f64 = 1.0 / 0.93;
/// How far behind the aim point the camera hangs, eye heights: far enough
/// that the walking lens's 50° field shows the figure head to feet.
const ARM: f64 = 3.0;
/// How far above the eye the camera aims, eye heights, so the figure stands
/// below the picture's centre and the way ahead is in view.
const LIFT: f64 = 0.25;
/// The nearest the camera comes to the aim point with a wall behind, eye
/// heights.
const NEAREST: f64 = 0.35;
/// The gap kept between the camera and a wall behind it, eye heights.
const CLEAR: f64 = 0.1;
/// Sides round each limb and the torso.
const SIDES: usize = 10;
/// The head's bands from crown to chin, and its sides.
const HEAD: (usize, usize) = (6, 12);

/// The figure's clothes and skin. Drawn over the model's own colours, not
/// the window, so not the theme's.
const SHIRT: Color32 = Color32::from_rgb(58, 108, 168); // NOT A THEME COLOUR: a figure in the model, not part of the window
const TROUSERS: Color32 = Color32::from_rgb(52, 56, 70); // NOT A THEME COLOUR: a figure in the model, not part of the window
const SKIN: Color32 = Color32::from_rgb(214, 172, 140); // NOT A THEME COLOUR: a figure in the model, not part of the window
const HAIR: Color32 = Color32::from_rgb(64, 46, 34); // NOT A THEME COLOUR: a figure in the model, not part of the window

/// A triangle of the figure, model coordinates, wound counter-clockwise
/// seen from outside, and its colour.
type Face = ([[f64; 3]; 3], Color32);

/// Where the figure stands and which way it faces: its feet, and unit
/// vectors right, forward and up.
#[derive(Debug, Clone, Copy)]
struct Stance {
    feet: [f64; 3],
    right: [f64; 3],
    forward: [f64; 3],
    up: [f64; 3],
    /// Its height, model units.
    height: f64,
}

impl Stance {
    fn of(walk: &Walk) -> Option<Self> {
        let up = walk.axes().up();
        let forward = walk.axes().look(walk.yaw, 0.0);
        let right = unit_cross(forward, up)?;
        let body = walk.body();
        Some(Self {
            feet: sub(walk.eye, scale(up, body)),
            right,
            forward,
            up,
            height: body * HEIGHT_PER_EYE,
        })
    }

    /// A point `x` right, `y` forward and `z` up of the feet, in heights.
    fn at(&self, x: f64, y: f64, z: f64) -> [f64; 3] {
        let h = self.height;
        let p = add(self.feet, scale(self.right, x * h));
        add(add(p, scale(self.forward, y * h)), scale(self.up, z * h))
    }
}

/// The point the camera aims through: above the walker's eye.
fn pivot(walk: &Walk) -> [f64; 3] {
    add(walk.eye, scale(walk.axes().up(), LIFT * walk.body()))
}

/// The camera behind `walk`, `arm` model units back from its aim point,
/// with the walking camera's lens, zoom and up.
pub(super) fn camera(walk: &Walk, arm: f64, aspect: f64, lens: &Lens) -> Camera {
    let mut camera = walk.camera(aspect, lens);
    let d = walk.direction();
    let pivot = pivot(walk);
    let eye = sub(pivot, scale(d, arm));
    if eye.iter().all(|c| c.is_finite()) {
        camera.eye = eye;
        camera.target = add(pivot, scale(d, 2.0 * walk.body()));
    }
    camera
}

/// How far back the camera hangs when `behind` is the distance to the
/// nearest surface behind the aim point, if any: the full arm, or short of
/// the surface, but never nearer than [`NEAREST`].
fn arm_before(body: f64, behind: Option<f64>) -> f64 {
    let full = ARM * body;
    match behind {
        Some(d) if d < full + CLEAR * body => (d - CLEAR * body).max(NEAREST * body),
        _ => full,
    }
}

/// An elliptical prism standing on `base` (in heights, as [`Stance::at`]),
/// `tall` high, half-widths `wide` across and `deep` front to back, capped.
fn prism(out: &mut Vec<Face>, s: &Stance, base: [f64; 3], tall: f64, half: [f64; 2], ink: Color32) {
    let ring = |z: f64| -> Vec<[f64; 3]> {
        (0..SIDES)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let a = std::f64::consts::TAU * i as f64 / SIDES as f64;
                s.at(base[0] + half[0] * a.cos(), base[1] + half[1] * a.sin(), z)
            })
            .collect()
    };
    let (low, high) = (ring(base[2]), ring(base[2] + tall));
    let (bottom, top) = (
        s.at(base[0], base[1], base[2]),
        s.at(base[0], base[1], base[2] + tall),
    );
    for i in 0..SIDES {
        let j = (i + 1) % SIDES;
        out.push(([low[i], low[j], high[j]], ink));
        out.push(([low[i], high[j], high[i]], ink));
        out.push(([top, high[i], high[j]], ink));
        out.push(([bottom, low[j], low[i]], ink));
    }
}

/// The head: a sphere of `radius` heights centred `z` heights up, hair over
/// its crown and back.
fn head(out: &mut Vec<Face>, s: &Stance, z: f64, radius: f64) {
    let (bands, sides) = HEAD;
    #[allow(clippy::cast_precision_loss)]
    let point = |band: usize, side: usize| {
        let polar = std::f64::consts::PI * band as f64 / bands as f64;
        let a = std::f64::consts::TAU * side as f64 / sides as f64;
        let (sp, cp) = polar.sin_cos();
        s.at(
            radius * sp * a.cos(),
            radius * sp * a.sin(),
            z + radius * cp,
        )
    };
    for band in 0..bands {
        for side in 0..sides {
            let next = (side + 1) % sides;
            let (a, b) = (point(band, side), point(band, next));
            let (c, d) = (point(band + 1, next), point(band + 1, side));
            // Hair over the crown, and down the back to the ears.
            #[allow(clippy::cast_precision_loss)]
            let facing = (std::f64::consts::TAU * (side as f64 + 0.5) / sides as f64).sin();
            let ink = if band < 2 || (band < 4 && facing < 0.0) {
                HAIR
            } else {
                SKIN
            };
            if band > 0 {
                out.push(([a, d, b], ink));
            }
            if band + 1 < bands {
                out.push(([b, d, c], ink));
            }
        }
    }
}

/// The figure for `walk`: legs, torso, arms and head, as triangles.
fn figure(walk: &Walk) -> Vec<Face> {
    let Some(s) = Stance::of(walk) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for x in [-0.055, 0.055] {
        prism(&mut out, &s, [x, 0.0, 0.0], 0.48, [0.05, 0.055], TROUSERS);
        prism(
            &mut out,
            &s,
            [x * 3.4, 0.0, 0.46],
            0.35,
            [0.035, 0.04],
            SHIRT,
        );
    }
    prism(&mut out, &s, [0.0, 0.0, 0.47], 0.36, [0.15, 0.085], SHIRT);
    prism(&mut out, &s, [0.0, 0.0, 0.82], 0.05, [0.035, 0.035], SKIN);
    head(&mut out, &s, 0.925, 0.072);
    out
}

/// `face`'s unit normal, from its winding.
fn normal(face: &[[f64; 3]; 3]) -> Option<[f64; 3]> {
    unit_cross(sub(face[1], face[0]), sub(face[2], face[0]))
}

/// The figure's faces turned to `camera`, far first, each shaded and placed
/// on a picture `size` pixels from `origin`; and the bounds they cover.
fn painted(faces: &[Face], camera: &Camera, origin: Pos2, size: [u32; 2]) -> (Mesh, Rect) {
    let view = normalize(sub(camera.target, camera.eye)).unwrap_or([0.0, 0.0, 1.0]);
    let light = normalize(add(scale(camera.up, 0.8), scale(view, -0.6))).unwrap_or(camera.up);
    let mut seen: Vec<(f64, [Pos2; 3], Color32)> = Vec::new();
    for (face, ink) in faces {
        let Some(n) = normal(face) else { continue };
        let centre = scale(add(add(face[0], face[1]), face[2]), 1.0 / 3.0);
        let toward = sub(centre, camera.eye);
        if dot(n, toward) >= 0.0 {
            continue;
        }
        let placed: Option<Vec<Pos2>> = face
            .iter()
            .map(|p| {
                let [x, y] = camera.pixel_of(*p, size[0], size[1])?;
                #[allow(clippy::cast_possible_truncation)]
                Some(origin + egui::vec2(x as f32, y as f32))
            })
            .collect();
        let Some(placed) = placed else { continue };
        #[allow(clippy::cast_possible_truncation)]
        let lit = (0.4 + 0.6 * dot(n, light).max(0.0)) as f32;
        seen.push((
            dot(toward, toward),
            [placed[0], placed[1], placed[2]],
            ink.gamma_multiply(lit).to_opaque(),
        ));
    }
    seen.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut mesh = Mesh::default();
    let mut bounds = Rect::NOTHING;
    for (_, corners, ink) in seen {
        let first = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
        for p in corners {
            mesh.colored_vertex(p, ink);
            bounds.extend_with(p);
        }
        mesh.add_triangle(first, first + 1, first + 2);
    }
    (mesh, bounds)
}

impl ModelView {
    /// The camera behind the walker, where walking is seen from behind.
    pub(super) fn behind_camera(&self, walk: &Walk, aspect: f64, lens: &Lens) -> Camera {
        let arm = self
            .nav
            .arm
            .filter(|(w, _)| w == walk)
            .map_or(ARM * walk.body(), |(_, arm)| arm);
        camera(walk, arm, aspect, lens)
    }

    /// Measure how far back the camera can hang for where the walker now
    /// is, once per place: a wall or floor behind pulls it in.
    pub(super) fn update_arm(&mut self) {
        let Some(walk) = self.nav.walk.filter(|_| self.nav.prefs.behind) else {
            return;
        };
        if self.nav.arm.is_some_and(|(w, _)| w == walk) {
            return;
        }
        let ray = Ray {
            origin: pivot(&walk),
            direction: scale(walk.direction(), -1.0),
            min_distance: 0.0,
        };
        let behind = self.hit(&ray).map(|h| h.distance);
        let arm = arm_before(walk.body(), behind);
        self.nav.arm = Some((walk, arm));
        crate::diag::trace_changed("model-behind-arm", || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("model-behind-arm eye-heights={:.2}", arm / walk.body())
        });
    }

    /// The figure over the picture `rect`, while walking is seen from
    /// behind.
    pub(super) fn paint_figure(&self, ui: &Ui, rect: Rect) {
        let Some(walk) = self.nav.walk.filter(|_| self.nav.prefs.behind) else {
            return;
        };
        let camera = self.behind_camera(&walk, aspect_of(rect), &self.lens);
        let (mesh, bounds) = painted(&figure(&walk), &camera, rect.min, whole(rect));
        if mesh.indices.is_empty() {
            return;
        }
        let triangles = mesh.indices.len() / 3;
        ui.painter().with_clip_rect(rect).add(Shape::mesh(mesh));
        crate::diag::ui_rect_visible(REGION_FIGURE, bounds, rect);
        crate::diag::trace_changed("model-figure", || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("model-figure triangles={triangles}")
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_3d_nav::Axes;

    fn walker() -> Walk {
        let axes = Axes::from_look([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]).unwrap();
        let mut walk = Walk::standing([0.0, 0.0, 1.6], [1.0, 0.0, 0.0], axes, 10.0);
        walk.height = Some(1.6);
        walk
    }

    #[test]
    fn the_camera_hangs_behind_and_above_the_head_looking_the_walkers_way() {
        let walk = walker();
        let c = camera(&walk, ARM * 1.6, 1.5, &Lens::default());
        let d = walk.direction();
        assert!(dot(sub(c.eye, walk.eye), d) < -2.0, "{:?}", c.eye);
        let view = normalize(sub(c.target, c.eye)).unwrap();
        assert!(dot(view, d) > 0.999, "{view:?} {d:?}");
        assert!(dot(sub(c.eye, walk.eye), walk.axes().up()) > 0.0);
    }

    #[test]
    fn a_wall_behind_pulls_the_camera_in_but_not_into_the_head() {
        assert!((arm_before(1.6, None) - ARM * 1.6).abs() < 1e-9);
        assert!((arm_before(1.6, Some(10.0)) - ARM * 1.6).abs() < 1e-9);
        assert!((arm_before(1.6, Some(1.6)) - 1.6 * (1.0 - CLEAR)).abs() < 1e-9);
        assert!((arm_before(1.6, Some(0.1)) - NEAREST * 1.6).abs() < 1e-9);
    }

    #[test]
    fn the_figure_stands_on_the_floor_under_the_eye_and_reaches_its_height() {
        let walk = walker();
        let faces = figure(&walk);
        let up = walk.axes().up();
        let heights: Vec<f64> = faces
            .iter()
            .flat_map(|(f, _)| f.map(|p| dot(p, up)))
            .collect();
        let low = heights.iter().copied().fold(f64::INFINITY, f64::min);
        let high = heights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(low.abs() < 1e-9, "{low}");
        assert!((high - 1.6 * HEIGHT_PER_EYE).abs() < 0.01, "{high}");
    }

    #[test]
    fn the_figure_is_drawn_whole_below_the_centre_of_the_picture() {
        let walk = walker();
        let c = camera(&walk, ARM * 1.6, 1.5, &Lens::default());
        let (mesh, bounds) = painted(&figure(&walk), &c, Pos2::ZERO, [600, 400]);
        assert!(!mesh.indices.is_empty());
        assert!((bounds.center().x - 300.0).abs() < 5.0, "{bounds:?}");
        assert!(bounds.top() > 200.0, "{bounds:?}");
        assert!(
            bounds.bottom() < 400.0,
            "the feet are off the picture: {bounds:?}"
        );
    }
}
