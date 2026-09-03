//! # `canvas::measure::resolve` — one derivation of *"where would this click
//! land, and on what"*
//!
//! Split out of [`super`] when R2's 1,500-line limit was reached, and the seam
//! is real rather than convenient: everything here answers a question **about
//! the pointer**, and everything left behind is about the tools and their
//! state.
//!
//! ## ★★ Why this is one function and not two
//!
//! [`Resolved`] carries the whole answer — the snapped point, the candidate
//! that produced it, and the entity under the pointer — because the indicator
//! the operator aims at and the point the next click commits must be *the same
//! value*, not two derivations that agree by construction.
//!
//! The cost of two is on the record. The marker was once resolved against a raw
//! screen position while the click used a converted canvas one, so they
//! disagreed by the scroll origin over the zoom — **zero at the top-left of an
//! unscrolled page at 100 %**, growing from there. It survived four days,
//! looked like *"sometimes it is fine"*, and no unit test could see it because
//! both functions were individually correct.
//!
//! ## ★ Neither reader may require stored state
//!
//! `MeasureState` is not written to `egui::Memory` until the operator has
//! clicked once — [`super::load`] builds a default and only the click paths
//! store it. Both this function and the paint site used to bail on that, which
//! meant the entire hover affordance switched on **after the first pick of a
//! gesture**: the snap marker and the entity highlight were dead in exactly the
//! moment they do their work.
//!
//! That was reported as *"the measuring tools don't give me any indication of
//! what is being selected"*, and it is why both sites now fall back to a value
//! built from the armed kind rather than declining.
//!
//! A read must not write. Persisting from here would make moving the pointer an
//! edit to shared state, and arming is the only thing that should decide what
//! is armed.

use egui::Pos2;
use pdfcer_core::vector::Point;
use pdfcer_core::vector::snap::{SnapCandidate, SnapConfig, snap_candidates};

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::target::CanvasTargetProvider;
use crate::canvas::viewer;

use super::{MEASURE_MEMORY_KEY, MeasureKind, MeasureState, hover, snap};

/// **Resolve a raw pointer position to the point the pick will actually
/// commit**, and say which candidate it came from.
///
/// This is the call the salvaged [`crate::canvas::snap`] primitives were
/// waiting for. Until it existed every pick was the raw pointer position,
/// which on a CAD sheet is the difference between a dimension that measures a
/// line and one that measures *near* a line — and the second is worse than no
/// dimension, because it is wrong by an amount nobody can see.
///
/// # The four gates, in order, and what each is for
///
/// 1. **The master toggle and the Alt override**, through
///    [`snap::snap_query_enabled`]. Alt is the operator saying *"not this
///    time"* — it is what makes the offer refusable, and it is why the catch
///    radius can afford to be generous ([`PageMapping::snap_tolerance`]).
/// 2. **A decomposition must exist.** No model, no candidates, raw point. This
///    is a real case rather than a defensive one: the model is built only when
///    something needs it, and a measure click is one of the things that asks.
/// 3. **The query**, `pdfcer_core::vector::snap_candidates` — the engine's, not
///    ours. `SnapConfig::new` leaves intersections off and axes on, which is
///    the shipped default; the grid is `None` because the canvas grid is a
///    *view* aid drawn in page space and snapping to it would be snapping to
///    something the document does not contain.
/// 4. **The Tab cycle**, through [`snap::active_snap_candidate`], which is what
///    lets the operator choose between an endpoint and a midpoint that are
///    within a few pixels of each other.
///
/// Returns the raw point unchanged when any gate declines, with `None` for the
/// candidate — so a caller never has to distinguish "snapping is off" from
/// "nothing was near", because neither changes what it does next.
pub(in crate::canvas) fn snapped(
    st: &MeasureState,
    raw: Point,
    alt_held: bool,
    targets: Option<&dyn CanvasTargetProvider>,
    page_index: usize,
    map: &PageMapping,
) -> (Point, Option<SnapCandidate>) {
    if !snap::snap_query_enabled(st.snap_master, alt_held) {
        return (raw, None);
    }
    let Some(model) = targets.and_then(|t| t.page_objects_model(page_index)) else {
        return (raw, None);
    };
    let config = SnapConfig::new(map.snap_tolerance());
    let candidates = snap_candidates(raw, &config, model);
    match snap::active_snap_candidate(&candidates, st.snap_cycle) {
        Some(c) => (c.point, Some(c)),
        None => (raw, None),
    }
}

