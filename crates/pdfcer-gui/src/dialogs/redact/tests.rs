//! # `dialogs::redact::tests` — the apply transaction's headless assertions
//!
//! Split out of [`super`] on 2026-09-04 (evening) under rule R2, when the
//! third destination took `dialogs/redact.rs` past the 1,500-line ceiling.
//! **Nothing moved but its address**: every test, fixture and paragraph of
//! reasoning is unchanged.
//!
//! ★ The seam is the one R2 asks for. `redact.rs` answers *"what does the
//! dialog do?"*; this answers *"what is guaranteed about the state machine
//! underneath it?"*, and the two grow for different reasons — the first when a
//! control is added, the second when a rule is discovered. The rules here are
//! all about the same thing: **which gestures may reach the irreversible
//! half**, which is why they are pure functions over
//! [`super::RedactDialog`]'s fields and not a driven UI.
//!
//! ★★ What is NOT asserted here, and is not assertable here: that the
//! disclosure is drawn *above* the confirm control. This suite proves it
//! EXISTS in the state where the button is live
//! ([`super::RedactDialog::staging_disclosure`]); the geometry is
//! `tools/ui-verify`'s, which publishes both rects and can compare them.

// ★ The INNER `#![cfg(test)]` is redundant — the module is declared
// `#[cfg(test)] mod tests;` — and it is here anyway, because
// `tools/gates/check-ui-strings.sh` exclusion 2 recognises a test-only FILE by
// exactly this attribute. Without it every assertion message below is read as
// operator copy outside the catalog, which is the 125-hit noise floor that
// exclusion was written to remove.
#![cfg(test)]

use super::*;

/// ★★ **The suggested name is never the file that was opened.**
///
/// The standing rule as a default, and the single most consequential
/// assertion in this module: the source file is the only remaining copy of
/// the content being removed, so a default that pointed at it would make
/// the safety of the operation depend on the operator reading a pre-filled
/// field before pressing Enter.
#[test]
fn the_suggested_name_is_never_the_source_file() {
    let source = PathBuf::from("D:\\jobs\\4471\\Sheet 1.pdf");
    let suggested = suggested_path(&source);
    assert_ne!(suggested, source);
    assert_eq!(
        suggested,
        PathBuf::from("D:\\jobs\\4471\\Sheet 1-redacted.pdf")
    );
    assert_eq!(
        suggested.parent(),
        source.parent(),
        "the copy should land beside the original, where the operator will look for it"
    );
}

/// A capitalised extension still produces a `.pdf`, and a bare filename
/// still produces a usable name.
#[test]
fn the_suggestion_is_always_a_usable_pdf_name() {
    for name in ["scan.PDF", "scan.pdf", "scan", "D:\\a.b.pdf"] {
        let suggested = suggested_path(Path::new(name));
        assert!(
            suggested.to_string_lossy().ends_with(".pdf"),
            "{name} suggested {suggested:?}"
        );
        assert_ne!(suggested, PathBuf::from(name));
    }
}

/// A dialog opened with nothing loaded is not built at all.
///
/// The guard matters more here than for print: [`RedactDialog::open`] runs
/// the whole removal, so one built against an empty shell would be a window
/// that had done a full rewrite of nothing in order to refuse.
#[test]
fn no_document_means_no_dialog() {
    assert!(open_for(&Status::Empty).is_none());
}

