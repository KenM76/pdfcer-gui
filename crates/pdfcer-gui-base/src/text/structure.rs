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
         editor. pdfcer compares it with this document and writes a new copy that \
         appends only the objects you changed, so a signature over anything you did \
         not touch stays valid. This document is not changed.",
    )
}

/// The export picker's title.
#[must_use]
pub const fn export_dialog_title() -> &'static str {
    "Save a copy for hand editing"
}

/// The compile picker's title, carrying the change it will write.
#[must_use]
pub fn compile_dialog_title(modified: usize, added: usize, removed: usize) -> String {
    format!(
        "Save the compiled copy: {} changed, {} added, {} removed",
        objects(modified),
        objects(added),
        objects(removed)
    )
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
pub fn compiled(path: &str, modified: usize, added: usize, removed: usize) -> String {
    format!(
        "Wrote {path}: {} changed, {} added, {} removed, appended to this document \
         as a new version. Open it to check the result.",
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
    "The edited copy holds no change from this document, so nothing was written. \
     Check that the text editor saved it."
}

/// The edited copy was exported from a different state of this document.
#[must_use]
pub const fn stale_base() -> &'static str {
    "That copy was not exported from this document as it is now: the document has \
     changed since, or the copy came from another one. Compiling it would undo the \
     difference, so nothing was written. Export again and redo the hand edits."
}

/// Hand edits are not compiled into an encrypted document.
#[must_use]
pub const fn compile_encrypted() -> &'static str {
    "This document is encrypted, and hand edits are compiled only into an \
     unencrypted one, so nothing was written."
}

/// An enforced certification forbids any change.
#[must_use]
pub const fn certified() -> &'static str {
    "This document carries a certification signature that forbids changes, so the \
     hand edits were not compiled."
}

/// The compiled copy would overwrite the open document or the edited copy.
#[must_use]
pub const fn would_overwrite() -> &'static str {
    "The compiled copy needs a name of its own: it cannot replace this document or \
     the edited copy it was made from. Nothing was written."
}

/// The export would overwrite the open document.
#[must_use]
pub const fn export_would_overwrite() -> &'static str {
    "The copy for hand editing cannot replace this document. Nothing was written."
}

/// The edited copy could not be read as a PDF.
#[must_use]
pub fn unreadable(detail: &str) -> String {
    format!("The edited copy could not be read as a PDF: {detail}. Nothing was written.")
}

/// Any other failure, with the engine's or the system's own words.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Nothing was written: {detail}")
}

/// An encrypted document is not exported in plaintext.
#[must_use]
pub const fn encrypted() -> &'static str {
    "This document is encrypted, and a copy for hand editing would hold its contents \
     unencrypted, so nothing was written."
}
