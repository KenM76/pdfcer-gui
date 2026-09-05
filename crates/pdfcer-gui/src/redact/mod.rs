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
//! ## ★★★ 1.0 CORRECTED 2026-09-05 — the third route STAGES, and the
//! collapsing one it replaces is gone
//!
//! Everything from §1.1 down was written on the morning of 2026-09-04, when
//! [`pdfcer_core::redact::apply_redactions`] took a `&Document` and returned
//! `Vec<u8>` and there was no way back into an `EditSession`. That stopped
//! being the shape of the world twice in two days, and the second change is the
//! one this section now describes.
//!
//! **2026-09-04 (evening).** The engine shipped
//! `EditSession::apply_redactions` (`Pass 250.1`, `225db51`, `pdfcer-core`
//! v0.35.0) and this module gained `apply_into_session`, which applied the
//! removal into the open document by **collapsing** the session onto a clean
//! redacted base — and cleared the whole undo log doing it. The operator's
//! ruling was *"finalizing the document and can't be undone is ok **for
//! now**"*, and the dialog disclosed the step count above the confirm control
//! because of that *for now*.
//!
//! **2026-09-05.** `Pass 250.2` (`41095eb`, v0.38.0) removed the cost, and
//! `apply_into_session` is **deleted rather than kept beside its replacement**.
//! Two apply routes with different undo semantics, on one dialog, on the one
//! operation that cannot be undone, is a choice an operator would have to
//! understand in order to make it safely. So there are three routes and they
//! differ in *where the bytes go*, never in *what undo means*:
//!
//! | route | what it produces | what it touches | who calls it |
//! |---|---|---|---|
//! | [`prepare_redaction_apply`] | the finished redacted **bytes**, in memory, proven | nothing | the dialog on open — the MEASUREMENT — and the two write-now destinations |
//! | [`stage_into_session`] | a **staged** removal: a flag, and nothing else | the session's pending-redaction flag; **not** its base, overlay, or undo/redo stack | the deferred destination, through `crate::app::actions::redact` |
//! | [`save_applying_pending`] | the finished redacted **bytes**, proven, at save time | nothing — it takes `&self` | `crate::app::save::write_copy`, on every save verb, while a redaction is staged |
//!
//! The measurement still has to exist *before* the confirmation on every path
//! (§2 of [`crate::dialogs::redact`]), so the dialog still runs
//! [`prepare_redaction_apply`] on open and the numbers on screen at the moment
//! of consent are still measurements rather than predictions. What changed is
//! that the click no longer commits anything: it arms a save.
//!
//! ### ★★★ 1.0.1 The proof MOVED, because the bytes moved
//!
//! `apply_redactions_deferred` runs the removal to produce its preview report
//! and **discards the bytes**. There is therefore nothing for [`proof`] to
//! sweep at staging time, and this module does not pretend otherwise: nothing
//! in [`stage_into_session`] says *"verified"*, and
//! [`crate::text::redact::staged_into_document`] does not either. The word is
//! earned at the save, by [`save_applying_pending`], over the exact buffer that
//! is one statement from the file system — which is §2.2's own rule arriving at
//! the only place the deferred route can still keep it.
//!
//! ### ★★★ 1.0.2 The §4.1 guard is REAL now, and it is a refusal
//!
//! Our engine request's §4.1 asked that a redacted session refuse an
//! incremental save by name. `Pass 250.1` answered that the hazard was gone at
//! the root instead (the collapse left no un-redacted base). `Pass 250.2`
//! cannot make that answer, because the un-redacted content **is still live in
//! the session** — that is the whole point of preserving undo — so the engine
//! ships the guard we asked for:
//!
//! * `EditSession::to_incremental_bytes` returns `WriteError::RedactionPending`
//!   (`pdfcer-core/src/edit.rs:8348-8353`);
//! * `EditSession::to_full_bytes` returns the same
//!   (`pdfcer-core/src/edit.rs:8374-8378`);
//! * the removal happens only through `save_applying_redaction(&self, ..)`
//!   (`pdfcer-core/src/edit.rs:8569`), which takes `&self`, so undo survives the
//!   save.
//!
//! ⇒ **The leak surface is larger and the guard is stronger, and neither was
//! taken on trust.** `tests` measures both directions on a synthetic fixture
//! and on `fixtures/a1-titleblock.pdf`: the two ordinary save modes refuse **by
//! name**, and the bytes `save_applying_redaction` produces carry no `/Prev`
//! and none of the removed text, with a positive control on each.
//!
//! ### ★ 1.0.3 What a staged redaction does NOT do, and it is the one
//! surprising thing about it
//!
//! **It does not change the page.** The session is untouched, so the content is
//! still drawn, the `/Redact` marks are still drawn, and a screenshot taken one
//! frame after the operator presses the confirm control is identical to one
//! taken a frame before it. That is rule 4 satisfied rather than violated —
//! this shell adds no badge, tint or provisional layer to say *"awaiting
//! removal"* — and it is a fact the operator has to be told in words, because
//! every redaction tool he has ever used changed the picture. The saying is
//! [`crate::text::redact::staged_into_document`]'s and
//! [`crate::text::redact::saved_applying_redaction`]'s.
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
//! that each of the engine's removal verbs is *called* in exactly one FILE —
//! this one — and exactly the number of times this module accounts for. A call
//! from anywhere else is a test failure naming the file.
//!
//! ★★★ 2026-09-05: it is *one file*, not *one call*, and it is now **four
//! subjects** rather than one. `Pass 250.2` split the removal across a verb
//! that stages it, a verb that performs it at save time, and a verb that
//! un-stages it, and a monopoly pinned to one identifier would have watched
//! three quarters of the feature walk out of the module:
//!
//! | subject | calls | where |
//! |---|---|---|
//! | `apply_redactions` (the free function) | 1 | [`prepare_redaction_apply`] |
//! | `apply_redactions_deferred` | 1 | [`stage_into_session`] |
//! | `save_applying_redaction` | 1 | [`save_applying_pending`] |
//! | `cancel_pending_redaction` | 1 | [`cancel_staged_redaction`] |
//!
//! ★ The count for `apply_redactions` went **down** from two on this pass, and
//! that is the movement the pin exists to make visible: the second call was
//! `apply_into_session`'s collapsing route, which is deleted rather than left
//! beside its replacement (§1.0). A ceiling would have let that deletion pass
//! unremarked; an exact count made it an edit somebody had to write down.
//!
//! ★ `cancel_pending_redaction` is in the table even though it removes nothing
//! — it *disarms* a removal, which is the same surface seen from behind, and a
//! second caller that un-staged a redaction the operator had confirmed would be
//! the quietest possible way to ship a file he believes is redacted. The reader
//! counts method calls as well as free calls, *"because a future engine that
//! moved it onto a type would otherwise slip the monopoly silently"* — which is
//! what happened, twice, and the check was already pointed at it both times.
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
//! it, because the operator asked for exactly the thing it forbade and because
//! deferring the write destroys nothing — **the original is on disk, untouched,
//! until he chooses to overwrite it.** What was actually load-bearing in the old
//! sentence is kept and is now §4.
//!
//! ★★★ **AND THE SENTENCE CAME BACK TRUE, 2026-09-05, for a different
//! reason.** [`stage_into_session`] does **not** mutate the open document: it
//! sets one flag, and base, overlay, undo and redo are all left exactly as they
//! were. So *"applying does not mutate the open document"* is once again an
//! accurate description of this shell — and it is still not a principle, which
//! is the whole lesson of having written it as one. It is a property of
//! `Pass 250.2`, dated, and it will be re-checked rather than quoted.
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
    /// ★★★ **A redaction is already STAGED on this session** (`Pass 250.2`).
    ///
    /// Added 2026-09-05. Reachable two ways, and both are ordinary rather than
    /// exceptional: the operator opens *Review & apply* a second time on a
    /// document he has already staged, or a second `Stage` action arrives
    /// before the first frame after the first one.
    ///
    /// ★ It is a **named refusal in the pipeline** rather than a condition the
    /// dialog checks, and the difference is the one this project keeps paying
    /// for. Without it the second open would reach
    /// [`prepare_redaction_apply`]'s `to_full_bytes`, which the engine now
    /// refuses with `WriteError::RedactionPending`, and the operator would be
    /// told *"this document cannot be rewritten in full"* — a true sentence
    /// about the wrong subject, arriving at the one surface where a wrong
    /// diagnosis costs most.
    AlreadyStaged,
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
    // ★★★ Asked FIRST — before the mark census — and the ORDER is load-bearing
    // twice over.
    //
    // 1. While a redaction is staged the engine refuses `to_full_bytes` by name
    //    (`edit.rs:8374-8378`), so without this the materialisation below would
    //    fail and the operator would read *"this document cannot be rewritten
    //    in full"* on a document that can.
    //
    // 2. ★★★ **It is what stops the operator being trapped.** Ask the mark
    //    census first and a staged document with **no marks left** — he took
    //    them off in the panel after arming the removal — answers
    //    `NothingToApply`, which the dialog draws as a refusal with no control
    //    on it. Meanwhile the engine is refusing both ordinary save modes, so
    //    that document cannot be saved by any route at all and the one control
    //    that would free him is behind a phase he cannot reach. Asking the flag
    //    first sends him to `Phase::Staged`, which carries *call the removal
    //    off*. `tests::a_staged_document_with_no_marks_left_can_still_be_called_off`
    //    is the assertion, and `edit.redact_apply`'s `enabled_when("doc.pages")`
    //    — rather than a marks predicate — is what keeps the command itself
    //    reachable in that state.
    if session.has_pending_redaction() {
        return Err(RedactApplyRefusal::AlreadyStaged);
    }
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
// ★★★ THE DEFERRED REDACTION — 2026-09-05, `Pass 250.2`
//
// This section REPLACES the collapsing `apply_into_session` that stood here
// from 2026-09-04 (evening) until this pass. See §1.0 for why it is a
// replacement rather than a sibling; the short form is that two apply routes
// with different undo semantics, on one dialog, on the one operation that
// cannot be undone, is a choice the operator would have to understand in order
// to make it safely.
// ===========================================================================

