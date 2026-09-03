//! # `canvas::measure::circpick` — the radius/diameter tool's point set
//!
//! Split out of [`super::pick`] on 2026-09-03 under **R2** (no `.rs` file over
//! 1,500 lines), and the seam is real rather than convenient.
//!
//! ## ★ What makes this a different subject from the picks beside it
//!
//! `LinearPick`, `TwoLinePick` and `ScalePick` are **fixed-arity state
//! machines**: two points, or two lines, and the machine knows when it is
//! finished. Everything about them is *"which click am I on?"*.
//!
//! This one has no arity. It is a **set** — points go in, points come out
//! again, the fit re-runs over whatever is there, and the operator says when it
//! is done. So its whole vocabulary is different: a removal radius, a row
//! index, an origin per member, a degenerate-set refusal. None of that means
//! anything to a two-click machine, and a reader looking for either subject was
//! finding both.
//!
//! ## ★★★ It picked whole OBJECTS until 2026-09-03
//!
//! The measurement, the operator's report and the argument for the change are
//! all on [`CircularPick`] itself, because that is where a reader arrives.
//! `OPERATOR_REQUESTS.md` O105–O107 carry the operator-facing version.
//!
//! ## What this module does NOT own
//!
//! The **fit** — [`pdfcer_core::dimension::fit_circle_taubin`] — and the
//! **authored value**, `pdfcer-core`'s own `DimensionKind`. Nothing here
//! computes a centre, a radius or a residual. What it owns is *composition*:
//! which points are in the set and how they get in and out.

use pdfcer_core::dimension::{DimensionKind, FitCircle, fit_circle_taubin};
use pdfcer_core::vector::Point;
use pdfcer_core::vector::snap::SnapKind;

/// **Where one picked point came from** — the disclosure that makes a free
/// position honest.
///
/// The circular tool takes two kinds of pick and they are not equally good
/// evidence, so the difference is carried rather than flattened:
///
/// * a **snapped** point is on geometry the document states, and the operator
///   can rely on it being exactly where the drawing says the edge is;
/// * a **free** point is the operator's own judgement of where an edge is,
///   which is the only thing available on a scanned or raster drawing.
///
/// ★★ Both are legitimate and neither is marked on the canvas as provisional —
/// rule 4 forbids that, and applied content renders exactly as saved content
/// will. The disclosure lives **off-canvas**, in the Tool panel's list, where
/// the operator can see that three of their five points were guesses and decide
/// whether the residual they are looking at is good enough. That is the whole
/// distinction between *fuzzy* and *sneaky*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickOrigin {
    /// The snap engine put the point on named geometry, of this kind.
    Snapped(SnapKind),
    /// Nothing was within the catch radius, so the point is where the operator
    /// clicked. **This is what makes a bitmap measurable** — see
    /// `OPERATOR_REQUESTS.md` O106.
    Free,
}

/// One point in the circular fit set.
///
/// Page space (PDF default user space), which is the frame
/// [`fit_circle_taubin`] consumes and the frame `pdfcer dimension-add
/// --points` takes — so a set assembled here and a set assembled on the command
/// line produce the identical circle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircPoint {
    /// Where the fit takes it.
    pub at: Point,
    /// What produced it — for the panel's row and the canvas marker.
    pub origin: PickOrigin,
}

