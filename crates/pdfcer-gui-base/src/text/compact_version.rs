//! # `text::compact_version` — *Make compatible with* in Save a compacted copy
//!
//! Every word of the version list and of the lines saying what lowering the
//! copy's version did or why it was refused.

use pdfcer_core::PdfVersion;

/// The list's label.
#[must_use]
pub const fn label() -> &'static str {
    "Make compatible with"
}

/// Hover on the list.
#[must_use]
pub const fn tooltip() -> &'static str {
    "Mark the copy as an older PDF version, for a reader or a print shop that refuses newer \
     ones. The copy is changed only when nothing in it needs a newer version; otherwise it is \
     saved as it is and the window says what needs more. A copy that packs its index is \
     rewritten with a plain one below PDF 1.5, which is larger and numbers its parts afresh."
}

/// One entry: *Keep* or a version.
#[must_use]
pub fn choice(target: Option<PdfVersion>) -> String {
    match target {
        None => "Keep its version".to_owned(),
        Some(v) => format!("PDF {v}"),
    }
}

/// Shown while the copy's version is being lowered.
#[must_use]
pub const fn working() -> &'static str {
    "Checking what the copy needs…"
}

/// Lowered.
#[must_use]
pub fn lowered(from: &PdfVersion, to: &PdfVersion) -> String {
    format!("The copy is marked PDF {to} instead of PDF {from}; nothing in it needs more.")
}

/// A catalog version was removed with the header change.
#[must_use]
pub const fn catalog_removed() -> &'static str {
    "A second version mark inside the file was removed, so readers see only the new one."
}

/// The copy was rewritten with a plain index.
#[must_use]
pub const fn rebuilt() -> &'static str {
    "The copy was rewritten with a plain index, which older readers need: it is larger, and \
     its parts are numbered afresh."
}

/// One construct and the version it needs.
#[must_use]
pub fn needs(version: &PdfVersion, construct: &str, objects: usize) -> String {
    format!("Needs PDF {version}: {construct}, in {objects} part(s).")
}

/// The content needs more than the version chosen.
#[must_use]
pub fn below_floor(target: &PdfVersion, floor: &PdfVersion) -> String {
    format!(
        "Not marked PDF {target}: the copy uses things that arrived in PDF {floor}, so it is \
         saved at its own version."
    )
}

/// The version chosen is not older than the copy's.
#[must_use]
pub fn not_lower(target: &PdfVersion, declared: &PdfVersion) -> String {
    format!(
        "The copy is already PDF {declared}, which is not newer than PDF {target}; it is saved as it is."
    )
}

/// Some streams could not be read, so the floor is not proven.
#[must_use]
pub fn unproven(floor: &PdfVersion, unreadable: usize) -> String {
    format!(
        "Not lowered: {unreadable} part(s) of the copy could not be read, so it may need more \
         than PDF {floor}. It is saved at its own version."
    )
}

/// The copy is encrypted.
#[must_use]
pub const fn encrypted() -> &'static str {
    "Not lowered: the copy is encrypted. Remove the password, save the copy, then encrypt it again."
}

/// The engine's own sentence for any other failure.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Not lowered: {detail}. The copy is saved at its own version.")
}

/// The thread ended without an answer.
#[must_use]
pub const fn stopped() -> &'static str {
    "Not lowered: the check stopped without an answer. The copy is saved at its own version."
}
