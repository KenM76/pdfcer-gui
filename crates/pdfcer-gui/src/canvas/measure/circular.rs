//! # `canvas::measure::circular` — the radius/diameter tool, and the gesture
//! the operator has to end
//!
//! The canvas hosting for [`MeasureKind::Circular`] alone: what one click does
//! to the fit set, what the two endings do, and what has to be resolved out of
//! the decomposition so the set can be drawn. [`super`] hosts the other two
//! tools and everything the three share — the memory, the snap resolution, the
//! preview painting.
//!
//! ## ★ Why this is a file of its own, and what the seam actually is
//!
//! **R2** (no `.rs` file over 1,500 lines) forced a split when the tool was
//! armed: [`super`] reached 1,617 lines. But the line count only says *that*
//! something had to move; it does not say what, and `tools/gates/check-file-size.sh`
//! says in its own header that shaving prose to fit a threshold is the
//! behaviour it exists to refuse. So the question was which subject was
//! separable, and this one is, for a reason none of the other tools give:
//!
//! > **Linear and two-line gestures end themselves. This one does not.**
//!
//! A linear dimension is finished at its third click and a two-line dimension
//! at its second, because both are picks of a **fixed arity** — the pick
//! machine in [`super::pick`] knows it is done, and [`super::click`] simply
//! raises whatever the machine hands back. A best-fit circle has no such
//! number. An arc drawn as four separate polyline objects needs four picks; the
//! same arc drawn as one needs one; nothing in the geometry can tell pdfcer
//! which the operator meant. So the operator says when, and the machinery for
//! *saying when* — two entrances, one commit path, a predicate the ribbon reads
//! every frame to decide whether the control is even live — is a subject the
//! other two tools have nothing corresponding to.
//!
//! That is the seam. Everything here answers *"when is this gesture over, and
//! what does ending it do?"*; everything left in [`super`] answers *"where did
//! that click land?"*.
//!
//! ## The two endings, and why there is exactly one commit path
//!
//! | ending | entrance | why it exists |
//! |---|---|---|
//! | **double-click** on the canvas | [`click`], via [`super::click`]'s `double` flag | what every drawing package's multi-pick tool uses; the standing *"make it work the way other programs do"* tie-breaker |
//! | **`measure.finish`** on the ribbon | [`finish`], via `app::dispatch` | discoverable without knowing the double-click, and reachable when the last picked arc sits somewhere awkward to double-click |
//!
//! Both call [`commit`] and nothing else raises a circular
//! `Action::CommitDimension`. Two arms that each assembled a `DimensionKind`
//! would be two derivations of one answer: they would agree on the day they
//! were written, diverge at the first change to either, and **the operator
//! would have no way to see it** — a circle fitted from the same points looks
//! the same whichever code drew it.
//!
//! Neither ending is an accept box floating over the canvas, which is what
//! decision 024 retired at the operator's instruction and what kept this tool
//! unarmed through Phase 7.
//!
//! ## This module owns no geometry either
//!
//! The fit is [`pdfcer_core::dimension::fit_circle_taubin`], reached through
//! [`super::pick::CircularPick`]; the authored value is `pdfcer-core`'s own
//! `DimensionKind`. Nothing here computes a centre, a radius or a residual.
//! What it owns is *composition and lifetime*: which objects are in the set,
//! when the set becomes a dimension, and when it is emptied.

use pdfcer_core::vector::snap::SnapKind;

use super::pick::PickOrigin;
use super::{MeasureKind, MeasureState, read, store};
use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;

/// **The circular pick set that is ready to become a dimension**, or `None`.
///
/// The single derivation behind both halves of the Finish control:
/// [`finishable`] asks whether to enable it and [`finish`] asks what to do when
/// it is pressed. Two spellings of "is there something to finish?" would
/// eventually disagree, and the way they would disagree is the worst available
/// — an enabled control that does nothing when pressed, which is precisely the
/// placeholder the no-placeholders invariant forbids.
///
/// Three conditions, and each rules out a state that really occurs:
///
/// 1. **The radius/diameter tool is armed.** The pick set outlives disarming
///    (`disarm_measure` puts the tool down; it does not discard work — see its
///    docs, and Escape's two rungs), so without this the ribbon would offer
///    Finish for a set the operator can no longer see being outlined.
/// 2. **A state exists.** Nothing has been picked on this page since the tool
///    was armed, so there is nothing to finish.
/// 3. **The fit is not degenerate** — [`super::pick::CircularPick::author`]
///    returns `None` for fewer than three usable points or a numerically
///    singular set, and its own docs say that is when Accept must not be
///    offered. One or two picked objects whose anchors lie on a line is the
///    ordinary way to reach it, not a pathological one.
fn pending(ctx: &egui::Context) -> Option<MeasureState> {
    if crate::canvas::tool::selected(ctx).measure_kind() != Some(MeasureKind::Circular) {
        return None;
    }
    let st = read(ctx)?;
    st.circular.author().map(|_| st)
}

