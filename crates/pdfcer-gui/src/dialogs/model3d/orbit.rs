//! The viewer's camera: a turn about an up axis, a zoom and a pan, each
//! relative to the view that fits the model. A named view turns about the
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

/// The axes a turn is measured in: yaw 0, pitch 0 looks along `front`, and
/// `up` appears upward. Both unit length and perpendicular.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Axes {
    front: [f64; 3],
    up: [f64; 3],
}

impl Axes {
    /// The operator's choice; perpendicular by construction.
    fn upright(upright: Upright) -> Self {
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
    fn right(&self) -> [f64; 3] {
        cross(self.front, self.up)
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
        })
    }

    /// The unit direction the camera looks along.
    pub fn direction(&self) -> [f64; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let (r, f, u) = (self.axes.right(), self.axes.front, self.axes.up);
        std::array::from_fn(|i| -sy * cp * r[i] + cy * cp * f[i] - sp * u[i])
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
            .camera(bounds, aspect, view)
            .map_or(2.0, |c| visible_height(&c))
            / radius(bounds);
        for (pan, off) in self.pan.iter_mut().zip(offset) {
            *pan += off * span * (1.0 / before - 1.0 / after);
        }
        self.zoom = after;
    }

    /// The camera for `bounds` in an image of `aspect` (width / height);
    /// `view` is the file's opening view, whose framing applies while
    /// [`Orbit::framed`] and orthographic.
    pub fn camera(
        &self,
        bounds: &Bounds,
        aspect: f64,
        view: Option<&ThreeDSavedView>,
    ) -> Result<Camera, pdfcer_3d::RenderError> {
        let dir = self.direction();
        let up = self.axes.up;
        let mut camera = Camera::fit(bounds, dir, up, self.perspective, aspect)?;
        if self.framed
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

/// The model's radius, or 1 for a model with no extent.
fn radius(bounds: &Bounds) -> f64 {
    match bounds.radius() {
        r if r.is_finite() && r > 0.0 => r,
        _ => 1.0,
    }
}

/// How much of the target plane the picture shows vertically, in model units.
fn visible_height(camera: &Camera) -> f64 {
    match camera.projection {
        Projection::Orthographic { height } => height,
        Projection::Perspective { fov_y } => {
            let distance = (0..3)
                .map(|i| (camera.eye[i] - camera.target[i]).powi(2))
                .sum::<f64>()
                .sqrt();
            2.0 * distance * (fov_y.to_radians() / 2.0).tan()
        }
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// `v` at unit length, or `None` for a zero or non-finite vector.
fn unit(v: [f64; 3]) -> Option<[f64; 3]> {
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
            .camera(&cube(), 1.5, Some(&view))
            .expect("a view forms");
        let look: [f64; 3] = std::array::from_fn(|i| camera.target[i] - camera.eye[i]);
        assert!(parallel(look, [0.6, 0.0, -0.8]), "{look:?}");
        assert!(parallel(camera.up, [0.0, 1.0, 0.0]), "{:?}", camera.up);
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
            .camera(&cube(), 2.0, Some(&view))
            .expect("a view forms");
        let fitted = orbit.camera(&cube(), 2.0, None).expect("a view forms");
        assert_ne!(
            framed.projection, fitted.projection,
            "the view's height applies"
        );
        orbit.perspective = true;
        let free = orbit
            .camera(&cube(), 2.0, Some(&view))
            .expect("a view forms");
        assert!(matches!(free.projection, Projection::Perspective { .. }));
    }

    #[test]
    fn zoom_brings_the_eye_closer_and_pan_moves_the_target() {
        let bounds = cube();
        let mut orbit = Orbit::named(1, true, Upright::default());
        let fitted = orbit.camera(&bounds, 1.0, None).expect("a view forms");
        orbit.zoom = 2.0;
        orbit.pan = [0.5, 0.0];
        let moved = orbit.camera(&bounds, 1.0, None).expect("a view forms");
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
        let camera = orbit.camera(bounds, aspect, None).expect("a view forms");
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
            orbit.zoom_at(&bounds, aspect, 2.5, offset, None);
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
        orbit.zoom_at(&cube(), 1.0, 3.0, [0.0, 0.0], None);
        assert_eq!(orbit.pan, [0.0, 0.0]);
    }
}