/// ★★ **The confirm control is not enabled until both gates are answered.**
///
/// §3, asserted over the state machine rather than over pixels. The
/// interesting direction is the residual one: an operator who ticks only
/// the permanence box on a report with residuals must **not** be able to
/// commit, because the two boxes answer different questions and treating
/// one as both is how a partially-redacted file gets handed over as a
/// complete one.
///
/// It is asserted here as well as at
/// `crate::redact::PreparedRedaction::write_to` deliberately: this is the
/// drawing decision and that is the mechanism, and a test for only one of
/// them would leave the other free to drift.
#[test]
fn the_confirm_control_needs_every_gate_that_applies() {
    let mut dialog = RedactDialog {
        source: PathBuf::from("x.pdf"),
        phase: Phase::Refused(RedactApplyRefusal::NothingToApply),
        acknowledged: false,
        residuals_acknowledged: false,
        destination: Destination::NewFile,
        overwrite_acknowledged: false,
        confirm_requested: false,
        cancel_requested: false,
        close_requested: false,
    };
    assert!(
        !dialog.ready_to_confirm(),
        "a refusal has nothing to confirm"
    );

    // A prepared, clean redaction: one box.
    let session = clean_session();
    let prepared = prepare_redaction_apply(&session).expect("the fixture applies");
    assert!(prepared.verification.is_clean());
    assert!(
        residual_lines(&prepared).is_empty(),
        "the fixture must have nothing to disclose, or the two cases below \
         are the same case"
    );
    dialog.phase = Phase::Prepared(Box::new(prepared));
    assert!(!dialog.ready_to_confirm(), "nothing acknowledged yet");
    dialog.acknowledged = true;
    assert!(
        dialog.ready_to_confirm(),
        "a clean report must not demand a tick nobody can give"
    );

    // …and the same value with a residual: two boxes.
    let session = clean_session();
    let mut prepared = prepare_redaction_apply(&session).expect("the fixture applies");
    prepared
        .verification
        .residuals
        .push(crate::redact::Residual {
            text: "MARGARETHALE".to_owned(),
            site: crate::redact::ResidualSite::RawBytes,
        });
    assert_eq!(residual_lines(&prepared).len(), 1);
    dialog.phase = Phase::Prepared(Box::new(prepared));
    dialog.acknowledged = true;
    dialog.residuals_acknowledged = false;
    assert!(
        !dialog.ready_to_confirm(),
        "★ the permanence box alone must not commit a report with residuals \
         — the two boxes answer different questions, and treating one as \
         both hands over a partially-redacted file as a complete one"
    );
    dialog.residuals_acknowledged = true;
    assert!(dialog.ready_to_confirm());

    // ★★★ …and the third gate, which appears only when the operator has
    // asked to replace the file they opened. It is the one that stands
    // between a click and the destruction of the last copy of the content.
    dialog.choose_destination(Destination::ReplaceOriginal);
    assert!(
        !dialog.ready_to_confirm(),
        "★★★ the two boxes that were enough for a COPY are not enough for a \
         REPLACE. They are about the content; this one is about the file, \
         and an operator can have understood the first and not noticed the \
         second"
    );
    dialog.overwrite_acknowledged = true;
    assert!(
        dialog.ready_to_confirm(),
        "…and with all three given, the operator's own instruction stands: \
         he may replace the file he opened"
    );
}

/// ★★★ **Changing the destination retires the overwrite acknowledgement.**
///
/// The sequence this forbids is not exotic — it is *"I'll just look at what
/// the other option says"*: tick the box, select **a new file**, change
/// your mind, select **replace** again, and find the button already live
/// with a consent you had explicitly withdrawn in between.
///
/// ★ It also asserts the *other* direction, which is the one a "tidying"
/// edit removes as pointless: arriving at [`Destination::NewFile`] must
/// clear it too. Retiring a tick that was not needed costs nothing;
/// deciding *which* changes matter is where the next edit gets it wrong.
#[test]
fn changing_the_destination_retires_the_overwrite_acknowledgement() {
    let mut dialog = RedactDialog {
        source: PathBuf::from("x.pdf"),
        phase: Phase::Refused(RedactApplyRefusal::NothingToApply),
        acknowledged: true,
        residuals_acknowledged: true,
        destination: Destination::NewFile,
        overwrite_acknowledged: false,
        confirm_requested: false,
        cancel_requested: false,
        close_requested: false,
    };

    dialog.choose_destination(Destination::ReplaceOriginal);
    dialog.overwrite_acknowledged = true;

    dialog.choose_destination(Destination::NewFile);
    assert!(
        !dialog.overwrite_acknowledged,
        "leaving the replace choice must retire the tick that was about it"
    );

    dialog.overwrite_acknowledged = true;
    dialog.choose_destination(Destination::ReplaceOriginal);
    assert!(
        !dialog.overwrite_acknowledged,
        "★ …and so must arriving back at it, or the withdrawn consent is \
         still standing when the button goes live"
    );

    // Re-selecting the destination already chosen is not a change, and must
    // not throw away a tick the operator has just given.
    dialog.overwrite_acknowledged = true;
    dialog.choose_destination(Destination::ReplaceOriginal);
    assert!(
        dialog.overwrite_acknowledged,
        "a radio redrawn with the same value every frame must not clear the \
         box on every frame — that would make the control impossible to use \
         rather than merely safe"
    );
}

