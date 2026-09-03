//! # `canvas::snap` — the GUI half of snapping: the gates, the cycle, the glyph
//!
//! ## What this group of primitives is
//!
//! The snap **maths** lives in `pdfcer_core::vector::snap` and is GUI-free: give
//! it a page, a query point and a page-space `SnapConfig::tolerance` and it
//! returns a priority-sorted list of [`SnapCandidate`]s. The engine deliberately
//! does **not** own two things, and those two things are this module:
//!
//! 1. **Zoom-invariance and the gates.** The engine takes a *page-space*
//!    tolerance; the operator experiences a *screen-space* catch radius. The
//!    conversion between them — and the decision of whether to run the query at
//!    all, given the persistent master toggle and the transient Alt override —
//!    is a GUI question about how the tool should *feel*.
//! 2. **The fuzzy indicator.** Which glyph marks a candidate, how the operator
//!    cycles between competing candidates with Tab, and how many clicks it takes
//!    to commit one. That last is the *fuzzy-never-sneaky* rule made concrete:
//!    the one candidate kind that is an **inference** about operator intent
//!    ([`SnapKind::DerivedCenterline`]) confirms in two clicks; every kind that
//!    is a deterministic fact about geometry already on the page commits on one.
//!
//! Everything here is a pure function over `bool`/`usize`/`&[..]`-shaped inputs
//! or a `Vec<Shape>` builder. Nothing reads global state, nothing mutates a
//! document, and every rule below is unit-tested without a window.
//!
//! ## Where it came from
//!
//! Salvaged from the old shell's `crates/pdfcer-gui/src/canvas.rs:1584-1892`
//! (`D:\Dev\pdfcer`, read-only to this project) — the block headed *"Fuzzy snap
//! indicator — GUI-side logic + rendering primitives (Pass 12.M1)"*, together
//! with its six unit tests at `:3046-3136`. The doc comments come across
//! verbatim, because `SALVAGE.md`'s procedure is explicit that *"the old GUI's
//! value is disproportionately in its doc comments; a snippet leaves those
//! behind and the next engineer re-derives a decision that was already made and
//! already paid for."* The `reason = "…"` strings on the `dead_code` allows are
//! the one exception, and §"The `reason` strings were rewritten" below says why.
//!
//! The old block's own header recorded that these primitives were built
//! *"against the announced contract"* before their consumer existed — the
//! substrate's *"design against the contract, name the asks"* idiom. That is
//! still exactly their status; only the name of the consumer has changed.
//!
//! ## What consumes it
//!
//! The **Phase 7 measure tools**, landing in
//! [`crate::canvas::measure`](super::measure) (`canvas/measure/`). They own the
//! tool-mode frame the indicator draws in, so they are the ones who:
//!
//! | per frame | calls |
//! |---|---|
//! | decide whether to query at all | [`snap_query_enabled`] |
//! | build the engine's `SnapConfig::tolerance` | [`snap_tolerance`] |
//! | pick which candidate is live | [`active_snap_candidate`] |
//! | advance the cycle on <kbd>Tab</kbd> | [`next_snap_index`] |
//! | decide whether a click commits or only proposes | [`snap_commit_clicks`] |
//! | paint the indicator | [`snap_marker_shapes`] |
//!
//! The paint call is a one-liner in the measure-tool overlay handler, unchanged
//! in shape from the old shell's:
//!
//! ```ignore
//! painter.extend(snap::snap_marker_shapes(screen_at, kind, tint, size));
//! painter.text(label_at, .., text::snap_indicator_label(kind), ..);
//! ```
//!
//! ## The two conversions this module does NOT own
//!
//! **Screen → page distance.** [`super::mapping::screen_tolerance_to_page`]
//! already exists in this shell and already carries the `1 / zoom` law with its
//! degenerate-input contract and its tests. This module does not re-derive it:
//! [`snap_tolerance`] is a one-line wrapper that pairs it with
//! [`SNAP_SCREEN_TOLERANCE_PX`]. `mapping`'s header states the invariant
//! plainly — *"there is no second place in `canvas/` that divides by `zoom`"* —
//! and a salvaged duplicate would have been exactly that second place. The old
//! shell's `canvas::screen_tolerance_to_page` therefore does **not** come
//! across; what came across is its snap-specific default and the test that pins
//! the snap radius's zoom-invariance.
//!
//! **Screen → page position.** [`super::mapping::PageMapping::to_page`]. The
//! query point handed to `snap_candidates` is a page coordinate, and the
//! measure tools already hold a `PageMapping` for the frame.
//!
//! ## The tint comes from the caller, and which role it must be
//!
//! [`snap_marker_shapes`] takes `color: Color32` rather than reaching for a
//! theme itself. That is the old shell's shape and it is kept, for the reason
//! that made it right there: the marker is painted *inside* the measure tool's
//! overlay pass, which already knows whether it is drawing a live proposal or a
//! committed dimension, and a painter that resolved its own colour would have to
//! be told that anyway — as a second argument, in a second vocabulary.
//!
//! What the caller must **not** do is invent the colour.
//! `crates/egui-shell/src/theme/overlays.rs` already defines the roles this work
//! needs, and [`SNAP_INDICATOR_ROLE`] / [`SNAP_COMMITTED_ROLE`] name them so a
//! call site cannot typo one into silence. [`snap_indicator_tint`] resolves the
//! first of them; see its docs for the honest `None` it can return and what that
//! `None` currently means in this shell.
//!
//! ## The `reason` strings were rewritten, deliberately
//!
//! Every item below still carries `#[allow(dead_code, reason = …)]`, because in
//! this shell the consumer is *arriving* and has not arrived: `canvas/measure/`
//! exists with a `mod.rs` and two empty leaves.
//!
//! But the inherited reasons all said *"consumed by the Pass 12.M2 measure
//! tools"*. Pass 12.M2 is a milestone in **another repository**, and pointing a
//! live annotation at it would be a claim about this crate that nobody here can
//! check — the same class of stale status-in-a-table that `SALVAGE.md`'s
//! correction note is about (*"a status word in a table is a claim, and it
//! decays"*). So each reason now names the consumer in **this** shell. The
//! provenance is not lost; it is recorded here, in prose, where it is a
//! statement about history rather than a promise about the future.

