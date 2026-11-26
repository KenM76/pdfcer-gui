//! Measure ▸ Centre line: with the tool armed, clicking one line and then a
//! parallel one commits the centre line between them, with no third click.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/centre_line.md`.

use super::dimdrive::{Fixture, click_on, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

// ui-text-exempt: command ids, trace event names and values, never displayed.
pub(super) const INVOKE: &str = "mode.review,measure.centre_line";
const CENTRE_EVENT: &str = "centre-line";
const REFUSED_EVENT: &str = "centre-line-refused";
pub(super) const ENGINE_EVENT: &str = "add-dimension";

pub(super) const TWO_LINES: Fixture = Fixture {
    file: "two-lines.pdf",
    method: "Rebuild it with `python fixtures/two-lines.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// On the lower line, (100,100)-(300,100), away from its ends.
pub(super) const LOWER: (f64, f64) = (160.0, 100.0);
/// On the upper line, (100,180)-(300,180).
pub(super) const UPPER: (f64, f64) = (240.0, 180.0);
/// The two lines overlap along their whole 200 pt.
const OVERLAP: &str = "200.00";

/// See the module documentation.
pub struct ACentreLineIsDrawnBetweenTwoLines;

impl Check for ACentreLineIsDrawnBetweenTwoLines {
    fn name(&self) -> &'static str {
        "a_centre_line_is_drawn_between_two_lines"
    }

    fn defect(&self) -> &'static str {
        "picking two parallel lines with Measure ▸ Centre line draws nothing, waits for a \
         third click, or draws the centre line of the wrong pair"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &TWO_LINES, INVOKE, "centre-line", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &TWO_LINES, 0, LOWER)?;
    let early = session
        .trace()?
        .events(ENGINE_EVENT)
        .any(|l| l.lineno > mark);
    if early {
        return Ok(Some(format!(
            "one click on one line already committed a ce dimension (`{ENGINE_EVENT}`): a \
             centre line needs two lines. Trace: {path}."
        )));
    }
    click_on(ctx, session, pointer, &TWO_LINES, 0, UPPER)?;
    let trace = session.trace()?;
    if let Some(refused) = trace.events(REFUSED_EVENT).find(|l| l.lineno > mark) {
        return Ok(Some(format!(
            "the second line was refused: `{}`. Trace: {path}.",
            refused.raw
        )));
    }
    let Some(centre) = trace.events(CENTRE_EVENT).find(|l| l.lineno > mark) else {
        return Ok(Some(format!(
            "two clicks on two lines traced no `{CENTRE_EVENT}`: the clicks did not pick both \
             lines, or the tool did not read them as a pair. Trace: {path}."
        )));
    };
    report.note(format!("centre: `{}`", centre.raw));
    if centre.get("relation") != Some("parallel") || centre.get("overlap") != Some(OVERLAP) {
        return Ok(Some(format!(
            "the centre line traced `{}`: `relation=parallel overlap={OVERLAP}` was wanted, so \
             it was not drawn between the fixture's two lines. Trace: {path}.",
            centre.raw
        )));
    }
    let committed = trace.events(ENGINE_EVENT).any(|l| l.lineno > centre.lineno);
    report.note(format!("engine committed: {committed}"));
    Ok((!committed).then(|| {
        format!(
            "the centre line was authored but the engine never traced `{ENGINE_EVENT}`: it did \
             not reach the document. Trace: {path}."
        )
    }))
}