/// **Which half of the staging transaction an action carries.**
///
/// An enum rather than two `Action` variants, and rather than a `bool`, for the
/// reason [`ResidualAcknowledgement`] gives one screen up: `stage(doc, true)`
/// at a call site says nothing, and the two values here are opposite acts on
/// the one operation in this program that cannot be undone once it reaches a
/// file.
///
/// ★ It lives in this module rather than beside the `Action` enum because the
/// vocabulary is this module's. `crate::app::actions::action` carries the
/// variant and points here, which is that file's own R2 rule — it is 1,500
/// lines of one enum and the reasoning goes next to the mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staging {
    /// Arm the removal: it happens at the next save, and until then nothing
    /// changes. [`stage_into_session`].
    Stage,
    /// Disarm it. [`cancel_staged_redaction`].
    ///
    /// ★★★ This exists because **a stageable operation that cannot be
    /// un-staged is a trap.** The collapsing route it replaces had no Cancel
    /// and needed none — there was nothing to cancel, the removal had already
    /// happened — and the moment the removal became a thing the document
    /// *carries* rather than a thing it *underwent*, an operator who changed
    /// his mind had no way out but to close the document and lose his edits.
    Cancel,
}

/// What [`stage_into_session`] armed, once it had armed it.
///
/// Distinct from [`PreparedRedaction`] and deliberately **not** a variant of
/// it: that type's whole shape is *"finished bytes nobody has written yet"*,
/// and there are no bytes here at all. The engine's staging verb runs the
/// removal to produce its preview and then throws the result away; what is left
/// is a flag on the session and the numbers below.
#[derive(Debug)]
pub struct StagedRedaction {
    /// The engine's **preview** report — what a save would remove, per carrier,
    /// plus its own disclosed residuals.
    ///
    /// ★ A preview and not a receipt, and the distinction is load-bearing
    /// enough that the engine states it in `apply_redactions_deferred`'s own
    /// doc comment: the actual removal re-runs at save over the **then**-current
    /// state, so an edit made in between changes what is removed. That is why
    /// [`save_applying_pending`] proves the bytes against the report the SAVE
    /// produced rather than against this one.
    pub report: RedactionReport,
    /// ★★★ **How many undo steps this did NOT destroy.**
    ///
    /// `EditSession::undo_depth()`, read after the call — which is safe here
    /// and was not safe before, and that inversion is the whole of what
    /// `Pass 250.2` bought. The collapsing route had to read the depth
    /// **before** the call because the call emptied the log; this one reads it
    /// after, because reading it after is the assertion: a build that had
    /// silently gone back to collapsing would report `0` here on every run.
    ///
    /// It is on the struct rather than derived at the call site so that
    /// `tests::staging_preserves_the_undo_log` and the trace line read the same
    /// number, and so that a regression shows up as a count rather than as a
    /// missing sentence.
    pub undo_depth_preserved: usize,
}

