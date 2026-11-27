//! The Picture tab's panorama: a band of the sphere round the view's eye, an
//! angle across by an angle up and down, drawn level or tilted with the view
//! (`Camera::panorama`) and saved as a JPEG. An equirectangular band is
//! tagged for photo viewers with its place on the sphere
//! (`pdfcer_render::export::partial_photo_sphere_xmp`); a cylindrical one has
//! no tag a viewer reads, and the tab says so. 360° by 180° is the whole
//! sphere.

use pdfcer_3d::{Camera, PanoramaExtent, PanoramaMapping, PanoramaTilt, Projection};

/// The longer side offered, pixels, smallest first.
pub(super) const LONG_SIDES: [u32; 3] = [2048, 4096, 8192];
/// The angle across, degrees.
pub(super) const ACROSS: std::ops::RangeInclusive<f64> = 1.0..=360.0;
/// The angle up and down, degrees; a cylindrical band stops short of the
/// poles, where its rows run to infinity.
pub(super) const UP_DOWN: std::ops::RangeInclusive<f64> = 1.0..=180.0;
/// The tallest cylindrical band, degrees.
pub(super) const UP_DOWN_CYLINDRICAL: f64 = 170.0;

/// What part of the sphere a panorama shows and how it is laid out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Pano {
    /// Degrees across, round the world's up.
    pub(super) across: f64,
    /// Degrees up and down, when not the view's own.
    pub(super) up_down: f64,
    /// The angle up and down is the view's own.
    pub(super) from_view: bool,
    /// The band is centred on the view's line of sight, not the horizon.
    pub(super) tilted: bool,
    /// Rows evenly spaced in tan(latitude) rather than in latitude.
    pub(super) cylindrical: bool,
}

impl Default for Pano {
    fn default() -> Self {
        Self {
            across: 360.0,
            up_down: 180.0,
            from_view: false,
            tilted: false,
            cylindrical: false,
        }
    }
}

/// The band a camera was turned into: what the picture holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Band {
    pub(super) across: f64,
    pub(super) up_down: f64,
    /// The band's middle, degrees above the horizon.
    pub(super) pitch: f64,
    pub(super) cylindrical: bool,
}

impl Pano {
    /// The extent `Camera::panorama` takes.
    pub(super) fn extent(&self) -> PanoramaExtent {
        PanoramaExtent {
            horizontal: self.across,
            vertical: (!self.from_view).then_some(self.up_down),
            tilt: if self.tilted {
                PanoramaTilt::View
            } else {
                PanoramaTilt::Level
            },
            mapping: if self.cylindrical {
                PanoramaMapping::Cylindrical
            } else {
                PanoramaMapping::Equirectangular
            },
        }
    }

    /// `view` turned into this panorama about `up`, heading `front`.
    pub(super) fn camera(
        &self,
        view: Camera,
        up: [f64; 3],
        front: [f64; 3],
    ) -> Result<(Camera, Band), pdfcer_3d::RenderError> {
        let camera = view.panorama(up, front, self.extent())?;
        let band = band_of(&camera).ok_or(pdfcer_3d::RenderError::Camera(
            "the panorama camera holds no band", // ui-text-exempt: engine-contract breach, never reached
        ))?;
        Ok((camera, band))
    }
}

/// The band `camera` draws, when it is a panorama.
pub(super) fn band_of(camera: &Camera) -> Option<Band> {
    match camera.projection {
        Projection::Panorama {
            horizontal,
            vertical,
            pitch,
            mapping,
        } => Some(Band {
            across: horizontal,
            up_down: vertical,
            pitch,
            cylindrical: mapping == PanoramaMapping::Cylindrical,
        }),
        _ => None,
    }
}

impl Band {
    /// Width over height for square pixels: the angles' ratio, or for a
    /// cylindrical band the across angle in radians over the rows' span in
    /// tan(latitude), square at the horizon.
    pub(super) fn ratio(&self) -> f64 {
        if self.cylindrical {
            self.across.to_radians() / (2.0 * (self.up_down / 2.0).to_radians().tan())
        } else {
            self.across / self.up_down
        }
    }

    /// The picture's size with `long` pixels on its longer side, square
    /// pixels, inside the render ceiling.
    pub(super) fn size(&self, long: u32) -> [u32; 2] {
        let ratio = self.ratio().max(1e-6);
        let long = f64::from(long);
        let (mut w, mut h) = if ratio >= 1.0 {
            (long, long / ratio)
        } else {
            (long * ratio, long)
        };
        let most = pdfcer_3d::MAX_RENDER_PIXELS as f64;
        if w * h > most {
            let k = (most / (w * h)).sqrt();
            w *= k;
            h *= k;
        }
        [w.round().max(1.0) as u32, h.round().max(1.0) as u32]
    }

    /// The band was moved to stop at a pole: its middle sits as far from
    /// the horizon as its height allows.
    pub(super) fn at_pole(&self) -> bool {
        self.pitch != 0.0 && (self.pitch.abs() - (90.0 - self.up_down / 2.0)).abs() < 1e-9
    }

    /// The photo-sphere tag for a `width` × `height` picture of this band;
    /// `None` for a cylindrical band, which photo viewers have no type for.
    pub(super) fn tag(&self, width: u32, height: u32) -> Option<String> {
        if self.cylindrical {
            return None;
        }
        pdfcer_render::export::partial_photo_sphere_xmp(
            width,
            height,
            self.across,
            self.up_down,
            self.pitch,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn band(across: f64, up_down: f64, cylindrical: bool) -> Band {
        Band {
            across,
            up_down,
            pitch: 0.0,
            cylindrical,
        }
    }

    #[test]
    fn the_whole_sphere_keeps_the_old_sizes() {
        let whole = band(360.0, 180.0, false);
        assert_eq!(whole.size(2048), [2048, 1024]);
        assert_eq!(whole.size(4096), [4096, 2048]);
        assert_eq!(whole.size(8192), [8192, 4096]);
    }

    #[test]
    fn a_strip_is_sized_to_its_angles_and_inside_the_ceiling() {
        assert_eq!(band(120.0, 40.0, false).size(2048), [2048, 683]);
        assert_eq!(band(40.0, 120.0, false).size(2048), [683, 2048]);
        let [w, h] = band(90.0, 90.0, false).size(16_384);
        assert!(u64::from(w) * u64::from(h) <= pdfcer_3d::MAX_RENDER_PIXELS);
        // Square at the horizon: 90° across is π/2, 90° up and down 2·tan 45°.
        let [w, h] = band(90.0, 90.0, true).size(2000);
        assert_eq!([w, h], [1571, 2000]);
    }

    #[test]
    fn a_cylindrical_band_is_untagged_and_an_equirectangular_one_tagged() {
        assert!(band(120.0, 40.0, true).tag(2048, 683).is_none());
        let tag = band(120.0, 40.0, false).tag(2048, 683).expect("tagged");
        assert!(tag.contains("FullPanoWidthPixels"), "{tag}");
    }

    #[test]
    fn a_band_at_the_pole_is_told_from_a_level_one() {
        let mut b = band(360.0, 60.0, false);
        assert!(!b.at_pole());
        b.pitch = 60.0;
        assert!(b.at_pole());
        b.pitch = -60.0;
        assert!(b.at_pole());
        b.pitch = 20.0;
        assert!(!b.at_pole());
    }

    #[test]
    fn the_long_side_names_match_the_sizes() {
        for (i, long) in LONG_SIDES.into_iter().enumerate() {
            let name = crate::text::model3dshot::sphere_size(i);
            assert!(name.starts_with(&long.to_string()), "{name}");
        }
    }
}
