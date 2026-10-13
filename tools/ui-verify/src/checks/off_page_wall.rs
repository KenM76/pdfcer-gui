//! `an_object_off_the_page_is_drawn_past_its_halo_wall` — the off-page square
//! is still painted after the zoom carries the halo box past the pixmap limit.
//! Driven off-screen with the scripted pointer; pixels from the window's own
//! frame.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/off_page_wall.md`.

use super::dimdrive::{Fixture, run_on, ui_rect_event};
use super::zoom_keeps_place::{CANVAS_REGION, settled};
use crate::checks::driving::declared;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// A 200 pt sheet with one square on it and one entirely off its left edge.
const FIXTURE: Fixture = Fixture {
    file: "off-page-object.pdf",
    method: "It is hand-written PDF syntax shared with the other `off_page_*` checks.",
    page: PageGeometry {
        width_pt: 200.0,
        height_pt: 200.0,
    },
};

/// Off-page display is off in Read, so the run enters Edit.
const INVOKE: &str = "mode.edit,view.page_single,view.zoom_actual";

/// The midpoint of the off-page square's bottom edge, in page points. The
/// square is `(-160, 100)`–`(-40, 140)`, wholly left of the sheet.
const EDGE: (f64, f64) = (-100.0, 100.0);

/// How far each patch's centre sits from the edge, in logical points: above
/// it inside the square, below it outside. Logical so both stay on screen at
/// any zoom.
const PATCH_OFFSET: f32 = 40.0;

/// The canvas' tier decision, carrying `tier=` and `offpage=`.
const HALO: &str = "canvas-halo";

/// Wheel notches to roll before giving up on reaching the wall.
const MAX_NOTCHES: usize = 90;

/// The tier the canvas takes once the SHEET outgrows the pixmap. A picture
/// taken there is past the window this check is about.
const SHEET_REGION: &str = "region";

/// A pixel at or below this in every channel is ink; the fixture fills black.
const INK: u8 = 96;

/// At least this fraction of the patch inside the square must be ink.
const INK_FRACTION: f64 = 0.90;

/// At most this fraction of the control patch below the square may be ink.
const PAPER_INK_FRACTION: f64 = 0.05;

/// Half the patch's side, in logical points.
const PATCH_HALF: f32 = 10.0;

/// See the module documentation.
pub struct AnObjectOffThePageIsDrawnPastItsHaloWall;

impl Check for AnObjectOffThePageIsDrawnPastItsHaloWall {
    fn name(&self) -> &'static str {
        "an_object_off_the_page_is_drawn_past_its_halo_wall"
    }

    fn defect(&self) -> &'static str {
        "content placed off the sheet disappears once the operator zooms in far enough: the \
         halo box (sheet plus overhang) outgrows the rasterizer's pixmap before the sheet does, \
         the halo is declined, and the page falls back to a whole-sheet raster that has no \
         pixels off the sheet — while the region tier, which can draw it, is not engaged \
         because the sheet alone still fits"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "off_page_wall", drive)
    }
}

