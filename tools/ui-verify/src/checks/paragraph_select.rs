//! `a_fourth_click_selects_the_paragraph` — **triple-click a line of a
//! paragraph and the line is selected; click four times and the whole
//! paragraph is**, across every run its lines are drawn from, as Acrobat does.
//!
//! The oracle is two `canvas-text-selection` lines: `via=line` with L
//! characters after a triple click, then `via=paragraph` with more than three
//! times L after a four-click burst — the fixture's paragraph is six lines of
//! similar length. Two separate triple clicks are not the gesture: each burst
//! opens with a plain click, which clears the line it would extend.

use super::dimdrive::{Fixture, run_on};
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
const SELECTION: &str = "canvas-text-selection"; // ui-text-exempt: a trace event name, never displayed
/// Inside line 2 of the fixture's six-line block.
const ON_LINE: (f64, f64) = (120.0, 668.0);

pub struct AFourthClickSelectsTheParagraph;

impl Check for AFourthClickSelectsTheParagraph {
    fn name(&self) -> &'static str {
        "a_fourth_click_selects_the_paragraph"
    }

    fn defect(&self) -> &'static str {
        "a fourth click in a burst keeps the line instead of taking its paragraph"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.read", "paragraph-select", drive)
    }
}

/// `(via, chars)` of the last selection line after `mark`.
fn last(session: &Session, mark: usize) -> Result<Option<(String, usize)>> {
    let trace = session.trace()?;
    Ok(trace.last_after(SELECTION, mark).map(|l| {
        (
            l.get("via").unwrap_or_default().to_owned(),
            l.get_usize("chars").unwrap_or(0),
        )
    }))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(45);
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, ON_LINE.0, ON_LINE.1))?;
    let mark = session.trace()?.mark();
    pointer.triple_click(session, at)?;
    session.settle(40);
    let line = last(session, mark)?;
    let mark = session.trace()?.mark();
    pointer.quadruple_click(session, at)?;
    session.settle(40);
    pointer.screenshot(session, &ctx.out("paragraph-select.png"))?;
    let paragraph = last(session, mark)?;
    report.note(format!(
        "three clicks: {line:?}; four clicks: {paragraph:?}"
    ));
    let Some((via, line_chars)) = line.clone().filter(|(v, c)| v == "line" && *c > 0) else {
        return Ok(Some(format!(
            "a triple click inside the paragraph did not select a line: {line:?}"
        )));
    };
    match paragraph {
        Some((v, chars)) if v == "paragraph" && chars > 3 * line_chars => Ok(None),
        other => Ok(Some(format!(
            "three clicks gave `via={via} chars={line_chars}`; four on the same line gave \
             {other:?}, not the paragraph (`via=paragraph`, more than {} characters)",
            3 * line_chars
        ))),
    }
}
