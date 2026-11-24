//! `a_circle_gets_a_centre_mark_with_or_without_its_measurement` — the
//! circular tool's two switches (`OPERATOR_REQUESTS.md` O454): the mark alone
//! commits on the finishing double-click with no placing click; both switches
//! commit one circular ce dimension carrying the mark.

use super::dimdrive::{Fixture, click_on, press, reveal};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review,measure.radius_diameter";
const PROPERTIES_TAB: &str = "dock.tab.file.properties";
const MEASUREMENT: &str = "properties.tool.circular_measurement";
const MARK: &str = "properties.tool.circular_centre_mark";
const OUTPUTS_EVENT: &str = "measure-circular-outputs";
const ADDED_EVENT: &str = "dimension-added";

const HOLE: Fixture = Fixture {
    file: "hole-in-a-big-object.pdf",
    method: "Rebuild it with `python tools/gen-hole-in-a-big-object-fixture.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
/// North, east and west of the fixture's r=30 hole at (306, 500).
const RIM: [(f64, f64); 3] = [(306.0, 530.0), (336.0, 500.0), (276.0, 500.0)];
/// Clear of the hole and of every segment, for the placing click.
const PLACE: (f64, f64) = (380.0, 560.0);

pub struct ACircleGetsACentreMarkWithOrWithoutItsMeasurement;

impl Check for ACircleGetsACentreMarkWithOrWithoutItsMeasurement {
    fn name(&self) -> &'static str {
        "a_circle_gets_a_centre_mark_with_or_without_its_measurement"
    }

    fn defect(&self) -> &'static str {
        "the radius/diameter tool cannot draw a centre mark: with only Add centre mark ticked, \
         finishing a circle draws nothing or waits for a placing click, and with both ticked \
         the measurement is drawn without its mark"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        super::dimdrive::run_on(self, ctx, &HOLE, INVOKE, "centre-mark", drive)
    }
}

/// Pick the three rim points, the last by double-click, which ends the set.
fn pick_hole(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    for at in &RIM[..2] {
        click_on(ctx, session, pointer, &HOLE, 0, *at)?;
    }
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, HOLE.page, 0)?;
    let last = mapping.doc_to_window(DocPoint::new(0, RIM[2].0, RIM[2].1))?;
    pointer.hover(session, last)?;
    session.settle(10);
    pointer.double_click(session, last)?;
    session.settle(25);
    Ok(())
}

/// The `shape=` of the first `dimension-added` after `mark`, if any.
fn added_after(session: &Session, mark: usize) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(ADDED_EVENT)
        .find(|l| l.lineno > mark)
        .map(|l| l.get("shape").unwrap_or("?").to_owned()))
}

/// Flip a switch and confirm the tool traced the outputs wanted.
fn set(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    want: &str,
) -> Result<Option<String>> {
    press(ctx, session, pointer, region)?;
    let trace = session.trace()?;
    let last = trace.events(OUTPUTS_EVENT).last().map(|l| l.raw.clone());
    Ok(match last {
        Some(raw) if raw.contains(want) => None,
        other => Some(format!(
            "pressing `{region}` left the switches at {other:?}; `{want}` was wanted. Trace: {}.",
            session.trace_path().display()
        )),
    })
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    reveal(ctx, session, pointer, MARK, PROPERTIES_TAB)?;
    if let Some(f) = set(ctx, session, pointer, MARK, "measurement=1 centre_mark=1")? {
        return Ok(Some(f));
    }
    if let Some(f) = set(
        ctx,
        session,
        pointer,
        MEASUREMENT,
        "measurement=0 centre_mark=1",
    )? {
        return Ok(Some(f));
    }
    let mark = session.trace()?.mark();
    pick_hole(ctx, session, pointer)?;
    let alone = added_after(session, mark)?;
    report.note(format!("mark alone committed: {alone:?}"));
    if alone.as_deref() != Some("centre-mark") {
        return Ok(Some(format!(
            "with only Add centre mark ticked, finishing the circle committed {alone:?} without \
             a placing click; `shape=centre-mark` was wanted. Trace: {path}."
        )));
    }

    if let Some(f) = set(
        ctx,
        session,
        pointer,
        MEASUREMENT,
        "measurement=1 centre_mark=1",
    )? {
        return Ok(Some(f));
    }
    let mark = session.trace()?.mark();
    pick_hole(ctx, session, pointer)?;
    if let Some(early) = added_after(session, mark)? {
        return Ok(Some(format!(
            "with the measurement ticked, finishing committed `{early}` before the placing \
             click: the label would land wherever the double-click was. Trace: {path}."
        )));
    }
    click_on(ctx, session, pointer, &HOLE, 0, PLACE)?;
    let both = added_after(session, mark)?;
    report.note(format!("both committed: {both:?}"));
    Ok((both.as_deref() != Some("circular+mark")).then(|| {
        format!(
            "with both switches ticked, the placed ce dimension committed {both:?}; \
             `shape=circular+mark` was wanted. Trace: {path}."
        )
    }))
}
