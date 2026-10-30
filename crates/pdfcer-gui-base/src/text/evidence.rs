//! The sentences for Security ▸ Add validation evidence…, which embeds
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
         can be added too. pdfcer downloads nothing. Save afterwards to keep it: \
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

/// The options window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Add validation evidence"
}

/// What the window does, above its options.
#[must_use]
pub fn intro(signatures: usize) -> String {
    format!(
        "This document has {signatures} signature(s). Choose the certificate, \
         revocation list (CRL) and OCSP response files to embed; pdfcer downloads \
         nothing. Save afterwards to keep them: the signatures stay valid."
    )
}

/// The checkbox that includes the signers' own certificates.
#[must_use]
pub const fn own_certs() -> &'static str {
    "Include the signers' own certificates"
}

/// Hover on the signers' certificates checkbox.
#[must_use]
pub const fn own_certs_hover() -> &'static str {
    "Copy every certificate already inside each signature into the evidence as \
     well, so the whole chain sits in one place, as the PDF standard describes. \
     Leave this on unless you supply those certificates yourself."
}

/// The warning shown on a document certified with no changes permitted.
#[must_use]
pub const fn no_changes_certified() -> &'static str {
    "This document is certified with no changes permitted. The PDF standard \
     allows adding validation evidence even so, but Acrobat is reported to count \
     it as a change and show the certification as broken."
}

/// The checkbox that accepts that warning.
#[must_use]
pub const fn allow_anyway() -> &'static str {
    "Add it anyway"
}

/// Hover on Choose files… while the warning is not accepted.
#[must_use]
pub const fn needs_allow() -> &'static str {
    "Tick \"Add it anyway\" first: this document is certified with no changes permitted."
}

/// The button that goes on to the file picker.
#[must_use]
pub const fn choose_button() -> &'static str {
    "Choose files…"
}

/// The button that closes the window.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}
