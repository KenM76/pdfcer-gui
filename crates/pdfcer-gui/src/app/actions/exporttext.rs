//! # `app::actions::exporttext` — the plan a text export is made of, and the
//! pure parts of making one
//!
//! `file.export_text`, wired 2026-09-04 on the operator's ask:
//!
//! > *"also the engine can export PDFs as text. we should have export/import
//! > for that."*
//!
//! This module is the **shell-side value** a text export is described by, plus
//! everything about producing one that can be computed without a `Document`:
//! which pages, how the pages are joined, how the bytes are encoded, what the
//! file is called, and what came out empty. [`super::export::text`] does the
//! parts that need the open document — the extraction, the picker and the
//! write.
//!
//! The split is [`super::imageexport`]'s, and for its reason: the decisions are
//! testable without a PDF and the extraction is not, so the decisions get tests
//! and the extraction gets one call site.
//!
//! ---
//!
//! # ★★★ THE IMPORT HALF DOES NOT EXIST, AND THIS IS WHERE THAT IS RECORDED
//!
//! The operator asked for **"export/import"**, one word with a slash in it, and
//! only one side of the slash was buildable. Writing down *which* side and
//! *why* is the whole of what this section is for, because the next reader's
//! first question is going to be "where is the import".
//!
//! `pdfcer-core` was read for it — `text_edit/`, `text_extract/`, `edit.rs`'s
//! `impl EditSession`, `ocr/`, and `D:\Dev\pdfcer\docs\core-api\` parts 1–3.
//! *"Import text"* turns out to be three different features wearing one name,
//! and the engine offers none of the three:
//!
//! | what an operator could mean | the nearest verb | why it is not that feature |
//! |---|---|---|
//! | **Make a PDF out of a text file** | none | There is no document builder at all. `pdfcer_core::build` is *build provenance* — the compile stamp — not document construction. The shell can make a blank page (`app::blank`, `set_media_box`) and `EditSession::add_text` can put a run on it, but `add_text` is **one page, one call, at coordinates**: it does not paginate, and `addtext.rs:32` is explicit that overflow past the page is *"EMITTED regardless — these are disclosures, never clips"*. So a two-page text file would produce one page with the second page's words present in the content stream and painted off the sheet. That is a data-loss trap wearing the shape of a feature. |
//! | **Replace a page's text with a text file's** | `EditSession::edit_text` (`edit.rs:8675`) | Addresses **one located run**, via a `find` string or a pinned operator span. There is no *"replace page N's text with this string"*. And the mapping cannot be reconstructed from an export: `plain_text()`'s line breaks and word spaces are pdfcer's own derivation (negative result S5), one glyph is not one character (§9.10.3), and 13 % of runs carry glyphs from more than one show operator (`operator_span_invariant.rs`, measured over 4,289 fixtures). A round trip built on that would edit the wrong text and say it had succeeded. |
//! | **Put a text layer over a scan** | `EditSession::add_ocr_layer` (`edit.rs:7313`) | Takes `&[OcrPageLayer]`, whose one payload field is `recognised: &crate::ocr::OcrPage` — **positioned words**, produced by the recogniser from the raster. A `.txt` file has no positions, so there is nothing to hand it. This is `file.ocr`, and it already ships. |
//!
//! ⇒ **Nothing was faked.** No half-import shipped, no control was drawn that
//! declines when pressed, and the window says nothing about a round trip. A
//! request naming what a shell would need has been filed at
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\request_there_is_no_route_from_a_text_file_back_into_a_pdf.md`.
//!
//! ---
//!
//! # ★★ The default plan writes the CLIPBOARD's own bytes
//!
//! `file.copy_document_text` has been putting
//! `extract_document_view(…).plain_text()` on the clipboard since 2026-08-20.
//! At [`TextExportPlan`]'s defaults this export writes exactly that string,
//! byte for byte — same extraction options (the settings funnel), same
//! `plain_text()`, same U+000C between pages, no BOM, no line-ending rewrite.
//!
//! Every departure is opt-in and every one is named in the receipt afterwards.
//! Two answers to *"what is the text of this document"* inside one program is
//! worse than either answer alone, because both of them look like text.

use std::path::{Path, PathBuf};

/// How one page is separated from the next in the written file.
///
/// ★ Two values rather than a `bool`, because the two are not *"a marker, on or
/// off"* — they are two different characters-in-the-file, one of which is the
/// engine's and one of which is pdfcer's own prose. A `bool` would have made
/// the second look like a formatting preference rather than like added content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageSeparator {
    /// U+000C, the form feed.
    ///
    /// The engine's own choice, and its `plain_text()` doc gives the reason it
    /// is the right character rather than merely the traditional one: U+000C is
    /// Unicode line-break class **BK**, a mandatory break, so a conforming text
    /// renderer starts a new line at it while a caller that wants page
    /// boundaries can still split on it unambiguously. *"A newline would be
    /// indistinguishable from a derived line break; a blank line would be two
    /// more invented characters."*
    #[default]
    FormFeed,
    /// A visible line naming the page that follows — `crate::text::export_text::page_marker`.
    ///
    /// ★ **Text pdfcer wrote, which the document does not contain.** Offered
    /// because a form feed is invisible in several editors and an operator
    /// reading a forty-page export needs to know where they are; disclosed in
    /// the window *and* in the receipt, because the window is gone by the time
    /// anyone else reads the file.
    Marker,
}

/// How lines end in the written file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEndings {
    /// Exactly as extracted — a bare `\n` wherever the engine derived a line
    /// break. The clipboard's own bytes.
    #[default]
    AsExtracted,
    /// `\r\n`, for a Windows tool that would otherwise show the file as one
    /// long line.
    Windows,
}

/// Everything a text export needs, frozen when Export was pressed.
///
/// Rides `super::write::WriteAction::Text`, so `Clone` and `PartialEq` for that
/// enum's derives — [`super::imageexport::ImagePlan`]'s reason, stated there.
///
/// ★ The pages are **resolved**, not a scope and a string. The window has
/// already parsed the typed range — it needs the answer to decide whether
/// Export is pressable — so re-parsing in the apply phase would be a second
/// reading of the same box against a document that may have changed pages in
/// between.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextExportPlan {
    /// Zero-based page indices, in the order they will be written.
    pub pages: Vec<usize>,
    /// What goes between one page and the next.
    pub separator: PageSeparator,
    /// How lines end.
    pub line_endings: LineEndings,
    /// Whether the file opens with a UTF-8 byte-order mark.
    pub byte_order_mark: bool,
}

/// The UTF-8 byte-order mark, U+FEFF encoded.
///
/// Written literally rather than as `'\u{FEFF}'.to_string()`, because what
/// lands in the file is three specific bytes and a reader checking this against
/// the Unicode standard should see them.
const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// **Where the save dialog opens, and what it calls the file.**
///
/// Beside the document, named after it, with `.txt`.
///
/// # ★★★ `set_file_name`, NEVER `set_extension` — and this is a defect that
/// has already shipped once in this crate
///
/// `Path::set_extension` replaces everything after the **last** dot. A document
/// called `plan.rev2.pdf` has a stem of `plan.rev2`, which contains a dot, so
/// `set_extension("txt")` produces **`plan.txt`** — the revision is silently
/// deleted.
///
/// That is not a cosmetic loss. `plan.rev2.pdf` and `plan.rev3.pdf` would both
/// suggest `plan.txt`, so exporting the second **overwrites the first**, in a
/// save dialog whose only warning is the operating system's generic *"a file
/// with that name already exists"*. `.rev2` / `.rev3` is an ordinary CAD naming
/// shape, and the two files that collide are the two an operator is most likely
/// to want side by side.
///
/// ★ The DXF export shipped with exactly this bug for weeks, under a **comment
/// asserting the behaviour it did not have** — *"appending would produce
/// `plan.rev2.dxf` either way"*. It was found on 2026-09-04 by the image
/// export, which tested the helper against `plan.rev2.pdf` on its first run and
/// watched it fail. This function is written the safe way and
/// [`tests::a_revision_in_the_stem_survives_the_suggested_name`] is the test
/// that keeps it that way — **a claim in a comment is not a test.**
///
/// A document with no extension at all still gains one: the stem of `plan` is
/// `plan`, and the format appends unconditionally.
#[must_use]
pub fn suggested_path(document: &Path) -> PathBuf {
    let mut path = document.to_path_buf();
    let stem = document
        .file_stem()
        .map_or_else(|| "export".to_owned(), |s| s.to_string_lossy().into_owned());
    path.set_file_name(format!("{stem}.txt")); // ui-text-exempt: a file extension, never displayed as prose
    path
}

/// What [`assemble`] produced: the text, and the facts the receipt needs.
///
/// A struct rather than a tuple because the caller has to pick a *different
/// sentence* per fact and three of the four are optional — a `(String, Vec,
/// usize, usize)` would put four unlabelled positions where a reader needs
/// four names.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Assembled {
    /// The whole file's text, before encoding.
    pub text: String,
    /// **One-based** page numbers that produced no characters at all.
    ///
    /// One-based here, not zero-based, because the only consumer is the
    /// operator-facing sentence and a conversion that happens at the point of
    /// display is a conversion that gets forgotten at one of several points of
    /// display. `crate::text::export_text::pages_without_text` takes these
    /// verbatim.
    pub empty_pages: Vec<usize>,
    /// Characters in [`Self::text`], excluding anything pdfcer added.
    ///
    /// ★ Excluding the added markers and separators deliberately: the receipt
    /// promises the operator a count of **their** words, and a number inflated
    /// by pdfcer's own page markers would make the same document report a
    /// different size depending on a formatting checkbox.
    pub characters: usize,
    /// How many page-marker lines pdfcer wrote, for the disclosure.
    pub markers_added: usize,
}

