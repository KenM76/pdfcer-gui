//! The sentences for Security ▸ Add archive time-stamp…, which writes a
//! copy of the document sealed by a document time-stamp (PAdES B-LTA when the
//! document is signed and carries validation evidence).

use crate::text::commands::CommandText;

/// Added to the file name the save picker suggests.
pub const SUFFIX: &str = "-archived"; // ui-text-exempt: a file-name suffix

/// `file.add_archive_timestamp` — the label and the hover.
#[must_use]
pub const fn file_add_archive_timestamp() -> CommandText {
    CommandText::new(
        "Add archive time-stamp…",
        "Save a copy sealed by a time-stamp from a time-stamping server, so the \
         whole file — its signatures and the evidence that validates them — can \
         be shown unchanged since that moment. On a signed document with \
         validation evidence this is the long-term archive form (PAdES B-LTA). \
         Needs the network. The open document is not changed.",
    )
}

/// The window's title, and the save picker's.
#[must_use]
pub const fn window_title() -> &'static str {
    "Add archive time-stamp"
}

/// What the window says the stamp will be, from what the document holds now.
#[must_use]
pub fn expected(signatures: usize) -> String {
    if signatures == 0 {
        "This document is not signed, so the copy gets a plain document \
         time-stamp: proof of when the file existed in this form."
            .to_owned()
    } else {
        format!(
            "This document has {signatures} signature(s). The time-stamp seals them \
             and the validation evidence already embedded. Add validation evidence \
             first if you have not, or the copy is not the full archive form."
        )
    }
}

/// The server field's label.
#[must_use]
pub const fn server_label() -> &'static str {
    "Time-stamping server:"
}

/// Hover on the server field.
#[must_use]
pub const fn server_hover() -> &'static str {
    "The address of an RFC 3161 time-stamping server, for example \
     http://timestamp.digicert.com. The same server the Sign window uses."
}

/// The button that goes on to the save picker.
#[must_use]
pub const fn save_button() -> &'static str {
    "Time-stamp and save as…"
}

/// Hover on the save button while the server field is blank.
#[must_use]
pub const fn needs_server() -> &'static str {
    "Enter a time-stamping server first."
}

/// Cancel.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// Status line: the copy was written.
#[must_use]
pub fn written(path: &str, time: &str, authority: &str, level: &str) -> String {
    format!("Saved a time-stamped copy to {path}. Time-stamp {time}, from {authority}. {level}")
}

/// What the written stamp is, from the engine's report.
#[must_use]
pub fn level(prior_signatures: usize, dss_present: bool) -> String {
    match (prior_signatures, dss_present) {
        (0, _) => "The document had no signature, so this is a document time-stamp \
                   rather than a signature archive."
            .to_owned(),
        (_, false) => "The document had no validation evidence, so the signatures \
                       it seals cannot be checked offline later: add validation \
                       evidence, then time-stamp again, for the archive form."
            .to_owned(),
        (n, true) => format!(
            "Archive form (PAdES B-LTA): {n} signature(s) and their validation \
             evidence are sealed. pdfcer did not check that the evidence is \
             complete for every signature."
        ),
    }
}

/// The picker named the open document itself.
#[must_use]
pub const fn not_the_source() -> &'static str {
    "The time-stamped copy cannot replace the open document. Choose another name."
}

/// The server could not be asked in this build.
#[must_use]
pub const fn unavailable() -> &'static str {
    "This build of pdfcer cannot reach a time-stamping server."
}

/// The engine or the server refused.
#[must_use]
pub fn refused(detail: &str) -> String {
    format!("The time-stamp was not added: {detail}")
}

/// The copy could not be written.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("The time-stamped copy could not be written: {detail}")
}

/// Another part of the program still holds the document.
#[must_use]
pub const fn busy() -> &'static str {
    "Another part of pdfcer is still using this document. Try again."
}
