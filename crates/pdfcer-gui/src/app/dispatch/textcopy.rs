//! # `app::dispatch::textcopy` — reading text out of the document and putting
//! it on the clipboard
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/textcopy.md`.

use eframe::egui;

use crate::app::PdfcerApp;
use crate::app::state::Status;

/// **Whether this module owns `id`.**
///
/// The same shape [`super::pages::handles`] uses, and for the same reason: the
/// `match` in `super` stays a list a reader can scan, and the routing predicate
/// lives beside the bodies it routes to — so a new verb here is one edit rather
/// than two.
#[must_use]
pub fn handles(id: &str) -> bool {
    matches!(id, "file.copy_page_text" | "file.copy_document_text")
}

/// Act on one of [`handles`]' ids.
///
/// Takes the whole application rather than a `&Status`, because one of the two
/// bodies records a decline and both read caches that live on the open
/// document.
pub fn dispatch(app: &mut PdfcerApp, ctx: &egui::Context, id: &str) {
    match id {
        // **The page's text comes from the per-page extraction cache**
        // (`app::cache::PageTextCache`), which is what canvas text selection
        // reads too. The session has no cheap route to one page's text —
        // `EditSession::find_text_with` needs `&mut` and walks the **whole
        // document** — so the cache is not an optimisation here, it is the
        // reason a per-page copy is affordable at all.
        "file.copy_page_text" => {
            if let Status::Open(doc) = &app.status {
                match doc.page_text() {
                    // `plain_text()` rather than `sourced_text()`: it
                    // carries the engine's derived word spaces and line
                    // breaks, so a copied page reads as a page. `sourced_`
                    // is the honest lower bound for a *test* asserting what
                    // the file provides, and it would paste as one
                    // unbroken word.
                    Some(text) => crate::canvas::textsel::copy(
                        ctx,
                        &text.plain_text(),
                        // ui-text-exempt: diagnostic trace field, never displayed
                        "page",
                    ),
                    None => {
                        // The engine's own reason where there is one, and
                        // a distinct token where there is not.
                        //
                        // Two facts reach here and they are traced apart:
                        // the page's content stream would not walk
                        // (`detail=` carries `pdfcer-core`'s error), and
                        // there is no such page at all. Another — the page
                        // extracted fine and has no text on it — is handled
                        // by `copy` rather than here. A reader of
                        // a trace from a machine they cannot see should not
                        // have to guess which kind of nothing happened;
                        // that is the same argument `objects-unavailable`
                        // makes one module over.
                        let detail = doc.page_text_failure().map(|e| e.clone());
                        crate::diag::trace(|| match &detail {
                            // ui-text-exempt: diagnostic trace, never displayed
                            Some(reason) => format!(
                                "command-declined id={id} reason=extract-failed \
                                 detail={reason:?}"
                            ),
                            // ui-text-exempt: diagnostic trace, never displayed
                            None => {
                                format!("command-declined id={id} reason=no-such-page")
                            }
                        });
                    }
                }
            }
        }
        // The whole-document twin. It really can block the window on a long
        // file — its own tooltip says so — because `extract_document_view`
        // walks every page. That cost is paid here and nowhere else: it is a
        // verb the operator invoked once, not a per-frame derivation, which
        // is exactly the line the page-level cache exists to draw.
        //
        // Deliberately NOT cached: a document-wide extraction keyed on the
        // edit epoch would hold the whole document's text alive for the life
        // of the session to serve a command pressed at most a handful of
        // times.
        "file.copy_document_text" => {
            if let Status::Open(doc) = &app.status {
                match pdfcer_core::text_extract::extract_document_view(
                    // The SESSION's revision, as everywhere else: the
                    // operator is copying the document they are looking at,
                    // unsaved edits included.
                    &doc.session.view(),
                    // The funnel, not `ExtractOptions::default()`. This
                    // and the page-level extraction in `app::cache` must
                    // agree, or the same document copied two ways would
                    // come out spaced two ways — and the operator's word-gap
                    // setting would apply to one of them.
                    &{
                        use crate::app::settings::SettingsExt;
                        doc.settings.extract_options()
                    },
                ) {
                    Ok(text) => crate::canvas::textsel::copy(
                        ctx,
                        &text.plain_text(),
                        // ui-text-exempt: diagnostic trace field, never displayed
                        "document",
                    ),
                    Err(e) => crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed
                        format!("command-declined id={id} reason=extract-failed detail={e}")
                    }),
                }
            }
        }
        // Unreachable: [`handles`] is the only route in and it names exactly
        // the ids above. Spelled rather than `unreachable!`, because an id
        // added to `handles` and forgotten here should do nothing visible and
        // say so on the trace, not abort the frame the operator is looking at.
        other => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "textcopy-unrouted id={other}"
            )
        }),
    }
}
