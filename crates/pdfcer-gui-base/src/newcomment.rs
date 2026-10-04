//! # `newcomment` — the comment kinds placed through their own engine verb
//!
//! Each variant authors one annotation the `TextAnnotSpec` route cannot
//! express, because the engine gives it a dedicated `EditSession` verb.

/// A comment to author, raised by the note dialog when it accepts.
#[derive(Debug, Clone, PartialEq)]
pub enum NewComment {
    /// **Attach a file to a page as a marker** (`/FileAttachment`,
    /// §12.5.6.15). The file was picked in `BeginTextAnnot`'s apply arm, so
    /// this carries its path; the bytes are read at apply time.
    Attachment {
        /// The 0-based page.
        page: usize,
        /// The marker's rectangle, in PDF user space.
        rect: pdfcer_core::page_tree::Rect,
        /// The file to embed.
        file: std::path::PathBuf,
        /// The marker's icon (`/Name`).
        icon: pdfcer_core::annot_author::AttachmentIcon,
        /// The description (`/Contents` and the filespec's `/Desc`), trimmed;
        /// `None` when the operator typed none.
        description: Option<String>,
    },
}
