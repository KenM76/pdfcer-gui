//! # `canvas::measure::pick` — the measure tools' pick state machines
//!
//! **Salvaged** from the old shell's `measure_tool.rs`
//! (`D:\Dev\pdfce\crates\pdfce-gui\src\measure_tool.rs`, Pass 12.M2b). That
//! file was 2,044 lines and breaks this project's R2 limit whole, so it was
//! split along **its own section banners** rather than at an invented seam:
//! the pick machines land here, the scale-entry and dimension-group model in
//! [`super::scale`], and the tool-entry container in [`super::state`]. The
//! reasoning below is the original's, carried across intact, because the doc
//! comments are the part of the old GUI worth the most.
//!
//! The **pure, GUI-free authoring-state logic** the three Pass 12.M2 measure
//! tools drive on the canvas (`docs/ui_specs/pass-12.M2-dimension-tools.md`,
//! decision 011 §2.3/§2.4). Pass 12.M2 shipped the dimensioning *engine* +
//! `pdfcer` (`c7c1744`); the old GUI shipped the "Measure ▾" menu, the three
//! `CanvasTool` variants, the 12.M1 snap-indicator primitives, and a status
//! overlay — but **not** the click-to-author interaction. This module is that
//! missing slice's testable heart: the pick state machines and the circular
//! fit-set, all expressed over `pdfcer-core` types (never egui), so every
//! transition is unit-tested here without a live frame — the same discipline
//! that keeps [`crate::canvas`]/[`crate::viewer`] headlessly testable while
//! `main.rs` stays a thin, compile-and-launch-only shell.
//!
//! ## What this module owns vs. what the shipped engine owns (REUSE, never reimplement)
//!
//! This module contains **zero** dimension geometry, Taubin math, scale
//! arithmetic, or storage. Every load-bearing computation is a call into the
//! already-shipped `pdfcer-core::dimension` / `pdfcer-core::vector`:
//!
//! - [`constrained_second_point`] / [`measured_length`] (12.M1) — the H/V/
//!   aligned projection and the measured page-space length.
//! - [`fit_circle_taubin`] (12.M2) — the best-fit circle over a sample set.
//! - [`author_from_two_lines`] (`Pass 68.0`) — the entire reading of a picked
//!   PAIR of lines: parallel-vs-angled, which of the four angles, whether the
//!   apex is virtual, whether the pair is collinear and must be refused.
//!   (Listed here rather than in the old header's four bullets only because
//!   `Pass 68.0` post-dates them; the rule it obeys is the same one.)
//! - [`DimensionKind`] (12.M2) — the immutable geometry the GUI hands to
//!   `EditSession::add_dimension`, **byte-for-byte the same value the CLI's
//!   `dimension-add` builds** (`pdfcer` stores `Linear { a: *a, b: *b,
//!   constraint }` from its two raw `--points`, and `Circular { fit,
//!   show_diameter }` from `fit_circle_taubin(&pts)` — so this module stores
//!   the **raw** snapped picks, NOT the constrained projection, matching the
//!   CLI exactly; the constrained segment is a *display-only* preview,
//!   ui-spec §2.5). The equivalence tests [`tests::gui_linear_kind_equals_cli_
//!   linear_kind`] / [`tests::gui_circular_kind_equals_cli_circular_kind`] pin
//!   this: identical `DimensionKind` ⇒ identical `add_dimension` call ⇒
//!   identical additive `/Line`+`/Measure`+`/PieceInfo`+`/OCG` bytes (rule:
//!   same engine path).
//!
//! The scale half of that list — [`pdfcer_core::dimension::preview_group_scale`]
//! and [`pdfcer_core::dimension::parse_length`] — is stated in full on
//! [`super::scale`], which is where the only callers of either now live.
//!
//! ## The three tools' state
//!
//! Split across this file and its two siblings, but it is one model and reads
//! as one:
//!
//! - [`LinearPick`] — the A→B two-click state machine (ui-spec §2.1),
//!   shared verbatim by [`super::scale::ScalePick`]'s reference line (§4.1).
//! - [`CircularPick`] — the tool's OWN object pick-set (ui-spec §3.1, NOT
//!   `canvas_selection`), live-refit on every toggle (§3.2), with the
//!   display-only radius/diameter toggle (§3.4).
//! - [`LinearPickMode`] + [`TwoLinePick`] — which geometry the linear tool's
//!   clicks target (`Pass 68.0`): two snapped POINTS, or two picked LINES that
//!   the engine reads into whichever ce dimension the geometry calls for.
//! - [`super::scale::ScalePick`] + [`super::scale::ScaleEntryFields`] — draw a
//!   reference line, then the two co-equal scale-entry paths (real-length
//!   recommended, ratio) that back-calc through `preview_group_scale` (§4).
//! - [`super::state::MeasureState`] — the container built on tool entry that
//!   holds all of the above plus the shared snap controls.
//!
//! Everything is `pdfcer-gui`-internal; `cargo tree -p pdfcer-core` is
//! unaffected (this module is not in core), and it adds no dependency.
//!
//! ## Adaptations made on the way across
//!
//! Recorded rather than silently applied, because the next reader will
//! otherwise wonder whether the original said something different:
//!
//! 1. **`CanvasTool::MeasureLinear` is prose, not a doc link.** This shell's
//!    [`crate::canvas::CanvasTool`] has `Select`, `Hand` and `Markup`; the
//!    three measure variants land with the canvas hosting. Every place the old
//!    file linked to that variant now names it in backticks, so the link
//!    cannot resolve to the wrong thing while it does not exist.
//! 2. **`GestureInterrupt::Discard` is prose too.** The old shell had a
//!    `crate::canvas::GestureInterrupt` enum; `grep GestureInterrupt` over this
//!    crate returns zero hits. The *concept* — a mid-gesture state that is safe
//!    to throw away — is what those doc comments are about, and it survives.
//! 3. **Nothing computational changed.** No arithmetic, no transition, no
//!    engine call was touched. The only edits are module paths and the two
//!    substitutions above.

use pdfcer_core::dimension::{
    DimensionKind, TwoLineAuthoring, TwoLinePlacement, TwoLineRefusal, author_from_two_lines,
};
use pdfcer_core::vector::linepick::{ParallelPolicy, PickedLine};
use pdfcer_core::vector::{AxisConstraint, Point, constrained_second_point, measured_length};

/// ★ The circular pick set lives in [`super::circpick`] as of 2026-09-03 (R2),
/// and is re-exported here.
///
/// Not a compatibility shim to be deleted: `pick` is the module every measure
/// tool's pick type is reached through, and a reader looking for "the circular
/// tool's pick" should find it named here whichever file it is written in. The
/// alternative — twenty call sites naming a second module — would make the
/// split a rename rather than a seam.
pub use super::circpick::{CircPoint, CircularPick, PickOrigin};

