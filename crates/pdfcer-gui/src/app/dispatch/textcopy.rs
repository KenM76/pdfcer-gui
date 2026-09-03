//! # `app::dispatch::textcopy` — reading text out of the document and putting
//! it on the clipboard
//!
//! Two ids, `file.copy_page_text` and `file.copy_document_text`, and one
//! subject: **the operator wants the words, somewhere else**.
//!
//! ## Why this is a module and not two match arms
//!
//! [`super::images`]' reason, in one sentence: `super`'s file crossed R2's
//! 1,500-line ceiling for the third time on 2026-08-20, and this is a family
//! whose two bodies are longer than most whole tabs and whose subject is
//! genuinely its own. It sits beside [`super::pages`] (six ids sharing an
//! operand rule) and [`super::images`] (one id whose body is a four-step
//! sequence) as the third application of the same seam.
//!
//! ## ★ Both read the SAME extraction, and that is the load-bearing fact
//!
//! `OpenDoc::page_text()` and the document-level `extract_*_view`, which is
//! also what a canvas text selection copies from. Two paths to *"the text of
//! this page"* is how a ribbon Copy and a swept selection come to disagree
//! about what is on it — and they would disagree **silently**, because both
//! answers look like text.
//!
//! ## ★ Neither raises an `Action`, and that is not an oversight
//!
//! A clipboard write touches no document and needs no frame boundary. It is
//! the same call `file.print` makes, for the same stated reason: the action
//! funnel exists for work that changes a document or that must not happen
//! mid-frame, and a copy is neither.
//!
//! `crate::canvas::textsel::copy` is the one place the clipboard is written and
//! the one place a copy is traced, so a copy from the ribbon and a copy from a
//! sweep leave the same evidence.

use eframe::egui;

use crate::app::PdfcerApp;
use crate::app::state::Status;

/// **Whether this module owns `id`.**
///
/// The same shape [`super::pages::handles`] uses, and for the same reason: the
/// `match` in `super` stays a list a reader can scan, and the routing predicate
/// lives beside the bodies it routes to — so a third verb here is one edit
/// rather than two.
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
        // ★ **The two text-copy verbs — registered since 2026-08-14 and
        // dead until now.**
        //
        // They were drawn on File ▸ Export, `Ctrl+Shift+C` was bound to the
        // page one, and neither had an arm: a live control that does
        // nothing, which is defect D1's shape and which this project's
        // own `both_text_copy_commands_are_offered_by_every_mode` test could
        // not see, because offering a command and implementing it are
        // different facts.
        //
        // What made them wirable was the per-page extraction cache
        // (`app::cache::PageTextCache`) arriving for canvas text selection.
        // Before it, `file.copy_page_text` had no cheap route to one page's
        // text: `EditSession::find_text_with` is the only text verb on the
        // session, it needs `&mut`, and it walks the **whole document**.
        //
        // ★ Both arms read `page_text()` / `extract_*_view`, so the string
        // an operator copies from the ribbon and the string a canvas
        // selection copies come from **one** extraction of one revision.
        // Two paths to "the text of this page" is how a Copy and a
        // selection come to disagree about what is on it.
        //
        // Neither raises an `Action`: a clipboard write touches no document
        // and needs no frame boundary — the same call `file.print` makes,
        // for the same stated reason. `canvas::textsel::copy` is the one
        // place the clipboard is written and the one place a copy is traced.
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
                        // ★ The engine's own reason where there is one, and
                        // a distinct token where there is not.
                        //
                        // Three states reach here and they are three
                        // different facts: the page's content stream would
                        // not walk (`detail=` carries `pdfcer-core`'s error),
                        // there is no such page at all, and — a fourth,
                        // handled by `copy` rather than here — the page
                        // extracted fine and has no text on it. A reader of
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
        // file — its own tooltip says so — because
        // `extract_document_view` walks every page, which `crate::find`
        // measured at 331–449 ms on this project's fixtures. That cost is
        // paid here and nowhere else: it is a verb the operator invoked
        // once, not a per-frame derivation, which is exactly the line the
        // page-level cache exists to draw.
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
                    // unsaved edits included (decision 018).
                    &doc.session.view(),
                    // ★ The funnel, not `ExtractOptions::default()`. This
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
        // the two ids above. Spelled rather than `unreachable!`, because a
        // third id added to `handles` and forgotten here should do nothing
        // visible and say so on the trace, not abort the frame the operator is
        // looking at.
        other => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "textcopy-unrouted id={other}"
            )
        }),
    }
}
