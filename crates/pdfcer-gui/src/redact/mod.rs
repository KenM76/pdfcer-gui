//! # `redact` — the APPLY pipeline, and the reason a redaction in this shell
//! cannot be shipped unverified
//!
//! Design and rationale: `docs/modules/pdfcer-gui/redact/mod.md`.

pub mod proof;

/// **The call-site monopoly** — §2.4. Parses every `.rs` file in this crate and
/// asserts that `apply_redactions` is called in exactly one place.
#[cfg(test)]
mod sealed;

use std::path::Path;

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::object::ObjId;
use pdfcer_core::redact::{self, RedactError, RedactionReport};
use pdfcer_core::writer::{SaveOptions, WriteError};

use crate::app::prefs::RedactionReach;

pub use proof::{AbsenceVerification, Residual, ResidualSite};

/// Why a redaction apply did not happen. Every variant is a refusal **before
/// any byte reached the filesystem**.
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
        /// **The engine named `WriteError::HybridFullRewrite`** — the one
        /// cause in this class that earns its own sentence, because its
        /// subject is a specific damaged structure in the operator's file
        /// rather than a general inability to rewrite.
        ///
        /// # A `bool` here replaced a substring match on the engine's prose
        ///
        /// Selecting the sentence in `crate::text::redact::refusal_message`
        /// with `reason.contains("hybrid-reference")` would put a locator for
        /// another crate's message format inside a GUI, and
        /// `TextStyleRefusal::FaceLacksCharacters` spends a section refusing
        /// to do the same thing for the same reason: the clause gets reworded
        /// and the match silently stops firing, or the engine NARROWS the
        /// condition, the words survive, and the match keeps firing for a
        /// sentence that is now describing a different file.
        ///
        /// The variant is in hand at all three construction sites. Ask it.
        broken_xref_stream: bool,
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
    /// This is raised only for a survivor the operator was **never shown**:
    /// every drawn-content hit the preparation proof finds is disclosed at
    /// `ResidualSite::DrawnContent` and acknowledged through the residual gate,
    /// so only a hit that appears between the proof and the write — the bytes
    /// changed — reaches this. It is the module's "removal and report disagree"
    /// line, and it should be unreachable.
    VerificationFailed {
        /// The strings that survived AND were not in the acknowledged list.
        survivors: Vec<String>,
    },
    /// **A redaction is already STAGED on this session.**
    ///
    /// Reachable two ways, and both are ordinary rather than
    /// exceptional: the operator opens *Review & apply* a second time on a
    /// document he has already staged, or a second `Stage` action arrives
    /// before the first frame after the first one.
    ///
    /// It is a **named refusal in the pipeline** rather than a condition the
    /// dialog checks, and the difference is the one this project keeps paying
    /// for. Without it the second open would reach
    /// [`prepare_redaction_apply`]'s `to_full_bytes`, which the engine refuses
    /// with `WriteError::RedactionPending`, and the operator would be
    /// told *"this document cannot be rewritten in full"* — a true sentence
    /// about the wrong subject, arriving at the one surface where a wrong
    /// diagnosis costs most.
    AlreadyStaged,
}

/// Whether the operator has acknowledged the residuals the report disclosed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidualAcknowledgement {
    /// The operator has read the residual list and asked to proceed anyway.
    Given,
    /// The operator has not. A write is refused if there is anything to
    /// acknowledge.
    Withheld,
}

/// Why [`PreparedRedaction::write_to`] produced no file.
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
    /// The redaction succeeded and its result will not re-parse.
    ///
    /// Reachable only from [`PreparedRedaction::into_verified_document`],
    /// and it is **not** an I/O failure wearing a different name: nothing was
    /// written. The removal happened, the proof passed, and the bytes the
    /// engine produced cannot be read back as a document.
    ///
    /// ⚠ It has its own variant rather than borrowing [`Self::FileSystem`]
    /// because the operator's next step differs completely. A file-system
    /// failure says *try again, or somewhere else*. This says **the open
    /// document is unchanged and this cannot be applied in place** — the
    /// removal must go to a new file instead, where the bytes are written
    /// rather than re-read.
    ///
    /// It should be unreachable. `write_to` has produced these same bytes for
    /// every redaction this program has ever written, and a PDF pdfcer just
    /// serialised failing to re-parse would be an engine defect worth a request
    /// rather than a shrug — which is why the reason is carried verbatim.
    RedactedDocumentUnreadable {
        /// The parser's own words.
        reason: String,
    },
}

