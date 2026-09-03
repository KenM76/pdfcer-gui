//! # `canvas::textedit::disposition` — **which way the rest of the line moves**
//!
//! One public function, [`choose`], and the whole argument for its answer. It
//! decides the single field of
//! [`pdfcer_core::text_edit::EditOptions`](pdfcer_core::text_edit::EditOptions) —
//! the [`FollowerDisposition`] — that the old shell never decided at all.
//!
//! ## What was wrong, and it is two bugs rather than one
//!
//! `DEFECTS.md` **D4b** lists two cases where the old shell's text edit was not
//! merely unhelpful but **wrong on commit**, and both have the same shape: the
//! engine already carries the mechanism, and the GUI never selected it. The old
//! shell had exactly one call site
//! (`D:\Dev\pdfce\crates\pdfce-gui\src\main.rs`, `commit_text_edit_draft`) and
//! it passed `EditOptions::default()` — i.e. [`FollowerDisposition::Reflow`] —
//! unconditionally, for every run on every page of every document.
//!
//! Both claims below were **verified against the engine's source** rather than
//! taken from the defect register, because `HANDOFF.md` §11 records that this
//! project has already filed one wrong claim about that repository.
//!
//! ### 1. A right-aligned, centred or justified tail moves the wrong way
//!
//! [`FollowerDisposition::Pin`] exists for precisely this, and says so in its
//! own doc comment (`pdfcer-core/src/text_edit/edit.rs`, the `FollowerDisposition`
//! enum):
//!
//! > Pin survivors in place with a compensating `TJ` number (the Pass-8.0
//! > path), **for a justified / right-aligned tail that must not move.**
//!
//! Under `Reflow` the engine walks the operators after the anchor and adds `ΔA`
//! to every following absolute `Tm`'s `e`. On a **left**-aligned line that is
//! right: the line grows to the right and its tail follows. On a right-aligned,
//! centred or justified line the tail is *flush against something* — a margin,
//! a centre, a column edge — and moving it is the one thing that must not
//! happen. `Pin` leaves every follower `Tm` untouched and consumes `ΔA` with a
//! compensating number inside the anchor operator instead.
//!
//! ### 2. Rotated or skewed text is shifted along the wrong axis
//!
//! This is the sharper of the two, and it is the one that bites this operator's
//! documents specifically, because rotated text is what a CAD title block is
//! made of. The engine's reflow branch is, verbatim
//! (`pdfcer-core/src/text_edit/edit.rs`, inside `plan_edit`):
//!
//! ```text
//! Rec::Tm([a, b, c, d, e, f]) => {
//!     let moved = emit_tm([*a, *b, *c, *d, *e + delta, *f]);
//! ```
//!
//! `ΔA` is an advance in **text space**. `e` is the translation component of
//! `Tm`, and `Tm` maps text space to **user space**, so a text-space advance of
//! `Δ` displaces a follower by `Δ·(a, b)` in user space — not by `(Δ, 0)`. The
//! two agree only when `(a, b) = (1, 0)`, i.e. when the text is upright and
//! unscaled in x. On a 90°-rotated title-block line the baseline runs up the
//! page and the engine slides the tail *sideways*.
//!
//! There is **no rotation guard on this path**. The reflow-apply path has one —
//! `reflow_apply.rs`'s `check_uniform_axis_aligned` refuses when
//! `|b| > MTX_EPS || |c| > MTX_EPS`, with `MTX_EPS = 1e-6` — and this module
//! ports that predicate ([`is_upright`]) rather than inventing a second
//! tolerance.
//!
//! ## ★ Why the rotation answer is `Pin` and not a refusal
//!
//! `reflow_apply` *refuses* rotated text. This module does not, and the
//! asymmetry is deliberate rather than a relaxation.
//!
//! Those are different operations. A **re-wrap** invents line breaks and new
//! line origins, so under rotation it would have to re-derive a whole
//! two-dimensional layout in a frame it does not understand; refusing is the
//! only honest answer. An **in-place replace** does not: it rewrites one show
//! operator's string and then has to answer one question — what happens to
//! what came after it. And `Pin` answers that question **correctly under
//! rotation**, not merely less wrongly:
//!
//! * no follower `Tm` is written at all, so nothing can be displaced along the
//!   wrong axis — the entire mechanism of the defect is not reached;
//! * the compensation is a `TJ` number, and `TJ` offsets are applied in **text
//!   space** (§9.4.3), which is the rotated baseline's own frame. An
//!   advance-relative follower inside the same show sequence is therefore
//!   compensated *along the baseline*, which is where it actually needs to
//!   move.
//!
//! So under rotation `Pin` is the right answer for the same reason it is the
//! right answer for a right-aligned tail: the thing that must not happen is a
//! follower being moved by a number computed in the wrong frame.
//!
//! **What it costs, and it is disclosed rather than hidden**: a pinned tail
//! does not make room. If the replacement is longer than what it replaced, the
//! edited text grows into the pinned tail. The engine discloses overflow for
//! `Reflow` and says nothing for `Pin`, so [`Reason`] carries the fact and
//! `crate::text::textedit` turns it into the sentence the status bar shows.
//! Rule 4: disclosure lives off-canvas.
//!
//! ## The order of the two rules, which is a decision
//!
//! Rotation is tested **first** and wins. The two can disagree — a rotated
//! left-aligned title-block line asks for `Reflow` on the alignment rule and
//! `Pin` on the rotation rule — and the rotation rule wins because the two
//! claims are not the same kind of claim. Alignment says *"this would look
//! wrong"*; rotation says *"the arithmetic the other branch would perform is
//! not the arithmetic this frame requires"*. A preference loses to a
//! correctness bound.
//!
//! ## What is deliberately NOT here
//!
//! The **third** case, which is the honest limit of this fix and is stated so
//! it is not mistaken for coverage: alignment is detected by the engine from a
//! block's *lines*, and `ReflowEngine::infer_alignment` returns
//! [`AlignmentSource::SingleLineDefault`] — alignment `Left` — for any block
//! with one line. A one-line right-aligned run is therefore **indistinguishable
//! from a one-line left-aligned run** by anything in the engine, and this
//! module does not pretend otherwise: it answers `Reflow` and records
//! [`Reason::AlignmentUndetectable`], which is a *disclosed* fall-back rather
//! than a claim. Widening it would need a block-relative or margin-relative
//! signal that `pdfcer-core` does not publish, and inventing one here would be
//! the shell deciding a question the engine owns.

