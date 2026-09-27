//! # `canvas::measure` — the dimensioning tools
//!
//! Phase 7. Placed under `canvas/` rather than at `tools/measure/`, following
//! the precedent this shell sets: [`crate::canvas::markup`] is the other
//! on-canvas
//! authoring tool, it lives here, and a measure tool is the same kind of thing
//! — a gesture that reads the page and raises an `Action`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/measure/mod.md`.

pub mod circular;
/// **What the tools draw**, and the one projection they draw through — the
/// pre-commit affordance, split from the picking under R2. Declared here rather
/// than beside the parent because its only caller reaches it through this
/// module's name; the re-export below is what keeps `measure::preview` and
/// `measure::page_to_screen` spelled the way every call site spells them.
mod draw;
pub(in crate::canvas) use draw::{Preview, SNAP_MARKER_PT, page_to_screen, preview};
/// The hover affordance: which line or entity a measuring click will take.
pub use pdfcer_gui_base::measurehover as hover;
/// One derivation of where a click would land and on what.
pub(in crate::canvas) mod resolve;

use resolve::snapped;
pub(in crate::canvas) use resolve::{Resolved, resolve_hover, snap_point};
/// The perimeter tool - click around a shape, one number for the whole way
/// round. A hybrid of the two tools beside it: point picks with snapping like
/// `Linear`, an open-ended gesture like `Circular`, plus an ending of its own
/// (click the first vertex to close the ring). Its header says why.
pub mod perimeter;

pub use pdfcer_gui_base::measure::kind::MeasureKind;
pub use pdfcer_gui_base::measure::{circpick, pick, scale};
pub mod state;

// ===========================================================================
// The canvas hosting
// ===========================================================================

use egui::Pos2;
use pdfcer_core::dimension::TwoLinePlacement;
use pdfcer_core::vector::Point;
use pdfcer_core::vector::linepick::pick_line_in_page;

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::snap;
use crate::canvas::target::CanvasTargetProvider;
use crate::viewer;

use state::{ClickOutcome, MeasureState};

/// Where the in-progress pick lives between frames.
///
/// `egui::Memory`, beside the armed tool and the gesture machine, and for the
/// same reason [`crate::canvas::tool`]'s header gives: this is **transient UI
/// state**, not document state. It must not sit on `OpenDoc`, because a
/// half-finished pick is not part of the document and a document saved
/// mid-gesture must not carry one.
// ui-text-exempt: an `egui::Id` source string, never displayed.
const MEASURE_MEMORY_KEY: &str = "pdfcer-measure-state";

/// Read the measure state, building one that already agrees with `kind` if
/// there is none.
fn load(ctx: &egui::Context, page_index: usize, kind: MeasureKind) -> MeasureState {
    let id = egui::Id::new(MEASURE_MEMORY_KEY);
    let mut st = ctx
        .data_mut(|d| d.get_temp::<MeasureState>(id))
        .unwrap_or_else(|| MeasureState::for_kind(page_index, kind));
    // Two synchronisations, and the order matters.
    //
    // The kind first, because `set_kind` is what knows which picks a kind
    // change invalidates. Then the page: a gesture begun on one sheet means
    // nothing on the next, and navigating away is not a reason to keep half a
    // dimension alive (ui-spec §1.3, carried across from the old shell).
    st.set_kind(kind);
    if st.page_index != page_index {
        st.page_index = page_index;
        st.clear_gesture();
    }
    st
}

/// Write the measure state back.
fn store(ctx: &egui::Context, st: MeasureState) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(MEASURE_MEMORY_KEY), st));
}

/// **Advance the snap cycle**, reporting whether there was anything to advance.
pub fn cycle_snap(ctx: &egui::Context) -> bool {
    let id = egui::Id::new(MEASURE_MEMORY_KEY);
    let Some(mut st) = ctx.data_mut(|d| d.get_temp::<MeasureState>(id)) else {
        return false;
    };
    // `usize::MAX` as the length: the advance is unbounded here on purpose —
    // see above — and `next_snap_index` is what decides the step.
    st.snap_cycle = snap::next_snap_index(st.snap_cycle, usize::MAX);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("measure-snap-cycle index={}", st.snap_cycle)
    });
    store(ctx, st);
    true
}

