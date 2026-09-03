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
//! that `apply_redactions` is *called* in exactly one place — the function
//! below. A second call site anywhere is a test failure naming the file.
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
//! job, and that surface asks every time. See §4.
//!
//! It does not mutate the open document. Applying produces a **new file**; the
//! session keeps its marks, its undo log and its epoch. That is not a
//! limitation — it is the property that makes an irreversible operation safe to
//! offer, and [`crate::text::redact`] says so on screen in as many words.
//!
//! ---
//!
//! # 4. ★ Save-as, never save-over — and why that is the sharpest rule here
//!
//! `HANDOFF.md` §3 item 5 records the operator's standing rule — *"Read may
//! produce a new document; it may not modify this one"* — with the enforcement
//! at the **save** rather than at the operation, and names redact-apply
//! explicitly as one of the capabilities it settles in advance.
//!
//! [`crate::app::save`] §3.4 draws the matching line for this shell: a save is a
//! **copy**, and the original is never overwritten. For a redaction that stops
//! being a convention and becomes the whole safety argument, because the source
//! file is *the only remaining copy of the content being removed*. Overwriting
//! it would be the most damaging single act this shell could perform, on the one
//! operation least able to survive a mistake.
//!
//! So: the destination is always asked for, and the suggested name is never the
//! file that was opened ([`crate::text::redact::suggested_suffix`], asserted by
//! [`crate::dialogs::redact`]'s own test in the shape
//! `crate::app::save::suggested_path` and `crate::dialogs::ocr::suggested_path`
//! both established). Nothing in this module can produce a path at all, which is
//! the structural half of the same rule.

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