// ---------------------------------------------------------------------------
// Linear pick — the A→B two-click state machine (ui-spec §2.1/§2.5)
// ---------------------------------------------------------------------------

/// The linear-dimension two-pick state machine (ui-spec §2.1): click A →
/// live preview to a constrained second point → click B commits.
///
/// [`Self::first`] being `None` means "awaiting point A"; `Some(a)` means "A
/// is set, the next commit is point B." [`Self::commit_point`] on the second
/// pick returns the authored [`DimensionKind::Linear`] and resets, so the
/// tool is immediately ready for the next dimension (the operator draws a
/// run of dimensions without re-entering the tool).
///
/// **Raw-second-point storage (byte-equivalence, module docs):** the authored
/// `b` is the RAW snapped second pick, exactly as the CLI stores it; the
/// constraint is recorded alongside so `measured_length`/`author_dimension`
/// apply the H/V projection at value/appearance time. The on-canvas preview
/// segment ([`Self::preview_segment`]) is the *constrained* line — display
/// only, so "what you see is what's measured" (ui-spec §2.5) without diverging
/// the stored geometry from the CLI's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearPick {
    /// Point A (page space) once picked; `None` while awaiting the first pick.
    pub first: Option<Point>,
    /// Point B, once picked — the tool is then in its PLACING state, waiting
    /// for the third click that decides where the dimension is drawn (Pass
    /// 27.1).
    ///
    /// # Why a third click
    ///
    /// The operator asked for SolidWorks behaviour, and SolidWorks dimensions
    /// in three: what, to what, and where. The third is not ceremony — it is
    /// the only chance to say how far off the drawing the dimension sits, and
    /// without it every dimension lands on top of the geometry it measures and
    /// has to be dragged off afterwards. pdfcer committed on the second click
    /// with a zero standoff, which is exactly that.
    pub second: Option<Point>,
    /// Whether this pick needs the third, PLACING click.
    ///
    /// True for a real ce dimension, which has to be told where to sit. False
    /// for [`super::scale::ScalePick`]'s reference line, which is a measurement
    /// aid that is never drawn as a dimension — asking the operator where to
    /// place something that is about to disappear would be ceremony with no
    /// meaning. One flag rather than two pick types, because every other
    /// transition is identical and duplicating them is how they drift (R92).
    pub place_after: bool,
    /// The H/V/aligned constraint the property-bar segmented control sets
    /// (ui-spec §2.5). Applied to the SECOND point's projection + measured
    /// length; the stored `b` remains the raw pick (module docs).
    pub constraint: AxisConstraint,
}

impl Default for LinearPick {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearPick {
    /// A fresh pick, awaiting point A, free (aligned) constraint.
    #[must_use]
    pub fn new() -> Self {
        Self {
            first: None,
            second: None,
            place_after: true,
            constraint: AxisConstraint::Aligned,
        }
    }

    /// Register a committed (snapped) pick point `p` (page space). If A was
    /// not yet set, this sets A and returns `None` (awaiting B). If A was set,
    /// this authors a [`DimensionKind::Linear`] with the RAW `b = p` and the
    /// current constraint (module docs), resets to awaiting-A, and returns it.
    pub fn commit_point(&mut self, p: Point) -> Option<DimensionKind> {
        match (self.first, self.second) {
            (None, _) => {
                self.first = Some(p);
                None
            }
            // Second pick: what is being measured is now known, but not where
            // the dimension goes. Enter the placing state rather than commit —
            // unless this pick does not need placing (a scale reference line),
            // in which case the second click still commits, exactly as before.
            (Some(a), None) => {
                if self.place_after {
                    self.second = Some(p);
                    None
                } else {
                    self.first = None;
                    Some(DimensionKind::Linear {
                        a,
                        b: p,
                        constraint: self.constraint,
                        offset: 0.0,
                        text_along: 0.0,
                    })
                }
            }
            // Third pick: where. The pointer resolves into the dimension's own
            // frame — perpendicular is the standoff, parallel is where the
            // number sits along the line — the two components of the placement
            // point SolidWorks' own API takes.
            (Some(a), Some(b)) => {
                let kind = self.placing_kind(a, b, p);
                self.first = None;
                self.second = None;
                Some(kind)
            }
        }
    }

    /// The dimension a placing click at `p` would author. Shared by
    /// [`Self::commit_point`] and [`Self::placing_preview`] so what the
    /// operator SEES while placing is definitionally what commits (R85).
    fn placing_kind(&self, a: Point, b: Point, p: Point) -> DimensionKind {
        let probe = DimensionKind::Linear {
            a,
            b,
            constraint: self.constraint,
            offset: 0.0,
            text_along: 0.0,
        };
        let (offset, text_along) = probe.placement_from_point(p).unwrap_or((0.0, 0.0));
        DimensionKind::Linear {
            a,
            b,
            constraint: self.constraint,
            offset,
            text_along,
        }
    }

    /// While placing, the dimension exactly as it would commit if the operator
    /// clicked at `p` right now — for the live preview.
    ///
    /// `None` unless both points are picked.
    #[must_use]
    pub fn placing_preview(&self, p: Point) -> Option<DimensionKind> {
        let (a, b) = (self.first?, self.second?);
        Some(self.placing_kind(a, b, p))
    }

    /// Whether the tool is waiting for the placing click.
    #[must_use]
    pub fn is_placing(&self) -> bool {
        self.first.is_some() && self.second.is_some()
    }

    /// A pick for a **reference line** rather than a ce dimension: two clicks,
    /// no placing step. Used by [`super::scale::ScalePick`].
    #[must_use]
    pub fn reference_line() -> Self {
        Self {
            place_after: false,
            ..Self::new()
        }
    }

    /// Discard the in-progress first pick (Escape stage 1 / Reject, ui-spec
    /// §1.3): stay in the tool, forget point A.
    pub fn clear(&mut self) {
        self.first = None;
        self.second = None;
    }

    /// Whether a first point is placed (the tool is mid-gesture — a discardable
    /// gesture, which the old shell modelled as
    /// `crate::canvas::GestureInterrupt::Discard`; this shell has no such enum
    /// yet, module docs §"Adaptations").
    #[must_use]
    pub fn in_progress(&self) -> bool {
        self.first.is_some()
    }

    /// The CONSTRAINED display segment `(a, projected_b)` for the live preview
    /// line (ui-spec §2.5), given the current raw pointer `raw`, or `None`
    /// while awaiting A. Display only — the authored `b` is the raw pick.
    #[must_use]
    pub fn preview_segment(&self, raw: Point) -> Option<(Point, Point)> {
        self.first
            .map(|a| (a, constrained_second_point(a, raw, self.constraint)))
    }

