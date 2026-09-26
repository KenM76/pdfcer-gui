//! # `app::save` — writing a copy of the open document to a file the operator
//! names
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/save.md`.

use std::path::{Path, PathBuf};

use crate::app::files::{self, Picked};
use crate::app::state::OpenDoc;
use crate::dialogs::signature::{Disclosure, impact_of_saving};

use pdfcer_gui_base::saveoutcome as outcome;

use outcome::{SaveError, Written};

/// **The sentence this save owes about the document's digital signatures, if
/// it owes one.**
fn signature_note(doc: &OpenDoc) -> Option<String> {
    let (disclosure, count) = impact_of_saving(doc);
    match disclosure {
        Disclosure::Silent => None,
        Disclosure::NoteAfterSaving => Some(crate::text::signature::preserved_note(count)),
        Disclosure::WarnBeforeSaving(_) => Some(crate::text::signature::invalidated_note(count)),
    }
}

/// **Ask where the copy goes, write it there, and say what happened.**
pub fn save_as(doc: &OpenDoc) -> Option<std::path::PathBuf> {
    let suggested = suggested_path(doc);
    match files::pick_save_path(&suggested, crate::text::files::save_as_dialog_title()) {
        Picked::Path(target) => {
            if write_and_report(doc, &target) {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // The OLD path is on the line as well as the new one,
                    // and that is the point of tracing this at all. "The
                    // document moved" and "a copy was written" produce the
                    // same `save-copy` line today; only the pair says which
                    // file the next Ctrl+S will reach.
                    format!(
                        "save-as from={} to={}",
                        doc.path.display(),
                        target.display()
                    )
                });
                Some(target)
            } else {
                None
            }
        }
        Picked::Cancelled => None,
        Picked::Unavailable => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "save-as-unavailable reason=no-picker-in-this-build".to_owned()
            });
            None
        }
    }
}

pub fn save_copy(doc: &OpenDoc) -> bool {
    let suggested = suggested_path(doc);
    match files::pick_save_path(&suggested, crate::text::files::save_copy_dialog_title()) {
        Picked::Path(target) => write_and_report(doc, &target),
        // A cancelled save is a complete, correct, uninteresting outcome.
        Picked::Cancelled => false,
        Picked::Unavailable => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "save-copy-unavailable reason=no-picker-in-this-build".to_owned()
            });
            false
        }
    }
}

/// **Does the active document have a real file behind it?**
#[must_use]
pub fn has_a_file(doc: &OpenDoc) -> bool {
    doc.path.is_file()
}

/// **Does this document have edits that are not on disk?** — the one
/// question, in the one place.
#[must_use]
pub fn has_unsaved_edits(doc: &OpenDoc) -> bool {
    (doc.session.is_modified() || doc.session.has_pending_redaction())
        && doc.edit_epoch != doc.saved_epoch
}

/// **Save. In place. The one every other program has.**
pub fn save_in_place(doc: &OpenDoc) -> bool {
    let target = doc.path.clone();
    let temporary = target.with_extension("pdfcer-tmp");
    // Asked before a byte moves — see [`signature_note`]'s for why the
    // engine documented this as a pre-save question, and why asking after
    // would return the same answer today and be wrong on principle.
    let signature = signature_note(doc);

    // Step 1 - materialise the whole replacement somewhere else on the same
    // volume. A failure here has touched nothing the operator owns.
    //
    // On a staged redaction the "replacement" is a single-revision full
    // rewrite with the marked content gone (§1.1), and the temp-then-rename
    // below matters more here than anywhere else in this module: the target is
    // the operator's own document, and it is the last remaining copy of the
    // content being removed.
    let written = match write_copy(doc, &temporary) {
        Ok(written) => written,
        Err(error) => {
            let _ = std::fs::remove_file(&temporary);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("save-in-place outcome=failed stage=write detail={error:?}")
            });
            // The operator-visible half, and the same sentence Save-a-copy
            // uses: a write that produced no file and no sentence is
            // indistinguishable from a control that does nothing.
            crate::app::status::decline::record_save_failure();
            redaction_refusal_note(doc, &error);
            return false;
        }
    };

    // Step 2 - the atomic act. A rename either happens or does not; on Windows
    // it fails outright if the target is open, which is precisely the guarantee
    // a truncating write does not give.
    if let Err(error) = std::fs::rename(&temporary, &target) {
        let _ = std::fs::remove_file(&temporary);
        crate::app::status::decline::record_save_failure();
        let _ = error;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The stage is in the line because the two failures mean different
            // things to whoever reads it: `write` means the bytes never
            // materialised, `rename` means they did and the swap was refused -
            // overwhelmingly "the file is open in another program".
            "save-in-place outcome=failed stage=rename".to_owned()
        });
        return false;
    }

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("save-in-place outcome=ok path={:?}", target)
    });
    // A receipt, not a celebration. The operator pressed a button and the
    // only observable change is that a marker disappeared from a tab - which is
    // a change you have to already know about to notice. One line naming the
    // file it went into, on the channel every other edit reports on.
    //
    // And, for a signed document, the sentence that receipt owes beside it.
    // `record_notes` rather than two `record_note` calls: the slot holds ONE
    // disclosure, so a second call replaces the first, and the sentence it
    // would have dropped would have been chosen by statement order rather than
    // by importance. The receipt leads because it answers *"did my save
    // happen"*, which is the question the operator actually pressed the button
    // to have answered; the signature sentence follows because it answers one
    // they did not know they had.
    let mut notes = vec![crate::text::files::saved_in_place(&target)];
    notes.extend(signature);
    crate::app::actions::record_notes(doc.edit_epoch, notes);
    // …and, when the save performed a staged redaction, the sentence that
    // says so — recorded AFTER `record_notes`, deliberately, so it is the
    // disclosure that stands.
    //
    // `record_note` replaces what is in the slot, and the ordering here is a
    // choice about which fact wins when only one can be shown. "Saved to
    // sheet.pdf" answers *"did my save happen"*, which the operator can also
    // read off the tab marker disappearing. "The content is out of the file,
    // the window still shows it, and the removal is still armed" answers three
    // questions he has no other way to answer, on the one operation that cannot
    // be undone. Rule 4 decides it and it points at the second.
    if let Written::RedactionApplied(report) = &written {
        redaction_receipt(doc, &target, report);
    }
    true
}

/// Write the copy and record the outcome on both channels.
///
fn write_and_report(doc: &OpenDoc, target: &Path) -> bool {
    // Before the write, for [`signature_note`]'s reason.
    let signature = signature_note(doc);
    match write_copy(doc, target) {
        // The staged-redaction save — §1.1. A different event with
        // different fields, so a different trace line and a different sentence.
        Ok(Written::RedactionApplied(report)) => {
            redaction_receipt(doc, target, &report);
            if let Some(note) = signature {
                crate::app::actions::record_note(doc.edit_epoch, note);
            }
            true
        }
        Ok(Written::Ordinary(report)) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // `appended=` beside `bytes=`, on the ink-trail rule:
                    // a build that writes a plain copy of the base file — no
                    // revision appended, the operator's edits silently absent
                    // — produces a file that opens, has the right page count
                    // and looks correct, and its trace line would be
                    // identical but for this one field. `identical=` is the
                    // same fact from the other side and is `true` exactly when
                    // nothing was edited.
                    //
                    // `epoch=` says WHICH revision was written, which is the
                    // only way a reader of a trace can tell a save that
                    // captured the operator's last edit from one that ran a
                    // frame too early.
                    //
                    // `path` is Debug-quoted, exactly as `open`'s is. A Windows
                    // path routinely contains a space, and a consumer splitting
                    // this line into `key=value` pairs would otherwise read
                    // `Files\a.pdf` as a field name and lose every field after
                    // it. `tools/ui-verify`'s parser honours double quotes;
                    // nothing else in the line needs them.
                    "save-copy path={:?} bytes={} appended={} objects={} verbatim={} \
                     reserialized={} promoted={} deleted={} identical={} delinearized={} \
                     epoch={} origin={:?}",
                    target,
                    report.bytes_written,
                    report.bytes_appended,
                    report.objects_written,
                    report.objects_verbatim,
                    report.objects_reserialized,
                    report.promoted.len(),
                    report.objects_deleted,
                    report.byte_identical,
                    report.delinearized,
                    doc.edit_epoch,
                    doc.origin,
                )
            });
            // The one sentence a successful save-a-copy is allowed to put
            // on the bar, and §5's *"no sentence is added"* ruling is not
            // being overturned by it.
            //
            // That ruling is about **narrating the act**: a status line saying
            // "a copy was written" would describe something whose whole
            // product is already visible in the operating system's own file
            // browser, at a path the operator typed a moment earlier. This is
            // not that. It is a fact about the **integrity of the file they
            // now have**, which appears nowhere — not on the canvas, not in
            // Explorer, and not in this shell's Signatures panel, which
            // reports what a document carries rather than what a save did to
            // it. Rule 4 governs, and it points the other way from §5.
            //
            // It is `None` for every unsigned document, so the common case
            // still adds nothing at all.
            if let Some(note) = signature {
                crate::app::actions::record_note(doc.edit_epoch, note);
            }
            true
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("save-copy-failed path={target:?} detail={error}")
            });
            // The operator-visible half. See §5: a write that produced no file
            // and no sentence is indistinguishable from a control that does
            // nothing.
            crate::app::status::decline::record_save_failure();
            redaction_refusal_note(doc, &error);
            // Mutually exclusive with the line above — one `SaveError` value
            // reaches both, and each is silent for the other's variant. See
            // `page_tree_refusal_note` for why it is a second function rather
            // than a second arm.
            page_tree_refusal_note(doc, &error);
            false
        }
    }
}

/// **The sentence a staged-redaction save owes, and why it is not the ordinary
/// receipt.**
fn redaction_receipt(doc: &OpenDoc, target: &Path, report: &pdfcer_core::redact::RedactionReport) {
    let residuals = crate::redact::residual_count(report, None);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // `path` is Debug-quoted, exactly as `save-copy`'s is: a Windows
            // path routinely contains a space.
            //
            // `still_staged=true` is unconditional and is the field worth
            // having. `save_applying_redaction` takes `&self` and does not
            // clear the flag, so the removal is armed again the instant this
            // returns — and a build that had started clearing it would emit an
            // otherwise identical line.
            "redact-saved path={:?} marks={} pages={} glyphs={} residuals={} epoch={} \
             still_staged=true",
            target,
            report.marks_applied,
            report.pages_redacted,
            report.glyphs_removed,
            residuals,
            doc.edit_epoch,
        )
    });
    crate::app::actions::record_note(
        doc.edit_epoch,
        crate::text::redact::saved_applying_redaction(
            &target.file_name().map_or_else(
                || target.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            ),
            report.marks_applied,
            report.pages_redacted,
            residuals,
        ),
    );
}

/// **The extra sentence a save refused *because of a staged redaction* owes.**
fn redaction_refusal_note(doc: &OpenDoc, error: &SaveError) {
    if let SaveError::RedactionRefused { refusal } = error {
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::redact::save_refused_message(refusal),
        );
    }
}

/// **The extra sentence a save refused by the page-tree guard owes.**
fn page_tree_refusal_note(doc: &OpenDoc, error: &SaveError) {
    let SaveError::PageTreeStale { audit } = error else {
        return;
    };
    let name = doc.path.file_name().map_or_else(
        || doc.path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    // Which refusal is owed — including the re-audit of the file on disk that
    // answers *"was it already like this when he opened it?"* — is decided by
    // `crate::pagetree`, beside the audit it reasons about; `text` words it.
    let origin = crate::pagetree::refusal_origin(audit, doc.stored_under());
    let sentence = crate::text::pagetree::refusal_sentence(&name, origin);
    crate::app::actions::record_note(doc.edit_epoch, sentence);
}

/// **Serialize the open document as an incremental update and write it to
/// `target`.**
fn write_copy(doc: &OpenDoc, target: &Path) -> Result<Written, SaveError> {
    // Through the funnel, not `SaveOptions::default()`.
    //
    // Two settings ride on this — the cross-reference entry line ending and the
    // trailing newline — and both change the bytes of the file the operator is
    // about to receive. A bare `::default()` here would honour neither, which is
    // a live setting rather than a formality: `xref_entry_eol`'s default is
    // what it is on an operator ruling, because a fixed form produces a
    // ten-thousand-byte diff on an unedited file.
    //
    // The producer policy is the funnel's, which is `Preserve` — carried over
    // from `identity()` rather than chosen, because what pdfcer writes into
    // `/Producer` is a decision about attribution rather than about bytes and
    // no setting governs it.
    use crate::app::settings::SettingsExt;
    let options = doc.settings.save_options();

    // THE FORK — see §1.1.
    //
    // Asked of the SESSION rather than of a flag this module keeps, for
    // `has_a_file`'s reason applied to a different question: a second source of
    // truth about whether a removal is armed would drift, and the direction it
    // would drift in is a save that quietly wrote the un-redacted document.
    //
    // There is no `else` that could fall back. While the flag is set, both
    // ordinary save modes return `WriteError::RedactionPending`, so a build
    // that did not fork here would not leak — it would stop being able to save
    // at all. That is the engine refusing rather than this module guarding, and
    // it is why the fork is a route rather than a gate.
    let (bytes, written, claims) = if doc.session.has_pending_redaction() {
        let (bytes, report) = crate::redact::save_applying_pending(&doc.session, &options)
            .map_err(|refusal| SaveError::RedactionRefused { refusal })?;
        // The claims that describe THESE bytes are the ones the removal that
        // produced them made — not the preview the staging recorded on the
        // document. The engine re-runs the removal over the current state, so
        // an operator who undid one mark of three between staging and saving
        // has a shorter list, and proving against the longer one would refuse a
        // legitimate save over a mark he deliberately took off.
        let claims = report.redacted_text.clone();
        (bytes, Written::RedactionApplied(Box::new(report)), claims)
    } else {
        let (bytes, report) = doc.session.to_incremental_bytes(&options)?;
        // The standing claim on the document. Empty on everything that has not
        // been staged, which is every ordinary save.
        (
            bytes,
            Written::Ordinary(report),
            doc.redaction_absence_claims.clone(),
        )
    };

    // THE ABSENCE PROOF, between the bytes and the syscall — the
    // shell's own, independent of the engine, on every save verb.
    //
    // `crate::redact::PreparedRedaction::write_to` makes this check one
    // statement from the write on the two destinations that produce a file
    // directly. The deferred destination — the default — arms the removal and
    // leaves the write to this function, minutes later, possibly after further
    // edits, through whichever save verb the operator reached for. So the proof
    // has to be made here or not at all, and "not at all" is the option our own
    // engine request ruled out in writing: *"the proof is not negotiable at
    // this end regardless of what the engine does."*
    //
    // What it is NOT. It is not a save gate on `has_applied_redaction()`,
    // which the engine asked us not to build. It is a sweep over the bytes,
    // whichever writer produced them, and it is expected to pass forever — a
    // check that is expected to pass is exactly the kind this project keeps
    // discovering was never wired, which is why `app::save::tests` falsifies
    // its bite rather than assuming it.
    //
    // It costs nothing on an ordinary save: `redaction_absence_claims` is
    // empty on every document that has not been staged, and `prove_saved_bytes`
    // returns without decoding a single stream.
    //
    // On the staged path it is deliberately the SECOND sweep of the same
    // bytes — `crate::redact::save_applying_pending` has already run one before
    // returning them. That is §2.2's rule twice rather than once, and the
    // reason is the same one it gives: the guarantee must not depend on how the
    // value was constructed, and this call site cannot see how it was.
    if let Err(survivors) = crate::redact::prove_saved_bytes(&bytes, &claims) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "save-refused-redaction-leak path={target:?} survivors={} of {}",
                survivors.len(),
                claims.len()
            )
        });
        return Err(SaveError::RedactionLeak { survivors });
    }

    // THE STRUCTURAL GUARD — the second proof this shell keeps
    // at this boundary, and it is here for the identical reason the first one
    // is. `crate::pagetree` carries the whole argument, the measured cost, and
    // the lesson; the three facts a reader of THIS function needs are:
    //
    // 1. **It is on the funnel, not on the delete-pages arm.** The defect is a
    //    writer's invariant, and `page-copy --cut` was measured producing
    //    byte-for-byte the same corruption as `delete-pages`. A guard on one
    //    verb would pass every other one straight through and would have to be
    //    remembered again for each verb added later.
    // 2. **It refuses; it does not repair.** `pdfcer-core` owns the page tree
    //    and is the only writer of `/Count`. A shell that patched the same key
    //    would be a second writer of one structure and the two would drift.
    // 3. **It cannot see the defect through this shell's own reader**, so it
    //    does not use it: `page_tree::pages` walks `/Kids` and reports a
    //    healthy 34-page document while the root still declares 36. The audit
    //    reads `/Count` raw and compares.
    //
    // Ungated, and NOT free: 1.78 ms on his 1.8 MB drawing set, 3.51 ms on
    // the 129,758-object CAD sheet — which on the first of those is MORE than
    // `to_incremental_bytes` cost to build the bytes it is checking. Measured
    // rather than assumed, and the first draft of this comment guessed and was
    // wrong. It stays on every save because a few milliseconds is invisible
    // inside a gesture that opens a file dialog and writes megabytes to disk.
    // `crate::pagetree` §9 has the table and the argument for why a *"has the
    // page count changed this session?"* gate was rejected on correctness
    // grounds as well as on cost.
    let audit = crate::pagetree::audit_saved_bytes(&bytes);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // `walked=` is the field worth having and is why this line is emitted
        // on the SUCCESS path too. A clean audit and an audit that never ran
        // produce the same `bad=0`, and this project's most-repeated failure
        // shape is a check that reported success having looked at nothing.
        format!(
            "save-pagetree walked={} pages={} declared={:?} levels={} bad={} nocount={} cycles={} deep={}",
            audit.walked,
            audit.reachable_pages,
            audit.declared_pages,
            // `levels=` because a flat tree (2) CANNOT exhibit the defect
            // this guard is for, and a reader of a trace who does not know
            // that will read a clean line as evidence the writer is sound.
            audit.depth,
            audit.disagreements.len(),
            audit.nodes_without_count,
            audit.cycles,
            audit.too_deep,
        )
    });
    if !audit.is_consistent() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The node ids, which the operator is never shown. A reader of a
            // trace needs to know HOW FAR UP the disagreement goes: the
            // immediate parent being correct and everything above it stale is
            // the signature of "there is no upward walk at all", which is a
            // different engine defect from "the walk stops one short".
            format!(
                "save-refused-pagetree path={target:?} nodes={:?}",
                audit
                    .disagreements
                    .iter()
                    .map(|d| (d.node.num, d.declared, d.reachable, d.root))
                    .collect::<Vec<_>>()
            )
        });
        return Err(SaveError::PageTreeStale { audit });
    }

    std::fs::write(target, &bytes)?;
    Ok(written)
}

/// **The name to suggest for the copy.**
#[must_use]
fn suggested_path(doc: &OpenDoc) -> PathBuf {
    let Some(source) = doc.stored_under() else {
        // A name, not a location. Offer the name; the picker chooses the
        // folder, which is the only honest answer when the document has never
        // been anywhere.
        return doc.path.clone();
    };
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy. `crate::dialogs::ocr::suggested_path` makes the same
        // fallback for the same reason.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let name = format!("{stem}{}.pdf", crate::text::files::save_copy_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

/// **Write an already-serialised compacted copy to a file the operator picks.**
pub fn compacted(doc: &OpenDoc, bytes: &[u8], before: u64) -> bool {
    let suggested = suggested_path(doc);
    let Picked::Path(target) =
        files::pick_save_path(&suggested, crate::text::compact::window_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "compact-cancelled".to_owned()
        });
        return false;
    };
    match std::fs::write(&target, bytes) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // `before=` and `after=` rather than a saving, so a reader of a
                // trace can see which of the two the build got wrong. A single
                // difference is the one number that cannot be checked against
                // anything.
                format!(
                    "compact-written path={:?} before={before} after={} epoch={}",
                    target,
                    bytes.len(),
                    doc.edit_epoch
                )
            });
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::compact::written(
                    &target.display().to_string(),
                    before,
                    bytes.len() as u64,
                ),
            );
            true
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("compact-failed path={target:?} detail={error}")
            });
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::compact::write_failed(&error.to_string()),
            );
            false
        }
    }
}

/// What must never be true of a file this shell wrote — see [`tests`]'s header
/// for the seam.
#[cfg(test)]
mod tests;
