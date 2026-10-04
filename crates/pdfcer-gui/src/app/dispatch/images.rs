//! # `app::dispatch::images` — choosing a picture, reading it, and refusing it
//! by name
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/images.md`.

use crate::app::state::Status;
use crate::dialogs::DialogsState;
use pdfcer_gui_base::picture::Picture;

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

/// Read and import the picture or drawing at `path` on this thread; a
/// failure is recorded as a note naming the reason, and answers `None`.
pub(crate) fn import(status: &Status, path: &std::path::Path) -> Option<Picture> {
    let outcome = std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| Picture::import(path, &bytes));
    match outcome {
        Ok(picture) => {
            crate::diag::trace(|| imported_line(&picture));
            Some(picture)
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

/// The `image-imported` trace line: a raster's format, pixels and declared
/// resolution, or a drawing's kind, size in points and import notes.
fn imported_line(picture: &Picture) -> String {
    match picture.raster() {
        Some(image) => format!(
            "image-imported kind=image format={:?} px={}x{} dpi={:?}", // ui-text-exempt: diagnostic trace
            image.format, image.width, image.height, image.dpi
        ),
        None => {
            let (w, h) = picture.natural_size_pt();
            format!(
                "image-imported kind={} pt={w:.2}x{h:.2} notes=\"{}\"", // ui-text-exempt: diagnostic trace
                picture.kind(),
                picture
                    .drawing_notes()
                    .map_or_else(|| "none".to_owned(), |n| n.replace('"', "'")) // ui-text-exempt: diagnostic trace
            )
        }
    }
}
