//! # `text::security` — every operator-facing string about encryption,
//! passwords and signatures
//!
//! `OPERATOR_REQUESTS.md` O108. One module for the whole subject, because the
//! subject's copy has one property the rest of this crate's does not: **it makes
//! claims about what protects a document, and a wrong one is worse than
//! silence.** An operator who believes a file is protected when it is not has
//! been actively misled by this program; an operator who is told nothing has
//! merely not been helped.
//!
//! ## ★★★ Two sentences here came from `pdfcer-core` and must not be re-worded
//!
//! [`permissions_are_advisory`] and [`signature_not_verified`] are the engine's
//! own wording, supplied in its reply on 2026-09-03 when this shell asked for
//! them by name. That was deliberate on both sides: the CLI prints the first
//! one too, and **two surfaces wording one limitation differently is worse than
//! either wording** — an operator who reads "permissions are advisory" in one
//! place and something softer in another has to work out which program is
//! lying.
//!
//! ★★★ **Amended 2026-09-05: only the FIRST of those two is still the engine's
//! wording.** [`signature_not_verified`] described a limitation that ended when
//! `pdfcer-core` v0.38.0 (`b01964f`) shipped `verify_all_with_trust`, so it has
//! been rewritten here rather than left standing — see its own doc comment. The
//! "do not re-word it" rule protected a sentence that had **expired**, which is
//! the failure mode a shared-wording rule is most exposed to: it makes the
//! sentence harder to change at exactly the moment it needs changing. The rule
//! stands for [`permissions_are_advisory`], which describes a property of the
//! PDF standard and cannot expire.
//!
//! The engine also corrected our draft of the second, and the correction is
//! instructive: our version said pdfcer *"cannot tell you the document is
//! unaltered"*, and the engine asked for wording that **will not have to be
//! unwritten** when its integrity check ships. So the sentence says *"does not
//! yet check"* and separates the clause that will change from the clause about
//! trust, which will not.
//!
//! ⇒ When `signature::verify` lands, the first two clauses change and the trust
//! clause stays. The engine will send the replacement wording with the verb.
//! **Do not guess at it in the meantime.**

use pdfcer_core::crypto::{AuthKind, Cipher, PermissionBit};

/// The Security tab's caption.
#[must_use]
pub const fn tab_security() -> &'static str {
    "Security"
}

// ---------------------------------------------------------------------------
// The password prompt
// ---------------------------------------------------------------------------

/// The password window's title.
#[must_use]
pub const fn password_title() -> &'static str {
    "This document needs a password"
}

/// The sentence above the box.
///
/// ★ It names the FILE, because an operator who has opened four drawings and
/// walked away needs to know which one is asking. The prompt appears in answer
/// to their own Open, but not always in the same minute as it.
#[must_use]
pub fn password_prompt(file_name: &str) -> String {
    format!("{file_name} is encrypted. Enter the password to open it.")
}

/// The label beside the field.
#[must_use]
pub const fn password_label() -> &'static str {
    "Password"
}

/// The button that tries it.
#[must_use]
pub const fn password_open() -> &'static str {
    "Open"
}

/// The button that gives up.
///
/// ★ *Cancel*, not *Close*: it abandons an attempt the operator started, and
/// the tab stays in the document list showing why it did not open. Nothing is
/// lost by pressing it.
#[must_use]
pub const fn password_cancel() -> &'static str {
    "Cancel"
}

/// What an operator is told after a password that did not work.
///
/// ★★ It says **which attempt** this was, and that is not decoration. Without
/// it, a second wrong password produces a dialog identical to the first, and an
/// operator who did not see the field clear cannot tell whether their press
/// registered at all — the same ambiguity the measure tools' running count
/// exists to remove.
///
/// It does **not** say how many attempts remain, because there is no limit:
/// pdfcer is reading a local file and rate-limiting the operator's own guesses
/// at their own document would be theatre.
#[must_use]
pub fn password_rejected(attempt: u32) -> String {
    format!(
        "That password did not open the document (attempt {attempt}). Try again — either the user password or the owner password will do."
    )
}

/// The **different** failure: pdfcer cannot normalise a non-ASCII password.
///
/// # ★★★ Why this is not "wrong password", and why the engine made it a
/// separate error
///
/// `DocError::PasswordRequiresNormalisation` exists, in `pdfcer-core`'s own
/// words, *"so that failure does not masquerade as `PasswordRequired`'s 'you
/// typed it wrong', which would send the operator to re-check a password that
/// was correct."*
///
/// The mechanism: `/R` 5 specifies **SASLprep** (RFC 4013) over the password
/// before hashing, and pdfcer does not implement it — no stringprep dependency
/// was taken for a read-only increment. For an all-ASCII password SASLprep is
/// the identity, so this can only ever arise from a password containing
/// something else.
///
/// ⇒ So the sentence tells the operator the true thing: **the password may be
/// perfectly correct**, and pdfcer cannot prove it either way. Sending them to
/// re-type it would be this program wasting their afternoon on its own
/// limitation.
#[must_use]
pub const fn password_needs_normalisation() -> &'static str {
    "That password contains characters outside plain ASCII, and pdfcer cannot process those the way this document's encryption requires — so it cannot open the file even if the password is correct. This is a limit in pdfcer, not a wrong password. A document whose password is plain ASCII will open normally."
}

