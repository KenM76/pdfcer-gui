//! `the_linear_tool_snaps_tangent_to_a_circle` — with a first point picked
//! off a radius/diameter ce dimension's circle, the linear tool offers the
//! point where a line from it touches the circle. See
//! `docs/modules/ui-verify/checks/tangent_snap.md`.

use super::centre_line_anchor::pick;
use super::circle_anchor::{CYCLES, INVOKE, LINEAR, OTHER, TAB, snap_pick};
use super::dimdrive::press_on_tab;
use super::hole_dim::{HOLE, measure};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The tangent point from [`OTHER`] on the r=30 hole at (306, 500), on the
/// lower side, 13 pt from the nearest quadrant node.
const TANGENT: (f64, f64) = (318.9, 472.9);

pub struct TheLinearToolSnapsTangentToACircle;

impl Check for TheLinearToolSnapsTangentToACircle {
    fn name(&self) -> &'static str {
        "the_linear_tool_snaps_tangent_to_a_circle"
    }

    fn defect(&self) -> &'static str {
        "the linear tool never offers a tangent snap on a ce dimension's circle: the snap \
         query is not told the first point, so no tangent can be computed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        super::dimdrive::run_on(self, ctx, &HOLE, INVOKE, "tangent-snap", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    _report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    measure(ctx, session, pointer)?;
    pointer.key(session, None, "Escape", None)?;
    session.settle(15);
    press_on_tab(ctx, session, pointer, LINEAR, TAB)?;
    pick(ctx, session, pointer, &HOLE, OTHER)?;
    let reached = snap_pick(ctx, session, pointer, TANGENT, "Tangent")?;
    Ok((!reached).then(|| {
        format!(
            "with the first point picked at {OTHER:?}, hovering the tangent point {TANGENT:?} \
             and pressing Tab {CYCLES} times never showed `kind=Tangent`. Trace: {}.",
            session.trace_path().display()
        )
    }))
}
