//! # `app::actions::merge` — **Combine several PDFs into one new file**
//!
//! `OPERATOR_REQUESTS.md` row **O68**, in the operator's words:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/merge.md`.

use std::path::{Path, PathBuf};

/// **Combine `sources`, in order, into a new document at `target`.**
pub(crate) fn write_merge(status: &crate::app::state::Status, sources: &[PathBuf], target: &Path) {
    use pdfcer_core::document::Document;

    // The revision the sentence is stamped with, or `None` with nothing
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
                // The whole merge stops rather than the unreadable source
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
    // `OutlinePolicy::PerSource` fires only when `titles` is non-empty, so
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

    // The FILE NAMES, a different list from the titles above: these are what
    // re-point a cross-file bookmark.
    //
    // A bookmark in one source that opens another of the files being merged is
    // repointed at that file's pages inside the combined document, and
    // `AssembleReport::outline_items_relinked` counts how many. Passing `&[]`
    // here compiles and drops every such bookmark silently, so this argument is
    // load-bearing rather than decorative.
    //
    // The full file NAME, not the stem: it is matched against a `/Launch` or
    // `/GoToR` file specification written by whoever authored the bookmark, and
    // that specification carries the extension.
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
    say(crate::text::merge::merged(
        sources.len(),
        report.pages,
        report.page_label_ranges,
    ));
}
