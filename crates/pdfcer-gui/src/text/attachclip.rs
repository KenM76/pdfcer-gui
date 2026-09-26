//! # `text::attachclip` — the words the attachment clipboard uses
//!
//! Copy, Cut and Paste for an embedded file, and the one question that has to
//! be asked **before** the paste rather than reported after it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/attachclip.md`.

/// The Copy control on an attachment row.
#[must_use]
pub fn copy_button() -> String {
    "Copy".to_owned()
}

/// What Copy does, said in terms of the thing it enables.
///
/// Names the **destination** rather than the mechanism. *"Copies the file to
/// the clipboard"* describes a data structure; an operator wants to know they
/// can now put it in the other document, which is the whole reason the verb
/// exists.
#[must_use]
pub fn copy_tooltip() -> String {
    "Takes a copy of this file, so you can paste it into another open document.".to_owned()
}

/// The Cut control.
#[must_use]
pub fn cut_button() -> String {
    "Cut".to_owned()
}

/// What Cut does, and the half of it that is not obvious.
///
#[must_use]
pub fn cut_tooltip() -> String {
    "Takes this file out of the document and onto the clipboard. The bytes stay recoverable \
     from the document's earlier version until it is saved out fresh — removing is not the \
     same as erasing."
        .to_owned()
}

/// The Paste control.
#[must_use]
pub fn paste_button() -> String {
    "Paste".to_owned()
}

/// What Paste does, naming the file so the operator can see what is on the
/// clipboard without pressing anything.
///
/// The name is in the **tooltip** rather than the button, because the button
/// sits in a row of two-word controls and *"Paste drawing-rev-C.dwg"* would be
/// the only one that changed width as the clipboard changed.
#[must_use]
pub fn paste_tooltip(name: &str) -> String {
    format!("Attaches {name} to this document.")
}

/// Said when the destination already has an attachment of that name.
///
/// Beside the button, before the press. See the module header: the engine
/// **replaces** rather than refusing, so without this the operator would lose
/// a file and see nothing at all.
#[must_use]
pub fn replaces_note(name: &str) -> String {
    format!(
        "This document already has a file called {name}. Pasting will put this one in its \
         place — the one that is there now stops being listed."
    )
}

/// The status line after a paste.
#[must_use]
pub fn pasted(name: &str) -> String {
    format!("Attached {name}.")
}

/// The status line after a paste that replaced something.
///
/// A **different** sentence from [`pasted`], because the operator who did not
/// read the note needs the fact afterwards too, and *"Attached X"* is true of
/// both cases and useful in only one.
#[must_use]
pub fn pasted_over(name: &str) -> String {
    format!("Attached {name}, in place of the file of the same name that was there before.")
}

/// Said when Paste is pressed and the clipboard holds no attachment.
///
/// Reachable only through a chord or the harness seam — the control is not
/// drawn at all when there is nothing to paste, per R9. It exists so that route
/// says something rather than nothing.
#[must_use]
pub fn nothing_to_paste() -> String {
    "There is no file on the clipboard. Copy one from another document's Attachments panel \
     first."
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The replacement note must name the file and must say what is lost.
    #[test]
    fn the_replacement_note_names_the_file_and_the_consequence() {
        let s = replaces_note("drawing-rev-C.dwg");
        assert!(s.contains("drawing-rev-C.dwg"));
        assert!(
            s.contains("in its place") || s.contains("stops being listed"),
            "the note must say the existing file is displaced: {s}"
        );
    }

    /// Cut must not imply erasure: `EditSession::detach_file` leaves the bytes
    /// in the earlier revision. This is the test that keeps a later rewording
    /// from dropping the disclosure.
    #[test]
    fn cut_does_not_imply_erasure() {
        let s = cut_tooltip();
        assert!(s.contains("recoverable"), "{s}");
        assert!(
            !s.to_lowercase().contains("erase") || s.contains("not the same as erasing"),
            "{s}"
        );
    }

    /// The two outcome sentences must be distinguishable — see [`pasted_over`].
    #[test]
    fn a_paste_that_replaced_says_so() {
        let plain = pasted("a.txt");
        let over = pasted_over("a.txt");
        assert_ne!(plain, over);
        assert!(over.contains("in place of"));
    }
}