/// **The radius/diameter tool's own pick set — a list of POINTS.**
///
/// Deliberately not `canvas_selection`: a circle-fit attempt has no meaning as
/// the substrate's general object selection (ui-spec §3.1). The fit re-runs
/// live on every change; Finish authors a [`DimensionKind::Circular`].
///
/// # ★★★ It picked whole OBJECTS until 2026-09-03, and that was the defect
///
/// `OPERATOR_REQUESTS.md` O105, in the operator's words:
///
/// > *"selecting a point sometimes makes a big circle, and selecting more
/// > points around a hole doesn't always get it to narrow down to the size of
/// > the hole."*
///
/// A click hit-tested for a **PDF path object** and contributed *every anchor
/// of every subpath of that object* to the fit. On his own drawing
/// (`SW41177.pdf`, page 1, measured with `pdfcer object-list`) three objects
/// carry **4,405**, **4,972** and **6,681** anchors, the largest spanning
/// 550 × 500 pt — half the sheet. One click anywhere on that object handed
/// Taubin's fit six thousand points scattered across the drawing, and the
/// best-fit circle through them is enormous. That is the "big circle", exactly,
/// and it is not intermittent: it depends on whether the hole's arc happens to
/// be its own small object or one of 1,194 subpaths inside a large one, which
/// the operator cannot see and has no reason to think about.
///
/// The second half followed from the same design. A second click on the same
/// object **toggled it out**, so clicking twice around one hole added it and
/// then removed it; and a click on a different object added another thousand-
/// anchor blob, making the fit worse. *"Add more points"* meant the opposite of
/// what it means everywhere else.
///
/// ⇒ **A click is now one point.** Three points on an arc give the arc, which
/// is the model every drafting package uses and the model the operator was
/// already working in — his own sentence says *"selecting more points around a
/// hole"*.
///
/// ★ What is lost is *"click the circle once and be done"*. That is worth
/// having back at **subpath** granularity, because a subpath is the drawn
/// entity and an object is not; it is recorded on O105 as a decision rather
/// than an omission, and is not built here.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CircularPick {
    /// The picked points, in pick order.
    points: Vec<CircPoint>,
    /// Display toggle: `true` ⇒ show the diameter, `false` ⇒ the radius
    /// (ui-spec §3.4). Purely a display choice on the same [`FitCircle`].
    pub show_diameter: bool,
}

impl CircularPick {
    /// A fresh, empty pick set showing the radius.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// **Add a point, or take one out** — the canvas's whole input.
    ///
    /// A click within `tolerance` of a point already in the set **removes that
    /// point**; otherwise the point is appended. Returns `true` when the set
    /// grew, `false` when it shrank.
    ///
    /// # ★ Why proximity rather than equality
    ///
    /// `OPERATOR_REQUESTS.md` O107: *"we should be able to unselect
    /// points/clicked locations"*. An operator taking a point back out aims at
    /// the marker they can see, and the pointer never lands on the exact `f64`
    /// they committed — a free position is a screen pixel converted through the
    /// zoom, so equality would make the canvas route unusable and leave only
    /// the panel.
    ///
    /// The radius is the **snap catch radius** (`PageMapping::snap_tolerance`),
    /// which is the right one and not merely a convenient one: inside it, a
    /// snapped click would have landed on the very point being removed, so the
    /// two readings of the gesture cannot disagree about which point is meant.
    ///
    /// A non-finite or negative tolerance removes nothing, which makes a
    /// degenerate mapping add points rather than silently eat them.
    pub fn toggle_point(&mut self, at: Point, origin: PickOrigin, tolerance: f64) -> bool {
        if tolerance.is_finite() && tolerance > 0.0 {
            let r2 = tolerance * tolerance;
            if let Some(pos) = self.points.iter().position(|p| {
                let dx = p.at.x - at.x;
                let dy = p.at.y - at.y;
                dx.mul_add(dx, dy * dy) <= r2
            }) {
                self.points.remove(pos);
                return false;
            }
        }
        self.points.push(CircPoint { at, origin });
        true
    }

    /// **Remove the point at `index`** — the Tool panel's route.
    ///
    /// Returns the point that went, or `None` for an out-of-range index. The
    /// index is the row's position in [`Self::points`], and the two cannot
    /// drift because the panel draws from that slice in the same frame it
    /// raises the removal.
    ///
    /// Out of range is `None` rather than a panic: a panel row is drawn from
    /// last frame's state and acted on in this one, so a removal racing a
    /// canvas click is an ordinary event and not a bug to crash on.
    pub fn remove(&mut self, index: usize) -> Option<CircPoint> {
        (index < self.points.len()).then(|| self.points.remove(index))
    }

