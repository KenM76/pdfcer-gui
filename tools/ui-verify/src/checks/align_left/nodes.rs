//! The Align check's node-mode phase: two corners of box 0 picked at the
//! Node rung, the panel switches to its node rows, and *Vertical line* then
//! *Horizontal line* line them up. Two distinct corners of a rectangle
//! differ on at least one axis, so at least one press must move an anchor,
//! and every press that moves must land as `move_nodes`. A panel that
//! ignores node mode produces no `align-pressed op=Node` line at all.

use crate::checks::driving::declared;
use crate::coords::ScreenPoint;
use crate::error::{Error, Result};
use crate::input::{Driver, Key};
use crate::launch::Session;
use crate::report::CheckReport;

const ANCHORS_EVENT: &str = "canvas-anchors"; // ui-text-exempt: a trace event name, never displayed
const PRESSED_EVENT: &str = "align-pressed"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "move-nodes"; // ui-text-exempt: a trace event name, never displayed
const SELECTED_ANCHOR: &str = "canvas.selected-anchor";
const ANCHORS: [&str; 4] = [
    "canvas.anchor.0",
    "canvas.anchor.1",
    "canvas.anchor.2",
    "canvas.anchor.3",
];
const NODE_BUTTONS: [&str; 2] = ["align.node.0", "align.node.1"];
const PANEL: &str = "align.panel";

/// Run the phase. `box0` is a screen point inside box 0 at its fixture
/// position.
pub(super) fn drive(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    box0: ScreenPoint,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let trace_path = session.trace_path().display().to_string();
    // Object, then Part: a click, then a double-click at the same point.
    driver.click_at(box0)?;
    session.settle(12);
    driver.double_click_at(box0)?;
    session.settle(20);
    let trace = session.trace()?;
    let first = declared(&trace, ui_rect, ANCHORS[0]).ok_or_else(|| {
        Error::new(format!(
            "the Part rung on box 0 published no `{}`; that is the ladder's, not Align's. \
             Trace: {trace_path}.",
            ANCHORS[0]
        ))
    })?;
    // The Node rung is descended into ON an anchor.
    driver.double_click_at(session.frame()?.declared_center(first))?;
    session.settle(16);
    // The marks are re-laid out by the descent: read them again.
    let trace = session.trace()?;
    let chosen = declared(&trace, ui_rect, SELECTED_ANCHOR);
    let second = ANCHORS
        .iter()
        .filter_map(|n| declared(&trace, ui_rect, n))
        .find(|r| {
            chosen
                .is_none_or(|s| (r.min.x - s.min.x).abs() > 2.0 || (r.min.y - s.min.y).abs() > 2.0)
        })
        .ok_or_else(|| {
            Error::new(format!(
                "box 0 showed no second anchor. Trace: {trace_path}."
            ))
        })?;
    driver.click_with_modifier(session.frame()?.declared_center(second), Key::Shift)?;
    session.settle(16);
    let picked = session
        .trace()?
        .last(ANCHORS_EVENT)
        .and_then(|l| l.get_usize("selected"))
        .unwrap_or(0);
    report.note(format!("{picked} anchor(s) of box 0 selected"));
    if picked != 2 {
        return Err(Error::new(format!(
            "{picked} anchors selected, not 2: the multi-node pick's defect, which \
             `multi_node_move_moves_every_picked_anchor` owns. Trace: {trace_path}."
        )));
    }

    let mark = session.trace()?.mark();
    for button in NODE_BUTTONS {
        let mut rect = None;
        for _ in 0..12 {
            let trace = session.trace()?;
            rect = declared(&trace, ui_rect, button);
            if rect.is_some() {
                break;
            }
            let Some(panel) = declared(&trace, ui_rect, PANEL) else {
                break;
            };
            driver.scroll_at(session.frame()?.declared_center(panel), 1)?;
            session.settle(12);
        }
        let Some(rect) = rect else {
            return Ok(Some(format!(
                "★ WITH TWO NODES SELECTED THE PANEL PUBLISHED NO `{button}`: it does not switch \
                 to node mode. Trace: {trace_path}."
            )));
        };
        driver.click_at(session.frame()?.declared_center(rect))?;
        session.settle(24);
    }
    let trace = session.trace()?;
    let presses: Vec<_> = trace
        .events(PRESSED_EVENT)
        .filter(|l| l.lineno > mark && l.raw.contains("op=Node("))
        .collect();
    let applied = trace.events(APPLIED).filter(|l| l.lineno > mark).count();
    for p in &presses {
        report.note(format!("`{}`", p.raw));
    }
    if presses.len() != 2 {
        return Ok(Some(format!(
            "★ TWO NODE BUTTONS WERE CLICKED AND {} `{PRESSED_EVENT} op=Node(…)` LINES \
             FOLLOWED. Trace: {trace_path}.",
            presses.len()
        )));
    }
    if applied == 0 {
        return Ok(Some(format!(
            "★★ NEITHER NODE PRESS MOVED AN ANCHOR: two corners of a rectangle differ on at least \
             one axis, so one of Vertical line and Horizontal line must land as `{APPLIED}`. \
             Trace: {trace_path}."
        )));
    }
    report.note(format!(
        "★★ node mode lined up two corners: {applied} `{APPLIED}` step(s)"
    ));
    Ok(None)
}
