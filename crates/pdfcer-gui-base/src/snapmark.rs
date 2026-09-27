//! # `snapmark` — the GUI half of snapping: the gates, the cycle, the glyph
//!
//! ## What this group of primitives is
//!
//! The snap **maths** lives in `pdfcer_core::vector::snap` and is GUI-free: give
//! it a page, a query point and a page-space `SnapConfig::tolerance` and it
//! returns a priority-sorted list of [`SnapCandidate`]s. The engine deliberately
//! does **not** own two things, and those two things are this module:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/snapmark.md`.

use egui::{Color32, Pos2, Shape, Stroke};
use pdfcer_core::vector::{SnapCandidate, SnapKind};

use crate::canvasmapping::screen_tolerance_to_page;

/// The default screen-space snap catch radius, in egui logical points
/// (decision 011 §2.2: "≈8–12 px"). Converted to a page-space tolerance each
/// frame by [`screen_tolerance_to_page`] so the snap "feel" is zoom-invariant.
#[allow(
    dead_code,
    reason = "the Pass 12.M1 snap default, salvaged ahead of its consumer; read by the Phase 7 measure tools in `canvas::measure`, which own the tool-mode frame the indicator draws in" // ui-text-exempt: clippy lint justification, never displayed
)]
pub const SNAP_SCREEN_TOLERANCE_PX: f32 = 10.0;

/// The overlay colour role a **live, uncommitted** snap indicator is tinted
/// with: `"preview"` — *an uncommitted proposal*, per
/// `egui-shell/src/theme/overlays.rs`.
pub const SNAP_INDICATOR_ROLE: &str = "preview"; // ui-text-exempt: a theme role key, never displayed

/// The overlay colour role a **committed** dimension is drawn with when
/// selected: `"dimension_selected"`, per `egui-shell/src/theme/overlays.rs`.
pub const SNAP_COMMITTED_ROLE: &str = "dimension_selected"; // ui-text-exempt: a theme role key, never displayed

/// The page-space snap tolerance for `zoom` (logical points per PDF user-space
/// unit) — the **zoom-invariance mechanism** (decision 011 §2.2; the page-space
/// value `snap_candidates` takes).
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
#[must_use]
pub fn snap_indicator_tint(ctx: &egui::Context) -> Option<Color32> {
    egui_shell::theme::Overlays::of(ctx).get(SNAP_INDICATOR_ROLE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvasmapping::SELECT_SCREEN_TOLERANCE_PX;

    /// **The snap catch radius is zoom-invariant on screen.**
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

    /// Every snap kind the **engine** offers draws something, and the derived
    /// centerline's glyph is not the routine one's.
    #[test]
    fn every_snap_kind_has_a_non_empty_marker_and_the_derived_one_is_distinct() {
        let kinds = SnapKind::all();
        // An empty or truncated list from the engine would make the loop below a
        // green test that asserted nothing at all. Cheap to rule out, and this is
        // the failure mode a test named "every" must not have.
        assert!(
            kinds.len() >= 8,
            "`SnapKind::all()` returned {} kinds; this test claims coverage and a \
             short list would make that claim vacuous",
            kinds.len()
        );
        for k in kinds.iter().copied() {
            // NOT A THEME COLOUR: an arbitrary argument; this asserts geometry.
            assert!(
                !snap_marker_shapes(Pos2::new(10.0, 10.0), k, Color32::RED, 4.0).is_empty(),
                "{k:?} draws no marker, so that snap would be silent on screen"
            );
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
    #[test]
    fn an_uninstalled_overlay_set_yields_no_tint_rather_than_a_fallback() {
        let ctx = egui::Context::default();
        assert_eq!(snap_indicator_tint(&ctx), None);
    }
}
