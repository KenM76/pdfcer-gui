//! # `sign` — putting the operator's own digital signature on a document
//!
//! The answer to the request this shell filed on **2026-09-03**: *"a document
//! cannot be signed."* `pdfcer-core` answered it on 2026-09-05 with
//! `pdfcer_core::sign` — 101 public items across `Pass 10.7` (PKCS#12 identity
//! loading), `10.8` (the CAdES `SignedData`, PAdES B-B) and `10.9`
//! (`EditSession::sign`) — whose own module header says it is *"the family the
//! `pdfcer-gui` request of 2026-09-03 asked for."*
//!
//! The window is [`crate::dialogs::sign`]; this module is everything that can
//! be decided **without a `Ui`** — what the document says about itself, what
//! may be offered, which identity is loaded, and the atomic write at the end.
//! The split is [`crate::protect`]'s and [`crate::redact`]'s, taken rather than
//! re-argued: every rule on this surface is a rule about **the operator's
//! file**, and a rule that can only be exercised by driving a window is a rule
//! that gets asserted once, by hand, and then drifts.
//!
//! ---
//!
//! # 1. ★★★ THE SUBSYSTEM WAS SHIPPED, ANSWERED OUR OWN REQUEST, AND WAS NOT
//! IN THE BINARY FOR THREE DAYS
//!
//! This module exists because of a defect worth stating before any of its
//! design, since the defect is the more transferable half.
//!
//! `crates/pdfcer-gui/Cargo.toml` took `pdfcer-core` with
//! `default-features = false` and forwarded `jpx` and `ocrs`. The engine's
//! `signing` feature is **default on**; a `default-features = false` dependency
//! that does not re-name it strips it. So `pdfcer_core::sign` did not exist in
//! this build at all. **Nothing failed to compile. No test went red.**
//!
//! ★★ That is the JPX incident, repeating, three days after the warning about
//! it was written into the very manifest that repeated it — the comment at
//! `Cargo.toml`'s feature block records the day the GUI silently lost JPEG 2000
//! decoding to the identical omission and says, in as many words, *"forgetting
//! to forward does not fail to compile."*
//!
//! ⇒ **A warning does not protect a code path written after it.** The
//! mechanism that does is `tools/gates/check-forwarded-features.sh`, which
//! reads the engine's own `default = [...]` and fails the build when a name in
//! it is neither forwarded nor listed with a reason.
//!
//! ★ And it went unnoticed for two of those days because
//! `tools/gates/check-verb-coverage.sh` scored `EditSession::sign` as
//! **consumed** on the bare word `sign` appearing in `app/actions/bookmarks.rs`
//! — in a documentation table about the arithmetic **sign of `/Count`**. That
//! gate now matches call shape. Two independent instruments, both green, both
//! about nothing.
//!
//! ---
//!
//! # 2. The engine's verbs, verified against the source at the pinned revision
//!
//! `D:\Dev\pdfcer\crates\pdfcer-core\src\`, at the revision `Cargo.lock` pins
//! (`pdfcer-core 0.41.0`, `f9bc7c8`). Checked at that commit rather than at
//! `main`, on [`crate::protect`]'s standing rule: what compiles here is the
//! pin, and a sentence about the engine has a shelf life measured in hours.
//!
//! ```text
//! sign::pkcs12::Pkcs12Signer::from_der(&[u8], &str) -> Result<Pkcs12Signer, Pkcs12Error>
//! Pkcs12Signer::report()                            -> &Pkcs12Report
//! EditSession::sign(&dyn Signer, &SignRequest, &SaveOptions)
//!                                                   -> Result<(Vec<u8>, SignReport), SignApplyError>
//! ```
//!
//! ## ★★ What the engine does that this module must NOT duplicate
//!
//! `EditSession::sign` **self-verifies**: step 5 of its own documentation
//! re-parses the bytes it is about to return and runs `signature_verify` over
//! them, and *"anything but `Integrity::Verified` with full coverage is a
//! refusal, and no bytes are returned. pdfcer does not hand out a signature it
//! cannot itself verify."*
//!
//! So there is no verification step in [`prepare`], and adding one would be a
//! second derivation of one fact — which is how two surfaces come to disagree.
//! What this module does instead is **state** it: [`Prepared::self_verified`]
//! carries the engine's own `SignReport::self_verified`, which that struct's
//! documentation says is *"always `true` on `Ok`; present so the fact is
//! stated, not assumed."*
//!
//! ⚠ That is emphatically **not** the same as this project's R1 bar. A field
//! saying the engine checked its own output is still the engine's word inside
//! one process. The independent read is `tools/ui-verify`'s
//! `a_document_can_be_signed_and_the_signature_is_in_the_file`, which reopens
//! the written file **in a fresh process** and reads it through the Signatures
//! panel — the verification side that shipped as `Pass 10.5` — so the oracle is
//! a different subsystem from the one under test.
//!
//! ---
//!
//! # 3. ★★★ Why the session is handed to the verb, and why the file is reopened
//! afterwards
//!
//! The opposite of [`crate::protect`]'s answer, and the asymmetry is the
//! engine's rather than a preference here.
//!
//! `EditSession::sign` takes **`&mut self`** and stages a real, undoable
//! `CommandKind::AddSignatureField` — the signature field, its widget and the
//! signature dictionary with its zero-filled `/Contents` hole — then serialises
//! the session as an **incremental update**. It must be the open session: the
//! operator's unsaved edits are in it, and a signature that did not cover them
//! would cover a document they are not looking at.
//!
//! ★★ But the session is **left holding the placeholder**, not the signature.
//! The engine says so outright: *"the session still holds the staged
//! placeholder objects (zeros in `/Contents`) … a caller that wants to keep
//! editing must re-open the returned bytes. A CLI writes the bytes and is done;
//! a GUI reloads."*
//!
//! ⇒ So after a successful write this surface offers, and does not perform,
//! **[`crate::dialogs::sign`]'s *Open the signed document*** — a second
//! document tab on the file that was just written. Offering rather than
//! performing, because replacing the operator's open document out from under
//! them would discard an undo history they can still see; and offered rather
//! than omitted, because the open document is now in a state whose only honest
//! description is *"this is not the file you signed"*, and an operator left to
//! discover that by pressing `Ctrl+S` would append a second revision on top of
//! a stale base.
//!
//! ⚠ **What is deliberately NOT done: no `Ctrl+Z` is pushed and no state is
//! rewound.** Undoing the staged command in the old session is, in the
//! engine's own words, *"harmless and pointless"*. Doing it would look like
//! tidying up and would put a spurious entry on the operator's undo stack for
//! an act that produced a file.
//!
//! ---
//!
//! # 4. The five refusals, all of which are STATED rather than discovered
//!
//! R9: *the control is absent or explained, never a button that fails on
//! press.* Whether **this** document can be signed is not knowable when the
//! command registry is built, so `file.sign` stays on the ribbon and the window
//! opens and says why instead of drawing a form whose only possible outcome is
//! a failure. [`Standing::refusal`] is the whole of that decision, as a pure
//! function.
//!
//! | [`Refusal`] | engine variant | why this shell can reach it |
//! |---|---|---|
//! | [`Refusal::Encrypted`] | `SignApplyError::Encrypted` | File ▸ Security ▸ Encrypt… ships (`O119`), so this shell can *make* an encrypted document and then be asked to sign it |
//! | [`Refusal::RedactionPending`] | `SignApplyError::RedactionPending` | deferred redaction ships (`Pass 250.2`), and a pending removal is a normal mid-session state |
//! | [`Refusal::CertificationForbids`] | `SignApplyError::CertificationForbids { permission: 1 }` | a certified document opened from disk |
//! | [`Refusal::RecoveredBase`] | `SignApplyError::RecoveredBase` | a damaged file that loaded through cross-reference recovery |
//! | [`Refusal::NotOnDisk`] | *(none — see below)* | File ▸ New makes a document that has never been written |
//!
//! ★★ The first two are the ones the build brief names, and they are the two
//! this shell can produce **in one session without leaving the application**,
//! which is what makes them reachable rather than theoretical.
//!
//! ★★★ The fifth has **no engine counterpart**, and that is the interesting
//! one. `EditSession::sign` would not refuse a document that was never on
//! disk — it would sign it, incrementally, over whatever base the session
//! holds. The refusal is this shell's, and the reason is that an incremental
//! update is *an appendix to a specific file*: signing a document that has
//! never been saved produces bytes whose base revision exists nowhere, so the
//! operator's next ordinary Save would write a different file that the
//! signature does not describe. Saying *"save it first"* is one sentence; the
//! alternative is a signed file with no ancestor.
//!
//! ---
//!
//! # 5. ★★★ THE PASSPHRASE, AND A RULE STRICTER THAN THE ONE BESIDE IT
//!
//! This module handles a **private key**, and that changes the standard from
//! the one [`crate::protect`] works to.
//!
//! * The passphrase becomes a [`Secret`] the instant it leaves the text field,
//!   for `crate::secret`'s reason: `Action` derives `Debug`, this crate traces
//!   to stderr under `PDFCER_DIAG`, and **`tools/ui-verify` captures that
//!   stderr to a file it keeps as evidence**. A single `{:?}` on the path would
//!   write it to disk in plain text.
//! * ⚠ **No trace line here carries the passphrase's LENGTH either**, and that
//!   is a deliberate departure from `protect::prepare`, which traces
//!   `user_chars=` and `owner_chars=`. There, the length plus "is it ASCII" is
//!   what explains a SASLprep normalisation refusal completely, and a document
//!   password is the operator's own gate on their own file. Here the secret
//!   guards a **private key** — the material an impersonation needs — and a
//!   length is a search-space reduction written into a file that outlives the
//!   session. `passphrase=set|empty` answers the only diagnostic question
//!   ("was one supplied at all?") and reduces nothing.
//! * [`Identity`] holds the engine's `Pkcs12Signer`, whose own `Debug` impl
//!   prints the report and never the key. This type does not derive `Debug` at
//!   all; see its note.
//! * Nothing writes the `.pfx` path into any persisted preference. A path is
//!   not key material, but a file picker that remembered where the operator
//!   keeps their identity would be a durable pointer at it, written by a
//!   convenience nobody asked for.
//!
//! ---
//!
//! # 6. Where the bytes go — [`crate::redact`]'s shape, part for part
//!
//! Settled by the operator hours before the redaction work started, in his own
//! words: *"why does it have to save to a new file right away? Why can't it
//! just wait on saving until I choose to save over the existing file or save as
//! a new file?"* So, unchanged here:
//!
//! 1. **A new file is the default**, and [`suggested_path`] never proposes the
//!    source. A safe default is a mechanism; a warning is something to click
//!    past.
//! 2. **Replacing the original is offered**, behind one extra acknowledgement
//!    that names the file, and it takes **no picker** — a picker pre-filled
//!    with the source is the shape of every accidental overwrite there has ever
//!    been.
//! 3. **The write is atomic** — temp file, then rename ([`Prepared::write_to`]).
//!
//! ★ Replacing is genuinely reasonable here in a way it is not for a redaction:
//! a signature is an *incremental update*, so the replaced file still contains
//! every byte it had. Nothing is lost by signing in place. It is still not the
//! default, because "the file I sent out" and "the file I signed" being one
//! keystroke apart is worth one deliberate act.

