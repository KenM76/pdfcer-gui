//! `with_paragraphs_off_a_click_opens_one_line` — **press Paragraphs on the
//! Edit tab, click in a line of a paragraph, and only that line opens.**
//!
//! The button is pressed with the pointer, not invoked by id, so a switch that
//! works but is not on the tab fails too.
//!
//! The oracle is the `text-edit-caret` line after the click: `run=N`, not
//! `block=N`. The same click with the switch on opens `block=` —
//! `dragging_a_paragraphs_edge_rewraps_it` needs that and is the control.
//! `open-paragraphs enabled=false` must precede it, so a click that opened a
//! line for some other reason cannot pass for the switch working.

use super::dimdrive::{Fixture, click_on, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "paragraph.pdf",
    method: "Regenerate it: python tools/gen-reflow-fixture.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

const SWITCHED: &str = "open-paragraphs"; // ui-text-exempt: a trace event name, never displayed
const TAB: &str = "ribbon.tab.edit"; // ui-text-exempt: a trace region name, never displayed
const BUTTON: &str = "ribbon.item.edit.paragraphs"; // ui-text-exempt: a trace region name, never displayed
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
/// Inside line 2 of the fixture's six-line block, as `reflow_width` uses.
const CARET_AT: (f64, f64) = (120.0, 668.0);

pub struct WithParagraphsOffAClickOpensOneLine;

impl Check for WithParagraphsOffAClickOpensOneLine {
    fn name(&self) -> &'static str {
        "with_paragraphs_off_a_click_opens_one_line"
    }

    fn defect(&self) -> &'static str {
        "the Paragraphs switch does nothing: with it off, a click in a paragraph still opens \
         the whole paragraph"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "paragraphs-off",
            drive,
        )
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    for region in [TAB, BUTTON] {
        let trace = session.trace()?;
        let Some((rect, vp)) = declared_in(&trace, ui_rect_event(ctx)?, region) else {
            return Ok(Some(format!(
                "`{region}` was never declared, so the switch cannot be reached. Trace: {}.",
                session.trace_path().display()
            )));
        };
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(14);
    }
    pointer.screenshot(session, &ctx.out("paragraphs-button.png"))?;
    let trace = session.trace()?;
    let Some(switched) = trace.last(SWITCHED) else {
        return Ok(Some(format!(
            "`{BUTTON}` was clicked and raised no `{SWITCHED}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("switched: `{}`", switched.raw));
    if switched.get("enabled") != Some("false") {
        return Ok(Some(format!(
            "pressing Paragraphs once from the default did not turn it off: `{}`.",
            switched.raw
        )));
    }
    let mark = trace.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, CARET_AT)?;
    pointer.screenshot(session, &ctx.out("paragraphs-off.png"))?;
    let trace = session.trace()?;
    let Some(caret) = trace.last_after(CARET, mark) else {
        return Ok(Some(format!(
            "a click inside the paragraph opened nothing: no `{CARET}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("caret: `{}`", caret.raw));
    if caret.get("block").is_some() {
        return Ok(Some(format!(
            "with Paragraphs off, the click still opened the whole paragraph: `{}`.",
            caret.raw
        )));
    }
    if caret.get("run").is_none() {
        return Ok(Some(format!(
            "the click opened neither a line nor the paragraph: `{}`.",
            caret.raw
        )));
    }
    Ok(None)
}