/// The refusal for an empty box.
///
/// ★ Refused here rather than sent on, and the reason is in
/// [`crate::secret::Secret::is_empty`]: pdfcer has *already* tried the empty
/// password before it prompted — every conforming reader does — so sending it
/// again would ask the engine a question it has answered and return an
/// identical rejection, which reads as "my password was wrong" about a password
/// that was never supplied.
#[must_use]
pub const fn password_empty() -> &'static str {
    "Type the password first. pdfcer already tried opening this document without one."
}

// ---------------------------------------------------------------------------
// What the document's encryption IS
// ---------------------------------------------------------------------------

/// The heading over the encryption facts.
#[must_use]
pub const fn encryption_heading() -> &'static str {
    "Encryption"
}

/// What is said when the document is not encrypted at all.
///
/// ★ Stated rather than left blank. *"This document is not encrypted"* is a
/// fact an operator checking a file wants confirmed; an empty panel is
/// indistinguishable from a panel that failed to load.
#[must_use]
pub const fn not_encrypted() -> &'static str {
    "This document is not encrypted. Anyone who has the file can open it."
}

/// The cipher and key length, in the operator's terms.
///
/// ★ The revision number (`/R`) is deliberately absent. It is the number that
/// matters to an implementer and means nothing to the person holding the
/// drawing; what they can act on is *how strong is this* and *is it modern*.
#[must_use]
pub fn cipher_line(cipher: Cipher) -> String {
    let described = match cipher {
        Cipher::Rc4 => "RC4, an old cipher that is no longer considered secure",
        Cipher::Aes128 => "AES-128",
        Cipher::Aes256 => "AES-256",
        // ★ `/None` means the security handler decrypts privately and pdfcer
        // cannot know how. A document routing real content through it is
        // refused before it reaches this shell, so what reaches here is the
        // Identity-like passthrough — encrypted in structure, not in content.
        // Worth saying rather than printing "none", which reads as an error.
        Cipher::None => {
            "a handler pdfcer does not implement — the document is marked encrypted and its content is not protected in any way pdfcer can see"
        }
    };
    format!("Encrypted with {described}.")
}

/// **Which password opened it**, which decides what the operator may do next.
///
/// # ★★ The engine asked for this by name
///
/// From its 2026-09-03 reply: *"`AuthKind` tells you which one opened the file
/// — surface that, because `remove_encryption` will refuse a
/// user-authenticated session and the operator should see WHY before pressing
/// it."*
///
/// So this is not a curiosity. It is the precondition of a control that does
/// not exist yet, shown before that control arrives, so the day it does the
/// refusal is already explained.
///
/// ★ [`AuthKind::EmptyUser`] is the case an operator never sees happen: the
/// document declares a user password of nothing, every conforming reader tries
/// it silently, and the file opens with no prompt. It is worth naming, because
/// *"this file is encrypted"* and *"you needed a password"* are then two
/// different facts and the operator has only observed one of them.
#[must_use]
pub const fn auth_line(auth: AuthKind) -> &'static str {
    match auth {
        AuthKind::EmptyUser => {
            "It opened without asking you, because the document's user password is empty — the encryption is there, but nothing is keeping anyone out."
        }
        AuthKind::User => {
            "You opened it with the user password, which grants the permissions listed below."
        }
        AuthKind::Owner => {
            "You opened it with the owner password, so the permissions below do not restrict you."
        }
    }
}

// ---------------------------------------------------------------------------
// Permissions
// ---------------------------------------------------------------------------

/// The heading over the permission bits.
#[must_use]
pub const fn permissions_heading() -> &'static str {
    "What this document allows"
}

/// ★★★ **The engine's own sentence, verbatim.** See the module header.
///
/// Supplied by `pdfcer-core` on 2026-09-03 with the instruction *"take this one
/// verbatim; it is the sentence the CLI will print too."* Do not re-word it,
/// do not shorten it for a narrow column, and do not soften *"a request, not a
/// lock"* — that clause is the whole of what an operator needs to know and it
/// is the one a marketing instinct would file off.
#[must_use]
pub const fn permissions_are_advisory() -> &'static str {
    "PDF permissions are a request, not a lock. A conforming reader honours them; any program that ignores the flag can print, copy or change this document freely. Only the password protects the content — and only the user password, which controls opening it."
}

