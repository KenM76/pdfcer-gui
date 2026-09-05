//! # `text::protect` — every operator-facing string on the two Security
//! controls that **write** protection into a file
//!
//! `OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04 with the instruction
//! that ended the question: *"yes add encryption and permissions … Always add
//! new features. never ask. just do."*
//!
//! ## Why this is a second module rather than more of [`crate::text::security`]
//!
//! Three reasons, and the first is the one that decides it.
//!
//! 1. **The two modules make opposite kinds of claim.** `text::security` is the
//!    READ side: it reports what a document already says about itself, and its
//!    header's standing rule is that *"it makes claims about what protects a
//!    document, and a wrong one is worse than silence."* This module is the
//!    WRITE side: every sentence here describes something pdfcer is about to
//!    **do to a file**, and the failure mode is not a wrong report, it is a
//!    wrong file. Mixing an "it is like this" catalogue with a "this is what
//!    will happen" catalogue would put the two under one reviewer's eye with
//!    one standard, and they need two.
//! 2. **R2.** `text::security` is 375 lines of dense argument and several of
//!    its neighbours in `text/` are at or near the 1,500-line ceiling. The
//!    build brief for this work names a new module for exactly that reason.
//! 3. **Everything the read side already wrote is REUSED rather than
//!    restated.** [`crate::text::security::permissions_are_advisory`],
//!    [`crate::text::security::permission_name`],
//!    [`crate::text::security::permission_state`],
//!    [`crate::text::security::cipher_line`],
//!    [`crate::text::security::auth_line`] and
//!    [`crate::text::security::not_encrypted`] are all called from the protect
//!    dialog verbatim. **A second wording of one fact is the defect**, and it
//!    is the defect `text::security`'s own header names about the engine and
//!    the CLI. So the split is by *kind of claim*, not by subject, and nothing
//!    is duplicated across it.
//!
//! ## ★★★ The three disclosures, and why they are three
//!
//! `OPERATOR_REQUESTS.md` O119 lists three things the operator said "change the
//! answer" before he gave one. All three are on screen, and the surface may not
//! ship without them:
//!
//! | # | fact | where it comes from | where it is drawn |
//! |---|---|---|---|
//! | 1 | **a permission is a request, not a lock** | the ENGINE's own sentence, already catalogued at [`crate::text::security::permissions_are_advisory`] | at the top of the permission list, in the danger role, on every job that writes permission bits |
//! | 2 | **a signed document is refused** | [`signed_refusal`] here | instead of the whole form — the dialog opens, states it, and offers nothing |
//! | 3 | **re-permissioning needs the owner password** | [`owner_password_note`] here | above the current-owner-password field, on every job that touches an already-protected file |
//!
//! ★ Number 1 is the important one and it is **not re-worded here**. The engine
//! supplied it, the CLI prints it, `text::security` catalogued it, and
//! `EncryptionSettings::PERMISSIONS_DISCLOSURE` is the same sentence in the
//! engine's own source. A UI that presented permissions as enforcement would be
//! lying, and softening the clause *"a request, not a lock"* is exactly the edit
//! a marketing instinct makes.
//!
//! ## What is deliberately NOT said
//!
//! No sentence here promises that a protected document is safe, secure, or
//! locked. The only true version of that claim is about the **user password**
//! and nothing else, and [`crate::text::security::permissions_are_advisory`]
//! already says it in the engine's own words.

use pdfcer_core::crypto::AuthKind;

use super::commands::CommandText;

// ===========================================================================
// THE RIBBON
// ===========================================================================

/// The **Security** group's caption, on the File tab.
///
/// ★ It lives here rather than in [`crate::text::ribbon`] with the other group
/// captions, on the precedent [`crate::text::acrobat::file_open_in_acrobat`]
/// set: when a feature's copy is one subject and one module, the caption is
/// part of that subject, and splitting three words off into another file buys
/// nothing but a second place to look. `crate::shell::manifest::file` calls it
/// by its full path, so the seam is visible at the call site.
#[must_use]
pub const fn group_file_security() -> &'static str {
    "Security"
}

