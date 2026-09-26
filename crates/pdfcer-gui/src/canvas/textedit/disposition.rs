//! # `canvas::textedit::disposition` — **which way the rest of the line moves**
//!
//! One public function, [`choose`], and the whole argument for its answer. It
//! decides the single field of
//! [`pdfcer_core::text_edit::EditOptions`](pdfcer_core::text_edit::EditOptions) —
//! the [`FollowerDisposition`] — which a caller that constructs
//! `EditOptions::default()` never decides at all.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/disposition.md`.

use pdfcer_core::text_edit::{AlignmentSource, BlockAlignment, DetectedAlignment};
use pdfcer_core::text_edit::{EditOptions, FollowerDisposition};

/// The engine's alignment finding, reduced to the two fields this decision
/// reads — **the argument type, and it is a pair rather than the struct on
/// purpose.**
pub type Finding = (BlockAlignment, AlignmentSource);

/// Reduce an engine detection to the pair [`choose`] reads.
///
/// The one place `DetectedAlignment`'s shape is known, so a future field on it
/// does not become a second thing the rule has to learn about.
#[must_use]
pub fn from_detection(d: DetectedAlignment) -> Finding {
    (d.alignment, d.source)
}

/// Axis-alignment tolerance for a text or transformation matrix off-diagonal.
pub const MTX_EPS: f64 = 1e-6;

/// **Why** a disposition was chosen — the operator-facing half of the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The run's `Tm` or CTM is rotated or skewed, so a follower shift computed
    /// in user-space x would be in the wrong frame. `Pin`.
    Rotated,
    /// The caret's **visual line is made of more than one show operator**,
    /// so what looks like one line is several independently positioned pieces.
    /// `Pin`.
    ///
    /// # Why this outranks the alignment rule
    ///
    /// Because on a multi-run line the "followers" are not a *tail of the same
    /// sentence* — they are **separate pieces of the drawing**, each with its
    /// own absolute `Tm`. A SolidWorks parts table writes one show operator per
    /// cell; a title block writes one per field. Under `Reflow` the engine adds
    /// `ΔA` to the `e` of every following absolute `Tm` in the text object, so
    /// widening `PART` to `PARTS` would slide `DESCRIPTION` and `QTY` sideways
    /// — **content the operator did not touch, moved by an edit that did not
    /// mention it.**
    ///
    /// `LeftAligned`'s argument — *"the line is meant to grow to the right"* —
    /// is true of a paragraph and false of a table row, and the alignment
    /// detector cannot tell them apart because both are left-flush. So this
    /// rung sits **above** it: a line made of separate pieces is not a line
    /// that grows, whatever its alignment reads as.
    ///
    /// # It is a disposition and NOT a refusal
    ///
    /// Refusing this shape outright — `canvas::textedit::resolve_run` answering
    /// `Refusal::SpansRuns` — refuses nearly every click on a CAD sheet, because
    /// a CAD sheet is made of multi-piece lines. The measurement is in that
    /// function's own comment. The useful half of a refusal is its
    /// **disclosure**, and that is what this reason carries — see
    /// `crate::text::textedit`.
    SharesTheLine,
    /// The engine detected a non-left alignment whose tail is flush against
    /// something. `Pin`.
    Flush(BlockAlignment),
    /// The engine detected left alignment. The line is meant to grow to the
    /// right. `Reflow`, which is also the engine's default.
    LeftAligned,
    /// The engine could not classify the block — one line, or no clear flush
    /// signal. `Reflow` as the engine's own default, **disclosed as a
    /// fall-back**.
    AlignmentUndetectable,
}

impl Reason {
    /// The disposition this reason implies.
    #[must_use]
    pub const fn disposition(self) -> FollowerDisposition {
        match self {
            Self::Rotated | Self::SharesTheLine | Self::Flush(_) => FollowerDisposition::Pin,
            Self::LeftAligned | Self::AlignmentUndetectable => FollowerDisposition::Reflow,
        }
    }

    /// Whether the operator is about to pay `Pin`'s cost — an untouched tail
    /// that does not make room for a longer replacement.
    #[must_use]
    pub const fn pins_the_tail(self) -> bool {
        matches!(self.disposition(), FollowerDisposition::Pin)
    }
}

/// **Whether a matrix pair is upright** — the engine's own axis-alignment test,
/// ported.
#[must_use]
pub fn is_upright(text_matrix: [f32; 6], ctm: [f32; 6]) -> bool {
    let off = |m: [f32; 6]| f64::from(m[1]).abs() <= MTX_EPS && f64::from(m[2]).abs() <= MTX_EPS;
    off(text_matrix) && off(ctm)
}