use std::path::{Path, PathBuf};

use pdfcer_core::edit::EditSession;
use pdfcer_core::page_tree::{Page, Rect};
use pdfcer_core::sign::apply::{SignApplyError, SignReport, SignRequest};
use pdfcer_core::sign::pkcs12::{Pkcs12Error, Pkcs12Report, Pkcs12Signer};
use pdfcer_core::writer::SaveOptions;

use crate::secret::Secret;

// ---------------------------------------------------------------------------
// What the document says today
// ---------------------------------------------------------------------------

/// **The document's signing situation, read before anything is offered.**
///
/// [`crate::protect::Standing`]'s twin, and it exists for that type's stated
/// reason applied to this surface: a form that opened offering to sign a
/// document the engine will refuse has told the operator a falsehood before
/// they touched anything.
///
/// Read **once**, when the window opens, and never re-read per frame. The
/// sentence under the heading must describe the document the operator chose to
/// act on; a value re-read every frame could change out from under the choices
/// seeded from it.
#[derive(Debug, Clone)]
pub struct Standing {
    /// Whether the document carries an `/Encrypt` dictionary.
    pub encrypted: bool,
    /// Whether a deferred redaction is staged and not yet applied or cancelled.
    pub redaction_pending: bool,
    /// Whether the base loaded through cross-reference recovery, which makes
    /// an incremental update impossible (engine decision 013).
    pub recovered: bool,
    /// How many signatures the document already carries. Not a refusal —
    /// PDF allows many — but the operator should be told they are adding to a
    /// set rather than starting one.
    pub prior_signatures: usize,
    /// The `/DocMDP` `/P` of a certification signature, if there is one.
    ///
    /// **`None` means "no certification signature", not "no `/P`"** — the
    /// engine's census documents that trap, and `/P` absent on a present
    /// certification reports `Some(2)`, because Table 254's default is
    /// permissive.
    pub certification_permission: Option<u8>,
    /// How many pages, for the visible-signature page chooser.
    pub pages: usize,
    /// Whether [`OpenDoc::path`] names a file that exists.
    ///
    /// Asked of the **file system** rather than carried as a flag, exactly as
    /// `crate::app::save::has_a_file` asks it: a second source of truth drifts,
    /// and the failure when it does is writing over the wrong file.
    pub on_disk: bool,
}