/// `file.encrypt` — the password.
///
/// # The label is a verb and the ellipsis is a promise
///
/// *Encrypt…* rather than *Password…*, because the label has to be true of all
/// three things the control does — set a password, change it, remove it — and
/// *Password…* reads as "set one". Encryption is what the file gains or loses;
/// the password is how it is keyed.
///
/// ★ **The tooltip names all three jobs**, because the control's most surprising
/// property is that the one button also takes protection OFF. A user who wants
/// to unprotect a drawing will not look under a button called Encrypt unless it
/// says so, and the alternative — three ribbon controls for one subject — is
/// three chances to press the wrong one.
///
/// ★★ And it names the refusal in the same sentence. A signed document is
/// refused by the engine, by name, and finding that out by pressing is the R9
/// failure this project has paid for: *an unavailable capability renders
/// nothing, and a button that fails on press is worse than either.* The control
/// stays present — whether THIS document is signed is not known when the ribbon
/// is built — so the hover says what will happen.
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
///
/// ★★★ The tooltip's **first job is the disclosure**, not the description. This
/// is the one control in pdfcer whose plain reading is false: a list of
/// tick-boxes labelled Print, Copy and Change looks exactly like a set of
/// locks, and it is not one. The full sentence is on screen the moment the
/// window opens ([`crate::text::security::permissions_are_advisory`]); the
/// tooltip carries the short form so an operator who only ever hovers still
/// meets it.
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
///
/// ★★★ This section exists because of one line in the build brief, and it is
/// the strongest requirement on this surface: *"A permissions dialog that opens
/// with everything ticked, on a document that forbids printing, has told the
/// operator a falsehood before he touches anything."*
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
///
/// ★ It says **passwords**, plural, and *keep* what it keeps. The one thing an
/// operator fears about this button is that it silently re-opens a document
/// they had restricted, and saying so at the control is cheaper than a receipt
/// that says it afterwards.
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
///
/// ★ Not a scold. The operator asked for this verb by name; what the sentence
/// adds is the fact a label cannot carry — that the *result* is a file anybody
/// can open, which is the point and is also the thing to be sure of.
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

/// ★★★ **The sentence that stops the two passwords being collapsed into one.**
///
/// The build brief made this explicit: *"Owner and user passwords are different
/// things; do not collapse them into one field without saying what you did."*
/// They are not collapsed, so what is owed instead is an explanation of why
/// there are two boxes where every other program in the operator's day has one.
///
/// It is two sentences and each carries one fact: what each password *does*,
/// and what an empty user password *means*. The second is not a footnote —
/// `EncryptionSettings`' own doc calls an empty user password a
/// **permissions-only document**, and it is a genuinely useful thing to want
/// (the drawing opens with no prompt and still declares what it allows). An
/// operator who left the box blank without being told would think they had
/// failed to protect anything.
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

/// ★★★ **Disclosure 3 of O119's three: re-permissioning needs the owner
/// password.**
///
/// The engine asked for this to be surfaced by name, in its 2026-09-03 reply:
/// *"`AuthKind` tells you which one opened the file — surface that, because
/// `remove_encryption` will refuse a user-authenticated session and the operator
/// should see WHY before pressing it."*
///
/// ★ It is drawn **above the field**, not after a refusal. A refusal that
/// arrives on press is a program that knew the answer and waited.
///
/// ★★ And it says *typed here even if you already used it*, because the honest
/// alternative is worse. pdfcer does not keep the password that opened the
/// document — [`crate::secret::Secret`] exists so it does not linger — so a
/// session opened with the owner password still cannot re-key without being
/// given it again. Without this clause the operator meets a field they believe
/// they have already filled in and concludes the program has lost track.
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

/// ★★ Why every box starts ticked on a document that is not protected yet.
///
/// The build brief's rule is that the dialog must show the CURRENT state before
/// offering to change it, and on an unprotected document the current state is
/// *everything is allowed* — there is no `/Encrypt` dictionary, so there is no
/// `/P`, so nothing is being declined. Eight ticks is therefore the true
/// read-back and not a convenient default, and saying so is what stops it
/// looking like one.
#[must_use]
pub const fn permissions_start_open() -> &'static str {
    "This document is not protected, so it declines nothing today — every box below starts ticked because that is what the file currently says, not because pdfcer chose it for you."
}

/// The note beside a bit whose current value is `None` — the document's
/// encryption revision has no such concept.
///
/// ★ `Permissions::granted` returns three values and the third one is not
/// "refused". `text::security::permission_state` already renders it; what this
/// adds is the consequence *for the change about to be made*: pdfcer writes
/// `/R` 6, where every one of the eight bits means something, so a box that is
/// currently "not stated" will be stated after this.
#[must_use]
pub const fn permission_becomes_stated() -> &'static str {
    "This document's encryption is too old to have an opinion about the last four permissions. pdfcer writes the current form of encryption, in which all eight mean something — so whatever you leave ticked or unticked below will be stated in the new file."
}

/// **One row of a permission list: the permission's name, and what is said
/// about it.**
///
/// ★ A catalogued function rather than a `format!` at the two call sites, and
/// the reason is not bookkeeping. The window draws this shape **twice** — once
/// under *"What it allows today"*, reporting the document's own three-valued
/// answer, and once in the editable list, where the one row that cannot be a
/// tick-box carries [`accessibility_always_granted`] instead. Two `format!`s
/// would be two separators, and the day one of them became a colon the two
/// lists would stop reading as one kind of thing.
///
/// The name comes from [`crate::text::security::permission_name`] and the state
/// from [`crate::text::security::permission_state`] or from this module; this
/// function owns only the join.
#[must_use]
pub fn permission_row(name: &str, said: &str) -> String {
    format!("{name}  —  {said}")
}

