//! # `checks::scale_aim` — getting to a zoom rung, and staying on the target
//!
//! Split out of [`super::scale_sweep`] under **R2** on 2026-09-05, when the
//! marquee arm, the aim-residual guard and the three repaired diagnoses took
//! that file to 1,598 lines.
//!
//! ## ★★ The seam is a real subject boundary, not a size-driven cut
//!
//! `scale_sweep` is now one thing: **what the mouse can do once you are
//! there.** Click-select, drag, marquee, nodes, handles, pan — a battery of
//! probes, each of which changes when a *gesture* changes.
//!
//! This file is the other thing: **how you get there and stay on the point.**
//! The zoom ladder, the closed-loop re-aim, the tier reading, and the
//! vocabulary those three share. It changes when the *zoom or position model*
//! changes — a new render tier, a new position anchor, a different zoom
//! gesture — and it is entirely uninterested in what a press means.
//!
//! ⇒ The two change for different reasons, which is this project's stated test
//! for a module boundary (`canvas::interact`'s header makes the identical
//! argument about composition versus interaction).
//!
//! ## ★★★ Why the aiming half is worth reading alone
//!
//! Because it is the half that can make every measurement in the other file a
//! statement about the harness. The 2026-09-05 sweep filed *"clicking directly
//! on the content the zoom is anchored to selected nothing"* at five rungs;
//! what had happened is written in [`re_aim`]'s own header — the correction
//! moves the **pointer**, and Ctrl+wheel holds the point under the pointer
//! fixed, so an error the correction cannot close is magnified by every
//! further notch rather than reduced. It was a limit of this loop and not
//! anything the application did. A reader who wants to know whether a
//! scale-sweep finding is real starts here, and [`aim_residual`] is the number
//! that answers it.
//!
//! ## The dependency runs one way
//!
//! `scale_sweep` uses this module; nothing here knows that `Rung`, the report
//! or any probe exists. The three trace names this layer reads live **here**
//! rather than upstairs, and are `pub` so the battery can share the ones it
//! also needs — one definition, imported upward, so a rename in the
//! application cannot leave one file corrected and the other quietly reading a
//! name that is never printed.

use crate::coords::ScreenPoint;
use crate::error::Result;
use crate::input::Driver;
use crate::launch::Session;
use crate::sys::vk;

/// `canvas rect=… zoom=… page=…` — the canvas's own state line.
pub const CANVAS_EVENT: &str = "canvas";
/// `canvas-pointer screen=(x,y) page=(x,y) pdf=(x,y) zoom=…`.
pub const POINTER_EVENT: &str = "canvas-pointer";
/// `canvas-pos at=… tier=… region=… want=… ext=…`.
///
/// ★ `region=none` is the whole-page tier and anything else is the region tier,
/// so this line — not an arithmetic guess — is what says which tier a rung
/// actually reached. `tier=` names the POSITION model, which is the third
/// boundary.
pub const POSITION_EVENT: &str = "canvas-pos";
/// The zoom the application last reported.
pub fn current_zoom(session: &Session) -> Result<f32> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0))
}

/// **What tier the application says it is in**, from its own position line.
///
/// `region=none` is the whole-page raster; anything else is the region tier.
/// `tier=` is the *position* model, the third boundary. Read rather than
/// computed, because a boundary this check derived itself would be a second
/// copy of arithmetic that lives in `render::strategy` and `viewer::ceiling`.
pub fn tier_of(session: &Session) -> Result<String> {
    let trace = session.trace()?;
    let Some(line) = trace.events(POSITION_EVENT).last() else {
        return Ok("no `canvas-pos` line".to_owned());
    };
    let raster = if line.get("region").is_none_or(|r| r == "none") {
        "whole-page raster"
    } else {
        "REGION raster"
    };
    Ok(format!(
        "{raster}, position tier `{}`",
        line.get("tier").unwrap_or("?")
    ))
}

