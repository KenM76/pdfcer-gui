//! `a_turned_page_draws_the_right_place_past_the_region_wall` — on a
//! `/Rotate 270` sheet, once the zoom is deep enough that only the visible
//! window is rasterized, the square is still drawn where the page puts it.
//! Driven off-screen with the scripted pointer; pixels from the window's own
//! frame.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/turned_region.md`.

use super::dimdrive::{Fixture, run_on};
use super::off_page_wall::{ink_fraction, locate_on};
use super::zoom_keeps_place::settled;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// A letter sheet with `/Rotate 270` and one black square near a corner.
const FIXTURE: Fixture = Fixture {
    file: "turned-sheet-mark.pdf",
    method: "It is hand-written PDF syntax from `fixtures/turned-sheet-mark.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

const INVOKE: &str = "view.page_single,view.zoom_actual";

/// The midpoint of the square's right edge, in PDF user space. The square is
/// `(60, 600)`–`(200, 740)`.
const EDGE: (f64, f64) = (200.0, 670.0);

/// The canvas' tier decision, carrying `tier=`.
const HALO: &str = "canvas-halo";

/// The tier in which only the visible window of the sheet is rasterized.
const REGION: &str = "region";

/// Wheel notches to roll before giving up on reaching the region tier.
const MAX_NOTCHES: usize = 90;

/// At least this fraction of the patch inside the square must be ink.
const INK_FRACTION: f64 = 0.90;

/// At most this fraction of the control patch outside it may be ink.
const PAPER_INK_FRACTION: f64 = 0.05;

/// How far each patch's centre sits from the edge, in logical points.
const PATCH_OFFSET: f32 = 40.0;

/// Half the patch's side, in logical points.
const PATCH_HALF: f32 = 10.0;

/// See the module documentation.
pub struct ATurnedPageDrawsTheRightPlacePastTheRegionWall;

impl Check for ATurnedPageDrawsTheRightPlacePastTheRegionWall {
    fn name(&self) -> &'static str {
        "a_turned_page_draws_the_right_place_past_the_region_wall"
    }

    fn defect(&self) -> &'static str {
        "on a page with /Rotate, zooming past the point where only the visible window is \
         rasterized makes the picture jump to another part of the sheet and stretch: the \
         window is handed to the renderer in canvas space (y down, turned) instead of PDF \
         user space (y up, unturned)"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "turned_region", drive)
    }
}

/// The trace line placing the canvas, carrying `region=` (the window raster
/// on screen) and `want=` (the one ordered), both in PDF user space.
const POS: &str = "canvas-pos";

/// Settle rounds to wait for the ordered window raster to land.
const LAND_ROUNDS: usize = 20;

/// The window raster on screen, once it is the one the canvas ordered, as
/// `(llx, lly, urx, ury)` in PDF user space.
fn landed(session: &Session) -> Result<Option<(String, [f64; 4])>> {
    for _ in 0..LAND_ROUNDS {
        let trace = session.trace()?;
        if let Some(l) = trace.last(POS)
            && let (Some(on), Some(want)) = (l.get_f64_list("region"), l.get_f64_list("want"))
            && on.len() == 4
            && on == want
        {
            return Ok(Some((l.raw.clone(), [on[0], on[1], on[2], on[3]])));
        }
        session.settle(30);
    }
    Ok(None)
}

/// Why the landed window does not cover both patches, in user space, at
/// `zoom` logical points per page point.
fn uncovered(window: [f64; 4], zoom: f64) -> Option<String> {
    let reach = f64::from(PATCH_OFFSET + PATCH_HALF) / zoom;
    let half = f64::from(PATCH_HALF) / zoom;
    let need = [EDGE.0 - reach, EDGE.1 - half, EDGE.0 + reach, EDGE.1 + half];
    let [llx, lly, urx, ury] = window;
    let covers = llx <= need[0] && lly <= need[1] && urx >= need[2] && ury >= need[3];
    (!covers).then(|| {
        format!(
            "the window raster on screen is ({llx:.1}, {lly:.1})-({urx:.1}, {ury:.1}) in user \
             space and does not cover the probed spot ({:.1}, {:.1})-({:.1}, {:.1}) under the \
             pointer: the canvas rasterized another part of the sheet",
            need[0], need[1], need[2], need[3]
        )
    })
}

/// The user-space point `(x, y)` in window points, and the canvas viewport.
fn locate(ctx: &CheckContext, session: &Session, x: f64, y: f64) -> Result<(WindowPoint, LRect)> {
    locate_on(ctx, session, FIXTURE.page, (x, y))
}

/// The unit window-space direction from the edge into the square, derived
/// from the mapping so that no assumption about the turn is written here.
fn inward(ctx: &CheckContext, session: &Session) -> Result<(f32, f32)> {
    let (inside, _) = locate(ctx, session, EDGE.0 - 1.0, EDGE.1)?;
    let (outside, _) = locate(ctx, session, EDGE.0 + 1.0, EDGE.1)?;
    let (dx, dy) = (inside.x() - outside.x(), inside.y() - outside.y());
    let len = dx.hypot(dy);
    if len <= f32::EPSILON {
        return Err(Error::new(
            "the mapping put both sides of the square's edge on one pixel. SKIPPED.",
        ));
    }
    Ok((dx / len, dy / len))
}

