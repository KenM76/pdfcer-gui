//! The picture a visible signature is drawn with
//! (`SignRequest::appearance_image`), read from a PNG or JPEG file.

use std::path::Path;
use std::sync::Arc;

use pdfcer_core::image_import::ImportedImage;

/// An imported picture, shared between the window and the request, equal
/// only to itself.
#[derive(Clone)]
pub struct AppearancePicture(Arc<ImportedImage>);

impl AppearancePicture {
    /// Read and import `path`, or the reason it is not a picture the engine
    /// can place.
    ///
    /// # Errors
    /// The file system's or the importer's message.
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        pdfcer_core::image_import::import(&bytes)
            .map(|image| Self(Arc::new(image)))
            .map_err(|e| e.to_string())
    }

    /// The engine's image.
    #[must_use]
    pub fn image(&self) -> &ImportedImage {
        &self.0
    }

    /// Width and height in pixels.
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.0.width, self.0.height)
    }
}

impl PartialEq for AppearancePicture {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for AppearancePicture {}

impl std::fmt::Debug for AppearancePicture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (w, h) = self.size();
        write!(f, "AppearancePicture({w}x{h})")
    }
}
