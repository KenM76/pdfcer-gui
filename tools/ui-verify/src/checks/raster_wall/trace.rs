//! Reading the raster-wall checks' evidence out of a `PDFCER_DIAG` trace.
//!
//! ===========================================================================
//! WHY THIS IS ITS OWN FILE
//! ===========================================================================
//!
//! Split out of `raster_wall.rs` on 2026-09-12, when that file reached 1,614
//! physical lines against standing rule R2's limit of 1,500. The response the
//! size gate asks for is to find the seam, not to shrink the prose, and the
//! seam here is obvious: everything in this file answers *"what did the
//! application say?"* and nothing in it drives, waits, or decides a verdict.
//!
//! That division is worth more than the line count. A trace reader is pure —
//! one `&Trace` in, one value out — so every function here is readable and
//! arguable without knowing anything about wheel notches, seams or zoom
//! ceilings. The two `part_*` functions in the parent are the opposite: they
//! are a sequence of actions whose order is the whole content. Mixing the two
//! kinds in one file is what made the original hard to navigate.
//!
//! ===========================================================================
//! THE ONE PROPERTY EVERY FUNCTION HERE SHARES, AND WHY IT MATTERS
//! ===========================================================================
//!
//! **Every reader that can be fooled by a stale line takes or returns a line
//! number.** `crate::diag::trace_changed` de-duplicates per slot, so a line
//! written once stays in the file for ever and `events(X).last()` can return a
//! fossil from before the state changed. Two separate defects in this project
//! came from exactly that — a healthy `canvas` line standing over a blank
//! screen, and a dock tab's last caption outliving the tab.
//!
//! So the contract of this module is:
//!
//!   * A reader that asks *"is this true NOW?"* compares line numbers, never
//!     event names alone — see [`unavailable_now`].
//!   * A reader that asks *"did this happen since I acted?"* takes an `after`
//!     line number from the caller's mark.
//!   * A reader whose answer will be ATTRIBUTED returns the line number it
//!     found, so the caller can ask what preceded it — see
//!     [`first_bad_raster_after`] and [`went_blank_between`], which together
//!     tell three different defects apart that all produce one sentence.
//!
//! ===========================================================================
//! WHAT IS DELIBERATELY NOT HERE
//! ===========================================================================
//!
//! No geometry (that is `raster_wall::park`), no driving, no verdict. Nothing
//! in this file may call `Session`, because a reader that can act is a reader
//! whose answer depends on when it was called, and then a report cannot be
//! reproduced from the trace file alone.

use super::{
    BAD_RASTER_EVENT, BEYOND_EVENT, LEARNED_EVENT, RENDER_EVENT, REQUESTED_EVENT,
    UNAVAILABLE_EVENT, UNFILLABLE_EVENT,
};
use crate::checks::fit_places_the_view::CANVAS_EVENT;
use crate::geom::LRect;
use crate::trace::Trace;
use std::collections::BTreeSet;
/// Everything the canvas said about its last drawn frame.
///
/// Read as one struct from one trace line on purpose. The alternative — a
/// helper per field — re-reads the trace per question and can answer two
/// questions from two different frames, which is how a check comes to report a
/// zoom from after a clamp against a visible count from before it.
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
///
/// ★ `canvas` and `canvas-unavailable` share one trace slot, so the channel
/// emits whichever changed and the other's last line stays in the file for
/// ever. Asking `events("canvas").last()` alone would read a fossil from before
/// the canvas went empty and report a healthy frame over a blank screen. The
/// comparison is by line number, which is the same property
/// `driving::declared` relies on for retired regions.
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
///
/// First rather than last: the first one is the one whose cause is still
/// legible, because `absorb_render` learns a ceiling from it and everything
/// after is a consequence of that.
///
/// ★ The line number is returned because the attribution depends on what came
/// BEFORE the refusal — see [`went_blank_between`]. Three different defects can
/// produce this one line and the only way to tell them apart is the order of the
/// trace.
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
///
/// ★★★ This is the discriminator between O186's third route and its first.
/// `canvas-unavailable reason=nothing-visible` says no part of any page was on
/// screen, which is upstream of everything: with nothing visible there is no
/// region, with no region the request is the whole sheet, and above the pixmap
/// ceiling a whole-sheet request is a refusal. A check that read only the
/// refusal would name `fill_strip` — the wrong function — and the next reader
/// would spend the investigation there.
///
/// Restricted to `reason=nothing-visible` on purpose. The other reasons the
/// canvas declines to draw (no document, a collapsed dock) are not this, and
/// matching the event name alone would make the discriminator fire on them.
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
///
/// Read only when part A's window turns out to be empty, to say *when* the
/// neighbour left rather than only that it was not there at the end. A climb
/// whose answer is `None` never had a neighbour at all, which is a different
/// fault from one that had it and lost it.
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
///
/// ⚠ These are **transitions, not frames.** `diag::trace_changed` writes only
/// when the formatted line changes, so a climb that declined for two hundred
/// consecutive frames counts 1. That is the right granularity for reading a
/// regime change and the wrong one for reading a duration, and a report that
/// called it a frame count would be overstating by two orders of magnitude.
///
/// Both halves are wanted. A count of declines alone cannot be distinguished
/// from a guard that never ran at all, which is why the guard traces both
/// transitions rather than only the interesting one.
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
///
/// `moved=false` lines are ignored deliberately: the shell traces its decision
/// either way, and a ceiling learned at a zoom the view was already below
/// changes nothing the operator can see — so it is not the event part B is
/// waiting for. A non-finite `to` is treated as no event at all rather than
/// compared against, because every comparison with `NaN` is false and a
/// silently-false assertion is worse than an absent one.
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
