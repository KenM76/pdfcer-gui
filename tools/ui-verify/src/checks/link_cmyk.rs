//! `a_links_border_colour_is_stored_in_cmyk` — a selected link's border
//! colour, set on the CMYK tab in Properties and applied, is written into the
//! link's `/C` as four components. Scripted pointer, window off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/link_cmyk.md`.

use super::band_cmyk::colour_arity;
use super::dimdrive::{click_on, pick_space, press, run_on};
use super::links::{FIXTURE, ON_LINK};
use super::stroke_style::into_view;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const STEM: &str = "link-cmyk";
const SWATCH: &str = "properties.link.colour";
const CMYK_TAB: &str = "properties.link.colour.space.cmyk";
const APPLY_BORDER: &str = "properties.link.apply-border";
const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
const REBORDERED: &str = "set-link-border-applied"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ALinksBorderColourIsStoredInCmyk;

impl Check for ALinksBorderColourIsStoredInCmyk {
    fn name(&self) -> &'static str {
        "a_links_border_colour_is_stored_in_cmyk"
    }

    fn defect(&self) -> &'static str {
        "a link's border colour offers no CMYK, or a colour chosen on its CMYK tab is not \
         written to the link's /C as CMYK"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, STEM, steps)
    }
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let copy = ctx.out(&format!("{STEM}.pdf"));
    let read = |p: &std::path::Path| {
        std::fs::read(p).map_err(|e| Error::new(format!("reading {}: {e}", p.display())))
    };
    // The fixture's only `/C` is RGB, so a four-component one can only be the edit's.
    if colour_arity(&read(&copy)?).contains(&4) {
        return Ok(Some(format!(
            "the fixture already holds a CMYK `/C`, so this check cannot tell the edit's. {}",
            FIXTURE.method
        )));
    }
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_LINK)?;
    if !session
        .trace()?
        .last_after(SELECTED, mark)
        .is_some_and(|l| l.get("subtype") == Some("Link"))
    {
        return Ok(Some(format!(
            "a click inside the link traced no `{SELECTED}` with subtype=Link. Trace: {path}."
        )));
    }
    session.settle(20);
    if let Some(failure) = pick_space(ctx, session, pointer, SWATCH, CMYK_TAB)? {
        return Ok(Some(format!("★★★ {failure}")));
    }
    into_view(ctx, session, pointer, APPLY_BORDER)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, APPLY_BORDER)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(applied) = trace.last_after(REBORDERED, mark) else {
        return Ok(Some(format!(
            "Apply border was pressed and no `{REBORDERED}` followed. Trace: {path}."
        )));
    };
    report.note(format!("re-bordered: `{}`", applied.raw));
    if !applied
        .get("colour")
        .is_some_and(|c| c.starts_with("cmyk:"))
    {
        return Ok(Some(format!(
            "★★★ the CMYK tab was chosen and the border applied as `{}`.",
            applied.raw
        )));
    }
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    if !session
        .trace()?
        .events(SAVED)
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Ok(Some(format!(
            "Ctrl+S traced no `{SAVED} outcome=ok`. Trace: {path}."
        )));
    }
    let arities = colour_arity(&read(&copy)?);
    report.note(format!(
        "/C component counts in the saved file: {arities:?}"
    ));
    Ok((!arities.contains(&4)).then(|| {
        format!(
            "★★★ the applied CMYK border saved no four-component `/C` (component counts \
             {arities:?}). Trace: {path}."
        )
    }))
}
