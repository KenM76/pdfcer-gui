//! # `app::actions::extract` — the one page verb that writes a NEW FILE
//!
//! Extract a set of sheets into a document of their own. Split out of
//! [`super::pages`] under **R2** on 2026-08-28, when the operator's separation
//! policy reached the delete verb and took that file past 1,500 lines.
//!
//! ## ★★ Why this is the seam, and not the enum
//!
//! [`super::pages`]' own header records the argument for the enum living *with*
//! its bodies — one family, one place a sixth verb has to answer the
//! invalidation question — and that argument still holds. Moving it back out
//! to make a line count work would be undoing a decision for a reason that has
//! nothing to do with why it was made.
//!
//! What genuinely does not belong beside the others is **this** verb, and the
//! distinction is not size:
//!
//! | every other page verb | extract |
//! |---|---|
//! | changes the open document | leaves it **untouched** |
//! | goes through `vector_edit` | goes through nothing — no epoch, no undo entry, no texture drop |
//! | is undoable | is a file on disk |
//! | needs no path | opens a **native save picker** and suggests a name |
//!
//! ⇒ It is a **save** wearing a page verb's clothes, and it shares its
//! machinery with `app::save` rather than with its neighbours: the same
//! picker, the same `PDFCER_DIAG_SAVE_PATH` seam that lets a driven check answer
//! a native modal no synthetic input can reach, and the same "write, then
//! report on both channels" shape.
//!
//! ## What stayed behind
//!
//! `super::pages::apply`'s routing arm. This module is bodies only, which is
//! the same division `super::annots` and `super::bookmarks` keep: the arm
//! routes, the module acts.

use std::path::{Path, PathBuf};

use crate::app::files::{self, Picked};
use crate::app::state::OpenDoc;

/// **Write the operand pages out as a new standalone document.**
///
/// The whole of [`Action::ExtractPages`], and the one page verb that does not
/// go anywhere near `vector_edit`: `pdfcer_core::pageops::extract` reads a
/// `DocumentView` and returns bytes. Nothing is mutated, so there is no worker
/// to cancel, no `Arc::get_mut` to fail, no epoch to bump and no texture to
/// drop — which is `crate::app::save`'s §2 argument for `file.save_copy`,
/// reaching the same conclusion for the same reason.
///
/// # ★ The view is the SESSION's, not the file's
///
/// `doc.session.view()` rather than the loaded `Document`, so an extraction
/// carries the operator's **unsaved edits** — decision 018, and the same choice
/// `file.copy_document_text` makes one dispatch arm over. An operator who
/// rotates three sheets and then extracts them must get the rotated sheets;
/// getting the file as it was opened would be a silent, plausible-looking
/// wrong answer.
///
/// # Why the destination is asked for rather than derived
///
/// The operator's standing rule — *Read may produce a new document; it may not
/// modify this one* — is enforced by **asking**, exactly as
/// `crate::app::files::pick_save_path`'s own docs describe: a path the operator
/// names cannot silently be the one they opened. [`suggested_path`] guarantees
/// the *suggestion* is never that file, so accepting the default without
/// reading it is safe too.
///
/// This is the third caller of that picker and it shares the
/// `PDFCER_DIAG_SAVE_PATH` seam with the other two, which is why
/// `tools/ui-verify`'s page-ops check can answer a native modal no synthetic
/// input can reach.
///
/// [`Action::ExtractPages`]: super::Action::ExtractPages
pub(super) fn extract(doc: &OpenDoc, pages: &[usize]) {
    if pages.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "extract-declined reason=no-pages".to_owned()
        });
        return;
    }
    let suggested = suggested_path(doc);
    let target =
        match files::pick_save_path(&suggested, crate::text::files::extract_pages_dialog_title()) {
            Picked::Path(path) => path,
            // A cancelled extraction is a complete, correct, uninteresting
            // outcome — `save_copy`'s wording, and its reasoning.
            Picked::Cancelled => return,
            Picked::Unavailable => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "extract-unavailable reason=no-picker-in-this-build".to_owned()
                });
                return;
            }
        };
    write_extract(doc, pages, &target);
}