/// **Stage every `/Redact` mark for removal AT SAVE, touching nothing.**
///
/// `OPERATOR_REQUESTS.md` O125, in the shape the operator asked for and the
/// engine could not express until `Pass 250.2`:
///
/// > *"why does it have to save to a new file right away? Why can't it just
/// > wait on saving until I choose to save over the existing file or save as a
/// > new file?"*
///
/// **Nothing is written and nothing is removed.** The session's base, its
/// overlay and its entire undo/redo history are left exactly as they were; one
/// flag is set, and from then until the redaction is saved or cancelled the
/// engine refuses both ordinary save modes by name and
/// [`save_applying_pending`] is the only way bytes leave.
///
/// # ★★★ 1. What this buys, stated as the cost it removes
///
/// The route it replaces (`Pass 250.1`'s `EditSession::apply_redactions`)
/// **finalized**: it collapsed the session onto a clean redacted base and
/// cleared the whole undo log — not only the redaction, and not only the steps
/// that touched the redacted region. The operator accepted that, in writing,
/// with a *"for now"* attached to it, and this shell disclosed the step count
/// above the confirm control because of the *for now*.
///
/// There is no step count to disclose any more. Undo works across the staging;
/// the operator can undo the marks themselves, or edit on and undo back past
/// the moment he pressed the button, and [`cancel_staged_redaction`] takes the
/// staging off without touching anything else.
///
/// # ★★ 2. Why nothing is proven here, and where the proof went
///
/// `apply_redactions_deferred` runs the removal only to compute its preview and
/// **discards the bytes** (`pdfcer-core/src/edit.rs:8517`). There is therefore
/// no buffer for [`proof`] to sweep, and this function does not invent one — it
/// would have to call `save_applying_redaction` a second time to get one, which
/// is a second full rewrite of the document to prove something about bytes
/// nobody will ever write.
///
/// The proof lives at the save instead, in [`save_applying_pending`], over the
/// exact buffer that is one statement from the file system. That is §2.2's rule
/// unchanged — *"the write proves it"*, rather than *"the constructor proved
/// it"* — and it is why nothing on this path says **verified**:
/// [`crate::text::redact::staged_into_document`] says what *will* be removed,
/// and [`crate::text::redact::saved_applying_redaction`] is the sentence that
/// earns the word, after the sweep, about a file that exists.
///
/// # ★ 3. The page does not change, and the operator is told so
///
/// §1.0.3. A screenshot one frame after this returns is identical to one taken
/// a frame before it: the content is still drawn, the `/Redact` marks are still
/// drawn, and this shell adds no badge, tint or provisional layer to mark its
/// own pending state (rule 4). The disclosure is off-canvas, in words, on the
/// edit-disclosure row the funnel writes.
///
/// # Errors
///
/// [`RedactApplyRefusal`]. Every variant means the session was **not touched**
/// and no redaction is staged — the engine's own guarantee (*"on any error the
/// pending flag is NOT set"*), not this function's inference.
pub fn stage_into_session(
    session: &mut EditSession,
) -> Result<StagedRedaction, RedactApplyRefusal> {
    // Idempotence, by refusal rather than by silence. Staging twice is not an
    // error the engine would report — the flag is already set and the second
    // call would simply run the removal again for a preview nobody asked for —
    // so the shell refuses by name and the dialog says which state the document
    // is in.
    if session.has_pending_redaction() {
        return Err(RedactApplyRefusal::AlreadyStaged);
    }
    // Same census, same graph, same reason as `prepare_redaction_apply`: the
    // marks that matter are the ones the operator just made, and the base
    // revision by construction does not have them.
    if redact::count_redaction_marks(&session.graph()) == 0 {
        return Err(RedactApplyRefusal::NothingToApply);
    }

    // ★★★ The engine's staging verb — one of the four calls `sealed` pins to
    // this file. See §2.4.
    let report = session.apply_redactions_deferred().map_err(map_refusal)?;

    // ★ Read AFTER the call, and that is the assertion rather than an
    // afterthought. The route this replaces had to read the depth before,
    // because the call destroyed it; a build that had silently gone back to
    // collapsing would report 0 here, and `tests` would say so.
    let undo_depth_preserved = session.undo_depth();

    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // `undo_kept=` is on the line for the reason the collapsing route's
            // `undo_cleared=` was: it is the one consequence of this route that
            // has no equivalent on the write-now routes, and it is the field
            // that tells a correct build from a regression to the old verb. A
            // build that collapsed would emit an otherwise identical line with
            // `undo_kept=0`.
            //
            // `verified=` is deliberately ABSENT, unlike every sibling trace in
            // this module. Nothing was proven here and there is nothing to
            // prove — see §2 — and a `verified=` field carrying a placeholder
            // would be read by a harness as a proof that ran.
            "redact-staged marks={} pages={} glyphs={} streams={} undo_kept={}",
            report.marks_applied,
            report.pages_redacted,
            report.glyphs_removed,
            report.content_streams_rewritten,
            undo_depth_preserved,
        )
    });

    Ok(StagedRedaction {
        report,
        undo_depth_preserved,
    })
}

