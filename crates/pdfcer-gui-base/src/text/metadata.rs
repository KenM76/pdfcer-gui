//! The words for **Security ▸ Protect ▸ Remove metadata…** — the command and its
//! window. No sentence quotes a field's value in a trace; the window shows it
//! to the operator, who owns it.

use super::commands::CommandText;

/// `file.remove_metadata`.
#[must_use]
pub const fn file_remove_metadata() -> CommandText {
    CommandText::new(
        "Remove metadata…",
        "Lists the document's title, author, subject and keywords, and removes the \
         ones you tick. Each removal is one step of Undo.",
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
    "Tick the entries to remove from the document's own description."
}

/// What the window can and cannot find, said where the operator decides.
#[must_use]
pub const fn not_listed() -> &'static str {
    "Only these four entries are listed. Dates, the producing program and \
     XMP metadata are not found here yet."
}

/// When the document holds none of the four.
#[must_use]
pub const fn none_present() -> &'static str {
    "This document has no title, author, subject or keywords."
}

/// Ticks every entry.
#[must_use]
pub const fn select_all() -> &'static str {
    "Select all"
}

/// Removes the ticked entries.
#[must_use]
pub const fn remove_button() -> &'static str {
    "Remove chosen"
}

/// Why Remove is greyed.
#[must_use]
pub const fn names_nothing() -> &'static str {
    "Tick at least one entry to remove."
}

/// Closes the window.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}
