//! `hand_breaks_are_read_from_marks` — a paragraph pdfcer has written keeps a
//! line broken by hand after its longest line, and one whose breaks were only
//! judged from layout says so off-canvas.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/hand_breaks.md`.

use super::dimdrive::{Fixture, click_on, run_on};
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
const WIDENED: &str = "text-edit-widened"; // ui-text-exempt: a trace event name, never displayed
const NOTE: &str = "text-edit-note"; // ui-text-exempt: a trace event name, never displayed
const REGION: &str = "status-group:draft-note"; // ui-text-exempt: a trace region name, never displayed
const ADDED: &str = "add-text"; // ui-text-exempt: a trace event name, never displayed
const BLOCK_APPLIED: &str = "edit-block-text-applied"; // ui-text-exempt: a trace event name, never displayed
/// A long line then a one-word line: the word fits nowhere but after the
/// longest line, so layout alone reads the break as a wrap.
const TYPED: [&str; 2] = ["A much longer first line", "x"];
const TYPED_AT: (f64, f64) = (100.0, 400.0);
/// A point on the first typed line's glyphs.
const ON_TYPED: (f64, f64) = (110.0, 403.0);

/// Hand breaks are read from pdfcer's marks.
pub struct HandBreaksAreReadFromMarks;

impl Check for HandBreaksAreReadFromMarks {
    fn name(&self) -> &'static str {
        "hand_breaks_are_read_from_marks"
    }

    fn defect(&self) -> &'static str {
        "a line broken by hand after a paragraph's longest line re-opens joined to the next, and \
         nothing says the breaks were guessed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "hand-breaks",
            drive,
        )
    }
}

/// Type the two lines, broken by Enter, into the open draft.
fn type_lines(session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    pointer.type_text(session, None, TYPED[0])?;
    pointer.key(session, None, "Enter", None)?;
    pointer.type_text(session, None, TYPED[1])?;
    Ok(())
}

/// Ctrl+Enter and wait for `event`.
fn commit(session: &Session, pointer: &ScriptedPointer, event: &str, what: &str) -> Result<()> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Enter", Some("ctrl"))?;
    session.settle(40);
    if session.trace()?.last_after(event, mark).is_none() {
        return Err(Error::new(format!(
            "{what} wrote nothing (no `{event}`). Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok(())
}

/// What one re-open traced: its `text-edit-widened` line, the note it raised
/// and whether the note's status region was drawn.
struct Reopened {
    line: crate::trace::TraceLine,
    note: Option<String>,
    disclosed: bool,
}

fn reopen(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<Reopened, String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_TYPED)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(WIDENED, mark).cloned() else {
        return Ok(Err(format!(
            "a click on the typed paragraph opened no paragraph draft. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let note = trace.last_after(NOTE, mark).map(|l| l.raw.clone());
    let disclosed = trace
        .events(ui_rect)
        .filter(|l| l.lineno > mark)
        .any(|l| l.get("name") == Some(REGION));
    Ok(Ok(Reopened {
        line,
        note,
        disclosed,
    }))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    // Written by Add text, which marks no breaks: the re-open infers them.
    click_on(ctx, session, pointer, &FIXTURE, 0, TYPED_AT)?;
    type_lines(session, pointer)?;
    commit(session, pointer, ADDED, "Ctrl+Enter on two typed lines")?;
    let inferred = match reopen(ctx, session, pointer)? {
        Ok(r) => r,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!("added: `{}`", inferred.line.raw));
    let said = inferred
        .note
        .as_deref()
        .is_some_and(|n| n.contains("note=breaks-inferred"));
    if inferred.line.get("source") != Some("inferred") || !said || !inferred.disclosed {
        return Ok(Some(format!(
            "a paragraph with no break marks re-opened as `{}`, note {:?}, `{REGION}` drawn: {}; \
             want source=inferred, and the breaks-inferred note shown. Trace: {path}.",
            inferred.line.raw, inferred.note, inferred.disclosed
        )));
    }
    // Retyped and committed through the paragraph, which marks its breaks.
    pointer.key(session, None, "a", Some("ctrl"))?;
    type_lines(session, pointer)?;
    commit(
        session,
        pointer,
        BLOCK_APPLIED,
        "Ctrl+Enter on the re-opened paragraph",
    )?;
    let marked = match reopen(ctx, session, pointer)? {
        Ok(r) => r,
        Err(failure) => return Ok(Some(format!("after an edit, {failure}"))),
    };
    report.note(format!("edited: `{}`", marked.line.raw));
    let len = TYPED.iter().map(|t| t.chars().count()).sum::<usize>() + 1;
    let ok = marked.line.get("source") == Some("marked")
        && marked.line.get_usize("breaks") == Some(1)
        && marked.line.get_usize("len") == Some(len)
        && marked.note.is_none();
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    Ok((!ok).then(|| {
        format!(
            "a paragraph pdfcer wrote re-opened as `{}`, note {:?}; want source=marked breaks=1 \
             len={len} and no note: the break typed after the longest line was read from layout. \
             Trace: {path}.",
            marked.line.raw, marked.note
        )
    }))
}
