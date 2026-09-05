//! # `redact` — the APPLY pipeline, and the reason a redaction in this shell
//! cannot be shipped unverified
//!
//! **Salvaged whole from `D:\Dev\pdfce\crates\pdfce-gui\src\redact_apply.rs`
//! (429 code + 280 test lines, `SALVAGE.md` Class A) on 2026-08-15**, with its
//! proof, its refusal taxonomy and every paragraph of its reasoning intact. The
//! absence proof lives in [`proof`]; the call-site monopoly that makes it
//! unskippable lives in `redact::sealed`.
//!
//! `SALVAGE.md`'s row for that file is the strongest one in the inventory:
//!
//! > **★ This file is currently the ONLY place the proof exists.**
//!
//! and the core team's `Pass 72.0` note says what follows from it:
//!
//! > **A shell calling `redact::apply_redactions` directly and writing the
//! > bytes ships an unverified redaction and will not know.** … Do not build a
//! > redaction UI against core's current surface.
//!
//! Both halves were re-checked against `D:\Dev\pdfcer` on 2026-08-15 rather than
//! quoted: [`pdfcer_core::redact::apply_redactions`] still returns
//! `Result<(Vec<u8>, RedactionReport), RedactError>` — a **report**, not a
//! verdict — and `RedactionVerdict` and `verify_redaction` appear nowhere in
//! `pdfcer-core`. The request channel is empty, so nothing is owed and Pass 72.0
//! has not closed. This module is therefore not a convenience; it is the
//! difference between a redaction that is proven and one that is asserted.
//!
//! ---
//!
//! # 1. The three properties carried across from the source
//!
//! ## ★★★ 1.0 CORRECTED 2026-09-04 (evening) — there are now TWO apply routes,
//! and the second one lands in the open document
//!
//! Everything from §1.1 down was written on the morning of 2026-09-04, when
//! [`pdfcer_core::redact::apply_redactions`] took a `&Document` and returned
//! `Vec<u8>` and there was no way back into an `EditSession`. That is no longer
//! the shape of the world. The engine shipped
//! [`pdfcer_core::edit::EditSession::apply_redactions`] the same afternoon
//! (`Pass 250.1`, `225db51`, `pdfcer-core` v0.35.0), in answer to this shell's
//! own request, and this module now has two entry points rather than one:
//!
//! | route | what it produces | what it touches | who calls it |
//! |---|---|---|---|
//! | [`prepare_redaction_apply`] | the finished redacted **bytes**, in memory, proven | nothing | the dialog on open — the MEASUREMENT — and the two write-now destinations |
//! | [`apply_into_session`] | a mutation of the **session**, proven | the open document | the deferred destination, through `crate::app::actions::redact` |
//!
//! Both are kept, and the reason is not symmetry. The measurement has to exist
//! *before* the confirmation on either path (§2 of [`crate::dialogs::redact`]),
//! and the deferred route is irreversible at the moment it runs — it clears the
//! undo log — so the operator must be reading a real report, of a removal that
//! really ran, before he chooses. The cost is that the removal runs twice on
//! the deferred path: once to measure, once to land. That is a second full
//! rewrite of the document on a deliberate click, and it is bought for the one
//! property that could not be had any other way — that the numbers on screen at
//! the moment of consent are measurements rather than predictions.
//!
//! ★★ **The `to_incremental_bytes` question, which is the whole of §4.1 of our
//! request, is answered by CONSTRUCTION rather than by a refusal, and it was
//! verified rather than believed** — see [`apply_into_session`], which carries
//! the measurement and the test that would catch a regression.
//!
//! ## 1.1 Apply is a FULL REWRITE, or it does not happen
//!
//! The engine's `ARCHITECTURE.md` §5 corollary and standing rule R35: an
//! incremental save **structurally preserves superseded content** — the old
//! bytes of every replaced object stay in the file by construction, in the
//! prior revision. For an ordinary edit that is a feature, and it is exactly
//! what [`crate::app::save`] relies on and promises in `file.save_copy`'s
//! tooltip. For a redaction it is the defeat of the entire operation: the
//! "removed" text would sit in the saved file one `startxref` hop away,
//! trivially recoverable by any parser that walks `/Prev`.
//!
//! So there are exactly two full rewrites in this pipeline and no third path:
//!
//! ```text
//!   EditSession (marks may be UNSAVED)
//!     │
//!     │  (1) EditSession::to_full_bytes   ← full rewrite #1: materialise
//!     ▼                                      this session's edits as ONE
//!   Vec<u8>  (one revision, no /Prev)         revision so apply can see the
//!     │                                       marks the operator just made
//!     │  Document::from_bytes
//!     ▼
//!   Document
//!     │
//!     │  (2) redact::apply_redactions     ← full rewrite #2: core's own
//!     ▼                                      forced full rewrite, which is
//!   Vec<u8>  (redacted, one revision)         where the removal happens
//! ```
//!
//! If **either** rewrite fails, this module returns a refusal and nothing is
//! written. There is deliberately no `to_incremental_bytes` call anywhere in
//! this file, and no fallback that could introduce one: a redaction that
//! silently degraded to an incremental save would produce a file the operator
//! has been told is redacted and which is not.
//!
//! ★ That is worth restating in this shell's terms, because this shell has an
//! incremental writer and the old one's *"there is no parameter anywhere that
//! could make an apply write incrementally"* has to stay true here.
//! [`crate::app::save::save_copy`] is incremental **by a promise printed on a
//! tooltip**; this path is full-rewrite **by the engine's own construction**,
//! and the two share no function, no options value and no code path. They are
//! two writers, deliberately, and neither can inherit the other's default.
//!
//! ## 1.2 Why the session must be materialised first (the un-saved-mark trap)
//!
//! [`pdfcer_core::redact::apply_redactions`] takes a `&Document` — a parsed file
//! — not an [`EditSession`]. The shell's marks, however, may exist only in the
//! session overlay: an operator can open a document, mark three regions and
//! press Apply without ever having saved. Handing `session.document()` (the
//! BASE revision) to `apply_redactions` would therefore apply **zero** of those
//! marks and report success — not a disclosure that stayed silent, but an
//! *apply* that removed nothing while saying it had.
//!
//! `to_full_bytes` is what closes it: it is the session's own edits rendered
//! into a real single-revision file, which `Document::from_bytes` then
//! re-parses into exactly the document the operator is looking at.
//!
//! ## 1.3 Absence is VERIFIED on the actual output bytes, not assumed
//!
//! See [`proof`], which owns the whole of that argument and the table of what
//! each class of survivor means.
//!
//! ---
//!
//! # 2. ★ How the proof is made **unskippable**, rather than merely available
//!
//! The salvage brief's second requirement, and the one that is not satisfied by
//! copying the file across. The old shell's own module docs end with *"Nothing
//! in this module can reach the filesystem"* — true, and it means the proof was
//! enforced by the **caller** remembering to run it. `pdfcer`'s
//! `redact-apply` is the counter-example living in the same repository: it
//! calls `apply_redactions`, writes the bytes and exits `SUCCESS` on a file it
//! never verified.
//!
//! Four mechanisms, in increasing order of how hard they are to defeat. Each
//! one alone would be a convention; together they are a structure.
//!
//! ## 2.1 The bytes are private, and there is no accessor
//!
//! [`PreparedRedaction::bytes`] is a private field of a type whose only
//! constructor is [`prepare_redaction_apply`], which always proves. There is no
//! `pub fn bytes()`, no `Deref`, no `AsRef<[u8]>`, no `IntoIterator`, and
//! [`PreparedRedaction`]'s [`std::fmt::Debug`] impl is **hand-written** so that
//! `{:?}` reports a length rather than emitting the buffer into a log. The only
//! expression in this crate that can obtain the redacted bytes is inside this
//! module. That is a compile-time fact, not a convention:
//! `PreparedRedaction { bytes: … }` does not typecheck outside `redact`, and
//! nor does `prepared.bytes`.
//!
//! ## 2.2 The write is a method on the proof, and it re-proves
//!
//! [`PreparedRedaction::write_to`] is the only way bytes leave this module, and
//! it runs the decoded-stream half of the proof **again**, over the exact
//! buffer it is one statement away from handing to the file system.
//!
//! That is not belt-and-braces about a check that already passed. It is what
//! moves the guarantee from *"the constructor proved it"* to *"the write
//! proves it"* — a distinction that matters the day someone adds a second
//! constructor, a `set_bytes`, a `#[cfg(test)]` builder or a deserialisation
//! path. Any of those would defeat 2.1 silently; none of them defeats this,
//! because the check is between the buffer and the syscall rather than at the
//! far end of the type's history.
//!
//! ## 2.3 The acknowledgement is a parameter, not a convention
//!
//! A disclosed residual (module [`proof`]'s middle row) requires the operator's
//! explicit acknowledgement, and the writer cannot be reached without stating
//! whether it was given: [`ResidualAcknowledgement`] is a required argument, and
//! [`WriteRefusal::ResidualsNotAcknowledged`] is what a `Withheld` produces when
//! there is something to acknowledge. A caller that forgets the checkbox does
//! not write a partially-redacted file believing it is clean; it gets a named
//! refusal.
//!
//! It is an enum rather than a `bool` for [`crate::app::actions`]' stated
//! reason: `write_to(path, true)` at a call site says nothing, and this is the
//! one call site in the program where a transposed boolean is a security
//! defect.
//!
//! ## 2.4 The call-site monopoly is asserted from the syntax tree
//!
//! `redact::sealed` parses **every `.rs` file in this crate** with `syn` and asserts
//! that `apply_redactions` is *called* in exactly one FILE — this one. A call
//! from anywhere else is a test failure naming the file.
//!
//! ★ 2026-09-04: it is *one file*, not *one call*, and since this afternoon
//! that distinction is load-bearing rather than pedantic. There are now two
//! calls in this file — the free function in [`prepare_redaction_apply`] and
//! the `EditSession` method in [`apply_into_session`] — and `sealed` pins the
//! number at exactly two so a third cannot arrive quietly. The reader counts
//! method calls as well as free calls, which it did before either existed,
//! *"because a future engine that moved it onto a type would otherwise slip the
//! monopoly silently"*. That future engine arrived; the check was already
//! pointed at it.
//!
//! It reads the abstract syntax tree rather than the text, for
//! `crate::shell::commands::reach`'s reasons applied to a different question: a
//! doc comment quoting `[`pdfcer_core::redact::apply_redactions`]` is not a call
//! and a grep cannot tell (this very header contains several). And it **fails
//! closed** — a sweep that finds *zero* call sites fails, because "the proof is
//! nowhere" and "the sweep read nothing" print the same thing otherwise, which
//! is `run-all.sh`'s three-state lesson arriving inside a test.
//!
//! ## 2.5 What is deliberately NOT claimed
//!
//! None of this stops a future author writing `pdfcer_core::redact` calls in a
//! *different crate*, and none of it stops a determined edit to this module. It
//! is not a sandbox. What it does is make the unverified path **impossible to
//! reach by accident and impossible to add quietly** — which is the failure
//! mode Pass 72.0 actually describes: a shell that calls the engine directly
//! *and will not know*.
//!
//! ---
//!
//! # 3. What this module deliberately does NOT do
//!
//! It does not implement any part of the removal. The surgery, the carrier
//! sweep, the object-stream decomposition and the forced full rewrite all live
//! in [`pdfcer_core::redact`] and are called, never re-derived — the GUI/core
//! separation rule, plus the plain fact that a second implementation of
//! security-critical byte surgery is how the two quietly diverge.
//!
//! It does not decide **where** the file goes. [`PreparedRedaction::write_to`]
//! takes a path; asking the operator for one is [`crate::dialogs::redact`]'s
//! job. See §4.
//!
//! ★★★ **CORRECTED 2026-09-04 (evening).** This paragraph used to read:
//!
//! > *"It does not mutate the open document. Applying produces a **new file**;
//! > the session keeps its marks, its undo log and its epoch. That is not a
//! > limitation — it is the property that makes an irreversible operation safe
//! > to offer."*
//!
//! It was a description of an engine surface, dressed as a principle. The
//! engine surface changed the same afternoon and the principle did not survive
//! it: [`apply_into_session`] mutates the open document, on purpose, because
//! the operator asked for exactly that and because deferring the write destroys
//! nothing — **the original is on disk, untouched, until he chooses to
//! overwrite it.** What was actually load-bearing in the old sentence is kept
//! and is now §4.
//!
//! ---
//!
//! # 4. ★★★ CORRECTED 2026-09-04 — the rule is *warn at the overwrite*, not
//! *never overwrite*
//!
//! This section used to be headed *"Save-as, never save-over — and why that is
//! the sharpest rule here"*, and it argued from `HANDOFF.md` §3 item 5 (*"Read
//! may produce a new document; it may not modify this one"*) and from
//! [`crate::app::save`] §3.4 that the source file must never be overwritten by
//! a redaction.
//!
//! The operator overruled it on 2026-09-04, and the argument that replaced it is
//! `OPERATOR_REQUESTS.md` O125's:
//!
//! > *"if the engine can't hold it we need to leave it up to the user to decide
//! > to overwrite the original or save to a new file… If someone is saving
//! > their changes while redacting they aren't going to keep having to save a
//! > new file every time."*
//!
//! The premise of the old rule is still true — the source file **is** the only
//! remaining copy of the content being removed — and the conclusion still does
//! not follow from it. Forcing a copy does not protect him from the decision;
//! it makes him perform it in two steps and leaves a stray file behind. What
//! survives, and survives as a *mechanism* rather than as a prohibition:
//!
//! 1. **Nothing in this module can produce a path.** [`PreparedRedaction::write_to`]
//!    takes one; it never invents one. That is unchanged and is the structural
//!    half of the rule.
//! 2. **The suggested name is never the file that was opened**
//!    ([`crate::text::redact::suggested_suffix`], asserted by
//!    [`crate::dialogs::redact`]'s own test, in the shape
//!    `crate::app::save::suggested_path` and `crate::dialogs::ocr::suggested_path`
//!    both established).
//! 3. **The write is atomic** — temp file, then rename — precisely because the
//!    destination may now be the source. See [`PreparedRedaction::write_to`].
//! 4. **The overwrite is warned about at the moment it is chosen**, in words,
//!    at a control the operator had to select. A warning, not a refusal: that
//!    distinction is the whole of O125.

