//! The viewer's camera: a turn about an up axis, a zoom and a pan, each
//! relative to the view that fits the model, or, once a finger or the mouse
//! has turned it, relative to the model point it turns about ([`Anchor`]).
//!
//! The fitted view frames the sphere round the model's centre that holds
//! every vertex, so turning the model or reshaping the picture never changes
//! its size or perspective. A named view turns about the
//! operator's chosen up ([`Upright`], z by default);
//! the file's own opening view (`ThreeDSavedView::aim`) turns about its own
//! up from its own direction, and an orthographic view that fixes its framing
//! keeps it (`SavedViewAim::frame`) until Fit or a named view drops it.

use std::f64::consts::FRAC_PI_2;

use pdfcer_3d::{Bounds, Camera, Projection};
use pdfcer_core::threed::{ThreeDSavedView, ViewFit};

use super::axes::Upright;

/// The named views as (yaw, pitch) in degrees, in `t::view_names` order.
/// Yaw 0 looks along the Front direction; pitch looks down.
const VIEWS: [(f64, f64); 5] = [
    (45.0, 35.264),
    (0.0, 0.0),
    (90.0, 0.0),
    (0.0, 89.0),
    (180.0, 0.0),
];

/// Kept off ±90° so the view never looks straight along the up axis.
pub(super) const PITCH_LIMIT: f64 = FRAC_PI_2 - 0.01;
/// The zoom range, as a multiple of the fitted view.
const ZOOM_RANGE: (f64, f64) = (0.05, 50.0);
/// The perspective field of view, degrees, vertically: the engine's.
const FOV_Y: f64 = 30.0;
/// The clear space round the fitted sphere, as a multiple of its radius.
const MARGIN: f64 = 1.05;

/// The model point an orbit turns about, its depth along the look and where
/// it sits off the picture's centre; an orthographic view shows the height a
/// perspective one would at that depth, so switching projection keeps the
/// scale. The point keeps its place on the picture however the view turns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Anchor {
    pub point: [f64; 3],
    pub distance: f64,
    /// The point's offset from the look's axis, right and up in the
    /// picture, model units.
    pub offset: [f64; 2],
}

impl Anchor {
    /// The point on the look's axis level with [`Anchor::point`], for a
    /// view whose picture right and up are `right` and `up`.
    fn base(&self, right: [f64; 3], up: [f64; 3]) -> [f64; 3] {
        std::array::from_fn(|i| self.point[i] - right[i] * self.offset[0] - up[i] * self.offset[1])
    }
}

/// The axes a turn is measured in: yaw 0, pitch 0 looks along `front`, and
/// `up` appears upward. Both unit length and perpendicular.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Axes {
    pub front: [f64; 3],
    pub up: [f64; 3],
}

impl Axes {
    /// The operator's choice; perpendicular by construction.
    pub(super) fn upright(upright: Upright) -> Self {
        Self {
            front: upright.front.vector(),
            up: upright.up.vector(),
        }
    }

    /// A look direction and an up, or `None` when they are not two
    /// independent directions.
    fn of(direction: [f64; 3], up: [f64; 3]) -> Option<Self> {
        let up = unit(up)?;
        let along = dot(direction, up);
        let front = unit(std::array::from_fn(|i| direction[i] - up[i] * along))?;
        Some(Self { front, up })
    }

    /// Image right at yaw 0, pitch 0.
    pub fn right(&self) -> [f64; 3] {
        cross(self.front, self.up)
    }

    /// The unit direction `yaw` round and `pitch` down from `front`.
    pub fn look(&self, yaw: f64, pitch: f64) -> [f64; 3] {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let (r, f, u) = (self.right(), self.front, self.up);
        std::array::from_fn(|i| -sy * cp * r[i] + cy * cp * f[i] - sp * u[i])
    }