/// ★★ **The outcome sentence tells the operator that the window they are
/// looking at no longer matches the file.**
///
/// The strangest consequence of the replace path, and the one nothing else
/// on screen would say: the session was not touched, so the canvas still
/// shows the marks and the content underneath them, while the file those
/// bytes came from contains neither. *"The document you still have open is
/// unchanged"* is a reassurance after a copy and a falsehood after a
/// replace.
#[test]
fn the_outcome_sentence_says_the_open_window_is_now_stale_after_a_replace() {
    let path = Path::new("D:/jobs/sheet-01.pdf");
    let copied = outcome_line(path, 4, 2, 0, false);
    let replaced = outcome_line(path, 4, 2, 0, true);
    assert_ne!(
        copied, replaced,
        "two different things happened and they must not share a sentence"
    );
    assert!(
        copied.contains("unchanged"),
        "after a copy the open document really is unchanged: {copied}"
    );
    assert!(
        !replaced.contains("unchanged"),
        "★ after a replace it is NOT unchanged in the sense that matters — \
         the file it came from is gone: {replaced}"
    );
    assert!(
        replaced.contains("sheet-01.pdf") && replaced.contains("open"),
        "the operator must be told, by name, which file to reopen to see \
         what is now in it: {replaced}"
    );
}

/// ★ **Every kind of residual reaches the list, and the list is what gates
/// the checkbox.**
///
/// One derivation for both, so a residual cannot be listed without being
/// acknowledgeable or acknowledged without being listed. The promotion
/// source is the one a tidying edit would drop, because it is the mildest —
/// and a report that silently drops the findings it judges harmless is one
/// whose judgement nobody can audit.
#[test]
fn every_source_of_a_residual_reaches_the_disclosed_list() {
    let session = clean_session();
    let mut prepared = prepare_redaction_apply(&session).expect("the fixture applies");
    assert!(residual_lines(&prepared).is_empty());

    prepared
        .verification
        .residuals
        .push(crate::redact::Residual {
            text: "MARGARETHALE".to_owned(),
            site: crate::redact::ResidualSite::RawBytes,
        });
    assert_eq!(residual_lines(&prepared).len(), 1);

    prepared
        .promoted_by_materialisation
        .push(pdfcer_core::object::ObjId {
            num: 7,
            generation: 0,
        });
    let lines = residual_lines(&prepared);
    assert_eq!(
        lines.len(),
        2,
        "a promotion is a disclosed residual too: {lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("compressed container")),
        "{lines:?}"
    );
}

/// ★ **A written outcome with residuals does not borrow the clean
/// sentence.**
///
/// The catalog's rule 1, at the one call site that chooses between the two.
/// A build that always used the clean form would produce a window saying a
/// file was *"verified absent"* over a report the operator had just
/// acknowledged as incomplete — which is worse than saying nothing, because
/// it contradicts the thing they read a moment earlier.
#[test]
fn the_written_sentence_follows_the_residual_count() {
    let path = Path::new("D:\\jobs\\Sheet 1-redacted.pdf");
    let clean = outcome_line(path, 4, 2, 0, false);
    let dirty = outcome_line(path, 4, 2, 1, false);
    assert_ne!(clean, dirty);
    assert!(clean.contains("verified"), "{clean}");
    assert!(
        !dirty.contains("verified"),
        "a file with an acknowledged residual is not verified absent: {dirty}"
    );
    assert!(dirty.contains("NOT be removed"), "{dirty}");
    // The file name, not the path — see `outcome_line`.
    assert!(clean.contains("Sheet 1-redacted.pdf"), "{clean}");
    assert!(!clean.contains("D:\\jobs"), "{clean}");
}

/// A session with one mark over a distinctive secret, applying cleanly.
///
/// Built from `crate::redact`'s own fixture shape rather than from a file,
/// so every byte in it is one this suite put there — which is what makes
/// "the report has no residuals" a fact about the fixture rather than a
/// property of somebody's producer.
fn clean_session() -> pdfcer_core::edit::EditSession {
    const SECRET: &str = "CONFIDENTIALWITNESSNAME";
    let content = format!("BT /F1 12 Tf 20 100 Td ({SECRET}) Tj ( KEEPTHIS) Tj ET");
    let stream = format!(
        "<< /Length {} >>\nstream\n{content}\nendstream",
        content.len()
    );
    let bodies: [&str; 5] = [
        "<< /Type /Catalog /Pages 2 0 R >>",
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
         /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
        &stream,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ];
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
        format!("trailer\n<< /Size {n} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n").as_bytes(),
    );
    let doc = pdfcer_core::document::Document::from_bytes(buf).expect("the fixture parses");
    let mut session = pdfcer_core::edit::EditSession::new(doc);
    session
        .mark_redactions_by_search(SECRET, false)
        .expect("the fixture's text is extractable");
    session
}

