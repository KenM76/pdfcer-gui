//! # `text::sign` — every operator-facing string on the control that puts the
//! operator's own signature into a document
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/sign.md`.

use crate::text::commands::CommandText;

// ===========================================================================
// THE RIBBON CONTROL
// ===========================================================================

/// `file.sign` — the label and the hover.
#[must_use]
pub const fn file_sign() -> CommandText {
    CommandText::new(
        "Sign…",
        "Put your own digital signature on this document, using a certificate \
         file (.pfx or .p12) and its passphrase. It writes a new signed file \
         rather than changing the one you have open. It is refused on an \
         encrypted document, and on one with a redaction waiting to be applied.",
    )
}

// ===========================================================================
// THE WINDOW
// ===========================================================================

/// The window title.
#[must_use]
pub const fn title() -> &'static str {
    "Sign document"
}

/// The framing sentence, above everything.
#[must_use]
pub const fn intro() -> &'static str {
    "Signing appends your signature to the end of the file, leaving every byte \
     already in it exactly where it is. Anyone who opens the result can check \
     that the part your signature covers has not been altered since."
}

// ---------------------------------------------------------------------------
// The refusals — shown INSTEAD of the form
// ---------------------------------------------------------------------------

/// The heading above a refusal, so the window does not read as a failure.
#[must_use]
pub const fn refusal_heading() -> &'static str {
    "This document cannot be signed yet"
}

/// [`crate::sign::Refusal::RedactionPending`].
#[must_use]
pub const fn refusal_redaction_pending() -> &'static str {
    "A redaction is armed on this document and has not been applied yet. \
     Signing now would sign the version that still contains what you marked \
     for removal, so pdfcer refuses it. Apply the redaction, or call it off, \
     and then sign."
}

/// [`crate::sign::Refusal::Encrypted`].
#[must_use]
pub const fn refusal_encrypted() -> &'static str {
    "This document is encrypted. A signature has to be added to the end of the \
     file, and pdfcer cannot append to an encrypted one — so an encrypted \
     document cannot be signed at all. Take the password off first (File > \
     Security > Encrypt…), sign, and put it back on afterwards."
}

/// [`crate::sign::Refusal::CertificationForbids`].
#[must_use]
pub fn refusal_certification_forbids(permission: u8) -> String {
    format!(
        "Somebody has certified this document with a permission setting \
         (/DocMDP {permission}) that allows no changes at all, including \
         adding a signature. Only whoever certified it can change that."
    )
}

/// [`crate::sign::Refusal::RecoveredBase`].
#[must_use]
pub const fn refusal_recovered_base() -> &'static str {
    "pdfcer had to rebuild this file's index when it opened it, because the \
     one in the file was damaged. Nothing can be safely appended to a file in \
     that state, and a signature has to be appended. Save a copy first (File > \
     Save a copy…) and sign that."
}

/// [`crate::sign::Refusal::NotOnDisk`].
#[must_use]
pub const fn refusal_not_on_disk() -> &'static str {
    "This document has never been saved. A signature is an addition to a file \
     that already exists, so there is nothing yet for it to be added to. Save \
     it first, then sign it."
}

/// **The sentence for each refusal.**
#[must_use]
pub fn refusal_line(refusal: crate::sign::Refusal) -> String {
    use crate::sign::Refusal;
    match refusal {
        Refusal::RedactionPending => refusal_redaction_pending().to_owned(),
        Refusal::Encrypted => refusal_encrypted().to_owned(),
        Refusal::CertificationForbids { permission } => refusal_certification_forbids(permission),
        Refusal::RecoveredBase => refusal_recovered_base().to_owned(),
        Refusal::NotOnDisk => refusal_not_on_disk().to_owned(),
    }
}

/// How many signatures the document already carries, when it carries any.
#[must_use]
pub fn already_signed(count: usize) -> String {
    if count == 1 {
        "This document already carries one signature. Yours will be added \
         beside it; the existing one stays valid, because nothing already in \
         the file moves."
            .to_owned()
    } else {
        format!(
            "This document already carries {count} signatures. Yours will be \
             added beside them; the existing ones stay valid, because nothing \
             already in the file moves."
        )
    }
}

// ---------------------------------------------------------------------------
// The certificate
// ---------------------------------------------------------------------------

/// The section heading for the identity.
#[must_use]
pub const fn certificate_heading() -> &'static str {
    "Your certificate"
}

/// The button that opens the file picker.
#[must_use]
pub const fn choose_certificate() -> &'static str {
    "Choose certificate…"
}

/// What stands where the path goes before one is chosen.
#[must_use]
pub const fn certificate_none_chosen() -> &'static str {
    "No certificate chosen yet."
}

/// The file-picker window's title.
#[must_use]
pub const fn certificate_picker_title() -> &'static str {
    "Choose a certificate file"
}

/// The file-picker filter's name.
#[must_use]
pub const fn certificate_filter() -> &'static str {
    "Certificate file (*.pfx, *.p12)"
}

/// The passphrase field's label.
#[must_use]
pub const fn passphrase_label() -> &'static str {
    "Passphrase"
}

/// What is done with the passphrase, said where it is typed.
#[must_use]
pub const fn passphrase_note() -> &'static str {
    "Your passphrase is used to open the certificate and is not saved, written \
     to any log, or kept after this window closes."
}

/// The button that opens the certificate so its contents can be shown.
#[must_use]
pub const fn open_certificate() -> &'static str {
    "Open certificate"
}

/// The heading above what was found inside the container.
#[must_use]
pub const fn identity_heading() -> &'static str {
    "This certificate says:"
}

/// The signer's own subject line, as the engine renders the DN.
#[must_use]
pub fn identity_subject(subject: &str) -> String {
    format!("Signed by: {subject}")
}

/// The `friendlyName` bag attribute, when the container carries one.
#[must_use]
pub fn identity_friendly_name(name: &str) -> String {
    format!("Stored as: {name}")
}

/// The key kind and the chain length.
#[must_use]
pub fn identity_key(key: &str, chain_length: usize) -> String {
    if chain_length == 1 {
        format!("Key: {key}, with the signer's certificate only")
    } else {
        format!("Key: {key}, with a chain of {chain_length} certificates")
    }
}

/// **Whether the container's integrity was checked, and what it means when
/// it was not.**
#[must_use]
pub fn identity_integrity(mac: Option<&str>) -> String {
    match mac {
        Some(mac) => format!(
            "Integrity: checked — the file's own {mac} checksum matched, so it \
             has not been altered since it was exported."
        ),
        None => "Integrity: not checked — this certificate file carries no \
                 checksum of its own, so pdfcer cannot tell whether it has been \
                 altered since it was exported. Your passphrase opened the key, \
                 which is the only assurance there is here."
            .to_owned(),
    }
}

/// Certificates in the container that belonged to no chain and were dropped.
#[must_use]
pub fn identity_unrelated(count: usize) -> String {
    if count == 1 {
        "One other certificate in the file does not belong to this key's chain \
         and will not be included."
            .to_owned()
    } else {
        format!(
            "{count} other certificates in the file do not belong to this \
             key's chain and will not be included."
        )
    }
}

/// The file could not be read at all — [`crate::sign::IdentityFailure::Unreadable`].
#[must_use]
pub fn identity_unreadable(detail: &str) -> String {
    format!("That certificate file could not be read: {detail}")
}

/// The container refused — [`crate::sign::IdentityFailure::Import`].
#[must_use]
pub fn identity_refused(detail: &str) -> String {
    format!("That certificate could not be opened: {detail}")
}

// ---------------------------------------------------------------------------
// What the operator authors
// ---------------------------------------------------------------------------

/// The section heading for the authored fields.
#[must_use]
pub const fn details_heading() -> &'static str {
    "What the signature will say"
}

/// The `/Reason` field's label.
#[must_use]
pub const fn reason_label() -> &'static str {
    "Reason"
}

/// The `/Reason` field's placeholder.
#[must_use]
pub const fn reason_hint() -> &'static str {
    "optional — for example, Approved for construction"
}

/// The `/Location` field's label.
#[must_use]
pub const fn location_label() -> &'static str {
    "Location"
}

/// The `/Location` field's placeholder.
#[must_use]
pub const fn location_hint() -> &'static str {
    "optional — wherever you say you signed it"
}

/// What happens to those two fields, said once under both.
#[must_use]
pub const fn authored_note() -> &'static str {
    "Both are written into the signature exactly as you type them, and are \
     shown by any reader that displays signature details. Leaving one empty \
     leaves it out altogether. pdfcer adds nothing of its own."
}

/// Where the signer's name comes from — the absence explained on screen.
#[must_use]
pub const fn name_comes_from_the_certificate() -> &'static str {
    "The signer's name is not typed here: it is read out of your certificate, \
     which is the only version of it anybody can check."
}

/// The signing time that will be written, shown before it is written.
#[must_use]
pub fn signing_time(stamp: &str) -> String {
    format!("Signing time, as it will be written: {stamp}")
}

/// The clock is unusable, so nothing can be signed.
#[must_use]
pub const fn clock_unusable() -> &'static str {
    "This machine's clock is set to a date before 1970, so pdfcer cannot write \
     a signing time — and a signature has to carry one. Fix the clock and try \
     again."
}

// ---------------------------------------------------------------------------
// Placement
// ---------------------------------------------------------------------------

/// The section heading for visibility.
#[must_use]
pub const fn placement_heading() -> &'static str {
    "On the page"
}

/// The invisible option.
#[must_use]
pub const fn placement_invisible() -> &'static str {
    "Draw nothing on the page (recommended)"
}

/// The visible option.
#[must_use]
pub const fn placement_visible() -> &'static str {
    "Draw a signature box on page"
}

/// **What the box will actually contain, said before it is chosen.**
#[must_use]
pub const fn placement_note() -> &'static str {
    "The box carries your name as your certificate spells it, the date, and \
     your reason and location if you give them — set in Helvetica and shrunk to \
     fit. Every reader also shows all of that in its own signature panel whether \
     the box is there or not, which is why drawing nothing is still the \
     recommended choice on a drawing."
}

// ---------------------------------------------------------------------------
// Signing into a box somebody else placed — `Pass 10.13`
// ---------------------------------------------------------------------------

/// The third placement option: sign into a pre-placed field.
#[must_use]
pub fn placement_existing(count: usize) -> String {
    if count == 1 {
        "Sign in the box already on this document (1 found)".to_owned()
    } else {
        format!("Sign in a box already on this document ({count} found)")
    }
}

/// Why the page and position controls went away when a box was picked.
#[must_use]
pub const fn placement_field_note() -> &'static str {
    "The page and the position come from the box itself — whoever prepared this \
     document chose them — so there is nothing here for you to place."
}

/// One field in the list: its name, and where it is.
#[must_use]
pub fn field_row(name: &str, page: Option<usize>) -> String {
    match page {
        Some(index) => format!("{name} — page {}", index + 1),
        None => format!("{name} — the document does not say which page"),
    }
}

/// A field whose own rectangle has no area.
#[must_use]
pub const fn field_invisible() -> &'static str {
    "This box has no size on the page: whoever prepared the document wanted a \
     signature that is recorded in the file but not drawn on the drawing. \
     Nothing will appear where it sits."
}

/// **The `/Lock` disclosure, and it is shown BEFORE the press.**
#[must_use]
pub fn field_locks(action: &str) -> String {
    let what = match action {
        "All" => "every other field in this document",
        "Include" => "the fields named in the document's own list",
        "Exclude" => "every field except the ones the document's own list names",
        // A `/Action` name outside Table 233's three. Echoed rather than
        // guessed at: the engine copies whatever is there into the transform,
        // so a sentence claiming to know which fields it means would be a
        // claim this shell cannot support.
        _ => "the fields the document nominates",
    };
    format!(
        "⚠ Signing here also locks {what} against further change. Whoever \
         prepared this document asked for that, and pdfcer honours it — it is \
         not something pdfcer adds and not something you can turn off here."
    )
}

/// The `/SV` disclosure — the author attached conditions to this box.
#[must_use]
pub const fn field_constrained() -> &'static str {
    "Whoever prepared this document attached conditions to this box — which \
     reason is acceptable, which kind of signature, and so on. pdfcer checks \
     every one of them and will say so by name if your signature does not meet \
     one."
}

/// A field this shell will not offer, and why.
#[must_use]
pub fn field_unusable(bar: crate::sign::FieldBar) -> String {
    match bar {
        crate::sign::FieldBar::HasKids => "pdfcer cannot sign in this box: it is \
             built as a group of several boxes sharing one name, and pdfcer signs \
             only a single one. Ask whoever prepared the document for a plain \
             signature box, or place your own with the option above."
            .to_owned(),
    }
}

/// Shown in place of the list when the document holds no empty box.
#[must_use]
pub const fn no_existing_fields() -> &'static str {
    "This document has no empty signature box in it. If you were told there \
     would be one, it may already have been signed, or the sender may have \
     drawn a rectangle rather than placing a signature field."
}

/// Where the box goes, with the measurements.
#[must_use]
pub const fn placement_where() -> &'static str {
    "It is placed 180 × 60 points, half an inch in from the bottom-right \
     corner of the page — beside where a title block usually sits."
}

/// The page-chooser's label.
#[must_use]
pub const fn page_label() -> &'static str {
    "Page"
}

// ---------------------------------------------------------------------------
// Approval or certification — `Pass 10.12`
// ---------------------------------------------------------------------------

/// The section heading for the kind of signature.
#[must_use]
pub const fn kind_heading() -> &'static str {
    "What kind of signature this is"
}

/// The default: an approval signature.
#[must_use]
pub const fn kind_approval() -> &'static str {
    "Approve this document — an ordinary signature"
}

/// The certifying option.
#[must_use]
pub const fn kind_certify() -> &'static str {
    "Sign as this document's author, and set what may be changed afterwards"
}

/// What certifying does, before it is chosen.
#[must_use]
pub const fn kind_certify_note() -> &'static str {
    "A reader calls this a certifying signature. It records that you are the \
     document's author and states, in the file, which later changes are allowed \
     without breaking your signature. There can only be one, and it has to be \
     the document's first signature."
}

/// The heading over the three `/DocMDP` levels.
#[must_use]
pub const fn mdp_heading() -> &'static str {
    "What anyone may change afterwards"
}

/// One `/DocMDP` level, as the operator reads it.
#[must_use]
pub fn mdp_level(permission: pdfcer_core::sign::apply::MdpPermission) -> &'static str {
    use pdfcer_core::sign::apply::MdpPermission as P;
    match permission {
        P::NoChanges => "Nothing at all — any change to the document breaks your signature",
        P::FormFillAndSign => {
            "Filling in form fields and adding further signatures — anything else breaks yours"
        }
        P::FormFillSignAnnotate => {
            "Filling in form fields, adding signatures, and adding or changing comments and \
             mark-up"
        }
    }
}

/// Why certifying is not on offer for this document.
#[must_use]
pub fn certify_unavailable(bar: crate::sign::CertifyBar) -> String {
    match bar {
        crate::sign::CertifyBar::AlreadyCertified => {
            "Somebody has already signed this document as its author, and a \
             document can only have one such signature. You can still add an \
             ordinary signature."
                .to_owned()
        }
        crate::sign::CertifyBar::NotFirst { existing } => {
            let count = if existing == 1 {
                "one signature".to_owned()
            } else {
                format!("{existing} signatures")
            };
            format!(
                "Signing as the author has to be the first signature on a \
                 document — it says what may be changed after it, and it cannot \
                 speak for changes made before it. This document already carries \
                 {count}. You can still add an ordinary signature."
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Confirming, and what happened
// ---------------------------------------------------------------------------

/// The confirm control, writing a new file.
#[must_use]
pub const fn confirm_button() -> &'static str {
    "Sign and save…"
}

/// The confirm control, replacing the open file. Names the file, always.
#[must_use]
pub fn confirm_button_replace(file_name: &str) -> String {
    format!("Sign and replace {file_name}")
}

/// Why the confirm control is disabled, on hover.
#[must_use]
pub const fn confirm_disabled_no_certificate() -> &'static str {
    "Choose a certificate file and open it first."
}

/// See [`confirm_disabled_no_certificate`].
#[must_use]
pub const fn confirm_disabled_overwrite() -> &'static str {
    "Tick the box to confirm you want to replace the file you have open."
}

/// The suffix [`crate::sign::suggested_path`] proposes.
#[must_use]
pub const fn suggested_suffix() -> &'static str {
    "-signed"
}

/// The outcome heading, after a successful write.
#[must_use]
pub const fn written_heading() -> &'static str {
    "Signed"
}

/// The outcome sentence.
#[must_use]
pub fn written(file_name: &str, replaced: bool) -> String {
    if replaced {
        format!("{file_name} has been signed, in place.")
    } else {
        format!("The signed document was written as {file_name}.")
    }
}

/// **The shape one line of a signature's appearance is shown in.**
const APPEARANCE_INDENT: &str = "\n    "; // string-gap-exempt: an indent, not a sentence.

/// **Everything one signing wrote**, as the sentence that discloses it needs
/// the facts.
pub struct Written<'a> {
    /// The signature field's `/T`.
    pub field: &'a str,
    /// The certificate's subject.
    pub subject: &'a str,
    /// Its serial, in hex — what a recipient quotes back.
    pub serial: &'a str,
    /// Whether the signature went into a box that was already on the document.
    pub reused: bool,
    /// The `/FieldMDP` written because the field carried a `/Lock`.
    pub lock: Option<&'a str>,
    /// The `/DocMDP` level's meaning, when this signing certified.
    pub certification: Option<&'a str>,
    /// Seed-value constraints the form author recommended and this signature
    /// does not meet.
    pub notes: &'a [String],
    /// The text a visible signature's box shows.
    pub appearance: &'a [String],
}

/// **What the report says pdfcer wrote — the rule-4 disclosure.**
#[must_use]
pub fn written_details(written: &Written<'_>) -> String {
    let &Written {
        field,
        subject,
        serial,
        reused,
        lock,
        certification,
        notes,
        appearance,
    } = written;
    let mut out = if reused {
        format!(
            "Signed in the box already on the document, {field} — by {subject}, \
             certificate serial {serial}."
        )
    } else {
        format!(
            "Signature field {field}, signed by {subject}, certificate serial \
             {serial}."
        )
    };
    if let Some(meaning) = certification {
        out.push_str(&format!(
            "\nThis is a certifying signature: you have signed as the document's \
             author, and what anyone may change afterwards without breaking your \
             signature is now — {meaning}."
        ));
    }
    if let Some(action) = lock {
        out.push_str(&format!(
            "\nThe box carried a lock, so signing it also froze the fields the \
             document nominated ({action}). That was the document author's \
             instruction, honoured."
        ));
    }
    if !appearance.is_empty() {
        out.push_str(
            "\nThe signature box on the page shows this text, which pdfcer \
             composed from your certificate, the time of signing, and the \
             reason and location you gave:",
        );
        for line in appearance {
            out.push_str(&format!("{APPEARANCE_INDENT}{line}"));
        }
    }
    for note in notes {
        out.push_str(&format!(
            "\nWhat the document asked for and this signature does not do: {note}"
        ));
    }
    out
}

/// **How many of `appearance`'s lines the composed sentence actually shows.**
#[must_use]
pub fn appearance_shown(details: &str, appearance: &[String]) -> usize {
    appearance
        .iter()
        .filter(|line| details.contains(&format!("{APPEARANCE_INDENT}{line}")))
        .count()
}

/// **What the open document is now, said rather than left to be
/// discovered.**
#[must_use]
pub const fn open_document_unchanged() -> &'static str {
    "The document you have open is not the signed one — it is the version you \
     started from. Open the signed file to see the signature, or carry on \
     editing this one and sign again when you are finished."
}

/// The button that opens what was just written.
#[must_use]
pub const fn open_the_signed_document() -> &'static str {
    "Open the signed document"
}

/// The engine refused after the form was filled in.
#[must_use]
pub fn engine_refused(detail: &str) -> String {
    format!("pdfcer did not sign the document: {detail}")
}

/// The reservation was too small — the one engine refusal whose own advice
/// this shell cannot follow.
#[must_use]
pub fn reservation_too_small(detail: &str) -> String {
    format!(
        "pdfcer did not sign the document: {detail} pdfcer reserves a fixed \
         amount of room, so this certificate's chain is too long for it to \
         sign with. Signing with a certificate that has a shorter chain will \
         work."
    )
}

/// **THE REFUSAL THE AUTHOR OF THE DOCUMENT IMPOSED — and the single most
/// important sentence added on 2026-09-06.**
#[must_use]
pub fn author_imposed(detail: &str) -> String {
    format!(
        "Whoever prepared this document set a condition on that signature box, \
         and this signature does not meet it — so nothing was written. What the \
         document asks for: {detail}\n\nThis is the document's own rule, not a \
         limit in pdfcer. pdfcer checks every condition a form author can set \
         and refuses rather than signing around one, which is stricter than some \
         other readers — so a document another program would sign can be refused \
         here. Sign in a different box if there is one, or ask whoever sent it to \
         you to relax the condition."
    )
}

/// The chosen box turned out not to be signable after all.
#[must_use]
pub fn field_refused(detail: &str) -> String {
    format!(
        "That signature box could not be used: {detail} Choose another box, or \
         place your own signature on the page instead."
    )
}

/// The composed appearance did not fit the box.
#[must_use]
pub fn appearance_overflow(detail: &str) -> String {
    format!(
        "Your name, the date and what you typed will not fit in the box at a \
         readable size, so nothing was written rather than a signature with its \
         text cut off: {detail} Shorten or clear the reason and location, or \
         choose \"do not draw anything on the page\" — the signature is recorded \
         either way and every reader shows it in its own panel."
    )
}

/// The file system refused the write.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("The signed document could not be written: {detail}")
}