    /// The yaw and pitch [`Axes::look`] turns into `direction`, the pitch
    /// held inside [`PITCH_LIMIT`]; `None` for a zero direction.
    pub fn angles(&self, direction: [f64; 3]) -> Option<(f64, f64)> {
        let d = unit(direction)?;
        let pitch = (-dot(d, self.up)).clamp(-1.0, 1.0).asin();
        let yaw = (-dot(d, self.right())).atan2(dot(d, self.front));
        Some((yaw, pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT)))
    }
}

/// Where the camera is, relative to a fitted view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Orbit {
    /// Radians around the up axis.
    pub yaw: f64,
    /// Radians above the horizon, looking down when positive.
    pub pitch: f64,
    /// 1 frames the whole model.
    pub zoom: f64,
    /// The target's shift along the image's right and up, in model radii.
    pub pan: [f64; 2],
    pub perspective: bool,
    axes: Axes,
    /// The file's view fixes the centre and scale, applied while the
    /// projection is orthographic.
    pub framed: bool,
    /// The point turned about, in place of the fitted view, from the first
    /// turn by hand until a fit or a named view.
    pub anchor: Option<Anchor>,
}

impl Orbit {
    /// Named view `index`, turned about `upright`.
    pub fn named(index: usize, perspective: bool, upright: Upright) -> Self {
        let (yaw, pitch) = VIEWS[index];
        Self {
            yaw: yaw.to_radians(),
            pitch: pitch.to_radians(),
            zoom: 1.0,
            pan: [0.0, 0.0],
            perspective,
            axes: Axes::upright(upright),
            framed: false,
            anchor: None,
        }
    }

    /// The file's opening view, or `None` when it carries no camera of its
    /// own or one whose direction and up coincide.
    pub fn saved(view: &ThreeDSavedView) -> Option<Self> {
        let aim = view.aim(1.0)?;
        Some(Self {
            yaw: 0.0,
            pitch: 0.0,
            zoom: 1.0,
            pan: [0.0, 0.0],
            perspective: !aim.orthographic,
            axes: Axes::of(aim.direction, aim.up)?,
            framed: matches!(aim.fit, ViewFit::Framed { .. }),
            anchor: None,
        })
    }

    /// The unit direction the camera looks along.
    pub fn direction(&self) -> [f64; 3] {
        self.axes.look(self.yaw, self.pitch)
    }

    /// Turn to look along `direction` from the side it points away from,
    /// fitted again; a direction along the up axis stops just short of it.
    pub fn look_along(&mut self, direction: [f64; 3]) {
        if let Some((yaw, pitch)) = self.axes.angles(direction) {
            (self.yaw, self.pitch) = (yaw, pitch);
            self.refit();
        }
    }

    /// Back to the view that fits the whole model, keeping the direction:
    /// no zoom, pan, file framing or point turned about.
    pub fn refit(&mut self) {
        (self.zoom, self.pan, self.framed) = (1.0, [0.0, 0.0], false);
        self.anchor = None;
    }

    /// Turn about `point` from now on, without moving the picture; `camera`
    /// is this orbit's camera, and `point` stays where it shows on it.
    pub fn anchor_at(&mut self, camera: &Camera, point: [f64; 3]) {
        let dir = self.direction();
        let right = normalise(cross(dir, self.axes.up));
        let image_up = cross(right, dir);
        let v: [f64; 3] = std::array::from_fn(|i| point[i] - camera.eye[i]);
        let depth = dot(v, dir);
        let offset = [dot(v, right), dot(v, image_up)];
        let distance = match camera.projection {
            Projection::Orthographic { height } => height / 2.0 / half_fov().tan(),
            Projection::Perspective { .. } | Projection::Fisheye { .. } => depth,
        };
        if distance.is_finite() && distance > 0.0 {
            self.anchor = Some(Anchor {
                point,
                distance,
                offset,
            });
            (self.zoom, self.pan, self.framed) = (1.0, [0.0, 0.0], false);
        }
    }