// =======================================================================
// ★★★ THE DEFERRED DESTINATION — 2026-09-04 (evening)
// =======================================================================

/// ★★★ **The default destination writes nothing.**
///
/// The safety property the default has to carry, expressed as a property
/// rather than as an identity. Until this afternoon the default was
/// [`Destination::NewFile`], and it was safe because it never *overwrote*;
/// it is now [`Destination::OpenDocument`], which is safe because it never
/// *writes*. An operator who presses the confirm control without reading
/// the destination group loses nothing on disk either way, and that is the
/// invariant a future re-ordering of the radio buttons must not break.
///
/// It asserts through [`Destination::writes_now`] rather than by comparing
/// to a variant, so a fourth destination made the default would have to be
/// a non-writing one to pass.
#[test]
fn the_default_destination_writes_nothing() {
    assert!(
        !DEFAULT_DESTINATION.writes_now(),
        "★★★ the default destination writes a file. An operator who \
         confirms without reading the destination group must not lose \
         anything on disk by doing so."
    );
    assert!(Destination::NewFile.writes_now());
    assert!(Destination::ReplaceOriginal.writes_now());
}

/// ★★★ **The staging consequence is disclosed on the destination that has
/// it, and only on that one.**
///
/// ★★★ **REWRITTEN 2026-09-05, and the sentence under test is now the
/// opposite of the one this test was written for.** It used to be
/// `the_undo_consequence_is_disclosed_before_the_operator_can_commit` and it
/// asserted that the disclosure named the number of undo steps the click
/// would destroy — the price of `Pass 250.1`'s collapsing verb, which the
/// operator accepted on condition he was told first.
///
/// `Pass 250.2` charges no such price: the undo log survives. What is
/// disclosed in the same place, in the same warning role, from the same
/// region, is the fact that replaced it — **the page does not change**, and
/// the removal happens at the save. That is more surprising rather than
/// less, and it is the one thing about this destination an operator cannot
/// work out by looking.
///
/// The assertion is made over [`RedactDialog::staging_disclosure`] — which
/// [`RedactDialog::gates`] draws **between the destination choice and the
/// confirm control**, so a disclosure that exists is a disclosure the
/// operator passed on the way to the button.
///
/// The negative half matters as much: the two write-now destinations really
/// do produce a file at the click, so a sentence saying nothing is written
/// would be a false claim there.
#[test]
fn the_staging_consequence_is_disclosed_before_the_operator_can_commit() {
    let session = clean_session();
    let prepared = prepare_redaction_apply(&session).expect("the fixture applies");
    let mut dialog = RedactDialog {
        source: PathBuf::from("x.pdf"),
        phase: Phase::Prepared(Box::new(prepared)),
        acknowledged: true,
        residuals_acknowledged: false,
        destination: DEFAULT_DESTINATION,
        overwrite_acknowledged: false,
        confirm_requested: false,
        cancel_requested: false,
        close_requested: false,
    };

    let disclosed = dialog
        .staging_disclosure()
        .expect("★★★ the deferred destination removes nothing at the click and must say so");
    let lower = disclosed.to_lowercase();
    assert!(
        lower.contains("nothing is removed") && lower.contains("save"),
        "★★ the two facts are load-bearing together. \"Nothing is removed\" \
         alone reads as a failure; \"it happens at Save\" alone leaves him \
         believing the page in front of him is already redacted: {disclosed}"
    );
    assert!(
        dialog.ready_to_confirm(),
        "the sentence must be reachable in the state where the button is \
         live, or \"before he commits\" is not what it means"
    );

    for other in [Destination::NewFile, Destination::ReplaceOriginal] {
        dialog.choose_destination(other);
        assert_eq!(
            dialog.staging_disclosure(),
            None,
            "★ {other:?} writes a file at the click, so a sentence saying \
             nothing is removed would be false there"
        );
    }
}