/// **Snap a point the way a measure pick would**, for a caller that is not a
/// measure tool.
///
/// ★★ Added 2026-08-20 for the perimeter's vertex drag, and the reason it is
/// here rather than a second snap in `canvas::dimdrag` is the rule this project
/// has paid for twice: **a predicate with two claimants must exist exactly
/// once.** `text_edit_focused()` cost the Delete key and then the space bar
/// because two places each had their own idea of one question. *"Where would
/// this land if it snapped"* is one question, and the operator's answer to it —
/// the master toggle, the Alt override, the tolerance, the Tab cycle — is one
/// set of settings. A drag that snapped by its own rules would honour a
/// different "Snap to content" switch from the tool beside it.
///
/// # The gap this closes, in the operator's terms
///
/// `ui-conventions/drag-moves.md` D6, found by the 2026-08-20 sweep:
///
/// > **A vertex drag does not snap**, while the tool that placed that vertex
/// > does — so you can pick a corner onto geometry and then be unable to put it
/// > back.
///
/// That is the worst shape a missing convention can take: the tool teaches the
/// operator that corners land exactly on lines, and then takes it away for the
/// one gesture whose entire purpose is correcting a corner that landed wrong.
///
/// # ★ It builds a `MeasureState` rather than requiring one
///
/// [`super::load`] persists nothing and [`super::read`] answers `None` until the operator has
/// clicked a measure tool at least once. A vertex drag can happen in a session
/// where no measure tool was ever armed, so requiring stored state would mean
/// *snapping switches on only after you have used a different tool* — the same
/// defect [`resolve_hover`] records at length, which shipped and was reported.
///
/// The fallback carries `snap_master: true`, the shipped default, and a
/// `snap_cycle` of 0. Tab-cycling between two nearby candidates is therefore
/// **not** offered during a vertex drag; that is a decision rather than an
/// oversight, because Tab during a drag is not a gesture any program in the
/// class defines.
pub(in crate::canvas) fn snap_point(
    ctx: &egui::Context,
    page_index: usize,
    raw: Point,
    alt_held: bool,
    targets: Option<&dyn crate::canvas::target::CanvasTargetProvider>,
    map: &PageMapping,
) -> (Point, Option<pdfcer_core::vector::snap::SnapCandidate>) {
    let st = super::read(ctx)
        .filter(|s| s.page_index == page_index)
        .unwrap_or_else(|| MeasureState::new(page_index));
    snapped(&st, raw, alt_held, targets, page_index, map)
}

/// **Where the pointer would pick, resolved once for the frame.**
///
/// ★ The indicator and the click read *this same value*, which is the whole
/// reason it exists as a type rather than as two calls to [`snapped`]. A
/// preview that re-ran the query would be a second derivation of the same
/// answer, and the two would agree right up until they did not — the operator
/// aiming at a marker drawn over an endpoint and committing a point somewhere
/// else. `pdfce_FeatureRequests/README.md` rule 4 is explicit that a
/// pre-commit affordance must describe *what is about to happen*; one
/// derivation is how that stays true rather than being maintained.
#[derive(Debug, Clone, Copy)]
pub(in crate::canvas) struct Resolved {
    /// The point a click would commit — snapped, or the raw pointer.
    pub at: Point,
    /// Which candidate produced it, if any. `None` means the raw pointer, and
    /// no marker is drawn.
    pub candidate: Option<SnapCandidate>,
    /// ★★ What the pointer is OVER, which is a different question from where
    /// the click will land.
    ///
    /// The operator's report this answers: *"the measuring tools don't give me
    /// any indication of what is being selected. I should be able to hover over
    /// a line or node and have it indicate that is what will be selected."*
    ///
    /// [`Self::candidate`] answers *"your click will land exactly here"*. This
    /// answers *"and it will be taken from THIS line"*, which on a drawing made
    /// of near-identical strokes is the half that decides whether the
    /// measurement is the one they meant. See [`hover`].
    ///
    /// It rides here rather than being queried at paint time for the reason
    /// this whole type exists: `PageObjects` is borrowed only during the
    /// resolve pass, and two derivations of one answer agree right up until
    /// they do not.
    pub entity: Option<hover::Entity>,
}