pub mod proof;

/// **The call-site monopoly** — §2.4. Parses every `.rs` file in this crate and
/// asserts that `apply_redactions` is called in exactly one place.
///
/// `#[cfg(test)]` because the reader parses Rust with `syn`, a
/// **dev**-dependency — the same posture `crate::shell::commands::reach` takes
/// for the same reason, and for the same reason nothing here is compiled into
/// `pdfcer-gui.exe`. See this crate's `Cargo.toml` for why a real parser and not
/// a grep, and `sealed`'s own header for what it refuses to claim.
#[cfg(test)]
mod sealed;

use std::path::Path;

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::object::ObjId;
use pdfcer_core::redact::{self, RedactError, RedactionReport};
use pdfcer_core::writer::SaveOptions;

pub use proof::{AbsenceVerification, Residual, ResidualSite};

/// Why a redaction apply did not happen. Every variant is a refusal **before
/// any byte reached the filesystem**.
///
/// There is no `Partial` or `DegradedToIncremental` variant, and adding one
/// would be a defect: the operations this models either complete as a full
/// rewrite or do not occur (§1.1).
///
/// Rendered by [`crate::text::redact::refusal_message`]; the variants carry
/// structured data and diagnostic strings from `pdfcer-core`'s own error
/// `Display`, never operator-facing prose — rule R1, the same split
/// `crate::app::save::SaveError` makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedactApplyRefusal {
    /// The document carries no `/Redact` marks, so there is nothing to apply.
    ///
    /// Reachable only if the marks vanished between the panel enabling its
    /// button and the action running (an undo in the same frame); the panel's
    /// own gate normally prevents it.
    NothingToApply,
    /// The session's edits could not be materialised as a single full revision
    /// — e.g. `WriteError::HybridFullRewrite`, which core refuses by name.
    ///
    /// **The refusal is the correct outcome**: the alternative is an
    /// incremental save that leaves the un-redacted content in a prior
    /// revision.
    FullRewriteUnavailable {
        /// `pdfcer-core`'s own diagnostic for the failed rewrite.
        reason: String,
    },
    /// The full-rewrite bytes could not be re-parsed into a document, so the
    /// apply could not run against them.
    ///
    /// Structurally the same refusal as [`Self::FullRewriteUnavailable`] and
    /// kept distinct only because it names a different suspect: the writer
    /// produced something the parser rejects, which is a pdfcer bug rather than
    /// a property of the operator's file.
    MaterialisedDocumentUnreadable {
        /// The parse diagnostic.
        reason: String,
    },
    /// `pdfcer-core` refused the apply itself: a region over a raster image it
    /// cannot destroy pixels in, an encrypted document, an unparsable page.
    ///
    /// These are the cardinal-rule refusals — core would rather produce nothing
    /// than a false redaction.
    CoreRefused {
        /// [`RedactError`]'s own message, which names the page and the
        /// condition.
        reason: String,
    },
    /// The apply completed in memory, but the absence proof found redacted text
    /// **still present in a decoded stream** of the output. Nothing is written.
    ///
    /// This is the module's own last line of defence and it should be
    /// unreachable: reaching it means core's removal and core's report
    /// disagree. It is a refusal rather than a disclosure because a decoded
    /// stream is content a reader will render or extract — there is no reading
    /// of that survival under which the file is safe to hand over.
    VerificationFailed {
        /// The strings that survived, for the message. Not the whole redacted
        /// set — only what actually leaked.
        survivors: Vec<String>,
    },
}