pub use proof::AbsenceVerification;

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
    /// # Why the write is not atomic, where the salvage source's was
    ///
    /// The old shell routed this through a `write_atomic` helper it shared with
    /// its ordinary save. This shell has no such helper — both of its existing
    /// writers (`crate::app::save::write_copy` and `crate::dialogs::ocr`) call
    /// `std::fs::write` — and inventing one here for this path alone would make
    /// the most security-critical write in the program the only one with a
    /// different mechanism, which is how a subsequent "consistency" edit
    /// removes it.
    ///
    /// It is also, specifically here, not a safety property. A truncated write
    /// can only *lose* trailing bytes of an already-redacted buffer: it cannot
    /// introduce un-redacted content, and a truncated PDF does not open, so the
    /// failure is loud rather than quiet. The hazard atomicity defends against
    /// — a plausible, working, wrong file — is not reachable from this
    /// direction. (An atomic writer shared by all three call sites would still
    /// be an improvement, and is recorded in this work's report as such rather
    /// than smuggled in here.)
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
        let residuals = self.verification.raw_byte_residuals.len();
        if residuals > 0 && acknowledgement == ResidualAcknowledgement::Withheld {
            return Err(WriteRefusal::ResidualsNotAcknowledged { residuals });
        }
        // ★ §2.2 — the proof between the buffer and the syscall.
        if let Some(survivors) =
            proof::survivors_in_decoded_streams(&self.bytes, &self.report.redacted_text)
        {
            return Err(WriteRefusal::VerificationFailed { survivors });
        }
        std::fs::write(target, &self.bytes).map_err(WriteRefusal::FileSystem)?;
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use pdfcer_core::annot_author::{Quad, RedactSpec};
    use pdfcer_core::page_tree::{self, Rect};
    use pdfcer_core::text_extract::{self, ExtractOptions};
    use pdfcer_core::vartext::Quadding;

    /// The secret this suite proves the absence of.
    ///
    /// Deliberately long and distinctive: a short token could be absent by
    /// luck, and a proof that can pass by luck proves nothing.
    const SECRET: &str = "CONFIDENTIALWITNESSNAME";

    /// A one-page document whose content stream shows `SECRET` followed by a
    /// word that must SURVIVE.
    ///
    /// The survivor is what stops the test from passing on a build that simply
    /// erased the page.
    fn secret_pdf() -> Vec<u8> {
        let content = format!("BT /F1 12 Tf 20 100 Td ({SECRET}) Tj ( KEEPTHIS) Tj ET");
        let stream = format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        );
        assemble(&[
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
            &stream,
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        ])
    }

    /// Assemble a classic single-revision PDF from object bodies `1..=n` with a
    /// correct xref table. Object 1 must be the catalog.
    ///
    /// The same fixture shape `pdfcer-core`'s own redaction tests use —
    /// synthetic, so that every byte in the file is one this suite put there.
    /// `pub(super)` so [`super::proof`]'s tests share it rather than growing a
    /// second, subtly different assembler.
    pub(super) fn assemble(bodies: &[&str]) -> Vec<u8> {
        let mut buf = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in bodies.iter().enumerate() {
            offsets.push(buf.len());
            buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref_at = buf.len();
        let n = bodies.len() + 1;
        buf.extend_from_slice(format!("xref\n0 {n}\n0000000000 65535 f \n").as_bytes());
        for off in &offsets {
            buf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        buf.extend_from_slice(
            format!("trailer\n<< /Size {n} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n")
                .as_bytes(),
        );
        buf
    }

    /// A session with ONE unsaved `/Redact` mark over the secret — the exact
    /// state an operator is in when they press Apply without having saved.
    fn session_with_unsaved_mark() -> EditSession {
        let doc = Document::from_bytes(secret_pdf()).unwrap();
        let mut session = EditSession::new(doc);
        let created = session
            .mark_redactions_by_search(SECRET, false)
            .expect("the fixture's text is extractable");
        assert!(!created.is_empty(), "the search must find the secret");
        session
    }

    /// A scratch path under the OS temporary directory, unique to this test.
    ///
    /// `std::env::temp_dir` rather than a path in the repository, exactly as
    /// `crate::app::save`'s tests do it: a test that writes beside the fixtures
    /// leaves a file somebody eventually commits.
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("pdfcer-gui-redact-tests");
        std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
        dir.join(name)
    }

    // -- THE SECURITY ASSERTION ---------------------------------------------

    /// ★★ **The headline gate for the apply path.**
    ///
    /// After apply-and-save through [`prepare_redaction_apply`], the redacted
    /// text must not be recoverable from the saved bytes by any means pdfcer
    /// itself offers. Three independent measures, because a single one could be
    /// satisfied by a build that merely hid the text:
    ///
    /// 1. **`extract-text`** — the very tool `pdfcer extract-text` and this
    ///    shell's Copy-text both use — finds nothing;
    /// 2. **every decoded stream** (content streams, XObjects, object-stream
    ///    containers, metadata) contains no occurrence;
    /// 3. **the raw file bytes** contain no occurrence.
    ///
    /// And the negative control: `KEEPTHIS`, which was never marked, is still
    /// extractable. Without it, a build that emitted an empty page would pass
    /// all three assertions above while destroying the document.
    ///
    /// This is deliberately an assertion of ABSENCE, not of appearance. A
    /// raster test could only show that the region is painted black, which is
    /// precisely the false-redaction failure ISO 32000-1 §12.5.6.23 forbids
    /// ("clipping or image masks shall not be used to hide that data") — a black
    /// box over live text is what this feature exists to never ship.
    #[test]
    fn applied_redaction_leaves_no_recoverable_trace_in_the_saved_bytes() {
        let session = session_with_unsaved_mark();
        let prepared = prepare_redaction_apply(&session).expect("the apply must succeed");

        // The bytes are private, so the assertion goes through the one door
        // that exists — which is itself the property this module is about.
        let target = scratch("headline.pdf");
        let _ = std::fs::remove_file(&target);
        let written = prepared
            .write_to(&target, ResidualAcknowledgement::Withheld)
            .expect("a clean redaction needs no acknowledgement");
        assert_eq!(written, prepared.byte_len());
        let bytes = std::fs::read(&target).expect("the redacted file must exist");

        // (3) raw bytes.
        assert!(
            !proof::contains(&bytes, SECRET.as_bytes()),
            "the redacted text survived in the raw saved bytes"
        );

        let back = Document::from_bytes(bytes.clone()).expect("the redacted output must re-parse");

        // (2) every decoded stream in the file — asked through the proof's own
        // sweep, which is the wide one.
        assert_eq!(
            proof::survivors_in_decoded_streams(&bytes, &[SECRET.to_owned()]),
            None,
            "the redacted text survived in a decoded stream of the saved file"
        );

        // (1) pdfcer's own text extraction — the tool an operator would actually
        // reach for to get the text back out.
        let extracted =
            text_extract::extract_document(&back, &ExtractOptions::default()).expect("extract");
        let all_text: String = extracted
            .pages
            .iter()
            .flat_map(|p| p.runs.iter())
            .map(|r| r.text.clone())
            .collect();
        assert!(
            !all_text.contains(SECRET),
            "the redacted text was recoverable via extract-text: {all_text:?}"
        );

        // The negative control — proof the test can fail.
        assert!(
            all_text.contains("KEEPTHIS"),
            "un-redacted text must survive; the page was not supposed to be emptied"
        );

        // And the mark itself is gone (§12.5.6.23 outcome 3).
        assert_eq!(
            redact::count_redaction_marks(&back),
            0,
            "the /Redact mark must be removed by apply"
        );
        let _ = std::fs::remove_file(&target);
    }

    /// The absence proof must REPORT that it ran, or the wording contract has
    /// nothing to read and the summary would have to fall back to the weaker
    /// word.
    #[test]
    fn the_absence_proof_reports_a_clean_verification() {
        let session = session_with_unsaved_mark();
        let prepared = prepare_redaction_apply(&session).unwrap();
        assert!(
            prepared.verification.strings_checked > 0,
            "the proof must have had something to check"
        );
        assert!(
            prepared.verification.is_clean(),
            "no residual expected on this fixture: {:?}",
            prepared.verification.raw_byte_residuals
        );
    }

    /// ★ **A mark that exists ONLY in the session overlay must still be
    /// applied.**
    ///
    /// The un-saved-mark trap §1.2 names: passing `session.document()` to
    /// `apply_redactions` would apply nothing and report success. The assertion
    /// that makes it bite is `marks_applied` — a build with that bug produces
    /// `NothingToApply` or a zero count, never a removal.
    #[test]
    fn a_mark_that_was_never_saved_is_still_applied() {
        let session = session_with_unsaved_mark();
        // The base revision genuinely has no mark — that is the trap.
        assert_eq!(redact::count_redaction_marks(session.document()), 0);
        assert!(redact::count_redaction_marks(&session.graph()) > 0);

        let prepared = prepare_redaction_apply(&session).unwrap();
        assert!(
            prepared.report.marks_applied >= 1,
            "an unsaved mark must be applied, not silently skipped"
        );
        assert!(prepared.report.glyphs_removed >= SECRET.len() as u64);
    }

    /// ★ **The output is a SINGLE revision.**
    ///
    /// A `/Prev` in the trailer would mean a prior revision is reachable in the
    /// saved file, which for a redaction is the un-redacted content one hop
    /// away — R35's whole point, and the reason §1.1 forbids the incremental
    /// writer this shell otherwise uses for every save.
    #[test]
    fn the_output_is_one_revision_with_no_prior_revision_to_walk_back_to() {
        let session = session_with_unsaved_mark();
        let prepared = prepare_redaction_apply(&session).unwrap();
        let target = scratch("one-revision.pdf");
        let _ = std::fs::remove_file(&target);
        prepared
            .write_to(&target, ResidualAcknowledgement::Withheld)
            .unwrap();
        let back = Document::from_bytes(std::fs::read(&target).unwrap()).unwrap();
        assert!(
            back.trailer().get(b"Prev").is_none(),
            "a redaction apply must leave no /Prev — a prior revision holds the un-redacted bytes"
        );
        let _ = std::fs::remove_file(&target);
    }

    /// A document with no marks is refused by name rather than producing an
    /// empty "successful" apply, so the caller can never present a report that
    /// describes nothing as if it were a removal.
    #[test]
    fn an_unmarked_document_is_refused_by_name() {
        let doc = Document::from_bytes(secret_pdf()).unwrap();
        let session = EditSession::new(doc);
        assert_eq!(
            prepare_redaction_apply(&session).unwrap_err(),
            RedactApplyRefusal::NothingToApply
        );
    }

    /// ★★★ **A region over a raster image now DESTROYS the samples**, and this
    /// test is the record of the day that changed.
    ///
    /// It read `a_region_over_an_image_refuses_the_whole_apply` until
    /// 2026-09-03 and asserted that the engine declined the entire document —
    /// which was true, was the operator's headline complaint
    /// (`OPERATOR_REQUESTS.md` O103, *"every time I've tried the redact feature
    /// it tells me it can't"*), and stopped being true with `pdfcer-core`
    /// v0.26.0 the same day.
    ///
    /// ★★ **A test asserting an external limitation goes red when the
    /// limitation lifts, and that red is a REPORT rather than a regression.**
    /// It is also the only member of that family that behaves well: the prose
    /// version of the same claim — in `text::redact`, in a UI string — went on
    /// compiling and passing, and had to be corrected because the engine's reply
    /// told us to. ⇒ Where a stale external claim can be spelled as an
    /// assertion, spell it as one.
    ///
    /// What it asserts now is the pair the operator cares about: the apply
    /// SUCCEEDS, and the report says the image was dealt with rather than
    /// quietly stepped over.
    #[test]
    fn a_region_over_an_image_destroys_the_samples_and_says_so() {
        let content = "q 200 0 0 100 20 20 cm /Im0 Do Q";
        let stream = format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        );
        let image = "<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace \
                     /DeviceGray /BitsPerComponent 8 /Length 1 >>\nstream\n\x00\nendstream";
        let bytes = assemble(&[
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /XObject << /Im0 5 0 R >> >> /Contents 4 0 R >>",
            &stream,
            image,
        ]);
        let doc = Document::from_bytes(bytes).unwrap();
        let mut session = EditSession::new(doc);
        session
            .add_redaction(
                0,
                &RedactSpec {
                    quads: vec![Quad::from_rect(Rect::from_corners(30.0, 30.0, 150.0, 90.0))],
                    fill: None,
                    overlay_text: None,
                    quadding: Quadding::Left,
                },
            )
            .unwrap();

        let prepared = prepare_redaction_apply(&session)
            .expect("a region over an image is applied, not refused, since pdfcer-core v0.26.0");
        let report = &prepared.report;
        assert!(
            report.images_cleared > 0 || report.images_removed > 0,
            "the region covers the only image on the page, so the report must say what happened to it — cleared or removed. Got cleared={} removed={} retained={}",
            report.images_cleared,
            report.images_removed,
            report.marks_retained
        );
        // ★ And the mark was APPLIED rather than retained. A retained mark is
        // the honest half-measure for an image the engine cannot decode; this
        // one is a 1x1 DeviceGray it certainly can, so a retention here would
        // mean the destroy path was not reached at all and the assertion above
        // was satisfied by something else.
        assert_eq!(
            report.marks_retained, 0,
            "a decodable image must not leave the mark unapplied"
        );
    }

    /// The census the review panel lists from and the census the status bar
    /// counts from must be the same walk.
    ///
    /// Asserted here because the shell reads both and a disagreement between
    /// them is unresolvable from the operator's side.
    #[test]
    fn the_mark_list_and_the_mark_count_agree() {
        let session = session_with_unsaved_mark();
        let graph = session.graph();
        assert_eq!(
            redact::redaction_marks(&graph).len(),
            redact::count_redaction_marks(&graph)
        );
        let pages = page_tree::pages_in(&graph).unwrap();
        for mark in redact::redaction_marks(&graph) {
            assert!(
                mark.page_index < pages.len(),
                "a listed mark must name a real page"
            );
        }
    }

    // -- THE WRITE GATE -----------------------------------------------------

    /// ★★ **A disclosed residual cannot be written past without an
    /// acknowledgement, and the refusal leaves no file behind.**
    ///
    /// §2.3, asserted rather than described. The dialog greys its confirm
    /// control until the box is ticked, and **a greyed control is a drawing
    /// decision, not a mechanism** — this is the mechanism. The failure it
    /// catches is the one that matters most: a partially-redacted file handed
    /// over as a complete one.
    ///
    /// The fixture builds the residual by hand rather than hunting for a
    /// document that happens to produce one, because the point under test is
    /// the *gate*, not the classification (which [`proof`]'s own tests cover).
    #[test]
    fn an_unacknowledged_residual_refuses_the_write() {
        let session = session_with_unsaved_mark();
        let mut prepared = prepare_redaction_apply(&session).unwrap();
        assert!(
            prepared.verification.is_clean(),
            "the fixture must start clean, or the assertion below proves nothing"
        );
        prepared
            .verification
            .raw_byte_residuals
            .push("MARGARETHALE".to_owned());

        let target = scratch("unacknowledged.pdf");
        let _ = std::fs::remove_file(&target);
        let refusal = prepared
            .write_to(&target, ResidualAcknowledgement::Withheld)
            .expect_err("a withheld acknowledgement must refuse");
        assert!(
            matches!(
                refusal,
                WriteRefusal::ResidualsNotAcknowledged { residuals: 1 }
            ),
            "{refusal}"
        );
        assert!(
            !target.exists(),
            "a refused write must leave nothing behind at the path it was aimed at"
        );

        // …and the same value writes once the operator has acknowledged.
        prepared
            .write_to(&target, ResidualAcknowledgement::Given)
            .expect("an acknowledged residual may proceed");
        assert!(target.is_file());
        let _ = std::fs::remove_file(&target);
    }

    /// A clean report needs no acknowledgement, in either position.
    ///
    /// The other direction of the gate, and the one that would make the feature
    /// unusable if it were wrong: a redaction with nothing to disclose must not
    /// demand a tick nobody can give.
    #[test]
    fn a_clean_report_writes_with_the_acknowledgement_withheld() {
        let session = session_with_unsaved_mark();
        let prepared = prepare_redaction_apply(&session).unwrap();
        for ack in [
            ResidualAcknowledgement::Withheld,
            ResidualAcknowledgement::Given,
        ] {
            let target = scratch("clean.pdf");
            let _ = std::fs::remove_file(&target);
            prepared
                .write_to(&target, ack)
                .expect("a clean redaction writes under either acknowledgement");
            assert!(target.is_file());
            let _ = std::fs::remove_file(&target);
        }
    }

    /// A write that cannot happen is reported rather than swallowed.
    ///
    /// `crate::app::save`'s equivalent test, for the writer that matters more:
    /// a redaction the operator believes landed, at a path that does not exist,
    /// is a file they will look for and not find at the moment they need it.
    #[test]
    fn a_write_that_cannot_happen_is_a_named_refusal() {
        let session = session_with_unsaved_mark();
        let prepared = prepare_redaction_apply(&session).unwrap();
        let target = scratch("no-such-folder").join("nested").join("out.pdf");
        let refusal = prepared
            .write_to(&target, ResidualAcknowledgement::Withheld)
            .expect_err("a missing folder cannot be written to");
        assert!(matches!(refusal, WriteRefusal::FileSystem(_)), "{refusal}");
        assert!(!target.exists());
    }

    /// ★ **`{:?}` on a prepared redaction does not print the document.**
    ///
    /// §2.1's hand-written [`std::fmt::Debug`], pinned. The failure it prevents
    /// is silent and total: a `#[derive(Debug)]` restored during a routine
    /// tidy-up would put a whole redacted PDF into any trace, panic or test
    /// failure that formatted this value — a log file nobody thinks of as
    /// containing document content.
    #[test]
    fn the_debug_impl_reports_a_length_rather_than_the_bytes() {
        let session = session_with_unsaved_mark();
        let prepared = prepare_redaction_apply(&session).unwrap();
        let rendered = format!("{prepared:?}");
        assert!(
            !rendered.contains("KEEPTHIS"),
            "the Debug impl emitted document content: {rendered}"
        );
        assert!(
            rendered.contains(&prepared.byte_len().to_string()),
            "…and it must still report the length, which is what a diagnostic \
             actually wants from that field: {rendered}"
        );
    }
}
