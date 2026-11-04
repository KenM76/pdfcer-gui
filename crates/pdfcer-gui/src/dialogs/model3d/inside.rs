//! A saved view whose camera stands inside the model opens walking from that
//! camera, looking where it looks, instead of orbiting fitted from outside:
//! an orbit's camera always stands outside the sphere round the model, so a
//! view made from inside a room would otherwise show the room's outside.
//!
//! Inside means the camera's position (`/C2W`'s translation, in the model's
//! own units) lies in the model's bounding box, and the view is perspective;
//! an orthographic view has no standing point and keeps its framing.

use pdfcer_3d::Bounds;
use pdfcer_core::threed::ThreeDSavedView;

use super::ModelView;
use super::orbit::{Axes, radius};
use super::walk::Walk;

/// Where `view`'s camera stands, when that is inside `bounds`; `Err` names
/// why not, as the trace says it.
fn eye_inside(view: &ThreeDSavedView, bounds: &Bounds) -> Result<[f64; 3], &'static str> {
    let Some(m) = view.camera_to_world.filter(|_| !view.orthographic) else {
        return Err("none"); // ui-text-exempt: trace value, never displayed
    };
    let eye = [m[9], m[10], m[11]];
    let inside = (0..3).all(|i| eye[i] >= bounds.min[i] && eye[i] <= bounds.max[i]);
    if inside {
        Ok(eye)
    } else {
        Err("outside") // ui-text-exempt: trace value, never displayed
    }
}

impl ModelView {
    /// Walk from `view`'s camera when it stands inside the model; whether it
    /// did.
    pub(super) fn walk_from_view(&mut self, view: &ThreeDSavedView) -> bool {
        let aim = view.aim(1.0);
        let eye = eye_inside(view, &self.bounds);
        let word = eye.as_ref().map_or_else(|why| *why, |_| "inside"); // ui-text-exempt: trace value
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("file-view-eye={word}"));
        let (Ok(eye), Some(aim)) = (eye, aim) else {
            return false;
        };
        let axes = Axes::upright(self.upright);
        let mut walk = Walk::standing(eye, aim.direction, axes, radius(&self.bounds));
        walk.height = self.eye_height();
        self.nav.walk = Some(walk);
        self.nav.drone = None;
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| "model-walk on=true".to_owned());
        true
    }

    /// Open walking from the file's opening view when its camera is inside
    /// the model, in place of the walking or orbiting start chosen.
    pub(crate) fn start_inside(&mut self) {
        if let Some(view) = self.opening.clone()
            && self.walk_from_view(&view)
        {
            self.nav.start_walking = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(eye: [f64; 3], orthographic: bool) -> ThreeDSavedView {
        let mut view = ThreeDSavedView::default();
        view.camera_to_world = Some([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, eye[0], eye[1], eye[2],
        ]);
        view.orthographic = orthographic;
        view
    }

    #[test]
    fn a_camera_in_the_box_is_inside_and_one_beyond_it_is_not() {
        let bounds = Bounds {
            min: [0.0; 3],
            max: [10.0, 10.0, 3.0],
        };
        assert_eq!(
            eye_inside(&view([5.0, 5.0, 1.6], false), &bounds),
            Ok([5.0, 5.0, 1.6])
        );
        assert_eq!(
            eye_inside(&view([5.0, -20.0, 1.6], false), &bounds),
            Err("outside")
        );
        assert_eq!(
            eye_inside(&view([5.0, 5.0, 1.6], true), &bounds),
            Err("none")
        );
    }
}
