//! # `text::running` — the words for **Header & footer**
//!
//! Pages ▸ Stamp ▸ Header & footer…: its window, its live check and its
//! receipt. Consumed by `pdfcer_gui::dialogs::running`.

use pdfcer_core::running_text::{RunningTextAlign, RunningTextError, RunningTextKind};

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Header & footer"
}

/// The standing rule, first line of the window.
#[must_use]
pub const fn intro() -> &'static str {
    "Writes one line of text along the top or bottom of each page, as page \
     content, so it prints and saves like the rest of the page."
}

/// The whole-document scope choice.
#[must_use]
pub fn scope_all(of: usize) -> String {
    format!("All {of} page(s)")
}

/// The rail-pick scope choice.
#[must_use]
pub fn scope_picked(picked: usize) -> String {
    format!("The {picked} page(s) picked in the page rail")
}

/// The header/footer choice's label.
#[must_use]
pub const fn kind_heading() -> &'static str {
    "Edge"
}

/// The name of one edge.
#[must_use]
pub const fn kind(k: RunningTextKind) -> &'static str {
    match k {
        RunningTextKind::Header => "Header (top)",
        RunningTextKind::Footer => "Footer (bottom)",
    }
}

/// The alignment choice's label.
#[must_use]
pub const fn align_heading() -> &'static str {
    "Along the edge"
}

/// The name of one alignment.
#[must_use]
pub const fn align(a: RunningTextAlign) -> &'static str {
    match a {
        RunningTextAlign::Left => "Left",
        RunningTextAlign::Center => "Centre",
        RunningTextAlign::Right => "Right",
    }
}

/// Label of the text box.
#[must_use]
pub const fn text() -> &'static str {
    "Text"
}

/// What the text box starts with.
#[must_use]
pub const fn default_text() -> &'static str {
    "Page {page} of {pages}"
}

/// Tooltip of [`text`]: the two tokens.
#[must_use]
pub const fn text_tooltip() -> &'static str {
    "{page} becomes each page's number and {pages} the document's page count."
}

/// Label of the margin box.
#[must_use]
pub const fn margin() -> &'static str {
    "Distance from the edges (mm)"
}

/// Label of the font-size box.
#[must_use]
pub const fn font_size() -> &'static str {
    "Text size (pt)"
}

/// The live preview: the line as drawn on the first page in scope.
#[must_use]
pub fn preview(page: usize, line: &str) -> String {
    format!("Page {page} will read: {line}")
}

/// Why the line cannot be written, in the operator's terms.
#[must_use]
pub fn problem(e: &RunningTextError) -> String {
    match e {
        RunningTextError::Empty => "Type the text to write.".to_owned(),
        RunningTextError::Unencodable(c) => format!(
            "\u{201c}{c}\u{201d} cannot be printed in the header font. Use another character."
        ),
        RunningTextError::Geometry { .. } => {
            "The distance from the edges must be 0 or more and the text size above 0.".to_owned()
        }
        RunningTextError::NoPages => "No pages are selected.".to_owned(),
        other => other.to_string(),
    }
}

/// Note under the preview: the line becomes part of the page.
#[must_use]
pub const fn permanence() -> &'static str {
    "Undo removes it while this document is open. Once saved, it is part of \
     the pages; a page given a header twice carries both."
}

/// The commit button.
#[must_use]
pub const fn add() -> &'static str {
    "Add"
}

/// The cancel button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// The receipt after the line was written.
#[must_use]
pub fn receipt(kind: RunningTextKind, pages: usize, first: &str) -> String {
    let what = match kind {
        RunningTextKind::Header => "header",
        RunningTextKind::Footer => "footer",
    };
    format!("Added a {what} to {pages} page(s); the first reads \u{201c}{first}\u{201d}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unprintable_character_is_named() {
        let s = problem(&RunningTextError::Unencodable('\u{4e2d}'));
        assert!(s.contains('\u{4e2d}'), "{s}");
    }

    #[test]
    fn the_receipt_names_the_edge_and_the_first_line() {
        let s = receipt(RunningTextKind::Header, 3, "Page 1 of 3");
        assert!(s.contains("header") && s.contains("Page 1 of 3"), "{s}");
    }
}
