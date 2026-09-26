//! # `text::measure` — what the measure tools say about what they inferred
//!
//! ## Rule 15
//!
//! What these tools author is a **ce dimension** — one pdfcer writes. A **pdf
//! dimension** is CAD-exported page content pdfcer reads and must not alter.
//! This catalog avoids the bare word, exactly as [`crate::text::scale`] and
//! [`crate::text::dimension_groups`] do.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/measure.md`.

use pdfcer_core::dimension::TwoLineRefusal;

/// Why a two-line pick could not author anything.
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

/// **What the two-line tool read the pair as** — the disclosure obligation
/// this build was not meeting.
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

/// **The disclosure a dragged perimeter vertex owes**: what the dimension
/// read before, and what it reads now.
#[must_use]
pub fn vertex_remeasured(previous: &str, current: &str) -> String {
    format!("That corner changed the measurement: {previous} is now {current}.")
}

/// **The disclosure an ADDED corner owes** — 2026-09-05.
#[must_use]
pub fn vertex_inserted(corners: usize, previous: &str, current: &str) -> String {
    format!("A corner was added — {corners} corners now, and {previous} is now {current}.")
}

/// **The disclosure a REMOVED corner owes** — [`vertex_inserted`]'s twin,
/// and the sentence the operator's own report of 2026-09-05 was about:
#[must_use]
pub fn vertex_removed(corners: usize, previous: &str, current: &str) -> String {
    format!("A corner was removed — {corners} corners now, and {previous} is now {current}.")
}

/// **Why a corner could not be added or taken away** — the shell's own reading
/// of the engine's refusals, 2026-09-05.
///
/// # A `Copy` enum rather than a `String`, and the reason is structural
///
/// `pdfcer_gui::app::status::decline::Declined` is `Copy` and its `line()` returns
/// `&'static str`. Both properties are load-bearing there — see that type — and
/// carrying the engine's own `Display` prose would break the second and would
/// also violate `check-ui-strings.sh`' exclusion 3, which says in as many words
/// that an error type's `Display` being exempt *"is not permission to route UI
/// text through an error type"*. This is [`crate::text::status::TextStyleRefusal`]'s
/// shape, adopted rather than re-argued.
///
/// # Why every one of these is worded, including the two that should be
/// unreachable
///
/// Because a corner handle is a **grip**, and this project's founding defect
/// shape is a grip that is dragged, released and does nothing with no
/// explanation. The operator's report that produced this whole surface was
/// exactly that shape one level up: he could not delete a node and nothing
/// anywhere said why. A refusal with a sentence is a limitation; a refusal
/// without one is a broken program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexEditRefusal {
    /// `EditError::PerimeterWouldBeDegenerate` — the shape has the fewest
    /// corners it can have.
    ///
    /// The reachable one, and the one an operator meets by doing something
    /// perfectly reasonable: a triangle they want to make into a line. The
    /// minimums are the engine's and they differ by shape — an open path keeps
    /// **two**, because one corner has no segment and therefore no length; a
    /// closed one keeps **three**, because two closed corners trace a line
    /// there and back and would print twice the distance between two points,
    /// attached to something that does not look like it.
    WouldLeaveTooFew,
    /// `EditError::DimensionVertexCountFixed` — a **linear** ce dimension is
    /// structurally two points.
    ///
    /// Its ends can be moved and their number cannot change: a length between
    /// three points is not a length. Reachable the moment a linear ce
    /// dimension grows corner handles, which it has not yet — see this
    /// module's note and `canvas::dimdrag::vertices`.
    CountFixed,
    /// `EditError::VertexNotPlaceable` — the coordinate is not a usable page
    /// value.
    ///
    /// A non-finite or absurd number, which on this canvas means the
    /// page-space conversion produced something the format cannot hold. Not an
    /// operator mistake, and the sentence does not imply one.
    Unplaceable,
    /// Everything else the engine can say no with: an unknown record, a stale
    /// group id, an index that names nothing, an encrypted document, an
    /// enforced certification, a sidecar written by a newer build.
    ///
    /// One sentence for all of them rather than six, because they divide into
    /// *cannot happen from a handle this shell drew* and *is a property of the
    /// file that no wording about corners would help with*, and neither class
    /// gives the operator a next act about corners. What they do get is the
    /// knowledge that the press was heard.
    Refused,
}

impl VertexEditRefusal {
    /// The sentence, for `Declined::line`.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::WouldLeaveTooFew => {
                "That shape has as few corners as it can have. Delete the whole \
                 measurement instead, or add a corner before taking one away."
            }
            Self::CountFixed => {
                "A straight measurement is two points. You can move either end, \
                 but not add or remove one."
            }
            Self::Unplaceable => {
                "That corner cannot go there — the position is off the page's \
                 usable range."
            }
            Self::Refused => "The corner could not be changed. The drawing is unchanged.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// **The number an override overrode is in the sentence.**
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
