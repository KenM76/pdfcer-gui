//! A 360° picture: the whole sphere round the view's eye, for a photo viewer
//! to look around in (`FisheyeMapping::Equirectangular`, tagged by
//! `pdfcer_render::export::photo_sphere_xmp`).
//!
//! A photo viewer takes the picture's top edge as straight up, so the camera
//! is levelled onto the model's up: left pitched, the horizon would rise and
//! fall as the operator looks round.

use pdfcer_3d::vec3::{dot, normalize, sub};
use pdfcer_3d::{Camera, FisheyeMapping, Projection};
use pdfcer_3d_nav::Upright;

/// The sizes offered, pixels, smallest first: 2:1, as the full sphere is.
pub(super) const SIZES: [[u32; 2]; 3] = [[2048, 1024], [4096, 2048], [8192, 4096]];

/// `camera` through the 360° lens from the same eye, keeping its heading round
/// `upright`'s up; a camera looking along that up takes the model's front.
pub(super) fn levelled(mut camera: Camera, upright: Upright) -> Camera {
    let (up, front) = (upright.up.vector(), upright.front.vector());
    let look = sub(camera.target, camera.eye);
    let gap = dot(look, look).sqrt();
    let along = dot(look, up);
    let flat: [f64; 3] = std::array::from_fn(|i| look[i] - up[i] * along);
    let ahead = if dot(flat, flat).sqrt() > gap * 1e-6 {
        normalize(flat).unwrap_or(front)
    } else {
        front
    };
    camera.target = std::array::from_fn(|i| camera.eye[i] + ahead[i] * gap);
    camera.up = up;
    camera.projection = Projection::Fisheye {
        fov: 360.0,
        mapping: FisheyeMapping::Equirectangular,
    };
    camera
}

#[cfg(test)]
mod tests {
    use super::*;

    const EYE: [f64; 3] = [1.0, 2.0, 3.0];

    fn camera(look: [f64; 3]) -> Camera {
        Camera {
            eye: EYE,
            target: std::array::from_fn(|i| EYE[i] + look[i]),
            up: [0.3, 0.0, 0.95],
            projection: Projection::Perspective { fov_y: 40.0 },
        }
    }

    #[test]
    fn a_pitched_view_is_levelled_and_keeps_its_heading_and_eye() {
        let upright = Upright::default();
        let up = upright.up.vector();
        let front = upright.front.vector();
        let look = std::array::from_fn(|i| front[i] * 3.0 - up[i] * 4.0);
        let done = levelled(camera(look), upright);
        assert_eq!(done.eye, EYE);
        assert_eq!(done.up, up);
        let d = sub(done.target, done.eye);
        assert!(dot(d, up).abs() < 1e-12, "{d:?}");
        assert!((dot(d, front) - 5.0).abs() < 1e-12, "{d:?}");
        assert!(matches!(
            done.projection,
            Projection::Fisheye { fov, mapping: FisheyeMapping::Equirectangular } if fov == 360.0
        ));
    }

    #[test]
    fn a_view_straight_down_looks_to_the_front() {
        let upright = Upright::default();
        let down = upright.up.vector().map(|c| -c * 2.0);
        let done = levelled(camera(down), upright);
        let d = sub(done.target, done.eye);
        assert!(
            (dot(d, upright.front.vector()) - 2.0).abs() < 1e-12,
            "{d:?}"
        );
    }

    #[test]
    fn the_sizes_are_two_to_one_within_the_ceiling_and_named_as_they_are() {
        for (i, [w, h]) in SIZES.into_iter().enumerate() {
            assert_eq!(w, h * 2);
            let name = crate::text::model3dshot::sphere_size(i);
            assert!(name.starts_with(&format!("{w} × {h} ")), "{name}");
            assert!(u64::from(w) * u64::from(h) <= pdfcer_3d::MAX_RENDER_PIXELS);
        }
    }
}
