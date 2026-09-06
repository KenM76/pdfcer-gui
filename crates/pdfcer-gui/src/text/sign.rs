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
/// R8b Rule 4 in its sharpest form: the box is **applied content**, it renders
/// exactly as the saved file will render, and there is nothing provisional about
/// it. An operator who found something on their drawing afterwards that nobody
/// had described would read it as a defect, and they would be right to.
///
/// # ★★★ CORRECTED 2026-09-06, AND THE CORRECTION IS THE INTERESTING PART
///
/// **What this string said until the pin moved:** *"The box is an empty frame:
/// pdfcer does not yet draw your name or the date inside it."* That was true of
/// `pdfcer-core` at `f9bc7c8` (v0.41.0), where a visible signature's appearance
/// was, in the engine's own words, *"a thin frame only — no text"*.
///
/// **It is false at `d6b998f` (v0.42.0), the revision `Cargo.lock` now pins.**
/// `Pass 10.14` (`187fa09`) composes the signer's CN, the date, and the reason
/// and location when given, in Helvetica, shrunk to fit from 10 pt to 4 pt, and
/// refuses a rectangle too small for them by name
/// (`SignApplyError::AppearanceOverflow`) **before anything is staged** rather
/// than clipping — because a signature box whose text is silently cut is a
/// signature box that misstates who signed.
///
/// ⇒ ★★ **The falsehood was an UNDER-promise, and that is why it needed an
/// alarm rather than a test.** An operator told the box would be empty, who then
/// finds his own name in it, has been pleasantly surprised; he files nothing.
/// No screen, no unit test and no gate could have gone red. What caught it was
/// that the old string's doc comment carried the engine commit, the measurement
/// (`git merge-base --is-ancestor`, not a changelog) and the instruction *"when
/// the pin moves past `f9bc7c8`, re-read this string first"*. **A claim about
/// the engine is a dated citation; write its expiry beside it.**
///
/// ★ The old wording is quoted above in full rather than deleted, so a future
/// improvement cannot reinstate it out of git history believing it to be a
/// simplification.
///
/// # What did NOT change, and why the recommendation survives
///
/// The default is still *draw nothing*, and the argument for it is now a
/// different argument rather than a weakened one. It used to be *"the box would
/// be empty and read as a defect"*. It is now: **a reader shows the signature in
/// its own panel whether the box is there or not**, so the box adds no
/// information and does add content the operator did not draw to a sheet he is
/// about to send out. An operator who wants the stamp gets a stamp with his name
/// in it, and the sentence below now tells him that truthfully.
///
/// ⚠ [`placement_where`]'s 180 × 60 and `crate::sign::default_rect`'s clamp were
/// re-examined with this: the clamp shrinks the box only on a page smaller than
/// 252 × 132 pt, and `AppearanceOverflow` is the engine's refusal on exactly
/// that case — by name, before staging, so a small page produces a sentence
/// rather than a clipped signature. Nothing to change; recorded because the
/// question was asked.
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
///
/// ★★★ **Worded from the operator's situation, not from the format.** He does
/// not think *"there is an empty `/FT /Sig` field in the AcroForm"*; he thinks
/// *"they sent it back with a box on it for me to sign in"*. The label names the
/// situation, and `count` is in it because the number is the one thing that
/// tells him whether the box he was told about was found.
#[must_use]
pub fn placement_existing(count: usize) -> String {
    if count == 1 {
        "Sign in the box already on this document (1 found)".to_owned()
    } else {
        format!("Sign in a box already on this document ({count} found)")
    }
}

/// Why the page and position controls went away when a box was picked.
///
/// ★★★ **R9's *absent* branch needs a sentence, and this is it.** The engine
/// refuses `--visible`/`--page` alongside a field name by name — *"the existing
/// field already has a rectangle"* — so this shell makes the combination
/// unrepresentable and the controls simply go. A control that vanishes without
/// explanation is indistinguishable from one that broke; a greyed control with
/// no hover is the thing O77's sweep found seven of.
#[must_use]
pub const fn placement_field_note() -> &'static str {
    "The page and the position come from the box itself — whoever prepared this \
     document chose them — so there is nothing here for you to place."
}