    /// The measured page-space length from A to the raw pointer under the
    /// current constraint (ui-spec §2.6 live readout), or `None` while
    /// awaiting A. `measured_length` uses the RAW second point (projecting
    /// first gives the identical result — `snap.rs` module docs).
    #[must_use]
    pub fn measured(&self, raw: Point) -> Option<f64> {
        self.first.map(|a| measured_length(a, raw, self.constraint))
    }
}

// ---------------------------------------------------------------------------
// Two-line pick — select two lines, pdfcer reads what they mean (`Pass 68.0`)
// ---------------------------------------------------------------------------

/// Which geometry the linear measure tool (`CanvasTool::MeasureLinear`)'s next
/// pick targets (ui-spec `pass-68.0` §1/§2.2).
///
/// A real change in what a click MEANS: [`Self::Points`] resolves any snap
/// candidate anywhere on the page, while [`Self::TwoLines`] calls
/// [`pdfcer_core::vector::linepick::pick_line_in_page`] and requires landing on
/// straight, already-drawn geometry — refusing curves and misses rather than
/// inventing a point.
///
/// # Why this is a mode and not a fourth tool
///
/// Because the operator declares it explicitly, in a control that is visible
/// the whole time the tool is armed. That is the test pass-46 §1.2 already
/// applied to `MarkupKind`'s ten kinds: a click's meaning may vary by mode, so
/// long as it never turns on state the operator cannot see. The full argument,
/// including why the `AddText`-vs-sub-mode precedent does NOT apply, is on
/// `CanvasTool::MeasureLinear`.
///
/// Switching mode discards any in-progress pick first — free, because nothing
/// has committed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinearPickMode {
    /// The original: two snapped POINT picks author the measurement.
    #[default]
    Points,
    /// Two LINE picks; pdfcer reads what they mean and authors accordingly.
    TwoLines,
}

/// The two-line ce-dimension pick: click one line, click another, and pdfcer
/// authors the ce dimension the geometry calls for — a LINEAR one between two
/// parallel lines, an ANGULAR one between two that meet.
///
/// The operator's request, verbatim (2026-08-12): *"dimensioning tool should
/// allow the selection of two lines. if those lines are parallel it makes a
/// linear dimension between them like SolidWorks would, if they are at an
/// angle it makes an angle dimension."*
///
/// # ★ Nothing here classifies anything
///
/// This struct holds two picks and a checkbox. Every geometric question — are
/// these parallel, which of the four angles did the operator mean, is the apex
/// virtual, is the pair collinear — is answered by
/// [`pdfcer_core::dimension::author_from_two_lines`], which is the same
/// function `pdfcer`'s `dimension-add --kind two-lines` calls.
///
/// That is deliberate and load-bearing. A second classifier living at the
/// canvas is exactly how the two shells acquire the disagreement
/// `Settings::parallel_epsilon_degrees` was introduced to prevent: the setting
/// centralises the *threshold*, and duplicating the code that consumes it
/// would reintroduce the divergence one level above the value that stops it.
/// The GUI owes a gesture and a disclosure surface, not a second reading of
/// the geometry.
///
/// # ★ Why this is NOT stored in `MeasureState::pending`
///
/// `pending` looks like the obvious home — it is already documented as "the
/// linear tool's completed-but-not-yet-authored dimension". It is the wrong
/// home, and the reason is a shipped piece of machinery that would break
/// silently. `PdfcerApp::committable_gesture` reads:
///
/// ```text
/// let measure = doc.active_tool() == Some(CanvasTool::MeasureLinear)
///     && doc.measure.as_ref().is_some_and(|s| s.pending.is_some());
/// ```
///
/// That is decision 031's commit-on-interrupt path: a completed *two-point*
/// pick is safe to auto-commit when something else interrupts the gesture,
/// because nothing about it is inferred — it is exactly what the operator
/// clicked. The same function deliberately EXCLUDES the circular tool, whose
/// best fit is inferred.
///
/// A two-line verdict is inferred in precisely the circular sense:
/// parallel-vs-angled, which of four angles, whether the apex is virtual. Put
/// it in `pending` and that `is_some()` check — which cannot tell an ordinary
/// pick from an inference, having only ever asked whether the field was
/// populated — would quietly make it interrupt-committable, reopening for this
/// gesture exactly the hazard decision 031 closed. A sibling field keeps the
/// existing, already-tested rule correct without teaching it a new distinction.
///
/// # ★ Why the verdict is DERIVED on every read instead of cached
///
/// The ui-spec proposed a `verdict` field recomputed "whenever `second` or
/// `force_parallel` changes". This implementation deliberately has no such
/// field, and the reason is not a preference — it is that the proposed write
/// list was already incomplete by one when it was written. **The epsilon can
/// change too**: `Settings::parallel_epsilon_degrees` has a slider in the
/// settings panel, and moving it re-reads the same two lines into a different
/// answer (pinned by
/// [`tests::changing_the_epsilon_setting_re_reads_the_same_pair`]). A cache
/// listing two of its three producers is the failure mode already recorded in
/// `D:\dev\rag\egui\a_derived_value_with_one_producer_cannot_drift_a_cached_copy_with_n_producers_will.md`
/// — found in this very codebase on `recovery_note`, and fixed in `149fd03` by
/// deleting the cache rather than adding the missing reset site.
///
/// The recomputation is a few dozen floating-point operations on two stored
/// segments, so the cache buys nothing and costs a synchronisation obligation.
/// [`Self::authoring`] re-derives on every call: there is nothing to keep in
/// sync, and the verdict the operator reads is definitionally the one that
/// will commit (R85).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TwoLinePick {
    /// The first picked line; `None` while awaiting it.
    pub first: Option<PickedLine>,
    /// The second picked line; `None` until the pair is complete.
    pub second: Option<PickedLine>,
    /// The operator's **"treat these two lines as parallel"** override.
    ///
    /// Requested directly (2026-08-12): *"When making or editing a dimension
    /// of this type, there should be a checkbox option to treat the two lines
    /// as parallel."* It exists because the automatic reading is a GUESS and a
    /// global threshold cannot be right for every pair in a drawing — two
    /// nominally-parallel edges can arrive 0.8° apart from an exporter's
    /// rounding, and the operator, looking at the part, knows which.
    ///
    /// Ticking it never fakes the measurement: the true angle survives in
    /// [`pdfcer_core::dimension::TwoLineAuthoring::measured_angle_degrees`] so
    /// the disclosure can state what was overridden.
    ///
    /// **Survives [`Self::clear`]**, like `snap_master` and the active group.
    /// The first instinct is the opposite — it is an assertion about two
    /// specific lines, so carrying it to the next pair looks like applying a
    /// claim the operator never made about them. What settles it is the
    /// failure the override was invented to avoid, stated in `linepick.rs`'s
    /// own docs: without it, the only remedy would be *"to change a global
    /// setting to author one dimension and change it back, which is how a
    /// setting becomes a thing people fight."* Resetting per pair recreates
    /// exactly that friction at smaller scale, for the operator dimensioning a
    /// whole drawing out of a sloppy exporter. It is safe to persist because
    /// the verdict — including the word "forced" and the true angle — is on
    /// screen before any Accept.
    pub force_parallel: bool,
}

