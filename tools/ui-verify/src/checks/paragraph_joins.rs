//! `edge_keys_open_the_paragraph` — with Paragraphs off, Backspace at a
//! line's start joins it to the line above, and a multi-line paste into a
//! line keeps its breaks: both open the line's paragraph first.
//! `enter_in_a_recognised_line_is_declined` — Enter in a recognised line
//! says why it cannot break it and leaves its paragraph alone.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/paragraph_joins.md`.

use super::dimdrive::{click_on, run_on};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const PROMOTED: &str = "text-edit-promoted"; // ui-text-exempt: a trace event name, never displayed
const TYPING: &str = "text-edit-typing"; // ui-text-exempt: a trace event name, never displayed
const PASTE: &str = "text-edit-paste"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "edit-block-text-applied"; // ui-text-exempt: a trace event name, never displayed
/// Line 2 of `paragraph.pdf`'s paragraph.
const LINE_2: (f64, f64) = (120.0, 668.0);
/// What is pasted: two lines, so a joined paste would trace `joined=1`.
const PASTED: &str = "x\ny";

const DECLINED: &str = "text-edit-enter-declined"; // ui-text-exempt: a trace event name, never displayed
/// On `SITE`, the first word of the layer's first line.
const ON_RECOGNISED: (f64, f64) = (85.0, 704.0);

/// Enter in a recognised line is declined, naming why.
pub struct EnterInARecognisedLineIsDeclined;

impl Check for EnterInARecognisedLineIsDeclined {
    fn name(&self) -> &'static str {
        "enter_in_a_recognised_line_is_declined"
    }

    fn defect(&self) -> &'static str {
        "Enter in a line of recognised text re-spaces its whole paragraph, moving the words off          the scan they sit over"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &super::ocr_select_block::FIXTURE,
            "mode.edit,view.ocr_layer,edit.text",
            "recognised-enter",
            drive_recognised,
        )
    }
}

/// Click the recognised line, End, Enter: declined as `recognised-word`,
/// and no paragraph opened.
fn drive_recognised(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let fixture = &super::ocr_select_block::FIXTURE;
    click_on(ctx, session, pointer, fixture, 0, ON_RECOGNISED)?;
    session.settle(20);
    pointer.key(session, None, "End", None)?;
    session.settle(5);
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Enter", None)?;
    session.settle(20);
    let trace = session.trace()?;
    let declined = trace.last_after(DECLINED, mark).map(|l| l.raw.clone());
    let promoted = trace.last_after(PROMOTED, mark).map(|l| l.raw.clone());
    report.note(format!("Enter: {declined:?}, {promoted:?}"));
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    let named = declined
        .as_deref()
        .is_some_and(|d| d.contains("reason=recognised-word"));
    Ok((!named || promoted.is_some()).then(|| {
        format!(
            "Enter in a recognised line traced {declined:?} and {promoted:?}; want it declined              with reason=recognised-word and no paragraph opened. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

/// Edge keys and multi-line paste reach the paragraph.
pub struct EdgeKeysOpenTheParagraph;

impl Check for EdgeKeysOpenTheParagraph {
    fn name(&self) -> &'static str {
        "edge_keys_open_the_paragraph"
    }

    fn defect(&self) -> &'static str {
        "Backspace at a line's start does nothing, and a pasted line break turns into a space, \
         when the line was opened on its own"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &super::reflow::FIXTURE,
            "mode.edit,edit.paragraphs,edit.text",
            "paragraph-joins",
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
    let path = session.trace_path().display().to_string();
    if let Some(failure) = join(ctx, report, session, pointer, &path)? {
        return Ok(Some(failure));
    }
    paste(ctx, report, session, pointer, &path)
}

/// Line 2, Home, Backspace, Ctrl+Enter: the paragraph opens, loses one
/// character (the break) and is written.
fn join(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &str,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &super::reflow::FIXTURE, 0, LINE_2)?;
    session.settle(20);
    pointer.key(session, None, "Home", None)?;
    session.settle(5);
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Backspace", None)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(promoted) = trace.last_after(PROMOTED, mark).cloned() else {
        return Ok(Some(format!(
            "Backspace at the start of line 2 opened no paragraph (no `{PROMOTED}`). Trace: {path}."
        )));
    };
    report.note(format!("Backspace: `{}`", promoted.raw));
    let before = promoted.get_usize("len");
    let after = trace
        .events(TYPING)
        .filter(|l| l.lineno > mark)
        .last()
        .and_then(|l| l.get_usize("len"));
    if before.is_none() || after != before.map(|n| n - 1) {
        return Ok(Some(format!(
            "the opened paragraph was {before:?} long and {after:?} after Backspace; want one \
             less — the line break removed. Trace: {path}."
        )));
    }
    let mark = trace.mark();
    pointer.key(session, None, "Enter", Some("ctrl"))?;
    session.settle(40);
    Ok(session
        .trace()?
        .last_after(APPLIED, mark)
        .is_none()
        .then(|| format!("the joined paragraph was not written (no `{APPLIED}`). Trace: {path}.")))
}

/// Line 2 again, End, paste two lines: the paragraph opens and the paste
/// keeps its break.
fn paste(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &str,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &super::reflow::FIXTURE, 0, LINE_2)?;
    session.settle(20);
    pointer.key(session, None, "End", None)?;
    session.settle(5);
    let mark = session.trace()?.mark();
    pointer.paste(session, None, PASTED)?;
    session.settle(20);
    let trace = session.trace()?;
    let promoted = trace.last_after(PROMOTED, mark).map(|l| l.raw.clone());
    let pasted = trace.last_after(PASTE, mark).map(|l| l.raw.clone());
    report.note(format!("paste: {promoted:?}, {pasted:?}"));
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    let kept = pasted.as_deref().is_some_and(|p| p.contains("joined=0"));
    Ok((promoted.is_none() || !kept).then(|| {
        format!(
            "a two-line paste into line 2 traced {promoted:?} and {pasted:?}; want the paragraph \
             opened and `joined=0`. Trace: {path}."
        )
    }))
}
