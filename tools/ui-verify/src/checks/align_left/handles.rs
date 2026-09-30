//! The Align check's on-canvas handles phase: with all three boxes selected
//! at their fixture positions, tick *On-canvas alignment* on the Align tab,
//! then click the handle on the selection's left edge. It must move exactly
//! as the panel's Align left does — boxes 1 and 2 to box 0's left edge, one
//! step — and the handles must be absent before the toggle is ticked.

use crate::checks::driving::declared;
use crate::error::Result;
use crate::input::Driver;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::sys::vk;

const ALIGN_TAB: &str = "align.tab.align";
const TOGGLE: &str = "align.on_canvas";
const LEFT_HANDLE: &str = "align.handle.3";
const MOVED_EVENT: &str = "move-each-applied"; // ui-text-exempt: a trace event name, never displayed
/// Align left, as the panel's own press moved them: box 0's left edge is
/// x 100; box 1 starts at 250, box 2 at 180.
const EXPECTED: &str = "1:-150.0,0.0;2:-80.0,0.0";

pub(super) fn drive(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let trace_path = session.trace_path().display().to_string();
    if declared(&session.trace()?, ui_rect, LEFT_HANDLE).is_some() {
        return Ok(Some(format!(
            "★ `{LEFT_HANDLE}` IS DRAWN BEFORE *On-canvas alignment* IS TICKED. Trace: {trace_path}."
        )));
    }
    for (region, notch) in [(ALIGN_TAB, 1), (TOGGLE, -1)] {
        if !super::arrange::press(session, driver, ui_rect, region, notch)? {
            return Ok(Some(format!(
                "★ THE PANEL PUBLISHED NO `{region}`. Trace: {trace_path}."
            )));
        }
    }
    let trace = session.trace()?;
    let Some(handle) = declared(&trace, ui_rect, LEFT_HANDLE) else {
        return Ok(Some(format!(
            "★ *On-canvas alignment* WAS TICKED WITH THREE OBJECTS SELECTED AND NO \
             `{LEFT_HANDLE}` WAS DRAWN. Trace: {trace_path}."
        )));
    };
    let mark = trace.mark();
    driver.click_at(session.frame()?.declared_center(handle))?;
    session.settle(24);
    let trace = session.trace()?;
    let Some(applied) = trace.last_after(MOVED_EVENT, mark) else {
        return Ok(Some(format!(
            "★ The left-edge handle was clicked and no `{MOVED_EVENT}` followed: the press \
             reached the canvas, or planned nothing. Trace: {trace_path}."
        )));
    };
    report.note(format!("★ `{}`", applied.raw));
    let moved = applied.get("moves").unwrap_or("");
    if applied.get("gesture") != Some("align") || moved != EXPECTED {
        return Ok(Some(format!(
            "★★ THE LEFT HANDLE MOVED `moves={moved}`, expected `{EXPECTED}` — Align left \
             relative to the selection area. `{}`",
            applied.raw
        )));
    }
    report.note("★★ the on-canvas left handle aligned the three boxes' left edges");
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(24);
    Ok(None)
}