impl TwoLinePick {
    /// A fresh pick, awaiting the first line, with the override off.
    ///
    /// Off is the honest default: the override is the operator asserting
    /// something about the drawing that pdfcer's own reading disagrees with, so
    /// it cannot be on until they say so.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Offer a picked line to the gesture. Returns `true` if it was taken.
    ///
    /// The three cases, and why each is what it is:
    ///
    /// - **Awaiting line A** — take it.
    /// - **Awaiting line B** — take it; the pair is now complete and the
    ///   verdict is on screen for review.
    /// - **Pair complete** — depends on the verdict, which is why this needs
    ///   `epsilon_degrees`:
    ///   - a valid verdict ⇒ **ignored**, matching `pending`'s own documented
    ///     "further picks are ignored, the operator is reviewing" rule. An
    ///     inference under review must not be replaced by a stray click.
    ///   - a REFUSED pair (collinear, or a degenerate line) ⇒ the new line
    ///     **replaces line B**. That is the natural "try a different second
    ///     line" recovery, and it needs no special case beyond reassigning
    ///     `second`. Line A is kept, so recovering costs one click rather
    ///     than two.
    ///
    /// Returning `false` for the ignored case lets the caller leave the
    /// disclosure untouched rather than re-announcing an unchanged verdict.
    pub fn offer_line(&mut self, line: PickedLine, epsilon_degrees: f64) -> bool {
        match (self.first, self.second) {
            (None, _) => {
                self.first = Some(line);
                true
            }
            (Some(_), None) => {
                self.second = Some(line);
                true
            }
            (Some(_), Some(_)) => {
                // Only a refused pair yields to a new pick.
                let refused = matches!(
                    self.authoring(epsilon_degrees, TwoLinePlacement::default()),
                    Some(Err(_))
                );
                if refused {
                    self.second = Some(line);
                }
                refused
            }
        }
    }

    /// What the current pair authors, re-derived from scratch.
    ///
    /// `None` while the pair is incomplete. `Some(Err(..))` when the pair
    /// cannot yield a ce dimension — collinear, or a zero-length line — which
    /// the caller is expected to disclose by name rather than swallow.
    ///
    /// `epsilon_degrees` comes from `Settings::parallel_epsilon_degrees` and is
    /// never a literal at the call site, so this tool and the CLI cannot
    /// disagree about when two lines count as parallel.
    #[must_use]
    pub fn authoring(
        &self,
        epsilon_degrees: f64,
        placement: TwoLinePlacement,
    ) -> Option<Result<TwoLineAuthoring, TwoLineRefusal>> {
        let (a, b) = (self.first?, self.second?);
        let mut policy = ParallelPolicy::from_setting(epsilon_degrees);
        if self.force_parallel {
            policy = policy.forcing_parallel();
        }
        Some(author_from_two_lines(&a, &b, policy, placement))
    }

    /// Discard the in-progress pair (Escape stage 1 / Reject), staying in the
    /// tool.
    ///
    /// [`Self::force_parallel`] deliberately SURVIVES, mirroring how
    /// `snap_master` and the active group survive
    /// [`super::state::MeasureState::clear_gesture`] — see that field's own
    /// docs for why.
    pub fn clear(&mut self) {
        self.first = None;
        self.second = None;
    }

    /// Whether any part of a pair is picked — a discardable gesture.
    #[must_use]
    pub fn in_progress(&self) -> bool {
        self.first.is_some() || self.second.is_some()
    }

    /// Whether both lines are picked and a verdict is on screen for review.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.first.is_some() && self.second.is_some()
    }
}

/// How many chords approximate a previewed dimension arc.
///
/// Twenty-four over the full turn is smooth at any zoom pdfcer offers, and an
/// angular ce dimension's wedge is a fraction of that — so the drawn arc is
/// visually smooth while staying a handful of segments.
const ARC_PREVIEW_STEPS: usize = 24;

