//! # `protect` — putting a password on a document, changing what it allows,
//! and taking the protection off
//!
//! `OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04. The window is
//! [`crate::dialogs::protect`]; this module is everything that can be decided
//! **without a `Ui`** — what the document says today, what may be offered about
//! it, which engine verb a choice reaches, and the atomic write at the end.
//!
//! The split is `crate::redact`'s and it exists for the same reason: every rule
//! on this surface is a rule about **the operator's file**, and a rule that can
//! only be exercised by driving a window is a rule that gets asserted once, by
//! hand, and then drifts.
//!
//! ---
//!
//! # 1. ★★★ The three engine verbs, verified against the source
//!
//! `D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs`, at the revision this crate
//! pins (`Cargo.lock`: `pdfcer-core 0.32.0`, `aa27596`). Checked at that commit
//! rather than at `main`, because `main` is 0.34.0 and what compiles here is the
//! pin:
//!
//! ```text
//! set_encryption   (&self,     &EncryptionSettings, &SaveOptions) -> Result<(Vec<u8>, SaveReport), EncryptError>
//! set_permissions  (&mut self, &EncryptionSettings, &SaveOptions) -> Result<(Vec<u8>, SaveReport), EncryptError>
//! remove_encryption(&mut self,                      &SaveOptions) -> Result<(Vec<u8>, SaveReport), EncryptError>
//! ```
//!
//! `docs/core-api/02-editing-and-saving.md` §1.5 states the same three, and the
//! two agree.
//!
//! ## ★★ The one place the engine's docs and the engine's source disagree
//!
//! `set_encryption`'s own rustdoc instructs a caller to *"surface
//! [`EncryptionSettings::permissions_disclosure`]"* and names
//! `EncryptionSettings::saslprep_gap` beside it. **Neither item exists.** They
//! are associated **constants** and they are `SCREAMING_CASE`:
//! `EncryptionSettings::PERMISSIONS_DISCLOSURE` and
//! `EncryptionSettings::SASLPREP_GAP` — which is what `docs/core-api` names and
//! what compiles. Two broken intra-doc links, nothing more, and recorded here
//! because the build brief asked for every difference between the docs and the
//! source: a caller who trusted the rustdoc would look for a method, not find
//! one, and conclude the disclosure was not supplied.
//!
//! # 2. ★★★ Why NONE of the three verbs is called on the open session
//!
//! This is the decision the whole module is shaped around, and it was taken on
//! evidence rather than caution.
//!
//! Two of the three take **`&mut self`**, and what they mutate is not an edit —
//! it is the session's own record of what the document IS:
//!
//! * `set_permissions` calls `self.base.clear_encryption()`;
//! * `remove_encryption` calls `self.base.clear_encryption()` **and**
//!   `self.trailer.remove(b"Encrypt")`.
//!
//! Now read `crate::writer::save_incremental`'s first guard, at
//! `writer/save.rs:316`: a document whose base is encrypted is refused with
//! `WriteError::EncryptedSaveUnsupported`, and that error's own doc explains
//! why in terms this shell must not undo — saving an encrypted document
//! verbatim would produce *"one that no reader can open, including pdfcer, and
//! it would look like a successful save."*
//!
//! ⇒ **So calling either mutating verb on the open session removes the guard
//! that stops the NEXT ordinary Save producing exactly that file.** The base
//! stops reporting itself as encrypted, `save_incremental` stops refusing, and
//! the operator's next `Ctrl+S` appends plaintext objects to a file whose
//! existing objects are AES ciphertext. Nothing would report a failure.
//!
//! That is not a risk to be weighed against convenience. It is the one outcome
//! this project's rules forbid outright, so the open session is never handed to
//! these verbs.
//!
//! ## What is done instead — and it differs by job, because the facts differ
//!
//! | the document is… | the verb | the session it is called on | do unsaved edits travel? |
//! |---|---|---|---|
//! | **not** encrypted | `set_encryption` (`&self`) | **the open one** | **yes** |
//! | encrypted | `set_permissions` / `remove_encryption` (`&mut self`) | a **throwaway**, loaded from the file with the owner password | there are none — see below |
//!
//! ★ The first row needs no ceremony at all: `set_encryption` takes `&self`,
//! mutates nothing, and applies `dirty_set()` — so an operator who has moved a
//! dimension and not saved gets that dimension in the protected file. This is
//! strictly better than re-reading the disk and it costs nothing.
//!
//! ★★ The second row loses no work, and that is a fact about the engine rather
//! than a claim about this code. `pdfcer-core` **refuses every content edit on
//! an encrypted document by name** — the engine's own regression test says so:
//! `an_encrypted_session_still_refuses_a_content_edit`, whose comment reads
//! *"the guards are load-bearing the moment an encrypted document can carry a
//! session (which it now can)"*. And an encrypted document cannot be saved
//! either, by `EncryptedSaveUnsupported` above. So an open encrypted document
//! has no unsaved edits to carry: there is no verb that could have made one.
//!
//! ★★★ **The throwaway load is also the authentication.** Both mutating verbs
//! are owner-only and refuse `NotOwner { opened_as }`; the way this module finds
//! out whether the operator has the owner password is by **using it** —
//! `Document::load_with_password(path, Some(owner))`, then reading
//! `encryption().auth`. One act, no second code path, and the failure is the
//! engine's own rather than a guess made here. It also means pdfcer never has to
//! keep the password that opened the document, which is what
//! [`crate::secret::Secret`] exists to prevent.
//!
//! # 3. Why the write is a destination CHOICE, and whose precedent that is
//!
//! `crate::dialogs::redact`'s, followed rather than re-argued — the operator
//! settled this shape hours before this work started, on the redaction, in his
//! own words: *"why does it have to save to a new file right away? Why can't it
//! just wait on saving until I choose to save over the existing file or save as
//! a new file?"*
//!
//! So the same three things hold here:
//!
//! 1. **A new file is the default** and [`suggested_path`] never proposes the
//!    source. A safe default is a mechanism; a warning is something to click
//!    past.
//! 2. **Replacing the original is offered**, gated behind one extra
//!    acknowledgement that names the file, and it takes **no picker** — a
//!    picker pre-filled with the source is the shape of every accidental
//!    overwrite there has ever been.
//! 3. **The write is atomic** — temp file, then rename ([`Prepared::write_to`]).
//!
//! ★ The half of his request the engine cannot express is the same half here as
//! there: *defer the write to a later Save*. There is no verb for it. All three
//! encryption verbs **return bytes**; none of them stages anything in a session,
//! and `EditSession` has no `replace_document`. Approximating it would mean
//! swapping a second session under the open document and silently discarding its
//! undo log, which `crate::app::save::save_as` refuses for the same reason.
//!
//! # 4. What happens to the open document: nothing, and it is disclosed
//!
//! Exactly `crate::dialogs::redact`'s outcome, and the divergence matters more
//! here because it is **invisible**: a redacted page looks different, and a
//! protected file looks identical to the one it came from.
//!
//! So after a replace, the window is deliberately stale and
//! [`crate::text::protect::written`]'s replace form says so by name, telling the
//! operator which file to re-open. Rule 4: report separately, and do not pretend.