/// **Take a staged redaction back off.**
///
/// The other half of [`Staging`], and the control that stops staging from being
/// a trap. The session was never mutated by the staging, so this clears one
/// flag and nothing else changes: the marks are still there, the content is
/// still there, undo is where it was, and the ordinary save modes start working
/// again.
///
/// It returns nothing because there is nothing to report. The engine's verb is
/// `const fn cancel_pending_redaction(&mut self)` (`edit.rs:8542`) and is
/// idempotent, so a cancel on a document with nothing staged is a no-op rather
/// than an error — which is the right shape for a control a caller may reach
/// from a stale frame.
///
/// ★ **The caller owes one thing beside this call**: clearing
/// `OpenDoc::redaction_absence_claims`. Those strings are the shell's statement
/// that *every file it writes for this document has this text removed from it*,
/// and after a cancel that statement is false — leaving them set would make the
/// next ordinary save refuse itself, correctly, over a removal the operator
/// deliberately called off. `crate::app::actions::redact` does it in the same
/// arm, one line below, and its comment says so.
pub const fn cancel_staged_redaction(session: &mut EditSession) {
    session.cancel_pending_redaction();
}

/// **Perform a staged redaction and hand back proven bytes — the only save
/// that succeeds while one is staged.**
///
/// `crate::app::save::write_copy` calls this instead of
/// `EditSession::to_incremental_bytes` whenever
/// `EditSession::has_pending_redaction()` is true, which is what stops all three
/// of this shell's save verbs from failing by name the moment a redaction is
/// armed.
///
/// # ★★★ 1. This is the boundary, and the proof is at it
///
/// The engine's `save_applying_redaction(&self, ..)` (`edit.rs:8569`) runs the
/// removal over the session's **current** state and returns single-revision
/// bytes with the content already gone. That is a guarantee about somebody
/// else's code, and this shell's standing posture — §2.2, and the whole of
/// [`PreparedRedaction::write_to`] — is that a guarantee must not depend on how
/// the value was constructed. So the decoded-stream sweep runs here, over the
/// buffer the caller is about to write, before the caller can see it.
///
/// ★ **Against the report the SAVE produced, not the one staging predicted.**
/// The engine is explicit that the removal re-runs over the then-current state,
/// so if the operator edited between the staging and the save, the two reports
/// differ — and the claims that are true of a set of bytes are the ones the
/// removal that produced *those* bytes made. Proving against the stale preview
/// would refuse a legitimate save the day an operator undid one mark of three.
///
/// # ★★ 2. It takes `&self`, and that is the feature
///
/// The session is not mutated, so the operator's undo history survives the
/// save: he can save, keep editing, undo back past the save, and save again.
/// The redaction stays staged across all of it — `save_applying_redaction` does
/// not clear the flag — which means every subsequent save applies it too, and
/// the ordinary save modes stay refused until he cancels. That is stated in
/// `crate::text::redact::saved_applying_redaction` rather than left to be
/// discovered, because *"I saved it, so it is done"* is exactly the assumption
/// this feature must not let stand.
///
/// # Errors
///
/// [`RedactApplyRefusal`], and every variant means **no bytes are returned** so
/// no file can be written from them. `NothingToApply` is the reachable one and
/// it has a specific cause worth naming: the operator staged a redaction and
/// then undid the marks. The remedy is Cancel, and
/// `crate::text::redact::save_refused_message` names it.
pub fn save_applying_pending(
    session: &EditSession,
    options: &SaveOptions,
) -> Result<(Vec<u8>, RedactionReport), RedactApplyRefusal> {
    // ★★★ The engine's save-applying verb — one of the four calls `sealed`
    // pins to this file, and the only one that produces bytes anybody writes.
    let (bytes, report) = session
        .save_applying_redaction(options)
        .map_err(map_refusal)?;
    // ★ §2.2's proof, moved to the only place the deferred route can still make
    // it: between the buffer and the caller's syscall.
    if let Err(survivors) = prove_saved_bytes(&bytes, &report.redacted_text) {
        return Err(RedactApplyRefusal::VerificationFailed { survivors });
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // `verified=true` is unconditional and that is honest rather than
            // vacuous: the only way to reach this line is through the refusal
            // above, so a build in which the proof had been removed would emit
            // this line with the field still saying true — which is why the
            // field is `claims=` beside it. A proof that checked NOTHING reads
            // as `claims=0`, and a reader of a trace can tell the two apart.
            "redact-save-applied marks={} pages={} glyphs={} streams={} claims={} bytes={} \
             verified=true",
            report.marks_applied,
            report.pages_redacted,
            report.glyphs_removed,
            report.content_streams_rewritten,
            report.redacted_text.len(),
            bytes.len(),
        )
    });
    Ok((bytes, report))
}

