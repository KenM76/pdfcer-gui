//! Two checks that the Select filter's **Form fields** and **Characters** rows
//! change what a click on the page reaches.
//!
//! - `form_fields_row_lets_clicks_through_fields` — off, a click on a field
//!   neither selects it in Edit nor focuses it for filling in Read; back on,
//!   the Read click focuses it.
//! - `characters_row_stops_text_sweeps` — off, a drag across a line selects no
//!   characters; back on, the same drag does.
//!
//! The filter is persisted, so both normalise it to All first and leave it
//! at All.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/filter_rows.md`.

use super::dimdrive::{Fixture, click_on, press, run_on, ui_rect_event};
use crate::checks::driving::click_mode_segment;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIELDS: Fixture = Fixture {
    file: "three-text-fields.pdf",
    method: "Rebuild it with `python fixtures/three-text-fields.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 300.0,
        height_pt: 260.0,
    },
};
const LINES: Fixture = Fixture {
    file: "word-fragmented-lines.pdf",
    method: "Rebuild it with `python fixtures/word-fragmented-lines.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
/// Inside `FieldOne`'s widget rectangle.
const ON_FIELD: (f64, f64) = (165.0, 237.0);
/// Along the first line, `Date Premises Required____`, at y=700 from x=72.
const SWEEP: ((f64, f64), (f64, f64)) = ((74.0, 704.0), (160.0, 704.0));
const BUTTON: &str = "status-group:filter"; // ui-text-exempt: a trace region name, never displayed
const ALL: &str = "status-filter-all"; // ui-text-exempt: a trace region name, never displayed
/// `PickClass::FormField`'s row, by its index in `PickClass::ALL`.
const FIELD_ROW: &str = "status-filter-row:8"; // ui-text-exempt: a trace region name, never displayed
/// `PickClass::Characters`'s row.
const CHARS_ROW: &str = "status-filter-row:10"; // ui-text-exempt: a trace region name, never displayed
const TOGGLED: &str = "pick-filter-toggled"; // ui-text-exempt: a trace event name, never displayed
const SELECTED: &str = "form-field-selected"; // ui-text-exempt: a trace event name, never displayed
const FOCUSED: &str = "form-focus"; // ui-text-exempt: a trace event name, never displayed
const SWEPT: &str = "canvas-text-selection"; // ui-text-exempt: a trace event name, never displayed
const STATUS: &str = "tool-status"; // ui-text-exempt: a trace event name, never displayed
/// The words the tool status line carries while Characters is off.
const NO_TEXT: &str = "selects no text"; // ui-text-exempt: matched against a trace line

/// The Form fields row lets clicks through fields.
pub struct FormFieldsRowLetsClicksThroughFields;

impl Check for FormFieldsRowLetsClicksThroughFields {
    fn name(&self) -> &'static str {
        "form_fields_row_lets_clicks_through_fields"
    }

    fn defect(&self) -> &'static str {
        "with Form fields switched off in the Select filter, a click on a field still selects it \
         in Edit and still focuses it for filling in Read"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIELDS,
            "mode.edit",
            "filter-fields",
            drive_fields,
        )
    }
}

/// The Characters row stops text sweeps.
pub struct CharactersRowStopsTextSweeps;

impl Check for CharactersRowStopsTextSweeps {
    fn name(&self) -> &'static str {
        "characters_row_stops_text_sweeps"
    }

    fn defect(&self) -> &'static str {
        "with Characters switched off in the Select filter, a drag across a line still selects \
         its characters"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &LINES, "mode.read", "filter-chars", drive_chars)
    }
}

/// Open the popup, press `region` inside it, close it.
fn in_popup(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<()> {
    press(ctx, session, pointer, BUTTON)?;
    press(ctx, session, pointer, region)?;
    press(ctx, session, pointer, BUTTON)
}

/// Switch `row` off from All, requiring the toggle line for `token`.
fn switch_off(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    row: &str,
    token: &str,
) -> Result<()> {
    in_popup(ctx, session, pointer, ALL)?;
    let mark = session.trace()?.mark();
    in_popup(ctx, session, pointer, row)?;
    let trace = session.trace()?;
    match trace.last_after(TOGGLED, mark) {
        Some(l) if l.get("class") == Some(token) && l.get("on") == Some("false") => Ok(()),
        other => Err(Error::new(format!(
            "pressing `{row}` did not switch `{token}` off: {}. Trace: {}.",
            other.map_or("no toggle line", |l| l.raw.as_str()),
            session.trace_path().display()
        ))),
    }
}

fn drive_fields(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    switch_off(ctx, session, pointer, FIELD_ROW, "form-field")?;
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIELDS, 0, ON_FIELD)?;
    if let Some(l) = session.trace()?.last_after(SELECTED, mark)
        && l.get("field").is_some()
    {
        return Ok(Some(format!(
            "with Form fields off, an Edit click on FieldOne selected it: `{}`.",
            l.raw
        )));
    }
    click_mode_segment(session, pointer, ui_rect_event(ctx)?, "read")?;
    session.settle(30);
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIELDS, 0, ON_FIELD)?;
    if let Some(l) = session.trace()?.last_after(FOCUSED, mark) {
        return Ok(Some(format!(
            "with Form fields off, a Read click on FieldOne focused it for filling: `{}`.",
            l.raw
        )));
    }
    in_popup(ctx, session, pointer, ALL)?;
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIELDS, 0, ON_FIELD)?;
    let trace = session.trace()?;
    let Some(l) = trace.last_after(FOCUSED, mark) else {
        return Ok(Some(format!(
            "with the filter back at All, a Read click on FieldOne focused nothing, so the \
             click never reached the field and the check proves nothing. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("back at All, the Read click focused: `{}`", l.raw));
    Ok(None)
}

/// Drag along the first line; the characters it selected, if any.
fn sweep(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<usize> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, LINES.page, 0)?;
    let at = |p: (f64, f64)| mapping.doc_to_window(DocPoint::new(0, p.0, p.1));
    let mark = session.trace()?.mark();
    pointer.drag(session, at(SWEEP.0)?, at(SWEEP.1)?, 8)?;
    session.settle(20);
    Ok(session
        .trace()?
        .last_after(SWEPT, mark)
        .and_then(|l| l.get_usize("chars"))
        .unwrap_or(0))
}

fn drive_chars(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    switch_off(ctx, session, pointer, CHARS_ROW, "characters")?;
    let said = session.trace()?.last(STATUS).map(|l| l.raw.clone());
    if !said.as_deref().is_some_and(|l| l.contains(NO_TEXT)) {
        in_popup(ctx, session, pointer, ALL)?;
        return Ok(Some(format!(
            "with Characters off, the tool status line does not say a drag selects no text: {}.",
            said.as_deref().unwrap_or("no `tool-status` line")
        )));
    }
    let off = sweep(ctx, session, pointer)?;
    if off > 0 {
        in_popup(ctx, session, pointer, ALL)?;
        return Ok(Some(format!(
            "with Characters off, a drag along the first line selected {off} characters."
        )));
    }
    in_popup(ctx, session, pointer, ALL)?;
    if let Some(l) = session.trace()?.last(STATUS)
        && l.raw.contains(NO_TEXT)
    {
        return Ok(Some(format!(
            "back at All, the tool status line still says a drag selects no text: `{}`.",
            l.raw
        )));
    }
    let on = sweep(ctx, session, pointer)?;
    if on == 0 {
        return Ok(Some(format!(
            "with the filter back at All, the same drag selected nothing, so the drag never \
             reached the text and the check proves nothing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("off: 0 characters; back at All: {on}"));
    Ok(None)
}
