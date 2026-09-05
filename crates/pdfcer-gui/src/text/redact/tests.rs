//! # `text::redact::tests` — the three wording rules, enforced
//!
//! Split out of [`super`] on 2026-09-04 (evening) under rule R2, when the
//! deferred destination's strings took `text/redact/mod.rs` past the
//! 1,500-line ceiling. **Nothing moved but its address.**
//!
//! ★ The seam is the one R2 asks for, and on this catalog it is sharper than
//! usual. `mod.rs` holds *sentences*; this holds the **rules those sentences
//! obey**, quoted in [`super`]'s header:
//!
//! > 1. Never say "removed" without qualification when anything was left.
//! > 2. Never say "verified" unless a verification step actually ran.
//! > 3′. Never suggest that Undo can recover content that has reached a file.
//!
//! ★★★ Rule 3 was **corrected in place** on 2026-09-05 — see [`super`]'s
//! header. It used to forbid the word near any "post-apply" state, which
//! `Pass 250.2` made false: the deferred route preserves undo completely, so a
//! staged removal CAN be undone and a sentence saying so is true and useful.
//! Undo reaches the **arming** and never reaches the **removal**, and the two
//! sweeps below are drawn on exactly that line.
//!
//! Each is a sweep over a list of strings, and the lists are the load-bearing
//! part: a string added to the catalog and not to the sweep is a string the
//! rules do not reach. That is why every test below enumerates rather than
//! sampling, and why adding copy to `mod.rs` means adding a row here.

// ★ The INNER `#![cfg(test)]` is redundant — the module is declared
// `#[cfg(test)] mod tests;` — and it is here anyway, because
// `tools/gates/check-ui-strings.sh` exclusion 2 recognises a test-only FILE by
// exactly this attribute.
#![cfg(test)]

use super::*;

/// ★★ **No marking string ever claims content was removed.**
///
/// Rule 1 of the module header, asserted rather than trusted. The failure
/// this catches is a copy pass tightening *"marked for redaction"* into
/// *"redacted"* — which reads better, is shorter, and is the exact
/// misunderstanding that ships marked documents.
#[test]
fn nothing_on_the_marking_surface_claims_a_removal() {
    let marking: Vec<String> = vec![
        panel_intro().to_owned(),
        mark_heading().to_owned(),
        mark_whole_page().to_owned(),
        mark_whole_page_tooltip().to_owned(),
        search_button_tooltip(true).to_owned(),
        search_hint(false).to_owned(),
        search_hint(true).to_owned(),
        marks_count(3),
        mark_remove_tooltip().to_owned(),
    ];
    for line in &marking {
        let lower = line.to_lowercase();
        for claim in ["has been removed", "was removed", "is redacted", "now gone"] {
            assert!(
                !lower.contains(claim),
                "`{line}` claims {claim:?} on the MARKING surface, where nothing \
                 has been removed. Marking is reversible; applying is not, and \
                 an operator who believes otherwise ships the marked file"
            );
        }
    }
}