/// Resolve the pointer for this frame, while the decomposition is still
/// borrowed.
///
/// Called from `canvas::interact` **before** it drops the provider, which is
/// the constraint that shaped this API: the draw happens after the drop, so the
/// query cannot happen there.
/// ★ `canvas_pos` is **CANVAS** space, not screen space, and the name says so
/// because getting it wrong is invisible.
///
/// # The defect this parameter was renamed after
///
/// It was called `pointer`, and `canvas::interact` handed it `screen_pos`
/// **unconverted** while every sibling call on the same value wrote
/// `map.to_page(p)`. The operator found it, 2026-08-18:
///
/// > *"when I click on measure it on the drawing the crosshairs click the
/// > right place under them, but the preview of what is being selected is
/// > offset from the crosshairs instead of being underneath them."*
///
/// That is exactly what the bug produces, and the shape is worth keeping. The
/// **click** path (`click`, via `Pick::canvas_point`) was handed a properly
/// converted point and committed the right place; only the **preview** read
/// the raw screen position, so the snap candidate was resolved near a
/// different part of the page. A wrong answer next to a right one, with
/// nothing to say which was which — and the offset is the scroll origin over
/// the zoom, so it is zero at the top-left of an unscrolled page at 100 % and
/// grows from there. It would look like "sometimes it is fine".
///
/// `Pos2` cannot carry its own space, so the only defences available are the
/// parameter's **name** and this paragraph. `canvas::mapping`'s header is the
/// standing argument for why these conversions live in one place; this is the
/// case that proves the argument was about the call sites too.
pub(in crate::canvas) fn resolve_hover(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    canvas_pos: Option<Pos2>,
    targets: Option<&dyn CanvasTargetProvider>,
    map: &PageMapping,
    kind: MeasureKind,
) -> Option<Resolved> {
    // ★★ Traced at ENTRY, naming the gate that declines.
    //
    // The first version of this instrument sat at the bottom, after five `?`
    // early returns, and emitted **nothing at all** on a run where the pointer
    // was demonstrably over the page — which told a reader only that the
    // function had not finished, not which gate stopped it. An instrument that
    // reports only the success path cannot diagnose a failure, and this suite
    // has spent a day on the cost of a confident wrong diagnosis.
    //
    // The five gates are individually cheap and each of them means something
    // different about the application, so each is named.
    let st_present = ctx
        .data_mut(|d| d.get_temp::<MeasureState>(egui::Id::new(MEASURE_MEMORY_KEY)))
        .map(|s| s.page_index);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "measure-hover-gate canvas_pos={} page={} state_page={:?} want_page={page_index}",
            u8::from(canvas_pos.is_some()),
            u8::from(doc.current_page().is_some()),
            st_present
        )
    });
    let (pointer, page) = (canvas_pos?, doc.current_page()?);
    let pdf = viewer::canvas_to_pdf_space(pointer, page)?;
    let raw = Point {
        x: f64::from(pdf.x),
        y: f64::from(pdf.y),
    };
    // ★★ NO state means a freshly armed tool, not a reason to decline —
    // and the `?` that used to be here is why the operator reported that *"the
    // measuring tools don't give me any indication of what is being selected"*.
    //
    // [`load`] builds a default when memory is empty and **does not store it**;
    // only the click and gesture paths call [`store`]. So `MeasureState` does
    // not exist in `egui::Memory` until the operator has already picked once,
    // and this function returned `None` on every frame before that — taking the
    // snap marker with it.
    //
    // The effect was that the whole hover affordance switched on *after the
    // first click of a gesture*. The paint site's own comment says what that is
    // worth, and it had been describing an intention rather than the behaviour:
    //
    // > It has to appear while the operator is still deciding *where to click
    // > first* — that is when it does its work. Gating it on a gesture already
    // > being in progress would show it only after the first pick, i.e.
    // > everywhere except the place it is needed most.
    //
    // ★ A read must not write, which is why this builds a value rather than
    // calling `load` and storing it. `resolve_hover` runs on every frame the
    // pointer moves; persisting from here would make a hover an edit to shared
    // state, and the arming path is the only thing that should decide what is
    // armed.
    //
    // `kind` comes from the caller's `active_tool.measure_kind()`, which it
    // already computed and was discarding — so the armed tool is known here
    // without asking memory anything.
    let st = ctx
        .data_mut(|d| d.get_temp::<MeasureState>(egui::Id::new(MEASURE_MEMORY_KEY)))
        .filter(|s| s.page_index == page_index)
        .unwrap_or_else(|| MeasureState::for_kind(page_index, kind));
    let alt_held = ctx.input(|i| i.modifiers.alt);
    let (at, candidate) = snapped(&st, raw, alt_held, targets, page_index, map);
    // ★ Resolved from the RAW pointer, not from the snapped point, and the
    // difference is the case the highlight was asked for.
    //
    // Snapping moves the query to the nearest target, and at an intersection
    // that target belongs to two lines equally. Asking "what is under the
    // snapped point" there answers "both, arbitrarily". Asking "what is under
    // the POINTER" answers "the one you are aiming at", which is the question
    // the operator's hand has already decided.
    let model = targets.and_then(|t| t.page_objects_model(page_index));
    let entity = model.and_then(|m| hover::resolve(m, raw, map.snap_tolerance()));
    // ★★ Traced whether or not anything was found, and that is the point.
    //
    // A hover affordance that draws nothing has three indistinguishable causes
    // from outside: the pointer is over blank paper, the decomposition is not
    // available, or the query found nothing within tolerance. A driven check
    // that sees no highlight cannot tell which, and this suite has spent a day
    // learning what a confident wrong diagnosis costs.
    //
    // So the line is emitted on every resolved frame with the three facts that
    // separate the cases: whether there was a model at all, what the tolerance
    // was, and what was found.
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "measure-hover model={} tol={:.2} entity={} snap={}",
            u8::from(model.is_some()),
            map.snap_tolerance(),
            u8::from(entity.is_some()),
            u8::from(candidate.is_some())
        )
    });
    Some(Resolved {
        at,
        candidate,
        entity,
    })
}

