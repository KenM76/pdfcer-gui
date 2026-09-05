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
//! > 3. Never put the word "Undo" near a post-apply state.
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
        ("undo_will_be_cleared", undo_will_be_cleared(3)),
        (
            "confirm_button_into_document",
            confirm_button_into_document().to_owned(),
        ),
        (
            "applied_into_document(residual)",
            applied_into_document(2, 1, 1, 3),
        ),
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
    // ★ And the deferred route's clean outcome earns it too, for the same
    // reason and off the same proof: `redact::apply_into_session` runs
    // `proof::prove` over the collapsed session before it returns, so the
    // word is a measurement there as much as it is on the write-now routes.
    assert!(
        applied_into_document(2, 1, 0, 0)
            .to_lowercase()
            .contains("verified"),
        "the deferred clean outcome earns the word: the absence proof ran"
    );
}

/// ★★ **The deferred outcome obeys rule 1, and says the one thing only it
/// has to say.**
///
/// Two assertions rather than one. The first is rule 1 mechanically — the
/// residual form names its count in the same sentence as the success and
/// does not borrow the clean form's wording. The second is this route's own
/// hazard, and it is reachable on no other destination: **nothing is on
/// disk**. An operator who applies, does not save, and then hands over the
/// original file has redacted nothing, and both forms have to tell him so.
#[test]
fn the_deferred_outcome_names_its_residuals_and_says_nothing_is_on_disk_yet() {
    let clean = applied_into_document(4, 2, 0, 0);
    let residual = applied_into_document(4, 2, 3, 0);
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
            line.contains("Nothing is on disk yet"),
            "★ the deferred outcome must say the file has not been written \
             — it is the one failure this destination has that the others \
             do not: {line}"
        );
    }
    // The undo clause appears only when there was something to discard.
    assert!(!clean.contains("undo step"));
    assert!(applied_into_document(4, 2, 0, 9).contains("9 undo step(s)"));
}

/// ★ **No post-apply sentence offers Undo.**
///
/// Rule 3. Every other edit in this shell teaches the operator that undo is
/// available until they save; this is the one moment that expectation is
/// wrong, so the copy has to correct it rather than merely omit it.
///
/// The marking strings are excluded from the sweep on purpose — they say
/// "Undo" and **should**, because taking a mark off genuinely is undoable
/// and telling the operator so is what makes marking feel reversible.
#[test]
fn no_post_apply_sentence_mentions_undo_as_a_way_back() {
    for line in [
        permanence_statement(false).to_owned(),
        permanence_statement(true).to_owned(),
        // ★ The deferred forms, added 2026-09-04 (evening). They are the
        // sentences that talk about undo the MOST — because that route is
        // the one that destroys the log — so leaving them out of this sweep
        // would have exempted exactly the strings the rule is about.
        permanence_statement_deferred().to_owned(),
        destination_open_document_tooltip().to_owned(),
        undo_will_be_cleared(0),
        undo_will_be_cleared(14),
        applied_into_document(1, 1, 0, 0),
        applied_into_document(1, 1, 0, 14),
        applied_into_document(1, 1, 2, 14),
        applied_clean("a.pdf", 1, 1, false),
        applied_clean("a.pdf", 1, 1, true),
        applied_with_residuals("a.pdf", 1, 1, false),
        applied_with_residuals("a.pdf", 1, 1, true),
        confirm_checkbox().to_owned(),
        overwrite_acknowledgement_checkbox("a.pdf"),
        destination_replace_tooltip().to_owned(),
    ] {
        let lower = line.to_lowercase();
        // ★★ The allowance list, and why it grew on 2026-09-04.
        //
        // Rule 3 forbids **offering** undo after an apply. It has never
        // forbidden the word, and the two negation forms below were enough
        // while the only true statement about undo was "it will not help
        // you". The deferred route adds a second true statement — *the undo
        // log is destroyed* — which is a stronger correction of the learned
        // expectation than a negation is, and which cannot be phrased with
        // "not undo" without reading as though undo were the subject rather
        // than the casualty.
        //
        // So the allowance is by MEANING, enumerated: a sentence may use
        // the word if it is saying undo does not reach the content, or that
        // the log itself is gone. Anything else is the failure this test is
        // for.
        // The allowance is by SUBJECT, and there are two subjects a
        // sentence can legitimately give the word: undo as a *way back*
        // (permitted only in an explicit negation) and the undo *log* as a
        // thing being destroyed. The second is not a negation and cannot be
        // phrased as one without making undo the subject again.
        let permitted = [
            "not undo",
            "not be undone",
            // the LOG, not the escape route
            "undo history",
            "undo step",
        ];
        assert!(
            !lower.contains("undo") || permitted.iter().any(|p| lower.contains(p)),
            "a post-apply sentence offers Undo, which is the one place in \
             pdfcer that learned expectation is wrong: {line}"
        );
    }
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
}

/// The census reads as an answer at zero and as a warning above it.
#[test]
fn the_census_changes_shape_rather_than_only_its_number() {
    assert!(marks_count(0).contains("No redaction marks"));
    assert!(!marks_count(0).contains('0'));
    assert!(marks_count(3).contains("STILL IN THIS FILE"));
}
