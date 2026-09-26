//! # `text::merge` — what pdfcer says about combining files
//!
//! Three sentences, for `OPERATOR_REQUESTS.md` row **O68**'s
//! `tools.merge_files`. They live in their own module rather than in
//! `text::files` because that one owns **dialog headings and filters** — what
//! the operating system draws — and these are what *pdfcer* says afterwards, on
//! its own status row. Two different surfaces with two different audiences and
//! two different lifetimes.
//!
//! ## Rule 4 governs all three
//!
//! A combine writes a **new file** and changes nothing the operator is looking
//! at. So none of this may become a mark on the page view — no badge on the
//! source pages, no tint, no overlay. These are off-canvas sentences, which is
//! where a disclosure belongs, and they are the only thing about the operation
//! that appears in the window at all.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/merge.md`.

use std::path::Path;

/// **It worked**, with the two numbers that say whether it worked *correctly*.
#[must_use]
pub fn merged(sources: usize, pages: usize) -> String {
    format!(
        "Combined {} into a new document of {}. The originals are unchanged.",
        files(sources),
        page_count(pages)
    )
}

/// **A source could not be read**, naming which one.
#[must_use]
pub fn failed_source(path: &Path) -> String {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    format!("Nothing was combined: {name} could not be read.")
}

/// **It did not work**, and pdfcer is not pretending to know why in one line.
#[must_use]
pub fn failed() -> &'static str {
    "The files could not be combined. Nothing was written."
}

/// `1 file` / `3 files`, written out.
///
/// *"1 files"* is the kind of thing that makes an operator trust the rest of
/// the sentence less, on a row whose entire job is being believed.
fn files(n: usize) -> String {
    if n == 1 {
        "1 file".to_owned()
    } else {
        format!("{n} files")
    }
}

/// `1 page` / `12 pages`, written out, for [`files`]' reason.
fn page_count(n: usize) -> String {
    if n == 1 {
        "1 page".to_owned()
    } else {
        format!("{n} pages")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Singular and plural are both written out, in both counts.
    #[test]
    fn the_counts_read_as_english() {
        let one = merged(1, 1);
        assert!(one.contains("1 file "), "{one}");
        assert!(one.contains("1 page"), "{one}");
        assert!(!one.contains("1 files"), "{one}");
        assert!(!one.contains("1 pages"), "{one}");
        let many = merged(3, 12);
        assert!(many.contains("3 files"), "{many}");
        assert!(many.contains("12 pages"), "{many}");
    }

    /// The success sentence carries BOTH counts.
    #[test]
    fn the_success_sentence_names_what_went_in_and_what_came_out() {
        let s = merged(4, 37);
        assert!(
            s.contains('4'),
            "the number of sources chosen is missing: {s}"
        );
        assert!(
            s.contains("37"),
            "the number of pages written is missing: {s}"
        );
    }

    /// Every failure says nothing was written.
    ///
    /// The one fact an operator cannot check for themselves without going to
    /// look, and the one that decides whether they panic.
    #[test]
    fn every_failure_says_nothing_was_written() {
        assert!(failed().contains("Nothing was written"));
        let src = failed_source(Path::new(r"D:\drawings\SHEET 12.pdf"));
        assert!(
            src.contains("Nothing was combined"),
            "a failure to read one source must say the whole operation did not happen: {src}"
        );
        assert!(
            src.contains("SHEET 12.pdf"),
            "the failure must name the file it could not read: {src}"
        );
        assert!(
            !src.contains(r"D:\drawings"),
            "the folder is noise on a one-line status row: {src}"
        );
    }

    /// The success sentence says the sources survived.
    #[test]
    fn the_success_sentence_says_the_originals_survived() {
        assert!(merged(2, 4).contains("originals are unchanged"));
    }
}
