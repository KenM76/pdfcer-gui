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
//!
//! The figure is [`super::figure`]'s, fitted to the operator's proportions,
//! posed by [`super::animate`]'s gait, and stood on the floor found by a ray
//! straight down from the eye; its eyes are at the walker's eye height.

use egui::{Color32, Mesh, Pos2, Rect, Shape, Ui};
use pdfcer_3d::vec3::{add, dot, normalize, scale, sub, unit_cross};
use pdfcer_3d::{Camera, Ray};
use pdfcer_3d_nav::{Lens, Walk};

use super::ModelView;
use super::animate::{Gait, posed};
use super::figure::{Kind, Measures, builtin, fitted};
use super::measure::{aspect_of, whole};
use super::shadow::{self, Floor};

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
/// What a wheel notch multiplies the camera's distance by, pulled in.
const NOTCH: f64 = 0.8;
/// The most notches the camera is pulled in, and let out.
const PULL: (f64, f64) = (8.0, -4.0);
/// The nearest the camera comes to the aim point with a wall behind, eye
/// heights.
const NEAREST: f64 = 0.35;
/// The gap kept between the camera and a wall behind it, eye heights.
const CLEAR: f64 = 0.1;
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
    /// Standing under `walk`, on the floor `floor` model units below its
    /// eye when one was found.
    fn of(walk: &Walk, floor: Option<f64>) -> Option<Self> {
        let up = walk.axes().up();
        let forward = walk.axes().look(walk.yaw, 0.0);
        let right = unit_cross(forward, up)?;
        let body = walk.body();
        Some(Self {
            feet: sub(walk.eye, scale(up, floor.unwrap_or(body))),
            right,
            forward,
            up,
            height: body * HEIGHT_PER_EYE,
        })
    }

    /// A point `x` right, `y` forward and `z` up of the feet, in heights.
    fn at(&self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        let h = self.height;
        let p = add(self.feet, scale(self.right, x * h));
        add(add(p, scale(self.forward, y * h)), scale(self.up, z * h))
    }
}

/// The height, metres, of a person the gait's pace is set for.
const PERSON: f64 = 1.8;
/// How far below the eye the floor is looked for, eye heights.
const FLOOR_REACH: f64 = 1.5;