/// ★★ **"Verified" appears in exactly one place.**
///
/// Rule 2, and it is a test rather than a doc comment because the word is
/// the single most valuable one on this surface: it is the difference
/// between a report and a claim, and it costs nothing to sprinkle it
/// somewhere it is not earned.
#[test]
fn only_the_verification_line_and_the_clean_outcome_say_verified() {
    let everything: Vec<(&str, String)> = vec![
        ("panel_intro", panel_intro().to_owned()),
        (
            "permanence_statement",
            permanence_statement(false).to_owned(),
        ),
        (
            "permanence_statement(replacing)",
            permanence_statement(true).to_owned(),
        ),
        (
            "permanence_statement_deferred",
            permanence_statement_deferred().to_owned(),
        ),
        ("destination_heading", destination_heading().to_owned()),
        ("destination_new_file", destination_new_file().to_owned()),
        (
            "destination_open_document",
            destination_open_document().to_owned(),
        ),
        (
            "destination_open_document_tooltip",
            destination_open_document_tooltip().to_owned(),
        ),
        (
            "removal_happens_at_save",
            removal_happens_at_save().to_owned(),
        ),
        (
            "confirm_button_into_document",
            confirm_button_into_document().to_owned(),
        ),
        (
            "staged_into_document(residual)",
            staged_into_document(2, 1, 1),
        ),
        ("staged_into_document(clean)", staged_into_document(2, 1, 0)),
        ("staged_heading", staged_heading().to_owned()),
        ("staged_body", staged_body().to_owned()),
        ("cancel_button_staged", cancel_button_staged().to_owned()),
        (
            "cancel_button_staged_tooltip",
            cancel_button_staged_tooltip().to_owned(),
        ),
        ("staging_cancelled", staging_cancelled(3)),
        (
            "destination_new_file_tooltip",
            destination_new_file_tooltip().to_owned(),
        ),
        ("destination_replace", destination_replace("a.pdf")),
        (
            "destination_replace_tooltip",
            destination_replace_tooltip().to_owned(),
        ),
        (
            "overwrite_acknowledgement_checkbox",
            overwrite_acknowledgement_checkbox("a.pdf"),
        ),
        ("confirm_button_replace", confirm_button_replace("a.pdf")),
        ("removal_summary", removal_summary(2, 1, 30, 1)),
        ("single_revision_note", single_revision_note().to_owned()),
        ("residual_heading", residual_heading().to_owned()),
        (
            "raw_residual_line",
            raw_residual_line("x", crate::redact::ResidualSite::FontProgram),
        ),
        ("scope_reminder", scope_reminder().to_owned()),
        ("confirm_checkbox", confirm_checkbox().to_owned()),
        ("confirm_button", confirm_button().to_owned()),
        ("marks_count", marks_count(2)),
        (
            "applied_with_residuals",
            applied_with_residuals("a.pdf", 2, 1, false),
        ),
        (
            "applied_with_residuals(replaced)",
            applied_with_residuals("a.pdf", 2, 1, true),
        ),
    ];
    for (name, line) in &everything {
        assert!(
            !line.to_lowercase().contains("verif"),
            "`{name}` uses the word \"verified\", which only a clean \
             AbsenceVerification earns: {line}"
        );
    }
    assert!(verified_line(3).to_lowercase().contains("verified"));
    for replaced in [false, true] {
        assert!(
            applied_clean("a.pdf", 2, 1, replaced)
                .to_lowercase()
                .contains("verified"),
            "★ the clean outcome earns the word on BOTH destinations — the \
             proof ran on the same bytes either way, and dropping it on the \
             replace path would make the more consequential outcome the \
             less informative one"
        );
    }
    // ★★★ And the deferred route's SAVE outcome earns it — but its STAGING
    // outcome must not, and that inversion is the whole of what `Pass 250.2`
    // did to this rule.
    //
    // Until 2026-09-05 the deferred route collapsed the session and proved the
    // result, so `applied_into_document` said "verified" and had earned it.
    // `apply_redactions_deferred` runs the removal only for a preview and
    // **discards the bytes** (`crate::redact` §1.0.1), so at staging time
    // nothing has been swept and the word would be a claim about a sweep that
    // did not happen — on the one surface where that word is the whole
    // difference between a report and an assertion.
    //
    // The word moved to the moment the bytes exist. `staged_into_document` is
    // in the sweep above, where it must stay.
    assert!(
        saved_applying_redaction("a.pdf", 2, 1, 0)
            .to_lowercase()
            .contains("verified"),
        "★★★ the STAGED SAVE's clean outcome earns the word: \
         `redact::save_applying_pending` swept the exact bytes before \
         returning them and `app::save` swept them again between the buffer \
         and the syscall"
    );
    assert!(
        !saved_applying_redaction("a.pdf", 2, 1, 3)
            .to_lowercase()
            .contains("verif"),
        "★★ …and its RESIDUAL form does not, because a save that left three \
         items behind has not verified the absence of anything. Rule 1 and \
         rule 2 meeting: the residual sentence never borrows the clean one's \
         vocabulary."
    );
}