/// **The decision.** Which [`FollowerDisposition`] a commit on this run must
/// use, and why.
#[must_use]
pub fn choose(
    text_matrix: [f32; 6],
    ctm: [f32; 6],
    shares_the_line: bool,
    alignment: Option<Finding>,
) -> Reason {
    // Rung 1 — the correctness bound. See the header for why it outranks the
    // alignment rule rather than being folded in beside it.
    if !is_upright(text_matrix, ctm) {
        return Reason::Rotated;
    }
    // Rung 2 — the caret's line is several independently positioned pieces.
    //
    // Above alignment and below rotation, and both placements are arguments.
    // Below rotation because rotation is the *correctness* bound — a follower
    // shift computed in user-space x on a rotated baseline is wrong in a way
    // that has nothing to do with how many pieces the line has, and a reader
    // who sees `Rotated` should be told the sharper fact.
    //
    // Above alignment because the alignment detector cannot see this: a table
    // row and a paragraph are both left-flush, and `LeftAligned`'s argument
    // ("the line is meant to grow to the right") is true of one and false of the
    // other. See `Reason::SharesTheLine`.
    if shares_the_line {
        return Reason::SharesTheLine;
    }
    // Rung 2 — the engine's finding, and only when it IS a finding.
    //
    // `source` is consulted as well as the alignment because
    // `DetectedAlignment::alignment` reads `Left` in three distinct situations
    // and only one of them means "this block is left aligned": `Detected` is
    // the measurement, `SingleLineDefault` and `AmbiguousDefault` are the
    // engine saying it could not tell. `Overridden` cannot arrive here —
    // nothing in this shell overrides an alignment on the edit path — but it is
    // spelled rather than swept into the wildcard, because it is an operator's
    // *statement* about the block and is therefore at least as good as a
    // measurement.
    //
    // The trailing `_` is not laziness: both enums are `#[non_exhaustive]`, so
    // a `match` from outside `pdfcer-core` must carry one. It answers
    // `AlignmentUndetectable`, which is the safe direction — a source this
    // shell has never heard of is by definition one it cannot interpret, and
    // treating it as a finding would be the shell claiming to have read
    // something it has not.
    match alignment {
        Some((a, AlignmentSource::Detected | AlignmentSource::Overridden)) => match a {
            BlockAlignment::Left => Reason::LeftAligned,
            other => Reason::Flush(other),
        },
        Some((_, AlignmentSource::SingleLineDefault | AlignmentSource::AmbiguousDefault)) => {
            Reason::AlignmentUndetectable
        }
        Some(_) | None => Reason::AlignmentUndetectable,
    }
}

