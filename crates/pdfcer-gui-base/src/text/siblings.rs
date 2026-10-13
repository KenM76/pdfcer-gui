//! The words for moving a document to another pdfcer-gui window.
//!
//! A document moves with its unsaved edits and the password it was opened
//! with, so the refusals here are the few things that cannot travel: a
//! document with no file behind it, and a staged redaction.

use std::borrow::Cow;

/// Why a document, or a selection dropped on another window, did not move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowMoveRefusal {
    /// The document has no file behind it: it was made here and never
    /// saved, or its file is gone.
    NoFile,
    /// A redaction is staged and not yet saved.
    RedactionStaged,
    /// The copy carrying the unsaved edits could not be written; the field
    /// is the reason.
    NotCarried(String),
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
            Self::NoFile => Cow::Borrowed(
                "Save this document first: it has no file another window could open. Nothing \
                 was moved.",
            ),
            Self::RedactionStaged => Cow::Borrowed(
                "Save or cancel the staged redaction first: the content it removes must not \
                 leave this window. Nothing was moved.",
            ),
            Self::NotCarried(why) => Cow::Owned(format!(
                "pdfcer could not hand over the unsaved edits: {why}. The document is still \
                 open here."
            )),
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

/// The receiving window's reason when it did not answer in time; shown by
/// the sending window.
#[must_use]
pub const fn no_answer() -> &'static str {
    "it did not answer in time"
}

/// The receiving window's reason when it already has the file open: a second
/// copy carrying other edits would make two documents of one file.
#[must_use]
pub const fn already_open() -> &'static str {
    "it already has this file open"
}

/// The receiving window's reason when the document did not open there.
#[must_use]
pub const fn not_opened() -> &'static str {
    "the document did not open there"
}

/// Said in the window a document arrived in with unsaved edits: the edits
/// came along, and the steps that made them stayed behind.
#[must_use]
pub fn arrived_with_edits(edits: u64) -> String {
    format!(
        "Moved here with {edits} unsaved edit{}. They are kept and saved with the document, but \
         cannot be undone in this window.",
        if edits == 1 { "" } else { "s" }
    )
}

/// Why unsaved edits could not travel: there is no settings folder to write
/// their copy in.
#[must_use]
pub const fn no_carry_folder() -> &'static str {
    "there is no settings folder to write them to"
}
