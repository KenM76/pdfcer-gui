//! `word_gap_opens_from_the_text_being_edited` — *Word gap…* on the caret's
//! menu opens Settings at the Text page, where the word-gap slider is. See
//! `docs/modules/ui-verify/checks/word_gap_here.md`.

use super::dimdrive::{Fixture, click_on, press, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "paragraph.pdf",
    method: "Regenerate it: python tools/gen-reflow-fixture.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

const ROW: &str = "menu.item.canvas.text.edit.word_gap"; // ui-text-exempt: a trace region name, never displayed
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
const PAGE: &str = "settings-page"; // ui-text-exempt: a trace event name, never displayed
/// Inside line 2 of the six-line block.
const AT: (f64, f64) = (120.0, 668.0);

/// See the module documentation.
pub struct WordGapOpensFromTheTextBeingEdited;

impl Check for WordGapOpensFromTheTextBeingEdited {
    fn name(&self) -> &'static str {
        "word_gap_opens_from_the_text_being_edited"
    }

    fn defect(&self) -> &'static str {
        "The word-gap setting, which decides how much of a line one click opens, is reachable \
         only from Settings: Word gap… on the text menu is missing or lands on another page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "word-gap-here",
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
    session.settle(30);
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, AT)?;
    if session.trace()?.last_after(CARET, mark).is_none() {
        return Ok(Some(format!(
            "no `{CARET}` after a click in the paragraph; nothing here can be judged."
        )));
    }
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let point = mapping.doc_to_window(DocPoint::new(0, AT.0, AT.1))?;
    let mark = session.trace()?.mark();
    pointer.right_click(session, point)?;
    session.settle(30);
    press(ctx, session, pointer, ROW)?;
    session.settle(50);
    let trace = session.trace()?;
    let Some(page) = trace.last_after(PAGE, mark) else {
        return Ok(Some(format!(
            "★ `{ROW}` was pressed and no Settings page was drawn after it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("landed: `{}`", page.raw));
    Ok((page.get("key") != Some("text")).then(|| {
        format!(
            "★★ Word gap… opened Settings at `{}`, not the Text page that holds the slider.",
            page.raw
        )
    }))
}
