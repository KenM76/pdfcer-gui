//! `checks::scripteddrive` — helpers shared by checks that press through the
//! scripted pointer and seed the sandbox's preferences.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scripteddrive.md`.

use crate::checks::CheckContext;
use crate::checks::comments_census::Census;
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

/// A document point on the canvas as it is laid out right now, refusing a
/// point on a page the canvas is not showing: the scripted twin of
/// `text_selection::aim`.
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
            "the canvas is showing page {}, and the point is on page {}. Converting a document \
             point against another page's rect would put it somewhere plausible and wrong.",
            shown.map_or_else(|| "an unreported index".to_owned(), |p| (p + 1).to_string()),
            at.page + 1
        )));
    }
    let mapping =
        crate::coords::CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, at.page)?;
    mapping.doc_to_window(at)
}

/// Click ribbon item `name` wherever it is (band, collapsed group, overflow)
/// in the viewport that declared it, and require a new `ribbon-invoke id=<id>`
/// from the shell.
pub fn click_command(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    (name, id): (&str, &str),
    settle: u32,
) -> Result<()> {
    use crate::checks::driving::{INVOKE_EVENT, declared_or_in_overflow, shell_trace};
    let invokes = |session: &Session| -> Result<usize> {
        Ok(shell_trace(session)?
            .events(INVOKE_EVENT)
            .filter(|l| l.get("id") == Some(id))
            .count())
    };
    let rect = declared_or_in_overflow(session, pointer, ui_rect, name)?.ok_or_else(|| {
        Error::new(format!(
            "`{name}` was not on the band, in any collapsed group, or in the overflow. Items \
             declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item."
            ))
        ))
    })?;
    let before = invokes(session)?;
    let viewport = declared_in(&session.trace()?, ui_rect, name).and_then(|(_, vp)| vp);
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(settle);
    if invokes(session)? <= before {
        return Err(Error::new(format!(
            "`{name}` DID NOT TAKE THE CLICK: it was declared at {rect:?} and the click produced \
             no `{INVOKE_EVENT} id={id}`."
        )));
    }
    Ok(())
}

/// The canvas object menu's redact row, its siblings, and the menu's own line.
const REDACT_ROW: &str = "menu.item.canvas.object.edit.redact_selection"; // ui-text-exempt: a trace region name, never displayed
const REDACT_ROW_PREFIX: &str = "menu.item.canvas.object."; // ui-text-exempt: a trace region name, never displayed
const CANVAS_MENU_EVENT: &str = "canvas-menu"; // ui-text-exempt: a trace event name, never displayed
const OBJECT_CONTEXT: &str = "canvas.object"; // ui-text-exempt: a trace token, never displayed

/// Right-click `at`, press the object menu's redact row, and read what it
/// requested: the scripted twin of `redact_menu::mark_through_the_menu`.
/// `Ok(Err(_))` is a failure sentence naming `what` was right-clicked.
pub(super) fn mark_through_the_menu(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    at: WindowPoint,
    what: &str,
) -> Result<std::result::Result<crate::checks::redact_menu::Marked, String>> {
    pointer.right_click(session, at)?;
    session.settle(35);
    let trace = session.trace()?;
    let Some(menu) = trace.events(CANVAS_MENU_EVENT).last() else {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED NO MENU AT ALL: no `{CANVAS_MENU_EVENT}` line \
             after a secondary click on the page. Trace: {}",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default();
    if context != OBJECT_CONTEXT {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED `{context}`, NOT `{OBJECT_CONTEXT}`: `{}`. The \
             secondary hit test lost the selection the left click made.",
            menu.raw
        )));
    }
    let Some((row, viewport)) = declared_in(&trace, ui_rect, REDACT_ROW) else {
        return Ok(Err(format!(
            "★★★ THE REDACT ROW IS NOT IN THE CANVAS OBJECT MENU: no `{REDACT_ROW}` region after \
             the menu opened on {what}. Rows it DID publish: {}.",
            list(&declared_names(&trace, ui_rect, REDACT_ROW_PREFIX))
        )));
    };
    if !row.is_substantial() {
        return Ok(Err(format!(
            "`{REDACT_ROW}` was published at {row:?}, which has no usable area."
        )));
    }
    let mark = trace.mark();
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(row))?;
    session.settle(30);
    Ok(read_mark(&session.trace()?, mark, what))
}

