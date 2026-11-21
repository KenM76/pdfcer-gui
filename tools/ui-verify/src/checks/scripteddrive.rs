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

/// The scripted twin of `scale_aim::zoom_to`: Ctrl+wheel at `aim` until the
/// reported zoom reaches `wanted`, re-aiming onto `target_canvas` (canvas
/// points, y down) after every rung and keeping the aim inside `viewport`.
/// Answers the zoom reached.
pub fn zoom_to(
    session: &Session,
    pointer: &ScriptedPointer,
    aim: &mut WindowPoint,
    target_canvas: (f32, f32),
    wanted: f32,
    viewport: Option<LRect>,
) -> Result<f32> {
    use crate::checks::scale_aim::current_zoom;
    let aimer = Aimer {
        session,
        pointer,
        target_canvas,
        cap: session.frame()?.client_logical().width() / 3.0,
        viewport,
    };
    let mut zoom = current_zoom(session)?;
    let mut spins = 0;
    *aim = aimer.re_aim(*aim, zoom)?;
    while zoom < wanted && spins < 240 {
        // Bigger steps while far away, one line when close, so the rung is
        // approached from below rather than jumped over.
        let ratio = wanted / zoom.max(0.001);
        let lines = if ratio > 8.0 {
            6
        } else if ratio > 2.0 {
            2
        } else {
            1
        };
        for _ in 0..lines {
            pointer.wheel(session, *aim, 1.0, true)?;
        }
        session.settle(8);
        let now = current_zoom(session)?;
        if (now - zoom).abs() < f32::EPSILON {
            break; // the ceiling, or the wheel is not landing
        }
        zoom = now;
        *aim = aimer.re_aim(*aim, zoom)?;
        spins += 1;
    }
    while zoom > wanted * 1.35 && spins < 300 {
        pointer.wheel(session, *aim, -1.0, true)?;
        session.settle(8);
        let now = current_zoom(session)?;
        if (now - zoom).abs() < f32::EPSILON {
            break;
        }
        zoom = now;
        *aim = aimer.re_aim(*aim, zoom)?;
        spins += 1;
    }
    // The region raster is slow; let it land before anything is measured.
    session.settle(60);
    let zoom = current_zoom(session)?;
    *aim = aimer.re_aim(*aim, zoom)?;
    Ok(zoom)
}

/// The scripted twin of `scale_aim::aim_residual`: how far the pointer at
/// `aim` is from `target_canvas`, in canvas points, as the application
/// reports where it is.
pub fn aim_residual(
    session: &Session,
    pointer: &ScriptedPointer,
    aim: WindowPoint,
    target_canvas: (f32, f32),
) -> Result<Option<f32>> {
    pointer.hover(session, aim)?;
    session.settle(6);
    let Some((x, y)) = crate::checks::scale_aim::reported_page_point(session)? else {
        return Ok(None);
    };
    Ok(Some((target_canvas.0 - x).hypot(target_canvas.1 - y)))
}

/// What [`zoom_to`] steers by: the target, the largest single correction in
/// logical points and the rect the aim is held inside.
struct Aimer<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    target_canvas: (f32, f32),
    cap: f32,
    viewport: Option<LRect>,
}

