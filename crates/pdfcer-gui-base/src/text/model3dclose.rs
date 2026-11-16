//! Strings for the question asked before a document, or the program, closes
//! while its 3D model window is open. Consumed by
//! `pdfcer_gui::dialogs::close_model`.

/// The window title.
#[must_use]
pub const fn title() -> &'static str {
    "Close the 3D model window?"
}

/// The question, when the document's tab is closing; `name` is its file name.
#[must_use]
pub fn question_tab(name: &str) -> String {
    format!("The 3D model window for {name} is open. Closing the document closes it too.")
}

/// The question, when the program is closing.
#[must_use]
pub const fn question_quit() -> &'static str {
    "A 3D model window is open. Closing pdfcer closes it too."
}

/// Why there is no answer that keeps the 3D window open on its own.
#[must_use]
pub const fn why_not_kept() -> &'static str {
    "The 3D window reads its saved views from the document and writes views, \
     measurements and pictures into it, so it cannot stay open without it."
}

/// The affirmative button, when the document's tab is closing.
#[must_use]
pub const fn close_both() -> &'static str {
    "Close both"
}

/// Its hover text.
#[must_use]
pub const fn close_both_hover() -> &'static str {
    "Close the 3D model window, then the document"
}

/// The affirmative button, when the program is closing.
#[must_use]
pub const fn close() -> &'static str {
    "Close"
}

/// Its hover text.
#[must_use]
pub const fn close_hover() -> &'static str {
    "Close the 3D model window, then pdfcer"
}

/// The cancelling button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// Its hover text.
#[must_use]
pub const fn cancel_hover() -> &'static str {
    "Keep both open"
}

/// The viewer's note while another document's tab is on screen.
#[must_use]
pub const fn other_document() -> &'static str {
    "Another document is on screen. Switch back to this model's document to \
     save views, measurements or a picture into it."
}

/// A file name for a document that has none.
#[must_use]
pub const fn untitled() -> &'static str {
    "this document"
}