/// The `quads=` and `bbox=` of the last redact request after `mark`.
fn read_mark(
    trace: &crate::trace::Trace,
    mark: usize,
    what: &str,
) -> std::result::Result<crate::checks::redact_menu::Marked, String> {
    use crate::checks::redact_menu::{Bounds, Marked, REQUESTED};
    let line = trace.last_after(REQUESTED, mark).ok_or_else(|| {
        format!(
            "★★★ THE ROW WAS PRESSED AND THE VERB DID NOT RUN: no `{REQUESTED}` line after \
             pressing the redact row on {what}. A build whose page filter kept no outline writes \
             `redact-mark-selection-declined … reason=no-bounds` instead."
        )
    })?;
    let bbox = line.get("bbox").and_then(Bounds::parse).ok_or_else(|| {
        format!(
            "★★★ THE VERB RAN AND DID NOT SAY WHAT IT MARKED AS A RECTANGLE: `{}`.",
            line.raw
        )
    })?;
    let quads = line.get_usize("quads").ok_or_else(|| {
        format!(
            "★★★ THE VERB RAN AND DID NOT SAY HOW MANY REGIONS IT BUILT: `{}` carries no usable \
             `quads=` field.",
            line.raw
        )
    })?;
    Ok(Marked { quads, bbox })
}

/// Click `at` until the application resolves it into one measure pick,
/// confirming a promoted derived candidate with a second click: the scripted
/// twin of `picking::resolve_pick`, with the same bound and the same verdicts.
pub fn resolve_pick(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut crate::report::CheckReport,
    label: &str,
    at: WindowPoint,
    settle: u32,
) -> Result<std::result::Result<crate::checks::picking::ResolvedPick, String>> {
    use crate::checks::picking::{MAX_CLICKS_PER_PICK, PICK_EVENT, PICK_PROMOTED, ResolvedPick};
    let mut clicks = 0usize;
    loop {
        let before = session.trace()?.events(PICK_EVENT).count();
        clicks += 1;
        pointer.click(session, at)?;
        session.settle(settle);
        let trace = session.trace()?;
        let new: Vec<&crate::trace::TraceLine> = trace.events(PICK_EVENT).skip(before).collect();
        let [line] = new.as_slice() else {
            return Ok(Err(format!(
                "GESTURE: click {clicks} of pick {label} produced {} `{PICK_EVENT}` line(s), not \
                 1. `canvas::measure::click` traces once per click it is handed, so zero means \
                 the click never became a pick and more than one means it was delivered twice. \
                 New lines: {}.",
                new.len(),
                crate::checks::driving::list_str(
                    &new.iter().map(|l| l.raw.as_str()).collect::<Vec<_>>()
                )
            )));
        };
        if line.get("outcome") != Some(PICK_PROMOTED) {
            return Ok(Ok(ResolvedPick {
                line: (*line).clone(),
                clicks,
            }));
        }
        report.note(format!(
            "pick {label}: the application promoted a derived snap candidate rather than \
             committing it (`{}`); clicking the same point again to confirm",
            line.raw
        ));
        if clicks >= MAX_CLICKS_PER_PICK {
            return Ok(Err(format!(
                "GESTURE: the two-click confirm did not converge. Pick {label} was promoted on \
                 click 1 and again on click {clicks} at the same point, so the point the snap \
                 query resolved to moved between two clicks of a stationary pointer. Line: `{}`.",
                line.raw
            )));
        }
    }
}

/// Launch `pdf` off the desktop with both diagnostic channels and `env`, and
/// require the start line. The step file is named after `trace_name`.
pub fn launch_with(
    ctx: &CheckContext,
    report: &mut crate::report::CheckReport,
    pdf: &std::path::Path,
    trace_name: &str,
    env: &[(&str, std::path::PathBuf)],
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(trace_name));
    spec.pdf = Some(pdf.to_path_buf());
    let shell = crate::checks::driving::SHELL_DIAG_ENV;
    for (k, v) in [ctx.profile.diag_env, shell] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    for (k, v) in env {
        spec.env.push(((*k).to_owned(), v.display().to_string()));
    }
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let stem = trace_name.trim_end_matches(".trace.txt");
    let pointer = offscreen(ctx, &mut spec, stem)?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} on {} as pid {}",
        exe.display(),
        pdf.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(40);
    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             the check has no oracle. Trace: {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    Ok((session, pointer))
}

/// Click region `name`, then type `text` into the viewport that declared it.
pub fn type_into(
    session: &Session,
    pointer: &ScriptedPointer,
    name: &str,
    text: &str,
) -> Result<()> {
    click(session, pointer, name)?;
    let viewport = declared_in(&session.trace()?, UI_RECT, name).and_then(|(_, vp)| vp);
    pointer.type_text(session, viewport.as_deref(), text)?;
    session.settle(10);
    Ok(())
}