use egui::{Color32, Pos2, Shape, Stroke};
use pdfcer_core::vector::{SnapCandidate, SnapKind};

use super::mapping::screen_tolerance_to_page;

/// The default screen-space snap catch radius, in egui logical points
/// (decision 011 §2.2: "≈8–12 px"). Converted to a page-space tolerance each
/// frame by [`screen_tolerance_to_page`] so the snap "feel" is zoom-invariant.
///
/// Deliberately a *sibling* of [`super::mapping::SELECT_SCREEN_TOLERANCE_PX`]
/// rather than the same constant, and deliberately **looser** than it: snapping
/// and selection answer different questions and are allowed to drift apart. A
/// snap that grabs a nearby vertex is a helpful correction the operator can see
/// and cycle through with <kbd>Tab</kbd>; a selection that grabs a neighbouring
/// object is a silent wrong answer. The failure modes are not symmetric, so the
/// tolerances are not either. (That asymmetry is stated from the selection side
/// in `mapping`'s own docs; [`tests::the_snap_radius_is_looser_than_the_selection_radius`]
/// pins the direction so a future tuning pass cannot invert it by accident.)
#[allow(
    dead_code,
    reason = "the Pass 12.M1 snap default, salvaged ahead of its consumer; read by the Phase 7 measure tools in `canvas::measure`, which own the tool-mode frame the indicator draws in" // ui-text-exempt: clippy lint justification, never displayed
)]
pub const SNAP_SCREEN_TOLERANCE_PX: f32 = 10.0;

/// The overlay colour role a **live, uncommitted** snap indicator is tinted
/// with: `"preview"` — *an uncommitted proposal*, per
/// `egui-shell/src/theme/overlays.rs`.
///
/// A snap marker is drawn while the operator is still aiming, before any click
/// has committed anything, so it is a proposal by definition. Naming the role in
/// a constant rather than spelling it at each call site is not ceremony:
/// [`egui_shell::theme::Overlays::get`] returns `Option` for an unknown role, so
/// a typo does not fail — it draws nothing, on whichever preset the typo was
/// written under.
pub const SNAP_INDICATOR_ROLE: &str = "preview"; // ui-text-exempt: a theme role key, never displayed

