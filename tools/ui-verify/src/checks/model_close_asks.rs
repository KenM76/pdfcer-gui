//! `checks::model_close_asks` — **the 3D viewer is a window of its own, and
//! closing its document asks about it first**
//!
//! Off-screen, scripted pointer and keys only. The engine corpus's
//! `assembly.prc` is placed from the ribbon and opened with *View…*.
//!
//! Oracles, in order:
//!
//! 1. the viewer's `model-view-owner` line reaches `owned=0`: the operating
//!    system reports no owner window for it, so it minimises, restores and
//!    goes behind independently of the document's window;
//! 2. Ctrl+W on the document traces `model-close-asked reason=tab` and neither
//!    a `model-view-closed` nor a `close slot=` line;
//! 3. Cancel traces `model-close-answered choice=cancel`, and still neither;
//! 4. Ctrl+W again and *Close both* trace `model-view-closed how=document`
//!    followed by a `close slot=` line.
//!
//! Design: `docs/modules/ui-verify/checks/model_close_asks.md`.

use crate::checks::driving::declared_in;
use crate::checks::model_view_window::{
    IMAGE_REGION, VIEW_REGION, launch_with_model, press_insert, ui_rect,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const OWNER: &str = "model-view-owner";
const ASKED: &str = "model-close-asked";
const ANSWERED: &str = "model-close-answered";
const CLOSED: &str = "model-view-closed";
/// `close slot=… path=…`, the line every tab close traces.
const CLOSE: &str = "close";
const ACCEPT: &str = "dialog.buttons.accept";
const CANCEL: &str = "dialog.buttons.cancel";
/// Tries, ten frames apart, for a line to appear.
/// The document's unsaved-edits question going up.
const UNSAVED_ASKED: &str = "unsaved-asked"; // ui-text-exempt: a trace event name, never displayed
/// Its discard button.
const DISCARD: &str = "unsaved.discard"; // ui-text-exempt: a trace region name
const WAITS: usize = 40;

/// See the module documentation.
pub struct ClosingTheDocumentAsksAboutItsModelWindow;

impl Check for ClosingTheDocumentAsksAboutItsModelWindow {
    fn name(&self) -> &'static str {
        "closing_the_document_asks_about_its_model_window"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer minimises or goes behind with the document's window, or closing the \
         document closes the viewer without asking, or Cancel closes either, or Close both \
         leaves one open"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model(ctx, &mut report, "model-close")
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let failure = match open_viewer(ctx, session, pointer)? {
        Some(failure) => Some(failure),
        None => match independent(report, session)? {
            Some(failure) => Some(failure),
            None => match ask_and_cancel(ctx, report, session, pointer)? {
                Some(failure) => Some(failure),
                None => close_both(ctx, report, session, pointer)?,
            },
        },
    };
    // A failure found outranks a parking error: the viewer may be gone.
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Place the model and open it with *View…*.
fn open_viewer(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    let Some((view, vp)) = declared_in(&session.trace()?, ui_rect, VIEW_REGION) else {
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region beside the placed model."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(view))?;
    session.settle(30);
    if declared_in(&session.trace()?, ui_rect, IMAGE_REGION).is_none() {
        return Ok(Some(format!(
            "no `{IMAGE_REGION}` region: View… opened no viewer."
        )));
    }
    Ok(None)
}

/// The operating system reports no owner for the viewer's window.
fn independent(report: &mut CheckReport, session: &Session) -> Result<Option<String>> {
    for _ in 0..WAITS {
        let trace = session.trace()?;
        if trace.last(OWNER).and_then(|l| l.get("owned")) == Some("0") {
            report.note("the viewer's window has no owner window");
            return Ok(None);
        }
        session.settle(10);
    }
    let last = session.trace()?.last(OWNER).map(|l| l.raw.clone());
    Ok(Some(match last {
        Some(raw) => format!("the viewer's window kept an owner window; last `{raw}`."),
        None => format!("no `{OWNER}` line: the viewer's window was never found."),
    }))
}

/// Ctrl+W on the document: asked, and nothing closed. Then Cancel.
fn ask_and_cancel(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    if let Some(failure) = ctrl_w_asks(session, pointer, mark)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = press(ctx, session, pointer, (CANCEL, mark))? {
        return Ok(Some(failure));
    }
    let Some(answer) = wait_after(session, ANSWERED, mark)? else {
        return Ok(Some(format!("Cancel traced no `{ANSWERED}` line.")));
    };
    if answer.get("choice") != Some("cancel") || answer.get("reason") != Some("tab") {
        return Ok(Some(format!("Cancel answered `{}`.", answer.raw)));
    }
    session.settle(20);
    if let Some(failure) = nothing_closed(session, mark, "Cancel")? {
        return Ok(Some(failure));
    }
    report.note("Cancel kept the document and its 3D viewer open");
    Ok(None)
}

/// Ctrl+W asks with `reason=tab` and closes nothing while it waits.
fn ctrl_w_asks(
    session: &Session,
    pointer: &ScriptedPointer,
    mark: usize,
) -> Result<Option<String>> {
    pointer.key(session, None, "W", Some("ctrl"))?;
    session.settle(20);
    let Some(asked) = wait_after(session, ASKED, mark)? else {
        return Ok(Some(format!(
            "Ctrl+W on the document traced no `{ASKED}` line with its viewer open."
        )));
    };
    if asked.get("reason") != Some("tab") {
        return Ok(Some(format!(
            "Ctrl+W asked `{}`, not `reason=tab`.",
            asked.raw
        )));
    }
    nothing_closed(session, mark, "while the question was up,")
}

/// Neither the viewer nor the tab closed after `mark`.
fn nothing_closed(session: &Session, mark: usize, when: &str) -> Result<Option<String>> {
    let trace = session.trace()?;
    if let Some(l) = trace.last_after(CLOSED, mark) {
        return Ok(Some(format!("{when} the viewer closed: `{}`.", l.raw)));
    }
    if let Some(l) = trace
        .last_after(CLOSE, mark)
        .filter(|l| l.get("slot").is_some())
    {
        return Ok(Some(format!("{when} the document closed: `{}`.", l.raw)));
    }
    Ok(None)
}

/// Ctrl+W again and *Close both*: the viewer closes, then the tab.
fn close_both(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    if let Some(failure) = ctrl_w_asks(session, pointer, mark)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = press(ctx, session, pointer, (ACCEPT, mark))? {
        return Ok(Some(failure));
    }
    let Some(closed) = wait_after(session, CLOSED, mark)? else {
        return Ok(Some(format!("Close both traced no `{CLOSED}` line.")));
    };
    if closed.get("how") != Some("document") {
        return Ok(Some(format!(
            "Close both closed the viewer as `{}`.",
            closed.raw
        )));
    }
    let after = closed.lineno;
    // The inserted model is an unsaved edit, so the document's own question
    // follows the viewer's; discarding lets the close finish.
    if wait_after(session, UNSAVED_ASKED, after)?.is_none() {
        return Ok(Some(format!(
            "Close both closed the viewer and the document's unsaved-edits question did not follow: no `{UNSAVED_ASKED}` line, so the tab close did not resume."
        )));
    }
    if let Some(failure) = press(ctx, session, pointer, (DISCARD, after))? {
        return Ok(Some(failure));
    }
    for _ in 0..WAITS {
        let trace = session.trace()?;
        if trace
            .last_after(CLOSE, after)
            .is_some_and(|l| l.get("slot").is_some())
        {
            report.note("Close both closed the viewer, then the document");
            return Ok(None);
        }
        session.settle(10);
    }
    Ok(Some(format!(
        "Close both closed the viewer and left the document open: no `{CLOSE} slot=` line."
    )))
}

/// Click the question's button `region`, waiting for it to draw after
/// `mark`: an earlier window publishes the same button names.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    (region, mark): (&str, usize),
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    for _ in 0..WAITS {
        let trace = session.trace()?;
        let fresh = trace
            .events(ui_rect)
            .any(|l| l.lineno > mark && l.get("name") == Some(region));
        if let Some((rect, vp)) = declared_in(&trace, ui_rect, region).filter(|_| fresh) {
            pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
            session.settle(20);
            return Ok(None);
        }
        session.settle(10);
    }
    Ok(Some(format!(
        "the close question drew no `{region}` button."
    )))
}

/// The last `event` line after `mark`, waiting for it.
fn wait_after(
    session: &Session,
    event: &str,
    mark: usize,
) -> Result<Option<crate::trace::TraceLine>> {
    for _ in 0..WAITS {
        if let Some(l) = session.trace()?.last_after(event, mark) {
            return Ok(Some(l.clone()));
        }
        session.settle(10);
    }
    Ok(None)
}
