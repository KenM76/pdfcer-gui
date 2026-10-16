//! `paragraphs_off_on_one_page_leaves_the_next` — press *Paragraphs on this
//! page* on page 1 of two identical pages: a click in page 1's paragraph opens
//! one line, and the same click on page 2 still opens the paragraph. See
//! `docs/modules/ui-verify/checks/paragraphs_page.md`.

use super::dimdrive::{Fixture, click_on, press_on_tab, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "paragraph-2p.pdf",
    method: "Regenerate it: python tools/gen-reflow-fixture.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

const SWITCHED: &str = "open-paragraphs-pages"; // ui-text-exempt: a trace event name, never displayed
const TAB: &str = "ribbon.tab.edit"; // ui-text-exempt: a trace region name, never displayed
const BUTTON: &str = "ribbon.item.edit.paragraphs_page"; // ui-text-exempt: a trace region name, never displayed
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
/// Inside line 2 of the six-line block, as `paragraphs_off` uses.
const CARET_AT: (f64, f64) = (120.0, 668.0);

/// See the module documentation.
pub struct ParagraphsOffOnOnePageLeavesTheNext;

impl Check for ParagraphsOffOnOnePageLeavesTheNext {
    fn name(&self) -> &'static str {
        "paragraphs_off_on_one_page_leaves_the_next"
    }

    fn defect(&self) -> &'static str {
        "Paragraphs cannot be set for one page: switching it for page 1 does nothing there, \
         or changes every page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,edit.text",
            "paragraphs-page",
            drive,
        )
    }
}

/// Click at [`CARET_AT`] on `page` and return the caret line's anchor:
/// `run`, `block` or the raw line when it is neither.
fn opens(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    page: usize,
) -> Result<std::result::Result<&'static str, String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, page, CARET_AT)?;
    pointer.screenshot(session, &ctx.out(&format!("paragraphs-page-{page}.png")))?;
    let trace = session.trace()?;
    let Some(caret) = trace.last_after(CARET, mark) else {
        return Ok(Err(format!("no `{CARET}` after the click")));
    };
    Ok(if caret.get("block").is_some() {
        Ok("block")
    } else if caret.get("run").is_some() {
        Ok("run")
    } else {
        Err(caret.raw.clone())
    })
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    press_on_tab(ctx, session, pointer, BUTTON, TAB)?;
    let trace = session.trace()?;
    let Some(switched) = trace.last(SWITCHED) else {
        return Ok(Some(format!(
            "`{BUTTON}` was clicked and raised no `{SWITCHED}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("switched: `{}`", switched.raw));
    if switched.get("enabled") != Some("false") || switched.get("pages") != Some("0") {
        return Ok(Some(format!(
            "pressing the switch on page 1 did not turn paragraphs off for page 1 alone: `{}`.",
            switched.raw
        )));
    }
    let first = opens(ctx, session, pointer, 0)?;
    report.note(format!("page 1 opened: {first:?}"));
    pointer.key(session, None, "Escape", None)?;
    session.settle(14);
    pointer.key(session, None, "PageDown", None)?;
    session.settle(30);
    let second = opens(ctx, session, pointer, 1)?;
    report.note(format!("page 2 opened: {second:?}"));
    Ok(match (first, second) {
        (Ok("run"), Ok("block")) => None,
        (Ok("block"), _) => Some(
            "★ with Paragraphs off for page 1, a click there still opened the whole paragraph."
                .to_owned(),
        ),
        (_, Ok("run")) => Some(
            "★★ Paragraphs off for page 1 also turned it off for page 2: the click there \
             opened one line."
                .to_owned(),
        ),
        (a, b) => Some(format!(
            "a click opened neither a line nor a paragraph: page 1 {a:?}, page 2 {b:?}. \
             Trace: {}.",
            session.trace_path().display()
        )),
    })
}
