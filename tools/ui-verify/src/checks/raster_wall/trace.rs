//! Reading the raster-wall checks' evidence out of a `PDFCER_DIAG` trace.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/raster_wall/trace.md`.

use super::{
    BAD_RASTER_EVENT, BEYOND_EVENT, LEARNED_EVENT, RENDER_EVENT, REQUESTED_EVENT,
    UNAVAILABLE_EVENT, UNFILLABLE_EVENT,
};
use crate::checks::fit_places_the_view::CANVAS_EVENT;
use crate::geom::LRect;
use crate::trace::Trace;
use std::collections::BTreeSet;
/// Everything the canvas said about its last drawn frame.
#[derive(Clone, Debug)]
pub(super) struct CanvasState {
    /// The zoom factor, as the canvas used it (1.0 = 100 %).
    pub(super) zoom: f32,
    /// The acting page's index.
    pub(super) page: usize,
    /// How many pages the canvas drew this frame.
    pub(super) visible: usize,
    /// How many of those had a raster to draw.
    pub(super) drawn: usize,
    /// `single | continuous | facing | facing-continuous`.
    ///
    /// Read on every notch rather than once at the start: `visible >= 2` only
    /// means "a neighbour sheet is on screen" in a CONTINUOUS layout, so a run
    /// that silently left that layout would be asserting about a spread
    /// instead, which is a different state from the one O186 is about.
    pub(super) display: String,
    /// The acting page's drawn rect, in window-logical points.
    pub(super) rect: LRect,
}

/// The canvas's last drawn frame, or `None` if it has never drawn one.
pub(super) fn latest_canvas(trace: &Trace) -> Option<CanvasState> {
    let line = trace.events(CANVAS_EVENT).last()?;
    Some(CanvasState {
        zoom: line.get_f32("zoom")?,
        page: line.get_usize("page")?,
        visible: line.get_usize("visible")?,
        drawn: line.get_usize("drawn")?,
        display: line.get("display")?.to_owned(),
        rect: line.get_rect("rect")?,
    })
}

/// The reason the canvas is drawing **nothing**, if that is the verdict that
/// currently stands.
pub(super) fn unavailable_now(trace: &Trace) -> Option<(String, usize)> {
    let line = trace.events(UNAVAILABLE_EVENT).last()?;
    let drew_at = trace.events(CANVAS_EVENT).last().map_or(0, |l| l.lineno);
    if line.lineno > drew_at {
        let reason = line.get("reason").unwrap_or("unstated").to_owned();
        Some((reason, line.lineno))
    } else {
        None
    }
}

/// How many visible pages the strip last said it could not order, counting only
/// lines written after `after`.
pub(super) fn beyond_after(trace: &Trace, after: usize) -> Option<usize> {
    trace
        .events(BEYOND_EVENT)
        .filter(|l| l.lineno > after)
        .last()
        .and_then(|l| l.get_usize("pages"))
}

/// The **first** engine refusal of the operator's kind after `after`, as
/// `(page, whole line, line number)`.
pub(super) fn first_bad_raster_after(
    trace: &Trace,
    after: usize,
) -> Option<(Option<usize>, String, usize)> {
    trace
        .events(BAD_RASTER_EVENT)
        .find(|l| l.lineno > after)
        .map(|l| (l.get_usize("page"), l.raw.clone(), l.lineno))
}

/// Did the canvas report itself EMPTY between `after` and `before`, and on which
/// line?
pub(super) fn went_blank_between(trace: &Trace, after: usize, before: usize) -> Option<usize> {
    trace
        .events(UNAVAILABLE_EVENT)
        .find(|l| {
            l.lineno > after && l.lineno < before && l.get("reason") == Some("nothing-visible")
        })
        .map(|l| l.lineno)
}

/// The highest zoom at which the canvas said it had drawn **two or more** pages
/// after `after`.
pub(super) fn last_two_visible_zoom(trace: &Trace, after: usize) -> Option<f32> {
    trace
        .events(CANVAS_EVENT)
        .filter(|l| l.lineno > after)
        .filter(|l| l.get_usize("visible").is_some_and(|v| v >= 2))
        .last()
        .and_then(|l| l.get_f32("zoom"))
}

/// How many times the shell recorded the current page's raster order as
/// unfillable, and as fillable, after `after`: `(declined, placed)`.
pub(super) fn unfillable_counts(trace: &Trace, after: usize) -> (usize, usize) {
    let mut declined = 0usize;
    let mut placed = 0usize;
    for line in trace.events(UNFILLABLE_EVENT).filter(|l| l.lineno > after) {
        match line.get("fillable") {
            Some("false") => declined += 1,
            Some("true") => placed += 1,
            _ => {}
        }
    }
    (declined, placed)
}

/// Every failed render after `after`, as raw lines.
pub(super) fn failed_renders_after(trace: &Trace, after: usize) -> Vec<String> {
    trace
        .events(RENDER_EVENT)
        .filter(|l| l.lineno > after && l.get("outcome") == Some("failed"))
        .map(|l| l.raw.clone())
        .collect()
}

/// Every page the strip actually ordered a raster for after `after`.
pub(super) fn requested_pages_after(trace: &Trace, after: usize) -> BTreeSet<usize> {
    trace
        .events(REQUESTED_EVENT)
        .filter(|l| l.lineno > after)
        .filter_map(|l| l.get_usize("page"))
        .collect()
}

/// A learned raster ceiling that actually moved the zoom.
#[derive(Clone, Debug)]
pub(super) struct Learned {
    /// The page the ceiling was measured on.
    pub(super) page: Option<usize>,
    /// The raster scale the refusal arrived at.
    pub(super) scale: f32,
    /// The zoom the view was pulled back to.
    pub(super) to: f32,
}

/// The last ceiling learned after `after` that moved the view.
pub(super) fn last_learned_after(trace: &Trace, after: usize) -> Option<Learned> {
    trace
        .events(LEARNED_EVENT)
        .filter(|l| l.lineno > after && l.get("moved") == Some("true"))
        .filter_map(|l| {
            let to = l.get_f32("to")?;
            let scale = l.get_f32("scale")?;
            if !to.is_finite() || to <= 0.0 || !scale.is_finite() {
                return None;
            }
            Some(Learned {
                page: l.get_usize("page"),
                scale,
                to,
            })
        })
        .last()
}