/// The overlay colour role a **committed** dimension is drawn with when
/// selected: `"dimension_selected"`, per `egui-shell/src/theme/overlays.rs`.
///
/// Not used by [`snap_marker_shapes`] — a snap marker is never committed state —
/// but named here beside its partner because the two form the **preview-vs-
/// committed pair** that `overlays.rs` exists to keep distinct:
///
/// > the measurement preview and the committed dimension differ because one is
/// > a proposal and one is document state […] a theme that merges two roles
/// > removes a cue that was doing work, and it would do so silently.
///
/// The measure hosting owes `Overlays::assert_distinct(&[…])` over both, once,
/// per preset — that is the test `overlays.rs` says the application owes and the
/// shell cannot write for it.
pub const SNAP_COMMITTED_ROLE: &str = "dimension_selected"; // ui-text-exempt: a theme role key, never displayed

/// The page-space snap tolerance for `zoom` (logical points per PDF user-space
/// unit) — the **zoom-invariance mechanism** (decision 011 §2.2; the page-space
/// value `snap_candidates` takes).
///
/// A constant on-screen catch radius maps to a *shrinking* page-space tolerance
/// as the operator zooms in, so the "feel" stays constant. The `1 / zoom`
/// distance law itself, and its contract that a non-finite or non-positive
/// `zoom` yields `0.0` (snapping disabled) rather than a NaN/∞ tolerance the
/// engine would reject anyway, both live in
/// [`super::mapping::screen_tolerance_to_page`] and are **not** re-implemented
/// here — see this module's header on why a second divider by `zoom` in
/// `canvas/` would be the defect rather than the salvage.
///
/// # Why this takes a bare `zoom` and not a [`super::mapping::PageMapping`]
///
/// `PageMapping` has no `zoom()` accessor, on purpose: its docs record that
/// *"the zoom's whole job here is to be divided by, and exposing it would be an
/// invitation to divide by it at a call site"*, and its one tolerance method
/// [`super::mapping::PageMapping::tolerance`] is the **selection** radius. So
/// there are two honest options and this is the smaller one — a caller that has
/// the frame's `ViewState::zoom` passes it. If the measure hosting turns out to
/// hold only a `PageMapping`, the right fix is a `snap_tolerance()` method on
/// `PageMapping` beside its selection sibling, which belongs to `mapping`'s
/// owner rather than here; this function then becomes its body.
#[allow(
    dead_code,
    reason = "the Pass 12.M1 zoom-invariance conversion, salvaged ahead of its consumer; called each frame by the Phase 7 measure tools in `canvas::measure` to build `SnapConfig::tolerance`" // ui-text-exempt: clippy lint justification, never displayed
)]
#[must_use]
pub fn snap_tolerance(zoom: f32) -> f64 {
    screen_tolerance_to_page(SNAP_SCREEN_TOLERANCE_PX, zoom)
}

/// Whether a snap query should run for the current pick (ui-spec §2.4): the
/// persistent master "Snap to content" toggle is ON **and** the transient Alt
/// override is NOT held. With snapping disabled either way, the pick is the raw
/// pointer position — no candidates queried, no indicator drawn.
#[allow(
    dead_code,
    reason = "the Pass 12.M1 master-toggle + Alt-override gate, salvaged ahead of its consumer; consulted before every pick by the Phase 7 measure tools in `canvas::measure`" // ui-text-exempt: clippy lint justification, never displayed
)]
#[must_use]
pub fn snap_query_enabled(master_on: bool, alt_held: bool) -> bool {
    master_on && !alt_held
}

/// The Tab-cycle index after advancing over a candidate list of `len`
/// (ui-spec §2.4), wrapping to `0` past the end. `len == 0` stays `0` (nothing
/// to cycle). Index 0 is the engine's default pick (highest priority, nearest);
/// Tab steps through the tied/competing candidates the engine returned.
#[allow(
    dead_code,
    reason = "the Pass 12.M1 Tab-cycle advance, salvaged ahead of its consumer; driven by the Phase 7 measure tools' key handling in `canvas::measure`" // ui-text-exempt: clippy lint justification, never displayed
)]
#[must_use]
pub fn next_snap_index(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (current + 1) % len }
}

