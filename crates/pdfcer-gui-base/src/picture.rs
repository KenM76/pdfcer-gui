//! `picture` — a file the operator chose to put on a page: a raster picture,
//! or a vector drawing that stays vector.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/picture.md`.

use std::path::Path;

use pdfcer_core::emf_import::ImportedEmf;
use pdfcer_core::image_import::ImportedImage;
#[cfg(feature = "svg-import")]
use pdfcer_core::svg_import::ImportedSvg;

/// Every extension the picker offers and a drop accepts as a picture; `svg`
/// and `emf` are read as vector drawings. A build without `svg-import` does
/// not offer `svg`.
#[cfg(feature = "svg-import")]
pub const EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "bmp", "gif", "tif", "tiff", "svg", "emf",
];
/// See the `svg-import` twin.
#[cfg(not(feature = "svg-import"))]
pub const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "gif", "tif", "tiff", "emf"];

/// One imported picture, ready for the engine's placement verb.
#[derive(Debug, Clone, PartialEq)]
pub enum Picture {
    /// PNG, JPEG, BMP, GIF or TIFF, placed by `EditSession::add_image`.
    Raster(ImportedImage),
    /// An SVG drawing, placed by `EditSession::add_svg`.
    #[cfg(feature = "svg-import")]
    Svg(ImportedSvg),
    /// A Windows enhanced metafile, placed by `EditSession::add_emf`.
    Emf(ImportedEmf),
}

impl Picture {
    /// Import `bytes`, choosing the importer by `path`'s extension: `.svg`
    /// and `.emf` are drawings, anything else goes to the raster importer,
    /// which recognises its formats by their signatures.
    ///
    /// # Errors
    ///
    /// The importer's own error, as text for the operator.
    pub fn import(path: &Path, bytes: &[u8]) -> Result<Self, String> {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase());
        match ext.as_deref() {
            #[cfg(feature = "svg-import")]
            Some("svg") => pdfcer_core::svg_import::import(bytes)
                .map(Self::Svg)
                .map_err(|e| e.to_string()),
            Some("emf") => pdfcer_core::emf_import::import(bytes)
                .map(Self::Emf)
                .map_err(|e| e.to_string()),
            _ => pdfcer_core::image_import::import(bytes)
                .map(Self::Raster)
                .map_err(|e| e.to_string()),
        }
    }

    /// The size it is drawn at unless told otherwise, in points: a raster's
    /// declared resolution, an SVG at 96 px per inch, an EMF's own frame.
    #[must_use]
    pub fn natural_size_pt(&self) -> (f64, f64) {
        match self {
            Self::Raster(image) => image.natural_size_pt(),
            #[cfg(feature = "svg-import")]
            Self::Svg(svg) => svg.natural_size_pt(),
            Self::Emf(emf) => emf.natural_size_pt(),
        }
    }

    /// The raster picture, when it is one.
    #[must_use]
    pub const fn raster(&self) -> Option<&ImportedImage> {
        match self {
            Self::Raster(image) => Some(image),
            _ => None,
        }
    }

    /// The token the trace and the apply arm's label use.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Raster(_) => "image",
            #[cfg(feature = "svg-import")]
            Self::Svg(_) => "svg",
            Self::Emf(_) => "emf",
        }
    }

    /// What the import of a drawing skipped or approximated, in the engine's
    /// operator line; `None` for a raster or a drawing carried exactly.
    #[must_use]
    pub fn drawing_notes(&self) -> Option<String> {
        let summary = match self {
            Self::Raster(_) => return None,
            #[cfg(feature = "svg-import")]
            Self::Svg(svg) => svg.notes().summary(),
            Self::Emf(emf) => emf.notes().summary(),
        };
        (!summary.is_empty()).then_some(summary)
    }
}

#[cfg(test)]
mod tests {
    #![cfg(feature = "svg-import")]

    use super::*;

    const SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
        <rect width="200" height="100" fill="red"/><text x="5" y="50">A</text></svg>"#;

    #[test]
    fn the_extension_picks_the_importer_without_regard_to_case() {
        let picture = Picture::import(Path::new("C:/x/LOGO.SVG"), SVG).expect("an SVG imports");
        assert_eq!(picture.kind(), "svg");
        assert!(picture.raster().is_none());
        assert_eq!(picture.natural_size_pt(), (150.0, 75.0));
        assert!(
            Picture::import(Path::new("logo.png"), SVG).is_err(),
            "a .png holding SVG goes to the raster importer, which refuses it"
        );
    }

    #[test]
    fn a_skipped_svg_feature_is_reported_and_a_raster_reports_nothing() {
        let picture = Picture::import(Path::new("a.svg"), SVG).expect("an SVG imports");
        let notes = picture.drawing_notes().expect("the <text> is not carried");
        assert!(notes.contains("text"), "{notes}");
    }
}
