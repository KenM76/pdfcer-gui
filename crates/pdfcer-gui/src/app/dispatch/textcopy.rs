//! # `app::dispatch::textcopy` — reading text out of the document and putting
//! it on the clipboard
//!
//! Both commands copy formatted text — paragraphs, bold, sizes, families —
//! through `pdfcer_gui_base::clipboard::richtext`, as the canvas selection's
//! Ctrl+C does, so the three routes paste alike.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/textcopy.md`.


use crate::app::PdfcerApp;
use crate::app::state::Status;
use crate::clipboard::richtext;

/// **Whether this module owns `id`.**
#[must_use]
pub fn handles(id: &str) -> bool {
    matches!(id, "file.copy_page_text" | "file.copy_document_text")
}

/// Act on one of [`handles`]' ids.
pub fn dispatch(app: &mut PdfcerApp, id: &str) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    match id {
        "file.copy_page_text" => match richtext::page(doc, doc.view.page_index, None) {
            // ui-text-exempt: diagnostic trace field, never displayed
            Some(rich) => crate::canvas::textsel::copy_rich(doc.edit_epoch, &rich, "page"),
            None => {
                // Two facts reach here and are traced apart: the page's
                // content would not walk (`detail=` is the engine's error),
                // and there is no such page. A page with no text on it is
                // `copy_rich`'s `nothing-to-copy`.
                let detail = doc.page_text_failure().map(|e| e.clone());
                crate::diag::trace(|| match &detail {
                    // ui-text-exempt: diagnostic trace, never displayed
                    Some(reason) => format!(
                        "command-declined id={id} reason=extract-failed detail={reason:?}"
                    ),
                    // ui-text-exempt: diagnostic trace, never displayed
                    None => format!("command-declined id={id} reason=no-such-page"),
                });
            }
        },
        // Walks every page, and can block the window on a long file — its
        // tooltip says so. Not cached: it is pressed rarely, and a cache would
        // hold every page's text for the session.
        "file.copy_document_text" => match richtext::document(doc) {
            // ui-text-exempt: diagnostic trace field, never displayed
            Ok(rich) => crate::canvas::textsel::copy_rich(doc.edit_epoch, &rich, "document"),
            Err(e) => crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("command-declined id={id} reason=extract-failed detail={e}")
            }),
        },
        // An id added to `handles` and forgotten here does nothing visible
        // and says so on the trace, rather than aborting the frame.
        other => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("textcopy-unrouted id={other}")
        }),
    }
}
