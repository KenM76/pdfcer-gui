//! # `text::sign` — every operator-facing string on the control that puts the
//! operator's own signature into a document
//!
//! The write side of a subject whose read side already has two modules:
//! [`crate::text::security`] reports what a document says about its
//! protection, [`crate::text::trust`] reports what pdfcer could and could not
//! check about a signature that already exists, and this one describes
//! something pdfcer is **about to do with a private key**.
//!
//! ## ★★★ THE STANDARD THIS MODULE IS HELD TO, AND IT IS NOT THE USUAL ONE
//!
//! [`crate::text::trust`]'s subject is a **verdict**, and its failure mode is
//! claiming more than the engine checked. This module's subject is an **act**,
//! and its failure mode is different and worse: a sentence here can persuade an
//! operator to attach their legal identity to a document. So two rules bind
//! every string below, and both are narrower than "be accurate".
//!
//! 1. **Nothing here calls a signature valid, trusted, secure or verified.**
//!    Not once, not as a summary, not in a tooltip. Authoring a signature and
//!    the signature being *trusted by a recipient* are different facts settled
//!    by different parties, and this surface only ever performs the first.
//!    [`crate::panels::signatures`] is the only place in pdfcer that reports
//!    the second, it reports three facts that never collapse into one, and a
//!    cheerful word here would undo that whole design before the panel is
//!    opened. `sign.svg`'s own note carries the same constraint for the glyph.
//! 2. **Every sentence about what will be written names what will be
//!    written.** `/Reason`, `/Location` and the signing time are the
//!    operator's words, copied verbatim into a legal artifact; the copy says
//!    so rather than describing them as "details".
//!
//! ## ★★ What is deliberately NOT offered, and it is a string's absence
//!
//! **There is no *Name* field**, and its absence is a decision rather than an
//! omission — recorded here because an absence cannot be read out of the code
//! that does not contain it.
//!
//! `SignRequest::name` writes `/Name`, and the engine's own note says `None`
//! *"omits the key and a verifier falls back to the certificate subject (Table
//! 252 says it should anyway)"*. A free-text name beside a certificate is a
//! **second, unverifiable claim about who signed**: nothing stops it saying
//! something the certificate does not, and a reader that trusts `/Name` over
//! the subject would show a name nobody vouched for. The certificate is the
//! name. So the window shows the subject it read out of the operator's own
//! `.pfx`, and offers no way to write a different one.
//!
//! ## ★ `/ContactInfo` is not offered either, for a smaller reason
//!
//! It is legitimate and harmless — a phone number for a verifier who wants to
//! reach the signer. It is left out because three free-text boxes on a form
//! whose two important controls are the certificate and the destination is
//! three boxes an operator scrolls past, and because nobody has asked for it.
//! Adding it is one field and one string; that is the right size for a request,
//! and the wrong size for a guess.

use crate::text::commands::CommandText;

// ===========================================================================
// THE RIBBON CONTROL
// ===========================================================================

/// `file.sign` — the label and the hover.
///
/// # The label is *Sign…* and not *Digitally sign…*
///
/// Because there is no other kind of signing in pdfcer, and a qualifier that
/// distinguishes nothing is a longer button. The tooltip carries the words a
/// person searching for the feature will have in mind — *certificate*,
/// *digital ID* — so the control is findable without the label carrying them.
///
/// # ★★ The tooltip names the two refusals in the same sentence
///
/// R9's *explained* branch. Whether **this** document is encrypted, or is
/// carrying a redaction the operator armed ten minutes ago, is not known when
/// the ribbon is built, so the control cannot be absent — and finding out by
/// pressing is the failure this project has paid for more than once. The hover
/// says what will happen before the press.
///
/// # ★ …and it says the signature goes in a new file by default
///
/// Because the alternative reading — that pressing this changes the document
/// on screen — is the reading every other verb on the Edit tab has taught, and
/// this one is the operator's legal artifact.
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
///
/// ★★ It states the **shape** of what is about to happen rather than the
/// limits of this build — [`crate::panels::signatures`]' header records why
/// that distinction matters: a sentence naming a limit was true when written
/// and false within hours, and the prose around it stayed true. A sentence
/// describing the mechanism cannot go stale the same way.
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
///
/// ★ Named first among the refusals and worded as one step rather than as a
/// wall, because it *is* one step: the operator armed the removal, and Edit ▸
/// Redact holds both the button that finishes it and the button that calls it
/// off.
#[must_use]
pub const fn refusal_redaction_pending() -> &'static str {
    "A redaction is armed on this document and has not been applied yet. \
     Signing now would sign the version that still contains what you marked \
     for removal, so pdfcer refuses it. Apply the redaction, or call it off, \
     and then sign."
}

