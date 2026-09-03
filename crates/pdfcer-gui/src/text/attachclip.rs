//! # `text::attachclip` — the words the attachment clipboard uses
//!
//! Copy, Cut and Paste for an embedded file, and the one question that has to
//! be asked **before** the paste rather than reported after it.
//!
//! ## ★★★ The disclosure this module exists for
//!
//! `EditSession::attach_file` builds its `/EmbeddedFiles` name-tree patch like
//! this (`edit.rs`, the *"sorted per §7.9.6"* block):
//!
//! ```text
//! entries.retain(|(k, _)| k != &name_bytes);
//! entries.push((name_bytes.clone(), Object::Reference(spec_id)));
//! ```
//!
//! ⇒ **A same-named attachment is REPLACED.** Not refused, not renamed, not
//! given a numeric suffix — the existing entry is dropped from the tree and the
//! new one takes its key. The bytes of the old one survive in the earlier
//! revision until a full rewrite, so it is recoverable, and **nothing on screen
//! would say it had happened**.
//!
//! That is the same class as the bookmark paste's dropped destination, and it
//! gets the same treatment: the question is asked beside the button, while the
//! operator can still choose, rather than reported as an outcome. See
//! [`replaces_note`].
//!
//! ★ It is a **statement**, not a confirmation dialog. `HANDOFF.md`'s rule is
//! *confirmed or clearly undoable*, and a paste is one `EditSession` command
//! and therefore one `Ctrl+Z`. What it must not be is **silent**, which is a
//! different requirement and the one being met here.
//!
//! ## ★★ What is deliberately NOT disclosed before the press
//!
//! **`AttachmentTreeUnsupported`** — a document whose `/EmbeddedFiles` root
//! holds `/Kids` rather than `/Names`. `attach_file` refuses it by name, and
//! rightly: inserting into a multi-node tree means repairing every `/Limits`
//! range up the chain, and getting that subtly wrong stops the document's
//! *existing* attachments resolving.
//!
//! This shell **cannot ask in advance**. `attachments::AttachmentNotes` reports
//! six conditions and the tree's shape is not among them, and nothing else in
//! the read API exposes it. So the refusal arrives after the press, in words,
//! through the ordinary decline path — which is honest but is one press worse
//! than R9 wants. Filed rather than worked around.

/// The Copy control on an attachment row.
#[must_use]
pub fn copy_button() -> String {
    "Copy".to_owned()
}

/// What Copy does, said in terms of the thing it enables.
///
/// ★ Names the **destination** rather than the mechanism. *"Copies the file to
/// the clipboard"* describes a data structure; an operator wants to know they
/// can now put it in the other document, which is the whole reason the verb
/// exists and the reason it was missing until 2026-09-01.
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
/// ★★ It says the bytes stay recoverable, because `detach_file`'s own doc
/// comment puts this shell under that obligation in as many words:
///
/// > *"This is NOT a redaction verb and must not be described as one … the
/// > attachment's bytes remain recoverable from the earlier revision … Shells
/// > are expected to say so rather than let 'delete' imply erasure."*
///
/// A Cut reads even more like erasure than a Remove does, so if the sentence
/// belongs anywhere it belongs here.
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
/// ★ The name is in the **tooltip** rather than the button, because the button
/// sits in a row of two-word controls and *"Paste drawing-rev-C.dwg"* would be
/// the only one that changed width as the clipboard changed.
#[must_use]
pub fn paste_tooltip(name: &str) -> String {
    format!("Attaches {name} to this document.")
}

/// ★★★ Said when the destination already has an attachment of that name.
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
/// ★ A **different** sentence from [`pasted`], because the operator who did not
/// read the note needs the fact afterwards too, and *"Attached X"* is true of
/// both cases and useful in only one.
#[must_use]
pub fn pasted_over(name: &str) -> String {
    format!("Attached {name}, in place of the file of the same name that was there before.")
}

/// Said when Paste is pressed and the clipboard holds no attachment.
///
/// ★ Reachable only through a chord or the harness seam — the control is not
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

    /// ★★★ The replacement note must name the file and must say what is lost.
    ///
    /// Both halves. A note saying only *"a file of that name exists"* leaves the
    /// operator to guess what pressing the button does — and the answer is the
    /// surprising one.
    #[test]
    fn the_replacement_note_names_the_file_and_the_consequence() {
        let s = replaces_note("drawing-rev-C.dwg");
        assert!(s.contains("drawing-rev-C.dwg"));
        assert!(
            s.contains("in its place") || s.contains("stops being listed"),
            "the note must say the existing file is displaced: {s}"
        );
    }

    /// ★★ Cut must not imply erasure. `detach_file` puts this shell under that
    /// obligation by name; this is the test that keeps a later rewording from
    /// dropping it.
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