/// **Join the extracted pages into one file's text.**
///
/// Takes `(one-based page number, that page's text)` pairs rather than the
/// engine's `PageText`, which is what makes this function pure and testable
/// without a PDF on disk. The caller does the `plain_text()` call; this decides
/// what goes *between* the results and counts what came out empty.
///
/// # The algorithm, and the two things it must not get wrong
///
/// 1. **A separator goes between pages, never before the first or after the
///    last.** `plain_text()`'s own rule (`mod.rs:1459-1467`: `if i > 0`). A
///    leading form feed makes a one-page export start with a page break that
///    means nothing; a trailing one leaves every file ending in a phantom page.
/// 2. **An empty page still counts as a page.** It contributes a separator and
///    a marker like any other, so page 5 of a six-page export is where page 5
///    is even when page 4 was a scan. Skipping it would silently renumber the
///    file.
///
/// ★ The marker replaces the form feed rather than joining it. Writing both
/// would give a reader two page boundaries per page and a `split('\u{000C}')`
/// that no longer lines up with the visible marks.
#[must_use]
pub fn assemble(pages: &[(usize, String)], separator: PageSeparator) -> Assembled {
    let mut out = Assembled::default();
    for (position, (number, page_text)) in pages.iter().enumerate() {
        if position > 0 {
            match separator {
                PageSeparator::FormFeed => out.text.push('\u{000C}'),
                PageSeparator::Marker => {
                    out.text
                        .push_str(&crate::text::export_text::page_marker(*number));
                    out.markers_added += 1;
                }
            }
        }
        if page_text.is_empty() {
            out.empty_pages.push(*number);
        }
        out.characters += page_text.chars().count();
        out.text.push_str(page_text);
    }
    out
}

