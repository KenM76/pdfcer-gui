//! # `app::actions::extract` — the one page verb that writes a NEW FILE
//!
//! Extract a set of sheets into a document of their own.
//!
//! ## Why this verb is not beside the other page verbs
//!
//! [`super::pages`] keeps its enum *with* its bodies — one family, one place a
//! sixth verb has to answer the invalidation question. What does not belong
//! beside the others is **this** verb, and the distinction is behavioural:
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
//! The routing arm stays in `super::pages::apply`. This module is bodies only,
//! which is the division `super::annots` and `super::bookmarks` keep too: the
//! arm routes, the module acts.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/extract.md`.

use std::path::{Path, PathBuf};

use pdfcer_core::pageops::{AssembleReport, ExtractedPageLabels, SeparationPolicy};

use crate::app::files::{self, Picked};
use crate::app::state::OpenDoc;
use crate::text::extract_pages as t;

/// **Write the operand pages out as a new standalone document.**
///
/// The whole of [`Action::ExtractPages`], and the one page verb that does not
/// go anywhere near `vector_edit`: `pdfcer_core::pageops::extract` reads a
/// `DocumentView` and returns bytes. Nothing is mutated, so there is no worker
/// to cancel, no `Arc::get_mut` to fail, no epoch to bump and no texture to
/// drop — which is `crate::app::save`'s §2 argument for `file.save_copy`,
/// reaching the same conclusion for the same reason.
///
/// # The view is the SESSION's, not the file's
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
/// The labels and separations choices reach `pageops::extract_with_labels`:
/// `labels` comes from the window, `separations` is the operator's Settings ▸
/// Pages policy, the same one a delete obeys.
///
/// Returns whether the file was written, which is what licenses the window's
/// *delete afterwards*.
///
/// [`Action::ExtractPages`]: super::Action::ExtractPages
pub(super) fn extract(
    doc: &OpenDoc,
    pages: &[usize],
    labels: ExtractedPageLabels,
    separations: SeparationPolicy,
) -> bool {
    if pages.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "extract-declined reason=no-pages".to_owned()
        });
        return false;
    }
    let suggested = suggested_path(doc);
    let target =
        match files::pick_save_path(&suggested, crate::text::files::extract_pages_dialog_title()) {
            Picked::Path(path) => path,
            // A cancelled extraction is a complete, correct, uninteresting
            // outcome — `save_copy`'s wording, and its reasoning.
            Picked::Cancelled => return false,
            Picked::Unavailable => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "extract-unavailable reason=no-picker-in-this-build".to_owned()
                });
                return false;
            }
        };
    let written = write_extract(doc, pages, &target, labels, separations);
    let note = match &written {
        Ok(report) => receipt(doc, report, &target),
        Err(detail) => t::failed(detail),
    };
    super::record_note(doc.edit_epoch, note);
    written.is_ok()
}

/// The status-line receipt for a written extraction. The labels sentence is
/// there only when the source has labels, since otherwise there was no choice.
fn receipt(doc: &OpenDoc, report: &AssembleReport, target: &Path) -> String {
    let file = target.file_name().map_or_else(
        || target.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let mut note = t::wrote(report.pages, &file);
    if report.page_labels_dropped {
        note.push(' ');
        note.push_str(t::labels_dropped());
    } else if report.page_label_ranges > 0 && doc.page_labels().is_some() {
        note.push(' ');
        note.push_str(t::labels_kept());
    }
    note
}

/// Assemble the new document and put it on disk, reporting on the trace.
/// `Err` carries the engine's or the file system's sentence.
fn write_extract(
    doc: &OpenDoc,
    pages: &[usize],
    target: &Path,
    labels: ExtractedPageLabels,
    separations: SeparationPolicy,
) -> Result<AssembleReport, String> {
    let assembled =
        pdfcer_core::pageops::extract_with_labels(&doc.session.view(), pages, separations, labels);
    let (bytes, report) = assembled.map_err(|error| {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "extract-failed path={target:?} n={} detail={error}",
                pages.len()
            )
        });
        error.to_string()
    })?;
    if let Err(error) = std::fs::write(target, &bytes) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "extract-failed path={target:?} bytes={} detail={error}",
                bytes.len()
            )
        });
        return Err(error.to_string());
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // `pages=` is what was written and `asked=` what was requested, so
            // a wrong count shows as the line disagreeing with itself. `path`
            // is Debug-quoted so a space in it cannot split the fields.
            "extract path={target:?} pages={} bytes={} asked={} labels={} labels_dropped={} label_ranges={}",
            report.pages,
            bytes.len(),
            pages.len(),
            labels_token(labels),
            u8::from(report.page_labels_dropped),
            report.page_label_ranges,
        )
    });
    Ok(report)
}

