//! The sizes of a 360° picture: the whole sphere round the view's eye, for a
//! photo viewer to look around in (`Camera::levelled_panorama`, tagged by
//! `pdfcer_render::export::photo_sphere_xmp`).

/// The sizes offered, pixels, smallest first: 2:1, as the full sphere is.
pub(super) const SIZES: [[u32; 2]; 3] = [[2048, 1024], [4096, 2048], [8192, 4096]];

#[cfg(test)]
mod tests {
    use super::*;

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