/// One field in the list: its name, and where it is.
///
/// ★★ The page number is 1-based and is omitted rather than guessed when the
/// document does not say. A widget's `/P` is optional in the standard, so
/// *"page 1"* on a field that names no page would be this shell inventing a
/// fact about the operator's document.
#[must_use]
pub fn field_row(name: &str, page: Option<usize>) -> String {
    match page {
        Some(index) => format!("{name} — page {}", index + 1),
        None => format!("{name} — the document does not say which page"),
    }
}

/// A field whose own rectangle has no area.
///
/// ★ Said because the operator would otherwise sign, look at the drawing, see
/// nothing, and conclude it had failed. The author chose this; §12.7.4.5 makes a
/// zero-area rectangle the standard way to place an invisible signature.
#[must_use]
pub const fn field_invisible() -> &'static str {
    "This box has no size on the page: whoever prepared the document wanted a \
     signature that is recorded in the file but not drawn on the drawing. \
     Nothing will appear where it sits."
}

/// ★★★ **The `/Lock` disclosure, and it is shown BEFORE the press.**
///
/// Table 233. Signing a field that carries a `/Lock` makes the engine write a
/// `/FieldMDP` reference copying the lock's Action and Fields (§12.8.2.4) —
/// which genuinely freezes other fields in the document. That is a consequence
/// the operator has to consent to, and consent given after the file is written
/// is not consent.
///
/// ★★ The sentence says **who decided**. The freeze is not pdfcer being
/// cautious; it is an instruction the person who prepared the document wrote
/// into it, and an operator who reads it as pdfcer's own behaviour will go
/// looking for a setting to turn off.
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
///
/// ★★★ **Stated as a possibility, not a verdict, and that is deliberate.** This
/// shell reads only whether `/SV` is present; the engine evaluates it in full at
/// signing time. Saying *"this will be refused"* would be a second, worse answer
/// to a question with one authoritative answer, and saying nothing would let the
/// refusal arrive as a surprise. So: a warning that a refusal is possible and
/// whose it would be.
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
///
/// ★ The option is drawn and disabled with this beneath it rather than hidden,
/// which is the opposite of this window's usual rule and is right here for one
/// reason: the operator was **told by the sender** that there is a box. *"The
/// option is missing"* and *"the box the sender promised is not in this file"*
/// are the same picture and completely different facts, and only the second is
/// true.
#[must_use]
pub const fn no_existing_fields() -> &'static str {
    "This document has no empty signature box in it. If you were told there \
     would be one, it may already have been signed, or the sender may have \
     drawn a rectangle rather than placing a signature field."
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
// Approval or certification — `Pass 10.12`
// ---------------------------------------------------------------------------

