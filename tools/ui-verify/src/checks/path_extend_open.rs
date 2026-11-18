//! Format ▸ Nodes ▸ Continue path and Open path here, driven on the real
//! binary: an open path's end node is drawn on by two clicks and Enter, the
//! saved file holds the two new segments, and undo takes them back; a closed
//! square opened at a node saves with no close operator, and undo closes it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/path_extend_open.md`.

use super::dimdrive::{Fixture, click_on, press_on_tab, run_on, ui_rect_event};
use super::invisible_text_scripted::streams;
use super::nodeshape::last_usize;
use crate::checks::driving::declared;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit";
const APPLIED: &str = "path-end-applied"; // ui-text-exempt: a trace event name, never displayed
const ARMED: &str = "path-extend-armed"; // ui-text-exempt: a trace event name, never displayed
const POINT: &str = "path-extend-point"; // ui-text-exempt: a trace event name, never displayed
const FINISH: &str = "path-extend-finish"; // ui-text-exempt: a trace event name, never displayed
const ANCHORS: &str = "canvas-anchors"; // ui-text-exempt: a trace event name, never displayed
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const UNDONE: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
const FORMAT_TAB: &str = "ribbon.tab.format";
const CONTINUE: &str = "ribbon.item.format.path_continue";
const OPEN: &str = "ribbon.item.format.path_open";
const PAGE: PageGeometry = PageGeometry {
    width_pt: 400.0,
    height_pt: 300.0,
};
const OPEN_PATH: Fixture = Fixture {
    file: "line-and-curve.pdf",
    method: "Rebuild it with `python fixtures/line-and-curve.PROVENANCE.py`.",
    page: PAGE,
};
const SQUARE: Fixture = Fixture {
    file: "closed-square.pdf",
    method: "Rebuild it with `python fixtures/closed-square.PROVENANCE.py`.",
    page: PAGE,
};
/// On the first straight segment of both fixtures, page space.
const ON_THE_LINE: (f64, f64) = (175.0, 100.0);
/// The open path's last node, `canvas.anchor.2` at the Part rung.
const OPEN_END: &str = "canvas.anchor.2";
/// The square's second corner, (250,100).
const CORNER: &str = "canvas.anchor.1";
/// Where Continue path clicks, page space, clear of the path.
const CLICKS: [(f64, f64); 2] = [(330.0, 250.0), (150.0, 260.0)];

/// Continue path, from the open path's last node.
pub struct APathIsContinuedFromItsEnd;

impl Check for APathIsContinuedFromItsEnd {
    fn name(&self) -> &'static str {
        "a_path_is_continued_from_its_end"
    }

    fn defect(&self) -> &'static str {
        "Continue path is missing, reaches no engine verb, writes no segment to the saved file, \
         or survives an undo"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &OPEN_PATH, INVOKE, "pathextend", drive_extend)
    }
}

/// Open path here, at a corner of the closed square.
pub struct AClosedPathIsOpenedAtANode;

impl Check for AClosedPathIsOpenedAtANode {
    fn name(&self) -> &'static str {
        "a_closed_path_is_opened_at_a_node"
    }

    fn defect(&self) -> &'static str {
        "Open path here is missing, reaches no engine verb, saves the subpath still closed, \
         or survives an undo"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &SQUARE, INVOKE, "pathopen", drive_open)
    }
}

