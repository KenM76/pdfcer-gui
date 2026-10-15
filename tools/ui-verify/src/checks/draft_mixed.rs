//! `a_mixed_paragraph_opens_whole_and_keeps_its_looks` — a paragraph that
//! mixes fonts and colours opens from a click as one draft, and an edit keeps
//! every look. See `docs/modules/ui-verify/checks/draft_mixed.md`.

use super::dimdrive::{Fixture, run_on};
use super::draft_paragraph::{applied, field, reopen};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "mixed-paragraph.pdf",
    method: "`python tools/gen-mixed-paragraph-fixture.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const BLOCK_APPLIED: &str = "edit-block-text-applied"; // ui-text-exempt: a trace event name, never displayed
/// On the first line's glyphs: baseline 700, left edge 72, size 12.
const ON_FIRST_LINE: (f64, f64) = (90.0, 703.0);
/// The paragraph's lines, as the fixture writes them.
const LINES: usize = 6;

/// A mixed paragraph opens whole and keeps its looks.
pub struct AMixedParagraphOpensWholeAndKeepsItsLooks;

impl Check for AMixedParagraphOpensWholeAndKeepsItsLooks {
    fn name(&self) -> &'static str {
        "a_mixed_paragraph_opens_whole_and_keeps_its_looks"
    }

    fn defect(&self) -> &'static str {
        "a paragraph with a bold or coloured word opens one line at a time, or an edit of it sets \
         every word in the first line's look (OPERATOR_REQUESTS.md O295)"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "draft-mixed",
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
    let opened = match reopen(ctx, session, pointer, &FIXTURE, ON_FIRST_LINE)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!("opened: `{}`", opened.raw));
    let looks = field(&opened, "looks");
    if field(&opened, "lines") != LINES || !(2..usize::MAX).contains(&looks) {
        return Ok(Some(format!(
            "the mixed paragraph opened as `{}`; want lines={LINES} and looks of 2 or more \
             (Helvetica, and the bold red word). Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    }
    pointer.type_text(session, None, "Z")?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Enter", Some("ctrl"))?;
    applied(
        session,
        BLOCK_APPLIED,
        mark,
        "Ctrl+Enter on the mixed paragraph",
    )?;
    let again = match reopen(ctx, session, pointer, &FIXTURE, ON_FIRST_LINE)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(format!("after an edit, {failure}"))),
    };
    report.note(format!("re-opened: `{}`", again.raw));
    Ok((field(&again, "looks") != looks).then(|| {
        format!(
            "after one character was typed and committed, the paragraph re-opened as `{}` with \
             looks={}, not {looks}: the rewrite set the bold red word in another look. Trace: \
             {}.",
            again.raw,
            field(&again, "looks"),
            session.trace_path().display()
        )
    }))
}
