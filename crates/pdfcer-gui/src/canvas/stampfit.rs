//! # `canvas::stampfit` — the operator's standing answer to *"what happens
//! when a stamp's words do not fit its box?"*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/stampfit.md`.

use pdfcer_core::annot_author::StampFit;

/// The context-data key. A string constant rather than an inline literal so
/// that a second reader of this preference cannot open a *different* slot by
/// mistyping it — the failure that produces would be a control that appears to
/// do nothing, which is the hardest kind to see.
const KEY: &str = "canvas.stampfit"; // ui-text-exempt: context-data key, never displayed

/// The fit policies this shell offers, **in the order a control lists them**.
///
///
/// ⇒ The order is *least surprising first*. `GrowToText` changes the geometry
/// and shows it; `ShrinkToBox` changes the number the operator typed;
/// `ClipToBox` removes characters they typed. A list read downwards is a list
/// of increasing consequence, and the entry already selected is at the top for
/// a fresh gallery.
pub const FITS: &[StampFit] = &[
    StampFit::GrowToText,
    StampFit::ShrinkToBox,
    StampFit::ClipToBox,
];

/// The policy a shell with no stored preference uses. See the module header.
pub const DEFAULT: StampFit = StampFit::GrowToText;

/// Read the standing fit policy, or [`DEFAULT`] if the operator has not
/// chosen one this session.
#[must_use]
pub fn read(ctx: &egui::Context) -> StampFit {
    ctx.data(|d| d.get_temp::<StampFit>(egui::Id::new(KEY)))
        .unwrap_or(DEFAULT)
}

/// Write it back.
pub fn store(ctx: &egui::Context, fit: StampFit) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), fit));
}

/// This choice as a **stable token for a machine** — the diagnostic trace, and
/// nothing else.
///
/// Never `{:?}`. The rule is absolute on this project and it was bought:
/// a driven check once reported the opposite of the truth while quoting the
/// truth in its own failure message, because it was pattern-matching a
/// `Debug` rendering whose shape had changed underneath it. `Debug` belongs to
/// a programmer at a breakpoint; a trace field belongs to a parser, and a
/// parser needs a contract. This is the contract:
///
/// | policy | token |
/// |---|---|
/// | [`StampFit::GrowToText`] | `grow` |
/// | [`StampFit::ShrinkToBox`] | `shrink` |
/// | [`StampFit::ClipToBox`] | `clip` |
///
/// ⚠ `StampFit` is `#[non_exhaustive]`, so a fourth arm the engine adds lands
/// on `other` rather than failing to compile. That is the right trade here —
/// a trace token is not worth a build break — but a driven check reading
/// `fit=other` should be read as *this build does not know what it just asked
/// for*, not as a policy.
#[must_use]
pub const fn trace_token(fit: StampFit) -> &'static str {
    match fit {
        // ui-text-exempt: diagnostic trace tokens, never displayed.
        StampFit::GrowToText => "grow",
        StampFit::ShrinkToBox => "shrink",
        StampFit::ClipToBox => "clip",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The default is the ENGINE's default**, so that a shell that omits
    /// `stamp_fit` and a shell that names it explicitly ask for the same
    /// thing.
    ///
    /// `TextAnnotStyle::stamp_fit`'s doc: *"`None` means
    /// `StampFit::GrowToText`, the authoring default"*. If this constant ever
    /// disagreed, the properties panel and the placing dialog would apply
    /// different rules to the same stamp and neither would say so.
    #[test]
    fn the_default_is_grow_to_text() {
        assert_eq!(DEFAULT, StampFit::GrowToText);
    }

    /// Every policy this shell knows about is offered exactly once.
    ///
    /// The failure this catches is a list that quietly holds two of the
    /// three — a policy an operator can never reach, with no error anywhere.
    /// The same test `pen::FACES` has, for the same reason.
    #[test]
    fn every_policy_is_offered_exactly_once() {
        for fit in [
            StampFit::GrowToText,
            StampFit::ShrinkToBox,
            StampFit::ClipToBox,
        ] {
            assert_eq!(
                FITS.iter().filter(|f| **f == fit).count(),
                1,
                "each policy appears exactly once in FITS"
            );
        }
        assert_eq!(FITS.len(), 3);
    }

    /// The three tokens are distinct, which is the property a parser needs and
    /// the one a copy-paste breaks.
    #[test]
    fn the_trace_tokens_are_distinct() {
        let mut seen: Vec<&str> = FITS.iter().copied().map(trace_token).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), FITS.len());
    }
}