/// The active snap candidate for a Tab-cycle index, wrapped into range
/// (ui-spec §2.4). Returns `None` for an empty list — no candidate within
/// tolerance, so the indicator is hidden and the pick is the raw pointer
/// position. A stale `cycle` past the list length wraps rather than panicking
/// (the list can shrink between frames as the pointer moves).
#[allow(
    dead_code,
    reason = "the Pass 12.M1 active-candidate selection, salvaged ahead of its consumer; read each frame by the Phase 7 measure tools in `canvas::measure`" // ui-text-exempt: clippy lint justification, never displayed
)]
#[must_use]
pub fn active_snap_candidate(cands: &[SnapCandidate], cycle: usize) -> Option<SnapCandidate> {
    if cands.is_empty() {
        None
    } else {
        Some(cands[cycle % cands.len()])
    }
}

/// How many clicks confirm a pick on a candidate of `kind` (ui-spec §2.3): TWO
/// for a derived centerline — the one fuzzy inference, where the first click
/// only *promotes* the candidate to "proposed" and a second confirms it (a
/// proportionate, non-modal two-click gate, never an auto-apply) — and ONE for
/// every routine kind, a deterministic geometry fact that commits on the single
/// pick. This is the fuzzy-never-sneaky gate (rule 4) encoded for the measure
/// pick handler; it reads `SnapKind::is_derived` so the policy lives in one place.
#[allow(
    dead_code,
    reason = "the Pass 12.M1 two-click-confirm policy, salvaged ahead of its consumer; enforced by the Phase 7 measure-tool pick handler in `canvas::measure`" // ui-text-exempt: clippy lint justification, never displayed
)]
#[must_use]
pub fn snap_commit_clicks(kind: SnapKind) -> u8 {
    if kind.is_derived() { 2 } else { 1 }
}

