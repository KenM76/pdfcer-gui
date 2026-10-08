//! `a_paragraph_reopens_as_one_draft` — text written as two or more lines
//! re-opens, from a click on any of them, as one paragraph draft, and an edit
//! keeps the line the author broke by hand.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/draft_paragraph.md`.

use super::dimdrive::{Fixture, click_on, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
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
const DECLINED: &str = "text-edit-widen-declined"; // ui-text-exempt: a trace event name, never displayed
const ADDED: &str = "add-text"; // ui-text-exempt: a trace event name, never displayed
const BLOCK_APPLIED: &str = "edit-block-text-applied"; // ui-text-exempt: a trace event name, never displayed
/// Two lines broken by Enter, written from a click at `TYPED_AT`. The first
/// is short enough for the second's first word to fit after it, so only a
/// kept break stops a re-wrap from merging them.
const TYPED: [&str; 2] = ["Notes", "second line here"];
const TYPED_AT: (f64, f64) = (100.0, 400.0);
/// A point on the first typed line's glyphs.
const ON_TYPED: (f64, f64) = (110.0, 403.0);
/// One line of words, typed into a box too narrow for it.
const WRAPPED: &str = "alpha beta gamma delta epsilon zeta eta theta";
const BOX: ((f64, f64), (f64, f64)) = ((100.0, 300.0), (220.0, 220.0));
/// A point on the box's first line.
const ON_WRAPPED: (f64, f64) = (110.0, 290.0);

/// A paragraph re-opens as one draft.
pub struct AParagraphReopensAsOneDraft;

impl Check for AParagraphReopensAsOneDraft {
    fn name(&self) -> &'static str {
        "a_paragraph_reopens_as_one_draft"
    }

    fn defect(&self) -> &'static str {
        "text written as two or more lines re-opens as separate one-line drafts, so a paragraph \
         cannot be edited as one"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "draft-paragraph",
            drive,
        )
    }
}

/// Wait for `event` after `mark`, or say what was being committed.
fn applied(session: &Session, event: &str, mark: usize, what: &str) -> Result<()> {
    session.settle(40);
    if session.trace()?.last_after(event, mark).is_none() {
        return Err(Error::new(format!(
            "{what} wrote nothing (no `{event}`). Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok(())
}

/// Click `at` and return the `text-edit-widened` line it traced, or the
/// failure naming what it traced instead.
fn reopen(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<std::result::Result<crate::trace::TraceLine, String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
    let trace = session.trace()?;
    if let Some(line) = trace.last_after(WIDENED, mark) {
        return Ok(Ok(line.clone()));
    }
    let instead = trace
        .last_after(DECLINED, mark)
        .or_else(|| trace.last_after("text-edit-caret", mark))
        .map_or_else(|| "nothing".to_owned(), |l| format!("`{}`", l.raw));
    Ok(Err(format!(
        "a click on ({:.0}, {:.0}), on a paragraph of two lines, opened no paragraph draft; it \
         traced {instead}. Trace: {}.",
        at.0,
        at.1,
        session.trace_path().display()
    )))
}

fn field(line: &crate::trace::TraceLine, key: &str) -> usize {
    line.get_usize(key).unwrap_or(usize::MAX)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    // Two lines broken by hand.
    click_on(ctx, session, pointer, &FIXTURE, 0, TYPED_AT)?;
    pointer.type_text(session, None, TYPED[0])?;
    pointer.key(session, None, "Enter", None)?;
    pointer.type_text(session, None, TYPED[1])?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Enter", Some("ctrl"))?;
    applied(session, ADDED, mark, "Ctrl+Enter on two typed lines")?;
    let line = match reopen(ctx, session, pointer, ON_TYPED)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!("typed: `{}`", line.raw));
    let len = TYPED.iter().map(|t| t.chars().count()).sum::<usize>() + 1;
    if field(&line, "lines") != 2 || field(&line, "breaks") != 1 || field(&line, "len") != len {
        return Ok(Some(format!(
            "the two typed lines re-opened as `{}`; want lines=2 breaks=1 len={len}, the break \
             typed with Enter kept. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    // An edit commits through the paragraph and keeps the hand break.
    pointer.type_text(session, None, "Z")?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Enter", Some("ctrl"))?;
    applied(
        session,
        BLOCK_APPLIED,
        mark,
        "Ctrl+Enter on the re-opened paragraph",
    )?;
    let again = match reopen(ctx, session, pointer, ON_TYPED)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(format!("after an edit, {failure}"))),
    };
    report.note(format!("edited: `{}`", again.raw));
    if field(&again, "lines") != 2 || field(&again, "breaks") != 1 {
        return Ok(Some(format!(
            "after one character was typed and committed, the paragraph re-opened as `{}`: \
             the line broken by hand was merged. Trace: {}.",
            again.raw,
            session.trace_path().display()
        )));
    }
    // Close the re-opened draft unchanged; Escape with none open disarms.
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    wrapped(ctx, report, session, pointer)
}

/// One line typed into a box wraps; it re-opens as one paragraph with no
/// hand break.
fn wrapped(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let from = mapping.doc_to_window(DocPoint::new(0, BOX.0.0, BOX.0.1))?;
    let to = mapping.doc_to_window(DocPoint::new(0, BOX.1.0, BOX.1.1))?;
    pointer.drag(session, from, to, 8)?;
    session.settle(20);
    pointer.type_text(session, None, WRAPPED)?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Enter", Some("ctrl"))?;
    // A box that closes before typing reaches it is this check's defect too:
    // under Edit text the draft must survive the drag.
    if let Err(e) = applied(
        session,
        ADDED,
        mark,
        "Ctrl+Enter on a box dragged under Edit text",
    ) {
        return Ok(Some(e.to_string()));
    }
    let line = match reopen(ctx, session, pointer, ON_WRAPPED)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(format!("for a wrapped box, {failure}"))),
    };
    report.note(format!("wrapped: `{}`", line.raw));
    let lines = field(&line, "lines");
    Ok((!(2..usize::MAX).contains(&lines)
        || field(&line, "breaks") != 0
        || field(&line, "len") != WRAPPED.chars().count())
    .then(|| {
        format!(
            "one line typed into a box re-opened as `{}`; want two or more lines, breaks=0 \
             and len={}, the wraps re-made by the engine. Trace: {}.",
            line.raw,
            WRAPPED.chars().count(),
            session.trace_path().display()
        )
    }))
}