/// The page-space line segments that draw a ce dimension's live preview.
///
/// # Why this is a function and not two painter loops
///
/// Two places preview a ce dimension before it is committed: the two-line
/// authoring gesture (previewing what Accept would author) and the placement
/// drag (previewing where a release would put it). They must draw the SAME
/// shape for the same kind, or the operator sees one thing while authoring and
/// a different thing while adjusting it.
///
/// Keeping it here rather than in `main.rs` also makes it testable — the arc
/// decomposition in particular has a wrap-around case (a wedge crossing ±π)
/// that is easy to get wrong and invisible until an operator picks the two
/// arms that trigger it.
///
/// Returns page-space pairs; the caller supplies the projection to screen.
///
/// # ★ The circular arm used to return nothing, and why it no longer does
///
/// This paragraph replaced *"empty for `DimensionKind::Circular`, which the
/// ce-dimension preview does not draw (the circular tool outlines its source
/// objects instead)"*, which was true and insufficient at the same time. The
/// outlines say **which objects are in the fit**; they cannot say **what circle
/// those objects imply**, and the circle is the entire output of the tool. An
/// operator looking at three outlined arcs has no way to tell a fit that lands
/// on their hole from one that has caught a leader line and bulged — the
/// residual is a number nobody sees until the dimension is on the page.
///
/// So the circle is drawn, and it is drawn **from here** rather than from a
/// loop in the canvas hosting, for this function's own stated reason one
/// paragraph up: the fit is an *inference*, and rule 4's pre-commit affordance
/// only means anything if what is previewed is derived from what will be
/// committed. `canvas::measure::preview` hands this the identical
/// `DimensionKind` that `circular::commit` raises on the action — so the circle
/// on screen and the circle in the file are one derivation, not two that agree.
#[must_use]
pub fn dimension_preview_segments(kind: &DimensionKind) -> Vec<(Point, Point)> {
    // ★★ The PERIMETER arm, 2026-08-20, and it is the shortest in this
    // function for a reason worth stating: **a perimeter's notation is its own
    // shape.**
    //
    // Every other kind here draws something that is not the geometry — a
    // dimension line standing off the drawing with witness lines reaching back
    // to it, an arc across a wedge, a fitted circle nobody drew. A perimeter's
    // drawn geometry IS the measured geometry, so there is nothing to derive:
    // the segments are the picks, plus the closing one when the operator said
    // closed.
    //
    // The engine draws it the same way and says why: no terminators (two arrows
    // would assert "the distance BETWEEN these points" when the number is the
    // distance ALONG a path, and a closed shape has no ends to put them on), no
    // witness lines, no ANSI break for the label. Each absence is a decision,
    // and this preview matching them is what makes it a preview rather than an
    // illustration.
    if let Some((points, closed)) = kind.polyline() {
        let mut segments: Vec<(Point, Point)> = points.windows(2).map(|w| (w[0], w[1])).collect();
        // ★ The closing segment is supplied HERE, and that is the hazard the
        // spec corpus names by name: a `/Polygon`'s `/Vertices` does not repeat
        // the first point, so a routine that forgets to close it draws — and
        // measures — a shape one segment short of what the operator picked.
        if closed
            && points.len() > 2
            && let (Some(first), Some(last)) = (points.first(), points.last())
        {
            segments.push((*last, *first));
        }
        return segments;
    }
    match *kind {
        DimensionKind::Linear { .. } => kind
            .linear_geometry()
            .map(|(dim_a, dim_b, ext_a, ext_b)| {
                vec![(dim_a, dim_b), (ext_a, dim_a), (ext_b, dim_b)]
            })
            .unwrap_or_default(),
        // Handled above, before the match, because it is the one kind whose
        // preview is its own geometry rather than a derivation from it.
        DimensionKind::Perimeter { .. } => Vec::new(),
        DimensionKind::Angular {
            apex,
            dir_a,
            dir_b,
            radius,
            ..
        } => {
            let start = dir_a.y.atan2(dir_a.x);
            // Take the SHORT way round between the two arms. Without this fold
            // a wedge whose arms straddle the ±π discontinuity would sweep the
            // long way and draw a reflex arc — the correct angle, illustrated
            // by the wrong picture.
            let mut sweep = dir_b.y.atan2(dir_b.x) - start;
            while sweep > std::f64::consts::PI {
                sweep -= std::f64::consts::TAU;
            }
            while sweep < -std::f64::consts::PI {
                sweep += std::f64::consts::TAU;
            }
            let at = |ang: f64| {
                Point::new(
                    radius.mul_add(ang.cos(), apex.x),
                    radius.mul_add(ang.sin(), apex.y),
                )
            };
            let mut out = Vec::with_capacity(ARC_PREVIEW_STEPS + 2);
            for step in 0..ARC_PREVIEW_STEPS {
                #[allow(clippy::cast_precision_loss)]
                let (t0, t1) = (
                    step as f64 / ARC_PREVIEW_STEPS as f64,
                    (step + 1) as f64 / ARC_PREVIEW_STEPS as f64,
                );
                out.push((at(sweep.mul_add(t0, start)), at(sweep.mul_add(t1, start))));
            }
            // A leg from the apex out along each arm. These matter most when
            // the apex is VIRTUAL: they are what shows the operator where
            // pdfcer decided the two lines would meet.
            out.push((apex, at(start)));
            out.push((apex, at(start + sweep)));
            out
        }
        // The fitted circle, plus the one mark that says which of the two
        // dimensions this is.
        //
        // ★ `show_diameter` is a *display* toggle on one fit (decision 011:
        // `diameter = 2 × radius`, the same stored geometry), so it must not
        // change the circle — only what is drawn across it. A radius draws one
        // spoke from the centre to the rim; a diameter draws the whole chord
        // through the centre. Without that mark the two dimensions preview
        // identically, and an operator toggling between them would see nothing
        // happen and reasonably conclude the toggle was broken.
        //
        // Along +x rather than towards the pointer, deliberately: the committed
        // dimension's leader is the engine's to place, and a preview that
        // pointed somewhere the commit will not would be describing a dimension
        // nobody is about to author.
        DimensionKind::Circular { fit, show_diameter } => {
            let at = |ang: f64| {
                Point::new(
                    fit.radius.mul_add(ang.cos(), fit.center.x),
                    fit.radius.mul_add(ang.sin(), fit.center.y),
                )
            };
            let mut out = Vec::with_capacity(ARC_PREVIEW_STEPS + 1);
            for step in 0..ARC_PREVIEW_STEPS {
                #[allow(clippy::cast_precision_loss)]
                let (t0, t1) = (
                    step as f64 / ARC_PREVIEW_STEPS as f64,
                    (step + 1) as f64 / ARC_PREVIEW_STEPS as f64,
                );
                out.push((
                    at(std::f64::consts::TAU * t0),
                    at(std::f64::consts::TAU * t1),
                ));
            }
            out.push((
                if show_diameter {
                    at(std::f64::consts::PI)
                } else {
                    fit.center
                },
                at(0.0),
            ));
            out
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp
)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    // ---- LinearPick A→B state machine (ui-spec §2.1) --------------------

    /// **Three clicks: what, to what, WHERE** (Pass 27.1).
    ///
    /// This test previously asserted that the SECOND click authored and reset.
    /// It changed because the operator asked for SolidWorks behaviour, and
    /// SolidWorks dimensions in three steps — the third is what says how far
    /// off the drawing the dimension sits. Committing on the second click
    /// meant every ce dimension landed on top of the geometry it measured,
    /// with a zero standoff, and had to be dragged clear afterwards.
    #[test]
    fn linear_pick_needs_a_third_placing_click_then_resets() {
        let mut lp = LinearPick::new();
        assert_eq!(lp.commit_point(p(10.0, 20.0)), None, "first: what");
        assert!(lp.in_progress());
        assert_eq!(lp.commit_point(p(50.0, 20.0)), None, "second: to what");
        assert!(lp.is_placing(), "both points known, awaiting placement");

        // Third: where. 15 above the measured line, and 5 right of its middle.
        let kind = lp.commit_point(p(35.0, 35.0)).unwrap();
        let DimensionKind::Linear {
            a,
            b,
            offset,
            text_along,
            ..
        } = kind
        else {
            panic!("expected a linear dimension")
        };
        assert_eq!((a, b), (p(10.0, 20.0), p(50.0, 20.0)), "the picks are kept");
        assert!(
            (offset - 15.0).abs() < 0.001,
            "the placing click's perpendicular component is the standoff, got {offset}"
        );
        assert!(
            (text_along - 5.0).abs() < 0.001,
            "and its parallel component is where the number sits, got {text_along}"
        );
        assert!(!lp.in_progress(), "the pick resets, ready for the next dim");
    }

    /// What is previewed while placing is what commits (R85).
    #[test]
    fn the_placing_preview_is_exactly_what_the_placing_click_authors() {
        let mut lp = LinearPick::new();
        lp.commit_point(p(10.0, 20.0));
        lp.commit_point(p(50.0, 20.0));
        let previewed = lp.placing_preview(p(35.0, 35.0)).expect("previewing");
        let committed = lp.commit_point(p(35.0, 35.0)).expect("commits");
        assert_eq!(
            previewed, committed,
            "the operator must not be shown one dimension and given another"
        );
    }

    /// A scale reference line still commits on the SECOND click.
    ///
    /// `ScalePick` reuses this state machine for a line that is never drawn as
    /// a dimension, so asking where to place it would be ceremony with no
    /// meaning. The opt-out is what keeps one state machine serving both.
    #[test]
    fn a_reference_line_pick_still_commits_on_the_second_click() {
        let mut lp = LinearPick::reference_line();
        assert_eq!(lp.commit_point(p(10.0, 20.0)), None);
        assert!(
            lp.commit_point(p(50.0, 20.0)).is_some(),
            "a reference line must not wait for a placing click"
        );
        assert!(!lp.in_progress());
    }

    #[test]
    fn linear_pick_stores_the_raw_second_point_even_under_hv_constraint() {
        // The byte-equivalence invariant: the STORED b is the raw pick, NOT
        // the constrained projection — the constraint is recorded alongside so
        // the measured length / appearance apply it (matching the CLI, module
        // docs). Only the PREVIEW segment is constrained.
        let mut lp = LinearPick::new();
        lp.constraint = AxisConstraint::Horizontal;
        lp.commit_point(p(10.0, 20.0));
        lp.commit_point(p(50.0, 80.0));
        // Placed on the measured line itself, so the placement is neutral and
        // this test stays about the STORED points.
        let kind = lp.commit_point(p(30.0, 20.0)).unwrap();
        let DimensionKind::Linear {
            a, b, constraint, ..
        } = kind
        else {
            panic!("expected a linear dimension")
        };
        assert_eq!(a, p(10.0, 20.0));
        assert_eq!(
            b,
            p(50.0, 80.0),
            "the stored b is the RAW pick, not (50,20)"
        );
        assert_eq!(constraint, AxisConstraint::Horizontal);
        // The measured value still honours the constraint (|Δx| = 40).
        assert_eq!(kind.measured_points(), 40.0);
    }

    #[test]
    fn linear_preview_segment_is_the_constrained_line_display_only() {
        let mut lp = LinearPick::new();
        lp.constraint = AxisConstraint::Horizontal;
        assert_eq!(lp.preview_segment(p(50.0, 80.0)), None); // awaiting A
        lp.commit_point(p(10.0, 20.0));
        // The preview projects the second point onto the page X axis (shares
        // A.y) — what you see is what's measured (ui-spec §2.5).
        assert_eq!(
            lp.preview_segment(p(50.0, 80.0)),
            Some((p(10.0, 20.0), p(50.0, 20.0)))
        );
        assert_eq!(lp.measured(p(50.0, 80.0)), Some(40.0));
    }

    #[test]
    fn linear_clear_discards_the_first_pick() {
        let mut lp = LinearPick::new();
        lp.commit_point(p(1.0, 1.0));
        assert!(lp.in_progress());
        lp.clear();
        assert!(!lp.in_progress());
    }

    /// **The canvas-authored == CLI-authored equivalence check (linear).** The
    /// GUI's `LinearPick` produces the IDENTICAL `DimensionKind` the CLI's
    /// `dimension-add` builds from the same two raw `--points` + constraint
    /// (`pdfcer` `main.rs`: `Linear { a: *a, b: *b, constraint }`). Identical
    /// kind ⇒ identical `EditSession::add_dimension` call ⇒ byte-identical
    /// additive output (same engine path — the acceptance gate).
    #[test]
    fn gui_linear_kind_equals_cli_linear_kind() {
        let a = p(72.0, 144.0);
        let b = p(216.0, 144.0);
        let constraint = AxisConstraint::Horizontal;

        // GUI path: two snapped picks, then the Pass 27.1 placing click.
        // Placed at the midpoint of the measured line, which is the NEUTRAL
        // placement — zero standoff, centred text — so this test still compares
        // the two paths' DEFAULTS rather than accidentally comparing a placed
        // dimension against an unplaced one.
        let mut lp = LinearPick::new();
        lp.constraint = constraint;
        lp.commit_point(a);
        lp.commit_point(b);
        let gui_kind = lp.commit_point(p(144.0, 144.0)).unwrap();

        // CLI path: the exact construction from pdfcer/src/main.rs with its
        // default --offset/--text-along.
        let cli_kind = DimensionKind::Linear {
            a,
            b,
            constraint,
            offset: 0.0,
            text_along: 0.0,
        };

        assert_eq!(gui_kind, cli_kind);
    }

    // ---- Two-line pick (`Pass 68.0`) ------------------------------------

    /// A picked line with its pick point at the midpoint.
    fn picked(sx: f64, sy: f64, ex: f64, ey: f64) -> PickedLine {
        PickedLine {
            target: pdfcer_core::vector::hit::HitTarget::Object(0),
            subpath: 0,
            segment: 0,
            start: p(sx, sy),
            end: p(ex, ey),
            pick: p(f64::midpoint(sx, ex), f64::midpoint(sy, ey)),
        }
    }

    const EPS: f64 = 0.5;

    #[test]
    fn two_line_pick_needs_both_lines_before_it_authors_anything() {
        let mut pick = TwoLinePick::new();
        assert!(!pick.in_progress());
        assert!(
            pick.authoring(EPS, TwoLinePlacement::default()).is_none(),
            "an empty pick authors nothing"
        );

        assert!(pick.offer_line(picked(100.0, 100.0, 300.0, 100.0), EPS));
        assert!(pick.in_progress() && !pick.is_complete());
        assert!(
            pick.authoring(EPS, TwoLinePlacement::default()).is_none(),
            "one line is not a pair"
        );

        assert!(pick.offer_line(picked(100.0, 140.0, 300.0, 140.0), EPS));
        assert!(pick.is_complete());
        assert!(pick.authoring(EPS, TwoLinePlacement::default()).is_some());
    }

    /// Two parallel edges author a LINEAR ce dimension of the gap.
    #[test]
    fn two_parallel_picks_author_a_linear_ce_dimension() {
        let mut pick = TwoLinePick::new();
        pick.offer_line(picked(100.0, 100.0, 300.0, 100.0), EPS);
        pick.offer_line(picked(100.0, 140.0, 300.0, 140.0), EPS);

        let authored = pick
            .authoring(EPS, TwoLinePlacement::default())
            .expect("complete")
            .expect("dimensionable");
        assert!(authored.is_linear());
        assert!(matches!(authored.kind, DimensionKind::Linear { .. }));
    }

    /// Two edges that meet author an ANGULAR ce dimension.
    #[test]
    fn two_angled_picks_author_an_angular_ce_dimension() {
        let mut pick = TwoLinePick::new();
        pick.offer_line(picked(100.0, 100.0, 300.0, 100.0), EPS);
        pick.offer_line(picked(100.0, 100.0, 100.0, 300.0), EPS);

        let authored = pick
            .authoring(EPS, TwoLinePlacement::default())
            .expect("complete")
            .expect("dimensionable");
        assert!(!authored.is_linear());
        assert!(matches!(authored.kind, DimensionKind::Angular { .. }));
    }

    /// A collinear pair surfaces the refusal rather than authoring a
    /// zero-length ce dimension the operator would go hunting for.
    #[test]
    fn a_collinear_pair_surfaces_the_refusal() {
        let mut pick = TwoLinePick::new();
        pick.offer_line(picked(0.0, 0.0, 100.0, 0.0), EPS);
        pick.offer_line(picked(200.0, 0.0, 300.0, 0.0), EPS);

        let result = pick
            .authoring(EPS, TwoLinePlacement::default())
            .expect("the pair is complete");
        assert_eq!(result, Err(TwoLineRefusal::Collinear));
    }

    /// ★ The whole reason the verdict is derived rather than cached: ticking
    /// the override AFTER both picks must change the answer immediately, with
    /// no re-pick and no cache to invalidate.
    #[test]
    fn ticking_the_override_after_both_picks_changes_the_verdict_immediately() {
        let t = 5f64.to_radians().tan();
        let mut pick = TwoLinePick::new();
        pick.offer_line(picked(0.0, 0.0, 100.0, 0.0), EPS);
        pick.offer_line(picked(0.0, 40.0, 100.0, 100.0f64.mul_add(t, 40.0)), EPS);

        let before = pick
            .authoring(EPS, TwoLinePlacement::default())
            .expect("complete")
            .expect("dimensionable");
        assert!(!before.is_linear(), "5 degrees apart reads as angled");

        // The operator ticks the box while looking at that verdict.
        pick.force_parallel = true;

        let after = pick
            .authoring(EPS, TwoLinePlacement::default())
            .expect("complete")
            .expect("dimensionable");
        assert!(after.is_linear(), "the override must take effect at once");
        assert!(after.forced_parallel);
        // ...and the overridden angle is still available to disclose.
        let measured = after
            .measured_angle_degrees
            .expect("the true angle survives the override");
        assert!((measured - 5.0).abs() < 0.01, "got {measured}");
    }

    /// ★ Changing the epsilon SETTING re-reads the same two lines too — the
    /// second state transition a cached verdict would have had to chase.
    #[test]
    fn changing_the_epsilon_setting_re_reads_the_same_pair() {
        let t = 0.2f64.to_radians().tan();
        let mut pick = TwoLinePick::new();
        pick.offer_line(picked(0.0, 0.0, 100.0, 0.0), EPS);
        pick.offer_line(picked(0.0, 20.0, 100.0, 100.0f64.mul_add(t, 20.0)), EPS);

        let loose = pick
            .authoring(0.5, TwoLinePlacement::default())
            .expect("complete")
            .expect("dimensionable");
        assert!(loose.is_linear(), "0.2 degrees is parallel under 0.5");

        let strict = pick
            .authoring(0.1, TwoLinePlacement::default())
            .expect("complete")
            .expect("dimensionable");
        assert!(!strict.is_linear(), "the same pair is angled under 0.1");
    }

    /// ★ A third pick is IGNORED while a valid verdict is under review — an
    /// inference awaiting Accept must not be swapped out by a stray click.
    /// Mirrors `pending`'s own documented "further picks are ignored" rule.
    #[test]
    fn a_third_pick_is_ignored_while_a_valid_verdict_is_under_review() {
        let mut pick = TwoLinePick::new();
        pick.offer_line(picked(0.0, 0.0, 100.0, 0.0), EPS);
        pick.offer_line(picked(0.0, 40.0, 100.0, 40.0), EPS);
        assert!(pick.is_complete());

        assert!(
            !pick.offer_line(picked(0.0, 80.0, 100.0, 80.0), EPS),
            "the pick must be refused while the operator is reviewing"
        );
        assert_eq!(
            pick.second.map(|l| l.start.y),
            Some(40.0),
            "line B must be untouched"
        );
    }

    /// ★ A REFUSED pair does yield: the new line replaces line B, so "try a
    /// different second line" costs one click and keeps line A.
    #[test]
    fn a_new_pick_replaces_line_b_when_the_pair_was_refused() {
        let mut pick = TwoLinePick::new();
        pick.offer_line(picked(0.0, 0.0, 100.0, 0.0), EPS);
        // Collinear with A — refused.
        pick.offer_line(picked(200.0, 0.0, 300.0, 0.0), EPS);
        assert_eq!(
            pick.authoring(EPS, TwoLinePlacement::default()),
            Some(Err(TwoLineRefusal::Collinear))
        );

        assert!(
            pick.offer_line(picked(0.0, 40.0, 100.0, 40.0), EPS),
            "a refused pair must accept a replacement line B"
        );
        assert_eq!(
            pick.first.map(|l| l.start.y),
            Some(0.0),
            "line A is kept through the recovery"
        );
        let authored = pick
            .authoring(EPS, TwoLinePlacement::default())
            .expect("complete")
            .expect("the replacement pair is dimensionable");
        assert!(authored.is_linear());
    }

    /// ★ The override SURVIVES a clear, like `snap_master` and the group.
    ///
    /// The instinct is the opposite — it is an assertion about two specific
    /// lines. What settles it is the friction the override exists to remove:
    /// `linepick.rs` documents that without it the remedy would be changing a
    /// global setting per dimension, *"which is how a setting becomes a thing
    /// people fight"*. Resetting per pair recreates that at smaller scale for
    /// anyone dimensioning a whole drawing out of a sloppy exporter, and it is
    /// safe to persist because the verdict says "forced" before any Accept.
    #[test]
    fn the_override_survives_a_clear_like_the_other_tool_preferences() {
        let mut pick = TwoLinePick::new();
        pick.offer_line(picked(0.0, 0.0, 100.0, 0.0), EPS);
        pick.force_parallel = true;

        pick.clear();
        assert!(!pick.in_progress(), "the picks are discarded");
        assert!(
            pick.force_parallel,
            "the operator's override is a standing preference, not part of the gesture"
        );
    }

    // ---- Shared ce-dimension preview shape (`Pass 68.0`) ----------------

    /// A linear ce dimension previews as its dimension line plus two extension
    /// lines — three segments, the same three the placement drag draws.
    #[test]
    fn a_linear_ce_dimension_previews_as_line_plus_two_extensions() {
        let segs = dimension_preview_segments(&DimensionKind::Linear {
            a: p(0.0, 0.0),
            b: p(100.0, 0.0),
            constraint: AxisConstraint::Horizontal,
            offset: 20.0,
            text_along: 0.0,
        });
        assert_eq!(segs.len(), 3, "dimension line + two extension lines");
    }

    /// An angular ce dimension previews as an arc plus a leg out along each
    /// arm. The legs are what show where a VIRTUAL apex sits.
    #[test]
    fn an_angular_ce_dimension_previews_as_an_arc_plus_two_arm_legs() {
        let apex = p(50.0, 50.0);
        let radius = 30.0;
        let segs = dimension_preview_segments(&DimensionKind::Angular {
            apex,
            dir_a: p(1.0, 0.0),
            dir_b: p(0.0, 1.0),
            radius,
            text_along: 0.0,
        });
        assert_eq!(segs.len(), ARC_PREVIEW_STEPS + 2, "arc chords + two legs");

        // Every arc point sits on the circle of the stated radius about the
        // apex — the check that catches a centre/radius mix-up.
        for (a, _) in segs.iter().take(ARC_PREVIEW_STEPS) {
            let r = (a.x - apex.x).hypot(a.y - apex.y);
            assert!((r - radius).abs() < 1e-9, "off the arc: {a:?} r={r}");
        }
        // The two legs start at the apex.
        for (a, _) in segs.iter().skip(ARC_PREVIEW_STEPS) {
            assert!(
                (a.x - apex.x).abs() < 1e-9 && (a.y - apex.y).abs() < 1e-9,
                "a leg must start at the apex, got {a:?}"
            );
        }
    }

    /// ★ A wedge whose arms straddle the ±π discontinuity takes the SHORT way
    /// round. Without the fold the preview sweeps the long way and draws a
    /// reflex arc — the correct angle illustrated by the wrong picture, which
    /// no unit on the value side would catch.
    #[test]
    fn an_arc_spanning_the_angle_wraparound_takes_the_short_way() {
        const RADIUS: f64 = 10.0;
        let apex = p(0.0, 0.0);
        let at = |deg: f64| p(deg.to_radians().cos(), deg.to_radians().sin());
        let segs = dimension_preview_segments(&DimensionKind::Angular {
            apex,
            // +150° and -150°. The short way between them is 60°, crossing
            // ±180°; the long way is 300°. A naive subtraction gives -300 and
            // draws the reflex arc.
            dir_a: at(150.0),
            dir_b: at(-150.0),
            radius: RADIUS,
            text_along: 0.0,
        });
        // Summing the chord lengths approximates the arc length. 60° of a
        // radius-10 circle is ~10.47; the reflex 300° would be ~52.4.
        let arc_len: f64 = segs
            .iter()
            .take(ARC_PREVIEW_STEPS)
            .map(|(a, b)| (b.x - a.x).hypot(b.y - a.y))
            .sum();
        let short_way = RADIUS * 60f64.to_radians();
        assert!(
            (arc_len - short_way).abs() < 0.05,
            "expected the short way ({short_way:.2}), got {arc_len:.2} \
             (the reflex arc would be {:.2})",
            RADIUS * 300f64.to_radians()
        );
    }

    /// ★ **A circular preview is the fitted circle itself**, and every point
    /// of it lies on that circle.
    ///
    /// This test used to assert the opposite — that the circular arm returned
    /// nothing. It changed when the radius/diameter tool was armed, because the
    /// outlines of the picked objects say which arcs are in the fit and cannot
    /// say what circle they imply, and the circle is the tool's entire output.
    ///
    /// The assertion is on the **radius of every drawn point**, not on the
    /// segment count: a count would be satisfied by twenty-four segments of any
    /// shape at all, which is the *"a test that checks a relation rather than a
    /// magnitude"* trap `HANDOFF.md` §2 names. A circle drawn at the wrong
    /// radius, or centred on the origin instead of on the fit, fails here.
    #[test]
    fn a_circular_preview_is_the_fitted_circle() {
        const R: f64 = 10.0;
        // A circle of radius 10 centred at (30, 40) — a centre far from the
        // origin, so a preview that forgot to translate cannot pass.
        let fit = pdfcer_core::dimension::fit_circle_taubin(&[
            p(40.0, 40.0),
            p(30.0, 50.0),
            p(20.0, 40.0),
            p(30.0, 30.0),
        ])
        .expect("fits");

        let segs = dimension_preview_segments(&DimensionKind::Circular {
            fit,
            show_diameter: false,
        });
        assert_eq!(
            segs.len(),
            ARC_PREVIEW_STEPS + 1,
            "the full turn plus one radius spoke"
        );
        for (a, b) in segs.iter().take(ARC_PREVIEW_STEPS) {
            for pt in [a, b] {
                let r = (pt.x - fit.center.x).hypot(pt.y - fit.center.y);
                assert!(
                    (r - R).abs() < 0.01,
                    "a preview point sits {r:.3} from the centre, not {R}"
                );
            }
        }
        // The ring closes: the last arc segment ends where the first began.
        let (first, last) = (segs[0].0, segs[ARC_PREVIEW_STEPS - 1].1);
        assert!(
            (first.x - last.x).abs() < 1e-9 && (first.y - last.y).abs() < 1e-9,
            "the ring must close, not leave a gap the operator reads as an arc"
        );
        // The radius mark runs centre → rim.
        assert_eq!(segs[ARC_PREVIEW_STEPS].0, fit.center);

        // ★ …and the diameter draws the SAME circle with the mark across it.
        // `show_diameter` is a display toggle on one fit, so a build that
        // re-fitted or re-sized for it would be committing decision 011's
        // mistake — and an operator toggling between the two would see the
        // circle move.
        let dia = dimension_preview_segments(&DimensionKind::Circular {
            fit,
            show_diameter: true,
        });
        assert_eq!(
            dia[..ARC_PREVIEW_STEPS],
            segs[..ARC_PREVIEW_STEPS],
            "the circle is the same circle; only the mark across it differs"
        );
        let chord = dia[ARC_PREVIEW_STEPS];
        assert!(
            (chord.0.x - chord.1.x).hypot(chord.0.y - chord.1.y) - 2.0 * R < 0.01,
            "the diameter mark spans the whole circle"
        );
    }
}
