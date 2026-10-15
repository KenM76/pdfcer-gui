//! `dragging_a_paragraphs_edge_rewraps_it` — **click into a paragraph, drag
//! its right-edge handle in, and the paragraph re-wraps onto more lines.**
//!
//! The oracle is `reflow-block-applied … lines=6->N` with N above 6 and a
//! `width=Some(..)` field: the six-line fixture paragraph is about 267 pt
//! wide, so wrapping it at about 128 pt must add lines. A handle that drew but
//! raised nothing, or raised a reflow at the block's own width, fails.

use super::dimdrive::{Fixture, click_on, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
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
const HANDLE: &str = "text.paragraph-width"; // ui-text-exempt: a trace region name, never displayed
const DRAGGED_EVENT: &str = "text-edit-width-dragged"; // ui-text-exempt: a trace event name, never displayed
const APPLIED_EVENT: &str = "reflow-block-applied"; // ui-text-exempt: a trace event name, never displayed

/// Inside line 2 of the fixture's block, as `reflow` uses.
const CARET_AT: (f64, f64) = (120.0, 668.0);
/// Where the edge is dropped: 128 pt right of the block's left at x = 72.
const DROP_AT: (f64, f64) = (200.0, 668.0);
const LINES_BEFORE: u32 = 6;

pub struct DraggingAParagraphsEdgeRewrapsIt;

impl Check for DraggingAParagraphsEdgeRewrapsIt {
    fn name(&self) -> &'static str {
        "dragging_a_paragraphs_edge_rewraps_it"
    }

    fn defect(&self) -> &'static str {
        "an open paragraph has no width handle, or dragging it does not re-wrap the paragraph \
         at the dragged width"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "reflow-width",
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
    click_on(ctx, session, pointer, &FIXTURE, 0, CARET_AT)?;
    let trace = session.trace()?;
    let Some((handle, vp)) = declared_in(&trace, ui_rect_event(ctx)?, HANDLE) else {
        return Ok(Some(format!(
            "a click inside a six-line paragraph drew no width handle: `{HANDLE}` was never \
             declared. Either the click opened a line rather than the paragraph, or the handle \
             is not offered. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("handle: {handle:?}"));
    pointer.screenshot(session, &ctx.out("reflow-width-open.png"))?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let to = mapping.doc_to_window(DocPoint::new(0, DROP_AT.0, DROP_AT.1))?;
    let mark = trace.mark();
    pointer.drag_in(
        session,
        vp.as_deref(),
        WindowPoint::centre_of(handle),
        to,
        8,
        "l",
    )?;
    session.settle(40);
    pointer.screenshot(session, &ctx.out("reflow-width-after.png"))?;
    let trace = session.trace()?;
    let Some(dragged) = trace.last_after(DRAGGED_EVENT, mark) else {
        return Ok(Some(format!(
            "the handle was dragged from x = {:.0} to x = {:.0} and raised nothing: no \
             `{DRAGGED_EVENT}`. Trace: {}.",
            WindowPoint::centre_of(handle).x(),
            to.x(),
            session.trace_path().display()
        )));
    };
    report.note(format!("dragged: `{}`", dragged.raw));
    let Some(applied) = trace.last_after(APPLIED_EVENT, mark) else {
        return Ok(Some(format!(
            "`{}` and nothing reached the document: no `{APPLIED_EVENT}`. Trace: {}.",
            dragged.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("applied: `{}`", applied.raw));
    if !applied.get("width").is_some_and(|w| w.starts_with("Some")) {
        return Ok(Some(format!(
            "the reflow ran at the block's own width, not the dragged one: `{}`.",
            applied.raw
        )));
    }
    let lines = applied.get("lines").unwrap_or_default();
    let (before, after) = super::reflow::parse_lines(lines).ok_or_else(|| {
        Error::new(format!(
            "`{APPLIED_EVENT}` reported lines={lines:?}; expected `A->B`."
        ))
    })?;
    if before != LINES_BEFORE {
        return Err(Error::new(format!(
            "the block was recognised as {before} lines, not the fixture's {LINES_BEFORE}. \
             Regenerate with `python tools/gen-reflow-fixture.py`."
        )));
    }
    if after <= before {
        return Ok(Some(format!(
            "the paragraph was narrowed to about half its width and kept {after} lines: `{}`.",
            applied.raw
        )));
    }
    Ok(None)
}
