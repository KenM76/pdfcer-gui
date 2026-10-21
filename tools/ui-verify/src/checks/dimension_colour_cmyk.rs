//! `a_dimensions_colour_is_stored_in_cmyk` — a ce dimension's colour, set in
//! the properties panel on the picker's CMYK tab, is stored as CMYK.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_colour_cmyk.md`.

use super::dimdrive::{press, reveal, run, ui_rect_event};
use super::dimension_text_select::select_by_line;
use super::stroke_style::into_view;
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const PROPERTIES_TAB: &str = "dock.tab.file.properties"; // ui-text-exempt: trace region name
const SET: &str = "dimension-properties.colour.set"; // ui-text-exempt: trace region name
const SWATCH: &str = "dimension-properties.colour"; // ui-text-exempt: trace region name
const CMYK_TAB: &str = "dimension-properties.colour.space.cmyk"; // ui-text-exempt: trace region name
const STYLED: &str = "dimension-style"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ADimensionsColourIsStoredInCmyk;

impl Check for ADimensionsColourIsStoredInCmyk {
    fn name(&self) -> &'static str {
        "a_dimensions_colour_is_stored_in_cmyk"
    }

    fn defect(&self) -> &'static str {
        "a ce dimension's colour picker offers no CMYK, or a colour chosen on its CMYK tab is \
         stored as RGB"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, "mode.review", "dim-cmyk", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    select_by_line(ctx, session, pointer)?;
    reveal(ctx, session, pointer, SET, PROPERTIES_TAB)?;
    into_view(ctx, session, pointer, SET)?;
    press(ctx, session, pointer, SET)?;
    into_view(ctx, session, pointer, SWATCH)?;
    press(ctx, session, pointer, SWATCH)?;
    let Some((_, vp)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, CMYK_TAB) else {
        return Ok(Some(format!(
            "the colour picker opened with no CMYK tab (`{CMYK_TAB}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, CMYK_TAB)?;
    // The close is the commit.
    pointer.key(session, vp.as_deref(), "Escape", None)?;
    session.settle(40);
    let trace = session.trace()?;
    let colour = trace
        .last_after(STYLED, mark)
        .and_then(|l| l.get("colour").map(str::to_owned));
    report.note(format!("stored: {colour:?}"));
    Ok(match colour {
        Some(c) if c.starts_with("cmyk:") => None,
        other => Some(format!(
            "after choosing the CMYK tab and closing the picker, the ce dimension's colour must \
             be stored as CMYK (`{STYLED} colour=cmyk:..`). It was {other:?}. Trace: {}.",
            session.trace_path().display()
        )),
    })
}
