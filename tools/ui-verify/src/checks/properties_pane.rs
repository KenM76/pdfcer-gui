//! Reaching a control inside the Properties panel's scroll area with the
//! scripted pointer: wheel the panel until the control's region is declared
//! visible, then click its centre.

use crate::checks::CheckContext;
use crate::checks::driving::declared_in;
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;

/// The Properties panel's dock body, the region the wheel turns over.
pub const PANEL_BODY: &str = "dock.body.file.properties";

/// How many wheel turns before a control is called unreachable.
const MAX_SCROLL: usize = 12;

/// Scroll the Properties panel until `region` is declared, click it, and
/// settle `settle` frames; returns the viewport it was declared in.
///
/// # Errors
///
/// No ui-rect event in the profile, no Properties panel, or `region` still
/// undeclared after [`MAX_SCROLL`] turns.
pub fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    settle: u32,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    for _ in 0..=MAX_SCROLL {
        let trace = session.trace()?;
        if let Some((rect, vp)) = declared_in(&trace, ui_rect, region) {
            pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
            session.settle(settle);
            return Ok(vp);
        }
        let (panel, vp) = declared_in(&trace, ui_rect, PANEL_BODY).ok_or_else(|| {
            Error::new(format!(
                "no `{PANEL_BODY}` region, so the Properties panel is not open."
            ))
        })?;
        pointer.wheel_in(session, vp.as_deref(), WindowPoint::centre_of(panel), -3.0)?;
        session.settle(10);
    }
    Err(Error::new(format!(
        "no visible `{region}` after {MAX_SCROLL} wheel turns. Trace: {}",
        session.trace_path().display()
    )))
}
