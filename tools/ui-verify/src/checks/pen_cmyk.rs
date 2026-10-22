//! `a_mark_drawn_with_a_cmyk_pen_is_stored_in_cmyk` — the Markup ▸ Style
//! colour, set on the CMYK tab under *More colours*, is written into a newly
//! drawn mark's `/C` as four components. Scripted pointer, window off the
//! desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/pen_cmyk.md`.

use super::band_cmyk::colour_arity;
use super::dimdrive::{Fixture, press, press_on_tab, reveal, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review";
/// One red RGB `/Square` at `[200 400 340 460]` with no appearance.
const FIXTURE: Fixture = Fixture {
    file: "square-no-appearance.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const STEM: &str = "pen-cmyk";
const MARKUP_TAB: &str = "ribbon.tab.markup";
/// Mirror `markupswatch::REGION_INK` and the regions its popup publishes.
const SWATCH: &str = "markup.style.ink";
const MORE: &str = "markup.style.ink.more_colours";
const CMYK_TAB: &str = "markup.style.ink.more.space.cmyk";
const RECTANGLE: &str = "ribbon.item.markup.rectangle";
/// The drag's corners, page space, clear of the fixture's square.
const FROM: (f64, f64) = (100.0, 600.0);
const TO: (f64, f64) = (220.0, 680.0);
const PEN: &str = "markup-pen"; // ui-text-exempt: a trace event name, never displayed
const ADDED: &str = "add-markup"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AMarkDrawnWithACmykPenIsStoredInCmyk;

impl Check for AMarkDrawnWithACmykPenIsStoredInCmyk {
    fn name(&self) -> &'static str {
        "a_mark_drawn_with_a_cmyk_pen_is_stored_in_cmyk"
    }

    fn defect(&self) -> &'static str {
        "the Markup ▸ Style colour's More colours offers no CMYK, or a mark drawn after choosing \
         it is not written to /C as CMYK"
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
    reveal(ctx, session, pointer, SWATCH, MARKUP_TAB)?;
    press(ctx, session, pointer, SWATCH)?;
    if declared_in(&session.trace()?, ui_rect_event(ctx)?, MORE).is_none() {
        return Ok(Some(format!(
            "the pen's swatch opened with no More colours disclosure (`{MORE}`). Trace: {path}."
        )));
    }
    press(ctx, session, pointer, MORE)?;
    let Some((_, vp)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, CMYK_TAB) else {
        return Ok(Some(format!(
            "★★★ More colours opened with no CMYK tab (`{CMYK_TAB}`). Trace: {path}."
        )));
    };
    press(ctx, session, pointer, CMYK_TAB)?;
    let ink = session
        .trace()?
        .last(PEN)
        .and_then(|l| l.get("ink").map(str::to_owned))
        .unwrap_or_default();
    report.note(format!("pen after the CMYK tab: ink={ink}"));
    if !ink.starts_with("cmyk:") {
        return Ok(Some(format!(
            "★★★ the CMYK tab was pressed and the pen holds `ink={ink}`. Trace: {path}."
        )));
    }
    pointer.key(session, vp.as_deref(), "Escape", None)?;
    session.settle(30);

    press_on_tab(ctx, session, pointer, RECTANGLE, MARKUP_TAB)?;
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let at = |p: (f64, f64)| mapping.doc_to_window(DocPoint::new(0, p.0, p.1));
    let mark = session.trace()?.mark();
    pointer.drag(session, at(FROM)?, at(TO)?, 8)?;
    session.settle(30);
    if session.trace()?.last_after(ADDED, mark).is_none() {
        return Ok(Some(format!(
            "a rectangle drag traced no `{ADDED}`. Trace: {path}."
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
            "★★★ a rectangle drawn with a CMYK pen saved no four-component `/C` (component \
             counts {arities:?}). Trace: {path}."
        )
    }))
}
