//! # `text::measure` — what the measure tools say about what they inferred
//!
//! ## Rule 15
//!
//! What these tools author is a **ce dimension** — one pdfcer writes. A **pdf
//! dimension** is CAD-exported page content pdfcer reads and must not alter.
//! This catalog avoids the bare word, exactly as [`crate::text::scale`] and
//! [`crate::text::dimension_groups`] do.
//!
//! ## ★ Why this module exists at all
//!
//! Because the two-line tool's output is an **inference**, and the shell was
//! swallowing every statement about it.
//!
//! `pdfcer-core` gives the two-line gesture three things a shell is expected to
//! surface, and until 2026-08-19 this build surfaced none of them:
//!
//! | fact | what it means | what happened without it |
//! |---|---|---|
//! | `TwoLineRefusal` | collinear or zero-length lines — refused **by name** | the second click did nothing, silently, and the operator clicked again |
//! | `measured_angle_degrees` | the true angle, reported **even when forced parallel** | a checkbox that hides the number it is overriding |
//! | `apex_is_real() == Some(false)` | the lines meet only if extended | ordinary in CAD, and a fact the operator may not have noticed |
//!
//! `docs/core-api/03-capabilities.md` §1.5 obligation 4 states the second of
//! those and quotes the engine's own reason: *"a checkbox that hides the number
//! it is overriding is withholding the fact that makes the decision a
//! decision."*
//!
//! ## Why the refusal is worded HERE and not taken from the error's `Display`
//!
//! `TwoLineRefusal`'s `thiserror` messages are written for an operator, and the
//! capabilities doc says to surface them. It is still not right to print them:
//! `check-ui-strings.sh`'s exclusion 3 says in as many words that an error
//! type's `Display` output being exempt *"is not permission to route UI text
//! through an error type"*.
//!
//! So the variant is matched and the sentence lives in this catalog — which is
//! the same resolution [`crate::text::markup::deleted_collateral`] reached for
//! `DeletionReport`. The **refusal is still surfaced by name**, which is what
//! the obligation actually asks for; what changes is which crate owns the
//! English, and that is decision 002 R1.

use pdfcer_core::dimension::TwoLineRefusal;

/// Why a two-line pick could not author anything.
///
/// ★ Both sentences say **what to do next**, not only what went wrong. A
/// refusal an operator cannot act on is a dead end with better manners: the
/// gesture is still armed, both picks are still there, and the corrective is
/// one more click on a different line.
#[must_use]
pub const fn two_line_refused(refusal: TwoLineRefusal) -> &'static str {
    match refusal {
        TwoLineRefusal::Collinear => {
            "Those two lines lie along the same line, so there is no distance \
             between them and no angle. Pick a line somewhere else on the \
             drawing."
        }
        TwoLineRefusal::Degenerate => {
            "One of those lines has no length, so it has no direction to \
             measure from. Pick a line with two distinct ends."
        }
        // The engine's enum is not `#[non_exhaustive]` today, and a `_` arm
        // would silently swallow a third refusal if it became so. Spelled as a
        // panic-free fallback rather than omitted, because a blank status line
        // is the exact failure this module exists to end.
        #[allow(unreachable_patterns, reason = "belt to the enum's braces")]
        // ui-text-exempt: lint justification, never displayed
        _ => "Those two lines cannot be dimensioned together.",
    }
}

/// ★ **What the two-line tool read the pair as** — the disclosure obligation
/// this build was not meeting.
///
/// Returns `None` when there is nothing an operator could not already see: an
/// ordinary angular reading of two lines that genuinely cross, with no
/// override. A sentence on every commit is a sentence nobody reads, and it
/// would bury the two cases that matter.
///
/// The three cases it does speak for:
///
/// 1. **Forced parallel.** The operator ticked the override and the lines are
///    *not* parallel. The number they overrode is stated — that is the whole of
///    obligation 4, and the engine populates `measured_angle_degrees`
///    **especially** in this case for exactly this sentence.
/// 2. **A virtual apex.** The lines meet only if extended. `pdfcer-core` is
///    explicit that this is *"perfectly ordinary … so it is not refused; but it
///    is a fact about the geometry the operator may not have noticed"*.
/// 3. **Both.**
///
/// # Why it is a status-line sentence and not a mark on the drawing
///
/// Rule 4. The authored ce dimension renders exactly as a saved one will —
/// no tint, no badge, no dashed line saying "this apex is imaginary". The
/// inference is disclosed **off-canvas**, which is what this function is for,
/// and the canvas is left alone.
/// # Why it takes three facts and not the `TwoLineAuthoring`
///
/// The catalog convention — [`crate::text::markup::deleted_collateral`] takes
/// four primitives out of a `DeletionReport` for the same reason — and here it
/// is forced as well as conventional: `TwoLineRelation` is not publicly
/// re-exported from `pdfcer-core`, so a `TwoLineAuthoring` cannot be built in a
/// test in this crate at all. A wording function that cannot be tested without
/// a running gesture is a wording function nobody tests.
///
/// The caller does the extraction, in one place, at the one call site.
#[must_use]
pub fn two_line_reading(
    forced_parallel: bool,
    measured_angle_degrees: Option<f64>,
    virtual_apex: bool,
) -> Option<String> {
    match (forced_parallel, virtual_apex) {
        (false, false) => None,
        (true, _) => Some(match measured_angle_degrees {
            // The number the override overrode. Two decimals because the
            // interesting values are fractions of a degree — a 0.8° pair read
            // as parallel is the case the threshold exists for, and "1°" would
            // round away the fact being disclosed.
            Some(degrees) => format!(
                "Read as parallel because you asked — the two lines are {degrees:.2}° apart."
            ),
            // Unreachable through the engine's own path, which populates the
            // angle whenever it forces. Worded rather than unwrapped: a panic
            // in a disclosure would take the window with it, and half the fact
            // beats none.
            None => "Read as parallel because you asked.".to_owned(),
        }),
        (false, true) => Some(
            "Those two lines do not actually meet — the angle is measured at \
             the point where they would cross if extended."
                .to_owned(),
        ),
    }
}