impl Aimer<'_> {
    /// Correct `at` towards the target until a move stops improving the
    /// residual by a tenth, at most a dozen moves: `scale_aim::re_aim`'s loop.
    fn re_aim(&self, at: WindowPoint, zoom: f32) -> Result<WindowPoint> {
        let mut at = at;
        let mut best = f32::INFINITY;
        for _ in 0..12 {
            let next = self.re_aim_once(at, zoom)?;
            let Some(got) = crate::checks::scale_aim::reported_page_point(self.session)? else {
                return Ok(next);
            };
            let (tx, ty) = self.target_canvas;
            let residual = ((tx - got.0).hypot(ty - got.1) * zoom).abs();
            at = next;
            // `>=` is false for NaN, so a degenerate reading ends the loop.
            if residual >= best * 0.9 || !residual.is_finite() {
                break;
            }
            best = residual;
            if residual < 2.0 {
                break;
            }
        }
        Ok(at)
    }

    /// One correction: hover, read where the application says the pointer
    /// is, move by the difference (capped), clamped into the viewport.
    fn re_aim_once(&self, at: WindowPoint, zoom: f32) -> Result<WindowPoint> {
        self.pointer.hover(self.session, at)?;
        self.session.settle(3);
        let Some(got) = crate::checks::scale_aim::reported_page_point(self.session)? else {
            return Ok(at);
        };
        let dx = (self.target_canvas.0 - got.0) * zoom;
        let dy = (self.target_canvas.1 - got.1) * zoom;
        if !dx.is_finite() || !dy.is_finite() {
            return Ok(at);
        }
        let (dx, dy) = (dx.clamp(-self.cap, self.cap), dy.clamp(-self.cap, self.cap));
        if dx.abs() < 0.5 && dy.abs() < 0.5 {
            return Ok(at);
        }
        let moved = offset(at, dx, dy);
        // A pointer outside the canvas gets no `canvas-pointer` line, so an aim
        // that leaves it is held at the edge where the next reading can still
        // improve it.
        let Some(vp) = self
            .viewport
            .filter(|vp| vp.width() >= 1.0 && vp.height() >= 1.0)
        else {
            return Ok(moved);
        };
        let fx = (moved.x() - vp.min.x) / vp.width();
        let fy = (moved.y() - vp.min.y) / vp.height();
        Ok(at_fraction(vp, fx.clamp(0.03, 0.97), fy.clamp(0.03, 0.97)))
    }
}

/// The scripted twin of `save_copy::click_tab`: click the ribbon tab declared
/// as `region` and require the shell to report `id` activated.
pub fn click_tab(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    (region, id): (&str, &str),
) -> Result<()> {
    use crate::checks::driving::{TAB_EVENT, shell_trace};
    let trace = session.trace()?;
    let rect = crate::checks::driving::declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let activations = |session: &Session| -> Result<usize> {
        Ok(shell_trace(session)?
            .events(TAB_EVENT)
            .filter(|l| l.get("tab") == Some(id))
            .count())
    };
    let before = activations(session)?;
    pointer.click(session, WindowPoint::centre_of(rect))?;
    session.settle(14);
    if activations(session)? <= before {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{TAB_EVENT} tab={id}` line: the tab moved \
             between the frame that declared its rect and the frame that received the click, \
             or it did not take the press."
        )));
    }
    Ok(())
}

/// The scripted twin of `properties_metadata::open_document_properties`: the
/// File tab, then its Document properties item, on the band or in the overflow.
pub fn open_document_properties(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<()> {
    const ITEM: &str = "ribbon.item.file.document_properties"; // ui-text-exempt: a trace region name, never displayed
    let trace = session.trace()?;
    let tab =
        crate::checks::driving::declared(&trace, ui_rect, "ribbon.tab.file").ok_or_else(|| {
            Error::new(format!(
                "no `ribbon.tab.file` region. Tabs declared: {}.",
                list(&declared_names(&trace, ui_rect, "ribbon.tab."))
            ))
        })?;
    pointer.click(session, WindowPoint::centre_of(tab))?;
    session.settle(14);
    let Some(item) =
        crate::checks::driving::declared_or_in_overflow(session, pointer, ui_rect, ITEM)?
    else {
        return Err(Error::new(format!(
            "the File tab declares no `{ITEM}`, on the band or in the overflow, so the Document \
             properties panel cannot be opened. Items the File tab did declare: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.file."
            ))
        )));
    };
    pointer.click(session, WindowPoint::centre_of(item))?;
    session.settle(20);
    Ok(())
}

/// The scripted twin of `text_selection::aim`: the window point of document
/// point `at`, refused when the canvas is showing another page.
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
    crate::coords::CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, at.page)?
        .doc_to_window(at)
}