/// **Steer the pointer back onto the target**, using the application's own
/// report of where it thinks the pointer is.
///
/// # ★★★ Why the aim is a loop and not a calculation
///
/// The subject of this sweep is a **0.85 pt** pair of cells on a US Letter
/// sheet. A single conversion at the opening zoom places the pointer to within
/// one screen pixel, which is about one page point there — larger than the
/// thing being aimed at. Every rung after that would then be measuring blank
/// paper beside the cells rather than the cells.
///
/// So the aim is corrected before every wheel batch from `canvas-pointer`,
/// which publishes the canvas-space point the application believes the pointer
/// is on. Because zoom-to-cursor keeps that point fixed, each correction is
/// applied at a higher magnification than the last and the error halves with
/// every doubling: one screen pixel of residual error is one page point at
/// 100 %, and 5 × 10⁻⁵ of one at twenty thousand percent.
///
/// ★★ It is also, incidentally, a **second** reading of the conversion under
/// test — if `screen_to_page` were lying, this loop would diverge rather than
/// converge, and the caller would see the aim wander. That is why the corrected
/// aim is reported at every rung.
///
/// The correction is capped at a third of the viewport per step: a larger jump
/// means the target has left the window entirely, and chasing it with one
/// enormous pointer move would land somewhere arbitrary. Capped, the loop still
/// converges over the following steps.
pub fn re_aim(
    session: &Session,
    driver: &Driver,
    at: ScreenPoint,
    target_canvas: (f32, f32),
    zoom: f32,
    cap: f32,
    viewport: Option<crate::geom::LRect>,
) -> Result<ScreenPoint> {
    // ★★★ **ITERATED, not one shot** — 2026-09-05, and the single correction
    // this replaces is why the sweep lost its target above 2,559 %.
    //
    // The correction is capped at `cap` (a third of the viewport) per move, so
    // an error larger than that needs several. It used to get exactly one per
    // Ctrl+wheel step — and the next Ctrl+wheel then MAGNIFIED whatever was
    // left, because zoom-to-cursor holds the point under the POINTER fixed, not
    // the point you meant. Once the residual exceeds the viewport it can never
    // be recovered by moving the pointer at all: at 2,298,020 % the whole
    // viewport spans 0.021 canvas points.
    //
    // Measured before this loop: residual 0.02 canvas pt at 2,559 % and
    // **4.49** at 6,957 % — one rung later, one pan probe and one ribbon click
    // in between. The sweep then reported *"clicking directly on the content
    // the zoom is anchored to selected nothing"* at every rung above, which was
    // a click on blank paper 312 px from the content.
    //
    // ⇒ Correct until it stops improving. Bounded, and it exits on the first
    // reading that is already good, so the ordinary case still costs one move.
    let mut at = at;
    let mut best = f32::INFINITY;
    for _ in 0..RE_AIM_STEPS {
        let next = re_aim_once(session, driver, at, target_canvas, zoom, cap, viewport)?;
        let Some(got) = reported_page_point(session)? else {
            return Ok(next);
        };
        let residual = ((target_canvas.0 - got.0).hypot(target_canvas.1 - got.1) * zoom).abs();
        at = next;
        // No improvement means the cap, the viewport clamp or the zoom has put
        // the target out of reach; another move would only jitter the pointer.
        //
        // ★ Written as `>=` rather than `!(… < …)`: a NaN residual — which a
        // degenerate mapping could produce — must end the loop, and `>=` is
        // false for NaN, so the `break` below it is reached. The negated form
        // says the same thing and clippy is right that nobody can see it.
        if residual >= best * RE_AIM_PROGRESS || !residual.is_finite() {
            break;
        }
        best = residual;
        if residual < RE_AIM_TOLERANCE_PX {
            break;
        }
    }
    Ok(at)
}

/// How many correction moves [`re_aim`] will make before giving up.
///
/// ★ Each is one pointer move and one 70 ms settle, and the ordinary case
/// exits after the first. Twelve is enough to close a full-viewport error at a
/// third of the viewport per step with room to spare.
const RE_AIM_STEPS: usize = 12;

/// The residual, in screen pixels, at which [`re_aim`] stops correcting.
///
/// ★★ Screen pixels rather than canvas points, deliberately: what the next
/// click needs is to land on the same ink, and "the same ink" is a screen
/// distance. In canvas points the same tolerance would be meaninglessly tight
/// at 100 % and meaninglessly loose at 200,000 %.
const RE_AIM_TOLERANCE_PX: f32 = 2.0;

/// The factor by which a correction must improve the residual to be worth
/// another. Anything closer to 1.0 is the loop chasing rounding.
const RE_AIM_PROGRESS: f32 = 0.9;

/// One correction move. See [`re_aim`], which iterates this.
#[allow(clippy::too_many_arguments, reason = "one move of a control loop")]
fn re_aim_once(
    session: &Session,
    driver: &Driver,
    at: ScreenPoint,
    target_canvas: (f32, f32),
    zoom: f32,
    cap: f32,
    viewport: Option<crate::geom::LRect>,
) -> Result<ScreenPoint> {
    driver.move_to(at)?;
    std::thread::sleep(std::time::Duration::from_millis(70));
    let Some(got) = reported_page_point(session)? else {
        return Ok(at);
    };
    let dx = (target_canvas.0 - got.0) * zoom;
    let dy = (target_canvas.1 - got.1) * zoom;
    if !dx.is_finite() || !dy.is_finite() {
        return Ok(at);
    }
    let dx = dx.clamp(-cap, cap);
    let dy = dy.clamp(-cap, cap);
    if dx.abs() < 0.5 && dy.abs() < 0.5 {
        return Ok(at);
    }
    let frame = session.frame()?;
    let moved = frame.offset_from(at, dx, dy);
    // ★★★ CLAMPED INTO THE CANVAS VIEWPORT, and the sweep ran away without it.
    //
    // The pan probe scrolls the view, so the next rung's first correction is a
    // large one; two rungs of that walked the aim off the top of the window
    // (measured: y = 304, then 184, then −56). A pointer outside the window
    // gets no `canvas-pointer` line, so the loop then steers from a stale
    // reading and the Ctrl+wheel lands on nothing — the zoom stalled at 21x and
    // three rungs reported a 100 % conversion error about an application that
    // was never asked anything. An aim that leaves the canvas is not a
    // correction, it is a lost target, and it must be held at the edge where the
    // next reading can still improve it.
    let Some(vp) = viewport else {
        return Ok(moved);
    };
    // ★ Expressed as a FRACTION of the viewport and rebuilt through
    // `declared_at`, because `ScreenPoint` has no public constructor — by
    // design, so that every screen coordinate in this harness comes from a
    // conversion rather than from arithmetic somebody did by hand.
    let p0 = frame.declared_at(vp, 0.0, 0.0);
    let p1 = frame.declared_at(vp, 1.0, 1.0);
    let span_x = (p1.x() - p0.x()) as f32;
    let span_y = (p1.y() - p0.y()) as f32;
    if span_x.abs() < 1.0 || span_y.abs() < 1.0 {
        return Ok(moved);
    }
    let fx = (moved.x() - p0.x()) as f32 / span_x;
    let fy = (moved.y() - p0.y()) as f32 / span_y;
    Ok(frame.declared_at(vp, fx.clamp(0.03, 0.97), fy.clamp(0.03, 0.97)))
}

