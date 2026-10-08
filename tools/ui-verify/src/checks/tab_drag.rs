//! `a_tab_dragged_out_moves_its_document` — a document tab dragged off the
//! strip, driven across two pdfcer-gui windows stacked off the desktop: A's
//! only tab, showing its last page, is dropped on B and arrives there at that
//! page while A closes; B's new tab is then dropped on B's own canvas and
//! tears off into a window of its own, started at that page; and a window
//! started with `--page` shows that page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/tab_drag.md`.

use super::window_move::{
    EMPTIED, OFFSCREEN, RECEIVED, Reaper, TABS, TORN_OFF, Window, await_line, exits, launch_at,
    names, peers_reach, torn_off_opened,
};
use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::trace::TraceLine;

/// B sits 1,400 pixels below A: off the desktop, and clear of it.
pub(super) const BELOW: &str = "-4200,-2800,1400,900";
const DROPPED: &str = "window-tab-dropped"; // ui-text-exempt: a trace event name
const RECT: &str = "window-rect"; // ui-text-exempt: a trace event name
const INNER: &str = "window-inner"; // ui-text-exempt: a trace event name
const CANVAS: &str = "canvas"; // ui-text-exempt: a trace event name
const TAB_PREFIX: &str = "doc-tab."; // ui-text-exempt: a trace region name
/// `four-pages.pdf`'s last page, zero-based; `End` goes there.
const LAST_PAGE: &str = "3";

/// See the module documentation.
pub struct ATabDraggedOutMovesItsDocument;

impl Check for ATabDraggedOutMovesItsDocument {
    fn name(&self) -> &'static str {
        "a_tab_dragged_out_moves_its_document"
    }

    fn defect(&self) -> &'static str {
        "A document tab dragged off the strip does nothing, lands in the wrong window, or \
         reopens at its first page instead of the page it was showing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut torn = Reaper(None);
        match assess(ctx, &mut report, &mut torn) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(
    ctx: &CheckContext,
    report: &mut CheckReport,
    torn: &mut Reaper,
) -> Result<Option<String>> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let a = launch_at(ctx, report, "four-pages.pdf", "drag-a", OFFSCREEN, &[])?;
    let b = launch_at(ctx, report, "pure-k-square.pdf", "drag-b", BELOW, &[])?;
    for w in [&a, &b] {
        if let Some(failure) = peers_reach(w, 1)? {
            return Ok(Some(failure));
        }
    }
    a.pointer.key(&a.session, None, "End", None)?;
    if let Some(last) = shows_last_page(&a)? {
        return Ok(Some(format!(
            "End did not keep the first window at its last page; its canvas last said {last:?}."
        )));
    }
    if let Some(failure) = drop_onto_other(&a, &b, ui_rect, report)? {
        return Ok(Some(failure));
    }
    let Some(line) = drop_onto_own_canvas(&b, ui_rect, report)? else {
        return Ok(Some(format!(
            "B's tab dropped on its own canvas traced no `{TORN_OFF}` line. Trace: {}.",
            b.session.trace_path().display()
        )));
    };
    let Some(pid) = line.get("pid").and_then(|p| p.parse().ok()) else {
        return Ok(Some(format!(
            "the tear-off named no window: `{}`.",
            line.raw
        )));
    };
    torn.0 = Some(pid);
    if line.get("page") != Some(LAST_PAGE) {
        return Ok(Some(format!(
            "the torn-off window was started at another page than B showed ({LAST_PAGE}): \
             `{}`.",
            line.raw
        )));
    }
    if let Some(failure) = torn_off_opened(ctx, pid, &a.doc)? {
        return Ok(Some(failure));
    }
    started_at_page(ctx, report)
}

/// A's only tab is dragged into B's client area: B opens it at the page A
/// showed, and A closes.
fn drop_onto_other(
    a: &Window,
    b: &Window,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let Some((target, to)) = other_centre(a, b)? else {
        return Ok(Some(format!(
            "a window published no `{RECT}` line, so nothing can be dropped on it."
        )));
    };
    let tab = tab_rect(a, ui_rect, 0)?;
    a.session.expect_exit();
    a.pointer
        .drag(&a.session, WindowPoint::centre_of(tab), target, 12)?;
    let dropped = await_line(&a.session, DROPPED, |_| true)?;
    report.note(format!("A's drop: {:?}", dropped.as_ref().map(|l| &l.raw)));
    let b_pid = b.session.pid().to_string();
    if dropped.as_ref().and_then(|l| l.get("onto")) != Some(b_pid.as_str()) {
        return Ok(Some(format!(
            "A's tab, dropped at B's centre ({target:?} in A's points; B is {to:?} on the \
             desktop), was not dropped onto B: {:?}.",
            dropped.map(|l| l.raw)
        )));
    }
    let Some(received) = await_line(&b.session, RECEIVED, |_| true)? else {
        return Ok(Some(format!("B traced no `{RECEIVED}` line.")));
    };
    if !names(&received, &a.doc) || received.get("page") != Some(LAST_PAGE) {
        return Ok(Some(format!(
            "B received `{}`, not A's document at page {LAST_PAGE}.",
            received.raw
        )));
    }
    if let Some(last) = shows_last_page(b)? {
        return Ok(Some(format!(
            "B received page {LAST_PAGE} and its canvas did not stay there; it last said \
             {last:?}."
        )));
    }
    if await_line(&b.session, TABS, |l| l.get("open") == Some("2"))?.is_none() {
        return Ok(Some("B never showed two document tabs.".to_owned()));
    }
    if await_line(&a.session, EMPTIED, |_| true)?.is_none() || !exits(&a.session) {
        return Ok(Some(
            "A gave its only tab away and did not close.".to_owned(),
        ));
    }
    Ok(None)
}

