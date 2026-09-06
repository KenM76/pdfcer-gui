//! # `app::actions::merge` — **Combine several PDFs into one new file**
//!
//! `OPERATOR_REQUESTS.md` row **O68**, in the operator's words:
//!
//! > *"Also the Merge files and Split files buttons don't do anything."*
//!
//! They did not. `tools.merge_files` was registered, drawn on the Tools tab,
//! given an icon and a tooltip promising *"Combine several PDFs into one new
//! file"*, and had **no arm** in `PdfcerApp::dispatch_command` — so every press
//! fell to the catch-all, traced `command-unimplemented`, and did nothing an
//! operator could see. This module is the arm's body.
//!
//! ## ★★★ The engine was never the blocker, and the record said it was
//!
//! `pdfcer_core::pageops::merge` implements Acrobat's Combine Files behaviour
//! **whole** — per-source bookmark generation, `Doc0_`/`Doc1_` duplicate-field
//! auto-renaming, the first source's `/Info`, no inherited page-label scheme —
//! and nothing in this shell had ever called it.
//!
//! The reachability register recorded the blocker as *"Salvage, Class C … the
//! pane has not been brought across"*, which names a missing **host**. Ten
//! lines below it in the same file, `tools.font_folders` had been retired from
//! that same list on 2026-08-28 with the finding that closes this one:
//!
//! > *"A blocker naming a missing HOST is weaker than one naming a missing
//! > capability, and it goes stale the moment any other host will do."*
//!
//! A dialog is a host. So is a picker. Nothing about combining files on disk
//! needed the salvaged batch pane, and the entry survived an audit that
//! deleted six of eleven neighbours for exactly this shape of reason.
//!
//! ## Why this is not `vector_edit`
//!
//! Because a merge **produces new file bytes and touches neither the session
//! nor the undo log**. That is `app::actions::extract`'s argument, made at its
//! line 45, and this is the same shape one step further out: extract reads the
//! open document, this reads several documents none of which need be open.
//!
//! Nothing here bumps `edit_epoch`, drops a raster, invalidates a cache or
//! writes an undo entry, because nothing about the open document changed.
//!
//! ## ★★ Rule 4, and it decides what this may draw
//!
//! A merge writes somewhere else. **Nothing about it may appear in the page
//! view of the open document** — no badge on the source pages, no provisional
//! tint, no preview overlay. The disclosures the engine returns are real and
//! are owed to the operator, and they go to the status row like every other
//! off-canvas disclosure in this shell.
//!
//! Two of them matter enough to name here, because they are pdfcer policy that
//! an operator would otherwise discover by comparing files afterwards:
//!
//! * the combined document takes the **first** source's `/Info` and no other's;
//! * it carries **no page-label scheme**, because there is no single source to
//!   inherit one from and Acrobat does not generate one either.

use std::path::{Path, PathBuf};

