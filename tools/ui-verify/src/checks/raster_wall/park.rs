//! Getting the pointer onto the seam between two pages, and knowing how much
//! room is left below it.
//!
//! ===========================================================================
//! WHY THIS IS ITS OWN FILE
//! ===========================================================================
//!
//!
//!   * [`seam_y`] says where the gap is this frame.
//!   * [`window_closes_at`] says how long the neighbour will survive there.
//!   * [`aim_at`] turns a window-logical point into something the OS can be
//!     told to click.
//!   * [`park_on_the_seam`] scrolls until the gap is in the band, reports the
//!     window it found, and REFUSES if that window is empty.
//!
//! ===========================================================================
//! THE LESSON THIS FILE IS THE RECORD OF
//! ===========================================================================
//!
//! ★★★ **A check's measuring window can be bounded below by the application
//! and above by the harness's own geometry, and then it is flaky until somebody
//! does the arithmetic.** Part A's window was bounded below at zoom 20.7 (where
//! the neighbour becomes unorderable) and above at 18.9 (where the growing gap
//! pushed it off a 770-pt canvas). The window was EMPTY — yet the check did not
//! fail. It climbed its whole budget, observed no refusal, and SKIPPED with a
//! message saying the state had never been entered. One earlier run had
//! squeezed inside the window and passed, which is precisely the shape of a
//! check that has stopped running without anyone noticing.
//!
//! The cure is in two parts, and both live in the parent's constants: a window
//! tall enough to leave room (`VIEWPORT`), and a seam band narrow enough to put
//! the seam where that room is (`SEAM_BAND`). What lives HERE is the third
//! part — [`park_on_the_seam`] computes the window before it spends a notch,
//! prints it into the report either way, and returns a SKIP carrying the
//! numbers when it is empty. A gate that reports *"the window was empty"* is a
//! finding. A gate that reports *"the state was never entered"* is a lie with
//! the grammar of a measurement.
//!
//! ===========================================================================
//! WHAT IS DELIBERATELY NOT HERE
//! ===========================================================================
//!
//! No verdict. [`park_on_the_seam`] returns `Err` for every refusal, which the
//! parent turns into a SKIP, never a FAIL — nothing in this file has measured
//! anything about O186, so nothing in it is entitled to fail a build. And no
//! trace parsing beyond calling `raster_wall::trace`: the geometry reads the
//! canvas's own published rect, and a check that assembles screen coordinates
//! from a window origin instead stops hitting anything the first time a dock
//! width changes.

use super::trace::{CanvasState, latest_canvas, unavailable_now};
use super::{
    BOTTOM_DEAD_BAND_PT, CLIMB_NOTCHES_A, EDGE_MARGIN_PT, NEIGHBOUR_UNORDERABLE_ZOOM, PAGE_COUNT,
    ROW_GAP_PT, SEAM_BAND, SEAM_SEARCH_NOTCHES, UNAVAILABLE_EVENT, WINDOW_MARGIN,
};
use crate::checks::CheckReport;
use crate::checks::fit_places_the_view::CANVAS_EVENT;
use crate::coords::ScreenPoint;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::Session;
/// Where the gap below the acting page's bottom edge is, in window-logical
/// points. See [`ROW_GAP_PT`].
pub(super) fn seam_y(state: &CanvasState) -> f32 {
    state.rect.max.y + ROW_GAP_PT * 0.5 * state.zoom
}

/// The zoom at which the neighbour below a seam parked at `seam` is expected to
/// leave the screen, closing part A's measuring window.
///
/// The neighbour's top edge is one whole gap below the acting page's bottom edge
/// and the gap is `ROW_GAP_PT × zoom`, so the edge descends past the pointer at
/// roughly that rate and runs out of room at
/// `(usable bottom - seam) / ROW_GAP_PT`. [`BOTTOM_DEAD_BAND_PT`] is what makes
/// "usable" different from "published".
///
/// ★ Deliberately an UNDER-estimate. The real rate measured nearer `10.7 ×
/// zoom`, because the pointer's document point slides down the screen a little as
/// the zoom rises, so dividing by 12 predicts the window closing earlier than it
/// does. An optimistic prediction here would let a run start a climb it cannot
/// finish and then report an unmeasured absence, which is the exact failure this
/// function exists to prevent.
pub(super) fn window_closes_at(canvas: LRect, seam: f32) -> f32 {
    ((canvas.max.y - BOTTOM_DEAD_BAND_PT - seam) / ROW_GAP_PT).max(0.0)
}
/// A window-logical point inside the canvas, as a screen point.
///
/// Expressed as fractions of the published canvas rect rather than assembled
/// from a window origin, because that is the crate's coordinate contract: a
/// check that builds its own screen coordinates stops hitting anything the
/// first time a panel width changes, and a stale coordinate is
/// symptom-identical to a broken conversion.
fn aim_at(session: &Session, canvas: LRect, x: f32, y: f32) -> Result<ScreenPoint> {
    let fx = (x - canvas.min.x) / canvas.width();
    let fy = (y - canvas.min.y) / canvas.height();
    Ok(session.frame()?.declared_at(canvas, fx, fy))
}