/// B's centre in A's points, and B's client area in desktop pixels, from both
/// windows' `window-rect` lines and A's `window-inner ppp=`.
pub(super) fn other_centre(a: &Window, b: &Window) -> Result<Option<(WindowPoint, [i32; 4])>> {
    let (Some(from), Some(to)) = (rect_px(a)?, rect_px(b)?) else {
        return Ok(None);
    };
    let ppp = a
        .session
        .trace()?
        .last(INNER)
        .and_then(|l| l.get("ppp")?.parse::<f32>().ok())
        .unwrap_or(1.0);
    let target = point(
        ((to[0] + to[2]) as f32 / 2.0 - from[0] as f32) / ppp,
        ((to[1] + to[3]) as f32 / 2.0 - from[1] as f32) / ppp,
    );
    Ok(Some((target, to)))
}

/// B's second tab is dropped well below the strip, on B's own canvas. B's
/// tear-off line, when it traced one.
fn drop_onto_own_canvas(
    b: &Window,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<TraceLine>> {
    let tab = tab_rect(b, ui_rect, 1)?;
    let from = WindowPoint::centre_of(tab);
    b.pointer
        .drag(&b.session, from, point(from.x(), from.y() + 400.0), 12)?;
    let dropped = await_line(&b.session, DROPPED, |_| true)?;
    report.note(format!("B's drop: {:?}", dropped.map(|l| l.raw)));
    let Some(line) = await_line(&b.session, TORN_OFF, |_| true)? else {
        return Ok(None);
    };
    report.note(format!("tear-off: `{}`", line.raw));
    Ok(Some(line))
}

/// `None` when the window's canvas reached the last page and is still there
/// after settling, so a page reached and then lost to a later placement
/// fails; otherwise the canvas line it last traced.
fn shows_last_page(w: &Window) -> Result<Option<Option<String>>> {
    let reached = await_line(&w.session, CANVAS, |l| l.get("page") == Some(LAST_PAGE))?;
    w.session.settle(30);
    let last = w.session.trace()?.last(CANVAS).cloned();
    if reached.is_some() && last.as_ref().and_then(|l| l.get("page")) == Some(LAST_PAGE) {
        return Ok(None);
    }
    Ok(Some(last.map(|l| l.raw)))
}

/// A window started with `--page 4` shows the fourth page.
fn started_at_page(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let c = launch_at(
        ctx,
        report,
        "four-pages.pdf",
        "drag-c",
        OFFSCREEN,
        &["--page", "4"], // ui-text-exempt: a command-line flag
    )?;
    if let Some(last) = shows_last_page(&c)? {
        return Ok(Some(format!(
            "a window started with `--page 4` did not show page index {LAST_PAGE}; its canvas \
             last said {last:?}."
        )));
    }
    Ok(None)
}

/// The window's published client area, desktop pixels.
fn rect_px(w: &Window) -> Result<Option<[i32; 4]>> {
    let Some(line) = await_line(&w.session, RECT, |_| true)? else {
        return Ok(None);
    };
    let n: Vec<i32> = line
        .get("rect")
        .unwrap_or_default()
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    Ok(<[i32; 4]>::try_from(n).ok())
}

fn tab_rect(w: &Window, ui_rect: &str, slot: usize) -> Result<LRect> {
    w.session.settle(10);
    let trace = w.session.trace()?;
    let name = format!("{TAB_PREFIX}{slot}");
    driving::declared(&trace, ui_rect, &name).ok_or_else(|| {
        Error::new(format!(
            "window {} declares no `{name}`; tabs declared: {}.",
            w.session.pid(),
            driving::list(&driving::declared_names(&trace, ui_rect, TAB_PREFIX))
        ))
    })
}

fn point(x: f32, y: f32) -> WindowPoint {
    WindowPoint::centre_of(LRect {
        min: Pt::new(x, y),
        max: Pt::new(x, y),
    })
}