/// The egui shapes that draw the distinct marker glyph for a snap candidate of
/// `kind` at screen position `at` (ui-spec §2.2). **Shape distinguishes the
/// kind — colour is never the sole signal** (rule 6): a node is a filled
/// square, an endpoint a filled circle, a center a crosshair-in-circle, a
/// midpoint a triangle, an intersection a cross, a routine centerline a dashed
/// tick, an axis a grid glyph, and the DERIVED centerline a **hatched square**,
/// visually unmistakable from the routine centerline tick so the extra-confirm
/// candidate always reads differently (§2.3.1). `size` is the marker half-extent
/// in points; `color` tints every stroke/fill. The measure tool paints these
/// via the live-preview overlay painter (never a re-raster) and draws the label
/// text as a separate galley beside them.
///
/// # The tint is an argument, and it must be a named role
///
/// `color` is supplied by the caller — the old shell's shape, kept. The caller
/// is the measure tool's overlay pass, and the colour it must supply for a
/// pre-commit indicator is the `"preview"` role: [`SNAP_INDICATOR_ROLE`], via
/// [`snap_indicator_tint`]. Nothing in this function chooses a colour, which is
/// why `tools/gates/check-theme-colors.sh` has nothing to say about it.
///
/// The one `Color32::TRANSPARENT` below is the *absence* of a fill on an
/// outline-only polygon, not a choice of colour — which is precisely why the
/// gate's pattern deliberately excludes it.
#[allow(
    dead_code,
    reason = "the Pass 12.M1 indicator rendering primitive, salvaged ahead of its consumer; painted by the Phase 7 measure tools' overlay pass in `canvas::measure`" // ui-text-exempt: clippy lint justification, never displayed
)]
#[must_use]
pub fn snap_marker_shapes(at: Pos2, kind: SnapKind, color: Color32, size: f32) -> Vec<Shape> {
    let s = size.max(1.0);
    let stroke = Stroke::new(1.5, color);
    let sq = |half: f32| -> Vec<Pos2> {
        vec![
            Pos2::new(at.x - half, at.y - half),
            Pos2::new(at.x + half, at.y - half),
            Pos2::new(at.x + half, at.y + half),
            Pos2::new(at.x - half, at.y + half),
        ]
    };
    match kind {
        SnapKind::Node => {
            // ◼ filled square.
            vec![Shape::convex_polygon(sq(s), color, Stroke::NONE)]
        }
        SnapKind::Endpoint => {
            // ● filled circle.
            vec![Shape::circle_filled(at, s, color)]
        }
        SnapKind::Center => {
            // ⊕ crosshair in a circle.
            vec![
                Shape::circle_stroke(at, s, stroke),
                Shape::line_segment(
                    [Pos2::new(at.x - s, at.y), Pos2::new(at.x + s, at.y)],
                    stroke,
                ),
                Shape::line_segment(
                    [Pos2::new(at.x, at.y - s), Pos2::new(at.x, at.y + s)],
                    stroke,
                ),
            ]
        }
        SnapKind::Midpoint => {
            // ▲ up-pointing triangle.
            let tri = vec![
                Pos2::new(at.x, at.y - s),
                Pos2::new(at.x + s, at.y + s),
                Pos2::new(at.x - s, at.y + s),
            ];
            vec![Shape::convex_polygon(tri, color, Stroke::NONE)]
        }
        SnapKind::Intersection => {
            // ✕ diagonal cross.
            vec![
                Shape::line_segment(
                    [Pos2::new(at.x - s, at.y - s), Pos2::new(at.x + s, at.y + s)],
                    stroke,
                ),
                Shape::line_segment(
                    [Pos2::new(at.x - s, at.y + s), Pos2::new(at.x + s, at.y - s)],
                    stroke,
                ),
            ]
        }
        SnapKind::SegmentCenterline => {
            // ┄ dashed tick: two short colinear dashes.
            vec![
                Shape::line_segment(
                    [Pos2::new(at.x - s, at.y), Pos2::new(at.x - s * 0.25, at.y)],
                    stroke,
                ),
                Shape::line_segment(
                    [Pos2::new(at.x + s * 0.25, at.y), Pos2::new(at.x + s, at.y)],
                    stroke,
                ),
            ]
        }
        SnapKind::DerivedCenterline => {
            // ▤ hatched square — a square OUTLINE plus two diagonal hatch
            // lines, deliberately distinct from the routine centerline tick so
            // the extra-confirm candidate is unmistakable (§2.3.1).
            vec![
                Shape::convex_polygon(sq(s), Color32::TRANSPARENT, stroke),
                Shape::line_segment(
                    [Pos2::new(at.x - s, at.y + s), Pos2::new(at.x + s, at.y - s)],
                    stroke,
                ),
                Shape::line_segment(
                    [Pos2::new(at.x - s, at.y), Pos2::new(at.x, at.y - s)],
                    stroke,
                ),
            ]
        }
        SnapKind::Axis => {
            // ⊞ grid glyph: a square outline crossed by one H and one V line.
            vec![
                Shape::convex_polygon(sq(s), Color32::TRANSPARENT, stroke),
                Shape::line_segment(
                    [Pos2::new(at.x - s, at.y), Pos2::new(at.x + s, at.y)],
                    stroke,
                ),
                Shape::line_segment(
                    [Pos2::new(at.x, at.y - s), Pos2::new(at.x, at.y + s)],
                    stroke,
                ),
            ]
        }
    }
}

