//! The sentences for File ▸ Security ▸ Add validation evidence…, which embeds
//! certificates, CRLs and OCSP responses in a signed document's `/DSS` so its
//! signatures can be checked later without going online (PAdES B-LT).

use crate::text::commands::CommandText;

/// `file.add_validation_evidence` — the label and the hover.
#[must_use]
pub const fn file_add_validation_evidence() -> CommandText {
    CommandText::new(
        "Add validation evidence…",
        "Embed certificates, revocation lists (CRLs) and OCSP responses in this \
         signed document, so its signatures can still be checked years from now \
         without going online. Choose the files; the signers' own certificates \
         are added too. pdfcer downloads nothing. Save afterwards to keep it: \
         the signatures stay valid. Refused on an unsigned document.",
    )
}

/// The picker's title.
#[must_use]
pub const fn picker_title() -> &'static str {
    "Choose certificates, CRLs and OCSP responses to embed"
}

/// The picker's filter name.
#[must_use]
pub const fn picker_filter() -> &'static str {
    "Certificates, CRLs and OCSP responses"
}

/// The status line after a write.
#[must_use]
pub fn added(certs: usize, crls: usize, ocsps: usize, own_certs: usize) -> String {
    format!(
        "Embedded {certs} certificate(s) ({own_certs} from the signatures themselves), \
         {crls} CRL(s) and {ocsps} OCSP response(s). Save to keep them; the signatures \
         stay valid. Press Ctrl+Z to undo."
    )
}

/// Appended when some supplied blobs were already present.
#[must_use]
pub fn duplicates(n: usize) -> String {
    format!("{n} item(s) were already in the document and were not added again.")
}

/// Appended when bare OCSP answers were wrapped.
#[must_use]
pub fn wrapped(n: usize) -> String {
    format!(
        "{n} OCSP answer(s) were bare and were wrapped in the form the standard \
         requires; their content is unchanged."
    )
}

/// Nothing new: everything supplied was already present.
#[must_use]
pub const fn nothing_new() -> &'static str {
    "Everything chosen is already in the document: nothing was added, and the \
     document is unchanged."
}

/// Refused: no signature.
#[must_use]
pub const fn unsigned() -> &'static str {
    "This document has no signature, so there is nothing for validation evidence \
     to support. The document is unchanged."
}

/// Refused: certified with no changes permitted.
#[must_use]
pub const fn no_changes_certified() -> &'static str {
    "This document is certified with no changes permitted. The standard allows \
     adding validation evidence even so, but Acrobat is reported to count it as a \
     change and mark the certification broken, so pdfcer does not add it. The \
     document is unchanged."
}

/// Refused: one file could not be read.
#[must_use]
pub fn file_unreadable(file: &str, why: &str) -> String {
    format!("{file} could not be read: {why}. Nothing was added.")
}

/// Refused: the engine could not store one blob.
#[must_use]
pub fn blob_refused(file: &str, why: &str) -> String {
    format!("{file} cannot be embedded: {why}. Nothing was added.")
}

/// Refused for any other engine reason.
#[must_use]
pub fn refused(why: &str) -> String {
    format!("Validation evidence was not added: {why}. The document is unchanged.")
}

/// The reason phrase for a PEM body that does not decode.
#[must_use]
pub const fn why_bad_base64() -> &'static str {
    "its PEM text is damaged"
}

/// The reason phrase for a PEM block of the wrong kind.
#[must_use]
pub fn why_unknown_label(label: &str) -> String {
    format!("it holds a {label}, which is not a certificate, CRL or OCSP response")
}

/// The reason phrase for a PEM block with no end line.
#[must_use]
pub const fn why_unterminated() -> &'static str {
    "a PEM block has no END line"
}

/// The reason phrase for an empty file.
#[must_use]
pub const fn why_empty() -> &'static str {
    "the file is empty"
}