impl Standing {
    /// Read it off the open document.
    ///
    /// `pages` is passed rather than re-derived, because `OpenDoc::pages` is
    /// *"the flattened page vector, resolved once at open"* and re-walking the
    /// tree here would be a second answer to a question the document already
    /// has one answer to.
    #[must_use]
    pub fn read(session: &EditSession, path: &Path, pages: usize) -> Self {
        let base = session.document();
        let census = session.signature_census();
        Self {
            encrypted: base.encryption().is_some(),
            redaction_pending: session.has_pending_redaction(),
            recovered: base.loaded_via_recovery(),
            prior_signatures: census.signatures,
            certification_permission: census.certification_permission,
            pages,
            on_disk: path.is_file(),
        }
    }

    /// **Whether this surface may offer anything at all, and if not, why.**
    ///
    /// Pure, so every arm is asserted headlessly rather than by driving a
    /// window. See §4 of this module's header for the table and for the one
    /// refusal that is this shell's rather than the engine's.
    ///
    /// # Order
    ///
    /// The order is *how early the operator can act on it*, not severity.
    /// A pending redaction is one press away from being applied or cancelled,
    /// so it is named first even though encryption is the harder wall: telling
    /// somebody about the wall when the gate beside it is merely latched wastes
    /// the one sentence they will read.
    #[must_use]
    pub fn refusal(&self) -> Option<Refusal> {
        if self.redaction_pending {
            return Some(Refusal::RedactionPending);
        }
        if self.encrypted {
            return Some(Refusal::Encrypted);
        }
        if self.certification_permission == Some(1) {
            return Some(Refusal::CertificationForbids { permission: 1 });
        }
        if self.recovered {
            return Some(Refusal::RecoveredBase);
        }
        if !self.on_disk {
            return Some(Refusal::NotOnDisk);
        }
        None
    }
}

