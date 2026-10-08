//! `arrows_in_a_text_draft_stay_with_the_caret` — while a text draft is open
//! on a page, the arrow keys move neither egui's keyboard focus (the ribbon
//! highlight) nor the current page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/draft_arrows.md`.

use super::dimdrive::{Fixture, click_on, run_on};
use crate::checks::driving::shell_trace;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "four-pages-unrotated.pdf",
    method: "It is checked in; see `fixtures/four-pages-unrotated.PROVENANCE.md`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const FOCUS: &str = "keyboard-focus"; // ui-text-exempt: a trace event name, never displayed
const TYPING: &str = "text-edit-typing"; // ui-text-exempt: a trace event name, never displayed
const CANVAS: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed
const INVOKED: &str = "ribbon-command-invoked"; // ui-text-exempt: a trace event name, never displayed
const ENTER: &str = "text-edit-enter"; // ui-text-exempt: a trace event name, never displayed
/// Typed after the arrows.
const AFTER: &str = " X";
/// Ctrl+Minus presses before the caret is placed.
const ZOOM_OUTS: usize = 3;
/// Blank paper near the top of page 1, in PDF points.
const CARET: (f64, f64) = (200.0, 700.0);
/// Each arrow twice: enough for a focus walk to leave the page and reach a
/// neighbour or the ribbon.
const ARROWS: [&str; 8] = [
    "ArrowDown",
    "ArrowDown",
    "ArrowUp",
    "ArrowUp",
    "ArrowLeft",
    "ArrowLeft",
    "ArrowRight",
    "ArrowRight",
];

/// Arrow keys in a text draft go only to the caret.
pub struct ArrowsInATextDraftStayWithTheCaret;

impl Check for ArrowsInATextDraftStayWithTheCaret {
    fn name(&self) -> &'static str {
        "arrows_in_a_text_draft_stay_with_the_caret"
    }

    fn defect(&self) -> &'static str {
        "while text is being edited on a page, the arrow keys also walk egui's focus: a ribbon \
         button lights up, and Up and Down move to another page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.add_text",
            "draft-arrows",
            drive,
        )
    }
}

/// The current page index the canvas last traced.
fn current_page(session: &Session) -> Result<usize> {
    session
        .trace()?
        .last(CANVAS)
        .and_then(|l| l.get_usize("page"))
        .ok_or_else(|| Error::new(format!("the canvas traced no `{CANVAS} page=`.")))
}

/// The draft's length, from the last `text-edit-typing` line with a draft open.
fn draft_len(session: &Session) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .events(TYPING)
        .filter(|l| l.get("draft") == Some("true"))
        .last()
        .and_then(|l| l.get_usize("len")))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    // Zoomed out so the next page is on screen: a focus walk then lands on it
    // and scrolls there.
    for _ in 0..ZOOM_OUTS {
        pointer.key(session, None, "Minus", Some("ctrl"))?;
        session.settle(15);
    }
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, CARET)?;
    pointer.type_text(session, None, "abc")?;
    session.settle(20);
    let trace = session.trace()?;
    if trace.last_after(FOCUS, mark).is_none() {
        return Err(Error::new(format!(
            "the click that opened the draft traced no `{FOCUS}` line, so this build cannot \
             show a focus walk and no later silence means anything. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let before_len = draft_len(session)?.ok_or_else(|| {
        Error::new(format!(
            "clicking the page with Add text armed and typing opened no draft. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    let page = current_page(session)?;
    let mark = session.trace()?.mark();
    for key in ARROWS {
        pointer.key(session, None, key, None)?;
        session.settle(8);
    }
    pointer.type_text(session, None, AFTER)?;
    session.settle(10);
    // Enter presses whatever button holds focus: this is how a focus walk
    // becomes a ribbon command or a page turn.
    // A modal opened by that command stops the frames, so the step can go
    // unacknowledged; the trace is read before the step's error is believed.
    let entered = pointer.key(session, None, "Enter", None);
    session.settle(20);
    if let Some(pressed) = shell_trace(session)?.last_after(INVOKED, mark) {
        return Ok(Some(format!(
            "Enter in an open text draft, after the arrow keys, also ran a ribbon command:              `{}`. Trace: {}.",
            pressed.raw,
            session.trace_path().display()
        )));
    }
    entered?;
    let trace = session.trace()?;
    let walked: Vec<String> = trace
        .events(FOCUS)
        .filter(|l| l.lineno > mark)
        .map(|l| l.raw.clone())
        .collect();
    let after_page = current_page(session)?;
    let after_len = draft_len(session)?.unwrap_or(0);
    let visible = trace
        .last(CANVAS)
        .and_then(|l| l.get_usize("visible"))
        .unwrap_or(0);
    report.note(format!(
        "{visible} pages visible; page {page} → {after_page}, draft {before_len} → {after_len} characters, {} focus moves",
        walked.len()
    ));
    if trace.last_after(ENTER, mark).is_none() {
        return Err(Error::new(format!(
            "Enter after the arrows never reached the draft (no `{ENTER}`). Trace: {}.",
            session.trace_path().display()
        )));
    }
    if let Some(first) = walked.first() {
        return Ok(Some(format!(
            "the arrow keys in an open text draft moved egui's keyboard focus: `{first}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    if after_page != page {
        return Ok(Some(format!(
            "the arrow keys in an open text draft turned the page from index {page} to \
             {after_page}. Trace: {}.",
            session.trace_path().display()
        )));
    }
    // The characters typed and the line break.
    let typed = AFTER.chars().count() + 1;
    Ok((after_len != before_len + typed).then(|| {
        format!(
            "after the arrows, typing `{AFTER}` took the draft from {before_len} to \
             {after_len} characters: the draft lost the keyboard. Trace: {}.",
            session.trace_path().display()
        )
    }))
}