/// The one mapping from the engine's [`RedactError`] to this module's refusal
/// taxonomy.
///
/// Free rather than repeated in each of the three call sites, because three
/// copies of a `match` over an error enum is how one of them comes to be
/// missing an arm — and the arm it would be missing is `NothingToApply`, which
/// is the one refusal the operator can actually reach. One table serves all
/// three, so `crate::text::redact::refusal_message` needs no second one.
fn map_refusal(err: RedactError) -> RedactApplyRefusal {
    match err {
        // A write failure is the same class of refusal as a failed
        // materialisation: the full rewrite did not happen.
        RedactError::Write(inner) => RedactApplyRefusal::FullRewriteUnavailable {
            reason: inner.to_string(),
        },
        RedactError::NothingToApply => RedactApplyRefusal::NothingToApply,
        other => RedactApplyRefusal::CoreRefused {
            reason: other.to_string(),
        },
    }
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
///
/// # ★★★ `verification` is an `Option` since 2026-09-05, and the `None` is a
/// statement rather than a convenience
///
/// The collapsing route this replaced proved its own output and always had an
/// [`AbsenceVerification`] to hand. [`stage_into_session`] has none and cannot
/// have one — the engine's staging verb discards the bytes (§1.0.1) — so the
/// caller passes `None`, and what comes back is the count of the residuals the
/// **engine's report** discloses, with the shell's own raw-byte residuals
/// simply absent from it.
///
/// Passing `Some(&AbsenceVerification::default())` would have compiled, read
/// identically at the call site, and told the operator that a sweep had run and
/// found nothing. It has not run. `None` is the difference between *"zero
/// residuals were found"* and *"nobody has looked yet"*, which on this surface
/// is the difference the whole feature turns on.
#[must_use]
pub fn residual_count(
    report: &RedactionReport,
    verification: Option<&AbsenceVerification>,
) -> usize {
    use pdfcer_core::redact::CarrierAction;
    report
        .carriers
        .iter()
        .filter(|c| c.action == CarrierAction::DisclosedNotScrubbed)
        .count()
        + usize::from(report.marks_retained > 0)
        + usize::from(report.vector_paths_intersecting > 0)
        + usize::from(report.vector_clips_kept > 0)
        + verification.map_or(0, |v| v.residuals.len())
}

/// **The absence proof, run over bytes that are one syscall from a file.**
///
/// §2.2's argument, moved to the one place the deferred route can still make
/// it. On the write-now route the proof sits inside
/// [`PreparedRedaction::write_to`], between the buffer and the syscall. The
/// deferred route has no such buffer at staging time — the bytes are built by
/// `crate::app::save` at save time, minutes later, possibly after further
/// edits, possibly by a different save verb — so the check has to be made
/// available *to* that module rather than owned by this one.
///
/// `claims` is [`pdfcer_core::redact::RedactionReport::redacted_text`]. **Which
/// report it comes from depends on which writer produced the bytes**, and the
/// rule is one sentence: *the claims that are true of a set of bytes are the
/// ones made by the removal that produced them.*
///
/// | writer | `claims` | why |
/// |---|---|---|
/// | [`save_applying_pending`] | the report that call returned | the removal re-ran over the current state; an edit since staging changes what came out |
/// | `EditSession::to_incremental_bytes` | `OpenDoc::redaction_absence_claims` | no removal ran at all, so the standing claim on the document is the only one there is |
///
/// An empty slice is the overwhelmingly common case (no redaction has been
/// staged on this document) and returns `Ok` without decoding anything, so an
/// ordinary save pays nothing.
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