/// The section heading for the kind of signature.
///
/// ★ A phrase rather than a caption, on this window's standing rule: `.strong()`
/// resolves to the accent-filled widget colour and draws pale text on a pale
/// panel (`DEFECTS.md` D11), so the hierarchy is carried by wording and layout.
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
///
/// ★★ The label avoids the word *certify* as its only cue and says what the act
/// means — signing **as the author** — because "certify" reads to most people as
/// a stronger synonym for "sign" rather than as the specific `/DocMDP` act it
/// is. The word is kept in the sentence beneath so the operator can match it to
/// what a reader will show him.
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
///
/// ★★★ **The engine's `MdpPermission` is the input, and the plain wording is
/// this shell's.** `MdpPermission::meaning` renders Table 254's own words — *"no
/// changes"*, *"form fill-in and signing"* — which are exact and are a
/// standard's phrasing, not a person's. What an operator needs is what happens
/// to *his* signature, so each line says that; the standard's own word is not
/// repeated, because two renderings of one fact on one screen is how a surface
/// starts disagreeing with itself.
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
///
/// ★★ R9's *explained* branch applied to an option rather than a window: both of
/// the engine's certification refusals are states of the document, knowable when
/// the window opens, so the option is **absent with this sentence** rather than
/// offered and then refused. The document can still be signed, and the sentence
/// says so — otherwise an operator reading a refusal on this window will read it
/// as a refusal of the window.
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
///
/// # ★★ What `Pass 10.12`–`10.14` added, and why each is on this screen
///
/// * **`reused`** — whether the signature went into a box that was already
///   there. Two outcomes that produce identical byte counts and can produce
///   identical field names: *"it signed the sender's box"* and *"it made a new
///   box beside it"*. Only the report can tell them apart, so it is said.
/// * **`lock`** — `SignReport::field_lock`, the `/FieldMDP` that was written
///   because the field carried a `/Lock`. Disclosed here **as well as** before
///   the press, because this is the sentence that says it *happened* rather
///   than that it *would*.
/// * **`certification`** — the `/DocMDP` level, with Table 254's own meaning
///   beside the number, so the operator can read what he just permitted.
/// * **`notes`** — `SignReport::notes`: seed-value constraints the form author
///   RECOMMENDED and this signature does not meet. ★★★ These are the ones that
///   did **not** refuse. Silence about them would be exactly the *"quiet
///   divergence"* the engine's own strictness exists to prevent, arriving one
///   layer up.
#[must_use]
pub fn written_details(
    field: &str,
    subject: &str,
    serial: &str,
    reused: bool,
    lock: Option<&str>,
    certification: Option<&str>,
    notes: &[String],
) -> String {
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
    for note in notes {
        out.push_str(&format!(
            "\nWhat the document asked for and this signature does not do: {note}"
        ));
    }
    out
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

/// ★★★ **THE REFUSAL THE AUTHOR OF THE DOCUMENT IMPOSED — and the single most
/// important sentence added on 2026-09-06.**
///
/// # The problem this string exists to solve
///
/// `Pass 10.13` enforces a signature field's `/SV` seed-value dictionary
/// (Table 234) **in full**, and the engine is deliberately **stricter than
/// Acrobat**. Three consequences follow, and the third is the dangerous one:
///
/// 1. A REQUIRED constraint the request does not meet is refused by name with
///    the satisfying values (`SeedValueViolated`).
/// 2. A constraint pdfcer cannot evaluate — `/Cert`, a required timestamp, a
///    legal attestation, revocation info, an unknown key — is **refused rather
///    than skipped** (`SeedValueUnevaluable`), because a condition the form
///    author wrote and the signer quietly ignored is worse than a refusal.
/// 3. ⇒ **So the operator will meet refusals on documents Acrobat would sign.**
///
/// [`engine_refused`]'s wording — *"pdfcer did not sign the document: …"* — is
/// correct for every other refusal on this surface and is **wrong for these**.
/// It puts pdfcer in the subject position of a sentence about somebody else's
/// rule, and an operator who reads *"pdfcer did not sign it"* beside a document
/// Acrobat signs has been told, in plain English, that pdfcer is broken. He
/// would be right to conclude that from the sentence, and wrong about the
/// program, and the feature would be reported as a defect.
///
/// # What this sentence does instead
///
/// **It names the author first, states the engine's own message second, and
/// gives two remedies that do not involve pdfcer changing.** And it says the
/// strictness is a **choice**, in one clause, because an operator comparing two
/// programs deserves to know which one is doing something unusual and why —
/// hiding it would leave him to discover the difference on his own and draw the
/// worse conclusion.
///
/// ★ The engine's message is quoted verbatim rather than paraphrased. It names
/// the constraint and the values that would satisfy it — *"requires SubFilter
/// one of: ETSI.CAdES.detached, adbe.pkcs7.detached"* — which is precisely what
/// the operator has to forward to whoever prepared the document. A paraphrase
/// would drop the values, which are the actionable half.
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
///
/// ★★ Reachable even though the window filters the list, and that is the point
/// of having it: the list is read once when the window opens, and the document
/// could have been signed by something else in between. Worded as a fact about
/// the box rather than as an error, with the engine's own sentence carrying the
/// detail.
#[must_use]
pub fn field_refused(detail: &str) -> String {
    format!(
        "That signature box could not be used: {detail} Choose another box, or \
         place your own signature on the page instead."
    )
}

/// The composed appearance did not fit the box.
///
/// ★ `SignApplyError::AppearanceOverflow`, new in `Pass 10.14`. The engine
/// refuses **before staging** rather than clipping, because a signature box
/// whose text is cut is a signature box that misstates who signed. Its message
/// carries the line count and the rectangle; this adds the remedy that is
/// actually available here, which is not the engine's `--visible` advice.
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
