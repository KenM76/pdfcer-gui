//! `a_document_moves_with_its_edits_and_password` — an encrypted document
//! with an unsaved rotation moves to another window, which opens it without
//! asking for its password, holds the rotation as unsaved, and saves it into
//! the original file still encrypted; that window then rotates it again and
//! tears it off into a new window, edits and all.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/window_carry.md`.

use super::encrypted_edit::{answer, await_new, press};
use super::window_move::{
    self, EMPTIED, OFFSCREEN, RECEIVED, Reaper, SENT, TAB, TABS, TO_NEW, TO_OTHER, TORN_OFF,
    Window, await_line, exits, launch_at, peers_reach, torn_off_opened,
};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};

const ENCRYPTED: &str = "encrypted-aes-128.pdf";
const UI_RECT: &str = "ui-rect";
const FIELD: &str = "password.field"; // ui-text-exempt: a trace region name, never displayed
const RAIL_ROTATE: &str = "rail.rotate.pages.rotate_right"; // ui-text-exempt: a trace region name, never displayed
const ROTATED: &str = "rotate-pages"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
const OPENED: &str = "window-move-opened"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ADocumentMovesWithItsEditsAndPassword;

impl Check for ADocumentMovesWithItsEditsAndPassword {
    fn name(&self) -> &'static str {
        "a_document_moves_with_its_edits_and_password"
    }

    fn defect(&self) -> &'static str {
        "a document with unsaved edits cannot move to another window until it is saved, the \
         window it moves to asks for its password again, or the edits it carried are lost or \
         saved decrypted"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut torn = Reaper(None);
        match assess(ctx, &mut report, &mut torn) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(
    ctx: &CheckContext,
    report: &mut CheckReport,
    torn: &mut Reaper,
) -> Result<Option<String>> {
    let a = launch_at(ctx, report, ENCRYPTED, "carry-a", OFFSCREEN, &[])?;
    if let Some(why) = rotate(&a, "userpw")? {
        return Ok(Some(format!("the first window: {why}")));
    }
    let original = length(&a.doc)?;
    let b = launch_at(ctx, report, "four-pages.pdf", "carry-b", OFFSCREEN, &[])?;
    if let Some(failure) = send_with_edits(&a, &b, report)? {
        return Ok(Some(failure));
    }
    if length(&a.doc)? != original {
        return Ok(Some(
            "the original file changed before the receiving window saved it.".to_owned(),
        ));
    }
    if let Some(failure) = save_carried(&b, &a.doc, original, report)? {
        return Ok(Some(failure));
    }
    let judged = tear_off_with_edits(ctx, &b, &a.doc, torn, report);
    b.pointer.gone(&b.session)?;
    judged
}

/// Answer the prompt with `password`, when given one, and rotate the active
/// document.
fn rotate(w: &Window, password: &str) -> Result<Option<String>> {
    if !password.is_empty()
        && let Some(why) = answer(&w.session, &w.pointer, password)?
    {
        return Ok(Some(why));
    }
    let before = w.session.trace()?.events(ROTATED).count();
    if !press(&w.session, &w.pointer, RAIL_ROTATE)? {
        return Ok(Some(format!(
            "no `{RAIL_ROTATE}` button was declared to press."
        )));
    }
    if await_new(&w.session, ROTATED, before)?.is_none() {
        return Ok(Some(format!("the rotation traced no `{ROTATED}`.")));
    }
    Ok(None)
}