/// Wheel the scroll area `body` until region `name` lies wholly inside it,
/// then click it; at most `notches` notches, measured again after the last.
pub fn click_scrolled(
    session: &Session,
    pointer: &ScriptedPointer,
    (name, body): (&str, &str),
    notches: usize,
    report: &mut crate::report::CheckReport,
) -> Result<()> {
    for attempt in 0..=notches {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
            Error::new(format!(
                "no `{name}` region to click. Declared beside it: {}.",
                list(&declared_names(
                    &trace,
                    UI_RECT,
                    name.split('.').next().unwrap_or(name)
                ))
            ))
        })?;
        let window = crate::checks::driving::declared(&trace, UI_RECT, body).ok_or_else(|| {
            Error::new(format!("no `{body}` region to measure `{name}` against."))
        })?;
        if window.contains_rect(rect) {
            if attempt > 0 {
                report.note(format!(
                    "`{name}` was below `{body}`'s fold; {attempt} scroll notch(es) brought it \
                     into view"
                ));
            }
            pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
            session.settle(18);
            return Ok(());
        }
        if attempt == notches {
            break;
        }
        pointer.wheel_in(
            session,
            viewport.as_deref(),
            WindowPoint::centre_of(window),
            -1.0,
        )?;
        session.settle(12);
    }
    Err(Error::new(format!(
        "`{name}` is declared and never came wholly inside `{body}` after {notches} scroll \
         notch(es), so a click at its centre would be clipped out of the scroll area. A control \
         in a window's footer is never inside the body and is clicked with `click`."
    )))
}

/// `comments_census::refresh` through any [`crate::input::Click`]: read a
/// census published after `after`, bringing the Comments panel to the front
/// first if it has gone quiet (its dock tab, then Markup > Comments).
pub fn census_refresh(
    session: &Session,
    pointer: &impl crate::input::Click,
    ui_rect: &str,
    after: usize,
    report: &mut crate::report::CheckReport,
) -> Result<Option<Census>> {
    use crate::checks::driving::{ITEM_PREFIX, TAB_EVENT, declared, shell_trace};
    const DOCK_TAB: &str = "dock.tab.markup.comments"; // ui-text-exempt: a trace region name, never displayed
    const COMMAND: &str = "ribbon.item.markup.comments"; // ui-text-exempt: a trace region name, never displayed
    const MARKUP_TAB: &str = "ribbon.tab.markup"; // ui-text-exempt: a trace region name, never displayed
    if let Some(census) = Census::since(&session.trace()?, after)? {
        return Ok(Some(census));
    }
    report.note(
        "the Comments panel published no census, so it is not the front tab of its stack; \
         bringing it forward before reading",
    );
    if let Some(rect) = declared(&session.trace()?, ui_rect, DOCK_TAB) {
        pointer.click_rect(session, rect)?;
        session.settle(14);
        if let Some(census) = Census::since(&session.trace()?, after)? {
            report.note(format!("the panel came forward on a click of `{DOCK_TAB}`"));
            return Ok(Some(census));
        }
    }
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, MARKUP_TAB) else {
        report.note(format!(
            "no `{MARKUP_TAB}` region either. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ));
        return Ok(None);
    };
    pointer.click_rect(session, tab)?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .all(|l| l.get("tab") != Some("markup"))
    {
        report.note(format!(
            "the click on `{MARKUP_TAB}` produced no `{TAB_EVENT} tab=markup`."
        ));
        return Ok(None);
    }
    let trace = session.trace()?;
    let Some(rect) = declared(&trace, ui_rect, COMMAND) else {
        report.note(format!(
            "the Markup tab is active and declares no `{COMMAND}`. Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        ));
        return Ok(None);
    };
    pointer.click_rect(session, rect)?;
    session.settle(18);
    let census = Census::since(&session.trace()?, after)?;
    if census.is_some() {
        report.note("the panel came forward on a press of `markup.comments`");
    }
    Ok(census)
}

/// `comments_census::baseline` through any [`crate::input::Click`]: enter
/// `mode`, bring the Comments panel forward, and require a clean census.
pub fn census_baseline(
    session: &Session,
    pointer: &impl crate::input::Click,
    ui_rect: &str,
    (mode, what, subject): (&str, &str, &str),
    report: &mut crate::report::CheckReport,
) -> Result<Census> {
    let before_click = session.trace()?.mark();
    crate::checks::driving::click_mode_segment(session, pointer, ui_rect, mode)?;
    session.settle(16);
    let anchor = session
        .trace()?
        .events("mode-changed")
        .filter(|l| l.get("to") == Some(mode))
        .last()
        .map_or(before_click, |l| l.lineno);
    let census = census_refresh(session, pointer, ui_rect, anchor, report)?.ok_or_else(|| {
        Error::new(format!(
            "the Comments panel traced no `{}` line for {what} after the switch to `{mode}`, and \
             neither its dock tab nor its ribbon control could raise it, so there is no oracle. \
             A persisted layout can name a different active tab. Trace: {}.",
            crate::checks::comments_census::EVENT,
            session.trace_path().display()
        ))
    })?;
    census.require_a_clean_fixture(what, subject)?;
    report.note(format!(
        "{what}: the Comments panel lists {} annotation(s) — `{}`",
        census.listed, census.raw
    ));
    Ok(census)
}
