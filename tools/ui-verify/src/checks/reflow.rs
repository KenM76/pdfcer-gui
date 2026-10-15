//! `reflowing_a_paragraph_rewraps_it` — **click into a paragraph, press
//! Reflow, and the block loses a line.**
//!
//! Driven with the scripted pointer off the desktop on a copy of
//! `fixtures/paragraph.pdf`: six short ragged lines that a re-wrap to the
//! block's own width packs into fewer. The click opens the whole paragraph
//! (`Anchor::Block`), so this also proves Reflow accepts an opened paragraph.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/reflow.md`.

use super::dimdrive::{Fixture, click_on, press_on_tab, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
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
/// `text-edit-caret page=… run=… len=…` — a click resolved text.
const CARET_EVENT: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
/// `text-edit-declined reason=…` — a click did not.
const DECLINED_EVENT: &str = "text-edit-declined"; // ui-text-exempt: a trace event name, never displayed
const REFLOW_ITEM: &str = "ribbon.item.edit.reflow_block"; // ui-text-exempt: a trace region name, never displayed
const EDIT_TAB: &str = "ribbon.tab.edit"; // ui-text-exempt: a trace region name, never displayed
/// `reflow-resolved page=… block=… from=…` — the caret became a block index.
const RESOLVED_EVENT: &str = "reflow-resolved"; // ui-text-exempt: a trace event name, never displayed
/// `reflow-declined reason=…` — it did not, and the shell said which way.
const DECLINE_EVENT: &str = "reflow-declined"; // ui-text-exempt: a trace event name, never displayed
/// `reflow-block-applied page=… block=… lines=A->B …` — the engine re-wrapped.
const APPLIED_EVENT: &str = "reflow-block-applied"; // ui-text-exempt: a trace event name, never displayed

/// Where to put the caret, in PDF user space: inside line 2.
const CARET_AT: (f64, f64) = (120.0, 668.0);
/// The line count the fixture starts with.
const LINES_BEFORE: u32 = 6;

/// See the module documentation.
pub struct ReflowingAParagraphRewrapsIt;

impl Check for ReflowingAParagraphRewrapsIt {
    fn name(&self) -> &'static str {
        "reflowing_a_paragraph_rewraps_it"
    }

    fn defect(&self) -> &'static str {
        "pressing Reflow in a paragraph does not re-wrap it — refused for the paragraph the \
         click opened, or run at a width that changes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.edit,edit.text", "reflow", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, CARET_AT)?;
    let trace = session.trace()?;
    let Some(caret) = trace.last_after(CARET_EVENT, mark) else {
        let instead = trace
            .last_after(DECLINED_EVENT, mark)
            .map_or_else(|| "nothing".to_owned(), |l| format!("`{}`", l.raw));
        return Ok(Some(format!(
            "a click at ({}, {}), inside line 2 of an upright single-font paragraph, placed no \
             caret; it traced {instead}. Trace: {}.",
            CARET_AT.0,
            CARET_AT.1,
            session.trace_path().display()
        )));
    };
    report.note(format!("caret: `{}`", caret.raw));

    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, REFLOW_ITEM, EDIT_TAB)?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(resolved) = trace.last_after(RESOLVED_EVENT, mark) else {
        let declined = trace
            .last_after(DECLINE_EVENT, mark)
            .map_or_else(|| "nothing".to_owned(), |l| format!("`{}`", l.raw));
        return Ok(Some(format!(
            "Reflow was pressed with the caret in a six-line paragraph and resolved no block; \
             it traced {declined}. `reason=no-caret` means the ribbon click cleared the draft; \
             `caret-not-on-a-run` means the opened paragraph was refused. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("resolved: `{}`", resolved.raw));

    let Some(applied) = trace.last_after(APPLIED_EVENT, mark) else {
        return Ok(Some(format!(
            "`{}` and nothing reached the document: no `{APPLIED_EVENT}`. The fixture is \
             upright, simple-font and one text object, so an engine refusal is itself the \
             finding. Trace: {}.",
            resolved.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("applied: `{}`", applied.raw));
    let lines = applied.get("lines").unwrap_or_default();
    let (before, after) = parse_lines(lines).ok_or_else(|| {
        Error::new(format!(
            "`{APPLIED_EVENT}` reported lines={lines:?}, which this check cannot read; it \
             expects `A->B`."
        ))
    })?;
    if before != LINES_BEFORE {
        return Err(Error::new(format!(
            "the block was recognised as {before} lines and the fixture is written with \
             {LINES_BEFORE}, so the line-count oracle would measure something other than the \
             re-wrap. Regenerate with `python tools/gen-reflow-fixture.py`."
        )));
    }
    if after >= before {
        return Ok(Some(format!(
            "the reflow ran and changed nothing: `{}`. The six ragged lines pack into fewer at \
             the block's own width. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    Ok(None)
}

/// `A->B` from the trace's `lines=` field.
pub(super) fn parse_lines(field: &str) -> Option<(u32, u32)> {
    let (before, after) = field.split_once("->")?;
    Some((before.trim().parse().ok()?, after.trim().parse().ok()?))
}