    /// Pan so `point` sits at the picture's centre, keeping the turn and
    /// the zoom.
    pub fn centre_on(&mut self, bounds: &Bounds, point: [f64; 3]) {
        let dir = self.direction();
        let right = normalise(cross(dir, self.axes.up));
        let image_up = cross(right, dir);
        let centre = self
            .anchor
            .map_or_else(|| bounds.centre(), |a| a.base(right, image_up));
        let off: [f64; 3] = std::array::from_fn(|i| point[i] - centre[i]);
        let r = radius(bounds);
        self.pan = [dot(off, right) / r, dot(off, image_up) / r];
        self.framed = false;
    }

    /// Multiply the zoom by `factor`, keeping the model point under the
    /// pointer where it is on the picture.
    ///
    /// `offset` is the pointer's position from the picture's centre, right and
    /// up, in picture heights. Exact on the plane through the target facing
    /// the camera, the plane the pan moves in.
    pub fn zoom_at(
        &mut self,
        bounds: &Bounds,
        reach: f64,
        aspect: f64,
        factor: f64,
        offset: [f64; 2],
        view: Option<&ThreeDSavedView>,
    ) {
        let before = self.zoom;
        let after = (before * factor).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
        let fitted = Self {
            zoom: 1.0,
            pan: [0.0, 0.0],
            ..*self
        };
        // The fitted view's visible height, in model radii: what one picture
        // height spans at zoom 1.
        let span = fitted
            .camera(bounds, reach, aspect, view)
            .map_or(2.0, |c| visible_height(&c))
            / radius(bounds);
        for (pan, off) in self.pan.iter_mut().zip(offset) {
            *pan += off * span * (1.0 / before - 1.0 / after);
        }
        self.zoom = after;
    }

    /// The camera for `bounds`, whose every vertex is within `reach` of its
    /// centre, in an image of `aspect` (width / height); `view` is the file's
    /// opening view, whose framing applies while [`Orbit::framed`] and
    /// orthographic.
    pub fn camera(
        &self,
        bounds: &Bounds,
        reach: f64,
        aspect: f64,
        view: Option<&ThreeDSavedView>,
    ) -> Result<Camera, pdfcer_3d::RenderError> {
        let dir = self.direction();
        let up = self.axes.up;
        let mut camera = match self.anchor {
            Some(a) => about(a, dir, up, self.perspective),
            None => fitted(bounds.centre(), reach, dir, up, self.perspective, aspect),
        };
        if self.anchor.is_none()
            && self.framed
            && !self.perspective
            && let Some(aim) = view.and_then(|v| v.aim(aspect))
        {
            aim.frame(&mut camera);
        }
        let radius = radius(bounds);
        let right = normalise(cross(dir, up));
        let image_up = cross(right, dir);
        let shift: [f64; 3] =
            std::array::from_fn(|i| (right[i] * self.pan[0] + image_up[i] * self.pan[1]) * radius);
        let target: [f64; 3] = std::array::from_fn(|i| camera.target[i] + shift[i]);
        camera.eye =
            std::array::from_fn(|i| target[i] + (camera.eye[i] - camera.target[i]) / self.zoom);
        camera.target = target;
        if let Projection::Orthographic { height } = camera.projection {
            camera.projection = Projection::Orthographic {
                height: height / self.zoom,
            };
        }
        Ok(camera)
    }
}

/// Half the vertical field of view, radians.
fn half_fov() -> f64 {
    (FOV_Y / 2.0).to_radians()
}

