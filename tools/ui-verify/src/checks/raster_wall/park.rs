//! Getting the pointer onto the seam between two pages, and knowing how much
//! room is left below it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/raster_wall/park.md`.

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
pub(super) fn window_closes_at(canvas: LRect, seam: f32) -> f32 {
    ((canvas.max.y - BOTTOM_DEAD_BAND_PT - seam) / ROW_GAP_PT).max(0.0)
}
/// A window-logical point inside the canvas, as a screen point.
fn aim_at(session: &Session, canvas: LRect, x: f32, y: f32) -> Result<ScreenPoint> {
    let fx = (x - canvas.min.x) / canvas.width();
    let fy = (y - canvas.min.y) / canvas.height();
    Ok(session.frame()?.declared_at(canvas, fx, fy))
}

/// Scroll the continuous strip until the gap after the acting page sits in the
/// middle of the canvas, and return the point to roll the wheel at.
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
            // The window is computed and REPORTED before a notch is spent.
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