/// Why this document cannot be signed at all.
///
/// A closed set with one sentence each in [`crate::text::sign`]. Every variant
/// but [`Self::NotOnDisk`] mirrors a `SignApplyError` the engine would raise;
/// stating them here means the operator meets the refusal **instead of** a
/// form, rather than after filling one in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// A deferred redaction is staged. Apply or cancel it first.
    RedactionPending,
    /// The document is encrypted, and pdfcer's incremental writer cannot
    /// append to an encrypted base.
    Encrypted,
    /// A certification signature's `/DocMDP` `/P` forbids adding another.
    CertificationForbids {
        /// The `/P` value. Only `1` reaches here — Table 254's `2` is exactly
        /// the permission that allows signing.
        permission: u8,
    },
    /// The base loaded through cross-reference recovery; nothing can be
    /// appended to it.
    RecoveredBase,
    /// The document has never been written to disk. See §4.
    NotOnDisk,
}

// ---------------------------------------------------------------------------
// The identity
// ---------------------------------------------------------------------------

/// **A loaded signing identity — a private key and its certificate chain.**
///
/// ⚠ **No `Debug`, derived or hand-written, and that is the point.** The
/// engine's [`Pkcs12Signer`] has a careful hand-written one that prints the
/// report and never the key, so a derive here would in fact be safe today. It
/// is still absent, because the thing that protects `crate::secret::Secret` is
/// that *the value cannot be formatted* — a property that survives somebody
/// adding a field. A `Debug` on the container is one refactor away from
/// printing whatever is put next to the signer.
///
/// Use [`Self::report`] for anything a human or a trace needs.
pub struct Identity {
    signer: Pkcs12Signer,
    /// Where it came from, for the window's summary line. A path, never a
    /// preference — see §5.
    source: PathBuf,
}

