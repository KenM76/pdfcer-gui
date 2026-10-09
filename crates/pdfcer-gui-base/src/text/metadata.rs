//! The words for **Security ▸ Protect ▸ Remove metadata…** — the command, its
//! window and its receipt. No trace quotes an item's preview; the window
//! shows it to the operator, who owns it.

use pdfcer_core::doc_metadata::MetadataKind;

use super::commands::CommandText;

/// `file.remove_metadata`.
#[must_use]
pub const fn file_remove_metadata() -> CommandText {
    CommandText::new(
        "Remove metadata…",
        "Lists the information the file carries besides its pages (description \
         entries, XMP, scripts, attachments, comments, thumbnails, earlier versions \
         and more) and saves a copy without the items you tick.",
    )
}

/// The window's title.
#[must_use]
pub const fn title() -> &'static str {
    "Remove metadata"
}

/// Above the list.
#[must_use]
pub const fn intro() -> &'static str {
    "Tick the items to leave out. They are removed from a copy you save; the \
     open document is not changed."
}

/// A kind's heading in the list.
#[must_use]
pub const fn kind(kind: MetadataKind) -> &'static str {
    match kind {
        MetadataKind::InfoEntry => "Description entries",
        MetadataKind::DocumentXmp => "Document XMP",
        MetadataKind::ObjectXmp => "XMP on pages, images and fonts",
        MetadataKind::PieceInfo => "Private data from the producing program",
        MetadataKind::Thumbnail => "Page thumbnails",
        MetadataKind::JavaScript => "Scripts",
        MetadataKind::Attachment => "Attachments",
        MetadataKind::Comment => "Comments",
        MetadataKind::HiddenLayer => "Layers hidden when the file opens",
        MetadataKind::FormData => "Values typed into the form",
        MetadataKind::EarlierRevisions => "Earlier versions kept in the file",
        MetadataKind::DocumentId => "File identifier",
        _ => "Other",
    }
}

/// What happens to a ticked file identifier.
#[must_use]
pub const fn document_id_note() -> &'static str {
    "A new random identifier is written in its place, so nothing links the copy \
     to earlier copies."
}

/// The list stopped short.
#[must_use]
pub const fn truncated() -> &'static str {
    "The file is too large to search completely; this list may be missing items."
}

/// When the document carries nothing to list.
#[must_use]
pub const fn none_present() -> &'static str {
    "This document carries no metadata or hidden information."
}

/// Ticks every item.
#[must_use]
pub const fn select_all() -> &'static str {
    "Select all"
}

/// Removes the ticked items into a copy.
#[must_use]
pub const fn remove_button() -> &'static str {
    "Remove and save a copy…"
}

/// Why Remove is greyed.
#[must_use]
pub const fn names_nothing() -> &'static str {
    "Tick at least one item to remove."
}

/// Closes the window.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The picker's title.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Save a copy without the chosen metadata"
}

/// An item's size, for its row.
#[must_use]
pub fn size(bytes: u64) -> String {
    super::panels::byte_size(usize::try_from(bytes).unwrap_or(usize::MAX))
}

/// The receipt's first line.
#[must_use]
pub fn wrote(path: &str, removed: usize) -> String {
    let word = if removed == 1 { "item" } else { "items" };
    format!(
        "Copy written to {path} with {removed} {word} removed. It holds one version \
         of the file. The open document is unchanged."
    )
}

/// Items the engine found but did not remove, each with its reason.
#[must_use]
pub fn not_removed(items: &[(String, String)]) -> String {
    let list: Vec<String> = items
        .iter()
        .map(|(id, why)| format!("{id} ({why})"))
        .collect();
    format!("Not removed, and still in the copy: {}.", list.join("; "))
}

/// Items that no longer exist by the time the copy was made.
#[must_use]
pub fn gone_before(count: usize) -> String {
    let (word, verb) = if count == 1 {
        ("item", "was")
    } else {
        ("items", "were")
    };
    format!("{count} {word} {verb} no longer in the document and had nothing to remove.")
}

/// The written copy still lists an item that was removed.
#[must_use]
pub fn still_present(ids: &[String]) -> String {
    format!(
        "The copy still held {}, so it was not written.",
        ids.join(", ")
    )
}

/// Refused on a signed document.
#[must_use]
pub fn refused_signed(signatures: usize) -> String {
    let word = if signatures == 1 {
        "signature"
    } else {
        "signatures"
    };
    format!(
        "Not done: the copy is rewritten as one version, which would break this \
         document's {signatures} {word}. Nothing was written."
    )
}

/// The engine refused or the write failed.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Could not remove the metadata: {detail}")
}