/// The distance to the floor below the eye that the figure stands on, from
/// `below`, the distance to the first surface straight down: `None`, so it
/// stands at the eye height, when none is near enough.
fn floor_at(body: f64, below: Option<f64>) -> Option<f64> {
    below.filter(|d| *d > 0.05 * body && *d < FLOOR_REACH * body)
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

/// The camera's distance behind the aim point pulled in `pull` wheel
/// notches, eye heights.
fn reach(pull: f64) -> f64 {
    (ARM * NOTCH.powf(pull.clamp(PULL.1, PULL.0))).max(NEAREST)
}

/// How far back the camera hangs, wanting `reach` eye heights, when
/// `behind` is the distance to the nearest surface behind the aim point, if
/// any: the full reach, or short of the surface, but never nearer than
/// [`NEAREST`].
fn arm_before(body: f64, reach: f64, behind: Option<f64>) -> f64 {
    let full = reach * body;
    match behind {
        Some(d) if d < full + CLEAR * body => (d - CLEAR * body).max(NEAREST * body),
        _ => full,
    }
}

/// The figure for `walk` standing at `stance`, fitted to `measures` and
/// posed by `gait`, as triangles in model coordinates.
fn figure(stance: &Stance, kind: Kind, measures: &Measures, gait: &Gait) -> Vec<Face> {
    let fit = fitted(builtin(kind), measures);
    posed(&fit, gait)
        .into_iter()
        .map(|(f, ink)| (f.map(|p| stance.at(p)), ink))
        .collect()
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
            .map_or(reach(self.nav.pull) * walk.body(), |(_, arm)| arm);
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
        let arm = arm_before(walk.body(), reach(self.nav.pull), behind);
        self.nav.arm = Some((walk, arm));
        crate::diag::trace_changed("model-behind-arm", || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("model-behind-arm eye-heights={:.2}", arm / walk.body())
        });
    }

    /// The wheel over the picture, seen from behind: pull the camera in or
    /// let it out by `wheel` points, 50 a notch.
    pub(super) fn pull_camera(&mut self, wheel: f64) {
        let pull = (self.nav.pull + wheel / 50.0).clamp(PULL.1, PULL.0);
        if pull != self.nav.pull {
            self.nav.pull = pull;
            self.nav.arm = None;
        }
    }

    /// Drags seen from behind, as a game's: the left button's `left` turns
    /// the camera round the figure, which keeps its heading; the right
    /// button's `right` first turns the figure to where the camera looks,
    /// then turns both.
    pub(super) fn drag_behind(&mut self, left: egui::Vec2, right: Option<egui::Vec2>) {
        let bind = self.nav.prefs.bind;
        let Some(walk) = &mut self.nav.walk else {
            return;
        };
        if left != egui::Vec2::ZERO {
            let (yaw, pitch) = bind.look(left, super::navmouse::LOOK_RATE);
            walk.look(yaw, pitch);
        }
        if let Some(right) = right {
            let (yaw, pitch) = bind.look(right, super::navmouse::LOOK_RATE);
            walk.turn(walk.look_yaw + yaw, pitch);
            walk.centre_look();
        }
    }

    /// Find the floor under the walker once per place, and advance the
    /// figure's gait by the ground covered since the last frame.
    pub(super) fn update_figure(&mut self, ui: &Ui) {
        self.read_typed(ui);
        let Some(walk) = self.nav.walk.filter(|_| self.nav.prefs.behind) else {
            self.nav.last_eye = None;
            return;
        };
        if !self.nav.stand.is_some_and(|(w, _)| w == walk) {
            let ray = Ray {
                origin: walk.eye,
                direction: scale(walk.axes().up(), -1.0),
                min_distance: 0.0,
            };
            let below = self.hit(&ray).map(|h| h.distance);
            let floor = floor_at(walk.body(), below);
            self.nav.stand = Some((walk, floor));
            crate::diag::trace_changed("model-figure-floor", || {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-figure-floor found={} eye-heights={:.3}",
                    floor.is_some(),
                    floor.unwrap_or(walk.body()) / walk.body()
                )
            });
        }
        let height = walk.body() * HEIGHT_PER_EYE;
        let dt = f64::from(ui.input(|i| i.stable_dt.min(0.1)));
        let moved = self
            .nav
            .last_eye
            .map_or([0.0; 3], |last| sub(walk.eye, last));
        self.nav.last_eye = Some(walk.eye);
        let up = walk.axes().up();
        let ahead = dot(moved, walk.axes().look(walk.yaw, 0.0));
        let level = sub(moved, scale(up, dot(moved, up)));
        let across = (dot(level, level) - ahead * ahead).max(0.0).sqrt();
        // The gait counts in metres of a figure as tall as a person.
        let metres = PERSON / height;
        let easing = self
            .nav
            .gait
            .step(ahead * metres, across * metres, dt, PERSON);
        if easing {
            ui.ctx().request_repaint();
        }
        let walking = self.nav.gait.swing > 0.5;
        crate::diag::trace_changed("model-figure-gait", || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("model-figure-gait walking={walking}")
        });
    }

    /// Typed words the figure answers to.
    fn read_typed(&mut self, ui: &Ui) {
        let typed: String = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Text(t) => Some(t.to_lowercase()),
                    _ => None,
                })
                .collect()
        });
        if typed.is_empty() || crate::canvas::textedit::composing(ui.ctx()) {
            return;
        }
        let heard = &mut self.nav.typed;
        heard.push_str(&typed);
        let extra = heard.chars().count().saturating_sub(16);
        *heard = heard.chars().skip(extra).collect();
        let next = [("werewolf", Kind::Werewolf), ("human", Kind::Human)]
            .into_iter()
            .find(|(word, _)| heard.ends_with(word))
            .map(|(_, kind)| kind);
        if let Some(kind) = next {
            heard.clear();
            if self.nav.prefs.kind != kind {
                self.nav.prefs.kind = kind;
                self.nav.store(ui.ctx());
            }
        }
    }

    /// The figure and its shadows over the picture `rect`, while walking is
    /// seen from behind.
    pub(super) fn paint_figure(&self, ui: &Ui, rect: Rect) {
        let Some(walk) = self.nav.walk.filter(|_| self.nav.prefs.behind) else {
            return;
        };
        let floor = self
            .nav
            .stand
            .filter(|(w, _)| *w == walk)
            .and_then(|(_, f)| f);
        let Some(stance) = Stance::of(&walk, floor) else {
            return;
        };
        let camera = self.behind_camera(&walk, aspect_of(rect), &self.lens);
        let m = self.nav.prefs.figure;
        let faces = figure(&stance, self.nav.prefs.kind, &m, &self.nav.gait);
        let (mesh, bounds) = painted(&faces, &camera, rect.min, whole(rect));
        if mesh.indices.is_empty() {
            return;
        }
        let triangles = mesh.indices.len() / 3;
        let painter = ui.painter().with_clip_rect(rect);
        let shadows = self.shadows(&stance, &walk, &faces, &camera, rect);
        let shadowed = !shadows.indices.is_empty();
        painter.add(Shape::mesh(shadows));
        painter.add(Shape::mesh(mesh));
        crate::diag::ui_rect_visible(REGION_FIGURE, bounds, rect);
        let feet = camera.pixel_of(stance.feet, whole(rect)[0], whole(rect)[1]);
        crate::diag::trace_changed("model-figure", || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-figure triangles={triangles} on-floor={} blob={} cast={} shadowed={shadowed} feet-y={}",
                floor.is_some(),
                m.blob,
                m.cast,
                feet.map_or_else(|| "none".to_owned(), |[_, y]| format!("{y:.0}"))
            )
        });
    }

    /// The shadows the operator chose for the figure `faces` at `stance`.
    fn shadows(
        &self,
        stance: &Stance,
        walk: &Walk,
        faces: &[Face],
        camera: &Camera,
        rect: Rect,
    ) -> Mesh {
        let m = self.nav.prefs.figure;
        let mut mesh = Mesh::default();
        let floor = Floor {
            at: stance.feet,
            up: stance.up,
            right: stance.right,
            forward: stance.forward,
        };
        if m.blob {
            let half = stance.height * 0.22 * m.build.max(0.5) * m.shoulders.max(0.7);
            shadow::patch(&mut mesh, &floor, half, camera, rect.min, whole(rect));
        }
        if m.cast {
            let light = shadow::light(stance.up, walk.axes().look(0.7, 0.0));
            shadow::cast(
                &mut mesh,
                &floor,
                light,
                faces,
                camera,
                rect.min,
                whole(rect),
            );
        }
        mesh
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
        assert!((arm_before(1.6, ARM, None) - ARM * 1.6).abs() < 1e-9);
        assert!((arm_before(1.6, ARM, Some(10.0)) - ARM * 1.6).abs() < 1e-9);
        assert!((arm_before(1.6, ARM, Some(1.6)) - 1.6 * (1.0 - CLEAR)).abs() < 1e-9);
        assert!((arm_before(1.6, ARM, Some(0.1)) - NEAREST * 1.6).abs() < 1e-9);
    }

    #[test]
    fn the_wheel_pulls_the_camera_in_and_out_within_bounds() {
        assert!((reach(0.0) - ARM).abs() < 1e-9);
        assert!(reach(2.0) < reach(1.0) && reach(-1.0) > ARM);
        assert!((reach(100.0) - reach(PULL.0)).abs() < 1e-9 && reach(PULL.0) >= NEAREST);
        assert!((reach(-100.0) - reach(PULL.1)).abs() < 1e-9);
    }

    fn drawn(walk: &Walk, kind: Kind) -> Vec<Face> {
        let stance = Stance::of(walk, None).unwrap();
        figure(&stance, kind, &Measures::default(), &Gait::default())
    }

    #[test]
    fn the_floor_below_is_stood_on_only_when_near() {
        assert_eq!(floor_at(1.6, Some(1.7)), Some(1.7));
        assert_eq!(floor_at(1.6, Some(30.0)), None);
        assert_eq!(floor_at(1.6, None), None);
        let walk = walker();
        let low = Stance::of(&walk, Some(1.7)).unwrap();
        assert!((low.feet[2] + 0.1).abs() < 1e-9, "{:?}", low.feet);
    }

    #[test]
    fn the_figure_stands_on_the_floor_under_the_eye_and_reaches_its_height() {
        let walk = walker();
        let faces = drawn(&walk, Kind::Human);
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
        let (mesh, bounds) = painted(&drawn(&walk, Kind::Werewolf), &c, Pos2::ZERO, [600, 400]);
        assert!(!mesh.indices.is_empty());
        assert!((bounds.center().x - 300.0).abs() < 5.0, "{bounds:?}");
        assert!(bounds.top() > 200.0, "{bounds:?}");
        assert!(
            bounds.bottom() < 400.0,
            "the feet are off the picture: {bounds:?}"
        );
    }
}