use std::path::{Path, PathBuf};

use pdfcer_core::crypto::{AuthKind, Cipher, PermissionBit};
use pdfcer_core::document::Document;
use pdfcer_core::edit::{EditSession, EncryptError, EncryptionSettings};

use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use crate::secret::Secret;

// ---------------------------------------------------------------------------
// What the operator asked for
// ---------------------------------------------------------------------------

/// Which ribbon control opened the window.
///
/// ★ Two commands, one window. The alternative — two windows — would put the
/// password fields, the destination choice, the disclosures and the atomic write
/// in two files, and the second copy is where a disclosure goes missing. What
/// the task decides is the **title**, which of the jobs is offered, and which
/// section the window opens on; everything below that is one implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    /// **Encrypt…** — the password: set it, change it, or remove it.
    Password,
    /// **Permissions…** — what a protected document says it allows.
    Permissions,
}

/// What the operator has chosen to do, once the document's own state has
/// narrowed the field.
///
/// ★ Four values rather than three, because *change the passwords* and *change
/// what it allows* reach the same engine verb (`set_permissions`) with opposite
/// intentions, and the difference is what the surface must protect: a password
/// change **preserves** the permission bits the document already has, and a
/// permission change **preserves** nothing about the passwords because the
/// engine cannot recover them (see [`Standing::preserved_grants`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Job {
    /// Plaintext document ⇒ protected. `EditSession::set_encryption`.
    SetPassword,
    /// Protected ⇒ re-keyed under new passwords, permissions carried over
    /// unchanged. `EditSession::set_permissions`.
    ChangePassword,
    /// Protected ⇒ plaintext. `EditSession::remove_encryption`.
    RemovePassword,
    /// Protected ⇒ re-keyed with a new `/P`. `EditSession::set_permissions`.
    SetPermissions,
}

