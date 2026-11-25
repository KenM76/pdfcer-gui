//! Settings > Display: what a document tab, and a closed document, remember.

/// The setting's name.
#[must_use]
pub const fn tab_mode_title() -> &'static str {
    "Each tab's mode"
}

/// What happens if you never touch it.
#[must_use]
pub const fn tab_mode_silence() -> &'static str {
    "Read, Review and Edit apply to the whole window: switching tabs keeps the mode you are in."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn tab_mode_radius() -> &'static str {
    "Applies at once. A newly opened document starts in the mode you are in. The mode is remembered while the document stays open."
}

/// The checkbox.
#[must_use]
pub const fn tab_mode_label() -> &'static str {
    "Remember the mode of each tab"
}

/// The reopen setting's name.
#[must_use]
pub const fn reopen_title() -> &'static str {
    "Reopening a document"
}

/// What happens if you never touch it.
#[must_use]
pub const fn reopen_silence() -> &'static str {
    "A document always opens at its first page, under the opening fit."
}

/// What it costs, and what it does not affect.
#[must_use]
pub const fn reopen_radius() -> &'static str {
    "Takes effect the next time a document opens. Where you were is recorded when a document is closed or pdfcer quits; the last 200 documents are remembered. A page chosen on the command line, or a document moved between windows, still opens at that page."
}

/// The checkbox.
#[must_use]
pub const fn reopen_label() -> &'static str {
    "Reopen each document at the page, position and zoom it was left at"
}