/// Whether the operator has acknowledged the residuals the report disclosed.
///
/// **An enum rather than a `bool`**, for `crate::app::actions::apply`'s stated
/// reason about `Direction`: `write_to(path, true)` says nothing at a call
/// site, and this is the one call site in the program where reading a
/// transposed boolean the wrong way round writes a partially-redacted file that
/// the operator has been told is clean.
///
/// It is a required argument rather than a field on [`PreparedRedaction`]
/// because it is a fact about the **operator**, not about the bytes. Storing it
/// on the prepared value would let it be set once and then travel with the
/// buffer; passing it makes every write state, at the write, what the person
/// pressing the button knew.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidualAcknowledgement {
    /// The operator has read the residual list and asked to proceed anyway.
    Given,
    /// The operator has not. A write is refused if there is anything to
    /// acknowledge.
    Withheld,
}

/// Why [`PreparedRedaction::write_to`] produced no file.
///
/// Every variant means **nothing was written**. Structured rather than a
/// `String` on `crate::app::lifecycle`'s rule that a branch is made on error
/// *data*, never by inspecting a message.
#[derive(Debug)]
pub enum WriteRefusal {
    /// The report disclosed residuals and the acknowledgement was
    /// [`ResidualAcknowledgement::Withheld`].
    ///
    /// Not reachable from [`crate::dialogs::redact`], whose confirm control is
    /// disabled until the box is ticked — and answered here anyway, because the
    /// dialog's gate is a *drawing* decision and this is the one that governs
    /// the file system. A control being greyed is not a mechanism.
    ResidualsNotAcknowledged {
        /// How many items were disclosed and not acknowledged.
        residuals: usize,
    },
    /// The write-time re-proof (§2.2) found redacted text in a decoded stream
    /// of the buffer about to be written.
    ///
    /// Unreachable through [`prepare_redaction_apply`], which refuses the same
    /// condition. It exists because §2.2's whole argument is that the guarantee
    /// must not depend on how the value was constructed.
    VerificationFailed {
        /// The strings that survived.
        survivors: Vec<String>,
    },
    /// The bytes were proven and the file system refused them: the folder is
    /// gone, the path is read-only, the volume is full.
    FileSystem(std::io::Error),
}