impl Identity {
    /// **Open a `.pfx`/`.p12` with `passphrase`.**
    ///
    /// The file is read here rather than by the caller so that the bytes have
    /// exactly one owner and one lifetime: they hold an encrypted private key,
    /// and a `Vec<u8>` of them passed around the dialog is a copy nobody is
    /// tracking.
    ///
    /// ★ A read failure and a parse failure are **different** answers, because
    /// they send the operator to different places — one to the file picker, one
    /// to the passphrase field. Flattening them into "could not open the
    /// certificate" is the shape of an afternoon spent retyping a correct
    /// passphrase.
    ///
    /// # Errors
    ///
    /// [`IdentityFailure`]: the file system refused, or the container did.
    pub fn open(path: &Path, passphrase: &Secret) -> Result<Self, IdentityFailure> {
        let bytes = std::fs::read(path).map_err(|e| IdentityFailure::Unreadable(e.to_string()))?;
        let signer = Pkcs12Signer::from_der(&bytes, passphrase.expose_str())
            .map_err(IdentityFailure::Import)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★★★ WHAT IS ABSENT HERE IS THE DESIGN. No passphrase, no length
            // of one, no path, no serial, no key bytes. §5 of this module's
            // header argues the length; the PATH is left out because a trace
            // file is kept as evidence and a durable pointer at where somebody
            // stores their digital ID is not ours to publish.
            //
            // What IS here is what a diagnosis needs: the container verified,
            // what protected it, what kind of key came out, and how long the
            // chain is. `subject` is deliberately NOT traced — it is the
            // operator's own name.
            let r = signer.report();
            format!(
                "sign-identity mac={} key={} chain={} unrelated={} scheme={}",
                r.mac.as_deref().unwrap_or("none"),
                r.key,
                r.chain_length,
                r.unrelated_certificates,
                r.key_scheme,
            )
        });
        Ok(Self {
            signer,
            source: path.to_path_buf(),
        })
    }

    /// What the container was made of — the engine's rule-4 disclosure.
    #[must_use]
    pub fn report(&self) -> &Pkcs12Report {
        self.signer.report()
    }

    /// The file it was loaded from.
    #[must_use]
    pub fn source(&self) -> &Path {
        &self.source
    }
}

/// Why an identity could not be loaded.
#[derive(Debug, Clone)]
pub enum IdentityFailure {
    /// The file system refused, already formatted.
    Unreadable(String),
    /// The container refused, by name. Every [`Pkcs12Error`] variant is a
    /// refusal that names its own cause; the window prints it verbatim rather
    /// than re-wording it, because the engine distinguishes *wrong passphrase*
    /// from *a scheme pdfcer does not implement* and flattening the two sends
    /// an operator to re-type a passphrase that was correct.
    Import(Pkcs12Error),
}

// ---------------------------------------------------------------------------
// Placement
// ---------------------------------------------------------------------------