/// **Read the measure state without creating one.**
#[must_use]
pub fn active_group(ctx: &egui::Context) -> Option<pdfcer_core::dimension::GroupId> {
    read(ctx).map(|st| st.group)
}

/// **Choose the group the next ce dimension will join.**
pub fn set_active_group(ctx: &egui::Context, group: pdfcer_core::dimension::GroupId) {
    let mut st = read(ctx).unwrap_or_else(|| MeasureState::new(0));
    if st.group == group {
        return;
    }
    st.group = group;
    store(ctx, st);
}

pub fn read(ctx: &egui::Context) -> Option<MeasureState> {
    ctx.data_mut(|d| d.get_temp::<MeasureState>(egui::Id::new(MEASURE_MEMORY_KEY)))
}
/// The radius/diameter tool's two public entrances, re-exported so that every
/// caller outside `canvas/` names one module.
///
/// `app::dispatch` calls [`finish`] for the `measure.finish` command and
/// `app::conditions` calls [`finishable`] to publish `measure.finishable`. Both
/// live in [`circular`] because both are about the gesture's **ending**, which
/// is that tool's subject alone; they are named here because a dispatcher
/// reaching two modules deep for one verb is a dispatcher that knows more about
/// the canvas than it should.
pub use circular::{finish as finish_circular, finishable as finishable_circular};

/// **Is there a gesture waiting for `measure.finish`?** - the application state
/// behind the `measure.finishable` condition, over every open-ended tool.
#[must_use]
pub fn finishable(ctx: &egui::Context) -> bool {
    match crate::canvas::tool::selected(ctx).measure_kind() {
        Some(MeasureKind::Circular) => finishable_circular(ctx),
        Some(MeasureKind::Perimeter | MeasureKind::PathLength) => {
            read(ctx).is_some_and(|st| st.perimeter.author().is_some())
        }
        // Every other kind has a fixed arity and finishes itself. Spelled as a
        // catch-all rather than enumerated because the property being asserted
        // is "this tool ends on its own", which is the default and which a new
        // fixed-arity tool inherits correctly.
        _ => false,
    }
}

/// **The `measure.finish` command's whole effect**, reporting whether it did
/// anything.
pub fn finish(ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
    match crate::canvas::tool::selected(ctx).measure_kind() {
        Some(MeasureKind::Circular) => finish_circular(ctx, actions),
        Some(MeasureKind::Perimeter | MeasureKind::PathLength) => {
            let Some(mut st) = read(ctx) else {
                return false;
            };
            let page_index = st.page_index;
            if !perimeter::commit(&mut st, page_index, actions) {
                return false;
            }
            store(ctx, st);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                //
                // Which of the three endings asked for the commit - a fact no
                // screenshot can carry and the engine cannot know.
                format!("measure-finish via=command kind=perimeter page={page_index}")
            });
            true
        }
        _ => false,
    }
}

/// **Take the completed calibration line's measured length, once.**
pub fn take_completed_scale_line(ctx: &egui::Context) -> Option<f64> {
    // Read the stored state DIRECTLY rather than through `load`, deliberately.
    // `load` synchronises the kind and the page and will happily build a fresh
    // state when there is none — which is exactly wrong for a question that
    // must answer "no" when nothing has happened. Building state here would
    // also make merely *asking* create a gesture, which is the hazard
    // `MeasureState::set_kind`'s docs name.
    let id = egui::Id::new(MEASURE_MEMORY_KEY);
    let mut st = ctx.data_mut(|d| d.get_temp::<MeasureState>(id))?;
    let measured = st.scale.drawn_pdf_length?;
    st.scale.clear();
    store(ctx, st);
    Some(measured)
}

/// **Abandon a pick in progress**, reporting whether there was one.
pub fn abandon(ctx: &egui::Context) -> bool {
    let id = egui::Id::new(MEASURE_MEMORY_KEY);
    let Some(mut st) = ctx.data_mut(|d| d.get_temp::<MeasureState>(id)) else {
        return false;
    };
    if !st.gesture_in_progress() {
        return false;
    }
    st.clear_gesture();
    store(ctx, st);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        "canvas-escape outcome=AbandonedMeasurePick".to_owned()
    });
    true
}