/// ★★ **The staged outcome obeys rule 1, and says the two things only it has
/// to say.**
///
/// ★★★ **REWRITTEN 2026-09-05.** Its predecessor asserted that both forms
/// said *"Nothing is on disk yet"*, which was the deferred route's own hazard
/// under `Pass 250.1`: the content had been removed from the document, the
/// file had not been written, and an operator who handed over the original had
/// redacted nothing.
///
/// Under `Pass 250.2` that sentence is no longer sufficient, because a second
/// thing is now also true and is the more surprising of the two: **nothing has
/// been removed either.** The page is unchanged. So both forms must say both
/// facts, and this test enumerates them rather than sampling — a form that
/// said only "nothing is on disk" would leave him believing the document in
/// front of him was already redacted, which is the marked-file failure this
/// whole feature exists to prevent.
#[test]
fn the_staged_outcome_names_its_residuals_and_says_nothing_has_happened_yet() {
    let clean = staged_into_document(4, 2, 0);
    let residual = staged_into_document(4, 2, 3);
    assert_ne!(clean, residual);
    assert!(
        residual.contains("NOT be removed") && residual.contains('3'),
        "the residual form must name what is left, in the same sentence as \
         the success: {residual}"
    );
    assert!(
        !clean.contains("could NOT"),
        "the clean form must not carry the residual wording: {clean}"
    );
    for line in [&clean, &residual] {
        assert!(
            line.contains("Nothing has been removed yet"),
            "★★★ the staged outcome must say the removal has NOT happened. \
             The page is unchanged, and an operator who reads this as a \
             completed redaction has a marked document he believes is a \
             clean one: {line}"
        );
        assert!(
            line.contains("the page has not changed"),
            "★★ …and it must say so about the PAGE specifically, because \
             that is the evidence in front of him and it says the opposite: \
             {line}"
        );
        assert!(
            line.contains("Save"),
            "★ and it must name what does it, or the fact above is a \
             complaint rather than an instruction: {line}"
        );
    }
    // ★ And no undo clause anywhere. `Pass 250.2` discards nothing, so a
    // sentence about discarded steps would be a false claim about the
    // operator's work — the exact claim the predecessor of this test asserted
    // was PRESENT.
    for line in [&clean, &residual] {
        assert!(
            !line.to_lowercase().contains("undo step"),
            "nothing is discarded on this route any more: {line}"
        );
    }
}

/// ★★★ **The staged-save outcome says the three things nothing else on
/// screen can.**
///
/// New 2026-09-05, and each clause is a defect if it is missing:
///
/// 1. **the file has the content removed** — the receipt, and the only one of
///    the three the operator would guess;
/// 2. **the window is stale** — the session was never mutated, so the canvas
///    goes on drawing the marks and the content while the file holds neither.
///    Without this he concludes the save did not work, or worse, that a page
///    still showing a name is a page whose name was removed;
/// 3. **the removal is still armed** — `save_applying_redaction` takes
///    `&self`, so the next save does it again and the ordinary modes stay
///    refused. *"I saved it, so it is done"* is the assumption that would
///    otherwise stand.
#[test]
fn the_staged_save_outcome_says_the_window_is_stale_and_the_removal_is_still_armed() {
    for residuals in [0_usize, 3] {
        let line = saved_applying_redaction("sheet-01.pdf", 4, 2, residuals);
        assert!(
            line.contains("sheet-01.pdf"),
            "it names the file, because the operator is being told about a \
             file: {line}"
        );
        assert!(
            line.contains("still shows the marks and the content"),
            "★★★ the window is stale and only this sentence says so: {line}"
        );
        assert!(
            line.contains("still set up"),
            "★★ the removal survives the save, and the next Ctrl+S does it \
             again. An operator who does not know that is surprised by his \
             own program: {line}"
        );
    }
    assert!(
        saved_applying_redaction("a.pdf", 4, 2, 3).contains("NOT be removed"),
        "rule 1: the residual is named in the same sentence as the success"
    );
}