/// [`crate::sign::Refusal::Encrypted`].
///
/// ★★ It names the engine's reason rather than stopping at "it is encrypted",
/// because the two suggest opposite next moves: an operator told only that the
/// document is protected will look for a permission to change, and the actual
/// remedy is to sign first and protect afterwards.
#[must_use]
pub const fn refusal_encrypted() -> &'static str {
    "This document is encrypted. A signature has to be added to the end of the \
     file, and pdfcer cannot append to an encrypted one — so an encrypted \
     document cannot be signed at all. Take the password off first (File > \
     Security > Encrypt…), sign, and put it back on afterwards."
}

/// [`crate::sign::Refusal::CertificationForbids`].
///
/// ★ The permission number is in the sentence because it is in the document
/// and an operator taking this to whoever certified the file needs to be able
/// to quote it.
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
///
/// ★ One pure function rather than a `match` at each of the two call sites —
/// the window, which draws it instead of a form, and
/// [`crate::app::actions::sign`], which reaches it when the document changed
/// between the window opening and the press. Two spellings of one mapping is
/// two chances for a refusal to be worded differently depending on when it was
/// noticed.
///
/// A new [`crate::sign::Refusal`] variant is a compile error here rather than a
/// silent fall-through to a catch-all.
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
///
/// ★ Not a refusal — a PDF may hold many — and shown anyway, because an
/// operator adding a second signature to a document they thought was unsigned
/// has learned something about the file they were handed.
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
///
/// ★★★ This is a **promise about behaviour**, and it is the one sentence in
/// this module that a reader is entitled to check the code against. It is true
/// because of `crate::secret::Secret` (a type whose value cannot be formatted)
/// and `crate::sign`'s §5 (no trace line carries the passphrase, its length, or
/// the certificate's path). If either of those changes, this sentence must be
/// the thing that changes with it.
#[must_use]
pub const fn passphrase_note() -> &'static str {
    "Your passphrase is used to open the certificate and is not saved, written \
     to any log, or kept after this window closes."
}

/// The button that opens the certificate so its contents can be shown.
///
/// ★★ A separate press rather than opening the file as soon as both boxes have
/// something in them. Two reasons, and the second decides it: a passphrase is
/// typed one character at a time, so an eager load would attempt — and fail —
/// on every keystroke, and some PKCS#12 containers use a key-derivation
/// function expensive enough for that to be felt. More importantly, **it puts
/// the identity on screen before the signing control is reachable at all**: the
/// operator sees whose certificate they are about to use, from the file itself,
/// rather than trusting that they picked the right one.
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
///
/// ★ Shown as *"stored as"* rather than as a name, because it is the label
/// whoever exported the file typed into their own certificate manager. It is
/// useful for recognising the right file and is not a claim about anything.
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

/// ★★★ **Whether the container's integrity was checked, and what it means when
/// it was not.**
///
/// A PKCS#12 file may carry no `macData` at all, in which case the passphrase
/// opened the key and nothing verified that the file is the one that was
/// exported. The engine reports `mac: None` for exactly that case, and this is
/// the one fact on this window an operator could act on that they would not
/// otherwise be told: a container whose integrity was never checked is one that
/// could have been altered between export and here.
///
/// It is stated in both directions rather than only in the bad one, because a
/// line that appears only when something is wrong is a line nobody learns to
/// look for.
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
///
/// ★ Disclosed rather than silently discarded. An operator whose file holds
/// four certificates and whose signature embeds two should be told which
/// happened, because the usual cause is a container exported with a whole
/// address book in it and the second usual cause is a chain that does not
/// actually chain.
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
///
/// ★★ The engine's own message is printed **verbatim** and is not re-worded.
/// `Pkcs12Error` distinguishes a wrong passphrase from a scheme pdfcer does not
/// implement, from a container with no private key, from a key algorithm it
/// cannot sign with — four different next moves — and every one of its variants
/// is already a sentence written to be read. Softening them into "the
/// certificate could not be opened" is how an operator comes to spend an
/// afternoon retyping a passphrase that was right.
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
///
/// ★ An example rather than an instruction, and a bland one on purpose: a
/// placeholder reading *"I approve this document"* is a suggestion, and a
/// suggested reason on a legal artifact is pdfcer putting words in somebody's
/// mouth. Leave-it-blank has to be an equally comfortable answer.
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
///
/// See this module's header for the full argument. It is on the window and not
/// only in the source, because an operator looking for a Name box needs to know
/// the box is missing on purpose.
#[must_use]
pub const fn name_comes_from_the_certificate() -> &'static str {
    "The signer's name is not typed here: it is read out of your certificate, \
     which is the only version of it anybody can check."
}

/// The signing time that will be written, shown before it is written.
///
/// ★★★ The engine reads no clock — its `SignRequest::signing_time` doc says a
/// GUI *"passes the time it showed the operator"* — so this string is not a
/// report of what was written, it is the **source** of it. The moment on screen
/// and the moment in the file are the same value.
#[must_use]
pub fn signing_time(stamp: &str) -> String {
    format!("Signing time, as it will be written: {stamp}")
}

/// The clock is unusable, so nothing can be signed.
///
/// The one failure `crate::app::clock::pdf_date_utc` can have. PAdES requires
/// `/M` and pdfcer will not invent one, so this is a refusal rather than a
/// signature with no time on it.
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