impl std::fmt::Display for WriteRefusal {
    /// Diagnostic prose for the trace, and for nothing else.
    ///
    /// `check-ui-strings.sh`'s exclusion 3 permits a `Display` impl to carry
    /// text that is not in the catalog **because it is diagnostic**, and states
    /// in the same breath that this "is not permission to route UI text through
    /// an error type". Nothing here reaches an operator; the sentences the
    /// dialog shows are [`crate::text::redact`]'s.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ResidualsNotAcknowledged { residuals } => {
                write!(f, "{residuals} disclosed residual(s) were not acknowledged")
            }
            Self::VerificationFailed { survivors } => write!(
                f,
                "the absence proof failed at the write: {} string(s) survived",
                survivors.len()
            ),
            Self::FileSystem(e) => write!(f, "the file could not be written: {e}"),
        }
    }
}

/// A completed, verified, **unwritten** redaction: the exact bytes that will
/// land on disk if — and only if — the operator confirms.
///
/// Holding the finished bytes across the confirmation (rather than recomputing
/// them after it) is deliberate and is what makes the Apply report honest: the
/// report describes what *did* happen in memory, so the numbers the operator
/// reads are measurements rather than predictions. It also removes the window
/// in which the document could change between the report and the write.
///
/// ★ [`Self::bytes`] is **private and has no accessor** — see §2.1. Everything
/// a surface needs in order to describe this value is public; the buffer itself
/// leaves only through [`Self::write_to`].
pub struct PreparedRedaction {
    /// The redacted document, as a single full-rewrite revision.
    ///
    /// Private, deliberately and load-bearingly. See §2.1: adding a `pub fn
    /// bytes()` here would restore exactly the surface `pdfcer`'s
    /// `redact-apply` uses to write an unverified file.
    bytes: Vec<u8>,
    /// Core's report — what was removed, per carrier, plus its own disclosed
    /// residuals.
    pub report: RedactionReport,
    /// This module's independent absence proof over the bytes.
    pub verification: AbsenceVerification,
    /// Objects that had to be promoted out of an object stream to materialise
    /// the session's edits (full rewrite #1).
    ///
    /// Surfaced because the engine's R38 requires promotion to be counted and
    /// named: promotion leaves the object's previous value inside the untouched
    /// container. In a redaction context that is worth saying out loud even
    /// though it is not itself a leak of redacted text — page content streams
    /// cannot live in an object stream at all (ISO 32000-1 §7.5.7: stream
    /// objects shall not be compressed into one), so the stale copy can only be
    /// a dictionary. The absence proof covers the case that matters anyway, by
    /// decoding the container and grepping it like any other stream.
    pub promoted_by_materialisation: Vec<ObjId>,
}