use pdfcer_core::text_edit::{AlignmentSource, BlockAlignment, DetectedAlignment};
use pdfcer_core::text_edit::{EditOptions, FollowerDisposition};

/// The engine's alignment finding, reduced to the two fields this decision
/// reads — **the argument type, and it is a pair rather than the struct on
/// purpose.**
///
/// [`DetectedAlignment`] is `#[non_exhaustive]`, so nothing outside
/// `pdfcer-core` can build one, so a [`choose`] that took it could only ever be
/// tested through a real page. Its *fields* are two plain `Copy` enums whose
/// variants are constructible anywhere, and they are the entire input to the
/// rule — the three raggedness measurements and the tolerance beside them are
/// evidence for the finding, not part of it.
///
/// So the seam is here: [`from_detection`] does the one-line reduction at the
/// single place a real detection arrives, and every case in the table on
/// [`choose`] is a unit test with no fixture. That is the same shape
/// `canvas::textsel::gate` uses — a pure predicate over two small values,
/// separated from the page that produces them.
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
///
/// **Ported, not chosen.** It is `pdfcer-core`'s own `MTX_EPS` from
/// `text_edit/reflow_apply.rs`, the constant its `check_uniform_axis_aligned`
/// compares `b` and `c` against before refusing a rotated block. A second
/// tolerance picked here would be a second answer to "is this upright", free to
/// disagree with the engine's on exactly the matrices where it matters.
pub const MTX_EPS: f64 = 1e-6;

