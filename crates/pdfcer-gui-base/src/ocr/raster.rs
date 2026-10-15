//! How a page becomes the raster a recogniser reads: the render options it is
//! drawn with, its size, and the optional enlargement before reading.
//!
//! Recognition reads the raster rather than showing it, so images drawn larger
//! than their own pixels are smoothed (`MagnifyFilter::Smooth`, the engine
//! CLI's OCR default) where the display keeps point sampling; PaddleOCR-VL
//! excepted (see [`magnify`]). Every other render option comes from the
//! operator's settings through the funnel.

use pdfcer_core::ocr::upscale::Upscale;
use pdfcer_core::page_tree::Page;
use pdfcer_core::settings::MagnifyFilter;
use pdfcer_render::RenderOptions;

/// The magnification filter `model`'s rasters are drawn with: smooth, except
/// for PaddleOCR-VL, which reads the display's point sampling. On a smoothed
/// scan VL falls into repeating one line and the engine's loop stop ends the
/// page minutes later with part of it read; point-sampled it reads the page in
/// seconds. It resizes every region itself, so smoothing first buys it
/// nothing.
#[must_use]
pub fn magnify(model: &pdfcer_core::ocr::addons::OcrModel) -> MagnifyFilter {
    // ui-text-exempt: an engine token, never displayed.
    if model.engine == "paddle-vl" {
        MagnifyFilter::PointSample
    } else {
        MagnifyFilter::Smooth
    }
}

/// `configured` (the settings funnel's options) as `model`'s raster draws them.
#[must_use]
pub fn raster_options(
    configured: RenderOptions,
    model: &pdfcer_core::ocr::addons::OcrModel,
) -> RenderOptions {
    configured.with_image_magnify(magnify(model))
}

/// The pixel size `page` is rasterised at for recognition, `(width, height)`;
/// `None` for a page with no area.
#[must_use]
pub fn raster_size(page: &Page) -> Option<(u32, u32)> {
    let b = page.crop_box;
    let (w, h) = (b.urx - b.llx, b.ury - b.lly);
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return None;
    }
    let scale = super::raster_scale(super::fitted_dpi(w, h));
    let (pw, ph, _) = pdfcer_render::page_device_geometry(page, scale);
    Some((pw, ph))
}

/// Whether `model` may be read through an enlargement: all but PaddleOCR-VL,
/// which resizes every region to its own model's size. Restates the rule
/// `OcrRunner::load` enforces with `RunnerError::UpscaleUnsupported`.
#[must_use]
pub fn can_upscale(model: &pdfcer_core::ocr::addons::OcrModel) -> bool {
    // ui-text-exempt: an engine token, never displayed.
    model.engine != "paddle-vl"
}

/// An enlargement a run applied, for the off-canvas report.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enlarged {
    /// What was asked of the engine.
    pub upscale: Upscale,
    /// The last page's raster, as rendered.
    pub from: (u32, u32),
    /// The same raster as the engine read it.
    pub to: (u32, u32),
}

impl Enlarged {
    /// The enlargement of a `from` raster, or `None` when `upscale` changes
    /// nothing.
    #[must_use]
    pub fn of(upscale: Option<Upscale>, from: (u32, u32)) -> Option<Self> {
        let upscale = upscale.filter(|u| !u.is_identity())?;
        Some(Self {
            upscale,
            from,
            to: upscale.output_size(from.0, from.1),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::ocr::upscale::UpscaleFilter;

    #[test]
    fn identity_is_no_enlargement() {
        let one = Upscale::new(1.0, UpscaleFilter::Bicubic).unwrap();
        assert_eq!(Enlarged::of(Some(one), (10, 20)), None);
        assert_eq!(Enlarged::of(None, (10, 20)), None);
    }

    #[test]
    fn enlargement_reports_both_sizes() {
        let three = Upscale::new(3.0, UpscaleFilter::Lanczos).unwrap();
        let e = Enlarged::of(Some(three), (100, 50)).unwrap();
        assert_eq!((e.from, e.to), ((100, 50), (300, 150)));
    }

    fn model(engine: &str) -> pdfcer_core::ocr::addons::OcrModel {
        pdfcer_core::ocr::addons::OcrModel {
            name: engine.to_owned(),
            engine: engine.to_owned(),
            folder: std::path::PathBuf::new(),
            root: std::path::PathBuf::new(),
            manifest: None,
        }
    }

    #[test]
    fn ocr_rasters_magnify_smoothly_but_for_vl() {
        assert_eq!(magnify(&model("ocrs")), MagnifyFilter::Smooth);
        assert_eq!(magnify(&model("paddle")), MagnifyFilter::Smooth);
        assert_eq!(magnify(&model("paddle-vl")), MagnifyFilter::PointSample);
    }
}
