//! A file's saved view, read into the navigation crate's terms: its aim
//! (`pdfcer_3d_nav::SavedAim`), its framing at a picture aspect
//! (`pdfcer_3d_nav::ViewFrame`), the orbit it opens on, and the walk a
//! camera standing inside the model starts.
//!
//! Inside means the camera's position (`/C2W`'s translation, in the model's
//! own units) lies in the model's bounding box and the view is perspective;
//! an orthographic view has no standing point and keeps its framing. An
//! orbit's camera always stands outside the sphere round the model, so a
//! view made from inside a room would otherwise show the room's outside.

use pdfcer_3d::NamedView;
use pdfcer_3d_nav::{Axes, Orbit, SavedAim, Upright, ViewFrame, Walk, radius};
use pdfcer_core::threed::{ThreeDSavedView, ViewFit};

use super::ModelView;

/// The named views in `text::panels::models::view_names` order; a choice's
/// index there is its index here.
const NAMED: [NamedView; 5] = [
    NamedView::Iso,
    NamedView::Front,
    NamedView::Right,
    NamedView::Top,
    NamedView::Back,
];

/// The orbit of named view `index` (Iso past the end).
pub(super) fn named(index: usize, perspective: bool, upright: Upright) -> Orbit {
    let view = NAMED.get(index).copied().unwrap_or(NamedView::Iso);
    Orbit::named(view, perspective, upright)
}

/// `view`'s aim, or `None` when it carries no camera of its own.
pub(super) fn saved_aim(view: &ThreeDSavedView) -> Option<SavedAim> {
    let aim = view.aim(1.0)?;
    let [.., tx, ty, tz] = view.camera_to_world?;
    Some(SavedAim {
        direction: aim.direction,
        up: aim.up,
        position: [tx, ty, tz],
        orthographic: aim.orthographic,
        framed: matches!(aim.fit, ViewFit::Framed { .. }),
    })
}

/// The orbit `view` opens on, or `None` when it carries no camera or one
/// whose direction and up coincide.
pub(super) fn saved_orbit(view: &ThreeDSavedView) -> Option<Orbit> {
    Orbit::saved(&saved_aim(view)?)
}

/// What `view` fixes of the framing in a picture of `aspect`, when it is an
/// orthographic view that fixes its own centre and scale.
pub(super) fn frame(view: Option<&ThreeDSavedView>, aspect: f64) -> Option<ViewFrame> {
    match view?.aim(aspect)?.fit {
        ViewFit::Framed { position, height } => Some(ViewFrame { position, height }),
        _ => None,
    }
}

impl ModelView {
    /// The framing the view being followed fixes at `aspect`.
    pub(super) fn frame(&self, aspect: f64) -> Option<ViewFrame> {
        frame(self.aimed.as_ref(), aspect)
    }

    /// Walk from `view`'s camera when it stands inside the model; whether it
    /// did.
    pub(super) fn walk_from_view(&mut self, view: &ThreeDSavedView) -> bool {
        let eye = saved_aim(view)
            .ok_or(pdfcer_3d_nav::NotInside::NoStandingPoint)
            .and_then(|aim| Ok((aim.eye_inside(&self.bounds)?, aim.direction)));
        let word = eye.as_ref().map_or_else(|why| why.as_str(), |_| "inside"); // ui-text-exempt: trace value
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("file-view-eye={word}"));
        let Ok((eye, direction)) = eye else {
            return false;
        };
        let axes = Axes::upright(self.upright);
        let mut walk = Walk::standing(eye, direction, axes, radius(&self.bounds));
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
