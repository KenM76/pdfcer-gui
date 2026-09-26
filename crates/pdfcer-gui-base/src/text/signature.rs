//! # `text::signature` — what this shell says about a digital signature before
//! and after it writes
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/signature.md`.

/// The title bar of the window that asks before an invalidating save.
#[must_use]
pub const fn window_title() -> &'static str {
    "This document is signed"
}

/// The headline when a **certification** signature is present — the
/// `ImpactBasis::SpecSourced` case.
#[must_use]
pub fn headline_certified(count: usize) -> String {
    if count == 1 {
        "Saving will invalidate this document's certification signature.".to_owned()
    } else {
        format!("Saving will invalidate this document's {count} signatures.")
    }
}

/// The headline when only **approval** signatures are present — the
/// `ImpactBasis::ConservativeReport` case.
#[must_use]
pub fn headline_approval(count: usize) -> String {
    if count == 1 {
        "This save changes the document after it was signed.".to_owned()
    } else {
        format!("This save changes the document after its {count} signatures were applied.")
    }
}

/// Why the verdict stands, when a certification signature is present.
#[must_use]
pub const fn basis_certified() -> &'static str {
    "A certification signature records the changes the person who certified this document \
     allowed. The PDF standard says any other change invalidates it, and none of the changes \
     pdfcer makes are on that list."
}

/// Why the verdict stands, when only approval signatures are present.
#[must_use]
pub const fn basis_approval() -> &'static str {
    "The PDF standard does not settle whether that invalidates a signature of this kind. pdfcer \
     reports it as invalidated rather than tell you nothing has changed, so this is a cautious \
     answer and not a measurement."
}

/// What an in-place save does to the file the signature is in.
#[must_use]
pub fn target_in_place(name: &str) -> String {
    format!(
        "This writes over {name}. Your edits are appended, so the signed version is still inside \
         the file — but the document as it now stands is the one a reader checks."
    )
}

/// What a save-a-copy does to the file the signature is in: nothing.
#[must_use]
pub const fn target_copy() -> &'static str {
    "This writes a new file. Your original is not changed and keeps its signature."
}

/// The button that goes ahead, when a certification signature is present.
#[must_use]
pub const fn proceed_certified() -> &'static str {
    "Save and invalidate the signature"
}

/// The button that goes ahead, when only approval signatures are present.
#[must_use]
pub const fn proceed_approval() -> &'static str {
    "Save anyway"
}

/// The button that does not save.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The footnote under the buttons: pdfcer has not checked anything.
#[must_use]
pub const fn verifies_nothing() -> &'static str {
    "This window does not look at any signature's certificate or its cryptography. It reports \
     what a save does to the bytes a signature covers, and nothing more. The Signatures panel \
     is where pdfcer reports what it can establish about a signature itself."
}

/// The status-bar note after a save whose signatures kept their byte range.
#[must_use]
pub fn preserved_note(count: usize) -> String {
    let subject = if count == 1 {
        "This document is signed"
    } else {
        "This document carries several signatures"
    };
    format!(
        "{subject}, and this save was appended, so the bytes each signature covers are \
         unchanged. That is not the same as the signature still being valid: whether these \
         changes are ones the signer allowed is a separate question, and pdfcer does not answer \
         it."
    )
}

/// The status-bar note after a save that pdfcer reports as invalidating.
#[must_use]
pub fn invalidated_note(count: usize) -> String {
    let (subject, object) = if count == 1 {
        ("This document is signed", "the signature")
    } else {
        ("This document carries several signatures", "them")
    };
    format!(
        "{subject}, and the save you just made changes it after signing. pdfcer reports that as \
         invalidating {object}; that is a reading of where the bytes moved, not of the \
         cryptography. The Signatures panel reports the rest."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The preserved-save note never reads as a reassurance.**
    #[test]
    fn the_preserved_note_pairs_the_fact_with_the_uncertainty() {
        for count in [1, 3] {
            let note = preserved_note(count);
            assert!(
                note.contains("not the same as the signature still being valid"),
                "the stage-1 fact is stated without the stage-2 denial beside it: {note}"
            );
            assert!(
                note.contains("separate question"),
                "the note must name stage 2 as unanswered, not merely decline to answer it: \
                 {note}"
            );
            assert!(
                note.contains("pdfcer does not answer it"),
                "the note must say who is not answering: {note}"
            );
        }
    }

    /// **The two footings do not share a sentence, and the certified one is
    /// the only one that asserts the outcome.**
    #[test]
    fn the_two_footings_are_worded_differently() {
        assert_ne!(headline_certified(1), headline_approval(1));
        assert_ne!(basis_certified(), basis_approval());
        assert_ne!(proceed_certified(), proceed_approval());

        assert!(
            headline_certified(1).contains("invalidate"),
            "Table 254's closed list supports the flat assertion and the headline should make it"
        );
        assert!(
            !headline_approval(1).contains("invalidate"),
            "ISO 32000-1 is silent for a plain approval signature; the headline must not put \
             pdfcer's cautious verdict in the largest type as though it were the standard's"
        );
    }

    /// **The approval footing says whose verdict it is.**
    #[test]
    fn the_approval_footing_attributes_the_verdict_to_pdfcer() {
        let basis = basis_approval();
        assert!(
            basis.contains("does not settle"),
            "the silence of the standard is the load-bearing fact: {basis}"
        );
        assert!(
            basis.contains("pdfcer reports"),
            "the verdict must be attributed, not asserted: {basis}"
        );
    }

    /// **No string in this file predicts another application's behaviour.**
    #[test]
    fn nothing_here_claims_what_another_reader_will_say() {
        let strings = [
            window_title().to_owned(),
            headline_certified(1),
            headline_approval(1),
            basis_certified().to_owned(),
            basis_approval().to_owned(),
            target_in_place("a.pdf"),
            target_copy().to_owned(),
            proceed_certified().to_owned(),
            proceed_approval().to_owned(),
            verifies_nothing().to_owned(),
            preserved_note(1),
            invalidated_note(1),
        ];
        for s in &strings {
            let lower = s.to_lowercase();
            for forbidden in ["acrobat", "pades", "other viewer", "other readers"] {
                assert!(
                    !lower.contains(forbidden),
                    "`{forbidden}` is an unsourced claim about another application: {s}"
                );
            }
        }
    }

    /// **Nothing here says a signature is valid, or that pdfcer checked
    /// one.**
    #[test]
    fn no_string_claims_a_signature_is_valid_or_was_verified() {
        assert!(
            preserved_note(1).contains("not the same as the signature still being valid"),
            "the one affirmative-looking phrase in the catalog must sit inside its own negation"
        );
        for s in [
            window_title().to_owned(),
            headline_certified(1),
            headline_approval(1),
            basis_certified().to_owned(),
            basis_approval().to_owned(),
            target_in_place("a.pdf"),
            target_copy().to_owned(),
            proceed_certified().to_owned(),
            proceed_approval().to_owned(),
            verifies_nothing().to_owned(),
            invalidated_note(1),
        ] {
            let lower = s.to_lowercase();
            for forbidden in [
                "still valid",
                "is valid",
                "are valid",
                "remains valid",
                "remain valid",
                "verified",
                "we checked",
                "pdfcer checked",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "`{forbidden}` asserts something pdfcer has not established: {s}"
                );
            }
        }
    }

    /// The singular and the plural are different sentences, and both read as
    /// English.
    #[test]
    fn the_counts_read_as_sentences() {
        for pair in [
            (headline_certified(1), headline_certified(4)),
            (headline_approval(1), headline_approval(4)),
            (preserved_note(1), preserved_note(4)),
            (invalidated_note(1), invalidated_note(4)),
        ] {
            assert_ne!(pair.0, pair.1, "the plural must not be the singular");
            assert!(!pair.0.contains("(s)"), "{}", pair.0);
            assert!(!pair.1.contains("(s)"), "{}", pair.1);
        }
    }

    /// **The in-place sentence names the file it is about to write over.**
    #[test]
    fn the_in_place_sentence_names_the_file() {
        let line = target_in_place("Sheet 1.pdf");
        assert!(line.contains("Sheet 1.pdf"), "{line}");
        assert_ne!(line, target_copy());
    }
}
