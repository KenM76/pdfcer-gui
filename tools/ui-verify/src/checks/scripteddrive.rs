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

/// A [`crate::input::Click`] that also clicks a named region in whichever
/// viewport declared it, so a helper shared by OS-driven and scripted checks
/// reaches a dialog's controls either way.
pub trait ClickNamed: crate::input::Click {
    /// Click the centre of `rect`, the region `name` as `trace` last declared
    /// it, in that region's own viewport.
    fn click_named(
        &self,
        session: &Session,
        trace: &crate::trace::Trace,
        ui_rect: &str,
        name: &str,
        rect: LRect,
    ) -> Result<()>;
}

impl ClickNamed for crate::input::Driver {
    fn click_named(
        &self,
        session: &Session,
        trace: &crate::trace::Trace,
        ui_rect: &str,
        name: &str,
        rect: LRect,
    ) -> Result<()> {
        let frame = crate::checks::driving::frame_of(session, trace, ui_rect, name)?;
        self.click_at(frame.declared_center(rect))
    }
}

impl ClickNamed for ScriptedPointer {
    fn click_named(
        &self,
        session: &Session,
        trace: &crate::trace::Trace,
        ui_rect: &str,
        name: &str,
        rect: LRect,
    ) -> Result<()> {
        let viewport = declared_in(trace, ui_rect, name).and_then(|(_, vp)| vp);
        self.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))
            .map(|_| ())
    }
}

/// Open the Pages panel from the View tab, wherever the responsive band has
/// put its item: the scripted twin of `pages_drag::open_pages_panel`.
pub fn open_pages_panel(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<()> {
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
    let Some(item) =
        declared_or_in_overflow(session, pointer, ui_rect, "ribbon.item.view.panel_pages")?
    else {
        return Err(Error::new(
            "the View tab declares no `ribbon.item.view.panel_pages`, on the band or in the \
             overflow, so the Pages panel cannot be opened.",
        ));
    };
    pointer.click(session, WindowPoint::centre_of(item))?;
    session.settle(20);
    Ok(())
}

/// Convert the document point `at` to a window point on the canvas: the
/// scripted twin of `text_selection::aim`, refusing a point on a page the
/// canvas is not showing.
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
            "the canvas is showing page {}, and this check's point is on page {}. Converting a \
             document point against another page's rect would put it somewhere plausible and \
             wrong.",
            shown.map_or_else(|| "an unreported index".to_owned(), |p| (p + 1).to_string()),
            at.page + 1
        )));
    }
    let mapping =
        crate::coords::CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, at.page)?;
    mapping.doc_to_window(at)
}

/// The canvas object menu's line on every frame carrying a secondary click.
const MENU_EVENT: &str = "canvas-menu"; // ui-text-exempt: a trace event name
/// The context the object menu declares.
const OBJECT_CONTEXT: &str = "canvas.object"; // ui-text-exempt: a trace token
/// The object menu's Redact selection row.
const REDACT_ROW: &str = "menu.item.canvas.object.edit.redact_selection"; // ui-text-exempt: a region name

