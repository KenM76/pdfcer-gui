//! # `protect` — putting a password on a document, changing what it allows,
//! and taking the protection off
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/protect/mod.md`.

use std::path::{Path, PathBuf};

use pdfcer_core::crypto::{AuthKind, Cipher, PermissionBit};
use pdfcer_core::document::Document;
use pdfcer_core::edit::{EditSession, EncryptError, EncryptionSettings};

use crate::opendoc::OpenDoc;
use crate::secret::Secret;
use crate::settings::SettingsExt;

// ---------------------------------------------------------------------------
// What the operator asked for
// ---------------------------------------------------------------------------

/// Which ribbon control opened the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    /// **Encrypt…** — the password: set it, change it, or remove it.
    Password,
    /// **Permissions…** — what a protected document says it allows.
    Permissions,
}

/// What the operator has chosen to do, once the document's own state has
/// narrowed the field.
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
    #[must_use]
    pub const fn edits_permissions(self) -> bool {
        matches!(self, Self::SetPassword | Self::SetPermissions)
    }

    /// Whether this job needs the document's **current** owner password.
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
    /// `Option<bool>` all the way to the screen, never flattened. `None` is
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
                // Not encrypted: every bit is GRANTED, and that is a
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
    #[must_use]
    pub fn refusal(&self, task: Task) -> Option<Refusal> {
        // Signed first, and it outranks everything. Both mutating verbs and
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
        // Only the encrypted branch needs a file: it is the branch that
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
    #[must_use]
    pub fn jobs(&self, task: Task) -> Vec<Job> {
        match (task, self.encrypted) {
            (Task::Password, false) => vec![Job::SetPassword],
            (Task::Password, true) => vec![Job::ChangePassword, Job::RemovePassword],
            // On an unprotected document this is empty and the window never
            // gets here — `refusal` has already returned `NotEncrypted`. It is
            // still written as the honest answer rather than as a `panic!`,
            // because a function that returns "the jobs" should return them.
            (Task::Permissions, false) => Vec::new(),
            (Task::Permissions, true) => vec![Job::SetPermissions],
        }
    }

    /// **The permission bits to carry over unchanged**, for a job that must not
    /// alter them.
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
/// A private mirror rather than the engine's own type, for one reason:
/// `EncryptError` is `#[non_exhaustive]`, is not `Clone`, and carries an
/// `io`-shaped `WriteError` that cannot sit in a dialog's state across frames.
/// The mirror is `Clone` and is **exhaustively** matched by
/// [`crate::text::protect::engine_refusal`], so every variant *this* enum has
/// owns a sentence or the build fails.
///
///
/// This comment used to end *"and turns a new engine variant into a **compile
/// error here** rather than a silent fall-through to a catch-all."*
///
/// **That was false the day it was written, and it cannot be made true.**
/// `EncryptError` is `#[non_exhaustive]`, so [`From`] below is *required* to
/// carry a wildcard; the compiler can never object to a variant it has never
/// seen, and the stated mechanism has never been able to fire once. The
/// exhaustiveness is real on the near side of the mirror and imaginary on the
/// far side, and the sentence claimed the far side.
///
/// ⇒ It was caught by `check-engine-api-drift`, not by a reader:
/// `EncryptError::RedactionPending` shipped 2026-09-05 **in answer to this
/// shell's own request**, fell into the wildcard for three days, and delivered
/// the engine's implementer-voiced message — naming `save_applying_redaction`
/// and `cancel_pending_redaction` — into a draughtsman's dialog. The gate held
/// it as a written exemption ending *"DELETE THIS LINE the day the arm is
/// added"*; the arm is added, the line is deleted, and this paragraph is what
/// the exemption was standing in for.
///
/// **The same shape bit twice in one evening, in unrelated code.** Hours
/// earlier, `app::actions::textstyle::reflow_refusal`'s wildcard was about to
/// swallow `ReflowApplyError::PageEditedThisSession` — the engine's reply
/// warned about it by name — and the repair there was the same: route through
/// a discriminant the engine declares **exhaustive on purpose**, so the
/// `match` is compiler-proved and a new refusal joins an existing arm instead
/// of vanishing.
///
/// ⚠ **No such discriminant exists for `EncryptError`.** So the wildcard below
/// stays, and this comment now says what it actually guarantees rather than
/// what would be reassuring: a new engine variant arrives here **silently**,
/// carrying the engine's own words, and the only thing that will notice is
/// `check-engine-api-drift`. That gate is the mechanism. This type is not.
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
    /// A deferred redaction is armed, so changing the protection would write
    /// the un-redacted content.
    ///
    ///
    /// > *"a deferred redaction is pending; encrypting/re-keying/removing
    /// > encryption now would write the un-redacted content -- apply it via
    /// > save_applying_redaction first, or cancel_pending_redaction"*
    ///
    /// ⚠ **Two internal Rust function names, in a draughtsman's dialog**, on a
    /// path reachable in one session: arm a redaction, open File ▸ Security ▸
    /// Encrypt…. [`crate::text::protect::engine_refusal`]'s own doc comment
    /// warned about exactly this — *"a `to_string()` of the engine's own
    /// message would put an implementer's sentence in front of a draughtsman"*
    /// — and the catch-all was the hole it did not cover.
    ///
    /// The Sign surface got this right on the day it shipped
    /// (`crate::text::sign::refusal_redaction_pending`). This one did not,
    /// because Sign matched the variant and Protect never added an arm. Two
    /// surfaces, one engine refusal, and only one of them was updated — which
    /// is why the wording here deliberately mirrors Sign's *"apply it, or call
    /// it off, and then …"* shape rather than inventing a second voice.
    RedactionPending,
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
            EncryptError::RedactionPending => Self::RedactionPending,
            // `EncryptError` is `#[non_exhaustive]`, so this arm is required
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
            // `job=` and `bytes=`, and NOTHING about the password — not its
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
        // The LENGTHS of the passwords and whether they are ASCII, never the
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
fn owner_session(path: &Path, owner: &Secret) -> Result<EditSession, PrepareFailure> {
    let document = Document::load_with_password(path, Some(owner.expose()))
        .map_err(|e| PrepareFailure::Reopen(e.to_string()))?;
    // Checked HERE as well as by the engine, and the duplication is
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
/// Free and `const`, so the trace and any driven check that reads it agree by
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
