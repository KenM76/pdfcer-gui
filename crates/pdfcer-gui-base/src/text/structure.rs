//! The words for **File ▸ Export ▸ Export for hand editing…** and **Compile
//! hand edits…** — the PDF-internals round trip qpdf calls QDF.

use super::commands::CommandText;

/// `file.export_structure`.
#[must_use]
pub const fn file_export_structure() -> CommandText {
    CommandText::new(
        "Export for hand editing…",
        "Writes a copy of this document laid out to be read and edited in a text \
         editor: every object on its own, every stream uncompressed. Edit it, then \
         use Compile hand edits… to bring the changes back.",
    )
}

/// `file.import_structure`.
#[must_use]
pub const fn file_import_structure() -> CommandText {
    CommandText::new(
        "Compile hand edits…",
        "Choose a copy made by Export for hand editing… and changed in a text \
         editor. pdfcer applies what you changed to this document as one step Undo \
         takes back, and Save appends only the objects you changed, so a signature \
         over anything you did not touch stays valid.",
    )
}

/// The export picker's title.
#[must_use]
pub const fn export_dialog_title() -> &'static str {
    "Save a copy for hand editing"
}

fn objects(count: usize) -> String {
    if count == 1 {
        "1 object".to_owned()
    } else {
        format!("{count} objects")
    }
}

/// The export receipt.
#[must_use]
pub fn exported(path: &str, objects_written: usize) -> String {
    format!(
        "Wrote {path} for hand editing: {}, every stream uncompressed. It is a full \
         rewrite of the file; Compile hand edits… appends only what you change.",
        objects(objects_written)
    )
}

/// The compile receipt.
#[must_use]
pub fn compiled(modified: usize, added: usize, removed: usize) -> String {
    format!(
        "Compiled the hand edits into this document: {} changed, {} added, {} \
         removed. Save appends them to the file as a new version; Undo takes them back.",
        objects(modified),
        objects(added),
        objects(removed)
    )
}

/// Streams that compared equal only once decoded.
#[must_use]
pub fn matched_after_decode(count: usize) -> String {
    format!(
        "{} judged unchanged by content although compressed differently, so not \
         written again.",
        if count == 1 {
            "1 stream was".to_owned()
        } else {
            format!("{count} streams were")
        }
    )
}

/// The edited copy matches this document.
#[must_use]
pub const fn nothing_changed() -> &'static str {
    "The edited copy holds no change from this document, so nothing was changed. \
     Check that the text editor saved it."
}

/// The edited copy was exported from a different state of this document.
#[must_use]
pub const fn stale_base() -> &'static str {
    "That copy was not exported from this document as it is now: the document has \
     changed since, or the copy came from another one. Compiling it would undo the \
     difference, so nothing was changed. Export again and redo the hand edits."
}

/// The edited copy records no source, so the stale-base check could not run.
#[must_use]
pub const fn unrecorded_base() -> &'static str {
    "That copy does not record which state of this document it was exported from, \
     so pdfcer could not check it. Anything changed in this document after the \
     export has been changed back; Undo takes the compile back."
}

/// The document's permissions forbid changing its contents.
#[must_use]
pub const fn compile_encrypted() -> &'static str {
    "This document's permissions forbid changing its contents, so the hand edits \
     were not compiled."
}

/// An enforced certification forbids any change.
#[must_use]
pub const fn certified() -> &'static str {
    "This document carries a certification signature that forbids changes, so the \
     hand edits were not compiled."
}

/// The export would overwrite the open document.
#[must_use]
pub const fn export_would_overwrite() -> &'static str {
    "The copy for hand editing cannot replace this document. Nothing was written."
}

/// The edited copy could not be read as a PDF.
#[must_use]
pub fn unreadable(detail: &str) -> String {
    format!("The edited copy could not be read as a PDF: {detail}. Nothing was changed.")
}

/// Any other failure, with the engine's or the system's own words.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Nothing was changed: {detail}")
}

/// An encrypted document is not exported in plaintext.
#[must_use]
pub const fn encrypted() -> &'static str {
    "This document is encrypted, and a copy for hand editing would hold its contents \
     unencrypted, so nothing was written."
}
