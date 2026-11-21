//! `a_moved_document_keeps_its_undo` — a document with an unsaved rotation
//! moves to another window, which replays its undo history onto the file and
//! takes the rotation back on Ctrl+Z.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/window_undo.md`.

use super::encrypted_edit::await_new;
use super::window_carry::rotate;
use super::window_move::{
    self, OFFSCREEN, RECEIVED, TAB, TO_OTHER, Window, await_line, launch_at, peers_reach,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::Result;

const UI_RECT: &str = "ui-rect";
const REPLAYED: &str = "window-move-history"; // ui-text-exempt: a trace event name, never displayed
const UNDONE: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "undo-declined"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AMovedDocumentKeepsItsUndo;

impl Check for AMovedDocumentKeepsItsUndo {
    fn name(&self) -> &'static str {
        "a_moved_document_keeps_its_undo"
    }

    fn defect(&self) -> &'static str {
        "a document moved to another window arrives with its unsaved edits but without the \
         steps that made them, so nothing done before the move can be undone"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let a = launch_at(ctx, report, "four-pages.pdf", "undo-a", OFFSCREEN, &[])?;
    if let Some(why) = rotate(&a, "")? {
        return Ok(Some(format!("the first window: {why}")));
    }
    let b = launch_at(ctx, report, "paragraph.pdf", "undo-b", OFFSCREEN, &[])?;
    let judged = move_and_undo(&a, &b, report);
    b.pointer.gone(&b.session)?;
    judged
}

/// A sends its rotated document to B; B must replay one undo step and take
/// it back on Ctrl+Z.
fn move_and_undo(a: &Window, b: &Window, report: &mut CheckReport) -> Result<Option<String>> {
    if let Some(failure) = peers_reach(a, 1)? {
        return Ok(Some(failure));
    }
    crate::checks::ocr::click_tab(&a.session, &a.pointer, UI_RECT, TAB)?;
    a.session.expect_exit();
    window_move::click(a, UI_RECT, TO_OTHER)?;
    if await_line(&b.session, RECEIVED, |_| true)?.is_none() {
        return Ok(Some(format!(
            "the rotated document never reached the second window: no `{RECEIVED}`."
        )));
    }
    let Some(replayed) = await_line(&b.session, REPLAYED, |_| true)? else {
        return Ok(Some(format!(
            "the second window replayed no undo history: no `{REPLAYED}` line. Trace: {}.",
            b.session.trace_path().display()
        )));
    };
    report.note(format!("replayed: `{}`", replayed.raw));
    if replayed.get("replayed") != Some("true") || replayed.get("undo") != Some("1") {
        return Ok(Some(format!(
            "the rotation did not arrive as one undo step: `{}`.",
            replayed.raw
        )));
    }
    let before = b.session.trace()?.events(UNDONE).count();
    b.pointer.key(&b.session, None, "Z", Some("ctrl"))?;
    b.session.settle(30);
    let Some(undone) = await_new(&b.session, UNDONE, before)? else {
        let declined = b.session.trace()?.last(DECLINED).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "Ctrl+Z in the second window undid nothing; decline: {declined:?}."
        )));
    };
    report.note(format!("undone: `{}`", undone.raw));
    Ok(None)
}