/// **Everything one measure click is resolved against.**
pub(super) struct Pick<'a> {
    /// Where the in-progress pick is stored between frames.
    pub ctx: &'a egui::Context,
    /// The open document, for the page transform.
    pub doc: &'a OpenDoc,
    /// The page the click landed on.
    pub page_index: usize,
    /// Which measure tool is armed.
    pub kind: MeasureKind,
    /// The click, in canvas space.
    pub canvas_point: Pos2,
    /// **This was the second click of a double-click.**
    ///
    /// Carried rather than re-read from `egui` for the same reason every other
    /// field is: the whole of one click's meaning arrives in one value, so a
    /// future arm cannot resolve half of a click against this frame and half
    /// against the input state as it is by the time the arm runs.
    ///
    /// Only the circular tool reads it, and what it means there is *"the set is
    /// complete"* — see [`click`]'s own section on the two endings. The linear
    /// and two-line picks ignore it: their gestures end at a known click count,
    /// so a double-click on a linear tool is simply two picks, which is what an
    /// operator hurrying through point A and point B actually means.
    pub double: bool,
    /// The page's decomposed geometry, for the two-line pick and the circular
    /// object pick. `None` when no decomposition was built this frame.
    pub targets: Option<&'a dyn CanvasTargetProvider>,
    /// The screen↔page mapping, for the pick tolerance.
    pub map: &'a PageMapping,
}

