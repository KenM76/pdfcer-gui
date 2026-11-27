//! `a_dimension_follows_a_circle_and_measures_to_its_far_edge` — a linear ce
//! dimension picked from a radius/diameter ce dimension's rim is anchored to
//! it; *Measure to* in Properties moves the anchor to the far edge, and
//! *Detach* ends it. See `docs/modules/ui-verify/checks/circle_anchor.md`.

use super::centre_line_anchor::pick;
use super::dimdrive::{click_on, press, press_on_tab, reveal, ui_rect_event};
use super::hole_dim::{HOLE, measure};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

// ui-text-exempt: trace region and event names, never displayed.
pub(super) const INVOKE: &str = "mode.review,measure.radius_diameter";
pub(super) const TAB: &str = "ribbon.tab.measure";
pub(super) const LINEAR: &str = "ribbon.item.measure.linear";
const PROPERTIES_TAB: &str = "dock.tab.file.properties";
const FAR: &str = "properties.dimension.anchor.arc.rim-max";
const DETACH: &str = "properties.dimension.anchor.detach";
const ADDED: &str = "dimension-added";
const SELECTED: &str = "annot-select";
const ARC: &str = "dimension-anchor-arc";
const DETACHED: &str = "dimension-anchor-detach";
const EDITED: &str = "set-dimension-anchor";
pub(super) const MARKER: &str = "measure-snap-marker";
const OUTLINE: &str = "canvas.selection-outline";
/// The least widening, window pixels, that moving to the far edge must give.
const GROWTH: f64 = 20.0;
/// Tab presses allowed to reach the rim snap among the drawing's own.
pub(super) const CYCLES: usize = 8;
/// On the hole's rim (r=30 at (306, 500)) at 45°, clear of its quadrant
/// nodes. The drawing's own outline outranks the circle's rim snap here, so
/// the pick Tabs to it.
const ON_RIM: (f64, f64) = (327.2, 521.2);
/// Level with [`ON_RIM`], inside the 120 pt the fixture keeps clear.
pub(super) const OTHER: (f64, f64) = (420.0, 521.2);
/// Below both picks; the label is drawn midway along the dimension line.
const PLACE: (f64, f64) = (373.6, 440.0);

pub struct ADimensionFollowsACircle;

impl Check for ADimensionFollowsACircle {
    fn name(&self) -> &'static str {
        "a_dimension_follows_a_circle_and_measures_to_its_far_edge"
    }

    fn defect(&self) -> &'static str {
        "a linear ce dimension picked from a circle's rim is not bound to it, Properties has \
         no Measure to choice for the point or choosing Far edge stores nothing, or Detach \
         does not end the anchor"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        super::dimdrive::run_on(self, ctx, &HOLE, INVOKE, "circle-anchor", drive)
    }
}

/// Measure the hole, then a linear ce dimension from its rim; the commit's
/// `anchors=`, or the failure.
fn place_linear(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<String, String>> {
    let path = session.trace_path().display().to_string();
    measure(ctx, session, pointer)?;
    pointer.key(session, None, "Escape", None)?;
    session.settle(15);
    press_on_tab(ctx, session, pointer, LINEAR, TAB)?;
    let mark = session.trace()?.mark();
    if !snap_pick(ctx, session, pointer, ON_RIM, "RimNearest")? {
        return Ok(Err(format!(
            "hovering the hole's rim and pressing Tab {CYCLES} times never offered the \
                 circle's rim snap (`{MARKER} kind=RimNearest`). Trace: {path}."
        )));
    }
    for at in [OTHER, PLACE] {
        pick(ctx, session, pointer, &HOLE, at)?;
    }
    let trace = session.trace()?;
    Ok(match trace.last_after(ADDED, mark) {
        Some(l) if !l.get("shape").is_some_and(|s| s.starts_with("circular")) => {
            Ok(l.get("anchors").unwrap_or("").to_owned())
        }
        _ => Err(format!(
            "three linear picks from the hole's rim committed no linear ce dimension \
             (`{ADDED}`). Trace: {path}."
        )),
    })
}

/// Hover `at` on the hole fixture, press Tab until the snap shown is `kind`,
/// and click there; whether `kind` was reached in [`CYCLES`] presses.
pub(super) fn snap_pick(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
    kind: &str,
) -> Result<bool> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, HOLE.page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?;
    pointer.hover(session, at)?;
    session.settle(10);
    for _ in 0..=CYCLES {
        let trace = session.trace()?;
        if trace.last(MARKER).and_then(|l| l.get("kind")) == Some(kind) {
            pointer.click(session, at)?;
            session.settle(20);
            return Ok(true);
        }
        pointer.key(session, None, "Tab", None)?;
        session.settle(10);
    }
    Ok(false)
}