impl Job {
    /// Whether this job writes a permission set the operator can see and edit.
    ///
    /// Decides whether the permission list is drawn as **controls** or as a
    /// read-back, and therefore whether
    /// [`crate::text::security::permissions_are_advisory`] is drawn beside
    /// controls the operator is about to use or beside a report.
    #[must_use]
    pub const fn edits_permissions(self) -> bool {
        matches!(self, Self::SetPassword | Self::SetPermissions)
    }

    /// Whether this job needs the document's **current** owner password.
    ///
    /// True for every job that acts on an already-protected document, which is
    /// every job but the first — and it is the condition that decides whether
    /// [`crate::text::protect::owner_password_note`], O119's third disclosure,
    /// is on screen.
    #[must_use]
    pub const fn needs_current_owner(self) -> bool {
        !matches!(self, Self::SetPassword)
    }

    /// Whether this job asks the operator for **new** passwords.
    ///
    /// False only for removal, which sets none.
    #[must_use]
    pub const fn sets_new_passwords(self) -> bool {
        !matches!(self, Self::RemovePassword)
    }
}

// ---------------------------------------------------------------------------
// What the document says today
// ---------------------------------------------------------------------------

/// **The document's protection as it stands, read before anything is offered.**
///
/// ★★★ The reason this type exists at all is one line of the build brief:
/// *"A permissions dialog that opens with everything ticked, on a document that
/// forbids printing, has told the operator a falsehood before he touches
/// anything."* Every control on the window is seeded from a field here, and
/// nothing on it has a hard-coded default.
#[derive(Debug, Clone)]
pub struct Standing {
    /// Whether the document carries an `/Encrypt` dictionary.
    pub encrypted: bool,
    /// The cipher, when it is encrypted. `stream_cipher`, which is the one an
    /// operator's question — *how strong is this* — is about.
    pub cipher: Option<Cipher>,
    /// Which password opened it, when it is encrypted.
    pub auth: Option<AuthKind>,
    /// The handler revision, which decides whether the last four permission
    /// bits mean anything at all. Zero when the document is not encrypted.
    pub revision: u8,
    /// **Every** permission bit, in Table 22 order, with the document's own
    /// three-valued answer.
    ///
    /// ★ `Option<bool>` all the way to the screen, never flattened. `None` is
    /// *"this document's encryption revision has no such concept"*, which is not
    /// `Some(false)` — rendering it as refused would show the operator a
    /// restriction nobody wrote. `PermissionBit`'s own doc makes the same point
    /// about enumerating all eight: *"a partial list would be worse than none."*
    pub grants: Vec<(PermissionBit, Option<bool>)>,
    /// How many digital signatures the document carries.
    pub signatures: usize,
    /// Whether [`OpenDoc::path`] names a file that exists.
    ///
    /// Asked of the **file system** rather than carried as a flag, exactly as
    /// `crate::app::save::has_a_file` asks it and for the reason recorded there:
    /// a second source of truth drifts, and the failure when it does is writing
    /// over the wrong file.
    pub on_disk: bool,
}

