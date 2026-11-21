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

/// A desktop-sized off-desktop geometry, for a check whose OS twin maximised
/// the window to widen the ribbon or the canvas.
pub const OFFSCREEN_WIDE: &str = "-4200,-4200,1920,1040";

/// [`offscreen`] at `geometry` (`x,y,w,h` in logical points) instead of
/// [`OFFSCREEN`]. `Session::maximize` would bring the window onto the
/// desktop, so a check that maximised launches at [`OFFSCREEN_WIDE`].
pub fn offscreen_at(
    ctx: &CheckContext,
    spec: &mut LaunchSpec,
    stem: &str,
    geometry: &str,
) -> Result<ScriptedPointer> {
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    spec.env
        .push((viewport_env.to_owned(), geometry.to_owned()));
    spec.place = false;
    ScriptedPointer::attach(spec, ctx.out(&format!("{stem}.pointer.txt")))
}

/// Turn the wheel `lines` at `at`, `times` times, with Ctrl held when `ctrl`:
/// the scripted twin of `Driver::scroll_at_held` (and of `scroll_at` with
/// `times` = 1). One OS notch is one wheel line; the sign is the notch's.
pub fn wheel_times(
    session: &Session,
    pointer: &ScriptedPointer,
    at: WindowPoint,
    lines: f32,
    ctrl: bool,
    times: usize,
) -> Result<()> {
    for _ in 0..times {
        pointer.wheel(session, at, lines, ctrl)?;
    }
    Ok(())
}

/// Press the View tab, then `item` on it or in its overflow: the scripted twin
/// of `fit_places_the_view::invoke`.
pub fn view_item(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    item: &str,
) -> Result<()> {
    use crate::checks::driving::{declared, declared_or_in_overflow};
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.view").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.view` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    pointer.click(session, WindowPoint::centre_of(tab))?;
    session.settle(14);
    let region = declared_or_in_overflow(session, pointer, ui_rect, item)?.ok_or_else(|| {
        Error::new(format!(
            "no `{item}` region on the View tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.view."
            ))
        ))
    })?;
    pointer.click(session, WindowPoint::centre_of(region))?;
    session.settle(24);
    Ok(())
}

/// Click the ribbon tab `region` and require the shell's `TAB_EVENT tab=<id>`
/// line for it: the scripted twin of the OS checks' `click_tab`.
pub fn click_tab_traced(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    (region, id): (&str, &str),
) -> Result<()> {
    use crate::checks::driving::{TAB_EVENT, declared, shell_trace};
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "no `{region}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let count = |s: &Session| -> Result<usize> {
        Ok(shell_trace(s)?
            .events(TAB_EVENT)
            .filter(|l| l.get("tab") == Some(id))
            .count())
    };
    let before = count(session)?;
    pointer.click(session, WindowPoint::centre_of(rect))?;
    session.settle(14);
    if count(session)? <= before {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{TAB_EVENT} tab={id}` line."
        )));
    }
    Ok(())
}

/// Click ribbon command `name` — on the band, in a collapsed group or in the
/// overflow — and require a new shell `INVOKE_EVENT id=<id>` line: the
/// scripted twin of the OS checks' `click_command`.
pub fn click_command_traced(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    (name, id): (&str, &str),
    settle: u32,
) -> Result<()> {
    use crate::checks::driving::{INVOKE_EVENT, declared_or_in_overflow, shell_trace};
    let rect = declared_or_in_overflow(session, pointer, ui_rect, name)?.ok_or_else(|| {
        Error::new(format!(
            "`{name}` was not on the band, in any collapsed group, or in the overflow."
        ))
    })?;
    let count = |s: &Session| -> Result<usize> {
        Ok(shell_trace(s)?
            .events(INVOKE_EVENT)
            .filter(|l| l.get("id") == Some(id))
            .count())
    };
    let before = count(session)?;
    pointer.click(session, WindowPoint::centre_of(rect))?;
    session.settle(settle);
    if count(session)? <= before {
        return Err(Error::new(format!(
            "`{name}` DID NOT TAKE THE CLICK: declared at {rect:?}, and no new \
             `{INVOKE_EVENT} id={id}` line followed."
        )));
    }
    Ok(())
}