/// Scroll the continuous strip until the gap after the acting page sits in the
/// middle of the canvas, and return the point to roll the wheel at.
///
/// Fails — as a SKIP — rather than guessing. A run that climbed from a seam
/// outside the band would push one of the two pages off screen early and would
/// then report "the neighbour was never visible", which is a statement about
/// the harness wearing the clothes of a statement about the application.
pub(super) fn park_on_the_seam(
    session: &Session,
    driver: &Driver,
    canvas: LRect,
    report: &mut CheckReport,
) -> Result<(ScreenPoint, CanvasState)> {
    let lo = canvas.min.y + canvas.height() * SEAM_BAND.0;
    let hi = canvas.min.y + canvas.height() * SEAM_BAND.1;
    let centre = session.frame()?.declared_at(canvas, 0.5, 0.5);
    let mut last = None;
    for _ in 0..=SEAM_SEARCH_NOTCHES {
        let trace = session.trace()?;
        if let Some((reason, _)) = unavailable_now(&trace) {
            return Err(Error::new(format!(
                "the canvas is drawing nothing (`{UNAVAILABLE_EVENT} reason={reason}`) before the \
                 climb has even started, so there is no seam to park on. SKIPPED."
            )));
        }
        let Some(state) = latest_canvas(&trace) else {
            return Err(Error::new(format!(
                "the canvas has never published a `{CANVAS_EVENT}` line, so this check cannot \
                 find a page, let alone the gap after it. Is the document open?"
            )));
        };
        let y = seam_y(&state);
        let after_last_page = state.page + 1 >= PAGE_COUNT;
        if !after_last_page && (lo..=hi).contains(&y) {
            let x = ((state.rect.min.x + state.rect.max.x) * 0.5)
                .clamp(canvas.min.x + EDGE_MARGIN_PT, canvas.max.x - EDGE_MARGIN_PT);
            let at = aim_at(session, canvas, x, y)?;
            // ★★★ The window is computed and REPORTED before a notch is spent.
            // See the module header: a run whose window is empty climbs the whole
            // budget and then reports an absence it was never in a position to
            // observe, which reads exactly like a passing measurement.
            let room = canvas.max.y - BOTTOM_DEAD_BAND_PT - y;
            let closes = window_closes_at(canvas, y);
            let need = NEIGHBOUR_UNORDERABLE_ZOOM * WINDOW_MARGIN;
            report.note(format!(
                "parked the pointer in the gap after page {page} at y={y:.0} pt (canvas \
                 {min:.0}..{max:.0}, band {lo:.0}..{hi:.0}), zoom {zoom:.2}; {room:.0} pt of room \
                 below the seam, so the window part A can measure in is about zoom \
                 {NEIGHBOUR_UNORDERABLE_ZOOM:.1}..{closes:.0}",
                page = state.page,
                min = canvas.min.y,
                max = canvas.max.y,
                zoom = state.zoom,
            ));
            if closes < need {
                return Err(Error::new(format!(
                    "the seam parked at y={y:.0} pt leaves only {room:.0} pt of room below it, so \
                     the neighbour is expected off screen by about zoom {closes:.0} — at or below \
                     the zoom {NEIGHBOUR_UNORDERABLE_ZOOM:.1} at which it becomes unorderable, \
                     which is the state part A measures. The window is EMPTY and nothing would be \
                     measured by climbing, so the run SKIPS here rather than spending \
                     {CLIMB_NOTCHES_A} notches to report an absence it could not have observed. \
                     Wanted at least zoom {need:.0} of headroom. The two constants that decide \
                     this are `VIEWPORT` (canvas height) and `SEAM_BAND` (where in it the seam may \
                     sit); both carry the arithmetic."
                )));
            }
            return Ok((at, state));
        }
        last = Some((state.page, y));
        // Scroll DOWN (negative) when the seam is below the band and there is
        // still a page after the acting one; UP otherwise, which covers both
        // "the seam is above the band" and "we have run past the last sheet,
        // where there is no neighbour to be unorderable".
        let notches = if y > hi && !after_last_page { -1 } else { 1 };
        driver.scroll_at(centre, notches)?;
        session.settle(10);
    }
    Err(Error::new(format!(
        "spent {SEAM_SEARCH_NOTCHES} wheel notches and never got the gap after a page into the \
         middle of the canvas (band {lo:.0}..{hi:.0} pt); it last sat at {last:?} (page, y). \
         Either the wheel is not scrolling the strip or the opening zoom makes one page taller \
         than the search can step through. SKIPPED: nothing about O186 has been measured."
    )))
}