/// The tint a **live, uncommitted** snap indicator must be painted with: this
/// frame's `"preview"` overlay role ([`SNAP_INDICATOR_ROLE`]).
///
/// # Why this returns `Option` and does not fall back
///
/// [`egui_shell::theme::Overlays::get`] returns `Option` on purpose, and its
/// docs say why: *"a missing role is a programming error — a typo, or a role the
/// preset forgot — and returning magenta or transparent would make it a
/// rendering question the reader has to notice, on the frame where it happens,
/// on the preset where it happens."* Substituting a fallback here would undo
/// that, one layer further from the palette.
///
/// # ★ It used to return `None` on every frame, and that WAS the finding
///
/// This section read: *"`pdfcer-gui` does **not** yet call
/// `Overlays::install` anywhere; the application-role map the shell provides
/// is unused."* True for a whole phase, during which the snap marker silently
/// fell back to the selection stroke — the exact shape of failure the `Option`
/// makes invisible, because nothing looks broken and the cue is simply not
/// there.
///
/// `crate::canvas::overlays::install` now runs beside `Theme::apply` in
/// `crate::app::frame`, so the role resolves. The `Option` stays, and stays
/// meaningful: it is still the honest answer for a role a future preset forgets
/// to define, and `overlays`' own test asserts that none of the roles this
/// canvas reads is one of them, on every preset rather than on the default.
#[must_use]
pub fn snap_indicator_tint(ctx: &egui::Context) -> Option<Color32> {
    egui_shell::theme::Overlays::of(ctx).get(SNAP_INDICATOR_ROLE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::mapping::SELECT_SCREEN_TOLERANCE_PX;

    /// **The snap catch radius is zoom-invariant on screen.**
    ///
    /// Carried from the old shell's
    /// `screen_tolerance_converts_inversely_with_zoom`, re-pointed at
    /// [`snap_tolerance`]. The old test asserted the raw conversion; that
    /// function now lives in [`super::super::mapping`] and is tested there, so
    /// re-asserting it here would be the duplicate this salvage set out to
    /// avoid. What is *not* tested there and is tested here is the pairing —
    /// that the SNAP radius is the one being converted, and that the degenerate
    /// contract survives the wrapper.
    #[test]
    fn the_snap_tolerance_converts_inversely_with_zoom() {
        // A fixed 10px catch radius is 10 page units at 100%, 5 at 200%, 20 at
        // 50% — the zoom-invariance the snap "feel" depends on.
        assert!((snap_tolerance(1.0) - 10.0).abs() < f64::EPSILON);
        assert!((snap_tolerance(2.0) - 5.0).abs() < f64::EPSILON);
        assert!((snap_tolerance(0.5) - 20.0).abs() < f64::EPSILON);
        // Degenerate zoom disables snapping (0 tolerance, which the engine
        // rejects) rather than yielding a NaN/inf.
        assert!((snap_tolerance(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((snap_tolerance(f32::NAN) - 0.0).abs() < f64::EPSILON);
    }

    /// **The snap radius is LOOSER than the selection radius, and the direction
    /// is the point.**
    ///
    /// New here; the old shell stated the asymmetry in prose on both constants
    /// and asserted it nowhere. A tuning pass that nudged one of the two numbers
    /// could silently invert the relation, and the result would not look like a
    /// bug — selection would just start grabbing neighbours while snapping got
    /// fussy, which is the pair of symptoms the prose exists to prevent.
    ///
    /// Both sides are constants, so this is a `const` block: the invariant is
    /// checked when the test module is *compiled*, and an inversion fails the
    /// build rather than one test run. Clippy insists on it
    /// (`assertions_on_constants`) and clippy is right — but the test wrapper is
    /// kept anyway, because a bare `const _: () = assert!(…)` at module scope
    /// has no name, and this invariant is one a reader should be able to find by
    /// running `cargo test canvas::snap` and reading the list.
    #[test]
    fn the_snap_radius_is_looser_than_the_selection_radius() {
        const {
            assert!(
                SNAP_SCREEN_TOLERANCE_PX > SELECT_SCREEN_TOLERANCE_PX,
                "a snap that grabs a nearby vertex is a visible, cyclable correction; \
                 a selection that grabs a neighbour is a silent wrong answer, so \
                 selection must stay the tighter of the two"
            );
        }
    }

    #[test]
    fn snap_is_enabled_only_with_master_on_and_alt_up() {
        assert!(snap_query_enabled(true, false));
        assert!(!snap_query_enabled(false, false)); // master toggle off
        assert!(!snap_query_enabled(true, true)); // Alt transiently suppresses
        assert!(!snap_query_enabled(false, true));
    }

    #[test]
    fn tab_cycle_wraps_and_handles_empty() {
        assert_eq!(next_snap_index(0, 3), 1);
        assert_eq!(next_snap_index(2, 3), 0); // wraps past the end
        assert_eq!(next_snap_index(0, 0), 0); // nothing to cycle
        assert_eq!(next_snap_index(5, 0), 0);
    }

    #[test]
    fn active_candidate_indexes_and_wraps() {
        let c = |k| SnapCandidate {
            point: pdfcer_core::vector::Point::new(0.0, 0.0),
            kind: k,
            source_object: None,
        };
        let list = [c(SnapKind::Node), c(SnapKind::Midpoint)];
        assert_eq!(
            active_snap_candidate(&list, 0).unwrap().kind,
            SnapKind::Node
        );
        assert_eq!(
            active_snap_candidate(&list, 1).unwrap().kind,
            SnapKind::Midpoint
        );
        // A stale index past the end wraps (3 % 2 == 1) rather than panicking.
        assert_eq!(
            active_snap_candidate(&list, 3).unwrap().kind,
            SnapKind::Midpoint
        );
        assert!(active_snap_candidate(&[], 0).is_none());
    }

    #[test]
    fn derived_centerline_needs_two_clicks_others_one() {
        // The fuzzy-never-sneaky gate: the derived centerline confirms in two
        // clicks; every deterministic kind commits on one.
        assert_eq!(snap_commit_clicks(SnapKind::DerivedCenterline), 2);
        assert_eq!(snap_commit_clicks(SnapKind::Node), 1);
        assert_eq!(snap_commit_clicks(SnapKind::SegmentCenterline), 1);
    }

    #[test]
    fn every_snap_kind_has_a_non_empty_marker_and_the_derived_one_is_distinct() {
        let kinds = [
            SnapKind::Node,
            SnapKind::Endpoint,
            SnapKind::Center,
            SnapKind::Midpoint,
            SnapKind::Intersection,
            SnapKind::DerivedCenterline,
            SnapKind::SegmentCenterline,
            SnapKind::Axis,
        ];
        for k in kinds {
            // NOT A THEME COLOUR: an arbitrary argument; this asserts geometry.
            assert!(!snap_marker_shapes(Pos2::new(10.0, 10.0), k, Color32::RED, 4.0).is_empty());
        }
        // The derived centerline's glyph must not be visually confused with the
        // routine centerline tick (§2.3.1) — here proven by a different shape
        // composition (a hatched square vs. two dashes).
        let derived =
            // NOT A THEME COLOUR: an arbitrary argument; this asserts geometry.
            snap_marker_shapes(Pos2::ZERO, SnapKind::DerivedCenterline, Color32::RED, 4.0);
        let routine =
            // NOT A THEME COLOUR: an arbitrary argument; this asserts geometry.
            snap_marker_shapes(Pos2::ZERO, SnapKind::SegmentCenterline, Color32::RED, 4.0);
        assert_ne!(derived.len(), routine.len());
    }

    /// **The role names are the ones `overlays.rs` defines, spelled once.**
    ///
    /// Cheap, and it guards the exact failure the `Overlays` docs describe: an
    /// unknown role is `None`, not an error, so a misspelled key draws nothing
    /// and says nothing. Pinning the literals here means a rename in the theme
    /// breaks a test rather than a frame.
    #[test]
    fn the_indicator_and_committed_roles_are_the_pair_the_theme_defines() {
        assert_eq!(SNAP_INDICATOR_ROLE, "preview"); // ui-text-exempt: a theme role key, never displayed
        assert_eq!(SNAP_COMMITTED_ROLE, "dimension_selected"); // ui-text-exempt: a theme role key, never displayed
        assert_ne!(
            SNAP_INDICATOR_ROLE, SNAP_COMMITTED_ROLE,
            "the preview-vs-committed pair must stay two roles, not one"
        );
    }

    /// **With no `Overlays` set installed, the tint is `None` rather than a
    /// substitute colour.**
    ///
    /// This is the state this shell is in today (see [`snap_indicator_tint`]'s
    /// docs), and it is asserted rather than merely noted so that the day the
    /// application starts installing a set, this test is the thing that says so.
    #[test]
    fn an_uninstalled_overlay_set_yields_no_tint_rather_than_a_fallback() {
        let ctx = egui::Context::default();
        assert_eq!(snap_indicator_tint(&ctx), None);
    }
}