/// Assemble the new document and put it on disk, reporting on the trace.
///
/// Split from [`extract`] so the picker and the write are separable in the
/// reading as well as in the testing — `crate::app::save::write_and_report`'s
/// reason, and this half is the one a unit test can reach, because it never
/// opens a dialog.
fn write_extract(doc: &OpenDoc, pages: &[usize], target: &Path) {
    let assembled = pdfcer_core::pageops::extract(&doc.session.view(), pages);
    let (bytes, report) = match assembled {
        Ok(pair) => pair,
        Err(error) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "extract-failed path={target:?} n={} detail={error}",
                    pages.len()
                )
            });
            return;
        }
    };
    match std::fs::write(target, &bytes) {
        Ok(()) => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                //
                // `pages=` beside `bytes=` for `HANDOFF.md` §2's reason about
                // the ink trail: a build that extracted the wrong count — the
                // whole document, say, or one page where three were picked —
                // writes a perfectly good PDF, and this field is the only
                // thing in the line that would differ. `path` is Debug-quoted
                // exactly as `save-copy`'s is, so a Windows path with a space
                // in it cannot make every field after it unreadable.
                "extract path={target:?} pages={} bytes={} asked={}",
                report.pages,
                bytes.len(),
                pages.len(),
            )
        }),
        Err(error) => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "extract-failed path={target:?} bytes={} detail={error}",
                bytes.len()
            )
        }),
    }
}