/// Ctrl+wheel at `aim` until the reported zoom reaches `wanted`.
///
/// Returns the zoom actually reached, and leaves `aim` corrected onto the
/// target — see [`re_aim`]. Rolls in small batches and re-reads, because one
/// notch's factor is egui's and not this harness's to know.
pub fn zoom_to(
    session: &Session,
    driver: &Driver,
    aim: &mut ScreenPoint,
    target_canvas: (f32, f32),
    wanted: f32,
    viewport: Option<crate::geom::LRect>,
) -> Result<f32> {
    let cap = session.frame()?.client_logical().width() / 3.0;
    let mut zoom = current_zoom(session)?;
    let mut spins = 0;
    *aim = re_aim(session, driver, *aim, target_canvas, zoom, cap, viewport)?;
    while zoom < wanted && spins < 240 {
        // Bigger steps while far away, one notch when close, so the rung is
        // approached from below rather than jumped over.
        let ratio = wanted / zoom.max(0.001);
        let notches = if ratio > 8.0 {
            6
        } else if ratio > 2.0 {
            2
        } else {
            1
        };
        driver.scroll_at_held(*aim, &[vk::CONTROL], 1, notches)?;
        session.settle(8);
        let now = current_zoom(session)?;
        if (now - zoom).abs() < f32::EPSILON {
            break; // the ceiling, or the wheel is not landing
        }
        zoom = now;
        *aim = re_aim(session, driver, *aim, target_canvas, zoom, cap, viewport)?;
        spins += 1;
    }
    while zoom > wanted * 1.35 && spins < 300 {
        driver.scroll_at_held(*aim, &[vk::CONTROL], -1, 1)?;
        session.settle(8);
        let now = current_zoom(session)?;
        if (now - zoom).abs() < f32::EPSILON {
            break;
        }
        zoom = now;
        *aim = re_aim(session, driver, *aim, target_canvas, zoom, cap, viewport)?;
        spins += 1;
    }
    // The region raster is slow; let it land before anything is measured.
    session.settle(60);
    let zoom = current_zoom(session)?;
    *aim = re_aim(session, driver, *aim, target_canvas, zoom, cap, viewport)?;
    Ok(zoom)
}

/// How far the pointer actually is from the document coordinate this sweep
/// aims at, in **canvas points**.
///
/// `None` when the application has published no `canvas-pointer` line at all,
/// which is a different fact and has its own line in [`probe_pointer`].
///
/// ★ Read from the application's own report of where the pointer is, never
/// computed from the harness's mapping. A residual computed through the same
/// conversion the sweep is testing would be zero by construction — the shape
/// of measurement this project calls a proxy.
/// ★★ The pointer is put back on `aim` first. [`probe_pointer`] leaves it
/// [`PROBE_PX`] away, and reading the residual from that position would report
/// the probe's own displacement as an aiming error — a harness measuring its
/// own last move.
pub fn aim_residual(
    session: &Session,
    driver: &Driver,
    aim: ScreenPoint,
    target_canvas: (f32, f32),
) -> Result<Option<f32>> {
    driver.move_to(aim)?;
    session.settle(6);
    let Some((x, y)) = reported_page_point(session)? else {
        return Ok(None);
    };
    let (dx, dy) = (target_canvas.0 - x, target_canvas.1 - y);
    Ok(Some(dx.hypot(dy)))
}

/// Parse `(1.23,4.56)` — the shape `canvas-pointer` prints its two spaces in.
pub fn parse_paren_pair(s: &str) -> Option<(f32, f32)> {
    let inner = s.trim().strip_prefix('(')?.strip_suffix(')')?;
    let (a, b) = inner.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// The canvas point the application says the pointer is on, right now.
pub fn reported_page_point(session: &Session) -> Result<Option<(f32, f32)>> {
    Ok(session
        .trace()?
        .events(POINTER_EVENT)
        .last()
        .and_then(|l| l.get("page"))
        .and_then(parse_paren_pair))
}
