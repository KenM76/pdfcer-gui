//! # `redact::reach` — how far a redaction is allowed to reach
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/redact/reach.md`.

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
    pub const ALL: &'static [Self] = &[Self::MarkedOnly, Self::HiddenCarriers, Self::WholeDocument];

    /// The token written to the preferences file.
    ///
    /// Stable across releases and deliberately not the display name, for
    /// `pdfcer_gui::app::prefs::quality::RenderQuality::key`'s reason.
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