/// The camera looking along `dir` that holds the sphere of `reach` round
/// `centre` in an image of `aspect`, whichever way it looks: so turning
/// never changes the model's size, and only the picture's shorter side does.
fn fitted(
    centre: [f64; 3],
    reach: f64,
    dir: [f64; 3],
    up: [f64; 3],
    perspective: bool,
    aspect: f64,
) -> Camera {
    let r = reach * MARGIN;
    let aspect = if aspect.is_finite() && aspect > 0.0 {
        aspect
    } else {
        1.0
    };
    let half = half_fov().min((half_fov().tan() * aspect).atan());
    let (distance, projection) = if perspective {
        (r / half.sin(), Projection::Perspective { fov_y: FOV_Y })
    } else {
        let height = 2.0 * r / aspect.min(1.0);
        (2.0 * r, Projection::Orthographic { height })
    };
    Camera {
        eye: std::array::from_fn(|i| centre[i] - dir[i] * distance),
        target: centre,
        up,
        projection,
    }
}

/// The camera looking along `dir` that shows `anchor`'s point at its depth
/// and offset.
fn about(anchor: Anchor, dir: [f64; 3], up: [f64; 3], perspective: bool) -> Camera {
    let right = normalise(cross(dir, up));
    let base = anchor.base(right, cross(right, dir));
    let projection = if perspective {
        Projection::Perspective { fov_y: FOV_Y }
    } else {
        Projection::Orthographic {
            height: 2.0 * anchor.distance * half_fov().tan(),
        }
    };
    Camera {
        eye: std::array::from_fn(|i| base[i] - dir[i] * anchor.distance),
        target: base,
        up,
        projection,
    }
}

/// How far the farthest vertex the meshes' triangles use lies from
/// `bounds`' centre: the fitted sphere's radius; [`radius`] when none.
pub(super) fn reach(meshes: &[pdfcer_3d::TriangleMesh], bounds: &Bounds) -> f64 {
    let c = bounds.centre();
    let far = meshes
        .iter()
        .flat_map(|m| {
            m.triangles
                .iter()
                .flatten()
                .filter_map(|&i| m.positions.get(i as usize))
        })
        .map(|p| {
            let d: [f64; 3] = std::array::from_fn(|i| p[i] - c[i]);
            dot(d, d)
        })
        .filter(|d| d.is_finite())
        .fold(0.0_f64, f64::max)
        .sqrt();
    if far > 0.0 { far } else { radius(bounds) }
}

/// The model's radius, or 1 for a model with no extent.
pub(super) fn radius(bounds: &Bounds) -> f64 {
    match bounds.radius() {
        r if r.is_finite() && r > 0.0 => r,
        _ => 1.0,
    }
}

/// How much of the target plane the picture shows vertically, in model units.
fn visible_height(camera: &Camera) -> f64 {
    match camera.projection {
        Projection::Orthographic { height } => height,
        // A fisheye's field spans its height as a perspective's does; past
        // 180° the plane is unbounded, so the span is taken just short of it.
        Projection::Perspective { fov_y } | Projection::Fisheye { fov: fov_y, .. } => {
            let fov_y = fov_y.min(179.0);
            let distance = (0..3)
                .map(|i| (camera.eye[i] - camera.target[i]).powi(2))
                .sum::<f64>()
                .sqrt();
            2.0 * distance * (fov_y.to_radians() / 2.0).tan()
        }
    }
}

pub(super) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// `v` at unit length, or `None` for a zero or non-finite vector.
pub(super) fn unit(v: [f64; 3]) -> Option<[f64; 3]> {
    let length = dot(v, v).sqrt();
    (length.is_finite() && length > 1e-12).then(|| v.map(|c| c / length))
}

