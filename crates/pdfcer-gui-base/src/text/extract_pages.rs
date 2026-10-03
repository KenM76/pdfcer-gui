//! The words for **Pages ▸ Extract…** — the window and its receipt.

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Extract Pages"
}

/// The sentence at the top of the window.
#[must_use]
pub const fn intro() -> &'static str {
    "Writes the chosen pages as a new PDF. This document is not changed unless \
     you ask for the pages to be deleted from it."
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
     Off: the new file numbers its pages 1, 2, 3."
}

/// The delete-afterwards box.
#[must_use]
pub const fn delete_after_label() -> &'static str {
    "Delete these pages from this document afterwards"
}

/// The delete-afterwards box's hover text.
#[must_use]
pub const fn delete_after_tooltip() -> &'static str {
    "One undo step. Nothing is deleted if the new file is not written."
}

/// The button that opens the save picker.
#[must_use]
pub const fn extract_button() -> &'static str {
    "Extract…"
}

/// The receipt once the file is written: how many pages, and where.
#[must_use]
pub fn wrote(pages: usize, file: &str) -> String {
    if pages == 1 {
        format!("Wrote 1 page to {file}.")
    } else {
        format!("Wrote {pages} pages to {file}.")
    }
}

/// The receipt's second sentence when the source has page labels and they
/// were carried.
#[must_use]
pub const fn labels_kept() -> &'static str {
    "Each page keeps the label it shows here."
}

/// The receipt's second sentence when the source's page labels were left out.
#[must_use]
pub const fn labels_dropped() -> &'static str {
    "The page labels were left out, so the new file numbers its pages from 1."
}

/// The receipt when the file could not be made or written.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("The pages were not extracted. {detail}")
}
