//! # `text::bates_batch` — the words for **Bates numbering across files**
//!
//! Tools ▸ Batch ▸ Bates numbering…: its file list, its naming and folder
//! choices, the refusals its preflight reports per file, and the outcome of a
//! run. Design: `docs/modules/pdfcer-gui/dialogs/bates_batch.md`.

use std::path::Path;

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Bates numbering across files"
}

/// The title of the file picker that chooses the documents.
#[must_use]
pub const fn pick_title() -> &'static str {
    "Choose the PDFs to Bates number, in any order"
}

/// The title of the folder picker for the copies.
#[must_use]
pub const fn folder_title() -> &'static str {
    "Choose the folder for the numbered copies"
}

/// The standing rule, first line of the window.
#[must_use]
pub const fn intro() -> &'static str {
    "Numbers every page of each file in the order listed, carrying the count \
     from one file to the next. Each file is written as a numbered copy; the \
     originals are not changed."
}

/// Heading above the file list.
#[must_use]
pub fn files_heading(n: usize) -> String {
    format!("Files, numbered top to bottom ({n})")
}

/// A row's page count while the file is still being read.
#[must_use]
pub const fn reading() -> &'static str {
    "reading\u{2026}"
}

/// A row's page count and label range.
#[must_use]
pub fn row_range(pages: usize, first: &str, last: &str) -> String {
    if first == last {
        format!("{pages} page(s), {first}")
    } else {
        format!("{pages} page(s), {first} to {last}")
    }
}

/// A row's page count when the numbering itself cannot be made.
#[must_use]
pub fn row_pages(pages: usize) -> String {
    format!("{pages} page(s)")
}

/// The copy a row will be written as.
#[must_use]
pub fn row_target(name: &str) -> String {
    format!("as {name}")
}

/// Move a row up.
#[must_use]
pub const fn up() -> &'static str {
    "Up"
}

/// Move a row down.
#[must_use]
pub const fn down() -> &'static str {
    "Down"
}

/// Take a row out of the batch.
#[must_use]
pub const fn remove() -> &'static str {
    "Remove"
}

/// Add more files to the end of the list.
#[must_use]
pub const fn add() -> &'static str {
    "Add files\u{2026}"
}

/// The empty-list line.
#[must_use]
pub const fn empty() -> &'static str {
    "No files. Add the PDFs to number."
}

/// Heading above the copy's folder and name.
#[must_use]
pub const fn output_heading() -> &'static str {
    "Numbered copies"
}

/// Folder choice: beside each original.
#[must_use]
pub const fn beside() -> &'static str {
    "In the same folder as each original"
}

/// Folder choice: one folder for all, with the folder named.
#[must_use]
pub fn in_folder(folder: Option<&Path>) -> String {
    folder.map_or_else(
        || "In one folder (none chosen yet)".to_owned(),
        |f| format!("In one folder: {}", f.display()),
    )
}

/// The button choosing the folder.
#[must_use]
pub const fn choose_folder() -> &'static str {
    "Choose\u{2026}"
}

/// Label of the naming choice.
#[must_use]
pub const fn naming() -> &'static str {
    "File names"
}

/// Naming: the original name unchanged.
#[must_use]
pub const fn name_keep() -> &'static str {
    "The original name"
}

/// Naming: the original name with text before and after it.
#[must_use]
pub const fn name_affix() -> &'static str {
    "The original name with text added"
}

/// Naming: the file's own label range.
#[must_use]
pub const fn name_range() -> &'static str {
    "The file's Bates range"
}

/// Naming: the label range, then the original name.
#[must_use]
pub const fn name_range_and_name() -> &'static str {
    "The Bates range, then the original name"
}

/// Label of the text-before box.
#[must_use]
pub const fn before() -> &'static str {
    "Before the name"
}

/// Label of the text-after box.
#[must_use]
pub const fn after() -> &'static str {
    "After the name"
}

