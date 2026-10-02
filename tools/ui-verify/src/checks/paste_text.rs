//! `checks::paste_text` — **text another program copied pastes as a text box
//! at the pointer**
//!
//! Drives the window off the desktop through the scripted pointer on a copy of
//! the engine corpus's `four-pages.pdf`, with the clipboard snapshotted and
//! restored as in [`super::os_image_paste`]. Two lines of text are pasted in
//! Edit, then in Review, then in Read.
//!
//! Oracles: in Edit a `clip-pasted kind=text as=content` box whose top-left
//! corner is within [`TOLERANCE_PT`] of the pointer, an `add-text n=2` line
//! and an `undo-applied` line after Ctrl+Z; in Review a `clip-pasted kind=text as=comment` box topped at the
//! pointer and an `add-text-annot` line; in Read no `clip-pasted` line and a
//! `command-declined id=edit.paste` line.

use super::os_image_paste::{self as osp, ClipGuard};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::sys;

const STEM: &str = "paste-text";
const WORDS: &str = "Hello\r\nWorld";
const CONTENT: &str = "add-text";
const COMMENT: &str = "add-text-annot";
const DECLINED: &str = "command-declined";
const UNDONE: &str = "undo-applied";
/// How far the box's top-left corner may sit from the pointer, in points.
const TOLERANCE_PT: f64 = 2.0;
const SECOND: (f64, f64) = (300.0, 300.0);

/// See the module documentation.
pub struct OsTextPastesAsATextBox;

impl Check for OsTextPastesAsATextBox {
    fn name(&self) -> &'static str {
        "text_copied_in_another_program_pastes_as_a_text_box_at_the_pointer"
    }

    fn defect(&self) -> &'static str {
        "text copied in another program pastes nothing unless a text box is already open, or \
         lands away from the pointer"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let driven = osp::launch(ctx, &mut report, STEM)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer, &mut guard));
        report.note(guard.release());
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
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    guard.set(&[(sys::CF_UNICODETEXT, osp::utf16(WORDS))])?;
    let mut failure = paste_as(ctx, report, session, pointer, osp::FIRST, "content")?;
    if failure.is_none() {
        failure = verb_ran(session, CONTENT, Some("2"))?;
    }
    if failure.is_none() {
        failure = undoes(session, pointer)?;
    }
    if failure.is_none() {
        failure = switch(session, pointer, "2")
            .and_then(|()| paste_as(ctx, report, session, pointer, SECOND, "comment"))?;
    }
    if failure.is_none() {
        failure = verb_ran(session, COMMENT, None)?;
    }
    if failure.is_none() {
        failure =
            switch(session, pointer, "1").and_then(|()| read_refuses(ctx, session, pointer))?;
    }
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Ctrl+Z takes the paste back.
fn undoes(session: &Session, pointer: &ScriptedPointer) -> Result<Option<String>> {
    let undos = osp::count(session, UNDONE)?;
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(20);
    Ok(
        (osp::count(session, UNDONE)? == undos)
            .then(|| "Ctrl+Z did not undo the paste.".to_owned()),
    )
}

/// Change mode with `Ctrl+digit`.
fn switch(session: &Session, pointer: &ScriptedPointer, digit: &str) -> Result<()> {
    pointer.key(session, None, digit, Some("ctrl"))?;
    session.settle(20);
    Ok(())
}

/// Paste at `p` and require a text box `as` `role`, topped and left-aligned
/// at the pointer.
fn paste_as(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    p: (f64, f64),
    role: &str,
) -> Result<Option<String>> {
    let before = osp::count(session, osp::PASTED)?;
    pointer.hover(session, osp::at(ctx, session, p)?)?;
    pointer.paste(session, None, WORDS)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(line) = trace.events(osp::PASTED).nth(before) else {
        return Ok(Some(format!(
            "pasting text as {role} traced no `{}` line.",
            osp::PASTED
        )));
    };
    if line.get("kind") != Some("text") || line.get("as") != Some(role) {
        return Ok(Some(format!(
            "the paste was not text as {role}: kind={} as={}",
            line.get("kind").unwrap_or("-"),
            line.get("as").unwrap_or("-")
        )));
    }
    let Some(r) = osp::rect(line) else {
        return Ok(Some(format!(
            "a `{}` line without its rectangle.",
            osp::PASTED
        )));
    };
    report.note(format!("text as {role} {}", osp::show(r)));
    let (dx, dy) = (r[0] - p.0, r[3] - p.1);
    Ok((dx.abs() > TOLERANCE_PT || dy.abs() > TOLERANCE_PT).then(|| {
        format!(
            "the {role} box's top-left corner is at ({:.2}, {:.2}), not at the pointer ({:.0}, {:.0}).",
            r[0], r[3], p.0, p.1
        )
    }))
}

/// The newest `verb` line exists, stating `n` lines when `n` is given.
fn verb_ran(session: &Session, verb: &str, n: Option<&str>) -> Result<Option<String>> {
    let trace = session.trace()?;
    Ok(match trace.events(verb).last() {
        None => Some(format!("the paste wrote nothing (no `{verb}` line).")),
        Some(line) if n.is_some() && line.get("n") != n => Some(format!(
            "`{verb}` wrote {} line(s), not {}.",
            line.get("n").unwrap_or("-"),
            n.unwrap_or("-")
        )),
        Some(_) => None,
    })
}

/// In Read a paste places nothing and is declined.
fn read_refuses(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let (pasted, declined) = (
        osp::count(session, osp::PASTED)?,
        osp::count(session, DECLINED)?,
    );
    pointer.hover(session, osp::at(ctx, session, osp::FIRST)?)?;
    pointer.paste(session, None, WORDS)?;
    session.settle(20);
    if osp::count(session, osp::PASTED)? != pasted {
        return Ok(Some("Read placed pasted text on the page.".to_owned()));
    }
    let trace = session.trace()?;
    let refused = trace
        .events(DECLINED)
        .nth(declined)
        .is_some_and(|l| l.get("id") == Some("edit.paste"));
    Ok((!refused).then(|| format!("Read traced no `{DECLINED} id=edit.paste` line.")))
}