/// ★ **A cancel says what survives, not merely that it happened.**
///
/// The one misreading available at this control is *"never mind, that is
/// dealt with"* — and it is not: the marks are still on the document and the
/// content is still in the file, which is exactly what the operator asked for
/// and exactly what he must not be allowed to forget.
#[test]
fn the_cancel_sentence_says_the_marks_are_still_there() {
    let line = staging_cancelled(3);
    assert!(line.contains('3'), "it names how many survive: {line}");
    assert!(
        line.contains("still on the document") && line.contains("still in the file"),
        "★★ un-arming is not un-marking and is not un-redacting. Both facts, \
         because a sentence with only one of them reads as the other: {line}"
    );
}

/// ★ **No sentence about content that has reached a FILE offers Undo.**
///
/// Rule 3′ — see the module header, where rule 3 was corrected in place on
/// 2026-09-05. The rule used to forbid the word near any "post-apply" state
/// and it survived `Pass 250.1` because that verb destroyed the undo log
/// outright. `Pass 250.2` preserves undo completely, so a staged removal
/// **can** be undone, and a rule forbidding the word near that state would
/// forbid the true and useful sentence.
///
/// The distinction that replaced it: undo reaches the **arming**, and never
/// reaches the **removal**. So this sweep's membership list is the
/// load-bearing half, and it is drawn on exactly that line:
///
/// * **In** — every sentence about content that is gone from a file, and
///   every sentence about what applying will permanently do.
/// * **Out** — the marking strings (taking a mark off genuinely is undoable),
///   and the staging strings, which are about a state undo *does* reach and
///   which have their own test above.
#[test]
fn no_post_apply_sentence_mentions_undo_as_a_way_back() {
    for line in [
        permanence_statement(false).to_owned(),
        permanence_statement(true).to_owned(),
        // ★ The deferred permanence statement stays IN the sweep even though
        // its route preserves undo, because the clause it shares with its two
        // siblings is about the content once the save has happened — and that
        // is precisely the state rule 3′ governs.
        permanence_statement_deferred().to_owned(),
        // ★★★ The staged SAVE outcome, added 2026-09-05, and it is the single
        // most important member of this list: it is the only sentence in the
        // catalog read at the exact moment content has left a file on this
        // route.
        saved_applying_redaction("a.pdf", 1, 1, 0),
        saved_applying_redaction("a.pdf", 1, 1, 2),
        applied_clean("a.pdf", 1, 1, false),
        applied_clean("a.pdf", 1, 1, true),
        applied_with_residuals("a.pdf", 1, 1, false),
        applied_with_residuals("a.pdf", 1, 1, true),
        confirm_checkbox().to_owned(),
        overwrite_acknowledgement_checkbox("a.pdf"),
        destination_replace_tooltip().to_owned(),
    ] {
        let lower = line.to_lowercase();
        // ★★ The allowance list, and why it SHRANK on 2026-09-05.
        //
        // Rule 3′ forbids **offering** undo as a way back to content that has
        // reached a file. It has never forbidden the word, and the two
        // negation forms below are what a sentence needs in order to correct
        // the learned expectation rather than merely omit it.
        //
        // The two log-related allowances that were added on 2026-09-04 —
        // "undo history" and "undo step" — are **removed**. They existed
        // because the collapsing verb destroyed the log and the sentences had
        // to say so, which is not a negation and could not be phrased as one.
        // Nothing destroys the log any more, so a sentence in this list
        // mentioning it would be a false claim about the operator's work, and
        // the allowance that would have let it through is gone with the verb
        // that needed it.
        let permitted = ["not undo", "not be undone"];
        assert!(
            !lower.contains("undo") || permitted.iter().any(|p| lower.contains(p)),
            "a sentence about content that has reached a file offers Undo, \
             which is the one place in pdfcer that learned expectation is \
             wrong: {line}"
        );
    }
}

