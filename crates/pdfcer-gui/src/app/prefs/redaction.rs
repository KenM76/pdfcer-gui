//! # `app::prefs::redaction` — how far a redaction is allowed to reach
//!
//! One preference, [`RedactionReach`], and it is the only one in this directory
//! that changes what pdfcer **destroys** rather than what it draws.
//!
//! ## Why it is a preference and not a per-act question
//!
//! An operator's answer to *"may a redaction edit text I did not mark?"* is a
//! standing position about the work, not a decision made afresh per phrase. It
//! is also the answer that has to be in force **before** the removal runs, and
//! the apply dialog runs the removal the instant it opens — so a value that
//! only existed while that window was on screen would arrive one full rewrite
//! too late to have decided anything.
//!
//! ## The pairing with the engine, and why the shell holds its own copy
//!
//! Each variant maps one-to-one onto a `pdfcer_core::redact::ResidualScope`,
//! and [`RedactionReach::scope`] is the only place the two are joined. The
//! engine's enum is `#[non_exhaustive]`, so a `match` on it here would need a
//! catch-all arm — and a catch-all is exactly wrong for a control that offers
//! every value: a fourth scope added upstream must break this build and be
//! offered, not fall silently into a default the operator never chose.

/// **How far beyond the marked regions a redaction may act on text matching
/// what was redacted.**
///
/// Applying a mark removes the content under its geometry. That is not
/// configurable. What this chooses is the **residual sweep** that runs
/// afterwards, looking for the same text surviving somewhere the surgery did
/// not reach.
///
/// Every value reports everything it finds; they differ only in what they are
/// permitted to change. Narrowing this changes what pdfcer edits, never what it
/// tells you — a match a narrower setting declines to act on is counted and
/// named off-canvas either way.
use pdfcer_core::redact::ResidualScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedactionReach {
    /// Act only on the marked regions. The sweep reports and changes nothing.
    ///
    /// The literal reading of *"redact what I selected"*. It leaves the weakest
    /// absence guarantee of the three:
    /// a matching string in the document information dictionary or an XMP
    /// packet survives, disclosed.
    MarkedOnly,
    /// **The shipped answer.** Act on the marked regions, and additionally
    /// scrub carriers the operator cannot see — the document information
    /// dictionary, XMP packets, and string entries in arbitrary dictionaries.
    /// Leave drawable page content alone.
    ///
    /// An invisible carrier cannot have been deliberately kept: nobody selects
    /// `/Keywords` and nobody notices when it is scrubbed. Drawable content is
    /// the opposite — it is the document, the operator can see it, and removing
    /// an unmarked occurrence of a common word is destruction disguised as
    /// diligence.
    #[default]
    HiddenCarriers,
    /// Act on every occurrence found anywhere, including text drawn on pages
    /// the operator did not mark.
    ///
    /// The strongest absence guarantee and the most destructive.
    WholeDocument,
}

impl RedactionReach {
    /// Every value, in the order the settings window lists them.
    ///
    /// Narrowest to widest, so the control reads top to bottom as *less … more*
    /// destruction. Unlike `super::quality::RenderQuality::ALL` the default is
    /// **not** in the middle by accident: it is second because that is where
    /// the scale puts it, and a reader who stops at the first two has met the
    /// only two values that never edit a page they did not look at.
    pub const ALL: &'static [Self] = &[Self::MarkedOnly, Self::HiddenCarriers, Self::WholeDocument];

