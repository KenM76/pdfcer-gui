//! `one_click_selects_a_paragraph_drawn_from_several_objects` — with the
//! Select tool, a click on a paragraph whose lines are separate text objects
//! selects all of them; a second click narrows to the line clicked. See
//! `docs/modules/ui-verify/checks/paragraph_pick.md`.

use super::dimdrive::{Fixture, click_on, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "paragraph-lines.pdf",
    method: "Regenerate it: python tools/gen-reflow-fixture.py.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};

const PICK: &str = "paragraph-pick"; // ui-text-exempt: a trace event name, never displayed
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// Inside line 3 of the six-line paragraph.
const AT: (f64, f64) = (120.0, 668.0);

/// See the module documentation.
pub struct OneClickSelectsAParagraphDrawnFromSeveralObjects;

impl Check for OneClickSelectsAParagraphDrawnFromSeveralObjects {
    fn name(&self) -> &'static str {
        "one_click_selects_a_paragraph_drawn_from_several_objects"
    }

    fn defect(&self) -> &'static str {
        "A click on a paragraph stored one text object per line selects one line, so the \
         paragraph cannot be moved or styled as one"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.edit", "paragraph-pick", drive)
    }
}

/// Click at [`AT`] and return the selection count the click left, or why not.
fn click(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<(usize, bool), String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, AT)?;
    session.settle(20);
    let trace = session.trace()?;
    let picked = trace.last_after(PICK, mark).is_some();
    Ok(trace
        .last_after(SELECTION, mark)
        .and_then(|l| l.get_usize("sel"))
        .map(|n| (n, picked))
        .ok_or_else(|| format!("no `{SELECTION}` line after the click")))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    let first = match click(ctx, session, pointer)? {
        Ok(v) => v,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!("first click: sel={} picked={}", first.0, first.1));
    if first != (6, true) {
        return Ok(Some(format!(
            "★ a click on the six-object paragraph selected {} object(s) (paragraph-pick {}), \
             not the six lines.",
            first.0,
            if first.1 { "traced" } else { "absent" }
        )));
    }
    let second = match click(ctx, session, pointer)? {
        Ok(v) => v,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!(
        "second click: sel={} picked={}",
        second.0, second.1
    ));
    Ok((second != (1, false)).then(|| {
        format!(
            "★★ a second click on the selected paragraph left {} object(s) selected, not the \
             one line clicked.",
            second.0
        )
    }))
}