impl std::fmt::Debug for PreparedRedaction {
    /// **Hand-written so that `{:?}` cannot emit the redacted document.**
    ///
    /// `#[derive(Debug)]` on a struct with a `Vec<u8>` prints every byte. This
    /// value is, by construction, the most sensitive buffer the program ever
    /// holds — and a `format!("{prepared:?}")` in a trace line, a panic message
    /// or a test failure would put a whole redacted PDF into a log file that
    /// nobody thinks of as containing document content.
    ///
    /// It reports the length instead, which is the only thing a reader of a
    /// diagnostic actually wants from that field.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedRedaction")
            .field("bytes", &self.bytes.len())
            .field("marks_applied", &self.report.marks_applied)
            .field("verification", &self.verification)
            .field("promoted", &self.promoted_by_materialisation.len())
            .finish()
    }
}

impl PreparedRedaction {
    /// How large the redacted document is, in bytes.
    ///
    /// A number, not the buffer — the one thing a surface legitimately wants to
    /// know about [`Self::bytes`] without being able to write it anywhere.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    /// **Write the redacted document to `target`, and prove it one last time
    /// first.**
    ///
    /// The only path by which these bytes leave this module. See §2.2 for why
    /// the proof runs again here rather than being trusted from the
    /// constructor, and §2.3 for why the acknowledgement is an argument.
    ///
    /// # Order of the two gates, and why it is this way round
    ///
    /// The acknowledgement is checked **first**, before the re-proof, because
    /// it is the cheaper question and because the two refusals mean completely
    /// different things: one is *"the operator has not agreed"* (an ordinary,
    /// expected state) and the other is *"pdfcer and pdfcer disagree about
    /// whether the text is gone"* (a defect). Running the expensive check to
    /// answer the ordinary case would also mean re-inflating every stream in
    /// the document each time an operator opened the dialog with a residual
    /// pending.
    ///
    /// # ★★★ Why the write IS atomic, as of 2026-09-04
    ///
    /// It was not, and the argument for that is kept below because it was
    /// sound **for the destination this method used to have** and stopped being
    /// sound the moment that changed.
    ///
    /// What it used to say: this shell had no `write_atomic` helper, both of
    /// its writers called `std::fs::write`, and inventing one here alone would
    /// make the most security-critical write in the program the only one with a
    /// different mechanism — *"which is how a subsequent 'consistency' edit
    /// removes it."* And, specifically, atomicity was not a safety property
    /// here: a truncated write could only *lose* trailing bytes of an
    /// already-redacted buffer, never introduce un-redacted content, and a
    /// truncated PDF does not open, so the failure was loud.
    ///
    /// ★ **Every clause of that depended on `target` never being the source
    /// file.** A torn write to `sheet-redacted.pdf` costs a file that did not
    /// exist five seconds ago. A torn write to `sheet.pdf` — which
    /// [`crate::dialogs::redact`] can now be asked for, on the operator's
    /// instruction of 2026-09-04 — destroys the **only remaining copy of the
    /// content being removed**, and leaves neither the original nor the
    /// redacted document. That is the one loss in this feature that cannot be
    /// undone by doing the work again.
    ///
    /// So: write to `<target>.pdfcer-tmp`, then `std::fs::rename` over the
    /// target. Same shape, same extension and same failure handling as
    /// [`crate::app::save::save_in_place`], deliberately — the "one mechanism"
    /// objection is answered by *matching the shell's in-place writer* rather
    /// than by staying unsafe, and rename-over-target is a single directory
    /// operation on every filesystem pdfcer runs on.
    ///
    /// It is applied to **both** destinations rather than only the dangerous
    /// one. A branch would mean the safe path and the dangerous path used
    /// different writers, which is the arrangement in which somebody later
    /// "simplifies" the wrong one.
    ///
    /// ★ The temporary file is removed if the rename fails, so a refusal leaves
    /// no half-written PDF beside the operator's document — and it carries the
    /// redacted bytes, which is one more reason not to leave it lying about.
    ///
    /// # Errors
    ///
    /// [`WriteRefusal`], and every variant of it means **no file was
    /// produced**. In particular there is no path in which an unacknowledged
    /// residual or a failed proof results in a partial write.
    pub fn write_to(
        &self,
        target: &Path,
        acknowledgement: ResidualAcknowledgement,
    ) -> Result<usize, WriteRefusal> {
        let residuals = self.verification.residuals.len();
        if residuals > 0 && acknowledgement == ResidualAcknowledgement::Withheld {
            return Err(WriteRefusal::ResidualsNotAcknowledged { residuals });
        }
        // ★ §2.2 — the proof between the buffer and the syscall.
        if let Some(survivors) =
            proof::survivors_in_content_streams(&self.bytes, &self.report.redacted_text)
        {
            return Err(WriteRefusal::VerificationFailed { survivors });
        }
        // ★ Temp-then-rename. See the "Why the write IS atomic" section: the
        // destination may now be the source document, and a torn write there
        // would destroy the last copy of the content being removed.
        let temporary = target.with_extension("pdfcer-tmp");
        std::fs::write(&temporary, &self.bytes).map_err(WriteRefusal::FileSystem)?;
        if let Err(err) = std::fs::rename(&temporary, target) {
            // The temporary holds a redacted document. Leaving it beside the
            // operator's file after a failure would be a stray artefact of the
            // most sensitive kind, so it goes even though the removal itself
            // may also fail — there is nothing further to try and nothing to
            // report about it that the rename's error does not already say.
            let _ = std::fs::remove_file(&temporary);
            return Err(WriteRefusal::FileSystem(err));
        }
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ `glyphs=` beside `marks=`, on `HANDOFF.md` §2's advice
                // about the ink trail: a build that applied every mark and
                // removed no character would emit an otherwise identical line,
                // and `glyphs=0` on a non-zero `marks=` is the shape of exactly
                // that failure. `verified=` is the proof's own verdict and is
                // the field `tools/ui-verify` reads; `residuals=` beside it is
                // what stops `verified=false` reading as "the proof did not
                // run".
                //
                // `path` is Debug-quoted, exactly as `save-copy`'s is: a
                // Windows path routinely contains a space, and a consumer
                // splitting this line into `key=value` pairs would otherwise
                // read `Files\a.pdf` as a field name and lose every field after
                // it.
                "redact-written path={:?} bytes={} marks={} pages={} glyphs={} streams={} \
                 checked={} residuals={} verified={} promoted={}",
                target,
                self.bytes.len(),
                self.report.marks_applied,
                self.report.pages_redacted,
                self.report.glyphs_removed,
                self.report.content_streams_rewritten,
                self.verification.strings_checked,
                residuals,
                self.verification.is_clean(),
                self.promoted_by_materialisation.len(),
            )
        });
        Ok(self.bytes.len())
    }
}

