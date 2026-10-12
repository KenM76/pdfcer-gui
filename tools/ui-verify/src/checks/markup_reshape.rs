//! A highlight's end dragged onto the next line covers both lines, and a
//! line's Fill swatch fills its closed arrowheads.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/markup_reshape.md`.

use super::dimdrive::{Fixture, click_on, pick_colour, run_on, ui_rect_event};
use super::stroke_style::into_view;
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: Fixture = Fixture {
    file: "markup-reshape.pdf",
    method: "Rebuild it with `python fixtures/markup-reshape.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const SELECT: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
const RESPAN: &str = "markup-respan"; // ui-text-exempt: a trace event name, never displayed
const RESPANNED: &str = "respan-text-markup-applied"; // ui-text-exempt: a trace event name, never displayed
const STYLED: &str = "set-markup-style-applied"; // ui-text-exempt: a trace event name, never displayed
/// Mirror `canvas::respan::END_REGION` and `panels::properties::markup::rows`.
const END: &str = "canvas.markup-end.1";
const FILL: &str = "properties.markup.fill";
const FILL_PICKER: &str = "properties.markup.fill.picker";
/// Inside the highlight over "beta", page space.
const ON_THE_HIGHLIGHT: (f64, f64) = (97.0, 303.0);
/// On the line's stroke, between its arrowheads.
const ON_THE_LINE: (f64, f64) = (180.0, 120.0);
/// The right half of the "a" ending "zeta" on line 2: the caret after it is
/// `40 + 7.2 * 12 = 126.4`.
const AFTER_ZETA: (f64, f64) = (125.0, 283.0);

/// Dragging a highlight's end grip onto the next line re-spans it over both.
pub struct AHighlightsEndDragsOntoTheNextLine;
/// A line's Fill swatch fills its closed arrowheads.
pub struct ALinesFillColoursItsClosedArrowheads;

impl Check for AHighlightsEndDragsOntoTheNextLine {
    fn name(&self) -> &'static str {
        "a_highlights_end_drags_onto_the_next_line"
    }

    fn defect(&self) -> &'static str {
        "a selected highlight shows no end grips, or dragging one does not change the words it \
         covers"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "markup-respan", respan)
    }
}

impl Check for ALinesFillColoursItsClosedArrowheads {
    fn name(&self) -> &'static str {
        "a_lines_fill_colours_its_closed_arrowheads"
    }

    fn defect(&self) -> &'static str {
        "a line with closed arrowheads has no Fill row, or picking a fill does not write it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "markup-line-fill", line_fill)
    }
}

fn respan(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = select(ctx, session, pointer, ON_THE_HIGHLIGHT, "highlight")? {
        return Ok(Some(failure));
    }
    let Some((grip, _)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, END) else {
        return Ok(Some(format!(
            "the selected highlight shows no `{END}` grip. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let to = mapping.doc_to_window(DocPoint::new(0, AFTER_ZETA.0, AFTER_ZETA.1))?;
    let mark = session.trace()?.mark();
    pointer.drag(session, WindowPoint::centre_of(grip), to, 10)?;
    session.settle(40);
    let shot = ctx.out("markup-respan-after.png");
    pointer.screenshot(session, &shot)?;
    report.artifact(shot);

    let trace = session.trace()?;
    let Some(raised) = trace.last_after(RESPAN, mark) else {
        return Ok(Some(format!(
            "the end grip was dragged onto line 2 and no `{RESPAN}` was raised. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("raised: `{}`", raised.raw));
    let Some(applied) = trace.last_after(RESPANNED, mark) else {
        return Ok(Some(format!(
            "`{}` was raised and the engine applied nothing (no `{RESPANNED}`). Trace: {}.",
            raised.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("applied: `{}`", applied.raw));
    // One quad per covered line: "beta gamma delta" and "epsilon zeta".
    Ok((applied.get("quads") != Some("1->2")).then(|| {
        format!(
            "`{}` must re-span one quad to two (quads=1->2).",
            applied.raw
        )
    }))
}

fn line_fill(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = select(ctx, session, pointer, ON_THE_LINE, "line")? {
        return Ok(Some(failure));
    }
    into_view(ctx, session, pointer, FILL)?;
    if declared_in(&session.trace()?, ui_rect_event(ctx)?, FILL).is_none() {
        return Ok(Some(format!(
            "the selected line shows no `{FILL}` row. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mark = session.trace()?.mark();
    if let Some(failure) = pick_colour(ctx, session, pointer, FILL, FILL_PICKER)? {
        return Ok(Some(failure));
    }
    let trace = session.trace()?;
    let styled: Vec<_> = trace.events(STYLED).filter(|l| l.lineno > mark).collect();
    let Some(line) = styled.last() else {
        return Ok(Some(format!(
            "a fill was picked for the line and no `{STYLED}` followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("filled: `{}` ({} edit(s))", line.raw, styled.len()));
    if styled.len() != 1 {
        return Ok(Some(format!(
            "one pick of the Fill swatch made {} edits; it must make one.",
            styled.len()
        )));
    }
    let shot = ctx.out("markup-line-fill-after.png");
    pointer.screenshot(session, &shot)?;
    report.artifact(shot);
    let filled = line.get("interior").is_some_and(|v| v != "none");
    let closed = line.get("endings") == Some("ClosedArrow/ClosedArrow");
    Ok((!(filled && closed)).then(|| {
        format!(
            "`{}` must read back a fill (interior other than none) with both closed arrowheads \
             kept (endings=ClosedArrow/ClosedArrow).",
            line.raw
        )
    }))
}

/// Click `at` and require an `annot-select` after it.
fn select(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
    what: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
    session.settle(20);
    Ok(session
        .trace()?
        .last_after(SELECT, mark)
        .is_none()
        .then(|| {
            format!(
                "a click on the {what} traced no `{SELECT}`. Trace: {}.",
                session.trace_path().display()
            )
        }))
}
