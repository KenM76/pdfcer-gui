//! # `text::print::batch` — printing several files with one setup
//!
//! The words for `pdfcer_gui::dialogs::print::batch`: the file list at the
//! top of the Print window when it was opened from Tools ▸ Batch ▸ Batch
//! print…, and the one sentence a finished or partial batch leaves.

/// Title of the picker that chooses the files.
#[must_use]
pub const fn pick_title() -> &'static str {
    "Choose the PDFs to print"
}

/// The heading over the file list.
#[must_use]
pub fn heading(files: usize) -> String {
    match files {
        1 => "Printing 1 file with these settings".to_owned(),
        n => format!("Printing {n} files with these settings, in this order"),
    }
}

/// Hover text on the heading.
#[must_use]
pub const fn heading_tooltip() -> &'static str {
    "Each file prints as its own job, with the printer, paper, pages and \
     layout chosen here. The preview shows the first file."
}

/// One file in the list, with its page count.
#[must_use]
pub fn file_row(name: &str, pages: usize) -> String {
    match pages {
        1 => format!("{name} — 1 page"),
        n => format!("{name} — {n} pages"),
    }
}

/// A file that could not be opened, and is left out.
#[must_use]
pub fn skipped_row(name: &str, why: &str) -> String {
    format!("{name} — left out: {why}")
}

/// Why a file was left out: it needs a password.
#[must_use]
pub const fn needs_password() -> &'static str {
    "it needs a password. Open it and print it on its own."
}

/// Why one file did not print: the printer had not answered for its paper.
#[must_use]
pub const fn printer_not_ready() -> &'static str {
    "the printer did not report its settings for this file's paper"
}

/// Why one file did not print: the chosen pages are not in it.
#[must_use]
pub const fn no_pages_chosen() -> &'static str {
    "none of the chosen pages are in this file"
}

/// The note a finished batch leaves.
#[must_use]
pub fn sent(files: usize, pages: usize) -> String {
    match (files, pages) {
        (1, 1) => "Sent 1 file, 1 page, to the printer.".to_owned(),
        (1, p) => format!("Sent 1 file, {p} pages, to the printer."),
        (f, p) => format!("Sent {f} files, {p} pages, to the printer."),
    }
}

/// What a batch that stopped part-way reports. The files that printed are
/// taken off the list, so pressing Print again prints only the rest.
#[must_use]
pub fn stopped(printed: usize, total: usize, name: &str, why: &str) -> String {
    format!(
        "{printed} of {total} files sent. {name} did not print: {why}. Print again to send \
         the files still listed."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_agree_with_their_nouns() {
        assert!(heading(1).contains("1 file "));
        assert!(heading(3).contains("3 files"));
        assert_eq!(sent(1, 1), "Sent 1 file, 1 page, to the printer.");
        assert!(sent(2, 7).contains("2 files, 7 pages"));
    }
}
