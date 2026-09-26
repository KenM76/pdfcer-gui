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
//! ## Two sentences here came from `pdfcer-core` and must not be re-worded
//!
//!
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/security.md`.

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
#[must_use]
pub const fn password_cancel() -> &'static str {
    "Cancel"
}

/// What an operator is told after a password that did not work.
#[must_use]
pub fn password_rejected(attempt: u32) -> String {
    format!(
        "That password did not open the document (attempt {attempt}). Try again — either the user password or the owner password will do."
    )
}

/// The **different** failure: pdfcer cannot normalise a non-ASCII password.
#[must_use]
pub const fn password_needs_normalisation() -> &'static str {
    "That password contains characters outside plain ASCII, and pdfcer cannot process those the way this document's encryption requires — so it cannot open the file even if the password is correct. This is a limit in pdfcer, not a wrong password. A document whose password is plain ASCII will open normally."
}

/// The refusal for an empty box.
///
/// Refused here rather than sent on, and the reason is in
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
#[must_use]
pub const fn not_encrypted() -> &'static str {
    "This document is not encrypted. Anyone who has the file can open it."
}

/// The cipher and key length, in the operator's terms.
#[must_use]
pub fn cipher_line(cipher: Cipher) -> String {
    let described = match cipher {
        Cipher::Rc4 => "RC4, an old cipher that is no longer considered secure",
        Cipher::Aes128 => "AES-128",
        Cipher::Aes256 => "AES-256",
        // `/None` means the security handler decrypts privately and pdfcer
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

/// **The engine's own sentence, verbatim.** See the module header.
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
#[must_use]
pub const fn perms_disagree() -> &'static str {
    "⚠  The permissions written in this document's encryption dictionary do not match the encrypted copy stored alongside them. That means the stated permissions were changed after the document was encrypted. pdfcer shows what the document declares — which is what every other viewer shows — and cannot tell you which of the two was intended."
}

/// The ordinary case for an older document: there is no `/Perms` entry to check.
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

/// **The engine's own sentence, reworded by the engine.** See the module
/// header.
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

/// The tab says what it cannot do, and it says it once, at the bottom.
#[must_use]
pub const fn cannot_author() -> &'static str {
    "pdfcer can add or remove a password, change these permissions, and sign a document — File > Security. Whether anyone else trusts a signature is a separate question, and the Signatures panel is where pdfcer reports what it could and could not check."
}