/// ★★★ **A document whose removal is already armed gets the staged phase,
/// with a control that calls it off.**
///
/// New 2026-09-05, and it is the assertion behind *a stageable operation
/// that cannot be un-staged is a trap*. Three things, each with a failure it
/// is looking for:
///
/// 1. **the pipeline refuses by name.** `prepare_redaction_apply` on a
///    staged session must answer [`RedactApplyRefusal::AlreadyStaged`] and
///    not `FullRewriteUnavailable`. Without this the operator would be told
///    *"this document cannot be rewritten in full"* — a true sentence about
///    the wrong subject, because the engine declines `to_full_bytes` while a
///    removal is armed;
/// 2. **the dialog turns that into [`Phase::Staged`]**, which is the only
///    phase carrying the control that unblocks him;
/// 3. **the control pushes the cancel action and closes.** A build that
///    pushed the *stage* action here would re-arm what he asked to call off,
///    and one that pushed nothing would leave him with a document that
///    cannot be saved by any ordinary means.
#[test]
fn a_staged_document_offers_the_control_that_calls_the_removal_off() {
    let mut session = clean_session();
    crate::redact::stage_into_session(&mut session).expect("the fixture stages");

    assert_eq!(
        prepare_redaction_apply(&session).unwrap_err(),
        RedactApplyRefusal::AlreadyStaged,
        "★ the pipeline must name this state rather than letting the \
         engine's `RedactionPending` surface as a full-rewrite failure"
    );

    let mut dialog = RedactDialog {
        source: PathBuf::from("x.pdf"),
        phase: Phase::Staged,
        acknowledged: false,
        residuals_acknowledged: false,
        destination: DEFAULT_DESTINATION,
        overwrite_acknowledged: false,
        confirm_requested: false,
        // ★ Set directly rather than by clicking: `staged::body` needs an
        // `egui::Ui` and this suite is headless by design. What is under test
        // here is what the FLAG does, which is the half a driven check cannot
        // see; `tools/ui-verify` owns the half where a real pointer presses a
        // real button.
        cancel_requested: true,
        close_requested: false,
    };
    assert!(
        !dialog.ready_to_confirm(),
        "there is nothing to confirm in this phase — the decision was taken"
    );

    let mut actions = Vec::new();
    dialog.take_cancel(&mut actions);
    assert!(
        matches!(
            actions.as_slice(),
            [crate::app::actions::Action::Redact(
                crate::app::actions::RedactAction::Pending(crate::redact::Staging::Cancel)
            )]
        ),
        "exactly the cancel, and nothing else: {actions:?}"
    );
    assert!(dialog.close_requested);
}

/// ★★ **Confirming the default destination pushes an action and touches no
/// file system.**
///
/// The whole of `OPERATOR_REQUESTS.md` O125's second half, asserted at the
/// one method that could break it. Three things are checked and each has a
/// failure it is looking for:
///
/// 1. **exactly one action, and it is the apply** — a build that pushed
///    nothing would leave the operator with a dialog that closed and a
///    document that never changed;
/// 2. **the phase is still `Prepared`** — this route must not fabricate a
///    `Written` outcome for a file that does not exist;
/// 3. **the dialog asks to close** — the outcome is reported by the funnel's
///    edit disclosure, and a window left open beside it would be a second
///    account of one event.
#[test]
fn confirming_the_default_destination_pushes_an_action_and_writes_no_file() {
    let session = clean_session();
    let prepared = prepare_redaction_apply(&session).expect("the fixture applies");
    let mut dialog = RedactDialog {
        source: PathBuf::from("x.pdf"),
        phase: Phase::Prepared(Box::new(prepared)),
        acknowledged: true,
        residuals_acknowledged: false,
        destination: DEFAULT_DESTINATION,
        overwrite_acknowledged: false,
        confirm_requested: false,
        cancel_requested: false,
        close_requested: false,
    };

    let mut actions = Vec::new();
    dialog.commit(&mut actions);

    assert_eq!(actions.len(), 1, "exactly one action: {actions:?}");
    assert!(
        matches!(
            actions[0],
            crate::app::actions::Action::Redact(crate::app::actions::RedactAction::Pending(
                crate::redact::Staging::Stage
            ))
        ),
        "the deferred route must raise the STAGE half of the pending-redaction \
         verb — a build that raised the cancel half here would close the dialog \
         having armed nothing: {:?}",
        actions[0]
    );
    assert!(
        matches!(dialog.phase, Phase::Prepared(_)),
        "★ nothing was written, so there is no `Written` outcome to claim. \
         The funnel reports what actually happened."
    );
    assert!(
        dialog.close_requested,
        "the dialog hands the event to the funnel and gets out of the way"
    );
    // The path this dialog would have written to on the other destinations.
    // It must not exist: this route reaches no file system at all.
    assert!(
        !suggested_path(Path::new("x.pdf")).exists(),
        "the deferred route wrote a file"
    );
}

