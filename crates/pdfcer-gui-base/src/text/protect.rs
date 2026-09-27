//! # `text::protect` — every operator-facing string on the two Security
//! controls that **write** protection into a file
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/protect.md`.

use pdfcer_core::crypto::AuthKind;

use super::commands::CommandText;

// ===========================================================================
// THE RIBBON
// ===========================================================================

/// The **Security** group's caption, on the File tab.
#[must_use]
pub const fn group_file_security() -> &'static str {
    "Security"
}

/// `file.encrypt` — the password.
#[must_use]
pub const fn file_encrypt() -> CommandText {
    CommandText::new(
        "Encrypt…",
        "Put a password on this document, change the password it has, or take \
         the protection off again. It writes a new file rather than changing \
         the one you have open, and it is refused on a signed document because \
         it rewrites every byte the signature covers.",
    )
}

/// `file.permissions` — what the document says it allows.
#[must_use]
pub const fn file_permissions() -> CommandText {
    CommandText::new(
        "Permissions…",
        "Choose what a protected document says it allows — printing, copying, \
         changing, form filling and the rest. These are a request to the \
         program that opens the file, not a lock: only the password keeps \
         anyone out. Changing them needs the owner password.",
    )
}

// ===========================================================================
// THE WINDOW
// ===========================================================================

/// The window's title when it was opened from **Encrypt…**.
#[must_use]
pub const fn title_password() -> &'static str {
    "Encrypt this document"
}

/// The window's title when it was opened from **Permissions…**.
#[must_use]
pub const fn title_permissions() -> &'static str {
    "What this document allows"
}

/// The heading over the read-back of what the document says **today**.
#[must_use]
pub const fn standing_heading() -> &'static str {
    "This document, as it is now"
}

/// The heading over the controls that change it.
#[must_use]
pub const fn change_heading() -> &'static str {
    "What to change"
}

// ===========================================================================
// THE THREE JOBS
// ===========================================================================

/// The radio label for *set a password on a document that has none*.
#[must_use]
pub const fn job_set() -> &'static str {
    "Put a password on this document"
}

/// The radio label for *change the passwords a protected document has*.
#[must_use]
pub const fn job_change() -> &'static str {
    "Change the passwords, keeping what the document allows"
}

/// The radio label for *take the protection off*.
#[must_use]
pub const fn job_remove() -> &'static str {
    "Remove the protection entirely"
}

/// What removing actually leaves behind, at the control that does it.
#[must_use]
pub const fn job_remove_note() -> &'static str {
    "The file this writes has no password and no permissions. Anyone who has it can open it, print it and change it."
}

// ===========================================================================
// PASSWORDS
// ===========================================================================

/// The heading over the password fields.
#[must_use]
pub const fn passwords_heading() -> &'static str {
    "Passwords"
}

/// **The sentence that stops the two passwords being collapsed into one.**
#[must_use]
pub const fn passwords_explained() -> &'static str {
    "The user password is the one that opens the document. The owner password is the one that changes these settings later, and it opens the document too. Leave the user password blank to make a document that opens with no prompt but still states what it allows."
}

/// The label on the user-password field.
#[must_use]
pub const fn user_password_label() -> &'static str {
    "User password (opens the document)"
}

/// The label on its confirmation.
#[must_use]
pub const fn user_password_again_label() -> &'static str {
    "…and again"
}

/// The label on the owner-password field.
#[must_use]
pub const fn owner_password_label() -> &'static str {
    "Owner password (changes these settings)"
}

/// The label on its confirmation.
#[must_use]
pub const fn owner_password_again_label() -> &'static str {
    "…and again"
}

/// The label on the field that authorises the change to an already-protected
/// document.
#[must_use]
pub const fn current_owner_password_label() -> &'static str {
    "The document's current owner password"
}

/// **Disclosure 3 of O119's three: re-permissioning needs the owner
/// password.**
#[must_use]
pub const fn owner_password_note() -> &'static str {
    "Changing or removing the protection on a document that already has it needs the OWNER password — not the password that merely opens it. It has to be typed here even if you already used it to open the document, because pdfcer does not keep passwords after it has used them."
}

/// Which password opened the document, said in the operator's terms.
///
/// A thin wrapper over [`crate::text::security::auth_line`], so the one sentence
/// serves both the read-side surface and this one.
#[must_use]
pub const fn opened_with(auth: AuthKind) -> &'static str {
    crate::text::security::auth_line(auth)
}

// ===========================================================================
// PERMISSIONS
// ===========================================================================

/// The heading over the tick-boxes.
#[must_use]
pub const fn permissions_heading() -> &'static str {
    "What the protected document will allow"
}

/// The heading over the read-back of the bits as they stand.
#[must_use]
pub const fn permissions_now_heading() -> &'static str {
    "What it allows today"
}

/// Why every box starts ticked on a document that is not protected yet.
#[must_use]
pub const fn permissions_start_open() -> &'static str {
    "This document is not protected, so it declines nothing today — every box below starts ticked because that is what the file currently says, not because pdfcer chose it for you."
}

/// The note beside a bit whose current value is `None` — the document's
/// encryption revision has no such concept.
#[must_use]
pub const fn permission_becomes_stated() -> &'static str {
    "This document's encryption is too old to have an opinion about the last four permissions. pdfcer writes the current form of encryption, in which all eight mean something — so whatever you leave ticked or unticked below will be stated in the new file."
}

/// **One row of a permission list: the permission's name, and what is said
/// about it.**
#[must_use]
pub fn permission_row(name: &str, said: &str) -> String {
    format!("{name}  —  {said}")
}

/// **Why one permission on the list has no tick-box.**
#[must_use]
pub const fn accessibility_always_granted() -> &'static str {
    "Always allowed, and pdfcer cannot turn it off. Every PDF writer is required to leave this permission granted so that screen readers keep working, so there is no tick-box for it — a file pdfcer writes will permit extraction for accessibility whatever else it declines."
}

/// The `/EncryptMetadata` checkbox.
#[must_use]
pub const fn encrypt_metadata_label() -> &'static str {
    "Encrypt the document's metadata as well"
}

/// What that switch is actually for.
#[must_use]
pub const fn encrypt_metadata_note() -> &'static str {
    "On by default. Turning it off leaves the title, author and keywords readable without the password, so a search index can still find the drawing — everything else stays encrypted either way."
}

// ===========================================================================
// REFUSALS
// ===========================================================================

/// **Disclosure 2 of O119's three: a signed document is refused.**
#[must_use]
pub fn signed_refusal(signatures: usize) -> String {
    format!(
        "This document carries {signatures} digital signature(s), so pdfcer will not protect it. Putting a password on a document rewrites every byte in the file, including the bytes the signature covers — the signature would no longer match what it signed, and every reader would report it as broken. Protect the drawing first and sign it afterwards; there is no order in which both can be done to the same file."
    )
}

/// The same refusal when it arrives from the engine rather than from this
/// surface's own census — see [`engine_refusal`]'s `Signed` arm for when that
/// can happen and why it carries no count.
#[must_use]
pub const fn signed_refusal_late() -> &'static str {
    "pdfcer will not protect a signed document. Putting a password on a document rewrites every byte in the file, including the bytes the signature covers — the signature would no longer match what it signed. Protect the drawing first and sign it afterwards. Nothing was written."
}

/// The refusal when **Permissions…** is opened on a document that carries no
/// encryption at all.
#[must_use]
pub const fn not_encrypted_refusal() -> &'static str {
    "This document is not protected, so it states no permissions — there is nothing here to change. A PDF can only declare what it allows as part of being encrypted. Use Encrypt… on the same group to put a password on it and choose what it allows at the same time."
}

/// The refusal when the document has never been written to disk.
#[must_use]
pub const fn no_file_refusal() -> &'static str {
    "This document has never been saved, so there is no file for pdfcer to re-open with the owner password. Save it first, then protect it."
}

/// The refusal when a deferred redaction is armed and the operator asks to
/// change the document's protection.
#[must_use]
pub const fn redaction_pending_refusal() -> &'static str {
    "A redaction is armed on this document and has not been applied yet. Changing the protection \
     now would write the version that still contains what you marked for removal, so pdfcer \
     refuses it. Apply the redaction, or call it off, on the Redact group of the Edit tab, and \
     then set the protection."
}

/// The engine refused the operation after the operator pressed.
#[must_use]
pub fn engine_refusal(refusal: &crate::protect::EngineRefusal) -> String {
    use crate::protect::EngineRefusal as R;
    match refusal {
        R::AlreadyEncrypted => {
            "This document is already protected, so a password cannot be added to it. Change the passwords it has, or remove the protection and put a new one on.".to_owned()
        }
        R::NotEncrypted => {
            "This document is not protected, so there is nothing to change or remove.".to_owned()
        }
        R::NotOwner { opened_as } => not_owner(*opened_as),
        // No count here, and that is not laziness. This arm is reached only
        // when the engine refused AFTER the press — i.e. a signature the
        // pre-flight census did not see, which by construction means the count
        // this surface holds is the one that was wrong. `signed_refusal`, drawn
        // instead of the form, is where the number belongs.
        R::Signed => signed_refusal_late().to_owned(),
        R::Rng => {
            "pdfcer could not reach this machine's random-number generator, so it could not make a key. Nothing was written. It will never substitute a weaker key to get past this.".to_owned()
        }
        R::RedactionPending => redaction_pending_refusal().to_owned(),
        R::Write(detail) => format!("pdfcer could not write the protected document: {detail}"),
    }
}

/// The password opened the file, and it was not the owner's.
#[must_use]
pub fn not_owner(opened_as: AuthKind) -> String {
    let which = match opened_as {
        AuthKind::EmptyUser => {
            "the document's empty user password, which every reader tries silently"
        }
        AuthKind::User => "the user password",
        AuthKind::Owner => "the owner password",
    };
    format!(
        "That opened the document with {which}, which is not enough to change what it allows. Only the owner password can re-key a protected document. Nothing was written."
    )
}

/// The password did not open the file at all.
#[must_use]
pub fn reopen_failed(detail: &str) -> String {
    format!(
        "pdfcer could not open the file with that owner password: {detail}. Nothing was written."
    )
}

// ===========================================================================
// LOCAL REFUSALS — the ones this surface makes on its own
// ===========================================================================

/// The two copies of a password do not match.
#[must_use]
pub const fn passwords_differ() -> &'static str {
    "The two copies of the password are not the same. Nothing is written until they match, because a password typed wrong twice is a document nobody can open."
}

/// The owner-password box is empty.
#[must_use]
pub const fn owner_password_required() -> &'static str {
    "The owner password cannot be blank. It is the password that lets the protection be changed or removed later — a blank one means anyone who opens the document can take the protection off."
}

/// The two passwords are the same.
#[must_use]
pub const fn passwords_must_differ() -> &'static str {
    "The user password and the owner password must be different. The owner password ignores every permission below, so if it is also the password people use to open the document, nothing on this list will restrict anybody."
}

/// The owner password to authorise with is empty.
#[must_use]
pub const fn current_owner_password_required() -> &'static str {
    "Type the document's current owner password to authorise this change."
}

/// Why the confirm control is greyed, naming the outstanding condition.
#[must_use]
pub fn confirm_disabled(
    current_owner_missing: bool,
    owner_missing: bool,
    mismatch: bool,
    same: bool,
    overwrite_unacknowledged: bool,
) -> String {
    let mut reasons: Vec<&str> = Vec::new();
    if current_owner_missing {
        reasons.push(current_owner_password_required());
    }
    if owner_missing {
        reasons.push(owner_password_required());
    }
    if mismatch {
        reasons.push(passwords_differ());
    }
    if same {
        reasons.push(passwords_must_differ());
    }
    if overwrite_unacknowledged {
        reasons.push(overwrite_outstanding());
    }
    reasons.join("\n\n")
}

/// The outstanding-condition line for the replace acknowledgement.
#[must_use]
pub const fn overwrite_outstanding() -> &'static str {
    "Tick the box confirming that the file you have open will be replaced."
}

// ===========================================================================
// DESTINATION — the same choice `dialogs::redact` offers, for the same reason
// ===========================================================================

/// The heading over the destination radios.
#[must_use]
pub const fn destination_heading() -> &'static str {
    "Where should the protected document go?"
}

/// The safe destination, and the default.
#[must_use]
pub const fn destination_new_file() -> &'static str {
    "A new file — you choose the name"
}

/// Why the default is the default.
#[must_use]
pub const fn destination_new_file_tooltip() -> &'static str {
    "The document you have open is left exactly as it is, and the file it came from is untouched."
}

/// The destination that replaces the source document. Names the file.
#[must_use]
pub fn destination_replace(file_name: &str) -> String {
    format!("Replace {file_name} with the protected document")
}

/// The consequence of replacing, stated where it is chosen.
#[must_use]
pub const fn destination_replace_tooltip() -> &'static str {
    "The file on disk is overwritten. Nothing in this document is lost, but the version you had — protected or not — is gone unless you kept a copy of it yourself."
}

/// The acknowledgement asked for only when the operator has chosen to replace.
#[must_use]
pub fn overwrite_acknowledgement_checkbox(file_name: &str) -> String {
    format!("I understand that {file_name} will be REPLACED by the protected document.")
}

/// The title on the system file-save dialog.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Save protected document"
}

/// The suffix appended to the original file's stem to suggest a name.
#[must_use]
pub const fn suggested_suffix() -> &'static str {
    "-protected"
}

/// The suffix used when the job is to **remove** protection.
#[must_use]
pub const fn suggested_suffix_unprotected() -> &'static str {
    "-unprotected"
}

// ===========================================================================
// THE CONFIRM CONTROL
// ===========================================================================

/// The confirm control's label when a picker is still to come.
#[must_use]
pub fn confirm_button(job_label: &str) -> String {
    format!("{job_label} & save as…")
}

/// The confirm control's label when the destination is the open file.
#[must_use]
pub fn confirm_button_replace(job_label: &str, file_name: &str) -> String {
    format!("{job_label} & replace {file_name} now")
}

/// The verb phrase each job contributes to the confirm control's label.
#[must_use]
pub const fn job_verb(job: crate::protect::Job) -> &'static str {
    use crate::protect::Job as J;
    match job {
        J::SetPassword => "Protect",
        J::ChangePassword => "Change the passwords",
        J::RemovePassword => "Remove the protection",
        J::SetPermissions => "Set what it allows",
    }
}

/// The control that closes without writing.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Close without changing anything"
}

// ===========================================================================
// OUTCOMES
// ===========================================================================

/// **The sentence shown once bytes are on disk.**
#[must_use]
pub fn written(job: crate::protect::Job, file_name: &str, replaced: bool) -> String {
    use crate::protect::Job as J;
    let what = match job {
        J::SetPassword => "It is protected with a password and states what it allows.",
        J::ChangePassword => {
            "It carries the new passwords, and allows exactly what it allowed before."
        }
        J::RemovePassword => {
            "It has no password and no permissions — anyone who has it can open it."
        }
        J::SetPermissions => "It states the permissions you chose, under the passwords you gave.",
    };
    if replaced {
        format!(
            "Written — {file_name} has been replaced. {what} ⚠  The window you are looking at still shows the document as it was, because pdfcer cannot change a document's protection in place — close it and open {file_name} again to work with the file as it now is."
        )
    } else {
        format!(
            "Written — {file_name}. {what} The document you have open is unchanged and still points at the file it came from."
        )
    }
}

/// A destination was named and no file appeared.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("Nothing was written: {detail}")
}

/// The SASLprep gap, surfaced only when a typed password contains a non-ASCII
/// byte.
#[must_use]
pub const fn saslprep_gap() -> &'static str {
    "⚠  That password contains characters outside plain ASCII. pdfcer applies passwords as UTF-8 cut to 127 bytes rather than the full normalisation the standard specifies, so a different reader may not accept this password even when it is typed correctly. An ASCII password is handled exactly."
}