/// **Run the whole apply pipeline in memory and prove the result.**
///
/// See §1 for the two-full-rewrite shape, why the session must be materialised
/// first, and what the absence proof does with each class of survivor.
/// **Nothing is written to disk here**; the caller writes through
/// [`PreparedRedaction::write_to`] after the operator confirms, and that method
/// proves the bytes again on the way past.
///
/// # ★ This is the one call site of [`pdfcer_core::redact::apply_redactions`] in
/// this crate
///
/// Asserted, not asked for: `redact::sealed` parses every `.rs` file in the crate and
/// fails if a second one appears, or if this one disappears. See §2.4.
///
/// # Errors
///
/// [`RedactApplyRefusal`] — and every variant of it means *no file was produced
/// and no file was touched*. In particular there is no path in which a failed
/// full rewrite degrades into an incremental save.
pub fn prepare_redaction_apply(
    session: &EditSession,
) -> Result<PreparedRedaction, RedactApplyRefusal> {
    // Read the mark census from the SESSION graph, never the base document:
    // the marks the operator is most likely to be applying are the ones they
    // just made, which the base revision by construction does not have. This is
    // the same walk `crate::panels::redact` lists from, for the same reason.
    if redact::count_redaction_marks(&session.graph()) == 0 {
        return Err(RedactApplyRefusal::NothingToApply);
    }

    // Full rewrite #1 — materialise. `to_full_bytes`, never
    // `to_incremental_bytes`: see §1.1. A failure here is a refusal, not a cue
    // to try the other method.
    let (materialised, materialise_report) = session
        .to_full_bytes(&SaveOptions::identity())
        .map_err(|err| RedactApplyRefusal::FullRewriteUnavailable {
            reason: err.to_string(),
        })?;

    let doc = Document::from_bytes(materialised).map_err(|err| {
        RedactApplyRefusal::MaterialisedDocumentUnreadable {
            reason: err.to_string(),
        }
    })?;

    // Full rewrite #2 — the removal itself. `apply_redactions` forces its own
    // full rewrite internally (R35); this call site cannot ask it for anything
    // else, which is the property that makes "apply is never incremental"
    // structural rather than a convention.
    let (bytes, report) =
        redact::apply_redactions(&doc, &SaveOptions::identity()).map_err(|err| match err {
            // A write failure is the same class of refusal as a failed
            // materialisation: the full rewrite did not happen.
            RedactError::Write(inner) => RedactApplyRefusal::FullRewriteUnavailable {
                reason: inner.to_string(),
            },
            other => RedactApplyRefusal::CoreRefused {
                reason: other.to_string(),
            },
        })?;

    let proven = proof::prove(&bytes, &report.redacted_text);
    if let Some(survivors) = proven.survivors {
        return Err(RedactApplyRefusal::VerificationFailed { survivors });
    }

    Ok(PreparedRedaction {
        bytes,
        report,
        verification: proven.verification,
        promoted_by_materialisation: materialise_report.promoted,
    })
}

// ===========================================================================
// ★★★ THE DEFERRED APPLY — 2026-09-04, `Pass 250.1`
// ===========================================================================

/// What [`apply_into_session`] did, once it had done it.
///
/// Distinct from [`PreparedRedaction`] and deliberately **not** a variant of
/// it: that type's whole shape is *"finished bytes nobody has written yet"*,
/// and there are no bytes here. The removal is in the session; the file system
/// has not been touched and will not be until the operator saves.
#[derive(Debug)]
pub struct AppliedRedaction {
    /// The engine's report — what was removed, per carrier, plus its own
    /// disclosed residuals. The same value [`PreparedRedaction::report`] holds,
    /// from the same engine call on the same marks.
    pub report: RedactionReport,
    /// This module's independent absence proof, run over the session's own
    /// serialisation **after** the collapse.
    pub verification: AbsenceVerification,
    /// ★★★ **How many undo steps the apply destroyed.**
    ///
    /// Read from `EditSession::undo_depth()` immediately before the call, and
    /// it is the number the operator has to be told — *before* he commits, by
    /// [`crate::text::redact::undo_will_be_cleared`], and again after, in the
    /// edit disclosure. The engine's verb *finalizes*: it collapses the session
    /// onto a clean redacted base with an empty edit and undo stack, so every
    /// step in the log — including the ones that have nothing to do with the
    /// redaction — is gone.
    ///
    /// A count rather than a `bool` because *"this will discard 14 undo
    /// steps"* and *"this will discard 1 undo step"* are different decisions,
    /// and *"0"* is a real and common state that is not worth a sentence.
    pub undo_steps_cleared: usize,
}

