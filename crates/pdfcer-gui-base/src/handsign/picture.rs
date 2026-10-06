//! # `handsign::picture` — a signature from a picture file
//!
//! Contract: a [`SigPicture`] carries the file's bytes unchanged and whether
//! its white background is to be clear; [`SigPicture::image`] imports it
//! through `pdfcer_core::image_import::import` every time, so what is placed
//! is what the engine decoded. A picture with its own transparency keeps it.
//! White is cleared with a colour-key `/Mask` (ISO 32000-1 §8.9.6.4), which
//! the engine carries on `ImportedImage::color_key_mask`; it is offered only
//! for grey and RGB samples with no transparency of their own
//! ([`can_clear_white`]).
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/handsign/picture.md`.

use std::sync::Arc;

use egui::{Vec2, vec2};
use pdfcer_core::image_import::{ImportColorSpace, ImportedImage};

/// The file a remembered picture signature is kept in, beside `settings.txt`.
pub const PICTURE_FILE: &str = "hand-signature-picture.bin"; // ui-text-exempt: a file name, never displayed as copy

/// The first line of the kept file; the picture's bytes follow it.
const HEADER: &[u8] = b"pdfcer-hand-signature-picture clear-white="; // ui-text-exempt: an on-disk header, never displayed

/// A sample at or above this fraction of full scale, in every component,
/// counts as white: a scan's or a JPEG's paper is rarely exactly 255.
const WHITE_FROM: f64 = 0.92;

/// A picture of a signature, as the operator chose it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigPicture {
    /// The file's bytes.
    pub bytes: Arc<Vec<u8>>,
    /// Whether white is made clear.
    pub clear_white: bool,
}

impl SigPicture {
    /// The picture as the engine imports it, white cleared when asked and
    /// possible.
    ///
    /// # Errors
    ///
    /// The importer's own sentence.
    pub fn image(&self) -> Result<ImportedImage, String> {
        let mut image =
            pdfcer_core::image_import::import(&self.bytes).map_err(|e| e.to_string())?;
        if self.clear_white {
            image.color_key_mask = white_key(&image);
        }
        Ok(image)
    }

    /// The kept-file form: the header line, then the bytes.
    #[must_use]
    pub fn to_kept(&self) -> Vec<u8> {
        let mut out = HEADER.to_vec();
        out.push(if self.clear_white { b'1' } else { b'0' });
        out.push(b'\n');
        out.extend_from_slice(&self.bytes);
        out
    }

    /// Read [`Self::to_kept`]'s form; `None` for anything else.
    #[must_use]
    pub fn from_kept(kept: &[u8]) -> Option<Self> {
        let rest = kept.strip_prefix(HEADER)?;
        let (flag, bytes) = match rest {
            [b'0', b'\n', bytes @ ..] => (false, bytes),
            [b'1', b'\n', bytes @ ..] => (true, bytes),
            _ => return None,
        };
        (!bytes.is_empty()).then(|| Self {
            bytes: Arc::new(bytes.to_vec()),
            clear_white: flag,
        })
    }
}

/// Whether white can be made clear in `image`: grey or RGB samples, with no
/// transparency of their own.
#[must_use]
pub fn can_clear_white(image: &ImportedImage) -> bool {
    white_key(image).is_some()
}

/// The colour-key range that clears near-white, in source sample space.
fn white_key(image: &ImportedImage) -> Option<Vec<i64>> {
    if image.soft_mask.is_some() || image.color_key_mask.is_some() {
        return None;
    }
    let components = match image.color_space {
        ImportColorSpace::DeviceGray => 1,
        ImportColorSpace::DeviceRgb => 3,
        _ => return None,
    };
    let max = (1_i64 << image.bits_per_component.min(16)) - 1;
    #[allow(
        clippy::cast_possible_truncation,
        reason = "max is at most 65535, exact in f64 and back" // ui-text-exempt: a lint justification
    )]
    let from = (max as f64 * WHITE_FROM).round() as i64;
    Some([from, max].repeat(components))
}

/// The picture's proportions as it appears: its displayed pixel size.
#[must_use]
pub fn ink_size(image: &ImportedImage) -> Vec2 {
    let (w, h) = image.display_size_px();
    #[allow(
        clippy::cast_precision_loss,
        reason = "a pixel count is far below f32's exact range" // ui-text-exempt: a lint justification
    )]
    vec2(w as f32, h as f32)
}

/// The largest side of [`preview`]'s raster, in pixels.
pub const PREVIEW_PX: f32 = 480.0;

/// `image` as the engine draws it under the operator's `options`, on a
/// see-through page no larger than [`PREVIEW_PX`] on its longer side: the
/// size in pixels and premultiplied RGBA. The engine draws it, so a cleared
/// white shows as cleared.
///
/// # Errors
///
/// The engine's sentence when the picture cannot be placed or drawn.
pub fn preview(
    image: &ImportedImage,
    options: &pdfcer_render::RenderOptions,
) -> Result<([usize; 2], Vec<u8>), String> {
    let doc = pdfcer_core::document::Document::from_bytes(crate::blank::picture_page(image)?)
        .map_err(|e| e.to_string())?;
    let pages = pdfcer_core::page_tree::pages(&doc).map_err(|e| e.to_string())?;
    let page = pages.first().ok_or_else(String::new)?;
    let (w, h) = image.natural_size_pt();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a page side in points is far inside f32's range" // ui-text-exempt: a lint justification
    )]
    let longest = w.max(h).max(1.0) as f32;
    let mut options = options.clone();
    options.backdrop = pdfcer_render::PageBackdrop::Transparent;
    let rendered = pdfcer_render::render_page_with(&doc, page, PREVIEW_PX / longest, &options)
        .map_err(|e| e.to_string())?;
    let pix = rendered.pixmap;
    Ok((
        [pix.width() as usize, pix.height() as usize],
        pix.data().to_vec(),
    ))
}

/// The remembered picture signature, if one is kept and readable.
#[must_use]
pub fn load() -> Option<SigPicture> {
    SigPicture::from_kept(&std::fs::read(super::kept_path(PICTURE_FILE)?).ok()?)
}

/// Keep `picture` on this computer. Returns whether it was written.
pub fn save(picture: &SigPicture) -> bool {
    super::keep(PICTURE_FILE, &picture.to_kept())
}

/// Delete the remembered picture signature. Absent already is success.
pub fn forget() -> bool {
    super::unkeep(PICTURE_FILE)
}

#[cfg(test)]
mod tests;
