//! # `app::dispatch::images` — choosing a picture, reading it, and refusing it
//! by name
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/images.md`.

use crate::app::state::Status;
use crate::dialogs::DialogsState;

/// The whole of `edit.insert_image` after its capability guard.
pub(super) fn insert(dialogs: &mut DialogsState, status: &Status) {
    let crate::app::files::Picked::Path(path) = crate::app::files::pick_image_source() else {
        return;
    };
    insert_path(dialogs, status, &path);
}

/// Import the file at `path` and open the placement window.
pub(crate) fn insert_path(dialogs: &mut DialogsState, status: &Status, path: &std::path::Path) {
    // Read and import on this thread — see the module header for why a worker
    // would be machinery for a wait nobody notices.
    let outcome = std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| pdfcer_core::image_import::import(&bytes).map_err(|e| e.to_string()));
    match outcome {
        Ok(image) => {
            let name = path
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "image-imported format={:?} px={}x{} dpi={:?}",
                    image.format, image.width, image.height, image.dpi
                )
            });
            dialogs.open_insert_image(status, std::sync::Arc::new(image), name);
        }
        Err(detail) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("image-import-failed detail={detail}")
            });
            // The one place a refusal is surfaced without an edit
            // to ride in on, and `record_note` is what that is for
            // — see `canvas::interact`'s caret decline. Stamped
            // with the CURRENT epoch, so it stands until the next
            // real edit moves past it.
            if let crate::app::state::Status::Open(doc) = status {
                crate::app::actions::record_note(
                    doc.edit_epoch,
                    crate::text::images::import_failed(&detail),
                );
            }
        }
    }
}