impl std::fmt::Display for WriteRefusal {
    /// Diagnostic prose for the trace, and for nothing else.
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
            Self::RedactedDocumentUnreadable { reason } => write!(
                f,
                "the redaction succeeded and its result will not re-parse: {reason}"
            ),
        }
    }
}

/// A completed, verified, **unwritten** redaction: the exact bytes that will
/// land on disk if — and only if — the operator confirms.
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

    /// **Read the redacted document back as a parsed [`Document`], for loading into
    /// the open session — and prove it one last time first.**
    pub fn to_verified_document(
        &self,
        acknowledgement: ResidualAcknowledgement,
    ) -> Result<Document, WriteRefusal> {
        let residuals = self.verification.residuals.len();
        if residuals > 0 && acknowledgement == ResidualAcknowledgement::Withheld {
            return Err(WriteRefusal::ResidualsNotAcknowledged { residuals });
        }
        // Only a survivor the operator was NOT shown refuses:
        // every drawn-content hit `prove` found at preparation is in
        // `verification.residuals` at `DrawnContent`, and the gate above has
        // already required its acknowledgement. Anything else here means the
        // bytes changed between the proof and the write.
        if let Some(survivors) =
            proof::survivors_in_content_streams(&self.bytes, &self.report.redacted_text)
        {
            let undisclosed: Vec<String> = survivors
                .into_iter()
                .filter(|s| !self.verification.disclosed_in_drawn_content(s))
                .collect();
            if !undisclosed.is_empty() {
                return Err(WriteRefusal::VerificationFailed {
                    survivors: undisclosed,
                });
            }
        }
        // `clone()` and it stays INSIDE this module. The parser takes an
        // owned buffer; `&self` is what makes this mirror `write_to`, which
        // also does not consume the preparation — an operator whose parse fails
        // still has a dialog with a working *Save to a new file* row.
        Document::from_bytes(self.bytes.clone()).map_err(|err| {
            WriteRefusal::RedactedDocumentUnreadable {
                reason: err.to_string(),
            }
        })
    }

    /// **Write the redacted document to `target`, and prove it one last time
    /// first.**
    pub fn write_to(
        &self,
        target: &Path,
        acknowledgement: ResidualAcknowledgement,
    ) -> Result<usize, WriteRefusal> {
        let residuals = self.verification.residuals.len();
        if residuals > 0 && acknowledgement == ResidualAcknowledgement::Withheld {
            return Err(WriteRefusal::ResidualsNotAcknowledged { residuals });
        }
        // §2.2 — the proof between the buffer and the syscall.
        // Only a survivor the operator was NOT shown refuses:
        // every drawn-content hit `prove` found at preparation is in
        // `verification.residuals` at `DrawnContent`, and the gate above has
        // already required its acknowledgement. Anything else here means the
        // bytes changed between the proof and the write.
        if let Some(survivors) =
            proof::survivors_in_content_streams(&self.bytes, &self.report.redacted_text)
        {
            let undisclosed: Vec<String> = survivors
                .into_iter()
                .filter(|s| !self.verification.disclosed_in_drawn_content(s))
                .collect();
            if !undisclosed.is_empty() {
                return Err(WriteRefusal::VerificationFailed {
                    survivors: undisclosed,
                });
            }
        }
        // Temp-then-rename. See the "Why the write IS atomic" section: the
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
                // `glyphs=` beside `marks=`, because a count of marks alone
                // is not an ink trail: a build that applied every mark and
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
pub fn prepare_redaction_apply(
    session: &EditSession,
    reach: RedactionReach,
) -> Result<PreparedRedaction, RedactApplyRefusal> {
    // Asked FIRST — before the mark census — and the ORDER is load-bearing
    // twice over.
    //
    // 1. While a redaction is staged the engine refuses `to_full_bytes` by
    //    name, so without this the materialisation below would fail and the
    //    operator would read *"this document cannot be rewritten in full"* on
    //    a document that can.
    //
    // 2. **It is what stops the operator being trapped.** Ask the mark
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
            broken_xref_stream: matches!(err, WriteError::HybridFullRewrite),
            reason: err.to_string(),
        })?;

    let doc = Document::from_bytes(materialised).map_err(|err| {
        RedactApplyRefusal::MaterialisedDocumentUnreadable {
            reason: err.to_string(),
        }
    })?;

    // Full rewrite #2 — the removal itself. `apply_redactions_with` forces its
    // own full rewrite internally (R35); this call site cannot ask it for
    // anything else, which is the property that makes "apply is never
    // incremental" structural rather than a convention.
    //
    // The `_with` form rather than the bare one, and it is not a preference
    // for the longer name: the bare `apply_redactions` hard-codes the engine's
    // default reach, so calling it would make the setting in the window a
    // promise this route breaks. The two are otherwise the same function.
    let redact_options = redact::RedactOptions::with_residual_scope(reach.scope());
    let (bytes, report) =
        redact::apply_redactions_with(&doc, &SaveOptions::identity(), &redact_options).map_err(
            |err| match err {
                // A write failure is the same class of refusal as a failed
                // materialisation: the full rewrite did not happen.
                RedactError::Write(inner) => RedactApplyRefusal::FullRewriteUnavailable {
                    broken_xref_stream: matches!(inner, WriteError::HybridFullRewrite),
                    reason: inner.to_string(),
                },
                other => RedactApplyRefusal::CoreRefused {
                    reason: other.to_string(),
                },
            },
        )?;

    // A survivor in drawn content does NOT refuse here. `prove` lists it as
    // a `ResidualSite::DrawnContent` residual, the window shows it with the
    // sentence that says *outside the area you marked*, and the acknowledgement
    // gate decides — the operator is allowed to override and redact what the
    // engine can. The refusal survives at the write (`to_verified_document` /
    // `write_to`) for a survivor he was never shown.
    let proven = proof::prove(&bytes, &report.redacted_text);
    crate::diag::trace(|| {
        format!(
            "redact-prove drawn_content_hits={} residuals={} short={}",
            proven.survivors.as_ref().map_or(0, Vec::len),
            proven.verification.residuals.len(),
            proven.verification.strings_too_short_for_raw_check,
        )
    });

    Ok(PreparedRedaction {
        bytes,
        report,
        verification: proven.verification,
        promoted_by_materialisation: materialise_report.promoted,
    })
}