impl Standing {
    /// Read it off the open document.
    #[must_use]
    pub fn read(session: &EditSession, path: &Path) -> Self {
        let base = session.document();
        let census = session.signature_census();
        let encryption = base.encryption();
        let grants = encryption.map_or_else(
            || {
                // ★ Not encrypted: every bit is GRANTED, and that is a
                // read-back rather than a default. A document with no
                // `/Encrypt` declines nothing — there is no `/P` in which to
                // decline it — so eight ticks is what the file actually says.
                // `crate::text::protect::permissions_start_open` puts that
                // sentence on screen so it does not merely look convenient.
                PermissionBit::all()
                    .iter()
                    .map(|bit| (*bit, Some(true)))
                    .collect()
            },
            |enc| {
                let permissions = enc.config.permissions();
                PermissionBit::all()
                    .iter()
                    .map(|bit| (*bit, permissions.granted(*bit)))
                    .collect()
            },
        );
        Self {
            encrypted: encryption.is_some(),
            cipher: encryption.map(|e| e.config.stream_cipher),
            auth: encryption.map(|e| e.auth),
            revision: encryption.map_or(0, |e| e.config.revision),
            grants,
            signatures: census.signatures,
            on_disk: path.is_file(),
        }
    }

    /// **Whether this surface may offer anything at all, and if not, why.**
    ///
    /// Pure, and the whole of R9's rule for these two controls: *no
    /// placeholders — the control is absent or explained, never a button that
    /// fails on press.* The controls stay on the ribbon, because whether THIS
    /// document is signed is not known when the registry is built; the window
    /// opens and states the refusal instead of drawing a form whose only
    /// possible outcome is a failure.
    #[must_use]
    pub fn refusal(&self, task: Task) -> Option<Refusal> {
        // ★ Signed first, and it outranks everything. Both mutating verbs and
        // `set_encryption` refuse `SignedDocument`, so no job on either control
        // can succeed and there is nothing to choose between.
        if self.signatures > 0 {
            return Some(Refusal::Signed {
                signatures: self.signatures,
            });
        }
        if task == Task::Permissions && !self.encrypted {
            return Some(Refusal::NotEncrypted);
        }
        // ★ Only the encrypted branch needs a file: it is the branch that
        // re-opens one to authenticate as owner (§2). A document created in
        // this session is never encrypted, so this is belt-and-braces — and it
        // is a named refusal rather than an `unwrap` on an "impossible" branch,
        // which is this project's standing preference.
        if self.encrypted && !self.on_disk {
            return Some(Refusal::NoFile);
        }
        None
    }

    /// The jobs this document may be offered, for the control that was pressed.
    ///
    /// Pure, and the single source of what appears on the window — so the radio
    /// group, the confirm control's label and the engine call cannot disagree
    /// about what is being done.
    #[must_use]
    pub fn jobs(&self, task: Task) -> Vec<Job> {
        match (task, self.encrypted) {
            (Task::Password, false) => vec![Job::SetPassword],
            (Task::Password, true) => vec![Job::ChangePassword, Job::RemovePassword],
            // ★ On an unprotected document this is empty and the window never
            // gets here — `refusal` has already returned `NotEncrypted`. It is
            // still written as the honest answer rather than as a `panic!`,
            // because a function that returns "the jobs" should return them.
            (Task::Permissions, false) => Vec::new(),
            (Task::Permissions, true) => vec![Job::SetPermissions],
        }
    }