/// ★★ The measure hover for this frame, resolved while the decomposition is
/// still borrowed.
///
/// Lifted out of `canvas::interact` when R2 asked for four lines back. The
/// constraint that shapes it: the page decomposition is borrowed **only here**
/// and dropped before anything is painted, so this query cannot happen at paint
/// time — and it must not be repeated. See this module's header for what two
/// derivations of one answer cost when the marker and the click disagreed by
/// the scroll origin over the zoom.
///
/// ★ **It returned a second value until 2026-09-03** — the circular pick set's
/// object outlines — and the pair travelled together because both needed the
/// borrow. The pick set is a list of POINTS now (`pick::CircularPick`), which
/// the preview reads straight out of `egui::Memory` and projects itself, so it
/// needs no decomposition and no channel through three call sites. The whole
/// borrow hazard went with it.
///
/// `kind` is `None` for every non-measure tool, and then this costs one
/// `Option` check and runs no query at all: panning a 129,758-object drawing
/// decomposes nothing.
pub(in crate::canvas) fn frame(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    kind: Option<MeasureKind>,
    canvas_pos: Option<Pos2>,
    targets: Option<&dyn CanvasTargetProvider>,
    map: &PageMapping,
) -> Option<Resolved> {
    // ★ The snap hover, resolved HERE because this is the last line at which
    // the decomposition is still borrowed.
    //
    // The indicator is drawn in the draw section far below, after the `drop`,
    // so the query cannot happen there — and it must not happen twice. One
    // `Resolved` is what makes the marker the operator aims at and the point
    // the next click commits provably the same value rather than two
    // derivations that agree by construction until they do not. See
    // `measure::Resolved`.
    //
    // `None` for every tool but a measure tool, so an un-armed canvas pays one
    // `Option` check and runs no query.
    kind.and_then(|kind| {
        resolve_hover(
            ctx, doc, page_index,
            // ★ CONVERTED — `resolve_hover` takes CANVAS space, and this line
            // handed it `screen_pos` raw until 2026-08-18. The click path
            // converted and the preview did not, so the marker sat away from
            // the pointer by the scroll origin over the zoom while the click
            // landed correctly. The operator reported it as *"the crosshairs
            // click the right place under them, but the preview … is offset"*.
            //
            // Every other reader of `screen_pos` on this path already wrote
            // this conversion (the gesture's own `pos`, the hit test, the
            // trace), which is what made the odd one out invisible: it looked
            // like the others because it named the same variable.
            canvas_pos, targets, map,
            // ★ The armed kind, which this closure already had and was
            // throwing away. `resolve_hover` needs it so a freshly armed tool
            // — one that has never been clicked, which is every tool at the
            // moment the operator most needs to see what it would pick — can
            // resolve a hover without a `MeasureState` in memory.
            kind,
        )
    })
}

#[cfg(test)]
mod tests {
    //! The snap resolution's own tests, moved here from `measure/mod.rs` on
    //! 2026-08-20 under R2 — with `snapped` and `snap_point`, which is where
    //! they always belonged.

    use super::*;
    use crate::canvas::measure::{MeasureKind, MeasureState};
    use crate::canvas::target::StubTargets;

