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
