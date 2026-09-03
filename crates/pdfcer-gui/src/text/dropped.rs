//! # `text::dropped` — the three sentences a drop can answer with
//!
//! Every one is a **disclosure on the status row**, and the reason they exist at
//! all is the reason the feature exists: until 2026-08-19 a file dragged onto
//! this window did nothing, silently. A drop that is ignored teaches an operator
//! that the program does not accept drops — a conclusion they will not revisit.
//!
//! ★ Two of the three say **what to do next**, because the operator's remedy is
//! not guessable from the refusal. The third names what pdfcer takes, which is
//! the only useful thing to say about a file it does not.

/// More than one file was dropped and only the first was acted on.
///
/// ★ Said rather than swallowed: an operator who drags four drawings and gets
/// one open has been told something false by the silence — that the other three
/// failed, or that they missed the window with them.
#[must_use]
pub fn only_the_first(count: usize) -> String {
    format!(
        "{count} files were dropped. pdfcer opened the first one; drop the others one at a time."
    )
}

/// An image was dropped with no document open.
///
/// ★★ The one refusal here that **must** name the remedy. There is no page to
/// put a picture on, and *"cannot insert"* leaves the operator to work out for
/// themselves that a document is the missing ingredient — which is exactly the
/// deduction they are least likely to make, because they were thinking about the
/// picture.
#[must_use]
pub const fn image_needs_a_document() -> &'static str {
    // ★ "the File tab" rather than the ribbon-path spelling with a
    // ▸ in it: `icons::glyphs` refused that codepoint here and was right
    // to. It runs through this project's COMMENTS and appears in no
    // operator-visible string, because the font stack cannot draw it — it
    // renders as a substitution box, which on a sentence whose whole job is
    // to tell the operator where to go is the worst place available for one.
    "A picture needs a page to go on. Open a PDF first, or make one from the File tab, \
     then drop the picture again."
}

/// The file is not one pdfcer takes.
///
/// Names the extension back, because a mis-drag is common and seeing *which*
/// file was caught is what tells the operator they grabbed the wrong one.
#[must_use]
pub fn not_accepted(ext: &str) -> String {
    if ext.is_empty() {
        "pdfcer takes a PDF to open, or a PNG, JPEG, BMP or TIFF to place on the page. That file \
         has no extension, so pdfcer could not tell what it was."
            .to_owned()
    } else {
        format!(
            "pdfcer takes a PDF to open, or a PNG, JPEG, BMP or TIFF to place on the page. It \
             does not read .{ext} files."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Every sentence names either a remedy or the accepted set.
    ///
    /// Asserted by length and terminal stop rather than by matching words: the
    /// property is *"this is an explanation, not a label"*, and a four-word
    /// refusal is the shape a future tidy-up would introduce.
    #[test]
    fn every_sentence_explains_rather_than_labels() {
        let all = [
            only_the_first(4),
            image_needs_a_document().to_owned(),
            not_accepted("dwg"),
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
    }
}