/// **Whether the signature is drawn on a page, and where.**
///
/// ★★★ THE DEFAULT IS INVISIBLE, AND IT IS A DECISION RATHER THAN A COPY OF
/// THE ENGINE'S.
///
/// `SignRequest::visible`'s own documentation says invisible *"is the default
/// for batch/CLI signing"*, which is an argument about batches. The argument
/// here is about what would be drawn: the engine states that a visible
/// signature's appearance is, in this first cut, *"a thin frame only — no
/// text"*.
///
/// A thin empty rectangle stamped on one of this operator's CAD sheets is
/// **indistinguishable from a defect**. He would read it as a stray annotation,
/// and it would be applied content — R8b Rule 4: a signature renders exactly as
/// saved content will, and there is nothing provisional about it. So the
/// default draws nothing, the visible option is offered, and the copy on the
/// control says in plain words what the frame will and will not contain. An
/// operator who wants the box gets the box and is not surprised by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// `/Rect [0 0 0 0]` on the first page; nothing drawn. The default.
    Invisible,
    /// A widget on `page` (0-based), at the default box — see [`default_rect`].
    Visible {
        /// 0-based page index.
        page: usize,
    },
}

/// The visible signature's box, in PDF points: **180 × 60, inset 36 pt from
/// the page's bottom-right corner.**
///
/// ★★ Every number here is stated rather than tuned, because this is content
/// written into the operator's file and *"about a third of the way up"* is not
/// a specification anyone can check. 36 pt is a half-inch margin — the same
/// inset a title block leaves and the value ISO 32000-1's own examples use;
/// 180 × 60 is the box Acrobat's own signature appearance defaults to at 100 %,
/// which is the size an operator's eye already expects.
///
/// ★ Bottom-**right** rather than bottom-left because a CAD sheet's title block
/// is bottom-right and a signature belongs beside it — and because the
/// alternative, bottom-left, is where every drawing frame in this operator's
/// own files puts its revision table.
///
/// ⚠ It is clamped to the page: on a page smaller than 252 × 132 pt the box
/// would otherwise be placed partly or wholly outside the media box, which the
/// engine would accept and no reader would draw. The clamp is
/// [`Self`]-contained arithmetic on `media` rather than a refusal, because a
/// small page is not an error and a signature on it is still wanted.
#[must_use]
pub fn default_rect(media: Rect) -> Rect {
    /// The box, in PDF points.
    const W: f64 = 180.0;
    /// See [`W`].
    const H: f64 = 60.0;
    /// The inset from the page edge, in PDF points. Half an inch.
    const INSET: f64 = 36.0;

    let page_w = (media.urx - media.llx).abs();
    let page_h = (media.ury - media.lly).abs();
    let w = W.min(page_w);
    let h = H.min(page_h);
    // The inset shrinks rather than pushing the box off the page when there is
    // not room for both it and the box.
    let inset_x = INSET.min((page_w - w).max(0.0) / 2.0);
    let inset_y = INSET.min((page_h - h).max(0.0) / 2.0);
    let llx = media.llx.min(media.urx);
    let lly = media.lly.min(media.ury);
    let right = llx + page_w - inset_x;
    let bottom = lly + inset_y;
    Rect {
        llx: right - w,
        lly: bottom,
        urx: right,
        ury: bottom + h,
    }
}

// ---------------------------------------------------------------------------
// Preparation
// ---------------------------------------------------------------------------

/// **What the operator authored, ready for the engine.**
///
/// Everything on this struct is the operator's own words or the operator's own
/// choice. pdfcer infers nothing into a signature dictionary — the engine's
/// rule-4 note says the reason: *"the signing time, name, reason, location and
/// contact are the caller's words, written verbatim."*
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authored {
    /// `/Reason`, free text. Empty means the key is omitted.
    pub reason: String,
    /// `/Location`, free text — the operator's words, not a resolved place.
    /// Empty means the key is omitted.
    pub location: String,
    /// Where the widget goes.
    pub placement: Placement,
    /// `/M`, the claimed signing time as a PDF date string. Captured when the
    /// window opened and **shown on screen**, so what is written is the time
    /// the operator was told — the engine reads no clock and says a GUI should
    /// pass *"the time it showed the operator."*
    pub signing_time: String,
}

