//! # `text::ocrfetch` — the copy for File ▸ Recognise ▸ Download OCR models…

use crate::text::commands::CommandText;

/// `file.fetch_ocr_models` — the label and the hover.
#[must_use]
pub const fn file_fetch_ocr_models() -> CommandText {
    CommandText::new(
        "Download OCR models…",
        "Download the model files a text recogniser needs into the models folder beside \
         pdfcer, checking each against its published fingerprint before it is saved. Use \
         it when Recognise text says a model is missing or damaged. Needs the network.",
    )
}

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Download OCR models"
}

/// The line naming one downloadable recogniser and how many files it has.
#[must_use]
pub fn engine_line(label: &str, files: usize) -> String {
    format!("{label}: {files} file(s), checked against their published fingerprints")
}

/// Where the files go.
#[must_use]
pub fn target_line(dir: &str) -> String {
    format!("Saved to {dir}")
}

/// The line under each recogniser: whose files they are and their licence.
#[must_use]
pub fn licence_line(creator: &str, source: &str, licence: &str) -> String {
    format!("By {creator}, from {source}. Licence {licence}.")
}

/// When the program's folder cannot be found, so there is nowhere to save.
#[must_use]
pub const fn no_target() -> &'static str {
    "pdfcer cannot find its own folder, so it has nowhere to save the models."
}

/// The download button.
#[must_use]
pub const fn download_button() -> &'static str {
    "Download"
}

/// The download button's hover.
#[must_use]
pub const fn download_hover() -> &'static str {
    "Replace this recogniser's model files with fresh copies. A file is saved only after \
     it matches its fingerprint."
}

/// The download button's hover while a download is running.
#[must_use]
pub const fn busy_hover() -> &'static str {
    "A download is already running."
}

/// While the files are coming.
#[must_use]
pub const fn running() -> &'static str {
    "Downloading…"
}

/// After every file is saved; `attribution` is the licence's required line.
#[must_use]
pub fn done(files: usize, attribution: &str) -> String {
    format!(
        "Downloaded and checked {files} file(s). Recognise text uses them from now on. \
         These model files are under {attribution}."
    )
}

/// After a failure; `why` is the engine's words.
#[must_use]
pub fn failed(why: &str) -> String {
    format!("The download did not finish: {why}. Files already checked and saved are kept.")
}

/// The failure words when the download thread ended without an answer.
#[must_use]
pub const fn stopped() -> &'static str {
    "the download stopped unexpectedly"
}

/// The close button.
#[must_use]
pub const fn close_button() -> &'static str {
    "Close"
}
