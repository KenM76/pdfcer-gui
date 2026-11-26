//! `a_dimensions_circle_can_be_hidden` — a radius or diameter ce dimension's
//! *Draw the circle* switch in the Properties panel hides and restores the
//! circle, and the stored state follows each press. See
//! `docs/modules/ui-verify/checks/circle_shown.md`.

use super::dimdrive::{Fixture, click_on, press, reveal};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review,measure.radius_diameter";
const PROPERTIES_TAB: &str = "dock.tab.file.properties"; // ui-text-exempt: trace region name
const SWITCH: &str = "properties.dimension.circle"; // ui-text-exempt: trace region name
const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name
const TOGGLED: &str = "dimension-circle"; // ui-text-exempt: a trace event name
const EDITED: &str = "set-circle-shown"; // ui-text-exempt: a trace event name

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
/// Clear of the hole and of every segment, for the placing click; the label
/// is drawn here, so a click here selects the ce dimension.
const PLACE: (f64, f64) = (380.0, 560.0);

pub struct ADimensionsCircleCanBeHidden;

impl Check for ADimensionsCircleCanBeHidden {
    fn name(&self) -> &'static str {
        "a_dimensions_circle_can_be_hidden"
    }

    fn defect(&self) -> &'static str {
        "a radius or diameter ce dimension always draws its circle: the Properties panel has \
         no Draw the circle switch, or pressing it does not change what is stored"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        super::dimdrive::run_on(self, ctx, &HOLE, INVOKE, "circle-shown", drive)
    }
}

/// Measure the hole and place the label.
fn measure(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    for at in &RIM[..2] {
        click_on(ctx, session, pointer, &HOLE, 0, *at)?;
    }
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, HOLE.page, 0)?;
    let last = mapping.doc_to_window(DocPoint::new(0, RIM[2].0, RIM[2].1))?;
    pointer.hover(session, last)?;
    session.settle(10);
    pointer.double_click(session, last)?;
    session.settle(25);
    click_on(ctx, session, pointer, &HOLE, 0, PLACE)?;
    Ok(())
}

/// Press the switch; the `shown=` it traced and whether the engine took it.
fn flip(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<(Option<String>, bool)> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, SWITCH)?;
    session.settle(20);
    let trace = session.trace()?;
    let shown = trace
        .last_after(TOGGLED, mark)
        .and_then(|l| l.get("shown").map(str::to_owned));
    Ok((shown, trace.last_after(EDITED, mark).is_some()))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    measure(ctx, session, pointer)?;
    pointer.key(session, None, "Escape", None)?;
    session.settle(15);
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &HOLE, 0, PLACE)?;
    let picked = session
        .trace()?
        .last_after(SELECTED, mark)
        .filter(|l| l.get("kind") == Some("CeDimension"))
        .is_some();
    if !picked {
        return Ok(Some(format!(
            "a click on the placed label selected no ce dimension. Trace: {path}."
        )));
    }
    reveal(ctx, session, pointer, SWITCH, PROPERTIES_TAB)?;
    let off = flip(ctx, session, pointer)?;
    report.note(format!("first press: {off:?}"));
    if off != (Some("0".to_owned()), true) {
        return Ok(Some(format!(
            "the first press of Draw the circle gave {off:?}; `shown=0` and a `{EDITED}` \
             commit were wanted. Trace: {path}."
        )));
    }
    // A second press reads the stored state: had the first not been kept,
    // the switch would still show the circle and trace `shown=0` again.
    let on = flip(ctx, session, pointer)?;
    report.note(format!("second press: {on:?}"));
    Ok((on != (Some("1".to_owned()), true)).then(|| {
        format!(
            "the second press gave {on:?}; `shown=1` and a `{EDITED}` commit were wanted, so \
             the hidden state was not stored. Trace: {path}."
        )
    }))
}