/// ★ **The deferred destination is not asked for the overwrite
/// acknowledgement, and the replace destination still is.**
///
/// The gate used to be spelled `destination == NewFile || acknowledged`,
/// which was correct while there were two destinations and became a trap
/// the moment there were three: a third value that is neither `NewFile` nor
/// `ReplaceOriginal` would have been asked for a tick at a checkbox that is
/// not on screen, leaving the confirm control dead with an explanation
/// pointing at nothing.
///
/// ★★ And the half that is the whole of O125: **Save-over-the-original
/// still warns.** It is a warning and not a refusal — the operator may do
/// it — but he may not do it without having said, at a control naming the
/// file, that he knows what it costs.
#[test]
fn the_overwrite_acknowledgement_is_owed_by_exactly_one_destination() {
    let session = clean_session();
    let prepared = prepare_redaction_apply(&session).expect("the fixture applies");
    assert!(residual_lines(&prepared).is_empty());
    let mut dialog = RedactDialog {
        source: PathBuf::from("x.pdf"),
        phase: Phase::Prepared(Box::new(prepared)),
        acknowledged: true,
        residuals_acknowledged: false,
        destination: DEFAULT_DESTINATION,
        overwrite_acknowledged: false,
        confirm_requested: false,
        cancel_requested: false,
        close_requested: false,
    };
    assert!(
        dialog.ready_to_confirm(),
        "the deferred destination overwrites nothing, so it must not demand \
         a tick at a box that is not drawn"
    );

    dialog.choose_destination(Destination::NewFile);
    assert!(dialog.ready_to_confirm(), "nor does a new file");

    dialog.choose_destination(Destination::ReplaceOriginal);
    assert!(
        !dialog.ready_to_confirm(),
        "★★ replacing the file the operator opened still warns, every time, \
         at a control he has to answer. That is O125's distinction: a \
         warning, not a refusal."
    );
    dialog.overwrite_acknowledged = true;
    assert!(dialog.ready_to_confirm());

    // And the warning names the file, so it is a sentence about a document
    // rather than about a role.
    assert!(t::overwrite_acknowledgement_checkbox("sheet-01.pdf").contains("sheet-01.pdf"));
    assert!(t::confirm_button_replace("sheet-01.pdf").contains("sheet-01.pdf"));
}

/// ★ **The dialog's residual list is the domain count plus promotion, and
/// nothing else.**
///
/// The other half of `crate::redact::tests::
/// the_residual_count_matches_the_disclosed_list_except_for_promotion`.
/// Two derivations exist because the deferred route has no materialisation
/// step of its own to observe a promotion in; this pins the difference at
/// exactly one item so it cannot quietly become two.
#[test]
fn the_disclosed_list_is_the_domain_count_plus_promotion() {
    let session = clean_session();
    let mut prepared = prepare_redaction_apply(&session).expect("the fixture applies");
    let count = |p: &PreparedRedaction| {
        crate::redact::residual_count(&p.report, Some(&p.verification))
            + usize::from(!p.promoted_by_materialisation.is_empty())
    };
    assert_eq!(residual_lines(&prepared).len(), count(&prepared));

    prepared
        .verification
        .residuals
        .push(crate::redact::Residual {
            text: "MARGARETHALE".to_owned(),
            site: crate::redact::ResidualSite::RawBytes,
        });
    assert_eq!(residual_lines(&prepared).len(), count(&prepared));

    prepared
        .promoted_by_materialisation
        .push(pdfcer_core::object::ObjId {
            num: 7,
            generation: 0,
        });
    assert_eq!(
        residual_lines(&prepared).len(),
        count(&prepared),
        "the list and the count must move together, or the number the \
         operator acknowledged and the number he is told afterwards can \
         disagree"
    );
    assert_eq!(residual_lines(&prepared).len(), 2);
}