    /// ★ **Snapping off means the raw pointer, unchanged.**
    ///
    /// The master toggle is the operator's, and a tool that snapped anyway
    /// would be applying an inference they had switched off — which is rule 4's
    /// definition of sneaky rather than fuzzy.
    #[test]
    fn the_master_toggle_off_returns_the_raw_point() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Linear);
        st.snap_master = false;
        let raw = Point { x: 10.5, y: 20.25 };
        let targets = StubTargets::default();
        let map = crate::canvas::mapping::PageMapping::new(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
            (200.0, 300.0),
            1.0,
        );
        let (at, candidate) = snapped(&st, raw, false, Some(&targets), 0, &map);
        assert_eq!(at, raw, "the pick is where the pointer was");
        assert!(candidate.is_none(), "nothing to draw an indicator for");
    }

    /// ★★ **A caller with no stored `MeasureState` still snaps.**
    ///
    /// The vertex drag's case, and the reason [`snap_point`] exists rather than
    /// the caller doing `read(ctx)?`. [`load`] persists nothing and [`read`]
    /// answers `None` until a measure tool has been *clicked*, so a `?` here
    /// would mean **snapping switches on only after you have used a different
    /// tool** — which is not a smaller version of the feature; it is the exact
    /// defect `resolve_hover`'s own header records, which shipped and was
    /// reported as *"the measuring tools don't give me any indication of what
    /// is being selected"*.
    ///
    /// Asserted through the master toggle rather than through a candidate,
    /// because what is being proved is *which state was used*: a build that
    /// declined on empty memory answers the raw point with `None`, and so does
    /// a build that found nothing nearby. The distinguishable fact is that the
    /// fallback carries `snap_master: true`.
    #[test]
    fn a_caller_with_no_stored_state_gets_the_shipped_defaults() {
        let fresh = MeasureState::new(3);
        assert!(
            fresh.snap_master,
            "the fallback `snap_point` builds must arrive with snapping ON, or a vertex drag would silently never snap in a session where no measure tool was ever armed"
        );
        assert_eq!(fresh.snap_cycle, 0, "and with no Tab cycle carried in");
    }

    /// ★ **Alt refuses the snap for one pick**, which is what makes a generous
    /// catch radius affordable — see `PageMapping::snap_tolerance`.
    #[test]
    fn alt_overrides_an_enabled_master_toggle() {
        let st = MeasureState::for_kind(0, MeasureKind::Linear);
        assert!(st.snap_master, "the toggle defaults on");
        let raw = Point { x: 1.0, y: 2.0 };
        let targets = StubTargets::default();
        let map = crate::canvas::mapping::PageMapping::new(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
            (200.0, 300.0),
            1.0,
        );
        let (at, candidate) = snapped(&st, raw, true, Some(&targets), 0, &map);
        assert_eq!(at, raw);
        assert!(candidate.is_none());
    }

    /// With no decomposition there is nothing to snap to, and that is a real
    /// case rather than a defensive one: the model is built only when something
    /// asks, and this is one of the things that asks.
    #[test]
    fn no_decomposition_means_no_snap_and_no_panic() {
        let st = MeasureState::for_kind(0, MeasureKind::Linear);
        let raw = Point { x: 3.0, y: 4.0 };
        let map = crate::canvas::mapping::PageMapping::new(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
            (200.0, 300.0),
            1.0,
        );
        let (at, candidate) = snapped(&st, raw, false, None, 0, &map);
        assert_eq!(at, raw);
        assert!(candidate.is_none());
    }

    /// ★ **The snap radius is wider than the selection radius, and stays so at
    /// every zoom.**
    ///
    /// Both are screen-pixel constants divided by the same zoom, so the
    /// relation is scale-invariant — asserting it at one zoom would be the
    /// *"relation rather than magnitude"* trap `HANDOFF.md` §2 names, so the
    /// magnitudes are checked too.
    #[test]
    fn the_snap_radius_is_wider_than_the_selection_radius_at_every_zoom() {
        for zoom in [0.05_f32, 0.5, 1.0, 4.0, 32.0] {
            let map = crate::canvas::mapping::PageMapping::new(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
                (200.0, 300.0),
                zoom,
            );
            let snap = map.snap_tolerance();
            let select = map.tolerance();
            assert!(
                snap > select,
                "zoom={zoom}: snap {snap} must out-reach selection {select}"
            );
            assert!(
                snap.is_finite() && snap > 0.0,
                "zoom={zoom}: a catch radius of {snap} would snap to everything or nothing"
            );
        }
    }
}
