//! # `text::fonts` — what the font-donor scan says when it skips a file
//!
//! Five sentences, all of them about something that did **not** happen.
//!
//! ## ★★★ Why a skip gets a sentence at all
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
//! ## ★ Each names the FILE
//!
//! Not the folder, and not a count. A folder holding two hundred files and one
//! problem needs the one named; *"3 files were skipped"* is a number that
//! sends somebody to look through two hundred.

use std::path::Path;

/// A configured folder could not be opened.
///
/// ★ It says the rest were still searched, because that is the fact an
/// operator needs in order to decide whether to care. A removable drive that is
/// not mounted is a normal state of a list that is otherwise fine.
#[must_use]
pub fn folder_unreadable(folder: &Path, detail: &str) -> String {
    format!(
        "Could not read the font folder {} ({detail}). The other folders were still searched.",
        folder.display()
    )
}

/// A file was past the size ceiling.
///
/// ★ It gives the size, because the ceiling is only actionable beside the
/// number that exceeded it — and because a genuinely enormous "font" is nearly
/// always something else with a font extension, which the operator can see at a
/// glance once they know which file.
#[must_use]
pub fn file_too_large(path: &Path, bytes: u64) -> String {
    format!(
        "Skipped {} — {:.1} MB is past the {} MB limit for a font file.",
        path.display(),
        bytes as f64 / (1024.0 * 1024.0),
        crate::app::fonts::MAX_FONT_FILE_BYTES / (1024 * 1024)
    )
}

/// A file could not be read from disk.
#[must_use]
pub fn file_unreadable(path: &Path) -> String {
    format!("Skipped {} — it could not be read.", path.display())
}

/// A file was read and is not a font this build understands.
///
/// ★ The parser's own reason is passed through rather than re-worded, for the
/// rule every other pass-through in this shell follows: *"unsupported table
/// format"* and *"truncated"* are different problems, and a generic sentence
/// throws away the only part that distinguishes them.
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

/// Where a **bundled** donor came from, for the row and for the engine's
/// `SuppliedFont::source`.
///
/// ★★★ It says *pdfcer's own copy*, in those words, because that is the fact the
/// operator needs and the one nothing else on screen carries. A row reading
/// `FoxitSans` beside a row reading `C:\Windows\Fonts\arial.ttf` invites the
/// reading that both are files and one of them has an odd path.
///
/// ★ `pdfcer` writes `"bundled: FoxitSans"` for the same value, and this
/// deliberately does not match it: that string is for a log a developer greps,
/// and this is a sentence in a window. The engine's field doc says the value is
/// *"never parsed; only reported"*, which is what makes them free to differ.
#[must_use]
pub fn bundled_source(face: &str) -> String {
    format!("pdfcer's own copy of {face}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every sentence names the file it is about.**
    ///
    /// The failure this guards is the tempting summary — *"3 files skipped"* —
    /// which is a number that sends an operator through a folder of two
    /// hundred looking for three.
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