fn normalise(v: [f64; 3]) -> [f64; 3] {
    unit(v).unwrap_or([1.0, 0.0, 0.0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> Bounds {
        Bounds {
            min: [-1.0; 3],
            max: [1.0; 3],
        }
    }

    /// A view looking along +x and down, with +y up: no named view's axes.
    fn corner() -> ThreeDSavedView {
        let mut view = ThreeDSavedView::default();
        view.name = "Corner".to_owned();
        view.camera_to_world = Some([
            -0.8, 0.0, -0.6, 0.0, 1.0, 0.0, 0.6, 0.0, -0.8, -6.0, 0.0, 8.0,
        ]);
        view
    }

    fn parallel(a: [f64; 3], b: [f64; 3]) -> bool {
        let (a, b) = (normalise(a), normalise(b));
        (dot(a, b) - 1.0).abs() < 1e-9
    }

    #[test]
    fn the_named_views_look_the_way_their_names_say() {
        let front = Orbit::named(1, false, Upright::default()).direction();
        assert!(
            (front[1] - 1.0).abs() < 1e-9,
            "front looks along +y: {front:?}"
        );
        let top = Orbit::named(3, false, Upright::default()).direction();
        assert!(top[2] < -0.99, "top looks down: {top:?}");
        let iso = Orbit::named(0, false, Upright::default()).direction();
        assert!(iso[0] < 0.0 && iso[1] > 0.0 && iso[2] < 0.0, "{iso:?}");
    }

    #[test]
    fn with_y_up_the_named_views_turn_about_y() {
        let upright = Upright::default().with_up(super::super::axes::Axis::PosY);
        let front = Orbit::named(1, false, upright).direction();
        assert!(front[2] < -0.999, "front looks along -z: {front:?}");
        let top = Orbit::named(3, false, upright).direction();
        assert!(top[1] < -0.99, "top looks down -y: {top:?}");
    }

    #[test]
    fn the_files_view_looks_along_its_own_axis_with_its_own_up() {
        let view = corner();
        let orbit = Orbit::saved(&view).expect("a camera matrix");
        assert!(orbit.perspective && !orbit.framed);
        let camera = orbit
            .camera(&cube(), radius(&cube()), 1.5, Some(&view))
            .expect("a view forms");
        let look: [f64; 3] = std::array::from_fn(|i| camera.target[i] - camera.eye[i]);
        assert!(parallel(look, [0.6, 0.0, -0.8]), "{look:?}");
        assert!(parallel(camera.up, [0.0, 1.0, 0.0]), "{:?}", camera.up);
    }

    #[test]
    fn a_named_view_written_into_the_file_reads_back_looking_the_same_way() {
        let upright = Upright::default().with_up(super::super::axes::Axis::PosY);
        for (i, perspective) in [(0, true), (1, false), (3, true), (4, false)] {
            let orbit = Orbit::named(i, perspective, upright);
            let camera = orbit
                .camera(&cube(), radius(&cube()), 1.5, None)
                .expect("a view forms");
            let view = ThreeDSavedView::from_camera("v", &camera, 1.5).expect("a view writes");
            let back = Orbit::saved(&view).expect("the written view has a camera");
            assert!(
                parallel(back.direction(), orbit.direction()),
                "view {i}: {:?} read back as {:?}",
                orbit.direction(),
                back.direction()
            );
            assert_eq!(back.perspective, perspective, "view {i}");
        }
    }

    #[test]
    fn a_view_without_a_camera_or_with_up_along_its_look_is_not_offered() {
        assert!(Orbit::saved(&ThreeDSavedView::default()).is_none());
        let mut view = corner();
        view.camera_to_world = Some([1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
        assert!(Orbit::saved(&view).is_none());
    }

    #[test]
    fn a_framed_view_keeps_its_scale_until_perspective_is_chosen() {
        let mut view = corner();
        view.orthographic = true;
        view.view_box = Some([200.0, 100.0]);
        let mut orbit = Orbit::saved(&view).expect("a camera matrix");
        assert!(orbit.framed && !orbit.perspective);
        let framed = orbit
            .camera(&cube(), radius(&cube()), 2.0, Some(&view))
            .expect("a view forms");
        let fitted = orbit
            .camera(&cube(), radius(&cube()), 2.0, None)
            .expect("a view forms");
        assert_ne!(
            framed.projection, fitted.projection,
            "the view's height applies"
        );
        orbit.perspective = true;
        let free = orbit
            .camera(&cube(), radius(&cube()), 2.0, Some(&view))
            .expect("a view forms");
        assert!(matches!(free.projection, Projection::Perspective { .. }));
    }

    #[test]
    fn zoom_brings_the_eye_closer_and_pan_moves_the_target() {
        let bounds = cube();
        let mut orbit = Orbit::named(1, true, Upright::default());
        let fitted = orbit
            .camera(&bounds, radius(&bounds), 1.0, None)
            .expect("a view forms");
        orbit.zoom = 2.0;
        orbit.pan = [0.5, 0.0];
        let moved = orbit
            .camera(&bounds, radius(&bounds), 1.0, None)
            .expect("a view forms");
        let gap = |c: &Camera| {
            (0..3)
                .map(|i| (c.eye[i] - c.target[i]).powi(2))
                .sum::<f64>()
                .sqrt()
        };
        assert!((gap(&moved) - gap(&fitted) / 2.0).abs() < 1e-9);
        assert!(moved.target != fitted.target);
        assert!(
            moved.target[2].abs() < 1e-9,
            "a sideways pan keeps the height"
        );
    }

    /// The model point on the target plane at `offset` (picture heights,
    /// right and up from the centre).
    fn under(orbit: &Orbit, bounds: &Bounds, aspect: f64, offset: [f64; 2]) -> [f64; 3] {
        let camera = orbit
            .camera(bounds, radius(bounds), aspect, None)
            .expect("a view forms");
        let dir = orbit.direction();
        let right = normalise(cross(dir, orbit.axes.up));
        let up = cross(right, dir);
        let h = visible_height(&camera);
        std::array::from_fn(|i| camera.target[i] + (right[i] * offset[0] + up[i] * offset[1]) * h)
    }

    #[test]
    fn a_wheel_zoom_keeps_the_point_under_the_pointer_still() {
        let bounds = Bounds {
            min: [-1.0, -2.0, -0.5],
            max: [3.0, 1.0, 2.0],
        };
        let saved = Orbit::saved(&corner()).expect("a camera matrix");
        for start in [
            Orbit::named(0, true, Upright::default()),
            Orbit::named(0, false, Upright::default()),
            saved,
        ] {
            let mut orbit = start;
            orbit.pan = [0.1, -0.2];
            let (aspect, offset) = (1.6, [0.3, -0.15]);
            let before = under(&orbit, &bounds, aspect, offset);
            orbit.zoom_at(&bounds, radius(&bounds), aspect, 2.5, offset, None);
            let after = under(&orbit, &bounds, aspect, offset);
            for i in 0..3 {
                assert!(
                    (before[i] - after[i]).abs() < 1e-9,
                    "{before:?} vs {after:?}"
                );
            }
            assert!((orbit.zoom - 2.5).abs() < 1e-12);
        }
    }

    #[test]
    fn a_wheel_zoom_at_the_centre_does_not_pan() {
        let mut orbit = Orbit::named(1, true, Upright::default());
        orbit.zoom_at(&cube(), radius(&cube()), 1.0, 3.0, [0.0, 0.0], None);
        assert_eq!(orbit.pan, [0.0, 0.0]);
    }

    fn gap(c: &Camera) -> f64 {
        (0..3)
            .map(|i| (c.eye[i] - c.target[i]).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    #[test]
    fn turning_or_widening_the_picture_keeps_the_models_size() {
        let bounds = Bounds {
            min: [-4.0, -1.0, 0.0],
            max: [4.0, 1.0, 0.5],
        };
        let mut orbit = Orbit::named(0, true, Upright::default());
        let first = orbit.camera(&bounds, 4.0, 1.5, None).expect("a view forms");
        for yaw in [0.3, 1.1, 2.0] {
            orbit.yaw = yaw;
            for aspect in [1.2, 1.5, 3.0] {
                let c = orbit
                    .camera(&bounds, 4.0, aspect, None)
                    .expect("a view forms");
                assert!((gap(&c) - gap(&first)).abs() < 1e-9, "{yaw} {aspect}");
                assert_eq!(c.target, first.target);
            }
        }
    }

    #[test]
    fn an_anchor_keeps_the_picture_then_turns_about_its_point() {
        let bounds = cube();
        for perspective in [true, false] {
            let mut orbit = Orbit::named(0, perspective, Upright::default());
            orbit.zoom = 1.7;
            orbit.pan = [0.2, -0.1];
            let before = orbit.camera(&bounds, 1.5, 1.4, None).expect("a view forms");
            let dir = orbit.direction();
            let point: [f64; 3] = std::array::from_fn(|i| before.target[i] + dir[i] * 0.4);
            orbit.anchor_at(&before, point);
            let after = orbit.camera(&bounds, 1.5, 1.4, None).expect("a view forms");
            if perspective {
                for i in 0..3 {
                    assert!((after.eye[i] - before.eye[i]).abs() < 1e-9, "{perspective}");
                }
            } else {
                assert!((visible_height(&after) - visible_height(&before)).abs() < 1e-9);
            }
            orbit.yaw += 0.8;
            orbit.pitch = 0.4;
            let turned = orbit.camera(&bounds, 1.5, 1.4, None).expect("a view forms");
            for (t, p) in turned.target.iter().zip(point) {
                assert!((t - p).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn a_refit_forgets_the_point_turned_about() {
        let bounds = cube();
        let mut orbit = Orbit::named(0, true, Upright::default());
        let fitted = orbit.camera(&bounds, 1.5, 1.4, None).expect("a view forms");
        let point = [0.3, 0.2, 0.1];
        orbit.anchor_at(&fitted, point);
        orbit.zoom = 2.0;
        orbit.refit();
        assert_eq!(orbit.anchor, None);
        let again = orbit.camera(&bounds, 1.5, 1.4, None).expect("a view forms");
        assert_eq!(again, fitted);
    }

    /// Where `p` shows on `c`'s picture: right and up of the centre, as a
    /// share of the visible height at `p`'s depth.
    fn shown(c: &Camera, p: [f64; 3]) -> [f64; 2] {
        let f = normalise(std::array::from_fn(|i| c.target[i] - c.eye[i]));
        let r = normalise(cross(f, c.up));
        let u = cross(r, f);
        let v: [f64; 3] = std::array::from_fn(|i| p[i] - c.eye[i]);
        let h = match c.projection {
            Projection::Perspective { fov_y } => 2.0 * dot(v, f) * (fov_y / 2.0).to_radians().tan(),
            Projection::Fisheye { .. } => unreachable!("orbit tests use flat lenses"),
            Projection::Orthographic { height } => height,
        };
        [dot(v, r) / h, dot(v, u) / h]
    }

    #[test]
    fn a_pressed_point_off_centre_keeps_its_place_as_the_view_turns() {
        let bounds = cube();
        for perspective in [true, false] {
            let mut orbit = Orbit::named(0, perspective, Upright::default());
            orbit.zoom = 1.3;
            let before = orbit.camera(&bounds, 1.5, 1.4, None).expect("a view forms");
            let f = orbit.direction();
            let r = normalise(cross(f, orbit.axes.up));
            let u = cross(r, f);
            let point: [f64; 3] =
                std::array::from_fn(|i| before.target[i] + r[i] * 0.3 + u[i] * 0.2 + f[i] * 0.4);
            let at = shown(&before, point);
            orbit.anchor_at(&before, point);
            for (yaw, pitch) in [(0.0, 0.0), (0.8, 0.4), (-1.9, -0.7)] {
                orbit.yaw += yaw;
                orbit.pitch = pitch;
                let c = orbit.camera(&bounds, 1.5, 1.4, None).expect("a view forms");
                let now = shown(&c, point);
                for i in 0..2 {
                    assert!(
                        (now[i] - at[i]).abs() < 1e-9,
                        "{perspective} {now:?} {at:?}"
                    );
                }
            }
        }
    }
}
