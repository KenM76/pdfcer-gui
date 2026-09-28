//! # `exporttext` — the plan a text export is made of, and the
//! pure parts of making one
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/exporttext.md`.

use std::path::{Path, PathBuf};

/// How one page is separated from the next in the written file.
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
    /// **Text pdfcer wrote, which the document does not contain.** Offered
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

/// In what order a page's words are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextOrder {
    /// The engine's `plain_text()`: lines top to bottom as drawn, so two
    /// columns interleave a line at a time. The clipboard's own bytes.
    #[default]
    AsDrawn,
    /// `pdfcer_core::block_layout` reading order: one block per line, a blank
    /// line between blocks, columns read one after the other, and running
    /// headers, footers and page numbers left out. Every kind decision is an
    /// inference and the receipt counts them.
    Reading,
}

/// Everything a text export needs, frozen when Export was pressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextExportPlan {
    /// Zero-based page indices, in the order they will be written.
    pub pages: Vec<usize>,
    /// What goes between one page and the next.
    pub separator: PageSeparator,
    /// In what order each page's words are written.
    pub order: TextOrder,
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
    /// Excluding the added markers and separators deliberately: the receipt
    /// promises the operator a count of **their** words, and a number inflated
    /// by pdfcer's own page markers would make the same document report a
    /// different size depending on a formatting checkbox.
    pub characters: usize,
    /// How many page-marker lines pdfcer wrote, for the disclosure.
    pub markers_added: usize,
}

/// **Join the extracted pages into one file's text.**
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

    /// **`plan.rev2.pdf` must suggest `plan.rev2.txt`, not `plan.txt`.**
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

    /// **The typed range is the print dialog's parser**, reached through
    /// [`super::imageexport::resolve_pages`].
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

    /// The form feed goes BETWEEN pages: never leading, never trailing.
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

    /// **An empty page still occupies its place**, so page numbers after it
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

    /// The default plan writes the string unchanged — the clipboard's own
    /// bytes. This is the invariant the whole design rests on.
    #[test]
    fn the_default_plan_writes_the_string_unchanged() {
        let plan = TextExportPlan {
            pages: vec![0],
            separator: PageSeparator::default(),
            order: TextOrder::default(),
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
            order: TextOrder::default(),
            line_endings: LineEndings::AsExtracted,
            byte_order_mark: true,
        };
        let bytes = encode("Ø50", &plan);
        assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF]);
        assert_eq!(&bytes[3..], "Ø50".as_bytes());
    }

    /// CRLF conversion must not double a `\r` the document already carried.
    #[test]
    fn windows_line_endings_do_not_double_an_existing_carriage_return() {
        let plan = TextExportPlan {
            pages: vec![0],
            separator: PageSeparator::FormFeed,
            order: TextOrder::default(),
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
            order: TextOrder::default(),
            line_endings: LineEndings::Windows,
            byte_order_mark: true,
        };
        let bytes = encode("Ø50 ±0.1\n30°", &plan);
        let round_tripped = String::from_utf8(bytes[3..].to_vec()).unwrap();
        assert_eq!(round_tripped, "Ø50 ±0.1\r\n30°");
    }
}