/// A sends its one document, rotated and unsaved, to B: B opens it carrying
/// the edit and the password, without a prompt, and A closes.
fn send_with_edits(a: &Window, b: &Window, report: &mut CheckReport) -> Result<Option<String>> {
    if let Some(failure) = peers_reach(a, 1)? {
        return Ok(Some(failure));
    }
    crate::checks::ocr::click_tab(&a.session, &a.pointer, UI_RECT, TAB)?;
    a.session.expect_exit();
    window_move::click(a, UI_RECT, TO_OTHER)?;
    let Some(received) = await_line(&b.session, RECEIVED, |_| true)? else {
        return Ok(Some(format!(
            "the rotated, unsaved document never reached the second window: no `{RECEIVED}`. \
             First trace: {}.",
            a.session.trace_path().display()
        )));
    };
    report.note(format!("received: `{}`", received.raw));
    if received.get("carried").is_none_or(|n| n == "0") || received.get("key") != Some("true") {
        return Ok(Some(format!(
            "the document travelled without its edits or its password: `{}`.",
            received.raw
        )));
    }
    let Some(opened) = await_line(&b.session, OPENED, |_| true)? else {
        return Ok(Some(format!("the second window traced no `{OPENED}`.")));
    };
    report.note(format!("opened: `{}`", opened.raw));
    if opened.get("ok") != Some("true") || opened.get("unsaved") != Some("true") {
        return Ok(Some(format!(
            "the second window did not open the document with its edits unsaved: `{}`.",
            opened.raw
        )));
    }
    if declared_in(&b.session.trace()?, UI_RECT, FIELD).is_some() {
        return Ok(Some(
            "the second window asked for the password the document was already opened with."
                .to_owned(),
        ));
    }
    if await_line(&b.session, TABS, |l| l.get("open") == Some("2"))?.is_none() {
        return Ok(Some(
            "the second window never showed two document tabs.".to_owned(),
        ));
    }
    let sent = await_line(&a.session, SENT, |_| true)?;
    let emptied = await_line(&a.session, EMPTIED, |_| true)?;
    if sent.is_none() || emptied.is_none() || !exits(&a.session) {
        return Ok(Some(format!(
            "the first window gave its only document away and did not close: `{SENT}` {:?}, \
             `{EMPTIED}` {:?}.",
            sent.map(|l| l.raw),
            emptied.map(|l| l.raw)
        )));
    }
    Ok(None)
}

/// B saves the document it received: the original file gains a revision
/// holding the rotation, still under its encryption.
fn save_carried(
    b: &Window,
    doc: &std::path::Path,
    original: u64,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    b.pointer.key(&b.session, None, "S", Some("ctrl"))?;
    b.session.settle(40);
    let Some(save) = await_new(&b.session, SAVED, 0)? else {
        return Ok(Some(format!("Ctrl+S traced no `{SAVED}` line.")));
    };
    if save.get("outcome") != Some("ok") {
        return Ok(Some(format!("the save failed: `{}`.", save.raw)));
    }
    let bytes = std::fs::read(doc).map_err(|e| Error::new(e.to_string()))?;
    let appended = bytes.get(usize::try_from(original).unwrap_or(usize::MAX)..);
    let rotated = appended.is_some_and(|tail| tail.windows(7).any(|w| w == b"/Rotate"));
    let encrypted = bytes.windows(8).any(|w| w == b"/Encrypt");
    report.note(format!(
        "saved: {original} -> {} bytes, appended /Rotate {rotated}, /Encrypt {encrypted}",
        bytes.len()
    ));
    if !rotated || !encrypted {
        return Ok(Some(format!(
            "the saved file holds the carried rotation: {rotated}; /Encrypt present: \
             {encrypted}. Both are owed."
        )));
    }
    Ok(None)
}

/// B rotates the document again and tears it off: the new window is sent the
/// edit and the password, and B's tab goes once it has them.
fn tear_off_with_edits(
    ctx: &CheckContext,
    b: &Window,
    doc: &std::path::Path,
    torn: &mut Reaper,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    if let Some(why) = rotate(b, "")? {
        return Ok(Some(format!("the second window: {why}")));
    }
    crate::checks::ocr::click_tab(&b.session, &b.pointer, UI_RECT, TAB)?;
    window_move::click(b, UI_RECT, TO_NEW)?;
    let Some(line) = await_line(&b.session, TORN_OFF, |_| true)? else {
        return Ok(Some(format!(
            "Move to new window traced no `{TORN_OFF}` line."
        )));
    };
    report.note(format!("tear-off: `{}`", line.raw));
    let Some(pid) = line.get("pid").and_then(|p| p.parse::<u32>().ok()) else {
        return Ok(Some(format!("`{}` names no process.", line.raw)));
    };
    torn.0 = Some(pid);
    if line.get("carried") != Some("true") {
        return Ok(Some(format!(
            "an unsaved, encrypted document was torn off without its edits and password: `{}`.",
            line.raw
        )));
    }
    let to = pid.to_string();
    if await_line(&b.session, SENT, |l| l.get("to") == Some(to.as_str()))?.is_none() {
        let refused = b
            .session
            .trace()?
            .last("window-move-refused")
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "the new window never took the document; refusal: {refused:?}."
        )));
    }
    if await_line(&b.session, TABS, |l| l.get("open") == Some("1"))?.is_none() {
        return Ok(Some(
            "the torn-off document's tab stayed in the window it left.".to_owned(),
        ));
    }
    torn_off_opened(ctx, pid, doc)
}

fn length(path: &std::path::Path) -> Result<u64> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| Error::new(format!("{}: {e}", path.display())))
}