    /// **The permission bits to carry over unchanged**, for a job that must not
    /// alter them.
    ///
    /// ★★★ This is what makes *change the passwords* a safe verb. It reaches
    /// `set_permissions`, which takes a whole `EncryptionSettings` and re-derives
    /// `/O`, `/U`, `/OE`, `/UE` and `/Perms` from scratch — so a caller that did
    /// not supply the current bits would silently **grant everything** to a
    /// document that had been restricting things, and the operator would have
    /// changed a password and quietly unlocked the drawing.
    ///
    /// `None` — the bit is not meaningful at this document's revision — becomes
    /// **granted**, and that is the conservative reading in the direction that
    /// matters. pdfcer writes `/R` 6, where all eight bits mean something, so
    /// every bit must take a side. An `/R` 2 document's author did not decline
    /// to permit form-filling; the concept did not exist to decline, and turning
    /// their silence into a prohibition would invent a restriction they never
    /// wrote. `PermissionBit::applies_at`'s own doc says exactly that.
    #[must_use]
    pub fn preserved_grants(&self) -> Vec<PermissionBit> {
        self.grants
            .iter()
            .filter(|(_, granted)| granted.unwrap_or(true))
            .map(|(bit, _)| *bit)
            .collect()
    }

    /// The tick-box state the permission list opens with: the document's own
    /// answer, with `None` read as granted for [`Self::preserved_grants`]'s
    /// reason.
    ///
    /// ★★ This is deliberately **not** the same list as [`Self::grants`], and
    /// the difference is a difference of tense. `grants` is *what this file
    /// says today* and is drawn under
    /// `crate::text::protect::permissions_now_heading`; this is *what the file
    /// pdfcer is about to write will say*, and it is the seed for controls the
    /// operator can move.
    ///
    /// They differ in exactly one place, and only on a document some other
    /// program wrote: a bit for which [`always_granted`] holds is forced on
    /// here even when the document declines it, because the engine will grant
    /// it on the way out no matter what this surface passes. Seeding the box
    /// from the file would show an unticked control that becomes ticked in the
    /// written result — a promise the program cannot keep.
    #[must_use]
    pub fn initial_ticks(&self) -> Vec<(PermissionBit, bool)> {
        self.grants
            .iter()
            .map(|(bit, granted)| (*bit, always_granted(*bit) || granted.unwrap_or(true)))
            .collect()
    }

    /// Whether any bit is *not stated at this encryption level* — the condition
    /// that draws [`crate::text::protect::permission_becomes_stated`].
    #[must_use]
    pub fn has_unstated_bits(&self) -> bool {
        self.grants.iter().any(|(_, granted)| granted.is_none())
    }
}

/// **Whether pdfcer is capable of DECLINING this permission at all.**
///
/// ★★★ Discovered by a test rather than assumed, on 2026-09-04, and it is the
/// one place this surface must not offer the operator a choice.
///
/// `pdfcer_core::crypto::encrypt::assemble_permissions` implements the engine's
/// write-path rule **W19**, and its own doc states the clause verbatim:
///
/// > bit **10** — writers `shall` always set it to 1 for 1.7-reader
/// > compatibility, regardless of whether accessibility extraction is granted
/// > (at `/R` 6 the bit no longer gates it).
///
/// Bit 10 is [`PermissionBit::AccessibilityExtract`]. The engine sets it on
/// **every** file it writes, whether or not the caller listed it in
/// `EncryptionSettings::permissions`, and the read side then reports it as
/// granted — correctly, because the file does say so.
///
/// ⇒ A tick-box for this bit would be a control the operator can clear and
/// which comes back ticked in the file that is written. That is the exact shape
/// of falsehood the build brief forbids — *"a permissions dialog that opens with
/// everything ticked, on a document that forbids printing, has told him a
/// falsehood before he touches anything"* — only worse, because it would happen
/// **after** he touched it. So the row is drawn as a fixed statement with
/// `crate::text::protect::accessibility_always_granted` beside it, and this
/// function is the single predicate both the drawing and
/// [`Standing::initial_ticks`] consult.
///
/// ★ It takes the whole [`PermissionBit`] and matches exhaustively rather than
/// comparing against one variant, so a future engine rule that pins a second
/// bit is a change in one place and a compile error if the enum grows.
#[must_use]
pub const fn always_granted(bit: PermissionBit) -> bool {
    match bit {
        PermissionBit::AccessibilityExtract => true,
        PermissionBit::Print
        | PermissionBit::PrintHighQuality
        | PermissionBit::ModifyContents
        | PermissionBit::Copy
        | PermissionBit::Annotate
        | PermissionBit::FillForms
        | PermissionBit::Assemble => false,
    }
}

