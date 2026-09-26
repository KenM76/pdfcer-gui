//! # `canvas::measure::circular` — the radius/diameter tool, and the gesture
//! the operator has to end
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/measure/circular.md`.

use pdfcer_core::vector::snap::SnapKind;

use super::pick::PickOrigin;
use super::{MeasureKind, MeasureState, read, store};
use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;

/// **The circular pick set that is ready to become a dimension**, or `None`.
fn pending(ctx: &egui::Context) -> Option<MeasureState> {
    if crate::canvas::tool::selected(ctx).measure_kind() != Some(MeasureKind::Circular) {
        return None;
    }
    let st = read(ctx)?;
    st.circular.author().map(|_| st)
}

/// **Is there a circle fit waiting to be committed?** — the application state
/// behind the `measure.finishable` condition.
#[must_use]
pub fn finishable(ctx: &egui::Context) -> bool {
    pending(ctx).is_some()
}

/// **End the gesture: author the dimension and empty the pick set.**
pub(super) fn commit(st: &mut MeasureState, page_index: usize, actions: &mut Vec<Action>) -> bool {
    let Some(kind) = st.circular.author() else {
        return false;
    };
    actions.push(Action::Dimension(DimensionAction::Commit {
        page: page_index,
        group: st.group,
        kind,
        // Nothing to disclose: a best-fit circle's output is the circle the
        // operator assembled, and its residual is already on screen through the
        // live preview. See `DimensionAction::Commit`'s field.
        disclosures: Vec::new(),
    }));
    // Emptied, not left standing. The next dimension starts from nothing, the
    // same way `LinearPick` resets on its placing click — otherwise a second
    // Finish would author the same circle again from a set the operator
    // believes they have already spent.
    st.circular.clear();
    true
}

/// **The `measure.finish` command's whole effect**, reporting whether it did
/// anything.
pub fn finish(ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
    let Some(mut st) = pending(ctx) else {
        return false;
    };
    let page_index = st.page_index;
    if !commit(&mut st, page_index, actions) {
        return false;
    }
    store(ctx, st);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // The `add-dimension` line the engine traces proves the edit landed;
        // this one proves which of the two endings asked for it, which a
        // screenshot cannot distinguish and neither can the engine.
        format!("measure-finish via=command page={page_index}")
    });
    true
}

/// **End the gesture on a double-click.**
pub(super) fn double_click(st: &mut MeasureState, page_index: usize, actions: &mut Vec<Action>) {
    if !commit(st, page_index, actions) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "measure-finish via=double-click outcome=declined reason=degenerate-fit".to_owned()
        });
        return;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("measure-finish via=double-click page={page_index}")
    });
}

/// **Take one point for the radius/diameter tool** — add it, or take it out.
pub(super) fn take_point(
    st: &mut MeasureState,
    at: pdfcer_core::vector::Point,
    candidate: Option<pdfcer_core::vector::snap::SnapCandidate>,
    tolerance: f64,
) -> bool {
    let origin = candidate.map_or(PickOrigin::Free, |c| PickOrigin::Snapped(c.kind));
    let added = st.circular.toggle_point(at, origin, tolerance);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        //
        // `origin` is on the line because a run that measured a raster and a
        // run that measured vector geometry produce the same numbers and are
        // not the same evidence — see `PickOrigin`.
        // The FIT is on this line, and it is the field the operator's
        // report is about.
        //
        // O105 is *"selecting more points around a hole doesn't always get it
        // to narrow down to the size of the hole"*, which is a claim about a
        // number converging. A trace carrying only the count says the click
        // registered and says nothing about whether the answer improved — and
        // on the build that produced the report, the count went up while the
        // radius stayed absurd. `r=none` is the honest reading of a set with
        // fewer than three usable points; it is not zero, because zero is a
        // radius.
        let fit = st.circular.fit();
        format!(
            "measure-circular-point action={} origin={} x={:.3} y={:.3} n={} r={} resid={}",
            if added { "add" } else { "remove" },
            origin_tag(origin),
            at.x,
            at.y,
            st.circular.point_count(),
            fit.map_or_else(|| "none".to_owned(), |f| format!("{:.3}", f.radius)),
            fit.map_or_else(|| "none".to_owned(), |f| format!("{:.4}", f.residual))
        )
    });
    added
}

/// **Remove the point at `index`** — the Tool panel's route into the same set.
pub fn remove_point(ctx: &egui::Context, index: usize) -> bool {
    let Some(mut st) = read(ctx) else {
        return false;
    };
    let Some(gone) = st.circular.remove(index) else {
        return false;
    };
    let remaining = st.circular.point_count();
    store(ctx, st);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "measure-circular-point action=remove via=panel index={index} x={:.3} y={:.3} n={remaining}",
            gone.at.x, gone.at.y
        )
    });
    true
}

/// A short, stable tag for a pick's origin — trace only.
fn origin_tag(origin: PickOrigin) -> &'static str {
    match origin {
        PickOrigin::Free => "free",
        PickOrigin::Snapped(kind) => match kind {
            SnapKind::Node => "node",
            SnapKind::Endpoint => "endpoint",
            SnapKind::Center => "center",
            SnapKind::Midpoint => "midpoint",
            SnapKind::Intersection => "intersection",
            SnapKind::SegmentCenterline => "on-segment",
            SnapKind::DerivedCenterline => "derived-centerline",
            SnapKind::Axis => "axis",
        },
    }
}

/// Plant a pick set in memory, for tests in sibling modules.
#[cfg(test)]
pub(crate) fn plant_pick_for_test(ctx: &egui::Context, page_index: usize) {
    let mut st = MeasureState::for_kind(page_index, MeasureKind::Circular);
    for at in samples_on_a_circle() {
        st.circular
            .toggle_point(at, PickOrigin::Snapped(SnapKind::Node), 0.0);
    }
    store(ctx, st);
}

/// A square inscribed in a circle of radius 10 centred at (30, 40) — a
/// four-point set that fits **exactly**, so the residual is 0 and any drift in
/// the authored geometry shows up rather than being absorbed by the fit.
#[cfg(test)]
fn samples_on_a_circle() -> Vec<pdfcer_core::vector::Point> {
    use pdfcer_core::vector::Point;
    vec![
        Point::new(40.0, 40.0),
        Point::new(30.0, 50.0),
        Point::new(20.0, 40.0),
        Point::new(30.0, 30.0),
    ]
}

#[cfg(test)]
#[allow(clippy::panic, reason = "a test that cannot destructure has failed")] // ui-text-exempt: clippy lint justification, never displayed
mod tests {
    use super::*;
    use crate::canvas::tool::{self, CanvasTool};
    use pdfcer_core::vector::Point;

    /// A snapped node, which is what most picks are.
    const NODE: PickOrigin = PickOrigin::Snapped(SnapKind::Node);

    /// The removal radius these tests pick with. Small enough that the four
    /// fixture points (10 apart) are never mistaken for each other, large
    /// enough that a deliberately-near click lands inside it.
    const TOL: f64 = 1.0;

    /// **A click adds one point; a click near an existing one takes that
    /// point out.**
    #[test]
    fn a_click_adds_a_point_and_a_click_near_it_takes_that_point_out() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);

        assert!(take_point(&mut st, Point::new(10.0, 10.0), None, TOL));
        assert_eq!(st.circular.point_count(), 1, "the click added a point");

        assert!(take_point(&mut st, Point::new(40.0, 40.0), None, TOL));
        assert_eq!(st.circular.point_count(), 2, "a second, far away, added");

        assert!(
            !take_point(&mut st, Point::new(10.4, 10.2), None, TOL),
            "a click INSIDE the removal radius of an existing point removes it"
        );
        assert_eq!(st.circular.point_count(), 1);
        assert_eq!(
            st.circular.points()[0].at,
            Point::new(40.0, 40.0),
            "and it removes the one that was near, not the last one added"
        );
    }

    /// **A click with nothing under it is still a point** —
    /// `OPERATOR_REQUESTS.md` O106, the ask that makes a bitmap measurable.
    #[test]
    fn a_pick_with_no_snap_candidate_is_recorded_as_a_free_position() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        take_point(&mut st, Point::new(1.0, 2.0), None, TOL);
        assert_eq!(
            st.circular.points()[0].origin,
            PickOrigin::Free,
            "no candidate means the operator's own judgement, and it is recorded as such"
        );
    }

    /// **Three free positions fit a circle**, which is the whole of O106.
    #[test]
    fn three_free_positions_on_a_raster_still_produce_a_circle() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        for at in samples_on_a_circle() {
            take_point(&mut st, at, None, TOL);
        }
        let fit = st.circular.fit().expect("four free positions fit");
        assert!(
            (fit.radius - 10.0).abs() < 1e-6 && (fit.center.x - 30.0).abs() < 1e-6,
            "the fit is the circle those points lie on: {fit:?}"
        );
    }

    /// **The panel's removal and the canvas's removal are the same act.**
    #[test]
    fn removing_a_point_from_the_panel_changes_the_set_the_canvas_draws() {
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        plant_pick_for_test(&ctx, 0);
        assert_eq!(read(&ctx).expect("planted").circular.point_count(), 4);

        assert!(remove_point(&ctx, 1), "the second row is removable");
        let st = read(&ctx).expect("still there");
        assert_eq!(st.circular.point_count(), 3);
        assert!(
            !st.circular.points().iter().any(|p| p.at.y > 49.0),
            "and the point that went is the one the row named: {:?}",
            st.circular.points()
        );

        assert!(
            !remove_point(&ctx, 9),
            "an out-of-range row is refused rather than panicking — a row drawn \
             from last frame and acted on in this one is an ordinary race"
        );
    }

    /// **The pick never reaches the selection.**
    ///
    /// A circle-fit attempt has no meaning as the substrate's general object
    /// selection (ui-spec §3.1), and the two must not leak into each other.
    #[test]
    fn the_pick_never_reaches_the_selection() {
        use crate::canvas::selection::{ClickHit, SelectionState};

        let mut selection = SelectionState::default();
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        take_point(&mut st, Point::new(20.0, 20.0), None, TOL);
        assert_eq!(st.circular.point_count(), 1);
        assert!(
            selection.is_empty(),
            "picking a point for a circle fit must not select anything"
        );

        selection.click(
            0,
            ClickHit {
                object: Some(crate::canvas::target::TargetId::Object(1)),
                ..ClickHit::default()
            },
            false,
            false,
        );
        take_point(&mut st, Point::new(21.0, 21.0), None, TOL);
        assert_eq!(selection.len(), 1, "the selection is untouched either way");
    }

    /// **The two endings author the same dimension from the same picks.**
    #[test]
    fn the_double_click_and_the_command_author_the_same_dimension() {
        // Ending 1: the double-click, taken by the canvas.
        let mut by_click = MeasureState::for_kind(2, MeasureKind::Circular);
        for at in samples_on_a_circle() {
            by_click.circular.toggle_point(at, NODE, 0.0);
        }
        let mut click_actions = Vec::new();
        double_click(&mut by_click, 2, &mut click_actions);

        // Ending 2: the ribbon command, through `egui::Memory`.
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        let mut by_command = MeasureState::for_kind(2, MeasureKind::Circular);
        for at in samples_on_a_circle() {
            by_command.circular.toggle_point(at, NODE, 0.0);
        }
        store(&ctx, by_command);
        let mut command_actions = Vec::new();
        assert!(finish(&ctx, &mut command_actions), "the command finishes");

        assert_eq!(
            click_actions, command_actions,
            "the two endings must place the same dimension, on the same page, \
             in the same group"
        );
        assert_eq!(click_actions.len(), 1, "exactly one dimension per ending");
        let Some(Action::Dimension(DimensionAction::Commit { page, kind, .. })) =
            click_actions.first()
        else {
            panic!("a dimension is committed")
        };
        assert_eq!(*page, 2, "on the page the pick was made on, not the view's");
        let pdfcer_core::dimension::DimensionKind::Circular { fit, .. } = kind else {
            panic!("a circular dimension")
        };
        assert!(
            (fit.radius - 10.0).abs() < 1e-6 && (fit.center.x - 30.0).abs() < 1e-6,
            "the committed circle is the fitted one: {fit:?}"
        );
    }

    /// **Both endings empty the pick set**, so a second Finish does not place
    /// the same circle twice.
    #[test]
    fn finishing_empties_the_pick_set_so_it_cannot_be_committed_twice() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        for at in samples_on_a_circle() {
            st.circular.toggle_point(at, NODE, 0.0);
        }
        let mut actions = Vec::new();

        assert!(commit(&mut st, 0, &mut actions));
        assert_eq!(actions.len(), 1);
        assert!(!st.circular.in_progress(), "the set is emptied");
        assert!(
            !commit(&mut st, 0, &mut actions),
            "a second finish has nothing to commit"
        );
        assert_eq!(actions.len(), 1, "and raises nothing");
    }

    /// **A degenerate set commits nothing, from either ending.**
    #[test]
    fn a_degenerate_fit_is_refused_by_both_endings() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        for at in [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(20.0, 0.0),
        ] {
            st.circular.toggle_point(at, NODE, 0.0);
        }
        assert!(st.circular.author().is_none(), "the fixture is degenerate");

        let mut actions = Vec::new();
        assert!(!commit(&mut st, 0, &mut actions));
        assert!(actions.is_empty(), "nothing is authored");
        assert!(
            st.circular.in_progress(),
            "and the picks survive, so the operator can add another point"
        );

        // …and the double-click reaches the same refusal rather than its own.
        double_click(&mut st, 0, &mut actions);
        assert!(actions.is_empty());
        assert!(st.circular.in_progress());
    }

    /// **`measure.finishable` is true exactly when pressing Finish would do
    /// something** — all five of the states that decide it.
    #[test]
    fn finish_is_offered_only_when_there_is_a_fit_and_the_tool_is_armed() {
        let ctx = egui::Context::default();

        // 1. Nothing armed, no state.
        assert!(!finishable(&ctx), "an unarmed canvas has nothing to finish");

        // 2. Armed, but nothing picked.
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        store(&ctx, MeasureState::for_kind(0, MeasureKind::Circular));
        assert!(!finishable(&ctx), "an empty pick set is not a circle");

        // 3. Armed with a real fit.
        plant_pick_for_test(&ctx, 0);
        assert!(finishable(&ctx), "four points on a circle are finishable");

        // 4. The same set, with the tool put down.
        tool::select(&ctx, CanvasTool::Select);
        assert!(
            !finishable(&ctx),
            "a set nothing is marking must not keep offering Finish"
        );
        let mut actions = Vec::new();
        assert!(
            !finish(&ctx, &mut actions),
            "…and the command refuses it too, by the same predicate"
        );
        assert!(actions.is_empty());

        // 5. A *different* measure tool armed is not this tool's ending.
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Linear));
        assert!(!finishable(&ctx));
    }

    /// **Asking whether Finish is available does not manufacture state.**
    #[test]
    fn asking_whether_finish_is_available_creates_no_measure_state() {
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        assert!(!finishable(&ctx));
        assert!(
            read(&ctx).is_none(),
            "the question must not answer itself into existence"
        );
    }
}