/// ★★★ **Why one permission on the list has no tick-box.**
///
/// Found by a test on 2026-09-04, not assumed — see
/// [`crate::protect::tests::changing_the_password_keeps_what_the_document_allowed`]'s
/// header for the failure that surfaced it and
/// [`crate::protect::always_granted`] for the rule.
///
/// `pdfcer-core` sets bit 10 on **every** file it writes, regardless of what the
/// caller asked for — its rule W19, for compatibility with PDF 1.7 readers. So
/// pdfcer cannot produce a document that declines accessibility extraction, and
/// a tick-box the operator could clear would come back ticked in the file.
///
/// ★ The sentence says what the program cannot do **and** why the limitation is
/// benign, in that order. An operator who reads only the first clause has been
/// told the truth; one who reads both knows it is not worth working around.
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
///
/// ★ The default is ON, and the reason to turn it off is a real one rather than
/// an expert's curiosity: a search indexer that cannot read the title and author
/// of a drawing cannot find it. That is the trade, stated as a trade.
#[must_use]
pub const fn encrypt_metadata_note() -> &'static str {
    "On by default. Turning it off leaves the title, author and keywords readable without the password, so a search index can still find the drawing — everything else stays encrypted either way."
}

// ===========================================================================
// REFUSALS
// ===========================================================================

/// ★★★ **Disclosure 2 of O119's three: a signed document is refused.**
///
/// The engine refuses it by name (`EncryptError::SignedDocument`) and this
/// surface refuses it *before* the form is drawn — there is nothing to fill in,
/// because there is no answer that would work.
///
/// # Why the sentence explains the mechanism rather than just the rule
///
/// Because the rule sounds arbitrary and the mechanism does not. "Encryption is
/// not allowed on signed documents" invites the operator to look for a setting.
/// *"It rewrites every byte the signature covers, so the signature would no
/// longer match"* is a fact about how signing works, and it tells them the real
/// remedy: protect first, sign second.
///
/// ★ It names the count, because a document with one approval signature and a
/// document with a certification plus four approvals are different problems and
/// the operator is the one who knows which theirs is.
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
///
/// ★ Not an empty list and not eight greyed boxes. Permissions live inside the
/// `/Encrypt` dictionary — an unprotected document does not permit everything,
/// it *says nothing*, and drawing eight ticked boxes would be this surface
/// inventing a declaration the file never made.
///
/// It names the other control, because the operator's next move is a real one
/// and a refusal that does not say what to do instead is half a sentence.
#[must_use]
pub const fn not_encrypted_refusal() -> &'static str {
    "This document is not protected, so it states no permissions — there is nothing here to change. A PDF can only declare what it allows as part of being encrypted. Use Encrypt… on the same group to put a password on it and choose what it allows at the same time."
}

/// The refusal when the document has never been written to disk.
///
/// ★ Reachable only in theory, and refused rather than unwrapped. Changing the
/// protection on an already-protected document is done by re-opening the FILE
/// with the owner password (see [`crate::protect`] for the whole argument), and
/// a document created in this session has no file to re-open. A document created
/// in this session is also never encrypted, so the two conditions cannot both
/// hold — which is exactly why this is a named refusal rather than a `panic!`
/// on an "impossible" branch.
#[must_use]
pub const fn no_file_refusal() -> &'static str {
    "This document has never been saved, so there is no file for pdfcer to re-open with the owner password. Save it first, then protect it."
}

/// The engine refused the operation after the operator pressed.
///
/// ★ One function with a match rather than six strings at six call sites,
/// because the whole value of `EncryptError` is that every variant names
/// something the operator can act on, and a `to_string()` of the engine's own
/// message would put an implementer's sentence in front of a draughtsman.
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
        // ★ No count here, and that is not laziness. This arm is reached only
        // when the engine refused AFTER the press — i.e. a signature the
        // pre-flight census did not see, which by construction means the count
        // this surface holds is the one that was wrong. `signed_refusal`, drawn
        // instead of the form, is where the number belongs.
        R::Signed => signed_refusal_late().to_owned(),
        R::Rng => {
            "pdfcer could not reach this machine's random-number generator, so it could not make a key. Nothing was written. It will never substitute a weaker key to get past this.".to_owned()
        }
        R::Write(detail) => format!("pdfcer could not write the protected document: {detail}"),
    }
}