/// Why a signing did not produce bytes.
#[derive(Debug)]
pub enum PrepareFailure {
    /// The document itself is out of scope. Reachable from [`prepare`] as well
    /// as from [`Standing::refusal`] because the two are asked at different
    /// moments and the document can change between them.
    Refused(Refusal),
    /// The engine refused, by name.
    Engine(SignApplyError),
}

/// **Finished bytes, waiting for a destination.**
pub struct Prepared {
    /// The signed document, in memory.
    bytes: Vec<u8>,
    /// What the engine says it wrote.
    report: SignReport,
}

impl Prepared {
    /// The engine's own account of the signature it wrote.
    #[must_use]
    pub const fn report(&self) -> &SignReport {
        &self.report
    }

    /// How many bytes, for the trace and the outcome sentence.
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    /// Whether the engine re-read its own output and found the signature
    /// intact before returning it. See §2 — always `true` on `Ok`, carried so
    /// the fact is stated rather than assumed, and **not** a substitute for an
    /// independent read.
    #[must_use]
    pub const fn self_verified(&self) -> bool {
        self.report.self_verified
    }

    /// **Write them to `target`, atomically.**
    ///
    /// Temp file, then rename — `crate::protect::Prepared::write_to`'s
    /// mechanism, taken deliberately. The destination may be the file the
    /// operator has open, and a torn write there leaves them with neither the
    /// signed document nor the one they started with.
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
            // ★ `path` is Debug-quoted for `protect-written`'s reason: a
            // Windows path routinely contains a space, and a consumer splitting
            // the line into `key=value` pairs would lose every field after it.
            //
            // ⚠ `subject` and `serial` are NOT here. They are on screen, in the
            // report the operator reads, which is where a disclosure about
            // whose key was used belongs. A trace file is kept and shared.
            format!(
                "sign-written path={:?} bytes={} field={} prior={} self_verified={}",
                target,
                self.bytes.len(),
                self.report.field_name,
                self.report.prior_signatures,
                u8::from(self.report.self_verified),
            )
        });
        Ok(self.bytes.len())
    }
}

/// **What became of one signing, as the dialog needs to hear it.**
///
/// The handler produces this and hands it back through
/// [`crate::dialogs::DialogsState::sign_outcome`]. A single type with two
/// variants rather than a `Result`, because the *failure* side here is already
/// a finished operator-facing sentence — every producer of one has more context
/// than the dialog does about which of five things went wrong — and a `Result`
/// whose error is a `String` invites a caller to add its own wording on top,
/// which is how one event comes to be described twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The bytes reached a path.
    Written {
        /// Where they went.
        path: PathBuf,
        /// Whether that path was the document the operator has open.
        replaced: bool,
        /// The rule-4 disclosure: what the engine says it wrote, already
        /// worded. See [`crate::text::sign::written_details`].
        details: String,
    },
    /// Nothing was written, and this is why — already an operator-facing
    /// sentence.
    Failed(String),
}

/// The file system refused, already formatted.
#[derive(Debug, Clone)]
pub struct WriteFailure(pub String);