/// **Take one click for the armed measure tool.**
pub(super) fn click(pick: Pick<'_>, actions: &mut Vec<Action>) {
    let Pick {
        ctx,
        doc,
        page_index,
        kind,
        canvas_point,
        double,
        targets,
        map,
    } = pick;
    let Some(page) = doc.current_page() else {
        return;
    };
    // The one conversion, through the renderer's own transform — never a
    // hand-written Y-flip. The rule `canvas::markup::endpoints` states.
    let Some(pdf) = viewer::canvas_to_pdf_space(canvas_point, page) else {
        return;
    };
    let picked = Point {
        x: f64::from(pdf.x),
        y: f64::from(pdf.y),
    };

    let mut st = load(ctx, page_index, kind);

    // The circular tool's DOUBLE-click is handled here, and its single
    // click is not.
    //
    // Taking the whole circular click ahead of the snap resolution would only
    // be right if the pick committed no point and merely toggled an OBJECT,
    // since the object under the pointer is the same object whether or not
    // there is a midpoint six pixels away. A circular pick is a **POINT** —
    // `pick::CircularPick`'s header carries the measurement, and
    // `OPERATOR_REQUESTS.md` O105 the report behind it — so it wants every part
    // of the machinery below: the snap, the raw fallback that makes a bitmap
    // measurable, and the derived-candidate confirm that keeps an inference
    // from being committed by the click that finds it.
    //
    // The double-click stays above it because it is not a pick at all: it ends
    // the gesture, and running an ending through a point resolution would be
    // asking where a click landed in order to throw the answer away.
    if kind == MeasureKind::Circular && double {
        let before = actions.len();
        circular::double_click(&mut st, page_index, actions);
        trace_pick(kind, &st, actions.len() > before);
        store(ctx, st);
        return;
    }

    // The pick is the SNAPPED point, not the pointer — and it is the point
    // the indicator was drawn over, because both read one `Resolved`.
    //
    // Falls back to the raw pick when the frame resolved nothing, which is the
    // honest answer: no decomposition, or snapping off, or nothing near.
    let alt_held = ctx.input(|i| i.modifiers.alt);
    let (p_raw, candidate) =
        match resolve_hover(ctx, doc, page_index, Some(canvas_point), targets, map, kind) {
            Some(r) => (r.at, r.candidate),
            None => snapped(&st, picked, alt_held, targets, page_index, map),
        };

    // The fuzzy-never-sneaky gate (rule 4). A **derived** candidate — a
    // centerline pdfcer inferred rather than one the file states — is not
    // committed by the click that finds it: the first click promotes it and a
    // second click on the same point confirms. That is the whole reason
    // `resolve_click` takes this flag, and passing a constant `false` (which is
    // what this call site did until the query above existed) quietly turned an
    // inference into a commitment.
    let is_derived = candidate.is_some_and(|c| snap::snap_commit_clicks(c.kind) > 1);
    let ClickOutcome::Commit(p) = st.resolve_click(p_raw, is_derived) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "measure-pick outcome=Promoted reason=derived-candidate-needs-confirm".to_owned()
        });
        store(ctx, st);
        return;
    };
    st.snap_cycle = 0;

    let before = actions.len();
    match kind {
        MeasureKind::Linear => {
            if let Some(authored) = st.linear.commit_point(p) {
                actions.push(Action::Dimension(DimensionAction::Commit {
                    page: page_index,
                    group: st.group,
                    kind: authored,
                    // Nothing to disclose: the linear tool measures between two
                    // points the operator picked, so its output is what they
                    // pointed at rather than a classification of it. See
                    // `DimensionAction::Commit`'s field.
                    disclosures: Vec::new(),
                }));
            }
        }
        // The perimeter tool takes its point HERE, after the snap
        // machinery above has run - unlike `Circular`, which is taken before
        // it. That is the whole reason this tool is a hybrid: its picks are
        // POINTS, so an operator tracing a building footprint gets the same
        // corner snapping they get from the linear tool, and none of that had
        // to be written twice.
        //
        // `canvas_point` and `page` are passed alongside the resolved point
        // because closing the ring is a CANVAS-space hit test against the first
        // vertex - same physical target size at every zoom. See
        // `perimeter::closes_the_ring`.
        MeasureKind::Perimeter | MeasureKind::PathLength => {
            perimeter::click(
                &mut st,
                perimeter::Click {
                    page_index,
                    picked: p,
                    canvas_point,
                    double,
                    page,
                    map,
                },
                actions,
            );
        }
        // The circular tool: one click is one point.
        //
        // It sits here, after the resolution, for the same reason the perimeter
        // tool does — its picks are POINTS, so it gets the drawing's own
        // geometry when there is any under the pointer and the operator's own
        // judgement when there is not. The double-click that ENDS the gesture
        // was taken before any of this ran; see the block above the resolution.
        //
        // `map.snap_tolerance()` is the removal radius, and it is the snap
        // catch radius rather than a number of its own: inside it a snapped
        // click would have landed on the very point being removed, so the two
        // readings of one gesture cannot disagree about which point is meant.
        MeasureKind::Circular => {
            circular::take_point(&mut st, p, candidate, map.snap_tolerance());
        }
        // The calibration pick. Two points measure a reference length; the
        // dialog then asks what that length IS on the real thing.
        //
        // It raises NO action on the picks themselves, which is the difference
        // from every arm above. `ScalePick::commit_point` returns `true` on the
        // click that completes the line, and the application notices
        // `dialog_open()` on the next frame and puts the Set-scale dialog up
        // with the measured length in it. Nothing is authored until the
        // operator says what the distance represents and accepts — the
        // fuzzy-never-sneaky rule, applied to the one gesture whose output is
        // a number every later dimension is multiplied by.
        MeasureKind::Scale => {
            if st.scale.commit_point(p) {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "measure-scale-line pdf_length={:.3}",
                        st.scale.drawn_pdf_length.unwrap_or(0.0)
                    )
                });
            }
        }
        MeasureKind::TwoLine => {
            // The pick is against the page's own geometry, so it needs the
            // decomposition — which is why `needs_targets` asks for one when a
            // measure tool is armed even in a mode that cannot select. Picking
            // a line is *reading* the page, not selecting it.
            let Some(model) = targets.and_then(|t| t.page_objects_model(page_index)) else {
                store(ctx, st);
                return;
            };
            if let Some(line) = pick_line_in_page(model, p, map.tolerance()) {
                // **The OPERATOR's threshold, not the default.**
                //
                // Reading `ParallelPolicy::default().epsilon_degrees` here
                // would make `Settings::parallel_epsilon_degrees` — persisted,
                // shown in the Settings window, edited, and reaching
                // `ScaleEntryFields` and the CLI — do **nothing** for the one
                // gesture it exists for. `pick.rs`'s own doc says the
                // value *"comes from `Settings::parallel_epsilon_degrees` and
                // is never a literal at the call site, so this tool and the CLI
                // cannot disagree about when two lines count as parallel"*, and
                // a hard-coded default IS a literal at the call site wearing a
                // constructor.
                //
                // `doc.settings` rather than the live `PdfcerApp` copy, because
                // that is the snapshot every other derived answer on this
                // document was computed under — the canvas has no route to the
                // application's settings and must not grow one.
                let epsilon = doc.settings.parallel_epsilon_degrees;
                st.two_lines.offer_line(line, epsilon);
                match st.two_lines.authoring(epsilon, TwoLinePlacement::default()) {
                    Some(Ok(authoring)) => {
                        // The disclosure travels WITH the action.
                        //
                        // Not recorded here through `record_note`: the apply
                        // phase runs after this frame and writes its own
                        // disclosure list to the same slot, so a note recorded
                        // now would be wiped by the commit it is about. The
                        // funnel's own mechanism is the closure's return value,
                        // and this is what it is for.
                        let disclosures = crate::text::measure::two_line_reading(
                            authoring.forced_parallel,
                            authoring.measured_angle_degrees,
                            authoring.apex_is_real() == Some(false),
                        )
                        .into_iter()
                        .collect();
                        actions.push(Action::Dimension(DimensionAction::Commit {
                            page: page_index,
                            group: st.group,
                            kind: authoring.kind,
                            disclosures,
                        }));
                        st.two_lines.clear();
                    }
                    // **The refusal, surfaced by name.** It was swallowed —
                    // `if let Some(Ok(..))` — so a collinear pair produced a
                    // second click that did nothing, silently, and an operator
                    // with no reason to suspect the geometry clicked again.
                    //
                    // `record_note` is right here where it was wrong above:
                    // NOTHING is being committed, so no apply phase will
                    // overwrite the slot, and the sentence has nowhere else to
                    // go. That is the same case `canvas::interact` records for
                    // a caret that cannot be placed, and it is stamped with the
                    // CURRENT epoch so it survives until the next real edit
                    // moves past it.
                    //
                    // The pair is cleared, because both picks are now known not
                    // to work together and leaving them would make the next
                    // click the third of a pair the operator thought was its
                    // first.
                    Some(Err(refusal)) => {
                        crate::app::actions::record_note(
                            doc.edit_epoch,
                            crate::text::measure::two_line_refused(refusal).to_owned(),
                        );
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed
                            format!("two-line-refused reason={refusal:?}")
                        });
                        st.two_lines.clear();
                    }
                    // One line picked, waiting for the second. Nothing to say.
                    None => {}
                }
            }
        }
    }

    trace_pick(kind, &st, actions.len() > before);
    store(ctx, st);
}

