//! # `text::ocrfetch` — the download copy inside File ▸ Recognise ▸ Manage OCR models…

/// The line under each recogniser: whose files they are and their licence.
#[must_use]
pub fn licence_line(creator: &str, source: &str, licence: &str) -> String {
    format!("By {creator}, from {source}. Licence {licence}.")
}

/// The download button.
#[must_use]
pub const fn download_button() -> &'static str {
    "Download"
}

/// A pinned set not installed: its label and, when stated, its languages.
#[must_use]
pub fn available(label: &str, languages: &str) -> String {
    if languages.is_empty() {
        format!("Available to download: {label}.")
    } else {
        format!("Available to download: {label}, languages {languages}.")
    }
}

/// After every file is saved; `attribution` is the licence's required line.
#[must_use]
pub fn done(label: &str, files: usize, attribution: &str) -> String {
    format!(
        "Downloaded and checked {label} ({files} file(s)). Recognise text can use it from \
         now on. These model files are under {attribution}."
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