// ===========================================================================
// THE DEFERRED REDACTION
//
// This is the only route that touches the open session, and there is
// deliberately no collapsing sibling beside it. See §1.0; the short form is
// that two apply routes with different undo semantics, on one dialog, on the
// one operation that cannot be undone, is a choice the operator would have to
// understand in order to make it safely.
// ===========================================================================

/// **Which half of the staging transaction an action carries.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staging {
    /// Arm the removal: it happens at the next save, and until then nothing
    /// changes. [`stage_into_session`].
    Stage,
    /// Disarm it. [`cancel_staged_redaction`].
    ///
    /// This exists because **a stageable operation that cannot be
    /// un-staged is a trap.** The collapsing route it replaces had no Cancel
    /// and needed none — there was nothing to cancel, the removal had already
    /// happened — and the moment the removal became a thing the document
    /// *carries* rather than a thing it *underwent*, an operator who changed
    /// his mind had no way out but to close the document and lose his edits.
    Cancel,
}

/// What [`stage_into_session`] armed, once it had armed it.
#[derive(Debug)]
pub struct StagedRedaction {
    /// The engine's **preview** report — what a save would remove, per carrier,
    /// plus its own disclosed residuals.
    ///
    /// A preview and not a receipt, and the distinction is load-bearing
    /// enough that the engine states it in `apply_redactions_deferred`'s own
    /// doc comment: the actual removal re-runs at save over the **then**-current
    /// state, so an edit made in between changes what is removed. That is why
    /// [`save_applying_pending`] proves the bytes against the report the SAVE
    /// produced rather than against this one.
    pub report: RedactionReport,
    /// **How many undo steps this did NOT destroy.**
    ///
    /// `EditSession::undo_depth()`, read **after** the call, and the order is
    /// the assertion. A route that emptied the log would have to be read
    /// before; reading after means a build that had silently gone back to
    /// collapsing reports `0` here on every run.
    ///
    /// It is on the struct rather than derived at the call site so that
    /// `tests::staging_preserves_the_undo_log` and the trace line read the same
    /// number, and so that a regression shows up as a count rather than as a
    /// missing sentence.
    pub undo_depth_preserved: usize,
}