/// **Apply every `/Redact` mark INTO the open session, and prove the result.**
///
/// The half of `OPERATOR_REQUESTS.md` O125 that could not be built this
/// morning:
///
/// > *"why does it have to save to a new file right away? Why can't it just
/// > wait on saving until I choose to save over the existing file or save as a
/// > new file?"*
///
/// **Nothing is written.** The redaction becomes part of the document the
/// operator is looking at, and `file.save`, `file.save_as` and
/// `file.save_copy` decide where it lands and when, exactly as they do for
/// every other edit.
///
/// # ★★★ 1. The property that had to be re-established: an incremental save
/// cannot leak
///
/// Our engine request's §4.1 asked for this to be enforced by a **refusal** —
/// a redacted session that declines `to_incremental_bytes` by name, ideally
/// through `Pass 73.0`'s `requires_full` layer. **The engine did not ship
/// that, and said so, and it is right.** Its reasoning, from
/// `EditSession::apply_redactions`' own doc comment:
///
/// > *"The request asked that a redacted session refuse incremental save,
/// > because a redaction left in a dirty set over the original base would leak
/// > via `/Prev`. This implementation removes that hazard at the root instead:
/// > after the collapse there IS no un-redacted base — the new base is a
/// > single-revision full rewrite with the content already gone — so an
/// > incremental save appends to clean bytes and cannot leak."*
///
/// That is a stronger guarantee than the refusal we asked for, and it is also
/// an unfalsifiable-sounding claim, so it was **measured rather than believed**.
/// Read against `pdfcer-core` at `8b24a0a` (v0.37.0) on 2026-09-04, the verb:
///
/// 1. serialises the session — base plus every pending edit, marks included —
///    with `to_full_bytes`;
/// 2. re-parses it, runs the free-function removal, and re-parses *that*;
/// 3. `*self = EditSession::new(redacted_base)` — so the un-redacted document
///    is dropped, not superseded — preserving only `quad_point_order`, and
///    sets its `redacted` flag;
/// 4. touches `self` only after every fallible step has succeeded, so a failed
///    apply leaves the session exactly as it was.
///
/// The measurements that back each claim, and the tests that would catch a
/// regression in any of them, are in this module's test suite:
///
/// | claim | test |
/// |---|---|
/// | an incremental save of a redacted session contains none of the removed text | `an_incremental_save_of_a_redacted_session_cannot_leak_the_removed_text` |
/// | …and still cannot after further ordinary edits appended a real revision | same test, second half — the output has a `/Prev`, and the revision it points at is the **redacted** base |
/// | the two save modes agree | `both_save_modes_of_a_redacted_session_are_clean` |
/// | it holds on a real document, not only a synthetic one | `a_real_drawing_survives_the_deferred_route` (`fixtures/a1-titleblock.pdf`) |
///
/// ⇒ **The shell therefore does NOT gate saving on `has_applied_redaction()`,**
/// on the engine's explicit instruction, because doing so would refuse a
/// legitimate incremental save of a document that is already clean. What the
/// shell does instead is prove the saved bytes — see [`prove_saved_bytes`],
/// which `crate::app::save` runs on the way to the file system. A guarantee by
/// construction plus a check at the boundary is the same posture §2.2 takes
/// about [`PreparedRedaction::write_to`], and for the same reason: the promise
/// must not depend on how the value was constructed.
///
/// # ★★ 2. It is irreversible the moment it runs, and the surface must say so
/// FIRST
///
/// The engine's verb **finalizes**. The operator can keep editing afterwards
/// but cannot undo past the redaction, and the undo steps he had *before* it go
/// too. [`AppliedRedaction::undo_steps_cleared`] carries the number and
/// [`crate::text::redact::undo_will_be_cleared`] is the sentence
/// [`crate::dialogs::redact`] draws **above the confirm control**, not after
/// it. Our own request §4.3 asked for exactly this and the operator's ruling
/// was *"finalizing the document and can't be undone is ok for now"* — it is
/// acceptable *because it is disclosed*, and the disclosure is the half we owe.
///
/// # 3. The proof still runs, and it runs on the session's own bytes
///
/// §1.3's rule is not relaxed for the deferred route. After the collapse the
/// session is serialised once more with `to_full_bytes` and [`proof::prove`] is
/// run over it. A decoded-stream survivor is a
/// [`RedactApplyRefusal::VerificationFailed`] — with the honest caveat, stated
/// here rather than hidden, that **the session has already been mutated by
/// then**: unlike [`prepare_redaction_apply`], this route cannot refuse before
/// the fact, because the engine's verb is the fact. What the caller must do
/// with that refusal is refuse to *save*, which is what [`prove_saved_bytes`]
/// enforces at the only place it can still be enforced.
///
/// It is not reachable in practice from [`crate::dialogs::redact`], which runs
/// [`prepare_redaction_apply`] on the identical marks seconds earlier and
/// refuses there. It is answered anyway, because "unreachable through the one
/// surface that exists today" is a claim about a caller.
///
/// # Errors
///
/// [`RedactApplyRefusal`]. Every variant except `VerificationFailed` means the
/// session was **not touched** — that is the engine's own guarantee, not this
/// function's inference. `VerificationFailed` means it was, and that no file
/// derived from it may be written.
pub fn apply_into_session(
    session: &mut EditSession,
) -> Result<AppliedRedaction, RedactApplyRefusal> {
    // Same census, same graph, same reason as `prepare_redaction_apply`: the
    // marks that matter are the ones the operator just made, and the base
    // revision by construction does not have them.
    if redact::count_redaction_marks(&session.graph()) == 0 {
        return Err(RedactApplyRefusal::NothingToApply);
    }
    // ★ Taken BEFORE the call, because the call is what destroys it. Reading it
    // afterwards would report 0 every time and the disclosure would say
    // "nothing was lost" on exactly the runs where something was.
    let undo_steps_cleared = session.undo_depth();

    // ★★★ The engine's session-level verb — the second and last call to
    // `apply_redactions` in this crate. See §2.4: `sealed` pins the count at
    // two, in this file, and counts this method call as readily as the free
    // one.
    let report = session.apply_redactions().map_err(|err| match err {
        // Same mapping as the byte route, so one refusal taxonomy serves both
        // and `crate::text::redact::refusal_message` needs no second table.
        RedactError::Write(inner) => RedactApplyRefusal::FullRewriteUnavailable {
            reason: inner.to_string(),
        },
        RedactError::NothingToApply => RedactApplyRefusal::NothingToApply,
        other => RedactApplyRefusal::CoreRefused {
            reason: other.to_string(),
        },
    })?;

    // §3 — prove what is now in the session, on the session's own
    // serialisation. `to_full_bytes` and never `to_incremental_bytes`: the
    // latter is banned inside this directory by `sealed`, and the ban stands
    // even though the engine has made it safe here, because what makes it safe
    // is a property of the engine that this directory must not start assuming.
    let (settled, _) = session
        .to_full_bytes(&SaveOptions::identity())
        .map_err(|err| RedactApplyRefusal::FullRewriteUnavailable {
            reason: err.to_string(),
        })?;
    let proven = proof::prove(&settled, &report.redacted_text);
    if let Some(survivors) = proven.survivors {
        return Err(RedactApplyRefusal::VerificationFailed { survivors });
    }

    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // `undo_cleared=` is on the line because it is the one consequence
            // of this route that has no equivalent on the write-now route, and
            // a build that had silently stopped disclosing it would produce an
            // otherwise identical line.
            "redact-into-session marks={} pages={} glyphs={} streams={} checked={} \
             residuals={} verified={} undo_cleared={}",
            report.marks_applied,
            report.pages_redacted,
            report.glyphs_removed,
            report.content_streams_rewritten,
            proven.verification.strings_checked,
            proven.verification.residuals.len(),
            proven.verification.is_clean(),
            undo_steps_cleared,
        )
    });

    Ok(AppliedRedaction {
        report,
        verification: proven.verification,
        undo_steps_cleared,
    })
}