/// The replace-existing choice.
#[must_use]
pub const fn replace_existing() -> &'static str {
    "Replace numbered copies already there"
}

/// Tooltip of [`replace_existing`].
#[must_use]
pub const fn replace_tooltip() -> &'static str {
    "Without this, a run that would write over any existing file stops before \
     writing anything. An original file is never replaced either way."
}

/// The batch's first and last label and the number after it.
#[must_use]
pub fn summary(files: usize, first: &str, last: &str, next: u64) -> String {
    format!("{files} file(s), {first} to {last}. The next number after this batch is {next}.")
}

/// A file the engine would refuse to number, with the engine's reason in the
/// operator's terms.
#[must_use]
pub fn refused(name: &str, why: &str) -> String {
    format!("{name}: {why}")
}

/// The file could not be read as a PDF.
#[must_use]
pub fn unreadable(detail: &str) -> String {
    format!("cannot be read ({detail}). Remove it from the list.")
}

/// The file needs a password to open.
#[must_use]
pub const fn needs_password() -> &'static str {
    "needs a password. Open it, remove its security, save, then add the saved copy."
}

/// The file is signed.
#[must_use]
pub fn signed(signatures: usize) -> String {
    format!(
        "carries {signatures} digital signature(s). Numbering it would add \
         changes after the signing, so it is left to be numbered by itself. \
         Remove it from the list."
    )
}

/// Any other refusal the engine gives before writing.
#[must_use]
pub fn engine_refused(detail: &str) -> String {
    format!("cannot be numbered: {detail}. Remove it from the list.")
}

/// The label holds a character a file name cannot.
#[must_use]
pub const fn label_not_a_name() -> &'static str {
    "The prefix or suffix holds a character a file name cannot (< > : \" / \\ | ? *), \
     so the Bates range cannot name the copies. Choose another naming."
}

/// A copy would be written over its own original.
#[must_use]
pub fn over_original(name: &str) -> String {
    format!(
        "The copy of {name} would replace the original. Add text to the name or \
         choose another folder."
    )
}

/// Two copies would be written to one name.
#[must_use]
pub fn same_target(name: &str) -> String {
    format!("Two files would both be written as {name}. Change the naming.")
}

/// Naming with nothing added names the copy after its original, which is only
/// a new file in another folder.
#[must_use]
pub const fn no_folder() -> &'static str {
    "Choose the folder for the numbered copies."
}

/// The run button.
#[must_use]
pub const fn run() -> &'static str {
    "Number files"
}

/// The cancel and close button.
#[must_use]
pub const fn close() -> &'static str {
    "Close"
}

/// While the run is writing.
#[must_use]
pub fn running(done: usize, of: usize) -> String {
    format!("Numbering file {} of {of}\u{2026}", (done + 1).min(of))
}

/// A run that stopped before writing because copies already exist.
#[must_use]
pub fn exists(names: &[String]) -> String {
    format!(
        "Nothing was written: {} already exist(s). Tick \u{201c}{}\u{201d} or change the naming.",
        names.join(", "),
        replace_existing()
    )
}

/// A run that failed on one file; nothing was written.
#[must_use]
pub fn failed(name: &str, detail: &str) -> String {
    format!("Nothing was written: {name} failed ({detail}).")
}

/// A run whose last step, moving the finished copies into place, failed part
/// way. The copies listed are written; the rest are not.
#[must_use]
pub fn partly_placed(written: usize, of: usize, detail: &str) -> String {
    format!(
        "{written} of {of} numbered copies were written; the rest could not be \
         put in place ({detail})."
    )
}

/// The outcome of a run.
#[must_use]
pub fn done(files: usize, pages: usize, first: &str, last: &str, next: u64) -> String {
    format!(
        "Wrote {files} numbered cop(ies), {pages} page(s), {first} to {last}. The \
         next number is {next}; Bates numbering starts there next time."
    )
}