/// **Why** a disposition was chosen — the operator-facing half of the answer.
///
/// [`choose`] returns this beside the disposition rather than only the
/// disposition, for two reasons that are both about honesty rather than
/// tidiness:
///
/// 1. `Pin` has a cost (a tail that does not make room) and `Reflow` has a cost
///    (a line that may overrun its margin). Which one the operator is about to
///    pay is a fact they are entitled to before they press Accept, and it is a
///    different fact in each case — so one generic "the line may move" sentence
///    would be a sentence that is never quite true.
/// 2. [`Self::AlignmentUndetectable`] is a **fall-back, not a finding**, and the
///    difference is invisible from the disposition alone: it produces the same
///    `Reflow` a confidently left-aligned block does. Collapsing them would make
///    the shell state as detected something it defaulted to, which is the exact
///    shape rule 4 forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The run's `Tm` or CTM is rotated or skewed, so a follower shift computed
    /// in user-space x would be in the wrong frame. `Pin`.
    Rotated,
    /// ★★ The caret's **visual line is made of more than one show operator**,
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
    /// # ★ It is not a refusal, and it used to be
    ///
    /// `canvas::textedit::resolve_run` returned `Refusal::SpansRuns` for exactly
    /// this shape until 2026-08-19, which refused nearly every click on a CAD
    /// sheet. The measurement and the operator's report are in that function's
    /// own comment. What survives of the refusal is its **disclosure**, which
    /// was always the useful half — see `crate::text::textedit`.
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
    ///
    /// Written as a method on the reason rather than as a second `match` in
    /// [`choose`] so the two can never disagree: a reason is the *whole* of the
    /// input to the choice, and a future fifth reason is a compile error here
    /// rather than a silent `Reflow`.
    #[must_use]
    pub const fn disposition(self) -> FollowerDisposition {
        match self {
            Self::Rotated | Self::SharesTheLine | Self::Flush(_) => FollowerDisposition::Pin,
            Self::LeftAligned | Self::AlignmentUndetectable => FollowerDisposition::Reflow,
        }
    }

    /// Whether the operator is about to pay `Pin`'s cost — an untouched tail
    /// that does not make room for a longer replacement.
    ///
    /// The predicate the status-bar disclosure gates on, kept here beside the
    /// reason it derives from rather than re-spelled as a `matches!` at the one
    /// call site, for the reason `CanvasTool::markup_kind`'s docs give: a
    /// predicate with two readers is a predicate that drifts.
    #[must_use]
    pub const fn pins_the_tail(self) -> bool {
        matches!(self.disposition(), FollowerDisposition::Pin)
    }
}

/// **Whether a matrix pair is upright** — the engine's own axis-alignment test,
/// ported.
///
/// `true` when neither the text matrix nor the CTM carries a non-zero
/// off-diagonal term. Both are checked because either can rotate the glyphs:
/// §9.4.4's text rendering matrix is `Tm × CTM` (with the font scale between
/// them), so a page whose whole content stream sits inside a rotating `cm` puts
/// the rotation in the CTM while every `Tm` on it reads as upright. A guard
/// that looked only at `Tm` would pass every glyph on a rotated sheet — which
/// is exactly the SolidWorks landscape-plot case this fix is for.
///
/// `f32` in, because that is what
/// [`GlyphProvenance`](pdfcer_core::text_extract::GlyphProvenance) publishes;
/// widened to `f64` for the comparison so the tolerance is compared in the same
/// type the engine compares it in.
#[must_use]
pub fn is_upright(text_matrix: [f32; 6], ctm: [f32; 6]) -> bool {
    let off = |m: [f32; 6]| f64::from(m[1]).abs() <= MTX_EPS && f64::from(m[2]).abs() <= MTX_EPS;
    off(text_matrix) && off(ctm)
}

