//! # `text::dropped` — the sentences a drop can answer with
//!
//! Each says what to do next or names what pdfcer takes, because the
//! operator's remedy is not guessable from a refusal.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/dropped.md`.

/// Pictures that a later engine may read, named with their own remedy.
const CONVERTIBLE: &[&str] = &["gif", "webp"];

/// Several pictures were dropped with Alt held, which opens the placement
/// window for the first.
#[must_use]
pub fn alt_takes_the_first(count: usize) -> String {
    format!(
        "{count} pictures were dropped with Alt held, which opens the placement window for the \
         first one only. Drop the others without Alt to place them where they land."
    )
}

/// An image was dropped with no document open.
#[must_use]
pub const fn image_needs_a_document() -> &'static str {
    // "the File tab" rather than a ribbon path with a triangle glyph: the
    // font stack cannot draw that codepoint.
    "A picture needs a page to go on. Open a PDF first, or make one from the File tab, \
     then drop the picture again."
}

/// A text file was dropped with no document open.
#[must_use]
pub const fn text_needs_a_document() -> &'static str {
    "A dropped text file becomes new pages after the one you are on, so it needs a document. \
     Open a PDF first, then drop the text file again."
}

/// The file is not one pdfcer takes; names the extension back, because seeing
/// which file was caught is what tells the operator they grabbed the wrong one.
#[must_use]
pub fn not_accepted(ext: &str) -> String {
    if ext.is_empty() {
        "pdfcer takes a PDF to open, a PNG, JPEG, BMP or TIFF to place on the page, or a .txt \
         file to add as pages. That file has no extension, so pdfcer could not tell what it was."
            .to_owned()
    } else if CONVERTIBLE.contains(&ext) {
        format!(
            "pdfcer cannot read .{ext} pictures yet. Save the picture as PNG or JPEG in another \
             program and drop that instead."
        )
    } else {
        format!(
            "pdfcer takes a PDF to open, a PNG, JPEG, BMP or TIFF to place on the page, or a \
             .txt file to add as pages. It does not read .{ext} files."
        )
    }
}

/// The dropped-PDF window's title.
#[must_use]
pub const fn drop_pdf_title() -> &'static str {
    "Dropped PDF"
}

/// The question the dropped-PDF window asks about `name`.
#[must_use]
pub fn drop_pdf_question(name: &str, pages: usize) -> String {
    let count = if pages == 1 {
        "1 page".to_owned()
    } else {
        format!("{pages} pages")
    };
    format!("{name} has {count}. Open it on its own, or bring it into this document?")
}

/// The insert answer: all `pages` pages after page `page` (1-based).
#[must_use]
pub fn drop_pdf_insert(pages: usize, page: usize) -> String {
    if pages == 1 {
        format!("Insert its page after page {page}")
    } else {
        format!("Insert its {pages} pages after page {page}")
    }
}

/// The place answer.
#[must_use]
pub const fn drop_pdf_place() -> &'static str {
    "Place its first page here, as artwork"
}

/// The open answer, the window's default.
#[must_use]
pub const fn drop_pdf_open() -> &'static str {
    "Open it"
}

/// The cancel answer.
#[must_use]
pub const fn drop_pdf_cancel() -> &'static str {
    "Cancel"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every sentence names either a remedy or the accepted set.
    #[test]
    fn every_sentence_explains_rather_than_labels() {
        let all = [
            alt_takes_the_first(4),
            image_needs_a_document().to_owned(),
            text_needs_a_document().to_owned(),
            not_accepted("dwg"),
            not_accepted("gif"),
            not_accepted(""),
        ];
        for s in all {
            assert!(s.len() > 50, "too short to be an explanation: {s:?}");
            assert!(s.ends_with('.'), "must be a sentence: {s:?}");
        }
    }

    /// The dropped-PDF answers count pages as the operator does.
    #[test]
    fn the_drop_pdf_answers_count_pages() {
        assert_eq!(drop_pdf_insert(1, 3), "Insert its page after page 3");
        assert_eq!(drop_pdf_insert(4, 3), "Insert its 4 pages after page 3");
        assert!(drop_pdf_question("a.pdf", 1).contains("has 1 page."));
    }

    /// The extension travels into the sentence, so a mis-drag is identifiable.
    #[test]
    fn an_unknown_extension_is_named_back() {
        assert!(not_accepted("dwg").contains(".dwg"));
        assert!(not_accepted("webp").contains(".webp"));
    }

    /// A GIF or WebP gets a remedy, not the list of what is taken.
    #[test]
    fn a_convertible_picture_is_told_how_to_convert() {
        for ext in CONVERTIBLE {
            assert!(not_accepted(ext).contains("PNG or JPEG"), "{ext}");
        }
    }
}
