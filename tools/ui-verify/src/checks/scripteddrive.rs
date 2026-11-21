//! `checks::scripteddrive` — helpers shared by checks that press through the
//! scripted pointer and seed the sandbox's preferences.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scripteddrive.md`.

use crate::checks::CheckContext;
use crate::checks::driving::{declared_in, declared_names, list};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

/// The trace event every region is declared under.
const UI_RECT: &str = "ui-rect";

/// Click the centre of region `name` in whichever viewport declared it, then
/// let the frame settle. Fails naming the regions declared beside it.
pub fn click(session: &Session, pointer: &ScriptedPointer, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        let prefix = name.rsplit_once('.').map_or(name, |(head, _)| head);
        Error::new(format!(
            "no `{name}` region. Declared under `{prefix}`: {}.",
            list(&declared_names(&trace, UI_RECT, prefix))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(15);
    Ok(())
}

/// The `userdata` folder beside the binary under test, where its
/// `preferences.txt` lives.
pub fn userdata(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    Ok(exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata"))
}

/// Where an off-desktop launch places its window: no OS input reaches it and
/// none of the operator's is taken.
pub const OFFSCREEN: &str = "-4200,-4200,1400,1000";

/// Place `spec`'s window off the desktop and attach a scripted pointer whose
/// step file is `<stem>.pointer.txt` in the scratch folder. Call before
/// [`Session::launch`].
pub fn offscreen(ctx: &CheckContext, spec: &mut LaunchSpec, stem: &str) -> Result<ScriptedPointer> {
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    ScriptedPointer::attach(spec, ctx.out(&format!("{stem}.pointer.txt")))
}

/// The window's own rendered frame, written to `png` and read back. Its pixels
/// are client pixels from the origin, so `WindowFrame::logical_to_capture_pixels`
/// maps declared rects into it as it does into an OS capture.
pub fn shot(session: &Session, pointer: &ScriptedPointer, png: &std::path::Path) -> Result<Image> {
    pointer.screenshot(session, png)?;
    Image::load_png(png)
}

/// Press `key` (with `mods`) in the root viewport until one of `evidence`'s
/// events gains a line, at most `driving::PRESS_TRIES` times. Answers whether
/// it ever did.
pub fn key_until_traced(
    session: &Session,
    pointer: &ScriptedPointer,
    key: &str,
    mods: Option<&str>,
    evidence: &[&str],
) -> Result<bool> {
    let count = |session: &Session| -> Result<usize> {
        let trace = session.trace()?;
        Ok(evidence.iter().map(|name| trace.events(name).count()).sum())
    };
    let before = count(session)?;
    for _ in 0..crate::checks::driving::PRESS_TRIES {
        pointer.key(session, None, key, mods)?;
        session.settle(12);
        if count(session)? > before {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The point `(fx, fy)` of the way across `r`: the scripted twin of
/// `WindowFrame::declared_at`.
#[must_use]
pub fn at_fraction(r: LRect, fx: f32, fy: f32) -> WindowPoint {
    let p = Pt {
        x: r.min.x + (r.max.x - r.min.x) * fx,
        y: r.min.y + (r.max.y - r.min.y) * fy,
    };
    WindowPoint::centre_of(LRect { min: p, max: p })
}

/// `p` moved by `(dx, dy)` logical points: the scripted twin of
/// `WindowFrame::offset_from`.
#[must_use]
pub fn offset(p: WindowPoint, dx: f32, dy: f32) -> WindowPoint {
    let q = Pt {
        x: p.x() + dx,
        y: p.y() + dy,
    };
    WindowPoint::centre_of(LRect { min: q, max: q })
}

/// A desktop-sized placement off the desktop, for a check whose ribbon group
/// must show open rather than collapsed: the off-desktop twin of
/// `Session::maximize`, which would carry the window onto the screen.
pub const OFFSCREEN_WIDE: &str = "-4200,-4200,1920,1080";

/// [`offscreen`] at [`OFFSCREEN_WIDE`].
pub fn offscreen_wide(
    ctx: &CheckContext,
    spec: &mut LaunchSpec,
    stem: &str,
) -> Result<ScriptedPointer> {
    let pointer = offscreen(ctx, spec, stem)?;
    if let Some(slot) = spec.env.iter_mut().rev().find(|(_, v)| v == OFFSCREEN) {
        OFFSCREEN_WIDE.clone_into(&mut slot.1);
    }
    Ok(pointer)
}

/// Aim document point `at` at the canvas as it is laid out now, in window
/// points: the scripted twin of `text_selection::aim`. Errs when the canvas
/// shows another page, since `at` would land somewhere plausible and wrong.
pub fn aim(
    ctx: &CheckContext,
    session: &Session,
    page: crate::coords::PageGeometry,
    at: crate::coords::DocPoint,
) -> Result<WindowPoint> {
    let trace = session.trace()?;
    let shown = trace
        .last(ctx.profile.vocab.canvas_event)
        .and_then(|l| l.get_usize("page"));
    if shown != Some(at.page) {
        return Err(Error::new(format!(
            "the canvas is showing page {}, and this check's point is on page {}.",
            shown.map_or_else(|| "an unreported index".to_owned(), |p| (p + 1).to_string()),
            at.page + 1
        )));
    }
    let mapping =
        crate::coords::CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, at.page)?;
    mapping.doc_to_window(at)
}

/// The centre of the window's client area in window points: the scripted twin
/// of `WindowFrame::layout_probe_point`.
pub fn window_centre(session: &Session) -> Result<WindowPoint> {
    let frame = session.frame()?;
    let half = |px: u32| px as f32 / frame.scale.max(f32::EPSILON) / 2.0;
    let p = Pt {
        x: half(frame.client_size.0),
        y: half(frame.client_size.1),
    };
    Ok(WindowPoint::centre_of(LRect { min: p, max: p }))
}

/// Press at `from`, move through `via` to `to` in `steps` moves per leg, and
/// release at `to`, holding at `via` for `hold` frames: the scripted twin of
/// `Driver::drag_via`, for a gesture whose middle frames must see a changing
/// pointer rather than a straight-line jump.
pub fn drag_via(
    session: &Session,
    pointer: &ScriptedPointer,
    (from, via, to): (WindowPoint, WindowPoint, WindowPoint),
    steps: u32,
    hold: u32,
) -> Result<()> {
    use crate::input::scripted::xy;
    let leg = |a: WindowPoint, b: WindowPoint| -> Result<()> {
        for i in 1..=steps {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / steps as f32;
            let p = offset(a, (b.x() - a.x()) * t, (b.y() - a.y()) * t);
            pointer.send(session, &format!("move {}", xy(p)))?;
        }
        Ok(())
    };
    pointer.send(session, &format!("down {}", xy(from)))?;
    leg(from, via)?;
    session.settle(hold);
    leg(via, to)?;
    pointer.send(session, &format!("up {}", xy(to)))?;
    Ok(())
}

/// Wheel one line at a time over `anchor`, in its own viewport, until `wanted`
/// is declared, up to `attempts` notches: the scripted twin of
/// `reaching::scroll_to`, with the same per-notch note. `Ok(None)` when it
/// never appears; an error when `anchor` itself leaves the screen.
pub fn wheel_to(
    session: &Session,
    pointer: &ScriptedPointer,
    (anchor, wanted): (&str, &str),
    attempts: usize,
    report: &mut crate::report::CheckReport,
) -> Result<Option<LRect>> {
    for attempt in 0..attempts {
        let trace = session.trace()?;
        if let Some((rect, _)) = declared_in(&trace, UI_RECT, wanted) {
            if attempt > 0 {
                report.note(format!(
                    "`{wanted}` was below the panel's fold; {attempt} scroll notch(es) brought \
                     it into view"
                ));
            }
            return Ok(Some(rect));
        }
        let Some((at, vp)) = declared_in(&trace, UI_RECT, anchor) else {
            return Err(Error::new(format!(
                "`{anchor}` stopped being visible while scrolling for `{wanted}`, so there is \
                 nothing left to aim the wheel at. Trace: {}.",
                session.trace_path().display()
            )));
        };
        let point = WindowPoint::centre_of(at);
        pointer.wheel_in(session, vp.as_deref(), point, -1.0)?;
        session.settle(12);
        if let Some((after, _)) = declared_in(&session.trace()?, UI_RECT, anchor) {
            report.note(format!(
                "scroll {attempt}: wheel at ({}, {}), `{anchor}` now {after:?}",
                point.x(),
                point.y()
            ));
        }
    }
    Ok(None)
}