/// ★★★ **…and the staging sentences are ALLOWED to say undo works, because
/// it does.**
///
/// The other side of rule 3′, and it is asserted rather than merely permitted
/// by omission. New 2026-09-05.
///
/// Before a save, undo genuinely reaches everything: the marks, the edits
/// around them, and — through the *call it off* control — the arming itself.
/// A catalog that stayed silent about that out of habit would leave the
/// operator believing a staged removal is as irreversible as a written one,
/// which is the wrong lesson in the *cautious* direction and costs him the
/// whole capability `Pass 250.2` bought.
///
/// So this test is a **positive** one: at least one string the operator reads
/// while choosing the deferred destination must tell him undo still works.
#[test]
fn the_staging_copy_says_undo_still_works() {
    let tooltip = destination_open_document_tooltip().to_lowercase();
    assert!(
        tooltip.contains("undo still works"),
        "★★★ the operator is choosing between a route that writes now and one \
         that does not, and the one that does not is REVERSIBLE. Saying so is \
         the difference between a control he uses and one he is afraid of: \
         {tooltip}"
    );
    assert!(
        staged_body().to_lowercase().contains("undoing"),
        "★★ and the phase he lands in afterwards says it too, because that is \
         where he goes to find out what he can still do"
    );
}

/// ★ **A residual outcome and a clean one do not share a sentence.**
///
/// Rule 1 mechanically: the residual form must name the leftover count in
/// the same sentence as the success, and must not be reachable by softening
/// the clean form.
#[test]
fn the_two_outcomes_read_differently_and_the_residual_one_names_its_count() {
    let clean = applied_clean("survey.pdf", 4, 2, false);
    let residual = applied_with_residuals("survey.pdf", 4, 2, false);
    assert_ne!(clean, residual);
    assert!(
        residual.contains("NOT be removed") && residual.contains('2'),
        "the residual outcome must name what is left, in the same sentence \
         as the success: {residual}"
    );
    assert!(
        !clean.contains("could NOT"),
        "the clean outcome must not carry the residual wording: {clean}"
    );
}

/// The suggested name can never be the file that was opened.
///
/// The suffix is the mechanism; `crate::dialogs::redact` asserts the
/// resulting path. This asserts the half that lives in the catalog, in the
/// shape `crate::text::ocr`'s equivalent test established.
#[test]
fn the_suggested_name_differs_from_the_original() {
    assert!(suggested_suffix().starts_with('-'));
    assert!(confirm_button().contains("as…"), "it must read as a prompt");
    assert!(
        !confirm_button().to_lowercase().contains("ok"),
        "the label IS the consequence"
    );
}

/// Every refusal says something different, and each names its own cause.
#[test]
fn each_named_refusal_says_something_different() {
    use crate::redact::RedactApplyRefusal as R;
    let all = [
        R::NothingToApply,
        R::FullRewriteUnavailable {
            reason: "hybrid".to_owned(),
        },
        R::MaterialisedDocumentUnreadable {
            reason: "bad xref".to_owned(),
        },
        R::CoreRefused {
            reason: "page 2 is an image".to_owned(),
        },
        R::VerificationFailed {
            survivors: vec!["x".to_owned()],
        },
        // ★ New 2026-09-05, and it is the one member of this set that is not
        // a failure. It must still have a sentence of its own, and the
        // sentence must not read as one — see `refusal_message`'s own note.
        R::AlreadyStaged,
    ];
    let mut seen: Vec<String> = Vec::new();
    for refusal in &all {
        let s = refusal_message(refusal);
        assert!(!s.is_empty());
        assert!(
            !seen.contains(&s),
            "{refusal:?} repeats a sentence another refusal already uses"
        );
        seen.push(s);
    }
    assert!(
        refusal_message(&all[3]).contains("page 2 is an image"),
        "the engine's own diagnosis is the actionable half and must survive"
    );
    // ★★ And the one that is not a failure does not read as one. It is the
    // ordinary state of a document whose removal is armed, and a sentence
    // beginning "Redaction refused" there would send an operator looking for
    // a fault in a document that has none.
    let staged = refusal_message(&R::AlreadyStaged);
    assert!(
        !staged.to_lowercase().contains("refused"),
        "an ordinary state must not be worded as a failure: {staged}"
    );
    assert!(
        staged.contains("Save"),
        "…and it must name what carries the removal out, because that is the \
         question the operator opened this window to ask: {staged}"
    );
}

/// The census reads as an answer at zero and as a warning above it.
#[test]
fn the_census_changes_shape_rather_than_only_its_number() {
    assert!(marks_count(0).contains("No redaction marks"));
    assert!(!marks_count(0).contains('0'));
    assert!(marks_count(3).contains("STILL IN THIS FILE"));
}