impl std::fmt::Display for WriteFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// **Sign the document and hand back the bytes.**
///
/// The one place `EditSession::sign` is called. See §3 for why it is the open
/// session and not a throwaway, and why nothing is undone afterwards.
///
/// # ★ The reservation is the engine's default and is not offered as a control
///
/// `SignRequest::reserve` defaults to 12 KiB, which the engine's own note says
/// *"fits a SHA-256/RSA-4096 CAdES signature with a three-certificate chain
/// about three times over"*. It is not a question an operator can answer — the
/// number is a property of their certificate chain, which pdfcer has just read
/// and they have not — so asking would be handing them arithmetic. If it is
/// ever too small the engine refuses by name with **both** numbers, and
/// [`crate::text::sign::reservation_too_small`] states that the reservation is
/// fixed, so the refusal is a fact rather than an instruction the operator
/// cannot follow.
///
/// ⚠ The alternative considered and rejected was **retrying automatically at a
/// larger reserve**. It would work, and it would mean the size of the hole in
/// the operator's file depended on a retry they were never told about. R8b
/// Rule 4: what is written is disclosed.
///
/// # Errors
///
/// [`PrepareFailure`] — the document is out of scope, or the engine refused.
pub fn prepare(
    session: &mut EditSession,
    pages: &[Page],
    identity: &Identity,
    authored: &Authored,
    options: &SaveOptions,
) -> Result<Prepared, PrepareFailure> {
    let mut request = SignRequest::at(authored.signing_time.clone());
    // Empty is omitted rather than written as an empty string: `/Reason ()` in
    // a signature dictionary is a claim that the operator gave a reason and it
    // was nothing, which is not what an untouched field means.
    request.reason = non_empty(&authored.reason);
    request.location = non_empty(&authored.location);
    // ★ `/Name` is deliberately NEVER set — see `crate::text::sign`'s header.
    // The engine: "`None` omits the key and a verifier falls back to the
    // certificate subject (Table 252 says it should anyway)." A free-text name
    // beside a certificate is a second, unverifiable claim about who signed.
    request.visible = match authored.placement {
        Placement::Invisible => None,
        Placement::Visible { page } => {
            // ★ The CROP box, not the media box, and the difference is what
            // the operator sees. Content is clipped to `/CropBox` at display
            // time (Table 30), so a box placed against a larger `/MediaBox` on
            // a trimmed sheet would be laid partly or wholly outside the
            // visible page — present in the file, and invisible in every
            // reader. `Page::crop_box` defaults to `media_box`, so the two
            // agree on every document that does not trim.
            //
            // A page index the vector does not hold cannot arise from the
            // chooser, which is built from this same vector; US Letter is the
            // fallback rather than a panic, on this project's standing
            // preference against panicking on a branch a guard excluded.
            let crop = pages
                .get(page)
                .map_or(Rect::from_corners(0.0, 0.0, 612.0, 792.0), |p| p.crop_box);
            Some((page, default_rect(crop)))
        }
    };

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "sign-requested visible={} page={} reason={} location={} time_len={}",
            u8::from(request.visible.is_some()),
            request.visible.map_or(usize::MAX, |(p, _)| p),
            u8::from(request.reason.is_some()),
            u8::from(request.location.is_some()),
            authored.signing_time.len(),
        )
    });

    let (bytes, report) = session
        .sign(identity.signer_ref(), &request, options)
        .map_err(PrepareFailure::Engine)?;

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "sign-prepared bytes={} field={} algorithm={:?} certificates={} cms={} reserved={} \
             prior={} level={} self_verified={}",
            bytes.len(),
            report.field_name,
            report.algorithm,
            report.certificates,
            report.cms_bytes,
            report.reserved_bytes,
            report.prior_signatures,
            report.pades_level,
            u8::from(report.self_verified),
        )
    });
    Ok(Prepared { bytes, report })
}

impl Identity {
    /// The engine's signer, as the trait object `EditSession::sign` takes.
    ///
    /// Private-in-spirit: it is `pub(crate)` rather than `pub` so that the key
    /// operation is reachable from [`prepare`] and from nowhere a future module
    /// might casually put it.
    pub(crate) fn signer_ref(&self) -> &dyn pdfcer_core::sign::Signer {
        &self.signer
    }
}

/// `None` for a field the operator left alone.
///
/// ★ Trims first. A field holding one space is an untouched field as far as
/// anybody looking at the screen is concerned, and writing `/Reason ( )` into a
/// legal document because of a stray keystroke is the kind of thing nobody ever
/// finds.
fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

/// **The name to suggest in the save picker — never the source file.**
///
/// The standing rule for every write that produces a second document.
#[must_use]
pub fn suggested_path(source: &Path) -> PathBuf {
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy. Every sibling suggestion function makes the same one.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let named = format!("{stem}{}.pdf", crate::text::sign::suggested_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&named), |parent| parent.join(&named))
}

#[cfg(test)]
mod tests;
