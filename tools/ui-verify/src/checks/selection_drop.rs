//! `a_selection_dropped_on_another_window_is_copied_there` — two pdfcer-gui
//! windows stacked off the desktop. A's selection is dragged off its canvas
//! and released over B: B pastes the same objects and A keeps them where they
//! were. Dragged again with Shift held, B pastes them and A cuts them. The
//! clipboard goes to a capture folder, so the operator's own is never touched.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/selection_drop.md`.

use super::cross_window_paste::{CAPTURE_ENV, click, to_edit_tab};
use super::tab_drag::{BELOW, other_centre};
use super::window_move::{OFFSCREEN, Window, await_line, launch_with_env, peers_reach};
use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::trace::TraceLine;

const SELECT_ALL: &str = "ribbon.item.edit.select_all"; // ui-text-exempt: a trace region name
const MODE: &str = "edit"; // ui-text-exempt: a ribbon mode id
const DROPPED: &str = "window-selection-dropped"; // ui-text-exempt: a trace event name
const COPIED: &str = "clipboard-copy"; // ui-text-exempt: a trace event name
const CUT: &str = "clipboard-cut"; // ui-text-exempt: a trace event name
const RECEIVED: &str = "window-paste-received"; // ui-text-exempt: a trace event name
const ADOPTED: &str = "clip-adopted"; // ui-text-exempt: a trace event name
const APPLIED: &str = "paste-objects-applied"; // ui-text-exempt: a trace event name
/// Every committed move traces an event with this prefix.
const MOVE_PREFIX: &str = "move-"; // ui-text-exempt: a trace event prefix
/// `pure-k-square.pdf`'s page is 200 points square; its square spans 50–150.
const SQUARE_CENTRE: (f64, f64) = (100.0, 100.0);
const PAGE_SIDE: f64 = 200.0;

/// See the module documentation.
pub struct ASelectionDroppedOnAnotherWindowIsCopiedThere;

impl Check for ASelectionDroppedOnAnotherWindowIsCopiedThere {
    fn name(&self) -> &'static str {
        "a_selection_dropped_on_another_window_is_copied_there"
    }

    fn defect(&self) -> &'static str {
        "A selection dragged onto another pdfcer-gui window does not arrive there, arrives as \
         something other than the objects dragged, or moves on its own page instead"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let dir = ctx.out("selection-drop-capture");
    let _ = std::fs::remove_dir_all(&dir);
    if dir.exists() {
        return Err(Error::new(format!("cannot clear {}.", dir.display())));
    }
    let capture = dir.display().to_string();
    let env = [(CAPTURE_ENV, capture.as_str())];
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let a = launch_with_env(
        ctx,
        report,
        "pure-k-square.pdf",
        "sel-a",
        OFFSCREEN,
        &[],
        &env,
    )?;
    let b = launch_with_env(ctx, report, "four-pages.pdf", "sel-b", BELOW, &[], &env)?;
    for w in [&a, &b] {
        if let Some(failure) = peers_reach(w, 1)? {
            return Ok(Some(failure));
        }
    }
    driving::click_mode_segment(&b.session, &b.pointer, ui_rect, MODE)?;
    to_edit_tab(&a.session, &a.pointer, ui_rect)?;
    click(&a.session, &a.pointer, ui_rect, SELECT_ALL)?;
    let pdf = driving::repo_fixture("pure-k-square.pdf", "It is checked in.")?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    if (page.width_pt, page.height_pt) != (PAGE_SIDE, PAGE_SIDE) {
        return Err(Error::new(format!(
            "pure-k-square.pdf is no longer {PAGE_SIDE} points square: {} by {}.",
            page.width_pt, page.height_pt
        )));
    }
    let mapping = CanvasMapping::from_trace(&a.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let from = mapping.doc_to_window(DocPoint::new(0, SQUARE_CENTRE.0, SQUARE_CENTRE.1))?;
    let Some((to, _)) = other_centre(&a, &b)? else {
        return Ok(Some(
            "a window published no `window-rect` line, so nothing can be dropped on it.".to_owned(),
        ));
    };
    if let Some(failure) = drop(report, &a, &b, (from, to), false)? {
        return Ok(Some(failure));
    }
    drop(report, &a, &b, (from, to), true)
}

/// Drag A's selection from `from` to B's centre `to`, with Shift held when
/// `shift`, and judge both windows.
fn drop(
    report: &mut CheckReport,
    a: &Window,
    b: &Window,
    (from, to): (WindowPoint, WindowPoint),
    shift: bool,
) -> Result<Option<String>> {
    let (a_mark, b_mark) = (a.session.trace()?.mark(), b.session.trace()?.mark());
    let mods = if shift { " mods=shift" } else { "" };
    a.pointer.send(
        &a.session,
        &format!(
            "drag {:.1} {:.1} {:.1} {:.1} steps=12{mods}",
            from.x(),
            from.y(),
            to.x(),
            to.y()
        ),
    )?;
    let which = if shift { "Shift-drop" } else { "drop" };
    let b_pid = b.session.pid().to_string();
    let dropped = after(&a.session, DROPPED, a_mark)?;
    report.note(format!(
        "A's {which}: {:?}",
        dropped.as_ref().map(|l| &l.raw)
    ));
    if dropped.as_ref().and_then(|l| l.get("onto")) != Some(b_pid.as_str()) {
        return Ok(Some(format!(
            "A's selection, released at B's centre ({to:?} in A's points), was not dropped onto \
             B: {:?}. Trace: {}.",
            dropped.map(|l| l.raw),
            a.session.trace_path().display()
        )));
    }
    let Some(copied) = after(&a.session, COPIED, a_mark)? else {
        return Ok(Some(format!("A traced no `{COPIED}` for the {which}.")));
    };
    let objects = copied.get("objects").map(str::to_owned);
    // The capture folder leaves the system clipboard's sequence where it was,
    // so B adopts A's clip once, on the first drop, and pastes its own copy of
    // it after that.
    let names: &[&str] = if shift {
        &[RECEIVED, APPLIED]
    } else {
        &[RECEIVED, ADOPTED, APPLIED]
    };
    for &name in names {
        let Some(line) = after(&b.session, name, b_mark)? else {
            return Ok(Some(format!(
                "B traced no `{name}` after A's {which}. Trace: {}.",
                b.session.trace_path().display()
            )));
        };
        report.note(format!("B: `{}`", line.raw));
        let count = if name == APPLIED { "pasted" } else { "objects" };
        if name != RECEIVED && line.get(count) != objects.as_deref() {
            return Ok(Some(format!(
                "A copied `{}` and B traced `{}`; the object counts must agree.",
                copied.raw, line.raw
            )));
        }
    }
    let trace = a.session.trace()?;
    if let Some(moved) = trace
        .lines
        .iter()
        .find(|l| l.lineno > a_mark && l.event.starts_with(MOVE_PREFIX))
    {
        return Ok(Some(format!(
            "A's {which} onto B also moved the selection on A's own page: `{}`.",
            moved.raw
        )));
    }
    let cut = after(&a.session, CUT, a_mark)?;
    if cut.is_some() != shift {
        return Ok(Some(format!(
            "the {which} {} A's selection: {:?}. A drop copies; Shift moves.",
            if shift { "did not cut" } else { "cut" },
            cut.map(|l| l.raw)
        )));
    }
    Ok(None)
}

/// The last `name` line after `mark`, waiting for it.
fn after(session: &crate::launch::Session, name: &str, mark: usize) -> Result<Option<TraceLine>> {
    await_line(session, name, |l| l.lineno > mark)
}