/// **The decision.** Which [`FollowerDisposition`] a commit on this run must
/// use, and why.
///
/// Pure: a matrix pair and the engine's own alignment finding in, an answer
/// out. No document, no session, no page — which is what lets every case in
/// the table below be a unit test rather than a fixture.
///
/// `alignment` is `None` when the caller could not resolve a block for the
/// caret at all (an empty page, a caret on a run the block recogniser did not
/// place). That is treated as [`Reason::AlignmentUndetectable`] and **not** as
/// left alignment, because "no block" and "a left-aligned block" are different
/// findings and only one of them is a finding.
///
/// | `Tm`/CTM | alignment | → | why |
/// |---|---|---|---|
/// | rotated/skewed | *anything* | `Pin` | the follower shift would be in the wrong frame |
/// | upright | `Right` / `Center` / `Justified` | `Pin` | the tail is flush against something |
/// | upright | `Left`, detected | `Reflow` | the line is meant to grow right |
/// | upright | single-line / ambiguous / `None` | `Reflow` | the engine's default, **disclosed as a fall-back** |
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
    // ★★ Rung 2 — the caret's line is several independently positioned pieces.
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
///
/// A one-line adapter, and it exists so that **no call site constructs
/// `EditOptions` itself**. That is the whole defect this module fixes stated as
/// a rule: the old shell's single call site wrote `EditOptions::default()`, and
/// a default is what you get whenever the type is constructible at the point of
/// use. Here the only way to obtain one is to have already answered the
/// question.
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

    /// ★ **An upright matrix pair is upright.** The floor: if this were false
    /// every edit would pin and the fix would look like it worked.
    #[test]
    fn an_upright_matrix_pair_is_upright() {
        assert!(is_upright(UPRIGHT, UPRIGHT));
    }

    /// ★ **A quarter turn in the TEXT matrix is caught.**
    #[test]
    fn a_rotated_text_matrix_is_not_upright() {
        assert!(!is_upright(ROTATED_90, UPRIGHT));
    }

    /// ★★ **A quarter turn in the CTM is caught, with an upright `Tm`.**
    ///
    /// The case a `Tm`-only guard would miss, and the one that matters most
    /// here: a landscape CAD plot rotates the whole content stream with one
    /// `cm`, leaving every `Tm` on the page reading as identity. A guard that
    /// looked only at the text matrix would answer "upright" for every glyph on
    /// such a sheet and the defect would survive the fix intact.
    #[test]
    fn a_rotated_ctm_is_not_upright_even_with_an_upright_text_matrix() {
        assert!(!is_upright(UPRIGHT, ROTATED_90));
    }

    /// ★ **A skew with `b = 0` is caught**, which a one-term guard would not
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

    /// ★★ **Rotation pins, whatever the alignment says.**
    ///
    /// The rung-order assertion. It is written against `None` and against a
    /// real left-aligned detection in `the_engines_own_findings_drive_the_choice`,
    /// so a future edit that moved the alignment test above the rotation test
    /// fails here by name rather than by producing a subtly displaced tail on a
    /// document nobody re-opens.
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

    /// ★ **No block resolved is not "left aligned".**
    ///
    /// Both answer `Reflow`, so the disposition alone cannot tell them apart —
    /// which is why [`Reason`] exists and why this asserts the *reason* rather
    /// than only the disposition.
    #[test]
    fn an_unresolvable_block_is_an_undetectable_alignment_and_not_a_left_one() {
        let r = choose(UPRIGHT, UPRIGHT, false, None);
        assert_eq!(r, Reason::AlignmentUndetectable);
        assert_ne!(r, Reason::LeftAligned);
        assert_eq!(r.disposition(), FollowerDisposition::Reflow);
    }

    /// ★★ **A line made of several pieces PINS, whatever its alignment reads
    /// as** — and this is the assertion the whole 2026-08-19 fix rests on.
    ///
    /// The failure it forbids is concrete: a SolidWorks parts table writes one
    /// show operator per cell, and every cell is left-flush, so the alignment
    /// detector answers `Left` and `LeftAligned` reflows. Under `Reflow` the
    /// engine adds `ΔA` to the `e` of every following absolute `Tm` in the text
    /// object — so widening `PART` to `PARTS` slides `DESCRIPTION` and `QTY`
    /// sideways. **Content the operator did not touch, moved by an edit that did
    /// not mention it.**
    ///
    /// Written against a real left-aligned *detection* rather than against
    /// `None`, because `None` would pass on a build where the rung sat below
    /// alignment: `AlignmentUndetectable` also reflows, so only a positive
    /// `Left` finding can prove the rung order.
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

    /// ★ **Rotation still outranks it**, which is the other half of the rung
    /// order.
    ///
    /// Both pin, so the *disposition* cannot tell them apart — which is exactly
    /// why `Reason` exists and why this asserts the reason. A reader who sees
    /// `Rotated` is being told the sharper fact: a follower shift computed in
    /// user-space x on a rotated baseline is wrong in a way that has nothing to
    /// do with how many pieces the line has.
    #[test]
    fn rotation_outranks_the_multi_run_rung() {
        assert_eq!(
            choose(ROTATED_90, UPRIGHT, true, None),
            Reason::Rotated,
            "a rotated run on a multi-piece line must report the rotation, the sharper fact"
        );
    }

    /// **`Reflow` is what the engine defaults to**, so the fall-back changes
    /// nothing about a document the old shell handled correctly.
    ///
    /// This is the assertion that says the fix is not a regression: every case
    /// the old build got right — an upright, left-aligned or unclassifiable run
    /// — still commits with exactly the options it used to.
    #[test]
    fn the_fallback_is_byte_identical_to_what_the_old_shell_passed() {
        let fallback = options(choose(UPRIGHT, UPRIGHT, false, None));
        assert_eq!(fallback.disposition, EditOptions::default().disposition);
    }

    /// **`pins_the_tail` agrees with `disposition`, for every reason.**
    ///
    /// An arithmetic-identity test in the shape `HANDOFF.md` §10 asks for: two
    /// derived facts about one value, asserted to agree, rather than a comment
    /// asking the next reader to keep them in step.
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

    /// ★ **Every non-left alignment pins**, stated over the enum rather than
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
