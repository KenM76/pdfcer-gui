//! # `canvas::measure::resolve` — one derivation of *"where would this click
//! land, and on what"*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/measure/resolve.md`.

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
#[derive(Debug, Clone, Copy)]
pub(in crate::canvas) struct Resolved {
    /// The point a click would commit — snapped, or the raw pointer.
    pub at: Point,
    /// Which candidate produced it, if any. `None` means the raw pointer, and
    /// no marker is drawn.
    pub candidate: Option<SnapCandidate>,
    /// What the pointer is OVER, which is a different question from where
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
pub(in crate::canvas) fn resolve_hover(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    canvas_pos: Option<Pos2>,
    targets: Option<&dyn CanvasTargetProvider>,
    map: &PageMapping,
    kind: MeasureKind,
) -> Option<Resolved> {
    // Traced at ENTRY, naming the gate that declines.
    //
    // An instrument below the five `?` early returns emits **nothing at all**
    // on a run where the pointer is demonstrably over the page, which tells a
    // reader only that the function did not finish — not which gate stopped it.
    // A trace that reports only the success path cannot diagnose a failure.
    //
    // The five gates are individually cheap and each means something different
    // about the application, so each is named.
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
    // NO state means a freshly armed tool, not a reason to decline. A `?`
    // here is what the operator reports as *"the measuring tools don't give me
    // any indication of what is being selected"*.
    //
    // `load` builds a default when memory is empty and **does not store it**;
    // only the click and gesture paths call `store`. So `MeasureState` does not
    // exist in `egui::Memory` until the operator has already picked once, and a
    // function that declines on its absence returns `None` on every frame
    // before that — taking the snap marker with it, so the whole hover
    // affordance switches on *after the first click of a gesture*. The
    // affordance has to appear while the operator is still deciding **where to
    // click first**; that is when it does its work.
    //
    // A read must not write, which is why this builds a value rather than
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
    // Resolved from the RAW pointer, not from the snapped point, and the
    // difference is the case the highlight was asked for.
    //
    // Snapping moves the query to the nearest target, and at an intersection
    // that target belongs to two lines equally. Asking "what is under the
    // snapped point" there answers "both, arbitrarily". Asking "what is under
    // the POINTER" answers "the one you are aiming at", which is the question
    // the operator's hand has already decided.
    let model = targets.and_then(|t| t.page_objects_model(page_index));
    let entity = model.and_then(|m| hover::resolve(m, raw, map.snap_tolerance()));
    // Traced whether or not anything was found, and that is the point.
    //
    // A hover affordance that draws nothing has three indistinguishable causes
    // from outside: the pointer is over blank paper, the decomposition is not
    // available, or the query found nothing within tolerance. A driven check
    // that sees no highlight cannot tell which.
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

/// The measure hover for this frame, resolved while the decomposition is
/// still borrowed.
pub(in crate::canvas) fn frame(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    kind: Option<MeasureKind>,
    canvas_pos: Option<Pos2>,
    targets: Option<&dyn CanvasTargetProvider>,
    map: &PageMapping,
) -> Option<Resolved> {
    // The snap hover, resolved HERE because this is the last line at which
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
            // CONVERTED — `resolve_hover` takes CANVAS space. Hand it
            // `screen_pos` raw and the click still lands correctly while the
            // marker sits away from the pointer by the scroll origin over the
            // zoom: *"the crosshairs click the right place under them, but the
            // preview … is offset"*.
            //
            // Every other reader of `screen_pos` on this path writes this same
            // conversion (the gesture's own `pos`, the hit test, the trace),
            // which is what would make an odd one out invisible — it names the
            // same variable as the rest.
            canvas_pos, targets, map,
            // The armed kind, which this closure already had and was
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
    //! The snap resolution's own assertions, beside `snapped` and
    //! `snap_point`.

    use super::*;
    use crate::canvas::measure::{MeasureKind, MeasureState};
    use crate::canvas::targetstub::StubTargets;

    /// **Snapping off means the raw pointer, unchanged.**
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

    /// **A caller with no stored `MeasureState` still snaps.**
    #[test]
    fn a_caller_with_no_stored_state_gets_the_shipped_defaults() {
        let fresh = MeasureState::new(3);
        assert!(
            fresh.snap_master,
            "the fallback `snap_point` builds must arrive with snapping ON, or a vertex drag would silently never snap in a session where no measure tool was ever armed"
        );
        assert_eq!(fresh.snap_cycle, 0, "and with no Tab cycle carried in");
    }

    /// **Alt refuses the snap for one pick**, which is what makes a generous
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

    /// **The snap radius is wider than the selection radius, and stays so at
    /// every zoom.**
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