/// The latest tier decision after `start`, if it is the region tier. Read
/// before the zoom's rise is judged: the top of the zoom range is where a
/// letter sheet first outgrows the pixmap.
fn region_since(session: &Session, start: usize) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .last_after(HALO, start)
        .filter(|l| l.get("tier") == Some(REGION))
        .map(|l| l.raw.clone()))
}

/// Climbs the zoom at the square's edge until the canvas takes the region
/// tier, returning the zoom and the decision line.
fn climb(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<(f64, String)> {
    let start = session.trace()?.mark();
    let mut zoom = 0.0_f64;
    for notch in 0..MAX_NOTCHES {
        let (at, canvas) = locate(ctx, session, EDGE.0, EDGE.1).map_err(|why| {
            Error::new(format!(
                "the square's edge is not on the canvas after {notch} notch(es): {why}. \
                 SKIPPED."
            ))
        })?;
        pointer.wheel(session, at, 1.0, true)?;
        let Some(now) = settled(session, canvas)? else {
            return Err(Error::new(
                "the canvas stopped publishing a zoom mid-climb. SKIPPED.",
            ));
        };
        if let Some(line) = region_since(session, start)? {
            return Ok((now.zoom, line));
        }
        if now.zoom <= zoom {
            return Err(Error::new(format!(
                "the zoom stopped rising at {:.0}% after {notch} notch(es), before the region \
                 tier. SKIPPED.",
                now.zoom * 100.0
            )));
        }
        zoom = now.zoom;
    }
    Err(Error::new(format!(
        "{MAX_NOTCHES} notches reached {:.0}% and the canvas never took the region tier. \
         SKIPPED.",
        zoom * 100.0
    )))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let (zoom, decision) = climb(ctx, session, pointer)?;
    report.note(format!("region tier at {:.0}%: `{decision}`", zoom * 100.0));
    let Some((pos, window)) = landed(session)? else {
        return Ok(Some(format!(
            "the canvas ordered a window raster at {:.0}% and none landed: `{}` never showed \
             `region=` equal to `want=`.",
            zoom * 100.0,
            POS
        )));
    };
    report.note(format!("window raster on screen: `{pos}`"));
    if let Some(why) = uncovered(window, zoom) {
        return Ok(Some(format!(
            "at {:.0}%, {why}. Suspect `rasterregion::PageFrame::canvas_to_user` for \
             /Rotate 270.",
            zoom * 100.0
        )));
    }
    let (edge, _) = locate(ctx, session, EDGE.0, EDGE.1)?;
    let (ix, iy) = inward(ctx, session)?;
    let frame = session.frame()?;
    let patch = |sign: f32| {
        let (cx, cy) = (
            edge.x() + sign * ix * PATCH_OFFSET,
            edge.y() + sign * iy * PATCH_OFFSET,
        );
        frame.logical_to_capture_pixels(LRect::new(
            Pt::new(cx - PATCH_HALF, cy - PATCH_HALF),
            Pt::new(cx + PATCH_HALF, cy + PATCH_HALF),
        ))
    };
    let shot = ctx.out("turned_region.png");
    pointer.screenshot(session, &shot)?;
    report.artifact(shot.clone());
    let image = Image::load_png(&shot)?;
    let (Some(ink), Some(paper)) = (
        ink_fraction(&image, patch(1.0)),
        ink_fraction(&image, patch(-1.0)),
    ) else {
        return Err(Error::new(format!(
            "a patch clipped to zero area in the capture. SKIPPED. Screenshot: {}.",
            shot.display()
        )));
    };
    report.note(format!(
        "at {:.0}%: {ink:.3} of a patch {PATCH_OFFSET:.0} pt inside the square's edge is ink; \
         {paper:.3} of the control the same distance outside it",
        zoom * 100.0
    ));
    if paper > PAPER_INK_FRACTION {
        return Ok(Some(format!(
            "the control patch outside the square is {paper:.3} ink: the region raster drew \
             ink where the sheet is blank, so it shows another part of the page. Suspect \
             `rasterregion::PageFrame::canvas_to_user` for /Rotate 270. Screenshot: {}.",
            shot.display()
        )));
    }
    if ink < INK_FRACTION {
        return Ok(Some(format!(
            "the square is not drawn where the page puts it at {:.0}%, in the region tier: \
             {ink:.3} of a patch inside it is ink, {INK_FRACTION:.2} required, with the control \
             clean. The window was rasterized from the wrong part of the sheet. Suspect \
             `rasterregion::PageFrame::canvas_to_user` for /Rotate 270. Screenshot: {}.",
            zoom * 100.0,
            shot.display()
        )));
    }
    Ok(None)
}