/// **How many items a report and a proof disclose as NOT removed.**
///
/// The count behind `crate::dialogs::redact::residual_lines`, lifted out of it
/// on 2026-09-04 because the deferred route needs the same number and has no
/// dialog to ask. One derivation, in the domain module, so a residual can never
/// be counted one way in the sentence the operator acknowledges and another way
/// in the sentence he is shown afterwards.
///
/// Five sources, and the list is the same one that module documents at length:
/// carriers the engine disclosed rather than scrubbed, retained marks, vector
/// geometry it could not cut, clips whose outline had to be kept, and the
/// proof's own raw-byte residuals.
///
/// ★ **Promotion is deliberately NOT counted here**, and that is the one place
/// the two lists differ. Objects promoted out of an object stream are a
/// leftover of *this shell's* materialisation step
/// ([`prepare_redaction_apply`]'s full rewrite #1) and are reported by it; the
/// deferred route has no such step of its own — the engine materialises
/// internally — so there is no promotion list to report and inventing a zero
/// would be a claim rather than a measurement.
/// `crate::dialogs::redact`'s own test pins the two together so the difference
/// stays exactly one item and cannot drift.
#[must_use]
pub fn residual_count(report: &RedactionReport, verification: &AbsenceVerification) -> usize {
    use pdfcer_core::redact::CarrierAction;
    report
        .carriers
        .iter()
        .filter(|c| c.action == CarrierAction::DisclosedNotScrubbed)
        .count()
        + usize::from(report.marks_retained > 0)
        + usize::from(report.vector_paths_intersecting > 0)
        + usize::from(report.vector_clips_kept > 0)
        + verification.residuals.len()
}

/// **The absence proof, run over bytes that are one syscall from a file.**
///
/// §2.2's argument, moved to the one place the deferred route can still make
/// it. On the write-now route the proof sits inside
/// [`PreparedRedaction::write_to`], between the buffer and the syscall. The
/// deferred route has no such buffer — the bytes are built by
/// `crate::app::save` at save time, minutes later, possibly after further
/// edits, possibly by a different save verb — so the check has to be made
/// available *to* that module rather than owned by this one.
///
/// `claims` is [`pdfcer_core::redact::RedactionReport::redacted_text`], carried
/// on the document since the apply. An empty slice is the overwhelmingly common
/// case (no redaction has been applied to this document) and returns `Ok`
/// without decoding anything, so an ordinary save pays nothing.
///
/// # ★ Why the DECODED-stream sweep and not a raw byte scan
///
/// Both were considered and the split is [`proof`]'s standing one. A raw byte
/// run that survives outside every decoded stream is a *disclosure*, not a
/// leak — it is routinely a font `name` table, which is the exact false refusal
/// that made this feature useless on `fixtures/a1-titleblock.pdf` until it was
/// fixed this morning. Refusing a save on one would re-create that defect at a
/// worse moment: after the operator has redacted, with his only route to a file
/// blocked. A survivor in a **decoded** stream is content a reader will render
/// or extract, and there is no reading of that under which the file is safe to
/// hand over.
///
/// # Errors
///
/// The strings that survived. A caller that gets one must not write the file.
pub fn prove_saved_bytes(bytes: &[u8], claims: &[String]) -> Result<(), Vec<String>> {
    if claims.is_empty() {
        return Ok(());
    }
    proof::survivors_in_content_streams(bytes, claims).map_or(Ok(()), Err)
}

/// The security assertions for this pipeline, in their own file since
/// 2026-09-04 — see [`tests`]'s header for the seam.
#[cfg(test)]
mod tests;