/// The point `(x, y)` of the page, in window points, wherever the canvas puts
/// it now — off the sheet included.
fn locate(ctx: &CheckContext, session: &Session, x: f64, y: f64) -> Result<(WindowPoint, LRect)> {
    let trace = session.trace()?;
    let canvas = declared(&trace, ui_rect_event(ctx)?, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{CANVAS_REGION}` region was declared. SKIPPED."
        ))
    })?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE.page, 0)?;
    Ok((
        mapping.doc_to_window_off_page(DocPoint::new(0, x, y), canvas)?,
        canvas,
    ))
}

/// The fraction of `patch` that is ink.
fn ink_fraction(image: &Image, patch: PixRect) -> Option<f64> {
    let total = u64::from(patch.w) * u64::from(patch.h);
    if total == 0 {
        return None;
    }
    let dark = image
        .pixels_in(patch)
        .filter(|p| p.r <= INK && p.g <= INK && p.b <= INK)
        .count() as u64;
    #[allow(
        clippy::cast_precision_loss,
        reason = "a patch is a few hundred pixels; f64 is exact far past that" // ui-text-exempt: clippy lint justification, never displayed
    )]
    Some(dark as f64 / total as f64)
}

/// The `tier=` of the first halo decision after `after` that is not `halo`,
/// with off-page display on — the canvas saying the halo stopped fitting.
fn left_the_halo(session: &Session, after: usize) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(HALO)
        .find(|l| {
            l.lineno > after && l.get("offpage") == Some("on") && l.get("tier") != Some("halo")
        })
        .map(|l| l.raw.clone()))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let (cx, cy) = EDGE;

    // Premise: at the starting zoom the halo tier is engaged, so the square is
    // drawn and the climb below starts from a working picture.
    let trace = session.trace()?;
    let Some(halo) = trace
        .events(HALO)
        .find(|l| l.get("tier") == Some("halo") && l.get("offpage") == Some("on"))
    else {
        return Err(Error::new(format!(
            "the canvas never chose the halo tier with off-page display on, so the climb would \
             start from a page with nothing drawn off the sheet. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let halo_line = halo.lineno;

    let mut wall: Option<(String, f64)> = None;
    let mut zoom = 0.0_f64;
    for notch in 0..MAX_NOTCHES {
        let (at, canvas) = locate(ctx, session, cx, cy).map_err(|why| {
            Error::new(format!(
                "the off-page square's bottom edge is not on the canvas after {notch} notch(es): \
                 {why}. That is `off_page_zoom`'s property, not this check's. SKIPPED."
            ))
        })?;
        pointer.wheel(session, at, 1.0, true)?;
        let Some(now) = settled(session, canvas)? else {
            return Err(Error::new(
                "the canvas stopped publishing a zoom mid-climb. SKIPPED.",
            ));
        };
        if now.zoom <= zoom {
            return Err(Error::new(format!(
                "the zoom stopped rising at {:.0}% after {notch} notch(es), before the halo's \
                 wall. SKIPPED: the run never reached the zoom this check is about.",
                now.zoom * 100.0
            )));
        }
        zoom = now.zoom;
        wall = left_the_halo(session, halo_line)?.map(|line| (line, zoom));
        if wall.is_some() {
            break;
        }
    }
    let Some((wall_line, wall_zoom)) = wall else {
        return Err(Error::new(format!(
            "{MAX_NOTCHES} notches reached {:.0}% and the canvas never left the halo tier, so \
             the halo box never outgrew the pixmap. SKIPPED.",
            zoom * 100.0
        )));
    };
    report.note(format!(
        "the canvas left the halo tier at {:.0}%: `{wall_line}`",
        wall_zoom * 100.0
    ));

    // Settle long enough for the order placed at the last notch to land.
    session.settle(60);
    let decided = session
        .trace()?
        .events(HALO)
        .last()
        .map(|l| (l.get("tier").unwrap_or_default().to_owned(), l.raw.clone()))
        .unwrap_or_default();
    if decided.0 == SHEET_REGION {
        return Err(Error::new(format!(
            "the notch that crossed the halo's wall also crossed the sheet's: `{}`. The window \
             between the two walls was stepped over, so this run sampled nothing in it. \
             SKIPPED.",
            decided.1
        )));
    }
    report.note(format!("the picture's tier decision: `{}`", decided.1));
    let (edge, _) = locate(ctx, session, cx, cy)?;
    let frame = session.frame()?;
    // Screen y grows down, so the patch above the edge is inside the square.
    let patch = |dy: f32| {
        frame.logical_to_capture_pixels(LRect::new(
            Pt::new(edge.x() - PATCH_HALF, edge.y() + dy - PATCH_HALF),
            Pt::new(edge.x() + PATCH_HALF, edge.y() + dy + PATCH_HALF),
        ))
    };
    let shot = ctx.out("off_page_wall.png");
    pointer.screenshot(session, &shot)?;
    let image = Image::load_png(&shot)?;
    report.artifact(shot.clone());
    let (Some(ink), Some(paper)) = (
        ink_fraction(&image, patch(-PATCH_OFFSET)),
        ink_fraction(&image, patch(PATCH_OFFSET)),
    ) else {
        return Err(Error::new(format!(
            "a patch clipped to zero area in the capture. SKIPPED. Screenshot: {}.",
            shot.display()
        )));
    };
    report.note(format!(
        "at {:.0}%: {ink:.3} of a patch {PATCH_OFFSET:.0} pt above the square's bottom \
         edge is ink; {paper:.3} of the control the same distance below it",
        zoom * 100.0
    ));
    if paper > PAPER_INK_FRACTION {
        return Ok(Some(format!(
            "the control patch below the square is {paper:.3} ink, so the probe is not looking \
             at the page and the reading inside the square ({ink:.3}) is no evidence. \
             Screenshot: {}.",
            shot.display()
        )));
    }
    if ink < INK_FRACTION {
        return Ok(Some(format!(
            "the off-page square is not painted at {:.0}%, where the halo box stopped fitting \
             and the sheet still fits: {ink:.3} of a patch inside it is ink, {INK_FRACTION:.2} \
             required, with the control clean. The canvas' decision there was `{}`. Suspect \
             `canvas::tier::decide`: a page whose halo is declined by `render::halo::walled` \
             must take the region tier, whose request `halo::reach` widens past the sheet. \
             Screenshot: {}.",
            zoom * 100.0,
            decided.1,
            shot.display()
        )));
    }
    Ok(None)
}