/// The one `measure-pick` trace line, emitted from both click paths.
fn trace_pick(kind: MeasureKind, st: &MeasureState, committed: bool) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            //
            // An armed measure tool and an un-armed one are the same
            // screenshot, and so are a first pick and a second — defect 8's
            // lesson. This line is how a harness proves a click became a pick.
            "measure-pick kind={kind:?} in_progress={} committed={committed}",
            st.gesture_in_progress(),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // The radius/diameter tool: the pick, and the two endings
    // =====================================================================

    /// A square inscribed in a circle of radius 10 centred at (30, 40) — a
    /// four-point set that fits exactly, so the residual is 0 and any drift in
    /// the authored geometry is visible rather than absorbed.
    fn circle_samples() -> Vec<Point> {
        vec![
            Point::new(40.0, 40.0),
            Point::new(30.0, 50.0),
            Point::new(20.0, 40.0),
            Point::new(30.0, 30.0),
        ]
    }

    /// **Changing tool discards the circular pick set.**
    #[test]
    fn arming_another_measure_tool_discards_the_circle_fit() {
        let ctx = egui::Context::default();
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        for at in circle_samples() {
            st.circular.toggle_point(at, pick::PickOrigin::Free, 0.1);
        }
        store(&ctx, st);

        let switched = load(&ctx, 0, MeasureKind::Linear);
        assert!(
            !switched.circular.in_progress(),
            "the fit set means nothing to the linear tool"
        );
    }

    /// **Leaving the page discards it too**, which is `load`'s other
    /// synchronisation and the one an operator reaches by paging through a
    /// drawing set with a tool still armed.
    #[test]
    fn navigating_to_another_page_discards_the_circle_fit() {
        let ctx = egui::Context::default();
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        for at in circle_samples() {
            st.circular.toggle_point(at, pick::PickOrigin::Free, 0.1);
        }
        store(&ctx, st);

        let moved = load(&ctx, 1, MeasureKind::Circular);
        assert_eq!(moved.page_index, 1);
        assert!(
            !moved.circular.in_progress(),
            "a fit assembled on sheet 1 means nothing on sheet 2"
        );
    }
}
