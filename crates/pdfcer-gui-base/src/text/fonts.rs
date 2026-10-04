//! # `text::fonts` — what the font-donor scan says when it skips a file
//!
//! Sentences, all of them about something that did **not** happen.
//!
//! ## Why a skip gets a sentence at all
//!
//! Because *"pdfcer could not embed HelveticaNeue"* and *"pdfcer skipped
//! HelveticaNeue.ttf because it is 40 MB"* are the same event to the program
//! and completely different events to an operator. The first is a dead end.
//! The second is a thing they can act on in ten seconds.
//!
//! A scan that silently ignored what it could not read would turn every one of
//! these into the first sentence, and an operator whose font folder contains
//! the right face in the wrong format would have no way to find that out.
//!
//! ## Each names the FILE
//!
//! Not the folder, and not a count. A folder holding two hundred files and one
//! problem needs the one named; *"3 files were skipped"* is a number that
//! sends somebody to look through two hundred.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/fonts.md`.

use std::path::Path;

/// A configured folder could not be opened.
#[must_use]
pub fn folder_unreadable(folder: &Path, detail: &str) -> String {
    format!(
        "Could not read the font folder {} ({detail}). The other folders were still searched.",
        folder.display()
    )
}

/// A file was past the size ceiling.
#[must_use]
pub fn file_too_large(path: &Path, bytes: u64) -> String {
    format!(
        "Skipped {} — {:.1} MB is past the {} MB limit for a font file.",
        path.display(),
        bytes as f64 / (1024.0 * 1024.0),
        crate::fontlibrary::MAX_FONT_FILE_BYTES / (1024 * 1024)
    )
}

/// A file could not be read from disk.
#[must_use]
pub fn file_unreadable(path: &Path) -> String {
    format!("Skipped {} — it could not be read.", path.display())
}

/// A file was read and is not a font this build understands.
#[must_use]
pub fn not_a_font(path: &Path, detail: &str) -> String {
    format!(
        "Skipped {} — it is not a font pdfcer can read ({detail}).",
        path.display()
    )
}

/// A file parsed and offers no name to match on.
#[must_use]
pub fn no_name(path: &Path) -> String {
    format!(
        "Skipped {} — it is a font but advertises no name, and its filename gives none either.",
        path.display()
    )
}

/// Why a letter could not yet be added to an embedded font from the font
/// folders: they are still being read.
#[must_use]
pub fn folders_still_indexing() -> String {
    "pdfcer is still reading the font folders; type the letter again in a moment.".to_owned()
}

/// Why a face the ladder picked could not be cut: the folders were read again
/// since it was offered.
#[must_use]
pub fn face_no_longer_offered() -> String {
    "the font folders changed while the replacement face was being chosen.".to_owned()
}

/// Why a face the ladder picked could not be cut: its file could not be read.
#[must_use]
pub fn face_unreadable() -> String {
    "its font file could not be read again.".to_owned()
}

/// Where a **bundled** donor came from, for the row and for the engine's
/// `SuppliedFont::source`.
#[must_use]
pub fn bundled_source(face: &str) -> String {
    format!("pdfcer's own copy of {face}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every sentence names the file it is about.**
    #[test]
    fn every_skip_names_its_file() {
        let path = Path::new("C:/fonts/Odd.ttf");
        for line in [
            file_too_large(path, 40 * 1024 * 1024),
            file_unreadable(path),
            not_a_font(path, "truncated"),
            no_name(path),
        ] {
            assert!(line.contains("Odd.ttf"), "does not name the file: {line}");
        }
    }

    /// **The folder note says the others were still searched.**
    ///
    /// Without that clause an operator with one unmounted drive in their list
    /// cannot tell a partial scan from an abandoned one.
    #[test]
    fn an_unreadable_folder_says_the_rest_were_searched() {
        let line = folder_unreadable(Path::new("E:/Fonts"), "not found");
        assert!(line.contains("E:/Fonts") || line.contains(r"E:\Fonts"));
        assert!(line.contains("still searched"), "{line}");
    }
}
