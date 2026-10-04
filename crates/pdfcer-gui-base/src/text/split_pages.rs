//! The words for **Pages ▸ Split…** — the window, its preview and its receipt.

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Split Document"
}

/// The sentence at the top of the window.
#[must_use]
pub const fn intro() -> &'static str {
    "Writes this document as several new PDFs, divided where you choose. This \
     document is not changed."
}

/// The heading over the three rules.
#[must_use]
pub const fn rule_heading() -> &'static str {
    "Start a new file"
}

/// The every-N rule's radio, before its number box.
#[must_use]
pub const fn rule_every() -> &'static str {
    "Every"
}

/// The words after the every-N number box.
#[must_use]
pub const fn rule_every_suffix() -> &'static str {
    "pages"
}

/// The after-pages rule's radio, before its page box.
#[must_use]
pub const fn rule_after() -> &'static str {
    "After pages"
}

/// The hint under the after-pages box.
#[must_use]
pub const fn rule_after_hint() -> &'static str {
    "For example 3, 7: the first file ends at page 3, the next at page 7. \
     Pages picked in the thumbnails are filled in when the window opens."
}

/// The top-level-bookmarks rule's radio.
#[must_use]
pub const fn rule_bookmarks() -> &'static str {
    "At each top-level bookmark"
}

/// The hover text on the bookmarks radio when the document has none.
#[must_use]
pub const fn rule_bookmarks_none() -> &'static str {
    "This document has no top-level bookmarks that divide it."
}

/// The file-name pattern box's label.
#[must_use]
pub const fn template_label() -> &'static str {
    "File names"
}

/// The hint under the file-name pattern box.
#[must_use]
pub const fn template_hint() -> &'static str {
    "{stem} is this file's name, {n} the file's number, {start} and {end} its \
     first and last page."
}

/// The output folder box's label.
#[must_use]
pub const fn folder_label() -> &'static str {
    "Folder"
}

/// The button that opens the folder picker.
#[must_use]
pub const fn browse_button() -> &'static str {
    "Browse…"
}

/// The native folder picker's title.
#[must_use]
pub const fn folder_dialog_title() -> &'static str {
    "Choose the folder for the split files"
}

/// The page-labels box.
#[must_use]
pub const fn keep_labels_label() -> &'static str {
    "Keep the page labels"
}

/// The page-labels box's hover text.
#[must_use]
pub const fn keep_labels_tooltip() -> &'static str {
    "On: each page shows the label it shows here, such as iv or A-3. \
     Off: each new file numbers its pages 1, 2, 3."
}

/// The heading over the preview list.
#[must_use]
pub fn preview_heading(files: usize) -> String {
    if files == 1 {
        "This writes 1 file:".to_owned()
    } else {
        format!("This writes {files} files:")
    }
}

/// One preview row: the file name and its pages, 1-based.
#[must_use]
pub fn preview_row(name: &str, first: usize, last: usize) -> String {
    if first == last {
        format!("{name} — page {first}")
    } else {
        format!("{name} — pages {first}-{last}")
    }
}

/// The preview before a rule is chosen.
#[must_use]
pub const fn choose_rule() -> &'static str {
    "Choose where each new file starts."
}

/// The preview when the every-N box does not hold a whole number above 0.
#[must_use]
pub const fn every_invalid() -> &'static str {
    "Type how many pages each file holds, 1 or more."
}

/// The preview when the after-pages box names no page of this document.
#[must_use]
pub fn after_invalid(pages: usize) -> String {
    format!("Type page numbers from 1 to {pages}, such as 3, 7.")
}

/// The preview when the chosen rule divides nothing.
#[must_use]
pub const fn no_split_points() -> &'static str {
    "That rule does not divide this document, so there is nothing to write. \
     A break after the last page, or one bookmark covering every page, \
     leaves it whole."
}

/// The preview when the pattern gives two files the same name.
#[must_use]
pub fn ambiguous_names(first: usize, second: usize) -> String {
    format!(
        "The file-name pattern gives files {first} and {second} the same name. \
         Add {{n}}, {{start}} or {{end}} so each file has its own."
    )
}

/// The preview when the pattern does not end in `.pdf`.
#[must_use]
pub const fn template_not_pdf() -> &'static str {
    "The file-name pattern must end in .pdf."
}

/// The preview when the pattern names a folder or a character Windows refuses.
#[must_use]
pub const fn template_bad_character() -> &'static str {
    "The file-name pattern may not contain \\ / : * ? \" < > or |. Choose the \
     folder in the box below."
}

/// The preview when no folder has been chosen.
#[must_use]
pub const fn folder_missing() -> &'static str {
    "Choose the folder the files are written to."
}

/// The preview when the typed folder does not exist.
#[must_use]
pub const fn folder_not_found() -> &'static str {
    "That folder does not exist."
}

/// The preview when one of the files would be this document itself.
#[must_use]
pub fn would_overwrite_source(name: &str) -> String {
    format!(
        "{name} is this document. Change the file names or the folder so it \
         is not written over."
    )
}

/// The preview line when some of the files already exist.
#[must_use]
pub fn replaces_existing(count: usize) -> String {
    if count == 1 {
        "1 of these files already exists in that folder and will be replaced.".to_owned()
    } else {
        format!("{count} of these files already exist in that folder and will be replaced.")
    }
}

/// The preview's engine refusal not otherwise worded here.
#[must_use]
pub fn plan_failed(detail: &str) -> String {
    format!("This document cannot be split: {detail}.")
}

/// The preview when the window's document is no longer the one on screen.
#[must_use]
pub fn other_document(name: &str) -> String {
    format!("This window splits {name}. Switch back to it to split it.")
}

/// The button that writes the files.
#[must_use]
pub const fn split_button() -> &'static str {
    "Split"
}

/// The receipt once every file is written.
#[must_use]
pub fn wrote(files: usize, folder: &str) -> String {
    if files == 1 {
        format!("Wrote 1 file to {folder}.")
    } else {
        format!("Wrote {files} files to {folder}.")
    }
}

/// The receipt's second sentence when the page labels were carried.
#[must_use]
pub const fn labels_kept() -> &'static str {
    "Each page keeps the label it shows here."
}

/// The receipt's second sentence when the page labels were left out.
#[must_use]
pub const fn labels_dropped() -> &'static str {
    "The page labels were left out, so each file numbers its pages from 1."
}

/// The receipt when a file could not be made or written. `written` files
/// before it are on disk.
#[must_use]
pub fn failed(written: usize, detail: &str) -> String {
    match written {
        0 => format!("Nothing was split. {detail}"),
        1 => format!("The split stopped after 1 file. {detail}"),
        n => format!("The split stopped after {n} files. {detail}"),
    }
}