/// The name and folder [`extract`] offers the picker.
///
/// `<stem>-pages.pdf` beside the document, which is
/// `crate::app::save::suggested_path`'s shape with
/// [`crate::text::files::extract_pages_suffix`] in place of `-copy`. It is a
/// separate function rather than a shared one taking a suffix because that one
/// is private to a module this work may not edit, and because the two are
/// allowed to diverge: an extraction of pages 3–7 could one day suggest a name
/// that says so, and a save-a-copy never could.
///
/// # ★ It is never the file that was opened
///
/// The promise [`crate::text::files::extract_pages_suffix`] makes, as a
/// mechanism. The extension is forced to `.pdf` for `save_copy`'s reason: the
/// bytes are a PDF whatever the source was called, and `SHEET.PDF` extracting
/// to `SHEET-pages.PDF` would be one more way for a downstream tool to disagree
/// about case.
fn suggested_path(doc: &OpenDoc) -> PathBuf {
    let Some(source) = doc.stored_under() else {
        // A created document has a name, not a location. Offer the name and
        // let the picker choose the folder — `save::suggested_path`'s answer
        // for the same state, and the only honest one.
        return doc.path.clone();
    };
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let name = format!("{stem}{}.pdf", crate::text::files::extract_pages_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, Origin, open_fixture};

    /// A scratch path under the OS temporary directory.
    ///
    /// `std::env::temp_dir` rather than a path in the repository, for
    /// `crate::app::save`'s stated reason: a test that writes beside the
    /// fixtures leaves a file somebody eventually commits.
    ///
    /// ★ A **copy** of `super::super::pages`' helper rather than an import, and
    /// deliberately so: it is five lines, and a test helper reaching across a
    /// module boundary makes two suites fail together for reasons neither is
    /// about. The directory name differs from that one's for the same reason —
    /// two suites writing into one scratch directory is a shared mutable state
    /// nobody declared.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("pdfcer-gui-extract-tests");
        std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
        dir.join(name)
    }

    /// Apply one engine verb to a fixture, the way `vector_edit` does.
    ///
    /// The four-step protocol is not re-run here — there is no render worker in
    /// a unit test and nothing else holds the `Arc` — but the step the
    /// assertion depends on is: the mutation. `an_extraction_carries_unsaved_edits`
    /// is the only consumer, and what it needs is a session with an edit in it.
    fn edit(doc: &mut OpenDoc, verb: impl FnOnce(&mut pdfcer_core::edit::EditSession)) {
        let session = std::sync::Arc::get_mut(&mut doc.session)
            .expect("nothing else holds the session in a test");
        verb(session);
        doc.edit_epoch += 1;
    }

    /// **★★ The extracted file is a real document containing exactly the pages
    /// that were asked for.**
    ///
    /// The round trip in the smallest form a unit test can hold, and the same
    /// shape as `app::save`'s: write it, re-open it from disk through the
    /// loader the application uses, and count. A build that wrote the whole
    /// document — the plausible wrong answer, since `extract` and `save_copy`
    /// both produce "a PDF beside the original" — passes any check that only
    /// asks whether a file appeared.
    #[test]
    fn an_extraction_writes_exactly_the_pages_it_was_given() {
        use pdfcer_core::document::Document;

        let doc = open_fixture(FOUR_PAGES);
        let target = scratch("extracted.pdf");
        let _ = std::fs::remove_file(&target);

        write_extract(&doc, &[1, 2], &target);

        let written = std::fs::read(&target).expect("the extraction must land on disk");
        assert!(
            written.starts_with(b"%PDF-"),
            "a freestanding PDF, not a fragment"
        );
        let reopened = Document::load(&target).expect("the extraction must open");
        let pages = pdfcer_core::page_tree::pages(&reopened).expect("its page tree must walk");
        assert_eq!(
            pages.len(),
            2,
            "two pages were asked for and the file has {}; a build that wrote the whole \
             document produces a perfectly good PDF and would pass any check that only asks \
             whether a file appeared",
            pages.len()
        );

        // …and the source is untouched. An extraction that modified the
        // document it read from would breach the operator's standing rule
        // outright, and it is asserted rather than assumed because `extract`
        // and `save_copy` share a picker and a suffix convention.
        assert_eq!(doc.pages.len(), 4);
        let _ = std::fs::remove_file(&target);
    }

    /// **★ An extraction carries the operator's unsaved edits.**
    ///
    /// Decision 018, asserted rather than trusted: the view handed to
    /// `pageops::extract` is the **session's**, so a rotation made in this
    /// sitting is in the file that comes out. A build that passed the loaded
    /// `Document` instead would produce a valid file with the right page count
    /// and the edit silently missing — which the test above would not catch.
    #[test]
    fn an_extraction_carries_unsaved_edits() {
        use pdfcer_core::document::Document;

        let mut doc = open_fixture(FOUR_PAGES);
        let before = doc.pages[0].rotate;
        edit(&mut doc, |s| {
            s.rotate_pages(&[0], 90).expect("a quarter turn is legal");
        });

        let target = scratch("extracted-rotated.pdf");
        let _ = std::fs::remove_file(&target);
        write_extract(&doc, &[0], &target);

        let reopened = Document::load(&target).expect("the extraction must open");
        let pages = pdfcer_core::page_tree::pages(&reopened).expect("its page tree must walk");
        assert_eq!(pages.len(), 1);
        assert_eq!(
            pages[0].rotate,
            (before + 90) % 360,
            "the extraction was assembled from the file as it was OPENED rather than from the \
             session, so the operator's rotation is not in it"
        );
        let _ = std::fs::remove_file(&target);
    }

    /// ★ **The suggested name is never the file that was opened.**
    ///
    /// `save::suggested_path`'s guarantee, for the second write-destination
    /// this shell asks about. An operator who accepts the suggestion without
    /// reading it must not overwrite the drawing they are extracting from.
    #[test]
    fn the_suggested_extract_name_is_never_the_source_file() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.path = PathBuf::from("D:\\jobs\\4471\\Sheet 1.pdf");
        doc.origin = Origin::Opened;

        let suggested = suggested_path(&doc);
        assert_ne!(suggested, doc.path);
        assert_eq!(
            suggested,
            PathBuf::from("D:\\jobs\\4471\\Sheet 1-pages.pdf")
        );
        assert_eq!(
            suggested.parent(),
            doc.path.parent(),
            "the extraction should land beside the original, where the operator will look"
        );
    }
}