/// Why this surface can offer nothing about this document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The document carries at least one digital signature. **O119's second
    /// disclosure**, and the engine refuses it by name.
    Signed {
        /// How many, because one approval signature and a certification plus
        /// four approvals are different problems.
        signatures: usize,
    },
    /// **Permissions…** on a document with no `/Encrypt` dictionary. There are
    /// no permissions to change: a PDF states what it allows only as part of
    /// being encrypted.
    NotEncrypted,
    /// The document has never been written to disk, so there is no file to
    /// re-open with the owner password.
    NoFile,
}

// ---------------------------------------------------------------------------
// The engine's refusals, restated as something a sentence can be written about
// ---------------------------------------------------------------------------

/// `pdfcer_core::edit::EncryptError`, flattened to something this crate owns.
///
/// ★ A private mirror rather than the engine's own type, for one reason:
/// `EncryptError` is `#[non_exhaustive]`, is not `Clone`, and carries an
/// `io`-shaped `WriteError` that cannot sit in a dialog's state across frames.
/// The mirror is `Clone`, is exhaustively matched by
/// [`crate::text::protect::engine_refusal`], and turns a new engine variant into
/// a **compile error here** rather than a silent fall-through to a catch-all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineRefusal {
    /// A password was offered to a document that already has one.
    AlreadyEncrypted,
    /// A change was offered to a document that has no protection.
    NotEncrypted,
    /// The session was not owner-authenticated, and this is which password did
    /// open it.
    NotOwner {
        /// The `AuthKind` that authenticated.
        opened_as: AuthKind,
    },
    /// The document is signed. Reachable here only if the census missed one.
    Signed,
    /// The OS CSPRNG was unreachable. A weaker key is never substituted.
    Rng,
    /// An underlying writer error, already formatted.
    Write(String),
}

