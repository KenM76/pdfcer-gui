//! # `app::dispatch::images` — choosing a picture, reading it, and refusing it
//! by name
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/images.md`.

use crate::app::state::Status;
use crate::dialogs::DialogsState;
use pdfcer_core::image_import::ImportedImage;

/// The whole of `edit.insert_image` after its capability guard.
pub(super) fn insert(dialogs: &mut DialogsState, status: &Status) {
    let crate::app::files::Picked::Path(path) = crate::app::files::pick_image_source() else {
        return;
    };
    insert_path(dialogs, status, &path);
}

/// Import the file at `path` and open the placement window.
pub(crate) fn insert_path(dialogs: &mut DialogsState, status: &Status, path: &std::path::Path) {
    let Some(image) = import(status, path) else {
        return;
    };
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    dialogs.open_insert_image(status, std::sync::Arc::new(image), name);
}

/// Read and import the picture at `path` on this thread; a failure is
/// recorded as a note naming the reason, and answers `None`.
pub(crate) fn import(status: &Status, path: &std::path::Path) -> Option<ImportedImage> {
    let outcome = std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| pdfcer_core::image_import::import(&bytes).map_err(|e| e.to_string()));
    match outcome {
        Ok(image) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "image-imported format={:?} px={}x{} dpi={:?}",
                    image.format, image.width, image.height, image.dpi
                )
            });
            Some(image)
        }
        Err(detail) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("image-import-failed detail={detail}")
            });
            // Stamped with the current epoch, so it stands until the next
            // real edit moves past it.
            if let Status::Open(doc) = status {
                crate::app::actions::record_note(
                    doc.edit_epoch,
                    crate::text::images::import_failed(&detail),
                );
            }
            None
        }
    }
}