fn drive_extend(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    enter_node(ctx, report, session, pointer, &OPEN_PATH, OPEN_END)?;
    let before = last_usize(session, ANCHORS, "total", 0)?;
    let mark = session.trace()?.mark();
    if let Some(failure) = press_command(ctx, session, pointer, CONTINUE)? {
        return Ok(Some(failure));
    }
    if session.trace()?.last_after(ARMED, mark).is_none() {
        return Ok(Some(format!(
            "Continue path armed no run (no `{ARMED}`). Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
    for (x, y) in CLICKS {
        pointer.click(session, mapping.doc_to_window(DocPoint::new(0, x, y))?)?;
        session.settle(20);
    }
    let points = session
        .trace()?
        .events(POINT)
        .filter(|l| l.lineno > mark)
        .count();
    pointer.key(session, None, "Enter", None)?;
    session.settle(40);
    let trace = session.trace()?;
    let finish = trace.last_after(FINISH, mark).map(|l| l.raw.clone());
    report.note(format!(
        "{points} click(s) taken; finish: `{}`",
        finish.clone().unwrap_or_default()
    ));
    if let Some(failure) = applied(session, report, mark, "extend")? {
        return Ok(Some(failure));
    }
    let after = last_usize(session, ANCHORS, "total", mark)?;
    if after != before + CLICKS.len() {
        return Ok(Some(format!(
            "the path was continued by {} clicks and shows {after} anchors; it had {before}. \
             Trace: {}.",
            CLICKS.len(),
            session.trace_path().display()
        )));
    }
    if let Some(failure) = save(session, pointer)? {
        return Ok(Some(failure));
    }
    let lines = operators(&ctx.out("pathextend.pdf"), " m", " l")?;
    report.note(format!("saved path holds {lines} line operator(s)"));
    if lines != 1 + CLICKS.len() {
        return Ok(Some(format!(
            "★★★ the saved path holds {lines} `l` operator(s); the fixture's one plus {} \
             continued segments make {}: the edit did not reach the file.",
            CLICKS.len(),
            1 + CLICKS.len()
        )));
    }
    undo_restores(session, pointer, report, before)
}

fn drive_open(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    enter_node(ctx, report, session, pointer, &SQUARE, CORNER)?;
    let before = last_usize(session, ANCHORS, "total", 0)?;
    let mark = session.trace()?.mark();
    if let Some(failure) = press_command(ctx, session, pointer, OPEN)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = applied(session, report, mark, "open")? {
        return Ok(Some(failure));
    }
    if let Some(failure) = save(session, pointer)? {
        return Ok(Some(failure));
    }
    let closes = operators(&ctx.out("pathopen.pdf"), " m", " h")?;
    report.note(format!("saved square holds {closes} close operator(s)"));
    if closes != 0 {
        return Ok(Some(format!(
            "★★★ the opened square still saves {closes} `h` operator(s): the subpath is \
             closed in the file."
        )));
    }
    if let Some(failure) = undo_restores(session, pointer, report, before)? {
        return Ok(Some(failure));
    }
    // The anchor count alone cannot tell open from closed, so undo is read back too.
    if let Some(failure) = save(session, pointer)? {
        return Ok(Some(failure));
    }
    let closes = operators(&ctx.out("pathopen.pdf"), " m", " h")?;
    report.note(format!(
        "after undo the saved square holds {closes} close operator(s)"
    ));
    Ok((closes != 1).then(|| {
        format!(
            "★★★ after Ctrl+Z the square saves {closes} `h` operator(s), not 1: undo did not \
             close it again."
        )
    }))
}

/// Press `item` on the Format tab. With the Node rung already reached, an
/// item that is not there is the defect, so its absence is a FAIL.
fn press_command(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    item: &str,
) -> Result<Option<String>> {
    Ok(press_on_tab(ctx, session, pointer, item, FORMAT_TAB)
        .err()
        .map(|why| format!("with one node selected, `{item}` cannot be pressed: {why}")))
}

/// Click the path, double-click into its Part rung, double-click `anchor`
/// into the Node rung with that one node selected.
fn enter_node(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    fixture: &Fixture,
    anchor: &str,
) -> Result<()> {
    let ui_rect = ui_rect_event(ctx)?;
    click_on(ctx, session, pointer, fixture, 0, ON_THE_LINE)?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, ON_THE_LINE.0, ON_THE_LINE.1))?;
    pointer.double_click(session, at)?;
    session.settle(30);
    let trace = session.trace()?;
    let mark_rect = declared(&trace, ui_rect, anchor).ok_or_else(|| {
        Error::new(format!(
            "no `{anchor}` after descending into the path (last `{ANCHORS}`: {}). Trace: {}.",
            trace.last(ANCHORS).map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        ))
    })?;
    let mark = trace.mark();
    pointer.double_click(session, WindowPoint::centre_of(mark_rect))?;
    session.settle(30);
    let trace = session.trace()?;
    let at_node = trace
        .last_after(SELECTION, mark)
        .is_some_and(|l| l.get("level") == Some("Node"));
    let picked = trace
        .last_after(ANCHORS, mark)
        .and_then(|l| l.get_usize("selected"));
    if !at_node || picked != Some(1) {
        return Err(Error::new(format!(
            "the double-click on `{anchor}` did not stand at the Node rung with one node \
             selected (selected={picked:?}). Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("Node rung on `{anchor}`"));
    Ok(())
}

/// The `path-end-applied` line after `mark` must name `edit` and have landed,
/// and the funnel must have traced the verb rather than its refusal.
fn applied(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    edit: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "the {edit} command reached no path verb (no `{APPLIED}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("applied: `{}`", line.raw));
    let funnel = if edit == "extend" {
        "extend-path"
    } else {
        "open-subpath"
    };
    let landed = trace.last_after(funnel, mark).is_some();
    let right = line.get("edit") == Some(edit) && line.get("ok") == Some("true") && landed;
    Ok((!right).then(|| {
        format!(
            "the press must read `edit={edit} ok=true` and trace `{funnel} page=`; it read `{}` \
             (funnel line {landed}). Trace: {}.",
            line.raw,
            session.trace_path().display()
        )
    }))
}

/// Ctrl+S, which must save in place.
fn save(session: &Session, pointer: &ScriptedPointer) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    let saved = session
        .trace()?
        .last_after(SAVED, mark)
        .is_some_and(|l| l.get("outcome") == Some("ok"));
    Ok((!saved).then(|| {
        format!(
            "Ctrl+S traced no `{SAVED} outcome=ok`. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

/// Ctrl+Z must traceably undo and leave `before` anchors drawn.
fn undo_restores(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    before: usize,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    let undone = session
        .trace()?
        .last_after(UNDONE, mark)
        .map(|l| l.raw.clone());
    let total = session
        .trace()?
        .last_after(ANCHORS, mark)
        .and_then(|l| l.get_usize("total"));
    report.note(format!(
        "undo: `{}`, anchors {total:?}",
        undone.clone().unwrap_or_default()
    ));
    Ok((undone.is_none() || total != Some(before)).then(|| {
        format!(
            "Ctrl+Z must trace `{UNDONE}` and leave {before} anchors; it traced {undone:?} \
             and {total:?}. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

/// How many `op` operators the last content stream holding `holding` carries:
/// an appended save keeps the fixture's own stream earlier in the file.
fn operators(pdf: &std::path::Path, holding: &str, op: &str) -> Result<usize> {
    let bytes =
        std::fs::read(pdf).map_err(|e| Error::new(format!("reading {}: {e}", pdf.display())))?;
    let body = streams(&bytes)
        .into_iter()
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .rev()
        .find(|b| b.contains(holding) && b.contains(" S"))
        .ok_or_else(|| Error::new(format!("no stream in {} holds `{holding}`", pdf.display())))?;
    Ok(body
        .split(|c: char| c.is_whitespace())
        .filter(|t| *t == op.trim())
        .count())
}