impl From<&EncryptError> for EngineRefusal {
    fn from(err: &EncryptError) -> Self {
        match err {
            EncryptError::AlreadyEncrypted => Self::AlreadyEncrypted,
            EncryptError::NotEncrypted => Self::NotEncrypted,
            EncryptError::NotOwner { opened_as } => Self::NotOwner {
                opened_as: *opened_as,
            },
            EncryptError::SignedDocument => Self::Signed,
            EncryptError::Rng(_) => Self::Rng,
            EncryptError::Write(inner) => Self::Write(inner.to_string()),
            // ★ `EncryptError` is `#[non_exhaustive]`, so this arm is required
            // by the compiler and is not dead. It carries the engine's own
            // message rather than inventing one, because a variant this build
            // has never seen is precisely the case where guessing is wrong.
            other => Self::Write(other.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Preparation
// ---------------------------------------------------------------------------

/// The passwords one job needs, as [`Secret`]s.
///
/// ★ `Secret` rather than `String` the moment they leave the text fields, for
/// `crate::dialogs::password`'s reason and its module's rule: the value never
/// enters a trace, a queue or an `Action` unwrapped. Everything traced about a
/// password on this surface is its **length** and whether it is ASCII.
#[derive(Debug, Clone)]
pub struct Passwords {
    /// The owner password that authorises a change to an already-protected
    /// document. Empty for [`Job::SetPassword`], which authorises nothing.
    pub current_owner: Secret,
    /// The new user password. Empty is legal and means *permissions-only*.
    pub user: Secret,
    /// The new owner password.
    pub owner: Secret,
}

/// Why a preparation did not produce bytes.
#[derive(Debug, Clone)]
pub enum PrepareFailure {
    /// The document itself is out of scope — signed, unprotected, or unsaved.
    Refused(Refusal),
    /// The owner password did not open the file. Carries the engine's own
    /// message, because pdfcer distinguishes *wrong password* from *a password
    /// pdfcer cannot normalise*, and flattening the two sends an operator to
    /// re-check a password that was correct.
    Reopen(String),
    /// The file opened, and not as the owner.
    NotOwner {
        /// Which password did open it.
        opened_as: AuthKind,
    },
    /// The engine refused the verb.
    Engine(EngineRefusal),
}

/// Finished bytes, waiting for a destination.
#[derive(Debug)]
pub struct Prepared {
    /// The document, protected (or unprotected), in memory.
    bytes: Vec<u8>,
    /// Which job produced them, for the outcome sentence and the trace.
    job: Job,
}

impl Prepared {
    /// The job these bytes came from.
    #[must_use]
    pub const fn job(&self) -> Job {
        self.job
    }

    /// How many bytes, for the trace.
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    /// **Write them to `target`, atomically.**
    ///
    /// ★★ Temp file, then rename — `crate::redact::PreparedRedaction::write_to`'s
    /// mechanism, taken deliberately. The destination may be the file the
    /// operator has open, and a torn write there leaves them with neither the
    /// protected document nor the one they started with.
    ///
    /// The temporary is removed if the rename fails, for the same reason it is
    /// there: a half-written copy of the operator's drawing sitting beside it
    /// under a name nothing will ever open again is an artefact, not a recovery.
    ///
    /// # Errors
    ///
    /// [`WriteFailure`] — the file system refused.
    pub fn write_to(&self, target: &Path) -> Result<usize, WriteFailure> {
        let temporary = target.with_extension("pdfcer-tmp");
        std::fs::write(&temporary, &self.bytes).map_err(|e| WriteFailure(e.to_string()))?;
        if let Err(err) = std::fs::rename(&temporary, target) {
            let _ = std::fs::remove_file(&temporary);
            return Err(WriteFailure(err.to_string()));
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ `job=` and `bytes=`, and NOTHING about the password — not its
            // value, not even here. `crate::secret`'s rule. A trace file is
            // written to disk and kept; a password in one outlives the session
            // that typed it.
            //
            // `path` is Debug-quoted for `redact-written`'s reason: a Windows
            // path routinely contains a space, and a consumer splitting the
            // line into `key=value` pairs would lose every field after it.
            format!(
                "protect-written path={:?} bytes={} job={}",
                target,
                self.bytes.len(),
                job_token(self.job),
            )
        });
        Ok(self.bytes.len())
    }
}

/// The file system refused, already formatted.
#[derive(Debug, Clone)]
pub struct WriteFailure(pub String);

impl std::fmt::Display for WriteFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// **Run the job and hand back the bytes.**
///
/// The one place any of the three engine verbs is called, and §2 of this
/// module's header is the whole of why the two branches differ.
///
/// # Errors
///
/// [`PrepareFailure`] — the document is out of scope, the owner password did
/// not open the file, it opened as somebody other than the owner, or the engine
/// refused the verb by name.
pub fn prepare(
    doc: &OpenDoc,
    job: Job,
    passwords: &Passwords,
    granted: &[PermissionBit],
    encrypt_metadata: bool,
) -> Result<Prepared, PrepareFailure> {
    let options = doc.settings.save_options();
    let mut settings = EncryptionSettings::new(
        passwords.user.expose().to_vec(),
        passwords.owner.expose().to_vec(),
    );
    settings.permissions = granted.to_vec();
    settings.encrypt_metadata = encrypt_metadata;

    let bytes = match job {
        // ── The plaintext branch. `&self`, no mutation, and the operator's
        //    unsaved edits ride along through `dirty_set()`.
        Job::SetPassword => doc
            .session
            .set_encryption(&settings, &options)
            .map(|(bytes, _report)| bytes)
            .map_err(|e| PrepareFailure::Engine(EngineRefusal::from(&e)))?,

        // ── The encrypted branch. A throwaway session over the FILE, opened
        //    with the owner password — which is also how the owner password is
        //    checked. See §2.
        Job::ChangePassword | Job::SetPermissions | Job::RemovePassword => {
            let mut session = owner_session(&doc.path, &passwords.current_owner)?;
            let result = if job == Job::RemovePassword {
                session.remove_encryption(&options)
            } else {
                session.set_permissions(&settings, &options)
            };
            result
                .map(|(bytes, _report)| bytes)
                .map_err(|e| PrepareFailure::Engine(EngineRefusal::from(&e)))?
        }
    };

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ The LENGTHS of the passwords and whether they are ASCII, never the
        // values — `crate::dialogs::password`'s rule, which exists because those
        // two facts explain a normalisation refusal completely and neither
        // carries the password.
        format!(
            "protect-prepared job={} bytes={} grants={} metadata={} user_chars={} owner_chars={} non_ascii={}",
            job_token(job),
            bytes.len(),
            granted.len(),
            u8::from(encrypt_metadata),
            passwords.user.len(),
            passwords.owner.len(),
            u8::from(settings.has_non_ascii_password()),
        )
    });
    Ok(Prepared { bytes, job })
}

/// **Open the file again, as the owner, into a session nothing else holds.**
///
/// The authentication and the throwaway in one act — see §2. The document is
/// read from disk rather than from the open session because the two mutating
/// verbs take `&mut EditSession` and what they mutate would disarm
/// `save_incremental`'s refusal on the session the operator is still using.
fn owner_session(path: &Path, owner: &Secret) -> Result<EditSession, PrepareFailure> {
    let document = Document::load_with_password(path, Some(owner.expose()))
        .map_err(|e| PrepareFailure::Reopen(e.to_string()))?;
    // ★ Checked HERE as well as by the engine, and the duplication is
    // deliberate: the engine's `NotOwner` carries the `AuthKind` and so does
    // this, but reaching the engine's version means having already built a
    // fresh `EditSession` over a whole document for an answer that was
    // available the moment the file opened. More importantly it keeps the two
    // messages one message — the operator is told which password worked,
    // whichever of the two guards noticed.
    match document.encryption() {
        None => {
            // The file on disk is not encrypted although the open session says
            // it is. Something changed the file underneath us; the engine would
            // refuse this with `NotEncrypted` and there is no reason to build a
            // session to hear it.
            Err(PrepareFailure::Engine(EngineRefusal::NotEncrypted))
        }
        Some(enc) if enc.auth != AuthKind::Owner => Err(PrepareFailure::NotOwner {
            opened_as: enc.auth,
        }),
        Some(_) => Ok(EditSession::new(document)),
    }
}

/// The single-token name of a job, for a trace line.
///
/// ★ Free and `const`, so the trace and any driven check that reads it agree by
/// construction rather than by two spellings that happen to match today.
#[must_use]
pub const fn job_token(job: Job) -> &'static str {
    match job {
        Job::SetPassword => "set-password", // ui-text-exempt: trace token, never displayed
        Job::ChangePassword => "change-password", // ui-text-exempt: trace token, never displayed
        Job::RemovePassword => "remove-password", // ui-text-exempt: trace token, never displayed
        Job::SetPermissions => "set-permissions", // ui-text-exempt: trace token, never displayed
    }
}

/// **The name to suggest in the save picker — never the source file.**
///
/// The standing rule for every write that produces a second document, and the
/// suffix depends on the job because the two files it can produce are opposites:
/// a protected one and an unprotected one. Suggesting `-protected` for a removal
/// would name the file after the thing it no longer is.
#[must_use]
pub fn suggested_path(source: &Path, job: Job) -> PathBuf {
    let suffix = if job == Job::RemovePassword {
        crate::text::protect::suggested_suffix_unprotected()
    } else {
        crate::text::protect::suggested_suffix()
    };
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy. Every sibling suggestion function makes the same one.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let named = format!("{stem}{suffix}.pdf");
    source
        .parent()
        .map_or_else(|| PathBuf::from(&named), |parent| parent.join(&named))
}

#[cfg(test)]
mod tests;
