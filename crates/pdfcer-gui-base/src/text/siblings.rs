//! The words for moving a document to another pdfcer-gui window.
//!
//! A move hands the receiving window the file's path and it opens the file
//! from disk, so every sentence here that can be about edits says the edits
//! must be saved first: that is the one limit the operator can act on.

use std::borrow::Cow;

/// Why a document, or a selection dropped on another window, did not move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowMoveRefusal {
    /// The document has edits that are not on disk, or was never saved.
    Unsaved,
    /// The other window did not take it; the field is the reason.
    NotSent(String),
    /// No new window could be started; the field is the reason.
    NotStarted(String),
    /// The window a selection was dropped on did not paste it; the field is
    /// the reason.
    SelectionNotSent(String),
}

impl WindowMoveRefusal {
    /// The sentence for this refusal.
    #[must_use]
    pub fn line(&self) -> Cow<'static, str> {
        match self {
            Self::Unsaved => Cow::Borrowed(
                "Save this document first. Another window opens it from the saved file, so \
                 unsaved edits would be left behind. Nothing was moved.",
            ),
            Self::NotSent(why) => Cow::Owned(format!(
                "The other window did not take the document: {why}. It is still open here."
            )),
            Self::NotStarted(why) => Cow::Owned(format!(
                "pdfcer could not open a new window: {why}. The document is still open here."
            )),
            Self::SelectionNotSent(why) => Cow::Owned(format!(
                "The other window did not take the selection: {why}. Nothing changed here."
            )),
        }
    }
}

/// The picker window's title.
#[must_use]
pub const fn picker_title() -> &'static str {
    "Move to which window?"
}

/// The sentence above the picker's buttons.
#[must_use]
pub const fn picker_prompt() -> &'static str {
    "The document closes here and opens in the window you pick."
}

/// One picker button: the documents the other window has open, or that it
/// has none.
#[must_use]
pub fn picker_window(documents: &str) -> String {
    if documents.is_empty() {
        "A window with no document open".to_owned()
    } else {
        format!("The window with {documents}")
    }
}

/// The picker's way out.
#[must_use]
pub const fn picker_cancel() -> &'static str {
    "Cancel"
}

/// What joins the document names a window publishes.
#[must_use]
pub const fn document_separator() -> &'static str {
    ", "
}

/// The receiving window's reason when a request is not one it understands;
/// it is shown by the sending window.
#[must_use]
pub const fn unreadable_request() -> &'static str {
    "it could not read the request"
}

/// Why a path is refused before it is sent: its name is not valid Unicode.
#[must_use]
pub const fn unsendable_name() -> &'static str {
    "the file's name cannot be passed to another window"
}

/// Why a move found nowhere to go: the other windows closed since the
/// command was offered.
#[must_use]
pub const fn no_other_window() -> &'static str {
    "no other pdfcer window is open"
}