/// The password opened the file, and it was not the owner's.
///
/// ★★ It names which password DID work, which is the fact that turns a dead end
/// into a next step: an operator told only *"wrong password"* re-types the one
/// they have, and an operator told *"that is the user password"* goes and finds
/// the other one.
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
///
/// ★ It carries the engine's own detail rather than flattening every failure to
/// "wrong password", for `crate::dialogs::password`'s reason: pdfcer reports a
/// non-ASCII password that it cannot normalise as a **different** error
/// precisely so the operator is not sent to re-check a password that was
/// correct.
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
///
/// ★★ Refused rather than allowed, and this is a decision the standard does not
/// make for us: `EncryptionSettings` will happily take an empty owner password.
/// A document with one is a document whose protection **anybody can remove**,
/// which is the opposite of what the operator pressed the button for, and it
/// would be an empty box's silent consequence rather than a choice.
#[must_use]
pub const fn owner_password_required() -> &'static str {
    "The owner password cannot be blank. It is the password that lets the protection be changed or removed later — a blank one means anyone who opens the document can take the protection off."
}

/// The two passwords are the same.
///
/// ★ Refused for the reason that makes the permission list mean anything: the
/// owner password ignores `/P` entirely, so if it is also the password that
/// opens the document, every reader authenticates as owner and the permissions
/// are decoration. The engine does not enforce this (*"because the standard does
/// not"*), so the surface does.
#[must_use]
pub const fn passwords_must_differ() -> &'static str {
    "The user password and the owner password must be different. The owner password ignores every permission below, so if it is also the password people use to open the document, nothing on this list will restrict anybody."
}

/// The owner password to authorise with is empty.
#[must_use]
pub const fn current_owner_password_required() -> &'static str {
    "Type the document's current owner password to authorise this change."
}

/// ★★ Why the confirm control is greyed, naming the outstanding condition.
///
/// `OPERATOR_REQUESTS.md` O77's sweep found seven greyed controls with no hover
/// explanation, and the reasoning `crate::text::redact::confirm_disabled`
/// records applies unchanged: several different conditions gate this one button
/// and they appear at different times, so *"fill in the form"* would be vague
/// exactly when it matters.
///
/// The flags are **outstanding** conditions, not satisfied ones, and each is
/// computed from the same expression that decides whether its control is drawn.
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
///
/// ★★★ The whole destination mechanism is `crate::dialogs::redact`'s, followed
/// deliberately rather than re-invented — see [`crate::dialogs::protect`]'s
/// header. The wording differs only where the act differs: a redaction destroys
/// content, and this replaces a file.
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
///
/// ★ Softer than the redaction's equivalent, and deliberately: nothing here
/// destroys content. What it destroys is the **unprotected copy**, and on the
/// remove-protection job the opposite — the protected copy. Both are recoverable
/// only by having kept the other one, which is what the sentence says.
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
///
/// ★ A suggestion, and it is never the source file — the standing rule this
/// project applies to every write that produces a second document
/// (`crate::text::redact::suggested_suffix`,
/// `crate::text::files::save_copy_suffix`). Here the reason is milder and still
/// real: the two files differ only in their protection, and two identical-looking
/// drawings one of which is protected is the pair an operator most needs told
/// apart by name.
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
///
/// ★ The label is the consequence and the ellipsis is a promise that a further
/// question is coming — `crate::text::redact::confirm_button`'s rule, and the
/// same one decides the replace form below.
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

/// ★★ **The sentence shown once bytes are on disk.**
///
/// It carries the fact the operator would otherwise discover by looking at a
/// window that disagrees with the file: **the open document is unchanged.**
/// This is `crate::dialogs::redact`'s ruling and it is stronger here, because
/// the divergence is invisible — a redacted page looks different, and a
/// protected file looks identical.
///
/// The replace form names the file to re-open. The new-file form does not need
/// to, because nothing the operator is looking at has become wrong.
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

/// ★ The SASLprep gap, surfaced only when a typed password contains a non-ASCII
/// byte.
///
/// The engine hands this over as `EncryptionSettings::SASLPREP_GAP` and asks for
/// it to be shown when `has_non_ascii_password()` is true. It is **conditional**
/// for `crate::dialogs::redact`'s standing reason about acknowledgements: a
/// warning that is always on screen is a warning nobody reads, and this one is
/// irrelevant to the overwhelming majority of passwords.
///
/// ★★ It is a warning rather than a refusal, because the password may well be
/// perfectly interoperable and pdfcer cannot know. Refusing every accented
/// character would be this program declining to write a file the standard
/// permits.
#[must_use]
pub const fn saslprep_gap() -> &'static str {
    "⚠  That password contains characters outside plain ASCII. pdfcer applies passwords as UTF-8 cut to 127 bytes rather than the full normalisation the standard specifies, so a different reader may not accept this password even when it is typed correctly. An ASCII password is handled exactly."
}
