//! What another program left on the clipboard, read as something a page can
//! take.
//!
//! Contract: [`read`] prefers a drawing (registered `application/pdf`, as a
//! snapshot copy places it) to a picture, and a picture to text; among
//! pictures the registered `PNG` format (lossless, carries alpha) over
//! `CF_DIBV5` over `CF_DIB`. A PDF that does not parse, or whose first page has
//! no area, is passed over for the picture beside it.
//! A picture that states no resolution is taken at 96 pixels per
//! inch, the resolution screen captures are made at, so a pasted screenshot
//! lands at the size it had on screen. [`rect_at`] is the placement rule for
//! a picture and [`textbox`] for text.

pub mod dib;
pub mod page;
pub mod textbox;

use pdfcer_core::image_import::ImportedImage;
use pdfcer_core::page_tree::Rect;

/// The resolution assumed for a clipboard picture that states none.
pub const SCREEN_DPI: f64 = 96.0;

/// The clipboard's contents, as far as pasting onto a page is concerned.
#[derive(Debug, Clone)]
pub enum Incoming {
    /// A picture, decoded and ready to place.
    Image {
        /// The picture.
        image: Box<ImportedImage>,
        /// The clipboard format it came from, for the trace.
        format: &'static str,
    },
    /// A one-page drawing: PDF bytes, and the first page's crop box width and
    /// height in points, unrotated, as a stamp's artwork draws it.
    Pdf {
        /// The document.
        bytes: Vec<u8>,
        /// The first page's crop box size, in points.
        size_pt: (f64, f64),
    },
    /// A picture that could not be decoded, and why.
    Unreadable(String),
    /// Text, and no picture.
    Text(String),
    /// Nothing pdfcer reads.
    Nothing,
}

/// The clipboard's change counter; it differs whenever any program has
/// written the clipboard since it was last read. Always 0 off Windows.
#[must_use]
pub fn sequence() -> u32 {
    native_clipboard::sequence()
}

/// Read the clipboard.
#[must_use]
pub fn read() -> Incoming {
    use native_clipboard::{CF_DIB, CF_DIBV5, CF_UNICODETEXT, Format, get};
    let pdf_format = Format::Registered(crate::clipboard::ClipFormat::Pdf.name());
    if let Some(drawing) = get(pdf_format).and_then(drawing) {
        return drawing;
    }
    let png_format = Format::Registered("PNG"); // ui-text-exempt: a Windows clipboard format name
    if let Some(png) = get(png_format) {
        return finish(
            pdfcer_core::image_import::import(&png).map_err(|e| e.to_string()),
            "png",
        );
    }
    for (id, name) in [(CF_DIBV5, "dibv5"), (CF_DIB, "dib")] {
        if let Some(bytes) = get(Format::Predefined(id)) {
            return finish(image_from_dib(&bytes), name);
        }
    }
    get(Format::Predefined(CF_UNICODETEXT))
        .map(|wide| text_from_utf16(&wide))
        .filter(|t| !t.trim().is_empty())
        .map_or(Incoming::Nothing, Incoming::Text)
}

/// The selection another pdfcer-gui window copied, as
/// `ObjectClip::to_bytes`: from the capture folder when a driven check set
/// one, else from the OS clipboard's private format.
#[must_use]
pub fn own_clip() -> Option<Vec<u8>> {
    use crate::clipimage::OBJECT_CLIP_FORMAT;
    if std::env::var_os(crate::clipboard::place::DIAG_CLIPBOARD_DIR).is_some() {
        return crate::clipboard::place::captured(OBJECT_CLIP_FORMAT);
    }
    native_clipboard::get(native_clipboard::Format::Registered(OBJECT_CLIP_FORMAT))
}

/// PDF bytes as a drawing, when they parse and their first page has area.
fn drawing(bytes: Vec<u8>) -> Option<Incoming> {
    let (_, pages) = page::opened(bytes.clone()).ok()?;
    let crop = pages.first()?.crop_box;
    let size_pt = (crop.width(), crop.height());
    (size_pt.0 > 0.0 && size_pt.1 > 0.0).then_some(Incoming::Pdf { bytes, size_pt })
}

