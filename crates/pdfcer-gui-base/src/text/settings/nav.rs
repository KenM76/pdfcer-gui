//! # `text::settings::nav` — the words of the Settings window's page list
//!
//! The section labels over the page list, the one page whose heading is not
//! a group's own, and the search box.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/settings/nav.md`.

/// The first page: which program opens PDFs.
#[must_use]
pub fn group_general() -> &'static str {
    "General"
}

/// Section label over the pages about pdfcer's own window.
#[must_use]
pub fn nav_program() -> &'static str {
    "Program"
}

/// Section label over the pages about what a document is made of.
#[must_use]
pub fn nav_document() -> &'static str {
    "Document"
}

/// Section label over the pages about adding to a document.
#[must_use]
pub fn nav_authoring() -> &'static str {
    "Authoring"
}

/// Section label over the pages about writing the file.
#[must_use]
pub fn nav_output() -> &'static str {
    "Output"
}

/// Placeholder in the search box.
#[must_use]
pub fn search_hint() -> &'static str {
    "Search options"
}

/// Hover text on the search box.
#[must_use]
pub fn search_hover() -> &'static str {
    "Shows only the pages holding an option whose name contains this text, and marks the matching names on the page."
}

/// The right-hand pane when no page matches the search.
#[must_use]
pub fn search_none(query: &str) -> String {
    format!("No option's name contains \u{201c}{query}\u{201d}.")
}
