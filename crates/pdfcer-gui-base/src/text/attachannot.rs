//! # `text::attachannot` — the words for attaching a file to a page
//!
//! The page-level counterpart of `text::panels::attachments`, whose files
//! belong to the document rather than to a marker on a page.

use pdfcer_core::annot_author::AttachmentIcon;

/// The label over the icon chooser.
#[must_use]
pub const fn icon_heading() -> &'static str {
    "Marker"
}

/// One marker icon's name, as an operator reads it.
#[must_use]
pub const fn icon_label(icon: &AttachmentIcon) -> &'static str {
    match icon {
        AttachmentIcon::PushPin => "Push pin",
        AttachmentIcon::Paperclip => "Paperclip",
        AttachmentIcon::Graph => "Graph",
        AttachmentIcon::Tag => "Tag",
        // Never offered; a file can carry one, a chooser never authors one.
        AttachmentIcon::Other(_) => "Another icon",
    }
}

/// The line naming the chosen file.
#[must_use]
pub fn file_line(name: &str, bytes: u64) -> String {
    format!(
        "File: {name} ({})",
        super::panels::byte_size(usize::try_from(bytes).unwrap_or(usize::MAX))
    )
}

/// The disclosure after a file is attached to a page.
#[must_use]
pub fn placed(name: &str, page: usize, bytes: u64) -> String {
    format!(
        "{name} is now stored in this PDF, attached to page {} ({}). It is a copy — the \
         original file is untouched. Deleting the marker removes the file from the PDF.",
        page + 1,
        super::panels::byte_size(usize::try_from(bytes).unwrap_or(usize::MAX))
    )
}

/// The refusal when the picked file cannot be read.
#[must_use]
pub fn unreadable(detail: &str) -> String {
    format!("pdfcer could not read that file, so nothing was attached: {detail}")
}
