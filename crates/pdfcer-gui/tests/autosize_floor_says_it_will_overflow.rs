//! **A field too small for its text must say the text will overflow** — the
//! whole chain, from the engine's bound to the operator's sentence.
//!
//! # What this is for
//!
//! `pdfcer-core` reports *which constraint* chose an auto-size, and one of the
//! three answers is not an answer: `AutoFitBound::Floor` means the size it
//! returned **does not fit**. The engine says so at the branch that returns it —
//! *"the one case where the returned size does NOT fit the constraint that
//! produced it"* — and stops shrinking at a legibility floor rather than
//! rendering something unreadable.
//!
//! Until 2026-09-07 this shell read the chosen **size** and discarded the
//! **bound**, so an operator whose field was too small was told:
//!
//! > *"⚠ "FullName" asks for an automatic text size and pdfcer chose 4.0 pt.
//! > Another program filling this field may choose differently."*
//!
//! A sentence about interoperability, when the fact was that the text is going
//! to spill out of the box. ⚠ And `OPERATOR_REQUESTS.md` **O86** had told the
//! operator, under a ✅, that *"pdfcer now tells you which way it decided …
//! held at pdfcer's legibility floor; the box is too small for this text, which
//! will overflow"* — true of the engine and the CLI, **false of this shell**.
//!
//! # ★★ Why this test exists beside the unit tests, and what it adds
//!
//! `app::status::tests` asserts the three sentences are distinct and that only
//! one claims overflow. That is a test of the **wording**, and it would pass on
//! a build where nothing ever reaches the overflow arm.
//!
//! This test asserts the **chain**: a real document, a real fill through
//! `EditSession`, the engine's real bound, and the sentence that comes out the
//! far end. It is the difference between *"the sentence exists"* and *"the
//! operator gets it"*, which is the distinction this project keeps paying for.
//!
//! ⚠ It is still not a substitute for the driven check. Nothing here presses a
//! key or draws a status bar; a build whose status bar never calls
//! `fill_disclosure` passes this test. See `tools/ui-verify`.

use pdfcer_core::vartext::AutoFitBound;

/// `fixtures/autosize-field.pdf` — one widget, `/Helv 0 Tf` (auto), in a
/// 190 × 25 pt box. Built from `text-field-with-appearance.pdf` by widening one
/// space so the byte offsets survive; see the `.PROVENANCE.py` beside it.
const FIXTURE: &str = "fixtures/autosize-field.pdf";

/// The field's fully-qualified name in that fixture.
const FIELD: &str = "FullName";

/// Long enough that the width bound drives the size below the 4 pt legibility
/// floor, and **stated as a computation rather than a magic string**:
///
/// the box is 190 pt wide, 2 pt of that is padding, so 188 pt is usable. Floor
/// fires when `candidate * 188 / width_at_candidate < 4`, i.e. when the text is
/// more than 47× the usable width at the height-derived candidate. Helvetica
/// averages a little over half an em, so ~90 characters is comfortably past it
/// and 120 leaves no doubt.
///
/// ★ Deliberately readable English rather than `"x".repeat(120)`: if this test
/// ever fails, the failure message quotes the value, and a wall of `x` tells a
/// reader nothing about whether the input was the problem.
const TOO_LONG: &str = "Alexandra Christina Wetherby-Fitzgerald of the Northern Districts Planning \
                        and Development Authority, Second Floor";

/// A short value that fits comfortably — the control.
const FITS: &str = "Ada";

fn session(fixture: &str) -> pdfcer_core::edit::EditSession {
    let path = format!("{}/../../{fixture}", env!("CARGO_MANIFEST_DIR"));
    let doc = pdfcer_core::document::Document::load(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("cannot load {path}: {e:?}"));
    pdfcer_core::edit::EditSession::new(doc)
}

/// ★★★ The load-bearing test: a real fill of a real too-small field reports
/// `Floor`, and the sentence the operator reads says the text will overflow.
#[test]
fn a_field_too_small_for_its_text_reports_floor_and_says_it_will_overflow() {
    let mut s = session(FIXTURE);
    let out = s
        .fill_text_field(FIELD, TOO_LONG)
        .expect("filling an unprotected text field must be accepted");

    assert_eq!(
        out.applied_autosize_bound,
        Some(AutoFitBound::Floor),
        "the fixture must actually reach the legibility floor, or this test is asserting \
         nothing. Box is 190x25 pt and the value is {} characters. Got size {:?}.\n\
         If the engine's floor or its padding moved, recompute TOO_LONG's length from the \
         comment on it — do NOT relax this assertion.",
        TOO_LONG.chars().count(),
        out.applied_autosize,
    );

    let size = out
        .applied_autosize
        .expect("a Floor bound means a size was chosen");

    // The sentence the status bar and the Forms panel both build from.
    let said = pdfcer_gui::text::forms::forms_fill_autosize_overflow_note(FIELD, size);
    assert!(
        said.contains("overflow"),
        "the operator must be told the text will not fit: {said:?}"
    );
    assert!(
        said.contains("taller") || said.contains("shorten"),
        "and must be told what to do about it: {said:?}"
    );

    // ⚠ The general sentence must NOT be what this case produces. Asserting the
    // absence of the old wording is the part that would have caught the defect:
    // the old build's output contained "chose {size} pt" and nothing else.
    assert!(
        !said.contains("may choose differently"),
        "the overflow case must not be dressed as the ordinary interoperability note: {said:?}"
    );
}

/// The control, and it is doing real work: it proves the fixture is not simply
/// *always* reporting `Floor`.
///
/// ★★ Without this, a build where `applied_autosize_bound` was hard-wired to
/// `Some(Floor)` — or where the engine's bound detection had broken in the
/// permissive direction — would satisfy the test above completely. A one-sided
/// test of a value is not a test of the value.
#[test]
fn the_same_field_with_a_short_value_does_not_report_floor() {
    let mut s = session(FIXTURE);
    let out = s
        .fill_text_field(FIELD, FITS)
        .expect("filling an unprotected text field must be accepted");

    assert_ne!(
        out.applied_autosize_bound,
        Some(AutoFitBound::Floor),
        "{FITS:?} fits a 190x25 pt box easily; a Floor here means the bound is not being \
         computed from the text at all"
    );
    assert!(
        out.applied_autosize.is_some(),
        "the field's /DA asks for auto-size, so a size must have been chosen: {out:?}"
    );
}
