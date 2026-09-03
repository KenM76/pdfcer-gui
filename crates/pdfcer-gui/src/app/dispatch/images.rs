//! # `app::dispatch::images` — choosing a picture, reading it, and refusing it
//! by name
//!
//! ## Why this is a module and not a match arm
//!
//! Because it is a **sequence**, not a verb. `edit.insert_image` is four steps
//! before anything is drawn — pick a file, read it, import it, and then either
//! refuse it or open a window — and a ninety-line sequence inside a `match` arm
//! is precisely how `super`'s file crossed 1,500 lines.
//!
//! It sits beside [`super::pages`], which is here for the same reason: that
//! module holds six ids whose bodies share an operand rule, and this one holds
//! one id whose body is longer than most tabs.
//!
//! ## ★ Why the import happens BEFORE the window opens
//!
//! So a file that cannot be placed is refused **at the moment it is chosen**,
//! naming the file's own problem — *"pdfcer does not place GIF images — it
//! places PNG, JPEG, BMP and TIFF"*, *"this image uses {feature}, which pdfcer
//! cannot place"* — rather than opening a window full of controls over a
//! picture that was never going to go in.
//!
//! It is also what lets the window state the picture's real facts: its format,
//! its pixel size **as displayed** (an EXIF-rotated photograph is transposed by
//! the importer, and the stored shape is not on screen anywhere), and whether
//! its resolution is one the file declared or one pdfcer assumed.
//!
//! ## ★ The refusal is the ENGINE's, passed through
//!
//! Unlike a `TwoLineRefusal`, whose wording lives in
//! [`crate::text::measure`]. The difference is what the message is **about**:
//! an `ImageImportError` names the operator's own file and the specific thing
//! wrong with it, so a catalog sentence would have to discard the half that is
//! the whole answer. `check-ui-strings.sh`'s exclusion 3 covers this shape, and
//! `crate::text::images::import_failed` is the wrapper that keeps the sentence
//! pdfcer's own while the detail stays the file's.
//!
//! ## Why the decode runs on this thread
//!
//! A drawing's logo is a few kilobytes and a site photograph is a few
//! megabytes; the decode is milliseconds either way. A worker would need a
//! channel, a pending state and a way to say the operator changed their mind —
//! machinery for a wait nobody notices. A **scan at 600 dpi** is the case that
//! would justify it, and it is the case to re-measure before building for
//! rather than the case to assume.

use crate::app::state::Status;
use crate::dialogs::DialogsState;

/// The whole of `edit.insert_image` after its capability guard.
///
/// The guard stays at the call site, with every other command's, because *"may
/// this mode edit content?"* is a question about the **command** and belongs
/// where the other answers to it are. Everything below is about a **file**.
pub(super) fn insert(dialogs: &mut DialogsState, status: &Status) {
    let crate::app::files::Picked::Path(path) = crate::app::files::pick_image_source() else {
        return;
    };
    insert_path(dialogs, status, &path);
}

/// Import the file at `path` and open the placement window.
///
/// # ★★ Why this is split out of [`insert`]
///
/// Because a **dropped** image has already answered the question `insert`'s
/// first line asks. The picker and the import were one function until
/// 2026-08-19, so drag-and-drop could not reuse the second half without opening
/// a file dialog over a file the operator had already chosen — which is the
/// shape of thing that gets built as a duplicate instead.
///
/// One import, one set of disclosures, one placement window, two doors. The
/// alternative is two code paths that agree today and disagree the first time
/// one of them learns something.
pub(crate) fn insert_path(dialogs: &mut DialogsState, status: &Status, path: &std::path::Path) {
    // Read and import on this thread. A drawing's logo is a few
    // kilobytes and a site photograph is a few megabytes; the
    // decode is milliseconds either way, and a worker would need a
    // channel, a pending state and a way to say the operator
    // changed their mind — machinery for a wait nobody notices.
    // A SCAN at 600 dpi is the case that would justify it, and it
    // is the case to re-measure before building it rather than the
    // case to assume.
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
