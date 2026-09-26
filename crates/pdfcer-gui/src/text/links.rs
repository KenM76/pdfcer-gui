//! # `text::links` — what the program says about a link it cannot follow
//!
//! Consumed by [`crate::canvas::links`]. Five sentences, and **four of them are
//! about failure**, which is the shape of the problem rather than pessimism.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/links.md`.

/// A `/GoTo` whose target page is not in this document's page tree.
#[must_use]
pub fn unmapped_page() -> &'static str {
    "This link points at a page that is not in this document. That usually means the page was deleted, or this file was made from a range of a larger one."
}

/// A named destination that neither §12.3.2.3 namespace defines.
#[must_use]
pub fn unresolved_name(name: &str) -> String {
    format!(
        "This link points at a destination named \"{name}\", which this document does not define. That usually means the name table was lost when the file was made."
    )
}

/// A `/GoToR` — a destination in a different file.
#[must_use]
pub fn remote(file: &str) -> String {
    format!("This link points into another file — {file}. Open that file to follow it.")
}

/// A `/URI`, `/Launch`, `/JavaScript`, `/SubmitForm` or other non-navigation
/// action.
#[must_use]
pub fn non_navigation(action: &str) -> String {
    format!(
        "This link is a {action} action, not a page jump. pdfcer shows what it is and does not run it."
    )
}

/// As [`non_navigation`], **naming the file the action opens**.
pub fn non_navigation_file(action: &str, file: &str) -> String {
    format!(
        "This link is a {action} action that opens {file}. pdfcer shows what it is and does not run it."
    )
}

/// **A remote file and a page number**, for [`remote`]'s hole.
#[must_use]
pub fn remote_page(file: &str, page: u64) -> String {
    format!("{file}, page {page}")
}

/// **A remote file and a destination NAME**, for [`remote`]'s hole.
#[must_use]
pub fn remote_named(file: &str, name: &str) -> String {
    format!("{file}, {name:?}")
}

/// A `/Link` carrying neither `/Dest` nor `/A`.
#[must_use]
pub fn no_destination() -> &'static str {
    "This link has no destination at all. It is a clickable box the document never finished."
}