/// The [`EditOptions`] a commit built from `reason` must carry.
#[must_use]
pub fn options(reason: Reason) -> EditOptions {
    EditOptions::default().with_disposition(reason.disposition())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An upright text matrix at unit scale.
    const UPRIGHT: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

    /// A quarter turn — the shape a SolidWorks title block's side text has.
    /// `[cos, sin, -sin, cos, e, f]` at 90°: `[0, 1, -1, 0, …]`.
    const ROTATED_90: [f32; 6] = [0.0, 1.0, -1.0, 0.0, 100.0, 100.0];

    /// A skew with no rotation — `c` non-zero, `b` zero. Included because a
    /// guard that tested only `b` would pass this, and an italic-by-matrix
    /// synthetic oblique is exactly this matrix.
    const SKEWED: [f32; 6] = [1.0, 0.0, 0.21, 1.0, 0.0, 0.0];

    // =======================================================================
    // The rotation guard — D4b case 2
    // =======================================================================

    /// **An upright matrix pair is upright.** The floor: if this were false
    /// every edit would pin and the fix would look like it worked.
    #[test]
    fn an_upright_matrix_pair_is_upright() {
        assert!(is_upright(UPRIGHT, UPRIGHT));
    }

    /// **A quarter turn in the TEXT matrix is caught.**
    #[test]
    fn a_rotated_text_matrix_is_not_upright() {
        assert!(!is_upright(ROTATED_90, UPRIGHT));
    }

    /// **A quarter turn in the CTM is caught, with an upright `Tm`.**
    #[test]
    fn a_rotated_ctm_is_not_upright_even_with_an_upright_text_matrix() {
        assert!(!is_upright(UPRIGHT, ROTATED_90));
    }

    /// **A skew with `b = 0` is caught**, which a one-term guard would not
    /// be. `reflow_apply`'s own test is `|b| > eps || |c| > eps`, both terms.
    #[test]
    fn a_skewed_matrix_with_a_zero_b_term_is_not_upright() {
        assert!(!is_upright(SKEWED, UPRIGHT));
        assert!(!is_upright(UPRIGHT, SKEWED));
    }

    /// **Floating-point dust is not rotation.** A matrix whose off-diagonals
    /// are at the tolerance is upright; one an order of magnitude above it is
    /// not. Without this a page written by a producer that rounds its matrices
    /// would pin every edit for no reason.
    #[test]
    fn the_tolerance_is_the_engines_and_separates_dust_from_rotation() {
        let dust = [1.0, 1e-7, 1e-7, 1.0, 0.0, 0.0];
        let real = [1.0, 1e-5, 0.0, 1.0, 0.0, 0.0];
        assert!(is_upright(dust, UPRIGHT), "1e-7 is below MTX_EPS");
        assert!(!is_upright(real, UPRIGHT), "1e-5 is above MTX_EPS");
    }

    /// **Rotation pins, whatever the alignment says.**
    #[test]
    fn a_rotated_run_pins_regardless_of_alignment() {
        assert_eq!(choose(ROTATED_90, UPRIGHT, false, None), Reason::Rotated);
        assert_eq!(choose(UPRIGHT, ROTATED_90, false, None), Reason::Rotated);
        assert_eq!(
            choose(ROTATED_90, UPRIGHT, false, None).disposition(),
            FollowerDisposition::Pin
        );
    }

    // =======================================================================
    // The fall-back, and the fact that it is disclosed as one
    // =======================================================================

    /// **No block resolved is not "left aligned".**
    #[test]
    fn an_unresolvable_block_is_an_undetectable_alignment_and_not_a_left_one() {
        let r = choose(UPRIGHT, UPRIGHT, false, None);
        assert_eq!(r, Reason::AlignmentUndetectable);
        assert_ne!(r, Reason::LeftAligned);
        assert_eq!(r.disposition(), FollowerDisposition::Reflow);
    }

    /// **A line made of several pieces PINS, whatever its alignment reads
    /// as** — the assertion the whole multi-run rung rests on.
    #[test]
    fn a_line_made_of_several_pieces_pins_over_a_left_alignment() {
        let left = Some((BlockAlignment::Left, AlignmentSource::Detected));
        assert_eq!(
            choose(UPRIGHT, UPRIGHT, false, left),
            Reason::LeftAligned,
            "the control: without the multi-run fact this run reflows"
        );
        assert_eq!(
            choose(UPRIGHT, UPRIGHT, true, left),
            Reason::SharesTheLine,
            "a left-aligned run on a multi-piece line must not reflow: that is a table row"
        );
        assert_eq!(
            choose(UPRIGHT, UPRIGHT, true, left).disposition(),
            FollowerDisposition::Pin
        );
    }

    /// **Rotation still outranks it**, which is the other half of the rung
    /// order.
    #[test]
    fn rotation_outranks_the_multi_run_rung() {
        assert_eq!(
            choose(ROTATED_90, UPRIGHT, true, None),
            Reason::Rotated,
            "a rotated run on a multi-piece line must report the rotation, the sharper fact"
        );
    }

    /// **`Reflow` is what the engine defaults to**, so the fall-back is
    /// byte-identical to what a caller that never decided would have passed.
    #[test]
    fn the_fallback_is_byte_identical_to_what_the_old_shell_passed() {
        let fallback = options(choose(UPRIGHT, UPRIGHT, false, None));
        assert_eq!(fallback.disposition, EditOptions::default().disposition);
    }

    /// **`pins_the_tail` agrees with `disposition`, for every reason.**
    #[test]
    fn pins_the_tail_agrees_with_the_disposition_for_every_reason() {
        for r in [
            Reason::Rotated,
            Reason::Flush(BlockAlignment::Right),
            Reason::Flush(BlockAlignment::Center),
            Reason::Flush(BlockAlignment::Justified),
            Reason::LeftAligned,
            Reason::AlignmentUndetectable,
        ] {
            assert_eq!(
                r.pins_the_tail(),
                r.disposition() == FollowerDisposition::Pin,
                "{r:?}"
            );
            assert_eq!(options(r).disposition, r.disposition(), "{r:?}");
        }
    }

    /// **Every non-left alignment pins**, stated over the enum rather than
    /// over the three names, so a fifth `BlockAlignment` variant added upstream
    /// fails here instead of silently reflowing.
    #[test]
    fn every_non_left_alignment_pins_and_left_reflows() {
        for a in [
            BlockAlignment::Right,
            BlockAlignment::Center,
            BlockAlignment::Justified,
        ] {
            assert_eq!(
                Reason::Flush(a).disposition(),
                FollowerDisposition::Pin,
                "{a:?} must pin — its tail is flush against something"
            );
        }
        assert_eq!(
            Reason::LeftAligned.disposition(),
            FollowerDisposition::Reflow
        );
    }
}