    /// The picked points, in pick order — what the Tool panel lists and what
    /// the canvas draws markers for.
    #[must_use]
    pub fn points(&self) -> &[CircPoint] {
        &self.points
    }

    /// How many points are in the fit set.
    #[must_use]
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// The fit's input — the picked points, in pick order.
    ///
    /// The same point set `pdfcer dimension-add --points` would pass, so the
    /// fit (and thus the authored kind) is byte-identical to the CLI's for the
    /// same picks.
    #[must_use]
    pub fn samples(&self) -> Vec<Point> {
        self.points.iter().map(|p| p.at).collect()
    }

    /// The live best-fit circle over the current set, or `None` for a
    /// degenerate one (fewer than three usable points, or numerically singular
    /// — ui-spec §3.3 / [`fit_circle_taubin`]). Re-run every frame the set
    /// changes; the preview draws it dashed with its residual surfaced.
    #[must_use]
    pub fn fit(&self) -> Option<FitCircle> {
        fit_circle_taubin(&self.samples())
    }

    /// The [`DimensionKind::Circular`] this set authors on Finish, or `None`
    /// when the fit is degenerate — Finish is then not offered, because an
    /// inference pdfcer cannot make is not made on the operator's behalf.
    /// `show_diameter` is the display toggle only.
    #[must_use]
    pub fn author(&self) -> Option<DimensionKind> {
        self.fit().map(|fit| DimensionKind::Circular {
            fit,
            show_diameter: self.show_diameter,
        })
    }

