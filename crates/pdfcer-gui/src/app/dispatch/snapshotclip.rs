//! # `app::dispatch::snapshotclip` — Copy while a snapshot box is laid
//!
//! The first rung of `edit.copy`: with a box on the page, Copy means the box.
//! Places the box's region; it neither reads the internal clipboard nor changes
//! the document.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/snapshotclip.md`.

use crate::app::PdfcerApp;
use crate::app::state::Status;

/// Whether Copy is about the snapshot box: a document is open and has one.
#[must_use]
pub fn owns_copy(app: &PdfcerApp) -> bool {
    matches!(&app.status, Status::Open(doc) if doc.snapshot.is_some())
}

/// Copy the box, then say what landed or why nothing did.
pub fn copy(app: &PdfcerApp) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    let epoch = doc.edit_epoch;
    match crate::clipboard::snapshot::copy_snapshot(doc) {
        Ok(copied) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "clipboard-snapshot-copy dpi={} asked={} w={} h={} vectors={} formats={}",
                    copied.dpi,
                    copied.asked_dpi,
                    copied.width_px,
                    copied.height_px,
                    copied.vectors.token(),
                    copied.formats.join(",")
                )
            });
            crate::app::actions::record_note(epoch, crate::text::snapshot::copied(&copied));
        }
        Err(refusal) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "clipboard-snapshot-copy-refused reason={}",
                    reason(&refusal)
                )
            });
            crate::app::actions::record_note(
                epoch,
                crate::text::clipboard::copy_out_refusal(&refusal),
            );
        }
    }
}

/// The refusal's trace token.
fn reason(refusal: &crate::clipboard::place::Refusal) -> &'static str {
    use crate::clipboard::place::Refusal;
    match refusal {
        Refusal::NoPage => "no-box", // ui-text-exempt: a trace token, never displayed
        Refusal::Render(_) => "render", // ui-text-exempt: a trace token, never displayed
        Refusal::WouldDegrade => "would-degrade", // ui-text-exempt: a trace token, never displayed
        Refusal::Clipboard(_) => "clipboard", // ui-text-exempt: a trace token, never displayed
    }
}
