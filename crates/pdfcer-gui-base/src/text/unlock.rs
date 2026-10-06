//! Every sentence about edits the password a document was opened with does
//! not permit, and the command that reopens it for its owner password.

use super::commands::CommandText;

/// `file.unlock`.
#[must_use]
pub const fn reopen() -> CommandText {
    CommandText::new(
        "Reopen with owner password…",
        "This file was opened with a password that does not allow every change. Reopening it \
         with its owner password allows them all. Unavailable while the file has unsaved \
         changes: save them first.",
    )
}

/// The decline when the opening password does not permit an edit.
#[must_use]
pub const fn refused() -> &'static str {
    "Not changed: the password this file was opened with does not allow this change. Reopen \
     it with the owner password to change it."
}