/// One permission bit, named the way an operator would name it.
#[must_use]
pub const fn permission_name(bit: PermissionBit) -> &'static str {
    match bit {
        PermissionBit::Print => "Print",
        PermissionBit::ModifyContents => "Change the content",
        PermissionBit::Copy => "Copy text and graphics",
        PermissionBit::Annotate => "Add comments and fill in fields",
        PermissionBit::FillForms => "Fill in form fields",
        PermissionBit::AccessibilityExtract => "Extract for accessibility",
        PermissionBit::Assemble => "Insert, delete and rotate pages",
        PermissionBit::PrintHighQuality => "Print at full quality",
    }
}

/// Whether a bit is granted, refused, or not applicable at this revision.
///
/// ★ Three states, not two. `Permissions::granted` returns `Option<bool>`, and
/// `None` means *the bit does not apply to this document's encryption
/// revision* — which is not "refused". Rendering it as refused would tell the
/// operator their document forbids something it has no opinion about.
#[must_use]
pub const fn permission_state(granted: Option<bool>) -> &'static str {
    match granted {
        Some(true) => "allowed",
        Some(false) => "not allowed",
        None => "not stated at this encryption level",
    }
}

/// The `/Perms` integrity disagreement — the one signal PDF gives that a
/// document's stated permissions are not the ones its encryptor recorded.
///
/// # ★★ Reported, never acted on, and the engine is emphatic about why
///
/// `/Perms` holds an **encrypted** copy of `/P`. The plaintext copy sits in the
/// `/Encrypt` dictionary where anyone can edit it, with no integrity protection
/// anywhere else in clause 7.6. So a disagreement means somebody changed the
/// stated permissions after encryption — and **no clause says what to do about
/// it**.
///
/// `pdfcer-core` reports it and prefers neither value, keeping `/P` because that
/// is what the file declares and what every other viewer shows. Its own doc
/// comment names the rule: *"Silently substituting the decrypted copy would be
/// pdfcer deciding, on an inference, what the operator is told — the exact shape
/// project rule 4"*.
///
/// ⇒ So this sentence states the disagreement and stops. It does not say the
/// document was tampered with, because pdfcer does not know that.
#[must_use]
pub const fn perms_disagree() -> &'static str {
    "⚠  The permissions written in this document's encryption dictionary do not match the encrypted copy stored alongside them. That means the stated permissions were changed after the document was encrypted. pdfcer shows what the document declares — which is what every other viewer shows — and cannot tell you which of the two was intended."
}

/// The ordinary case for an older document: there is no `/Perms` entry to check.
///
/// ★ Said, and said as ordinary. `pdfcer-core`: *"`NotApplicable` for every
/// `/R` ≤ 4 document, where the entry does not exist. That is the ordinary
/// answer, not a failed check, and a front end must not render it as one."*
#[must_use]
pub const fn perms_not_applicable() -> &'static str {
    "This document's encryption predates the permissions integrity check, so there is nothing to compare against. That is normal for an older file, not a problem."
}

// ---------------------------------------------------------------------------
// Signatures
// ---------------------------------------------------------------------------

/// The heading over the signature facts.
#[must_use]
pub const fn signatures_heading() -> &'static str {
    "Signatures"
}

/// What is said when the document carries none.
#[must_use]
pub const fn not_signed() -> &'static str {
    "This document is not signed."
}

/// ★★★ **The engine's own sentence, reworded by the engine.** See the module
/// header.
///
/// Our draft said pdfcer *"cannot tell you the document is unaltered"*. The
/// engine asked for wording that **will not have to be unwritten** when its
/// integrity check ships — because *unaltered* is exactly what that check will
/// answer. So: *"does not yet check"*, and the clause that will change is
/// separated from the clause about trust, which will not.
///
/// ★★★ **THAT DAY CAME — 2026-09-05 — and this sentence was false for it.**
/// The paragraph above said *"when `signature::verify` lands, the first two
/// clauses change"*. It landed: `pdfcer-core` v0.38.0 (`b01964f`) carries
/// `signature::verify_all_with_trust`, `crate::trust::examine` calls it, and
/// `crate::panels::signatures` draws integrity, coverage and trust as three
/// separate labelled lines. The clause *"It does not yet check the signature
/// itself"* was untrue from that moment.
///
/// ⚠ **And nothing went red, because this function has ZERO call sites.** The
/// panel was built against [`crate::text::trust`] and [`crate::text::panels`]
/// instead, which orphaned this whole signature group — `signatures_heading`,
/// `not_signed`, `signature_count` and `coverage_line` are likewise
/// unreferenced. A dead string cannot mislead an operator, but it can and did
/// mislead a reader auditing what this build claims, which is what
/// `FEATURES.md` is re-measured against.
///
/// The wording below is **not guessed**: it is scoped down to the two facts
/// this catalog's own surviving callers deal in, and everything about
/// verdicts is deferred to [`crate::text::trust`], which is the catalog the
/// engine's reply was actually spent on. ⇒ *An absence claim is a claim about
/// every route; when the absence ends, grep for the sentence, not just for
/// the caller.*
#[must_use]
pub const fn signature_not_verified() -> &'static str {
    "pdfcer can see that this document is signed and can tell you whether anything was appended after the signature. What each signature covers, whether its bytes were altered, and whether its signer is one you trust are reported together in the Signatures panel."
}

