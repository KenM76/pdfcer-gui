//! `one_paragraph_opens_as_lines_while_its_neighbour_opens_whole` — two
//! identical paragraphs on one page: *This paragraph* on the caret's menu in
//! the first sends it back to lines, and the second still opens whole; pressed
//! again on a line, the first opens whole again. See
//! `docs/modules/ui-verify/checks/paragraph_here.md`.

use super::dimdrive::{Fixture, click_on, press, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "paragraph-two.pdf",
    method: "Regenerate it: python tools/gen-reflow-fixture.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

const SWITCHED: &str = "open-paragraph-here"; // ui-text-exempt: a trace event name, never displayed
const ROW: &str = "menu.item.canvas.text.edit.paragraph_here"; // ui-text-exempt: a trace region name, never displayed
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
/// Inside line 2 of the first six-line block.
const FIRST: (f64, f64) = (120.0, 668.0);
/// The same place in the second block, 200 pt lower.
const SECOND: (f64, f64) = (120.0, 468.0);

/// See the module documentation.
pub struct OneParagraphOpensAsLinesWhileItsNeighbourOpensWhole;

impl Check for OneParagraphOpensAsLinesWhileItsNeighbourOpensWhole {
    fn name(&self) -> &'static str {
        "one_paragraph_opens_as_lines_while_its_neighbour_opens_whole"
    }

    fn defect(&self) -> &'static str {
        "Paragraphs cannot be set for one paragraph: This paragraph does nothing, or changes \
         every paragraph on the page, or cannot open a paragraph whole again"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "paragraph-here",
            drive,
        )
    }
}

/// Click at `at` and return the caret line's anchor: `run`, `block` or the
/// raw line when it is neither. Escape then drops the draft; an Escape with
/// none would disarm the tool.
fn opens(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
    escape: bool,
) -> Result<std::result::Result<&'static str, String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
    let trace = session.trace()?;
    let Some(caret) = trace.last_after(CARET, mark) else {
        return Ok(Err(format!("no `{CARET}` after the click")));
    };
    if escape {
        pointer.key(session, None, "Escape", None)?;
        session.settle(14);
    }
    Ok(if caret.get("block").is_some() {
        Ok("block")
    } else if caret.get("run").is_some() {
        Ok("run")
    } else {
        Err(caret.raw.clone())
    })
}

/// Right-click at `at`, press *This paragraph* and drop the draft; the
/// switch line it traced.
fn switch(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<std::result::Result<String, String>> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let point = mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?;
    let mark = session.trace()?.mark();
    pointer.right_click(session, point)?;
    session.settle(30);
    press(ctx, session, pointer, ROW)?;
    pointer.key(session, None, "Escape", None)?;
    session.settle(14);
    let trace = session.trace()?;
    Ok(trace.last_after(SWITCHED, mark).map_or_else(
        || {
            Err(format!(
                "`{ROW}` was clicked and raised no `{SWITCHED}` line. Trace: {}.",
                session.trace_path().display()
            ))
        },
        |l| Ok(l.raw.clone()),
    ))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    let before = opens(ctx, session, pointer, FIRST, false)?;
    if before != Ok("block") {
        return Ok(Some(format!(
            "with Paragraphs on, a click in the first paragraph opened {before:?}, not the \
             paragraph; nothing here can be judged."
        )));
    }
    let off = match switch(ctx, session, pointer, FIRST)? {
        Ok(l) => l,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("off: `{off}`"));
    if !off.contains("enabled=false") || !off.contains("now=true") {
        return Ok(Some(format!(
            "This paragraph in an open paragraph did not send it back to lines: `{off}`."
        )));
    }
    let first = opens(ctx, session, pointer, FIRST, true)?;
    let second = opens(ctx, session, pointer, SECOND, true)?;
    report.note(format!("first opened {first:?}, second {second:?}"));
    match (&first, &second) {
        (Ok("run"), Ok("block")) => {}
        (Ok("block"), _) => {
            return Ok(Some(
                "★ with This paragraph off, a click in it still opened the whole paragraph."
                    .to_owned(),
            ));
        }
        (_, Ok("run")) => {
            return Ok(Some(
                "★★ This paragraph off in the first paragraph also turned it off for the \
                 second: the click there opened one line."
                    .to_owned(),
            ));
        }
        (a, b) => {
            return Ok(Some(format!(
                "a click opened neither a line nor a paragraph: first {a:?}, second {b:?}."
            )));
        }
    }
    let _ = opens(ctx, session, pointer, FIRST, false)?;
    let on = match switch(ctx, session, pointer, FIRST)? {
        Ok(l) => l,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("on: `{on}`"));
    if !on.contains("enabled=true") || !on.contains("now=true") {
        return Ok(Some(format!(
            "This paragraph on a line of a paragraph sent back to lines did not open it \
             whole: `{on}`."
        )));
    }
    let again = opens(ctx, session, pointer, FIRST, false)?;
    report.note(format!("first opened {again:?} after the second press"));
    Ok((again != Ok("block")).then(|| {
        format!("after This paragraph was pressed again, a click in it opened {again:?}.")
    }))
}
