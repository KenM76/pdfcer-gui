//! # `canvas::stampfit` — the operator's standing answer to *"what happens
//! when a stamp's words do not fit its box?"*
//!
//! One preference, read and written through the egui context, shared by the
//! **two** surfaces that can ask a stamp's label to change size:
//!
//! * `dialogs::textannot` — placing a new stamp, where the operator picks a
//!   size before the mark exists;
//! * `panels::properties::markup::textannot` — retyping the size of a stamp
//!   already on the page, which `pdfcer-core` `Pass 292.0` made possible.
//!
//! ## ★★★ Why this is a preference and not a property read off the file
//!
//! Because **nothing in a PDF records the author's fit intent**, and the
//! engine says so in the field's own doc rather than leaving it to be
//! discovered:
//!
//! > It is a caller's choice rather than a recovered one because *nothing in
//! > the file records the author's fit intent*, and a guess from the current
//! > geometry would invent a decision nobody made. Same reason
//! > `text_spec_from_dict` does not recover it.
//!
//! ⇒ A stamp on the page can be asked *what size are your words* (there is a
//! `Tf` in its appearance, or a `/DA`) and cannot be asked *what did your
//! author want to happen if they stopped fitting* — that question has no
//! answer in the bytes. A control that opened on a per-stamp value would
//! therefore be showing a value it had made up, differently for each stamp,
//! with nothing to compare against. A **standing preference** is the honest
//! shape: it says *this is what I want done*, which is a fact about the
//! operator and is exactly what it claims to be.
//!
//! ## ★★ Why the two surfaces share ONE preference rather than each keeping
//! ## their own
//!
//! Because they are the same question asked twice about the same stamp, and
//! an operator who has answered it once at placement time has not changed
//! their mind by opening a properties panel. Two independent stores would let
//! a stamp be placed under *make the stamp wider* and resized under *cut the
//! words off*, with no surface anywhere showing that the rule had changed
//! between the two acts.
//!
//! ⚠ **It is `insert_temp`, so it does not survive a restart**, exactly like
//! `canvas::textedit::pen`. That is deliberate for now: a preference that
//! persists into the settings file is one that needs a row in the settings
//! window, a default the settings window can restore, and a migration when the
//! engine's enum grows a fourth arm. If the operator reports re-choosing it
//! every session, promote it — the read/write pair here is the seam that makes
//! that a one-file change.
//!
//! ## The default, and why it is the engine's default rather than a choice
//! ## made here
//!
//! [`pdfcer_core::annot_author::StampFit::GrowToText`]. Not because it is
//! first in the enum but because it is the only one of the three that **cannot
//! be quietly wrong** (R8b rule 4): the box widens, the operator sees a wider
//! stamp, and the inference discloses itself by being visible. The other two
//! change what is drawn *without changing what is seen to have been asked
//! for* — a smaller label, or a shorter one — and each therefore owes an
//! off-canvas sentence, which is why
//! `text::panels::textannotstyle::stamp_label_shrunk` and `…_clipped` exist.

use pdfcer_core::annot_author::StampFit;

/// The context-data key. A string constant rather than an inline literal so
/// that a second reader of this preference cannot open a *different* slot by
/// mistyping it — the failure that produces would be a control that appears to
/// do nothing, which is the hardest kind to see.
const KEY: &str = "canvas.stampfit"; // ui-text-exempt: context-data key, never displayed

/// The fit policies this shell offers, **in the order a control lists them**.
///
/// ★★★ **All three, and the middle two are the point of this pass.**
/// `canvas::textannot::StampSize`'s header used to argue at length that
/// `ShrinkToBox` and `ClipToBox` could not be offered, because the size the
/// engine actually drew at was not reported and offering them would mean
/// either staying silent about an inference or re-deriving the engine's rule
/// here. That argument was correct, was filed rather than worked around, and
/// **`Pass 291.0` answered it**: `StampLabelFit` reports the drawn size, the
/// requested size, the hidden-character count and the grown width, and
/// `is_inference()` says which of the four outcomes owes a sentence at all.
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
/// ★★★ Never `{:?}`. The rule is absolute on this project and it was bought:
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

    /// ★★ **The default is the ENGINE's default**, so that a shell that omits
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
    /// ★ The failure this catches is a list that quietly holds two of the
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