    /// Discard the whole set (Escape stage 1, ui-spec §1.3): stay in the tool,
    /// forget the points. Keeps the display toggle.
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Whether any point is picked — the tool is mid-gesture and discardable.
    #[must_use]
    pub fn in_progress(&self) -> bool {
        !self.points.is_empty()
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

    /// ★ **A click adds a point; a click within the removal radius of one
    /// already in the set takes that one out.**
    #[test]
    fn circular_toggle_adds_then_removes_points() {
        let mut cp = CircularPick::new();
        assert!(cp.toggle_point(p(0.0, 0.0), PickOrigin::Free, 1.0));
        assert!(cp.toggle_point(p(10.0, 10.0), PickOrigin::Free, 1.0));
        assert_eq!(cp.point_count(), 2);
        // A click NEAR the first — not on it — removes the first.
        assert!(!cp.toggle_point(p(0.3, -0.4), PickOrigin::Free, 1.0));
        assert_eq!(cp.point_count(), 1);
        assert_eq!(cp.points()[0].at, p(10.0, 10.0));
    }

    /// ★★ **A zero or non-finite tolerance ADDS rather than silently eating
    /// the point.**
    ///
    /// The degenerate-mapping case, and the direction of the failure is the
    /// whole point: a tool that stopped accepting points would look broken, and
    /// a tool that accepted them and removed something else would be worse —
    /// the operator would watch their set shrink as they clicked.
    #[test]
    fn a_degenerate_removal_radius_never_removes() {
        let mut cp = CircularPick::new();
        cp.toggle_point(p(0.0, 0.0), PickOrigin::Free, 1.0);
        assert!(cp.toggle_point(p(0.0, 0.0), PickOrigin::Free, 0.0));
        assert!(cp.toggle_point(p(0.0, 0.0), PickOrigin::Free, f64::NAN));
        assert!(cp.toggle_point(p(0.0, 0.0), PickOrigin::Free, -1.0));
        assert_eq!(cp.point_count(), 4);
    }

    /// ★ **A row's index removes that row's point, and an out-of-range index
    /// is refused** — the Tool panel's route (`OPERATOR_REQUESTS.md` O107).
    #[test]
    fn a_point_can_be_removed_by_index() {
        let mut cp = CircularPick::new();
        for i in 0..3 {
            cp.toggle_point(p(f64::from(i), 0.0), PickOrigin::Free, 0.5);
        }
        let gone = cp.remove(1).expect("the middle row");
        assert_eq!(gone.at, p(1.0, 0.0));
        assert_eq!(cp.point_count(), 2);
        assert!(cp.remove(9).is_none(), "an out-of-range row is refused");
    }

    /// ★★ **The origin is carried, because a free position and a snapped node
    /// are not the same evidence.**
    ///
    /// The disclosure `OPERATOR_REQUESTS.md` O106 rests on: five free positions
    /// and five snapped nodes produce the same numbers, and only one of the two
    /// is the drawing's own geometry. The canvas does not distinguish them —
    /// rule 4 forbids marking applied content — so this value is the only thing
    /// the Tool panel has to tell the operator with.
    #[test]
    fn a_picks_origin_survives_into_the_set() {
        let mut cp = CircularPick::new();
        cp.toggle_point(p(0.0, 0.0), PickOrigin::Free, 0.5);
        cp.toggle_point(p(5.0, 0.0), PickOrigin::Snapped(SnapKind::Midpoint), 0.5);
        assert_eq!(cp.points()[0].origin, PickOrigin::Free);
        assert_eq!(
            cp.points()[1].origin,
            PickOrigin::Snapped(SnapKind::Midpoint)
        );
    }

    #[test]
    fn circular_fits_a_circle_from_picked_samples_and_authors_it() {
        // Four points on a unit circle centred at (5,5): a clean fit.
        let mut cp = CircularPick::new();
        for at in [p(6.0, 5.0), p(5.0, 6.0), p(4.0, 5.0), p(5.0, 4.0)] {
            cp.toggle_point(at, PickOrigin::Free, 0.1);
        }
        let fit = cp.fit().expect("a 4-point circle fits");
        assert!((fit.center.x - 5.0).abs() < 1e-9);
        assert!((fit.center.y - 5.0).abs() < 1e-9);
        assert!((fit.radius - 1.0).abs() < 1e-9);
        // Author: radius by default.
        let kind = cp.author().unwrap();
        assert_eq!(
            kind,
            DimensionKind::Circular {
                fit,
                show_diameter: false
            }
        );
        assert_eq!(kind.measured_points(), 1.0);
        // Flip the display toggle → diameter (SAME fit, no re-fit).
        cp.show_diameter = true;
        let dia = cp.author().unwrap();
        assert!(matches!(
            dia,
            DimensionKind::Circular {
                show_diameter: true,
                ..
            }
        ));
        assert_eq!(dia.measured_points(), 2.0);
    }

    #[test]
    fn circular_degenerate_set_authors_nothing() {
        let mut cp = CircularPick::new();
        // Fewer than 3 points → no fit, nothing to accept (fuzzy-never-sneaky).
        cp.toggle_point(p(0.0, 0.0), PickOrigin::Free, 0.1);
        cp.toggle_point(p(1.0, 0.0), PickOrigin::Free, 0.1);
        assert!(cp.fit().is_none());
        assert!(cp.author().is_none());
    }

    /// **The canvas-authored == CLI-authored equivalence check (circular).**
    /// The GUI fits its picked points; the CLI
    /// fits the same `--points` vector. Same points ⇒ same `fit_circle_taubin`
    /// result ⇒ identical `DimensionKind::Circular` ⇒ byte-identical output.
    #[test]
    fn gui_circular_kind_equals_cli_circular_kind() {
        let pts = vec![p(10.0, 0.0), p(0.0, 10.0), p(-10.0, 0.0), p(0.0, -10.0)];

        // GUI path: the points arrive as four clicks.
        let mut cp = CircularPick::new();
        for at in &pts {
            cp.toggle_point(*at, PickOrigin::Free, 0.1);
        }
        cp.show_diameter = true;
        let gui_kind = cp.author().unwrap();

        // CLI path: fit the same points, diameter display.
        let cli_kind = DimensionKind::Circular {
            fit: fit_circle_taubin(&pts).unwrap(),
            show_diameter: true,
        };

        assert_eq!(gui_kind, cli_kind);
    }
}
