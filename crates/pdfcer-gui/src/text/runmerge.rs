//! Copy for `format.merge_text_runs`: why merging text runs was declined, and
//! the width sentence when the engine's report names none.
//!
//! The engine's own disclosures state the merge itself; this module words only
//! what the operator could not otherwise see.

use pdfcer_core::text_edit::FormatError;
use pdfcer_core::vector::VectorEditError;

/// Why the selected runs were not merged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMergeRefusal {
    /// Fewer than two runs were selected.
    NeedsTwoRuns,
    /// The runs are not neighbours in the text.
    NotNeighbours,
    /// The run after the merge takes its position from the last merged run.
    WouldMoveNextRun,
    /// A run differs from the first in font, size, spacing, rise, rendering or colour.
    StylesDiffer,
    /// A run is shown with `'` or `"`, which also starts a new line.
    StartsNewLine,
    /// A marked-content boundary lies between the runs.
    CrossesTaggedContent,
    /// The runs use a composite (Type 0) font.
    CompositeFont,
    /// The span fit had no width to cover, or the positions could not be measured.
    NoWidthToSpan,
    /// The document is encrypted.
    Encrypted,
    /// The selected runs no longer exist as selected.
    Stale,
    /// Anything else the engine refused.
    Other,
}

impl RunMergeRefusal {
    /// Classify the preflight's answer.
    #[must_use]
    pub fn of_vector(error: &VectorEditError) -> Self {
        match error {
            VectorEditError::MergeNeedsTwoRuns { .. } => Self::NeedsTwoRuns,
            VectorEditError::MergeRunsNotContiguous { .. } => Self::NotNeighbours,
            VectorEditError::MergeWouldMoveNextRun { .. } => Self::WouldMoveNextRun,
            VectorEditError::TextRunOutOfRange { .. }
            | VectorEditError::ObjectOutOfRange { .. } => Self::Stale,
            _ => Self::Other,
        }
    }

    /// Classify the verb's answer.
    #[must_use]
    pub fn of_format(error: &FormatError) -> Self {
        match error {
            FormatError::TextRun(inner) => Self::of_vector(inner),
            FormatError::MergeStateDiffers { .. } => Self::StylesDiffer,
            FormatError::MergeLineShowOperator { .. } => Self::StartsNewLine,
            FormatError::MergeCrossesMarkedContent => Self::CrossesTaggedContent,
            FormatError::MergeCompositeFont { .. } => Self::CompositeFont,
            FormatError::MergeRunsOutOfOrder | FormatError::MergePositionUnknown => {
                Self::NoWidthToSpan
            }
            FormatError::Encrypted => Self::Encrypted,
            FormatError::PageIndex(_) => Self::Stale,
            _ => Self::Other,
        }
    }

    /// The status-line sentence.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::NeedsTwoRuns => {
                "Merging needs at least two text runs. Select a line made of more than one run."
            }
            Self::NotNeighbours => {
                "Only text runs that follow one another can be merged. Nothing was changed."
            }
            Self::WouldMoveNextRun => {
                "Merging these runs would move the text after them. Include that text in the \
                 selection, or split it off first. Nothing was changed."
            }
            Self::StylesDiffer => {
                "These text runs differ in font, size, spacing, colour or rendering, so they \
                 cannot be shown as one. Nothing was changed."
            }
            Self::StartsNewLine => {
                "One of these text runs also starts a new line, so it cannot be merged with \
                 its neighbours. Nothing was changed."
            }
            Self::CrossesTaggedContent => {
                "These text runs belong to different tagged sections of the page, so they \
                 cannot be merged. Nothing was changed."
            }
            Self::CompositeFont => {
                "These text runs use a composite font, which merging does not support yet. \
                 Nothing was changed."
            }
            Self::NoWidthToSpan => {
                "pdfcer could not measure where these text runs start and end, so it cannot \
                 stretch the merged run over them. Nothing was changed."
            }
            Self::Encrypted => {
                "This document is encrypted, so its text runs cannot be merged. Nothing was \
                 changed."
            }
            Self::Stale => "The selected text changed before the merge ran. Select it again.",
            Self::Other => "pdfcer could not merge these text runs. Nothing was changed.",
        }
    }
}

/// The disclosure for a width change, when the engine's report states none.
#[must_use]
pub fn width_changed(before: f64, after: f64) -> String {
    format!("The merged text was stretched from {before:.0}% to {after:.0}% of its normal width.")
}

#[cfg(test)]
mod tests {
    use super::RunMergeRefusal as R;

    const ALL: [R; 11] = [
        R::NeedsTwoRuns,
        R::NotNeighbours,
        R::WouldMoveNextRun,
        R::StylesDiffer,
        R::StartsNewLine,
        R::CrossesTaggedContent,
        R::CompositeFont,
        R::NoWidthToSpan,
        R::Encrypted,
        R::Stale,
        R::Other,
    ];

    #[test]
    fn every_refusal_is_a_distinct_sentence() {
        for (i, a) in ALL.iter().enumerate() {
            assert!(a.line().ends_with('.'), "{a:?}");
            for b in &ALL[i + 1..] {
                assert_ne!(a.line(), b.line(), "{a:?} and {b:?} share a sentence");
            }
        }
    }

    #[test]
    fn the_preflight_refusals_classify_by_name() {
        use pdfcer_core::vector::VectorEditError as E;
        assert_eq!(
            R::of_vector(&E::MergeNeedsTwoRuns { count: 1 }),
            R::NeedsTwoRuns
        );
        assert_eq!(
            R::of_vector(&E::MergeRunsNotContiguous { after: 1, next: 3 }),
            R::NotNeighbours
        );
        assert_eq!(
            R::of_vector(&E::MergeWouldMoveNextRun { index: 4 }),
            R::WouldMoveNextRun
        );
    }
}