/// The trace token for a labels choice, owned here rather than `{:?}`.
fn labels_token(labels: ExtractedPageLabels) -> &'static str {
    match labels {
        ExtractedPageLabels::Keep => "keep",
        ExtractedPageLabels::Drop => "drop",
        // ui-text-exempt: a trace token, never displayed
        _ => "other",
    }
}

/// The name and folder [`extract`] offers the picker.
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
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("pdfcer-gui-extract-tests-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
        dir.join(name)
    }

    /// Apply one engine verb to a fixture, the way `vector_edit` does.
    fn edit(doc: &mut OpenDoc, verb: impl FnOnce(&mut pdfcer_core::edit::EditSession)) {
        let session = std::sync::Arc::get_mut(&mut doc.session)
            .expect("nothing else holds the session in a test");
        verb(session);
        doc.edit_epoch += 1;
    }

    /// **The extracted file is a real document containing exactly the pages
    /// that were asked for.**
    #[test]
    fn an_extraction_writes_exactly_the_pages_it_was_given() {
        use pdfcer_core::document::Document;

        let doc = open_fixture(FOUR_PAGES);
        let target = scratch("extracted.pdf");
        let _ = std::fs::remove_file(&target);

        write_extract(
            &doc,
            &[1, 2],
            &target,
            ExtractedPageLabels::Keep,
            SeparationPolicy::default(),
        )
        .expect("the extraction must be written");

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

    /// **An extraction carries the operator's unsaved edits.**
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
        write_extract(
            &doc,
            &[0],
            &target,
            ExtractedPageLabels::Keep,
            SeparationPolicy::default(),
        )
        .expect("the extraction must be written");

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

    /// **Keep carries the pages' labels; Drop writes none.** Pages 1-2 of
    /// `i ii 1 2` show `i ii` in the new file under Keep and `1 2` under Drop.
    #[test]
    fn the_labels_choice_reaches_the_new_file() {
        use pdfcer_core::document::Document;
        use pdfcer_core::page_labels::{label_ranges, page_labels};

        let doc = crate::app::state::open_local_fixture("labelled-pages.pdf");
        for (labels, want, ranges) in [
            (ExtractedPageLabels::Keep, ["i", "ii"], 1),
            (ExtractedPageLabels::Drop, ["1", "2"], 0),
        ] {
            let target = scratch(&format!("labelled-{}.pdf", labels_token(labels)));
            let _ = std::fs::remove_file(&target);
            let report = write_extract(&doc, &[0, 1], &target, labels, SeparationPolicy::default())
                .expect("the extraction must be written");
            assert_eq!(report.page_labels_dropped, ranges == 0);
            let reopened = Document::load(&target).expect("the extraction must open");
            assert_eq!(
                page_labels(&reopened).expect("a page tree"),
                want,
                "{labels:?}"
            );
            assert_eq!(label_ranges(&reopened).len(), ranges, "{labels:?}");
            let _ = std::fs::remove_file(&target);
        }
    }

    /// **The suggested name is never the file that was opened.**
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