fn finish(image: Result<ImportedImage, String>, format: &'static str) -> Incoming {
    match image {
        Ok(mut image) => {
            if image.dpi.is_none() {
                image.dpi = Some((SCREEN_DPI, SCREEN_DPI));
            }
            Incoming::Image {
                image: Box::new(image),
                format,
            }
        }
        Err(why) => Incoming::Unreadable(why),
    }
}

/// `CF_UNICODETEXT` bytes to a string, stopping at the terminating NUL.
#[must_use]
pub fn text_from_utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|u| *u != 0)
        .collect();
    String::from_utf16_lossy(&units)
}

/// A clipboard bitmap as an imported picture, its samples handed straight to
/// `ImportedImage::from_rgba8`; the bitmap's stated resolution, when it has
/// one, is its `dpi` on both axes.
///
/// # Errors
/// A sentence naming why the bitmap could not be read.
pub fn image_from_dib(bytes: &[u8]) -> Result<ImportedImage, String> {
    let rgba = dib::decode(bytes).map_err(|e| crate::text::ospaste::dib_error(e).to_owned())?;
    let mut image = ImportedImage::from_rgba8(rgba.width, rgba.height, &rgba.pixels)
        .map_err(|e| e.to_string())?;
    image.dpi = rgba.dpi.map(|d| (f64::from(d), f64::from(d)));
    Ok(image)
}

/// Where a picture of `natural` points lands: centred on `at`, scaled down
/// (shape kept) to fit `page` if larger, then slid wholly onto `page`.
#[must_use]
pub fn rect_at(at: (f64, f64), natural: (f64, f64), page: Rect) -> Rect {
    let (pw, ph) = (page.width(), page.height());
    let (nw, nh) = natural;
    let scale = if nw > 0.0 && nh > 0.0 {
        (pw / nw).min(ph / nh).min(1.0)
    } else {
        1.0
    };
    let (w, h) = (nw * scale, nh * scale);
    let llx = (at.0 - w / 2.0).clamp(page.llx, (page.urx - w).max(page.llx));
    let lly = (at.1 - h / 2.0).clamp(page.lly, (page.ury - h).max(page.lly));
    Rect::from_corners(llx, lly, llx + w, lly + h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Rect {
        Rect::from_corners(0.0, 0.0, 612.0, 792.0)
    }

    #[test]
    fn a_picture_lands_centred_on_the_point_at_its_natural_size() {
        let r = rect_at((300.0, 400.0), (48.0, 24.0), page());
        assert_eq!((r.llx, r.lly, r.urx, r.ury), (276.0, 388.0, 324.0, 412.0));
    }

    #[test]
    fn a_picture_near_an_edge_is_slid_onto_the_page() {
        let r = rect_at((5.0, 790.0), (48.0, 24.0), page());
        assert_eq!((r.llx, r.ury), (0.0, 792.0));
    }

    #[test]
    fn a_picture_larger_than_the_page_is_shrunk_with_its_shape_kept() {
        let r = rect_at((306.0, 396.0), (1224.0, 396.0), page());
        assert!((r.width() - 612.0).abs() < 1e-9 && (r.height() - 198.0).abs() < 1e-9);
    }

    #[test]
    fn utf16_text_stops_at_its_terminator() {
        let bytes: Vec<u8> = "Hi\0junk"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(text_from_utf16(&bytes), "Hi");
    }

    #[test]
    fn a_dib_becomes_a_picture_at_its_stated_resolution() {
        let mut b = Vec::new();
        for v in [40u32, 64, 32] {
            b.extend(v.to_le_bytes());
        }
        b.extend(1u16.to_le_bytes());
        b.extend(32u16.to_le_bytes());
        for v in [0u32, 0, 3780, 3780, 0, 0] {
            b.extend(v.to_le_bytes());
        }
        b.resize(40 + 64 * 32 * 4, 200);
        let image = image_from_dib(&b).unwrap();
        assert_eq!((image.width, image.height), (64, 32));
        let (w, h) = image.natural_size_pt();
        assert!((w - 48.0).abs() < 0.1 && (h - 24.0).abs() < 0.1, "{w}x{h}");
    }
}
