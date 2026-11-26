//! `a_placed_dimension_takes_and_drops_a_centre_mark` — a placed radius or
//! diameter ce dimension's *Centre mark* switch in the Properties panel adds a
//! mark, its *Extension lines* switch appears under it and is kept, and the
//! mark can be removed again; each press commits. See
//! `docs/modules/ui-verify/checks/centre_mark_after.md`.

use super::dimdrive::{click_on, press, reveal};
use super::hole_dim::{HOLE, PLACE, measure};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review,measure.radius_diameter";
const PROPERTIES_TAB: &str = "dock.tab.file.properties"; // ui-text-exempt: trace region name
const SWITCH: &str = "properties.dimension.centre-mark"; // ui-text-exempt: trace region name
const EXTENSION: &str = "properties.dimension.centre-mark.extension"; // ui-text-exempt: trace region name
const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name
const TOGGLED: &str = "dimension-centre-mark"; // ui-text-exempt: a trace event name
const EDITED: &str = "set-centre-mark"; // ui-text-exempt: a trace event name

pub struct APlacedDimensionTakesAndDropsACentreMark;

impl Check for APlacedDimensionTakesAndDropsACentreMark {
    fn name(&self) -> &'static str {
        "a_placed_dimension_takes_and_drops_a_centre_mark"
    }

    fn defect(&self) -> &'static str {
        "a placed radius or diameter ce dimension cannot gain or lose a centre mark: the \
             Properties panel has no Centre mark switch, its Extension lines switch never \
             appears, or a press does not change what is stored"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        super::dimdrive::run_on(self, ctx, &HOLE, INVOKE, "centre-mark-after", drive)
    }
}

/// Press `region`; the `on=` and `extension=` it traced, and whether the
/// engine committed. `None` fields mean the press traced nothing.
fn flip(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<(Option<String>, Option<String>, bool)> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, region)?;
    session.settle(20);
    let trace = session.trace()?;
    let toggled = trace.last_after(TOGGLED, mark);
    let get = |k: &str| toggled.and_then(|l| l.get(k).map(str::to_owned));
    let committed = trace.last_after(EDITED, mark).is_some();
    Ok((get("on"), get("extension"), committed))
}

fn want(got: &(Option<String>, Option<String>, bool), on: &str, ext: &str) -> bool {
    got.0.as_deref() == Some(on) && got.1.as_deref() == Some(ext) && got.2
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
    let mut added_from = session.trace()?.mark();
    let mut first = flip(ctx, session, pointer, SWITCH)?;
    report.note(format!("first press: {first:?}"));
    if want(&first, "0", "0") {
        // The tool placed it with a mark; that press removed it.
        added_from = session.trace()?.mark();
        first = flip(ctx, session, pointer, SWITCH)?;
        report.note(format!("re-add: {first:?}"));
    }
    if !want(&first, "1", "0") {
        return Ok(Some(format!(
            "pressing Centre mark gave {first:?}; `on=1 extension=0` and a `{EDITED}` \
             commit were wanted. Trace: {path}."
        )));
    }
    // The extension switch is drawn only while the stored kind carries a
    // mark, so its rect after the press is the stored state's witness.
    let since = added_from;
    let drawn = session
        .trace()?
        .events("ui-rect")
        .any(|l| l.lineno > since && l.get("name") == Some(EXTENSION));
    if !drawn {
        return Ok(Some(format!(
            "after the mark was added the Extension lines switch was not drawn, so the \
             mark was not stored. Trace: {path}."
        )));
    }
    let ext = flip(ctx, session, pointer, EXTENSION)?;
    report.note(format!("extension press: {ext:?}"));
    if !want(&ext, "1", "1") {
        return Ok(Some(format!(
            "pressing Extension lines gave {ext:?}; `on=1 extension=1` and a commit were \
             wanted. Trace: {path}."
        )));
    }
    let off = flip(ctx, session, pointer, SWITCH)?;
    report.note(format!("removal: {off:?}"));
    Ok((!want(&off, "0", "0")).then(|| {
        format!(
            "pressing Centre mark again gave {off:?}; `on=0 extension=0` and a commit were \
             wanted. Trace: {path}."
        )
    }))
}
