//! `hand_breaks_are_read_from_marks` — a paragraph pdfcer has written keeps a
//! line broken by hand after its longest line; `inferred_breaks_are_disclosed`
//! — one whose breaks were only judged from layout says so off-canvas.
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
/// The last line the re-opened paragraph is retyped with: as short as `x`.
const RETYPED: &str = "y";
const TYPED_AT: (f64, f64) = (100.0, 400.0);
/// A point on the first typed line's glyphs.
const ON_TYPED: (f64, f64) = (110.0, 403.0);

/// A point on line 2 of `paragraph.pdf`'s paragraph, which carries no marks.
const ON_FOREIGN: (f64, f64) = (120.0, 668.0);

/// Hand breaks are read from pdfcer's marks.
pub struct HandBreaksAreReadFromMarks;

/// A paragraph with no break marks says its breaks were inferred.
pub struct InferredBreaksAreDisclosed;

impl Check for InferredBreaksAreDisclosed {
    fn name(&self) -> &'static str {
        "inferred_breaks_are_disclosed"
    }

    fn defect(&self) -> &'static str {
        "a paragraph whose line breaks were judged from layout opens with nothing saying they \
         were guessed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &super::reflow::FIXTURE,
            "mode.edit,edit.text",
            "breaks-inferred",
            drive_inferred,
        )
    }
}

/// Open the foreign paragraph; the failure sentence unless it traced
/// `source=inferred` and drew the breaks-inferred note.
fn drive_inferred(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let opened = match reopen(ctx, session, pointer, &super::reflow::FIXTURE, ON_FOREIGN)? {
        Ok(r) => r,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!("opened: `{}`", opened.line.raw));
    let said = opened
        .note
        .as_deref()
        .is_some_and(|n| n.contains("note=breaks-inferred"));
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    Ok(
        (opened.line.get("source") != Some("inferred") || !said || !opened.disclosed).then(|| {
            format!(
                "a paragraph with no break marks opened as `{}`, note {:?}, `{REGION}` drawn: {}; \
                 want source=inferred, and the breaks-inferred note shown. Trace: {}.",
                opened.line.raw,
                opened.note,
                opened.disclosed,
                session.trace_path().display()
            )
        }),
    )
}

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

/// Type the long line, Enter, then `last` into the open draft.
fn type_lines(session: &Session, pointer: &ScriptedPointer, last: &str) -> Result<()> {
    pointer.type_text(session, None, TYPED[0])?;
    pointer.key(session, None, "Enter", None)?;
    pointer.type_text(session, None, last)?;
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
    fixture: &Fixture,
    at: (f64, f64),
) -> Result<std::result::Result<Reopened, String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, fixture, 0, at)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(WIDENED, mark).cloned() else {
        return Ok(Err(format!(
            "a click on the paragraph opened no paragraph draft. Trace: {}.",
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
    let len = TYPED.iter().map(|t| t.chars().count()).sum::<usize>() + 1;
    let kept = |r: &Reopened| {
        r.line.get("source") == Some("marked")
            && r.line.get_usize("breaks") == Some(1)
            && r.line.get_usize("len") == Some(len)
            && r.note.is_none()
    };
    // Written by Add text, which marks its breaks as the paragraph does.
    click_on(ctx, session, pointer, &FIXTURE, 0, TYPED_AT)?;
    type_lines(session, pointer, TYPED[1])?;
    commit(session, pointer, ADDED, "Ctrl+Enter on two typed lines")?;
    let added = match reopen(ctx, session, pointer, &FIXTURE, ON_TYPED)? {
        Ok(r) => r,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!("added: `{}`", added.line.raw));
    if !kept(&added) {
        return Ok(Some(format!(
            "a paragraph Add text wrote re-opened as `{}`, note {:?}; want source=marked \
             breaks=1 len={len} and no note. Trace: {path}.",
            added.line.raw, added.note
        )));
    }
    // Retyped with a different last word, so the paragraph writes, and
    // committed through the paragraph, which marks its breaks.
    pointer.key(session, None, "a", Some("ctrl"))?;
    type_lines(session, pointer, RETYPED)?;
    commit(
        session,
        pointer,
        BLOCK_APPLIED,
        "Ctrl+Enter on the re-opened paragraph",
    )?;
    let marked = match reopen(ctx, session, pointer, &FIXTURE, ON_TYPED)? {
        Ok(r) => r,
        Err(failure) => return Ok(Some(format!("after an edit, {failure}"))),
    };
    report.note(format!("edited: `{}`", marked.line.raw));
    let ok = kept(&marked);
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