/// Press `region`; whether it traced `event` and the engine committed.
fn press_commits(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    event: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, region)?;
    session.settle(20);
    let trace = session.trace()?;
    let pressed = trace.last_after(event, mark).map(|l| l.raw.clone());
    Ok(match (pressed, trace.last_after(EDITED, mark).is_some()) {
        (Some(raw), true) => Some(raw),
        _ => None,
    })
}

/// The selected ce dimension's outline width, window pixels.
fn outline_width(ctx: &CheckContext, session: &Session) -> Result<Option<f64>> {
    let trace = session.trace()?;
    Ok(
        declared_in(&trace, ui_rect_event(ctx)?, OUTLINE)
            .map(|(r, _)| f64::from(r.max.x - r.min.x)),
    )
}

/// Press *Far edge*: it must ask for `rim-max`, commit, and lengthen the ce
/// dimension, which is the stored anchor's witness rather than the request's.
fn far_edge(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let before = outline_width(ctx, session)?;
    let Some(far) = press_commits(ctx, session, pointer, FAR, ARC)? else {
        return Ok(Some(format!(
            "pressing Measure to: Far edge traced no `{ARC}` with a `{EDITED}` commit. \
                 Trace: {path}."
        )));
    };
    session.settle(20);
    let after = outline_width(ctx, session)?;
    report.note(format!(
        "far edge: `{far}`; outline {before:?} -> {after:?} px"
    ));
    if !far.contains("condition=rim-max") {
        return Ok(Some(format!(
            "Far edge asked for `{far}`; `condition=rim-max` was wanted. Trace: {path}."
        )));
    }
    Ok(match (before, after) {
        (Some(b), Some(a)) if a > b + GROWTH => None,
        _ => Some(format!(
            "Far edge committed but the ce dimension's outline went from {before:?} to \
                 {after:?} px; moving its end across the r=30 hole should widen it by more \
                 than {GROWTH} px. Trace: {path}."
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
    let anchors = match place_linear(ctx, session, pointer)? {
        Ok(a) => a,
        Err(f) => return Ok(Some(f)),
    };
    report.note(format!("anchors: `{anchors}`"));
    if !anchors.split(',').any(|t| t.starts_with("0:0:rim-min")) {
        return Ok(Some(format!(
            "the linear ce dimension picked from the rim committed `anchors={anchors}`: its \
             first point should follow the near edge of the circle (`0:0:rim-min`). \
             Trace: {path}."
        )));
    }
    pointer.key(session, None, "Escape", None)?;
    session.settle(15);
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &HOLE, 0, PLACE)?;
    let picked = session
        .trace()?
        .last_after(SELECTED, mark)
        .is_some_and(|l| l.get("kind") == Some("CeDimension"));
    if !picked {
        return Ok(Some(format!(
            "a click on the linear ce dimension's label selected nothing. Trace: {path}."
        )));
    }
    reveal(ctx, session, pointer, FAR, PROPERTIES_TAB)?;
    if let Some(f) = far_edge(ctx, report, session, pointer)? {
        return Ok(Some(f));
    }
    let detached = press_commits(ctx, session, pointer, DETACH, DETACHED)?;
    report.note(format!("detach: {detached:?}"));
    Ok(detached.is_none().then(|| {
        format!("pressing Detach traced no `{DETACHED}` with a `{EDITED}` commit. Trace: {path}.")
    }))
}