/// Right-click at `at`, press the redact row, and read what it requested: the
/// scripted twin of `redact_menu::mark_through_the_menu`, whose failure arms
/// and their readings it shares. `Ok(Err(_))` is a failure message.
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
    let Some(menu) = trace.events(MENU_EVENT).last() else {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED NO MENU AT ALL: no `{MENU_EVENT}` line after a \
             secondary click on the page, so the click never reached the canvas response. Trace: \
             {}",
            session.trace_path().display()
        )));
    };
    if menu.get("context") != Some(OBJECT_CONTEXT) {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} DID NOT RESOLVE `{OBJECT_CONTEXT}`: `{}`. The secondary \
             hit test lost the selection the left click made. Trace: {}",
            menu.raw,
            session.trace_path().display()
        )));
    }
    let Some((row, viewport)) = declared_in(&trace, ui_rect, REDACT_ROW) else {
        return Ok(Err(format!(
            "★★★ THE REDACT ROW IS NOT IN THE CANVAS OBJECT MENU: no `{REDACT_ROW}` region after \
             the menu opened on {what}. Rows it DID publish: {}. Trace: {}",
            list(&declared_names(&trace, ui_rect, "menu.item.canvas.object.")),
            session.trace_path().display()
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
    read_marked(session, mark, what)
}

/// The `redact-mark-selection-requested` line after `mark`, as a
/// [`crate::checks::redact_menu::Marked`]. `Ok(Err(_))` is a failure message.
fn read_marked(
    session: &Session,
    mark: usize,
    what: &str,
) -> Result<std::result::Result<crate::checks::redact_menu::Marked, String>> {
    use crate::checks::redact_menu::{Bounds, Marked, REQUESTED};
    let trace = session.trace()?;
    let Some(line) = trace.last_after(REQUESTED, mark) else {
        return Ok(Err(format!(
            "★★★ THE ROW WAS PRESSED AND THE VERB DID NOT RUN: no `{REQUESTED}` line after \
             pressing the redact row on {what}. Grep for `redact-mark-selection-declined`. \
             Trace: {}",
            session.trace_path().display()
        )));
    };
    let Some(bbox) = line.get("bbox").and_then(Bounds::parse) else {
        return Ok(Err(format!(
            "★★★ THE VERB RAN AND DID NOT SAY WHAT IT MARKED: `{}` carries no rectangle in \
             `bbox=`.",
            line.raw
        )));
    };
    let Some(quads) = line.get_usize("quads") else {
        return Ok(Err(format!(
            "★★★ THE VERB RAN AND DID NOT SAY HOW MANY REGIONS IT BUILT: `{}` carries no usable \
             `quads=` field.",
            line.raw
        )));
    };
    Ok(Ok(Marked { quads, bbox }))
}

/// Press the View tab's text-chunk switch and confirm it went to `want`: the
/// scripted twin of `text_chunks::press_the_toggle`. `Ok(Some(_))` is a
/// failure message.
pub fn press_the_chunk_toggle(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    want: bool,
    report: &mut crate::report::CheckReport,
) -> Result<Option<String>> {
    use crate::checks::driving::{
        INVOKE_EVENT, TAB_EVENT, VIEW_TAB, declared, declared_or_in_overflow,
    };
    use crate::checks::text_chunks::{SWITCH_EVENT, TOGGLE_ID, TOGGLE_REGION, toggle_invokes};
    if declared(&session.trace()?, ui_rect, TOGGLE_REGION).is_none() {
        let Some(tab) = declared(&session.trace()?, ui_rect, VIEW_TAB.0) else {
            return Ok(Some(format!(
                "no `{}` region to open the toggle's tab.",
                VIEW_TAB.0
            )));
        };
        pointer.click(session, WindowPoint::centre_of(tab))?;
        session.settle(14);
        if !crate::checks::driving::shell_trace(session)?
            .events(TAB_EVENT)
            .any(|l| l.get("tab") == Some(VIEW_TAB.1))
        {
            return Ok(Some(format!(
                "the click on `{}` produced no `{TAB_EVENT} tab={}` line.",
                VIEW_TAB.0, VIEW_TAB.1
            )));
        }
    }
    let Some(item) = declared_or_in_overflow(session, pointer, ui_rect, TOGGLE_REGION)? else {
        return Ok(Some(format!(
            "★★★ THE SWITCH IS REGISTERED AND UNREACHABLE: the View tab declares no \
             `{TOGGLE_REGION}`, on the band, in a collapsed group or in the overflow."
        )));
    };
    let invokes_before = toggle_invokes(session)?;
    let mark = session.trace()?.mark();
    pointer.click(session, WindowPoint::centre_of(item))?;
    session.settle(22);
    if toggle_invokes(session)? <= invokes_before {
        return Ok(Some(format!(
            "★★ THE CONTROL DID NOT TAKE THE CLICK: no new `{INVOKE_EVENT} \
             id={TOGGLE_ID}` after a click on `{TOGGLE_REGION}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let wanted = if want { "true" } else { "false" };
    let trace = session.trace()?;
    match trace.last_after(SWITCH_EVENT, mark) {
        Some(l) if l.get("enabled") == Some(wanted) => {
            report.note(format!("★ the toggle was pressed: `{}`", l.raw));
            Ok(None)
        }
        other => Ok(Some(format!(
            "★★★ THE SWITCH DID NOT GO TO `enabled={wanted}`: {}. Trace: {}.",
            other.map_or_else(
                || format!("no `{SWITCH_EVENT}` line followed"),
                |l| format!("`{}`", l.raw)
            ),
            session.trace_path().display()
        ))),
    }
}

/// The Comments panel's dock tab, its ribbon tab and its show command.
const COMMENTS_DOCK_TAB: &str = "dock.tab.markup.comments"; // ui-text-exempt: a region name
const COMMENTS_RIBBON_TAB: &str = "ribbon.tab.markup"; // ui-text-exempt: a region name
const COMMENTS_COMMAND: &str = "ribbon.item.markup.comments"; // ui-text-exempt: a region name

/// Read a Comments census published after `after`, bringing the panel forward
/// by its dock tab and then its show command if it has gone quiet: the
/// `&impl Click` twin of `comments_census::refresh`.
pub fn comments_refresh(
    session: &Session,
    driver: &impl crate::input::Click,
    ui_rect: &str,
    after: usize,
    report: &mut crate::report::CheckReport,
) -> Result<Option<crate::checks::comments_census::Census>> {
    use crate::checks::comments_census::Census;
    use crate::checks::driving::declared;
    if let Some(census) = Census::since(&session.trace()?, after)? {
        return Ok(Some(census));
    }
    report.note(
        "the Comments panel published no census — a dock draws only its active tab. Bringing it \
         forward before reading."
            .to_owned(),
    );
    for (region, settle) in [
        (COMMENTS_DOCK_TAB, 14),
        (COMMENTS_RIBBON_TAB, 14),
        (COMMENTS_COMMAND, 18),
    ] {
        let Some(rect) = declared(&session.trace()?, ui_rect, region) else {
            continue;
        };
        driver.click_rect(session, rect)?;
        session.settle(settle);
        if let Some(census) = Census::since(&session.trace()?, after)? {
            report.note(format!("the panel came forward on a click of `{region}`"));
            return Ok(Some(census));
        }
    }
    Ok(None)
}

/// Enter `mode`, bring the Comments panel forward and report its census: the
/// `&impl Click` twin of `comments_census::baseline`, anchored on the mode
/// change when one is traced.
pub fn comments_baseline(
    session: &Session,
    driver: &impl crate::input::Click,
    ui_rect: &str,
    mode: &str,
    (what, subject): (&str, &str),
    report: &mut crate::report::CheckReport,
) -> Result<crate::checks::comments_census::Census> {
    let before_click = session.trace()?.mark();
    crate::checks::driving::click_mode_segment(session, driver, ui_rect, mode)?;
    session.settle(16);
    let anchor = session
        .trace()?
        .events("mode-changed") // ui-text-exempt: a trace event name
        .filter(|l| l.get("to") == Some(mode))
        .last()
        .map_or(before_click, |l| l.lineno);
    let census = comments_refresh(session, driver, ui_rect, anchor, report)?.ok_or_else(|| {
        Error::new(format!(
            "the Comments panel traced no `{}` line for {what} after the switch to `{mode}`, and \
             neither its dock tab nor its ribbon control could raise it. Clear `userdata/` and \
             run again. Trace: {}.",
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