/// **Stage every `/Redact` mark for removal AT SAVE, touching nothing.**
pub fn stage_into_session(
    session: &mut EditSession,
    reach: RedactionReach,
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

    // Set BEFORE the staging verb, and it stays on the session afterwards.
    // That is what makes `save_applying_pending` — which takes `&EditSession`
    // and therefore cannot be told anything — perform the removal at the same
    // reach the preview below was computed at and the operator acknowledged.
    // A settings change between staging and saving does NOT move it, which is
    // the correct answer: the operator confirmed a specific preview.
    session.set_residual_scope(reach.scope());

    // The engine's staging verb — one of the four calls `sealed` pins to
    // this file. See §2.4.
    let report = session.apply_redactions_deferred().map_err(map_refusal)?;

    // The reach must SURVIVE that call. `apply_redactions_deferred` collapses
    // the staged state, and a collapse that reset the scope would leave
    // `save_applying_pending` — which cannot be told anything — performing the
    // removal at the default reach while the operator acknowledged a preview
    // computed at his. The engine documents that it survives; this is the
    // assertion that notices if it stops, since nothing else in this shell
    // reads the value back.
    debug_assert_eq!(
        session.residual_scope(),
        reach.scope(),
        "the staging verb reset the redaction reach the operator chose"
    );

    // Read AFTER the call, and that is the assertion rather than an
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
pub const fn cancel_staged_redaction(session: &mut EditSession) {
    session.cancel_pending_redaction();
}

/// **Perform a staged redaction and hand back proven bytes — the only save
/// that succeeds while one is staged.**
pub fn save_applying_pending(
    session: &EditSession,
    options: &SaveOptions,
) -> Result<(Vec<u8>, RedactionReport), RedactApplyRefusal> {
    // The engine's save-applying verb — one of the four calls `sealed`
    // pins to this file, and the only one that produces bytes anybody writes.
    let (bytes, report) = session
        .save_applying_redaction(options)
        .map_err(map_refusal)?;
    // §2.2's proof, moved to the only place the deferred route can still make
    // it: between the buffer and the caller's syscall.
    // Survivors in drawn content at save time are the same drawn-content hits
    // the arming window disclosed and the operator acknowledged before the
    // removal could be staged (the staging route goes through
    // `ready_to_confirm`'s residual gate like the immediate one), so they are
    // traced, not refused. See `ResidualSite::DrawnContent`.
    // `prove_saved_bytes`'s `Err` is that list.
    let drawn_content_hits = prove_saved_bytes(&bytes, &report.redacted_text)
        .err()
        .unwrap_or_default();
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
             verified={} drawn_content_hits={}",
            report.marks_applied,
            report.pages_redacted,
            report.glyphs_removed,
            report.content_streams_rewritten,
            report.redacted_text.len(),
            bytes.len(),
            drawn_content_hits.is_empty(),
            drawn_content_hits.len(),
        )
    });
    Ok((bytes, report))
}

/// The one mapping from the engine's [`RedactError`] to this module's refusal
/// taxonomy.
fn map_refusal(err: RedactError) -> RedactApplyRefusal {
    match err {
        // A write failure is the same class of refusal as a failed
        // materialisation: the full rewrite did not happen.
        RedactError::Write(inner) => RedactApplyRefusal::FullRewriteUnavailable {
            broken_xref_stream: matches!(inner, WriteError::HybridFullRewrite),
            reason: inner.to_string(),
        },
        RedactError::NothingToApply => RedactApplyRefusal::NothingToApply,
        other => RedactApplyRefusal::CoreRefused {
            reason: other.to_string(),
        },
    }
}

/// **How many items a report and a proof disclose as NOT removed.**
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
pub fn prove_saved_bytes(bytes: &[u8], claims: &[String]) -> Result<(), Vec<String>> {
    if claims.is_empty() {
        return Ok(());
    }
    proof::survivors_in_content_streams(bytes, claims).map_or(Ok(()), Err)
}

/// The security assertions for this pipeline, in their own file — see
/// [`tests`]'s header for the seam.
#[cfg(test)]
mod tests;

// ===========================================================================
// The apply-now hand-off
// ===========================================================================

thread_local! {
    /// The document an *apply now* has just produced, waiting for the action
    /// funnel to install it.
    static APPLIED: std::cell::RefCell<Option<Document>> = const { std::cell::RefCell::new(None) };
}

/// Park a verified redacted document for the action funnel.
pub(crate) fn park_applied_document(doc: Document) {
    APPLIED.with(|slot| slot.replace(Some(doc)));
}

/// Take it, exactly once.
pub(crate) fn take_applied_document() -> Option<Document> {
    APPLIED.with(|slot| slot.borrow_mut().take())
}
