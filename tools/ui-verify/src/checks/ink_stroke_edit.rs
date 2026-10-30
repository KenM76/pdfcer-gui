//! `a_freehand_marks_strokes_move_and_delete` — with the Points tool armed, a
//! Alt+drag on any point of a freehand mark moves that point's whole stroke,
//! and the markup menu's *Delete this stroke* removes the stroke right-clicked;
//! on the mark's last stroke the row is not drawn. Scripted pointer, window off
//! the desktop.
//!
//! The fixture is one `/Ink` of two strokes: stroke 0 is anchors 0–2, stroke 1
//! anchors 3–4 (`fixtures/ink-two-strokes.PROVENANCE.py`).

use super::dimdrive::{Fixture, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::{ScriptedPointer, xy};
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "ink-two-strokes.pdf",
    method: "Rebuild it with `python fixtures/ink-two-strokes.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// Stroke 0's middle point, and stroke 1's first.
const GRAB: &str = "canvas.markup-node.1";
const SECOND: &str = "canvas.markup-node.3";
/// A point on stroke 0's first segment, which selects the mark.
const ON_STROKE: (f64, f64) = (110.0, 200.0);
/// Where the grabbed point is dragged: clear of both strokes.
const DESTINATION: (f64, f64) = (200.0, 220.0);
const ROW: &str = "menu.item.canvas.markup.markup.remove_stroke";
/// Mirrors `canvas::annotnodes::stroke::TRACE_MOVE_STROKE`.
const MOVED: &str = "markup-stroke-move"; // ui-text-exempt: a trace event name
const APPLIED: &str = "ink-reshape-applied"; // ui-text-exempt: a trace event name
const MENU: &str = "markup-node-menu"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct AFreehandMarksStrokesMoveAndDelete;

impl Check for AFreehandMarksStrokesMoveAndDelete {
    fn name(&self) -> &'static str {
        "a_freehand_marks_strokes_move_and_delete"
    }

    fn defect(&self) -> &'static str {
        "a freehand mark's strokes can only be edited point by point: a whole stroke cannot be \
         moved or deleted on its own"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.review", "ink-stroke-edit", steps)
    }
}

fn at(ctx: &CheckContext, session: &Session, p: (f64, f64)) -> Result<WindowPoint> {
    CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?
        .doc_to_window(DocPoint::new(0, p.0, p.1))
}

/// The centre of the published region `name`, if drawn.
fn region(ctx: &CheckContext, session: &Session, name: &str) -> Result<Option<WindowPoint>> {
    Ok(declared_in(&session.trace()?, ui_rect_event(ctx)?, name)
        .map(|(rect, _)| WindowPoint::centre_of(rect)))
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    pointer.key(session, None, "A", None)?;
    session.settle(20);
    pointer.click(session, at(ctx, session, ON_STROKE)?)?;
    session.settle(30);
    let Some(grab) = region(ctx, session, GRAB)? else {
        return Ok(Some(format!(
            "with the Points tool armed the freehand mark showed no `{GRAB}`. Trace: {path}."
        )));
    };
    let to = at(ctx, session, DESTINATION)?;
    let mark = session.trace()?.mark();
    pointer.send(
        session,
        &format!("drag {} {} steps=8 mods=alt", xy(grab), xy(to)),
    )?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(moved) = trace.last_after(MOVED, mark) else {
        return Ok(Some(format!(
            "★★★ Alt+drag on stroke 0's point traced no `{MOVED}`: the drag did not reach the \
             whole-stroke move. Trace: {path}."
        )));
    };
    report.note(format!("`{}`", moved.raw));
    // The destination is 40 right and 20 up of the grabbed point (160,200);
    // an axis-locked move drops the 20.
    if moved.get("dy") != Some("20.00") || moved.get("dx") != Some("40.00") {
        return Ok(Some(format!(
            "★★★ the stroke moved by `{}`; the drag asked dx=40.00 dy=20.00.",
            moved.raw
        )));
    }
    let Some(applied) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "★★★ the stroke move was raised and no `{APPLIED}` followed. Trace: {path}."
        )));
    };
    report.note(format!("`{}`", applied.raw));
    let wanted = [
        ("edit", "move-stroke"),
        ("stroke", "0"),
        ("strokes", "2->2"),
    ];
    if wanted.iter().any(|(k, v)| applied.get(k) != Some(v)) {
        return Ok(Some(format!(
            "★★★ the move applied as `{}`; stroke 0 moved whole, two strokes kept, was asked.",
            applied.raw
        )));
    }
    delete(ctx, report, session, pointer, &path)
}

/// Right-click stroke 1, take *Delete this stroke*, then right-click the one
/// stroke left and require the row to be absent.
fn delete(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &str,
) -> Result<Option<String>> {
    let Some(second) = region(ctx, session, SECOND)? else {
        return Ok(Some(format!(
            "after the move the mark showed no `{SECOND}`. Trace: {path}."
        )));
    };
    pointer.right_click(session, second)?;
    session.settle(30);
    let Some(row) = region(ctx, session, ROW)? else {
        let menu = session.trace()?.last(MENU).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ the markup menu on stroke 1 drew no `{ROW}`; its rows: {menu:?}. Trace: {path}."
        )));
    };
    let mark = session.trace()?.mark();
    pointer.click(session, row)?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(applied) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "★★★ Delete this stroke traced no `{APPLIED}`: the document did not change. \
             Trace: {path}."
        )));
    };
    report.note(format!("`{}`", applied.raw));
    let wanted = [
        ("edit", "remove-stroke"),
        ("stroke", "1"),
        ("strokes", "2->1"),
    ];
    if wanted.iter().any(|(k, v)| applied.get(k) != Some(v)) {
        return Ok(Some(format!(
            "★★★ the delete applied as `{}`; stroke 1 removed, one stroke left, was asked.",
            applied.raw
        )));
    }
    last_stroke(ctx, report, session, pointer, path)
}

/// On the mark's only stroke the engine refuses, so the row is not drawn.
fn last_stroke(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &str,
) -> Result<Option<String>> {
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    let Some(grab) = region(ctx, session, GRAB)? else {
        return Ok(Some(format!(
            "after the delete the mark showed no `{GRAB}`. Trace: {path}."
        )));
    };
    let mark = session.trace()?.mark();
    pointer.right_click(session, grab)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(menu) = trace.last_after(MENU, mark) else {
        return Ok(Some(format!(
            "right-clicking the last stroke traced no `{MENU}`. Trace: {path}."
        )));
    };
    report.note(format!("`{}`", menu.raw));
    if menu.get("stroke") != Some("absent") {
        return Ok(Some(format!(
            "★★★ on the mark's only stroke the menu reads `{}`; the row must be absent.",
            menu.raw
        )));
    }
    pointer.key(session, None, "Escape", None)?;
    Ok(None)
}