/// How many signatures, and how many of those cover the whole file.
#[must_use]
pub fn signature_count(total: usize) -> String {
    format!("This document carries {total} signature(s).")
}

/// The coverage verdict — the one signature fact pdfcer *can* state today.
///
/// ★★ `covers_to_eof` is a real answer and it is the useful half of the two:
/// content appended after a signature is the ordinary way a signed document
/// stops meaning what it said, and it needs no cryptography to detect.
#[must_use]
pub const fn coverage_line(covers: bool) -> &'static str {
    if covers {
        "The signed byte range reaches the end of the file, so nothing has been appended since it was signed."
    } else {
        "⚠  The signed byte range stops short of the end of the file — content was added after this document was signed. Whatever is in that added part is NOT covered by the signature."
    }
}

// ---------------------------------------------------------------------------
// What pdfcer cannot do here
// ---------------------------------------------------------------------------

/// ★★★ The tab says what it cannot do, and it says it once, at the bottom.
///
/// # Why this exists rather than eight greyed buttons
///
/// R9: an unavailable capability renders **nothing**, and greying is reserved
/// for *temporarily* unavailable. A row of dead *Encrypt*, *Set permissions*,
/// *Remove encryption* and *Sign* controls would be eight promises this build
/// cannot keep, and an operator would spend their time discovering that one at
/// a time.
///
/// ★★ But *nothing at all* is the other failure. An operator opening a tab
/// called Security and finding only readouts will reasonably conclude the
/// feature is half-built and stop looking — which is the discoverability defect
/// that produced the Tool panel, arriving from the opposite direction. So the
/// tab states the boundary once, in one sentence, as a fact about this build.
///
/// ★★★ **CORRECTED 2026-09-05 — two of its three clauses were false, and it
/// too has ZERO call sites.** *"Encryption first, signing later"* was the
/// right prediction and it came true on 2026-09-04: `file.encrypt` and
/// `file.permissions` are registered, dispatched through
/// `crate::app::dispatch::security`, drawn on **File ▸ Security**, and
/// `crate::protect` calls `set_encryption`, `set_permissions` and
/// `remove_encryption`. So pdfcer can add a password, remove one, and change
/// these permissions. Only *sign a document* is still true.
///
/// The tab this was written for was superseded by that window and by
/// [`crate::text::protect`], which is where the live wording lives — so this
/// sentence sat false and unreferenced for a day. It is scoped to the one
/// clause that survives, and it now names where the rest went.
///
/// ★★★ **CORRECTED AGAIN 2026-09-06, and the last surviving clause is gone
/// too.** It read *"It cannot sign a document; that is still being built in the
/// engine."* Both halves were false by then: `pdfcer_core::sign` shipped on
/// 2026-09-05 — 101 public items, written in answer to this shell's own
/// request — and `file.sign` is now registered, dispatched and drawn on the
/// same File > Security band as its two neighbours.
///
/// ⇒ **This is the THIRD correction to one sentence, and it has had ZERO call
/// sites throughout.** That is the finding worth keeping: a string nothing
/// draws cannot be caught by looking at the screen, cannot be caught by a
/// driven check, and is corrected only when somebody happens to grep past it.
/// It has now been wrong about encryption, wrong about permissions, and wrong
/// about signing, in that order, each time by outliving a capability's arrival.
///
/// ⚠ The function is kept rather than deleted for the reason its own header
/// gives — the tab it belongs to states a boundary, and a boundary that is
/// merely absent reads as a half-built feature — but what it now states is the
/// **shape** of the boundary rather than a list of missing verbs, because a
/// list of missing verbs is a dated citation and this one has now expired three
/// times. `crate::panels::signatures` is named because that is the honest limit:
/// pdfcer authors a signature and reports what it can check about one, and
/// whether a recipient trusts it is not pdfcer's to say.
#[must_use]
pub const fn cannot_author() -> &'static str {
    "pdfcer can add or remove a password, change these permissions, and sign a document — File > Security. Whether anyone else trusts a signature is a separate question, and the Signatures panel is where pdfcer reports what it could and could not check."
}