/// ★★ **The disclosure a dragged perimeter vertex owes**: what the dimension
/// read before, and what it reads now.
///
/// # Why BOTH numbers, when one of them is on the page
///
/// Because the operator can see the new one and cannot see the old one — the
/// geometry it was measured from no longer exists. A line reading `13.85 m`
/// tells them nothing they did not already have; `12.40 m → 13.85 m` tells them
/// what their drag cost, which is the only fact here they cannot recover.
///
/// The engine carries `previous_label` on `VertexOutcome` for exactly this, and
/// it is the reason this sentence can exist at all: nothing on this side could
/// reconstruct it afterwards.
///
/// # Why it is off-canvas, and why that is the RULE rather than a preference
///
/// Rule 4, as narrowed by decision 059: the reshaped dimension and its new
/// label render **exactly as they will render after Save** — no tint, no badge,
/// no "recently changed" marking on the page. The change is stated here, on the
/// status row, where it does not become a second rendering path for the same
/// content. Two rendering paths drift; a sentence does not.
///
/// # And why it is silent when the number did not move
///
/// The caller drops this when `previous == current`. A corner dragged along its
/// own segment changes the shape and not the length, and reporting `13.85 m →
/// 13.85 m` would train the operator to ignore the one line that matters when
/// it does move.
#[must_use]
pub fn vertex_remeasured(previous: &str, current: &str) -> String {
    format!("That corner changed the measurement: {previous} is now {current}.")
}

#[cfg(test)]
mod tests {
    use super::*;
    /// ★ **The number an override overrode is in the sentence.**
    ///
    /// The one assertion this module exists for. A build that says "read as
    /// parallel" without the angle has a checkbox hiding the fact that makes
    /// the decision a decision — `pdfcer-core`'s own words, and the reason it
    /// populates the field precisely when the override fires.
    #[test]
    fn a_forced_parallel_reading_states_the_angle_it_overrode() {
        let note = two_line_reading(true, Some(0.8), false)
            .expect("a forced reading always says something");
        assert!(
            note.contains("0.80"),
            "the overridden angle is missing: {note}"
        );
        assert!(note.contains("because you asked"), "{note}");
    }

    /// An ordinary angular reading says nothing.
    ///
    /// Deliberate, and worth pinning: a sentence on every commit is a sentence
    /// nobody reads, and it would bury the two that matter.
    #[test]
    fn an_ordinary_reading_is_silent() {
        assert_eq!(two_line_reading(false, Some(37.0), false), None);
    }

    /// A virtual apex is disclosed, unasked.
    #[test]
    fn a_virtual_apex_says_the_lines_do_not_meet() {
        let note = two_line_reading(false, Some(37.0), true)
            .expect("a virtual apex is a fact the operator may not have noticed");
        assert!(note.contains("do not actually meet"), "{note}");
    }

    /// Both refusals are worded, and both say what to do next.
    ///
    /// The second half is the assertion with teeth. A refusal an operator
    /// cannot act on leaves them clicking the same two lines again.
    #[test]
    fn every_refusal_says_what_to_do_next() {
        for refusal in [TwoLineRefusal::Collinear, TwoLineRefusal::Degenerate] {
            let text = two_line_refused(refusal);
            assert!(!text.is_empty(), "{refusal:?} has no wording");
            assert!(
                text.contains("Pick a line"),
                "{refusal:?} states the problem and not the corrective: {text}"
            );
        }
    }
}