/// ★★★ **What the box will actually contain, said before it is chosen.**
///
/// The engine's `EditSession::sign` states it plainly: a visible signature's
/// appearance is, in this first cut, *"a thin frame only — no text; the details
/// live in the report and in any reader's signature panel."*
///
/// So the operator is told, in advance, that the box is empty. This is R8b Rule
/// 4 in its sharpest form — the box is **applied content**, it renders exactly
/// as the saved file will render, and there is nothing provisional about it. An
/// operator who found an empty rectangle on their drawing afterwards would read
/// it as a defect, and they would be right to.
///
/// ⚠⚠⚠ **THIS SENTENCE IS A DATED CITATION AND IT IS ALREADY OUT OF DATE
/// UPSTREAM.** `Cargo.lock` pins `pdfcer-core` at `f9bc7c8` (v0.41.0), where
/// the appearance is a frame. The engine's **unreleased** `Pass 10.14`
/// (`187fa09`, measured with `git merge-base --is-ancestor` to be **outside**
/// the pin) composes the signer's CN, the date, the reason and the location
/// into the box in Helvetica, shrink-to-fit 10 → 4 pt, and refuses a box too
/// small by name before anything is staged.
///
/// ⇒ **When the pin moves past `f9bc7c8`, re-read this string first.** It will
/// be false, and it is the kind of false that looks fine: an operator told the
/// box is empty who then finds their name in it has been under-promised rather
/// than lied to, so nothing will report it. [`placement_where`]'s measurements
/// and `crate::sign::default_rect`'s clamp both want re-examining with it,
/// because the engine's new `AppearanceOverflow` refusal is about exactly the
/// small box that clamp produces.
///
/// ★ Recorded in `ENGINE_BACKLOG.md` under **Signing hardening** as well as
/// here, because that file has a gate reading it and this doc comment does
/// not. `check-engine-backlog` is what surfaced `Pass 10.14` on the day it
/// shipped — which is the mechanism working, and is why the claim is filed
/// there rather than only argued here.
#[must_use]
pub const fn placement_note() -> &'static str {
    "The box is an empty frame: pdfcer does not yet draw your name or the date \
     inside it. Every reader shows those in its own signature panel whether the \
     box is there or not, which is why drawing nothing is the recommended \
     choice on a drawing."
}

/// Where the box goes, with the measurements.
///
/// ★ The numbers are in the sentence because the box is content in the
/// operator's file and *"near the bottom right"* is not something anybody can
/// check against the result.
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
///
/// ★ One function returning the FIRST outstanding thing rather than a list,
/// because a hover is read in one glance and because the conditions are met in
/// this order anyway. `crate::text::protect::confirm_disabled` is the same
/// shape for the same reason.
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

/// ★★★ **What the report says pdfcer wrote — the rule-4 disclosure.**
///
/// The engine's `SignReport` exists so a front end can state what it wrote
/// rather than assume it. This is that statement, and it names the field, whose
/// certificate was used and its serial — the two things a recipient will quote
/// back when they ask *"is this really you?"*
#[must_use]
pub fn written_details(field: &str, subject: &str, serial: &str) -> String {
    format!(
        "Signature field {field}, signed by {subject}, certificate serial \
         {serial}."
    )
}

/// ★★ **What the open document is now, said rather than left to be
/// discovered.**
///
/// `crate::sign`'s §3: the session still holds the placeholder, not the
/// signature, and the engine's own instruction to a GUI is to reload. An
/// operator who pressed `Ctrl+S` after this without being told would append a
/// second revision onto a base that is no longer the file on disk.
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
///
/// ★ The engine's message verbatim, for [`identity_refused`]'s reason:
/// `SignApplyError` has a distinct, already-written sentence per variant, and
/// two of them (the encrypted document, the pending redaction) are the ones
/// this window is supposed to have caught earlier. Reaching one of those here
/// means the document changed between the window opening and the press — which
/// is a real thing that can happen and is worth reading in the engine's own
/// words rather than in a paraphrase.
#[must_use]
pub fn engine_refused(detail: &str) -> String {
    format!("pdfcer did not sign the document: {detail}")
}

/// The reservation was too small — the one engine refusal whose own advice
/// this shell cannot follow.
///
/// ★★ `SignApplyError::ReservationTooSmall`'s message ends *"sign again with a
/// larger reserve"*, and there is no control here that sets one, deliberately
/// (`crate::sign::prepare`'s note argues why asking would be handing the
/// operator arithmetic). So the engine's sentence is shown **and then
/// corrected**, rather than shown alone as an instruction that leads nowhere.
#[must_use]
pub fn reservation_too_small(detail: &str) -> String {
    format!(
        "pdfcer did not sign the document: {detail} pdfcer reserves a fixed \
         amount of room, so this certificate's chain is too long for it to \
         sign with. Signing with a certificate that has a shorter chain will \
         work."
    )
}

/// The file system refused the write.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("The signed document could not be written: {detail}")
}