/// **Combine `sources`, in order, into a new document at `target`.**
///
/// Every page of every source, which is what `pageops::merge` does and what
/// *Combine Files* means. A partial merge is a different verb and would need a
/// page chooser; this one has nothing left to ask, which is why it opens no
/// options window between the two pickers.
///
/// # The order is the operator's, and it is the picker's
///
/// `rfd::FileDialog::pick_files` returns the selection in the order the
/// platform reports it. That is not nothing — it is the order the combined
/// document's pages come out in — and it is deliberately **not** re-sorted
/// here. Sorting by name would be a rule pdfcer invented; leaving it alone means
/// the answer is whatever the operator's file manager showed them, which is the
/// only order they have any expectation about. When a reorder is wanted, it is
/// a list with drag handles in a dialog, and that is a feature rather than a
/// default.
///
/// # What is reported, and where
///
/// One trace line naming the counts, and the engine's own disclosures on the
/// status row. Failures are traced and reported; nothing is silent, which is
/// the standing rule this whole command was violating.
pub(crate) fn write_merge(status: &crate::app::state::Status, sources: &[PathBuf], target: &Path) {
    use pdfcer_core::document::Document;

    // ★ The revision the sentence is stamped with, or `None` with nothing
    // open. Both status-row channels — `app::status::disclosure` and
    // `app::status::decline` — take an `&OpenDoc`, so **with no document open
    // there is nowhere on screen for a sentence to go.**
    //
    // That is a real gap and it is stated rather than papered over: this
    // command is deliberately live with nothing open (it produces a document
    // from files on disk), so a merge run from an empty window reports only to
    // the trace. Closing it needs a document-free disclosure slot, which is a
    // change to the status row rather than to this verb, and is owed.
    let epoch = match status {
        crate::app::state::Status::Open(doc) => Some(doc.edit_epoch),
        _ => None,
    };
    let say = |note: String| {
        if let Some(epoch) = epoch {
            crate::app::actions::record_note(epoch, note);
        }
    };

    if sources.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "merge-files-declined reason=no-sources".to_owned()
        });
        return;
    }

    // 1. Open every source. Held in a `Vec` for the whole call because a
    //    `DocumentView` borrows the `Document` it came from, so the documents
    //    have to outlive the views — which is why this is two loops rather
    //    than one `map`.
    let mut documents: Vec<Document> = Vec::with_capacity(sources.len());
    for path in sources {
        match Document::load(path) {
            Ok(doc) => documents.push(doc),
            Err(error) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "merge-files-failed path={path:?} detail={error}"
                    )
                });
                // ★ The whole merge stops rather than the unreadable source
                // being skipped. A combine that silently produced a document
                // missing one of the files the operator chose is the worst
                // available outcome: it succeeds, it writes, and the loss is
                // invisible until somebody counts the pages.
                say(crate::text::merge::failed_source(path));
                return;
            }
        }
    }

    let views: Vec<_> = documents.iter().map(Document::view).collect();

    // 2. The titles, which are what make per-source bookmarks appear.
    //
    // ★ `OutlinePolicy::PerSource` fires only when `titles` is non-empty, so
    // supplying these is not cosmetic — it is the difference between a combined
    // document with a top-level bookmark per source and one with no outline at
    // all. The file **stem** rather than the full name, because the extension
    // in a bookmark title is noise.
    let titles: Vec<Vec<u8>> = sources
        .iter()
        .map(|p| {
            p.file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
                .into_bytes()
        })
        .collect();

    // ★★ The FILE NAMES, which are a different list from the titles above and
    // are what re-point a cross-file bookmark — `Pass` of 2026-09-06.
    //
    // A bookmark in one source that opens another of the files being merged
    // used to be **dropped**; given these it is repointed at that file's pages
    // inside the combined document, and `AssembleReport::outline_items_relinked`
    // counts how many. The engine shipped this unasked, with `&[]` preserving
    // the old behaviour — and `&[]` is exactly what a mechanical fix to the new
    // three-argument signature would have passed, silently keeping the drop.
    //
    // ⇒ **A compile error is an invitation to read the reply, not to satisfy
    // the compiler.** The full file NAME here, not the stem: it is matched
    // against a `/Launch` or `/GoToR` file specification written by whoever
    // authored the bookmark, and that specification carries the extension.
    let files: Vec<Vec<u8>> = sources
        .iter()
        .map(|p| {
            p.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
                .into_bytes()
        })
        .collect();

    // 3. The merge itself.
    let (bytes, report) = match pdfcer_core::pageops::merge(&views, &titles, &files) {
        Ok(pair) => pair,
        Err(error) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "merge-files-failed n={} detail={error}",
                    sources.len()
                )
            });
            say(crate::text::merge::failed().to_owned());
            return;
        }
    };

    // 4. The write.
    if let Err(error) = std::fs::write(target, &bytes) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "merge-files-failed path={target:?} bytes={} detail={error}",
                bytes.len()
            )
        });
        say(crate::text::merge::failed().to_owned());
        return;
    }

    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // `sources=` beside `pages=` for the reason `extract`'s line gives
            // about the ink trail: a build that combined the wrong set — two
            // files where three were chosen — writes a perfectly good PDF, and
            // these two fields are the only things in the line that would
            // differ. `path` is Debug-quoted so a Windows path with a space in
            // it cannot make every field after it unreadable.
            "merge-files path={target:?} sources={} pages={} bytes={}",
            sources.len(),
            report.pages,
            bytes.len(),
        )
    });

    // 5. The disclosure, off-canvas. See this module's header on rule 4: the
    //    page view of the open document is untouched by a merge and must
    //    remain so, and what the operator is owed is a sentence rather than a
    //    mark on a page that has nothing to do with the file just written.
    say(crate::text::merge::merged(sources.len(), report.pages));
}
