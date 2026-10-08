//! The words for **Security ▸ Remove old passwords…** — the command,
//! the picker, the refusals and the receipt. No sentence ever quotes a value:
//! the engine never reports one, and a receipt that did would leak it.

use super::commands::CommandText;

/// `file.purge_password_values`.
#[must_use]
pub const fn file_purge_password_values() -> CommandText {
    CommandText::new(
        "Remove old passwords…",
        "A form saved more than once can still hold every password ever typed \
         into its password fields, hidden in earlier versions inside the file. \
         This checks for them and writes a new copy with none left. It is \
         refused on a signed document because it rewrites every byte the \
         signature covers.",
    )
}

/// The picker's title.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Save a copy without the stored passwords"
}

/// Nothing to remove.
#[must_use]
pub fn none_found(revisions: usize) -> String {
    let versions = if revisions == 1 {
        "its one saved version".to_owned()
    } else {
        format!("any of its {revisions} saved versions")
    };
    format!("No password field holds a stored value in {versions}. Nothing was written.")
}

/// Versions of the file that could not be read, so were not checked.
#[must_use]
pub fn unreadable(count: usize) -> String {
    let (word, verb) = if count == 1 {
        ("version", "was")
    } else {
        ("versions", "were")
    };
    format!(
        "{count} earlier {word} of the file could not be opened on its own and {verb} not \
         checked. The new copy has only one version, so nothing from them survives in it."
    )
}

/// The receipt's first line: stored values removed, counted by where they were.
#[must_use]
pub fn wrote(path: &str, earlier: usize, current: usize) -> String {
    let total = earlier + current;
    let word = if total == 1 { "value" } else { "values" };
    let found = match (earlier, current) {
        (0, _) => "all in the current version".to_owned(),
        (_, 0) => "all in earlier versions".to_owned(),
        _ => format!("{earlier} in earlier versions, {current} in the current one"),
    };
    format!(
        "Copy written to {path} with {total} stored password {word} removed ({found}). \
         The open document is unchanged."
    )
}

/// Read-only fields that were cleared anyway.
#[must_use]
pub fn read_only_cleared(names: &[String]) -> String {
    format!(
        "Cleared although marked read-only: {}. Read-only stops typing, not removal.",
        names.join(", ")
    )
}

/// Values inherited from a parent field, which the engine leaves in place.
#[must_use]
pub fn inherited_left(names: &[String]) -> String {
    format!(
        "Not removed, because the value belongs to a parent field shared with others: {}. \
         It is still in the new copy.",
        names.join(", ")
    )
}

/// The written copy still holds a value, so it was not kept.
#[must_use]
pub fn still_present(count: usize) -> String {
    let word = if count == 1 { "value" } else { "values" };
    format!("The cleaned copy still held {count} stored password {word}, so it was not written.")
}

/// Refused on a signed document.
#[must_use]
pub fn refused_signed(signatures: usize) -> String {
    let word = if signatures == 1 {
        "signature"
    } else {
        "signatures"
    };
    format!(
        "Not done: the copy is rewritten as one version, which would break this \
         document's {signatures} {word}. Nothing was written."
    )
}

/// The engine refused or the write failed.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Could not remove the stored passwords: {detail}")
}