/// **Turn the assembled text into the bytes that land on disk.**
///
/// Two transformations, in this order, and the order matters:
///
/// 1. **Line endings.** `\n` becomes `\r\n` when asked. Done on the text so a
///    `\r` the *document itself* contained is not doubled — the replacement
///    matches a bare `\n`, and any `\r\n` already present is normalised through
///    a `\n` first rather than becoming `\r\r\n`.
/// 2. **The byte-order mark**, prepended to the finished bytes. It must be the
///    first three bytes of the file or it is not a byte-order mark.
///
/// UTF-8 throughout, because that is what a Rust `String` is and what the
/// window promised. There is no code-page option and there will not be one: a
/// CAD drawing carries degree signs and diameter marks, and offering an
/// encoding that cannot represent them is offering a way to lose them.
#[must_use]
pub fn encode(text: &str, plan: &TextExportPlan) -> Vec<u8> {
    let body = match plan.line_endings {
        LineEndings::AsExtracted => text.to_owned(),
        // Normalise first, then expand — see the doc comment. `replace` on a
        // two-character pattern is a single pass and cannot produce `\r\r\n`.
        LineEndings::Windows => text.replace("\r\n", "\n").replace('\n', "\r\n"),
    };
    let mut bytes = Vec::with_capacity(body.len() + UTF8_BOM.len());
    if plan.byte_order_mark {
        bytes.extend_from_slice(&UTF8_BOM);
    }
    bytes.extend_from_slice(body.as_bytes());
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    // ======================================================================
    // The filename — the defect that already shipped once
    // ======================================================================

    /// ★★★ **`plan.rev2.pdf` must suggest `plan.rev2.txt`, not `plan.txt`.**
    ///
    /// The DXF path shipped this bug for weeks behind a comment asserting the
    /// opposite. It is asserted here rather than described, because the whole
    /// lesson of that incident is that a claim in a comment is not a test.
    ///
    /// The second half of the assertion is the one that makes it a **data-loss**
    /// test rather than a cosmetic one: two revisions of the same drawing must
    /// not suggest the same output name, because the save dialog's only
    /// protection is the operating system's generic overwrite warning.
    #[test]
    fn a_revision_in_the_stem_survives_the_suggested_name() {
        let two = suggested_path(Path::new(r"C:\jobs\plan.rev2.pdf"));
        let three = suggested_path(Path::new(r"C:\jobs\plan.rev3.pdf"));
        assert_eq!(two.file_name().unwrap(), "plan.rev2.txt");
        assert_eq!(three.file_name().unwrap(), "plan.rev3.txt");
        assert_ne!(
            two, three,
            "two revisions must not suggest the same file — that is the overwrite"
        );
    }

    /// The ordinary case, and the directory is kept.
    #[test]
    fn the_suggestion_sits_beside_the_document() {
        let path = suggested_path(Path::new(r"C:\jobs\drawing.pdf"));
        assert_eq!(path.file_name().unwrap(), "drawing.txt");
        assert_eq!(path.parent().unwrap(), Path::new(r"C:\jobs"));
    }

    /// A document with no extension still gains one.
    #[test]
    fn a_document_with_no_extension_still_gains_txt() {
        assert_eq!(
            suggested_path(Path::new("plan")).file_name().unwrap(),
            "plan.txt"
        );
    }

    // ======================================================================
    // The page range — called, not copied
    // ======================================================================

    /// ★ **The typed range is the print dialog's parser**, reached through
    /// [`super::imageexport::resolve_pages`].
    ///
    /// Asserted here as well as in `dialogs::print::tabs` because the claim
    /// being made is not *"the parser works"* — that is tested there — it is
    /// *"this feature reaches THAT parser"*. A second implementation would pass
    /// the first assertion and fail the intent, and the way it would be
    /// noticed is by these expectations diverging from the print window's.
    ///
    /// The expectations are the print dialog's own: `5,1-2` keeps the order
    /// typed, `1,1` is two entries and not one, and a range past the end
    /// refuses the whole spec rather than clamping — *"clamping would turn a
    /// typo into a job."*
    #[test]
    fn the_typed_range_is_the_print_dialogs_parser() {
        use super::super::imageexport::{PageScope, resolve_pages};
        let at = |spec: &str| resolve_pages(PageScope::Typed, spec, 10, 0);
        assert_eq!(at("3"), Some(vec![2]));
        assert_eq!(at("1-4"), Some(vec![0, 1, 2, 3]));
        assert_eq!(at("5,1-2"), Some(vec![4, 0, 1]));
        assert_eq!(at("1,1"), Some(vec![0, 0]));
        assert_eq!(at("11"), None, "past the end refuses, never clamps");
        assert_eq!(at("0"), None, "page zero is not a page");
        assert_eq!(at("5-3"), None, "a backwards range is refused");
        assert_eq!(at(""), None, "an empty box names no page");
    }

    /// The two non-typed scopes, which need no parser.
    #[test]
    fn the_other_two_scopes_answer_without_a_parse() {
        use super::super::imageexport::{PageScope, resolve_pages};
        assert_eq!(
            resolve_pages(PageScope::CurrentPage, "", 10, 6),
            Some(vec![6])
        );
        assert_eq!(
            resolve_pages(PageScope::AllPages, "", 3, 0),
            Some(vec![0, 1, 2])
        );
        assert_eq!(
            resolve_pages(PageScope::AllPages, "", 0, 0),
            None,
            "a document with no pages names none"
        );
    }

    // ======================================================================
    // Assembly — separators, empty pages, counts
    // ======================================================================

    /// ★ The form feed goes BETWEEN pages: never leading, never trailing.
    ///
    /// This is `plain_text()`'s own `if i > 0`, and asserting it here is what
    /// keeps this export producing the clipboard's own string rather than one
    /// that merely resembles it.
    #[test]
    fn the_form_feed_separates_and_does_not_bracket() {
        let one = assemble(&[(1, "alpha".to_owned())], PageSeparator::FormFeed);
        assert_eq!(one.text, "alpha", "a single page carries no separator");

        let three = assemble(
            &[
                (1, "alpha".to_owned()),
                (2, "beta".to_owned()),
                (3, "gamma".to_owned()),
            ],
            PageSeparator::FormFeed,
        );
        assert_eq!(three.text, "alpha\u{000C}beta\u{000C}gamma");
        assert_eq!(three.text.matches('\u{000C}').count(), 2);
        assert_eq!(three.characters, 14);
        assert_eq!(three.markers_added, 0);
    }

    /// ★★ **An empty page still occupies its place**, so page numbers after it
    /// are not silently shifted.
    #[test]
    fn an_empty_page_keeps_its_place_and_is_named() {
        let out = assemble(
            &[
                (1, "front".to_owned()),
                (2, String::new()),
                (3, "back".to_owned()),
            ],
            PageSeparator::FormFeed,
        );
        assert_eq!(out.text, "front\u{000C}\u{000C}back");
        assert_eq!(
            out.empty_pages,
            vec![2],
            "the empty page is named by its ONE-based number"
        );
        assert_eq!(
            out.characters, 9,
            "pdfcer's separators are not the operator's characters"
        );
    }

    /// Every page empty: the caller's refusal condition is a zero character
    /// count, and it must hold whatever the separator.
    #[test]
    fn a_document_of_scans_assembles_to_no_characters_at_all() {
        for separator in [PageSeparator::FormFeed, PageSeparator::Marker] {
            let out = assemble(
                &[(1, String::new()), (2, String::new()), (3, String::new())],
                separator,
            );
            assert_eq!(
                out.characters, 0,
                "this zero is what makes the export refuse instead of writing an empty file"
            );
            assert_eq!(out.empty_pages, vec![1, 2, 3]);
        }
    }

    /// The marker replaces the form feed, names the page that FOLLOWS it, and
    /// is counted so the receipt can disclose it.
    #[test]
    fn the_marker_replaces_the_form_feed_and_names_the_following_page() {
        let out = assemble(
            &[
                (4, "alpha".to_owned()),
                (5, "beta".to_owned()),
                (6, "gamma".to_owned()),
            ],
            PageSeparator::Marker,
        );
        assert!(
            !out.text.contains('\u{000C}'),
            "one page boundary per page, not two"
        );
        assert!(out.text.contains("Page 5") && out.text.contains("Page 6"));
        assert!(
            !out.text.contains("Page 4"),
            "the first page gets no marker — nothing precedes it"
        );
        assert_eq!(out.markers_added, 2);
        assert_eq!(
            out.characters, 14,
            "the marker's own words are pdfcer's, not the document's"
        );
    }

    /// Nothing at all in, nothing at all out.
    #[test]
    fn no_pages_assembles_to_nothing() {
        let out = assemble(&[], PageSeparator::FormFeed);
        assert!(out.text.is_empty() && out.empty_pages.is_empty());
        assert_eq!(out.characters, 0);
    }

    // ======================================================================
    // Encoding
    // ======================================================================

    /// ★★ The default plan writes the string unchanged — the clipboard's own
    /// bytes. This is the invariant the whole design rests on.
    #[test]
    fn the_default_plan_writes_the_string_unchanged() {
        let plan = TextExportPlan {
            pages: vec![0],
            separator: PageSeparator::default(),
            line_endings: LineEndings::default(),
            byte_order_mark: false,
        };
        let text = "Ø50 ±0.1\n30°\u{000C}second page";
        assert_eq!(encode(text, &plan), text.as_bytes());
    }

    /// The BOM is the first three bytes or it is not a BOM.
    #[test]
    fn the_byte_order_mark_leads_the_file() {
        let plan = TextExportPlan {
            pages: vec![0],
            separator: PageSeparator::FormFeed,
            line_endings: LineEndings::AsExtracted,
            byte_order_mark: true,
        };
        let bytes = encode("Ø50", &plan);
        assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF]);
        assert_eq!(&bytes[3..], "Ø50".as_bytes());
    }

    /// ★ CRLF conversion must not double a `\r` the document already carried.
    #[test]
    fn windows_line_endings_do_not_double_an_existing_carriage_return() {
        let plan = TextExportPlan {
            pages: vec![0],
            separator: PageSeparator::FormFeed,
            line_endings: LineEndings::Windows,
            byte_order_mark: false,
        };
        assert_eq!(encode("a\nb", &plan), b"a\r\nb");
        assert_eq!(
            encode("a\r\nb", &plan),
            b"a\r\nb",
            "already-CRLF text must come out CRLF, not CRCRLF"
        );
    }

    /// UTF-8 is not negotiable, and a degree sign proves it survives both
    /// transformations at once.
    #[test]
    fn a_drawings_symbols_survive_every_option() {
        let plan = TextExportPlan {
            pages: vec![0],
            separator: PageSeparator::Marker,
            line_endings: LineEndings::Windows,
            byte_order_mark: true,
        };
        let bytes = encode("Ø50 ±0.1\n30°", &plan);
        let round_tripped = String::from_utf8(bytes[3..].to_vec()).unwrap();
        assert_eq!(round_tripped, "Ø50 ±0.1\r\n30°");
    }
}