    /// The token written to the preferences file.
    ///
    /// Stable across releases and deliberately not the display name, for
    /// `super::quality::RenderQuality::key`'s reason.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::MarkedOnly => "marked-only",
            // ui-text-exempt: a file token, never displayed.
            Self::HiddenCarriers => "hidden-carriers",
            // ui-text-exempt: a file token, never displayed.
            Self::WholeDocument => "whole-document",
        }
    }

    /// Read a token back, or `None` if it names nothing.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|r| r.key() == key)
    }

    /// **The one place this choice becomes the engine's.**
    ///
    /// Exhaustive on this crate's enum by construction, which is the whole
    /// reason the shell carries one: the engine's `ResidualScope` is
    /// `#[non_exhaustive]` and a match on it would compile with a catch-all
    /// that silently swallowed a fourth value.
    #[must_use]
    pub const fn scope(self) -> ResidualScope {
        // Spelled out rather than aliased. `tools/gates/check-engine-api-drift.sh`
        // decides a variant is consumed by finding its `Owner::Variant`
        // spelling, so an alias here reads to the gate as three engine
        // variants nobody names -- the exact false negative it exists to
        // prevent, produced by the one file that does name them.
        match self {
            Self::MarkedOnly => ResidualScope::MarkedOnly,
            Self::HiddenCarriers => ResidualScope::HiddenCarriers,
            Self::WholeDocument => ResidualScope::WholeDocument,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The ladder is the engine's ladder**, asserted against the engine's own
    /// predicates rather than against this file's understanding of them.
    ///
    /// Each value carries a label the operator reads and acts on, and every one
    /// of those labels is a claim about what the engine will do. Hand-written,
    /// they are a snapshot of the engine on the day they were typed. Measured
    /// against `scrubs_hidden_carriers` and `blanks_content_streams`, a
    /// redefinition on the engine's side fails here — in the one place that can
    /// still stop the wrong sentence reaching him — rather than shipping as a
    /// setting that quietly no longer does what it says.
    #[test]
    fn each_reach_does_what_its_label_promises() {
        assert!(
            !RedactionReach::MarkedOnly.scope().scrubs_hidden_carriers(),
            "\"only what I marked\" claims the parts he cannot see are LEFT"
        );
        assert!(
            RedactionReach::HiddenCarriers
                .scope()
                .scrubs_hidden_carriers(),
            "the default's whole promise is the parts he cannot see"
        );
        assert!(
            RedactionReach::WholeDocument
                .scope()
                .scrubs_hidden_carriers(),
            "the widest reach cannot do less than the default"
        );

        assert!(
            !RedactionReach::MarkedOnly.scope().blanks_content_streams(),
            "★ a reach that edited drawing instructions outside the marks would \
             change what the pages PAINT, which is not what \"only what I \
             marked\" can be read to mean"
        );
        assert!(
            !RedactionReach::HiddenCarriers
                .scope()
                .blanks_content_streams(),
            "★ the DEFAULT must not edit drawable content beyond the marks: \
             that is the whole difference between it and the widest reach, and \
             it is the difference the operator is never asked about"
        );
        assert!(
            RedactionReach::WholeDocument
                .scope()
                .blanks_content_streams(),
            "\"everywhere the same text appears\" means the drawn copies too"
        );
    }

    /// **`ALL` is the whole enum, in widening order.**
    ///
    /// The settings group iterates `ALL`, so a value missing from it is a value
    /// the operator cannot choose and no other test would notice. The order is
    /// asserted too: the radio list reads as a ladder from least to most
    /// destructive, and a list that stopped being monotonic would be a control
    /// whose shape lies about what it does.
    #[test]
    fn every_reach_is_offered_and_the_ladder_only_widens() {
        assert_eq!(RedactionReach::ALL.len(), 3);
        let mut carriers = false;
        let mut streams = false;
        for reach in RedactionReach::ALL {
            let scope = reach.scope();
            assert!(
                scope.scrubs_hidden_carriers() >= carriers
                    && scope.blanks_content_streams() >= streams,
                "{reach:?} reaches LESS far than the value above it in the list"
            );
            carriers = scope.scrubs_hidden_carriers();
            streams = scope.blanks_content_streams();
        }
        assert!(carriers && streams, "the last rung must be the widest");
    }

    /// **Every key round-trips, and no two values share one.**
    ///
    /// `from_key` is the parser for `preferences.txt`; a duplicate key would
    /// make one value unreachable from a saved file while the settings window
    /// went on offering it.
    #[test]
    fn every_key_round_trips_and_is_unique() {
        let mut seen = Vec::new();
        for reach in RedactionReach::ALL {
            let key = reach.key();
            assert!(!seen.contains(&key), "two reaches share the key {key:?}");
            seen.push(key);
            assert_eq!(RedactionReach::from_key(key), Some(*reach));
        }
        assert_eq!(RedactionReach::from_key("wide-open"), None);
    }
}
