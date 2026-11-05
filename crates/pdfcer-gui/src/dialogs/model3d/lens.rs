//! The lens the picture is drawn through: flat (rectilinear), fisheye, or
//! Auto, which is flat until the vertical field passes [`Lens::auto_at`] and
//! fisheye beyond it, as a phone camera changes lens when zoomed out.
//!
//! A fisheye's field is measured across the picture's height, as a flat
//! one's is, so changing lens at a field keeps what touches the picture's
//! top and bottom edges where it was; only the bend between differs.
//! Orthographic views have no lens and pass through unchanged.

use pdfcer_3d::{Camera, FisheyeMapping, Projection};

/// The widest vertical field a flat picture reaches walking, degrees.
pub(super) const FLAT_WIDEST: f64 = 100.0;
/// The widest vertical field a fisheye reaches walking, degrees: a half
/// sphere top to bottom.
pub(super) const FISHEYE_WIDEST: f64 = 180.0;
/// The range [`Lens::auto_at`] is chosen in, degrees. It stops at
/// [`FLAT_WIDEST`]: past it a flat picture no longer widens, so a later
/// change point would leave zooming out doing nothing until it.
pub(super) const AUTO_RANGE: std::ops::RangeInclusive<f64> = 40.0..=FLAT_WIDEST;

/// Which lens is chosen; the order is the preferences file's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum LensKind {
    #[default]
    Flat,
    Fisheye,
    Auto,
}

/// Every [`LensKind`], in the drop-down's and the preferences file's order.
pub(super) const KINDS: [LensKind; 3] = [LensKind::Flat, LensKind::Fisheye, LensKind::Auto];
/// Every fisheye curve offered, in the same sense.
pub(super) const CURVES: [FisheyeMapping; 2] =
    [FisheyeMapping::Stereographic, FisheyeMapping::Equidistant];

/// The operator's lens choice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Lens {
    pub kind: LensKind,
    /// The fisheye's curve, for Fisheye and for Auto past its change point.
    pub curve: FisheyeMapping,
    /// Auto's change point: the vertical field, degrees, past which it
    /// draws a fisheye.
    pub auto_at: f64,
}

impl Default for Lens {
    fn default() -> Self {
        Self {
            kind: LensKind::Flat,
            curve: FisheyeMapping::Stereographic,
            auto_at: 90.0,
        }
    }
}

impl Lens {
    /// Whether a picture `fov` degrees tall is drawn as a fisheye.
    pub fn fisheye_at(&self, fov: f64) -> bool {
        match self.kind {
            LensKind::Flat => false,
            LensKind::Fisheye => true,
            LensKind::Auto => fov > self.auto_at,
        }
    }

    /// The widest vertical field walking zooms out to under this lens.
    pub fn widest(&self) -> f64 {
        match self.kind {
            LensKind::Flat => FLAT_WIDEST,
            LensKind::Fisheye | LensKind::Auto => FISHEYE_WIDEST,
        }
    }

    /// The projection for a vertical field of `fov` degrees.
    pub fn projection(&self, fov: f64) -> Projection {
        if self.fisheye_at(fov) {
            Projection::Fisheye {
                fov,
                mapping: self.curve,
            }
        } else {
            Projection::Perspective { fov_y: fov }
        }
    }

    /// `camera` drawn through this lens: a perspective turned fisheye where
    /// the lens draws one at its field; anything else as it was.
    pub fn applied(&self, mut camera: Camera) -> Camera {
        if let Projection::Perspective { fov_y } = camera.projection {
            camera.projection = self.projection(fov_y);
        }
        camera
    }
}

/// Whether `camera` is drawn through a fisheye.
pub(super) fn is_fisheye(camera: &Camera) -> bool {
    matches!(camera.projection, Projection::Fisheye { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(fov_y: f64) -> Camera {
        Camera {
            eye: [0.0, 0.0, 5.0],
            target: [0.0; 3],
            up: [0.0, 1.0, 0.0],
            projection: Projection::Perspective { fov_y },
        }
    }

    #[test]
    fn auto_turns_fisheye_only_past_its_change_point() {
        let lens = Lens {
            kind: LensKind::Auto,
            ..Lens::default()
        };
        assert!(!is_fisheye(&lens.applied(camera(lens.auto_at))));
        let wide = lens.applied(camera(lens.auto_at + 1.0));
        assert!(matches!(
            wide.projection,
            Projection::Fisheye { fov, mapping: FisheyeMapping::Stereographic }
                if (fov - lens.auto_at - 1.0).abs() < 1e-12
        ));
    }

    #[test]
    fn flat_and_orthographic_pass_through() {
        let flat = Lens::default();
        assert_eq!(flat.applied(camera(60.0)), camera(60.0));
        let fisheye = Lens {
            kind: LensKind::Fisheye,
            ..Lens::default()
        };
        let mut ortho = camera(60.0);
        ortho.projection = Projection::Orthographic { height: 3.0 };
        assert_eq!(fisheye.applied(ortho), ortho);
        assert!(is_fisheye(&fisheye.applied(camera(30.0))));
    }

    #[test]
    fn only_a_fisheye_lens_zooms_past_the_flat_limit() {
        assert!((Lens::default().widest() - FLAT_WIDEST).abs() < 1e-12);
        for kind in [LensKind::Fisheye, LensKind::Auto] {
            let lens = Lens {
                kind,
                ..Lens::default()
            };
            assert!((lens.widest() - FISHEYE_WIDEST).abs() < 1e-12);
        }
    }
}