/// **Is there a circle fit waiting to be committed?** — the application state
/// behind the `measure.finishable` condition.
///
/// Published by `crate::app::PdfcerApp::conditions` and read by
/// `measure.finish`'s `enabled_when`. See [`pending`] for what the three
/// conditions are and why each one is needed.
#[must_use]
pub fn finishable(ctx: &egui::Context) -> bool {
    pending(ctx).is_some()
}

/// **End the gesture: author the dimension and empty the pick set.**
///
/// ★ **The one commit path**, reached by both endings — see the module header
/// for the argument, which is the reason this function exists rather than two
/// arms that each build a `DimensionKind`.
///
/// Pure over the state and the action list — no `egui`, no context, no memory —
/// which is what makes both endings assertable without a window.
///
/// Returns `false` and raises nothing when the fit is degenerate. That is the
/// same refusal [`super::pick::CircularPick::author`] states: an inference
/// pdfcer cannot make is not made silently on the operator's behalf.
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
///
/// The second entrance to [`commit`], and the only thing it adds is the trip
/// through `egui::Memory`: read the state, run the one commit path, write it
/// back. The page comes from the **state**, not from the current view, because
/// the pick was made on that page and a state whose page has been left behind
/// is cleared by `super::load` on the next frame anyway — reading
/// `doc.view.page_index` here would be a second source of truth for a fact the
/// state already carries.
///
/// Returns `false` when there is nothing to finish, so the dispatcher can say
/// which kind of nothing happened rather than tracing a success it did not
/// have.
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
///
/// The canvas half of the two endings — the other is `measure.finish` on the
/// ribbon, and both reach [`commit`] and nothing else. Traces which ending
/// asked, because a screenshot cannot distinguish them and neither can the
/// engine.
///
/// ★ **The first click of the pair has already picked a point**, and that is
/// deliberate rather than an accident of how `egui` reports a double-click.
/// Swallowing the pair would make the operator's last point need a separate
/// click *and* a double-click somewhere harmless. It is also what
/// `SelectionState::click` does with the same flag: the second click gets its
/// own meaning rather than repeating the first's.
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
///
/// `at` is the point [`super::click`]'s snap machinery resolved: the anchor
/// under the pointer, or the raw pointer position when nothing was within the
/// catch radius. `candidate` is what produced it, `None` meaning the raw
/// position.
///
/// # ★★★ Why this goes THROUGH the snap machinery, when it used to go around it
///
/// Until 2026-09-03 the circular pick was taken **before** the point
/// resolution, and there was a written argument for it: the pick committed no
/// point, it toggled an *object*, and the object under the pointer is the same
/// object whether or not there is a midpoint six pixels away. That argument was
/// sound and its premise is gone — see [`super::pick::CircularPick`] for the
/// measurement that removed it, and `OPERATOR_REQUESTS.md` O105.
///
/// Now that a pick **is** a point, every part of that machinery is exactly what
/// this tool wants:
///
/// * the **snap** puts the point on the drawing's own geometry, so three clicks
///   round a hole give the hole rather than three approximations of it;
/// * the **raw fallback** is what makes a bitmap measurable at all (O106) —
///   `resolve::snapped` returns the pointer unchanged when nothing is near, and
///   the operator's judgement is then the measurement;
/// * the **derived-candidate two-click confirm** is rule 4 doing its job. A
///   centerline pdfcer inferred is not committed by the click that finds it.
///   Under the object pick that confirm was a cost with no benefit; under a
///   point pick it is the difference between fuzzy and sneaky.
///
/// # Returns
///
/// `true` when the set grew, `false` when the click took a point back out.
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
        // ★★★ The FIT is on this line, and it is the field the operator's
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
///
/// ★★ Two routes to one capability, and the panel's is the one that cannot be
/// substituted. A pick set on a dense CAD sheet is invisible: the operator
/// cannot tell four picked points from five, and cannot tell *which* four. See
/// `OPERATOR_REQUESTS.md` O107.
///
/// Reads, mutates and writes back through [`super::read`]/[`super::store`],
/// which is the same trip `finish` makes, so a panel removal and a canvas click
/// are the same act on the same state.
///
/// Returns `false` when there is no such point — an out-of-range index is an
/// ordinary race between a panel row drawn from last frame and a canvas click
/// taken in this one, not a bug to crash on.
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
///
/// Deliberately not the operator-facing wording: a trace field is a machine
/// contract a driven check matches on, and coupling it to a translatable string
/// would make a wording change break the harness. `crate::text` owns what the
/// operator reads.
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
///
/// Two test modules need one and neither can build it the honest way, so the
/// visibility widens rather than the helper being written twice —
/// `crate::app::state::open_fixture`'s own note makes the identical argument.
/// `canvas::keys` owns Escape's precedence and has to assert that a circular
/// pick set is abandoned one press *before* the tool is put down;
/// `app::conditions` has to assert that a finishable set is still not offered
/// with no document open. Neither can assemble one the real way — that needs a
/// laid-out page, a decomposition and a click inside a drawn object — so the
/// state they must react to is planted directly, exactly as
/// `crate::canvas::guides::plant_drag_for_test` plants a guide drag for the
/// same Escape test.
///
/// `#[cfg(test)]` so it cannot become a second way for production code to build
/// a pick set. The real one is [`take_point`], and a second entry point is how two
/// code paths come to disagree about what a pick is.
///
/// The four points are a square inscribed in a circle of radius 10 centred at
/// (30, 40) — a **non-degenerate** set, so the planted state is one
/// [`finishable`] answers `true` for. A collinear or too-small set would make
/// every Escape test pass for the wrong reason, since the fit would be `None`
/// and nothing downstream would ever be offered.
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

    /// ★★★ **A click adds one point; a click near an existing one takes that
    /// point out.**
    ///
    /// The whole of the pick, and both halves matter. A build that only added
    /// would pass a test of the first click alone, and the operator's complaint
    /// would be `OPERATOR_REQUESTS.md` O107 — *"I can't unselect things once I
    /// have selected them"* — which is how this behaviour came to be asked for
    /// in the first place.
    ///
    /// ★ The second assertion is the one that could not exist under the old
    /// object pick: it adds a point 0.5 pt from the first, and requires the set
    /// to SHRINK. Under an object pick there was no such distance — the unit
    /// was the whole object — which is exactly why *"selecting more points
    /// around a hole"* did not narrow the fit.
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

    /// ★★ **A click with nothing under it is still a point** —
    /// `OPERATOR_REQUESTS.md` O106, the ask that makes a bitmap measurable.
    ///
    /// The origin is carried rather than discarded, because a set of five free
    /// positions and a set of five snapped nodes produce the same numbers and
    /// are not the same evidence. The Tool panel says which; the canvas does
    /// not, which is rule 4's disclosure boundary.
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

    /// ★ **Three free positions fit a circle**, which is the whole of O106.
    ///
    /// Asserted on the fit rather than on the count, because *"the points went
    /// in"* is not the claim — the claim is that a drawing with no vector
    /// geometry at all can still be measured.
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

    /// ★★ **The panel's removal and the canvas's removal are the same act.**
    ///
    /// `OPERATOR_REQUESTS.md` O107 asks for both routes, and the failure to
    /// guard against is two pick sets: a panel that removed from its own copy
    /// would leave the canvas drawing markers for points the fit no longer
    /// contains, which is worse than having no panel.
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

    /// ★ **The pick never reaches the selection.**
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

    /// ★ **The two endings author the same dimension from the same picks.**
    ///
    /// The property the one-commit-path design exists for, asserted the only
    /// way that means anything: run *both* endings over identical states and
    /// compare the actions they raise. Two arms that each built a
    /// `DimensionKind` would agree on the day they were written, drift on the
    /// first change to either, and the operator would have no way to see it — a
    /// circle fitted from the same points looks the same whichever code drew
    /// it.
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

    /// ★ **Both endings empty the pick set**, so a second Finish does not place
    /// the same circle twice.
    ///
    /// The failure without it is quiet and expensive: the operator presses
    /// Finish, sees the dimension land, presses it again out of habit or
    /// because they did not see the first, and gets two dimensions stacked
    /// exactly on top of each other — indistinguishable on screen and two undo
    /// steps to remove.
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

    /// ★ **A degenerate set commits nothing, from either ending.**
    ///
    /// `CircularPick::author` returns `None` for fewer than three usable points
    /// or a numerically singular set, and its docs say that is precisely when
    /// Finish must not be offered. Three points the operator clicked along a
    /// straight edge is the ordinary way to reach it — not a contrived one —
    /// and the honest response is to place nothing rather than to guess.
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

    /// ★ **`measure.finishable` is true exactly when pressing Finish would do
    /// something** — all five of the states that decide it.
    ///
    /// This is the condition behind a ribbon control, so each `false` row is a
    /// control that would otherwise be live and inert. The fourth row is the
    /// one that is easy to miss: putting the tool down does **not** discard the
    /// pick set (Escape's two rungs, `disarm_measure`'s own docs), so without
    /// the armed-tool check the ribbon would keep offering Finish for a set
    /// nothing is marking any more.
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

    /// ★ **Asking whether Finish is available does not manufacture state.**
    ///
    /// [`finishable`] runs on every frame, for every document, armed or not. If
    /// it went through `super::load` — which builds a `MeasureState` when there
    /// is none — the ribbon merely *drawing itself* would leave a measure state
    /// in memory for a tool nobody armed, and the next `store` would persist
    /// it.
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
