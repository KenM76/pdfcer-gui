//! `a_snapshot_box_moves_resizes_and_clears` — a drag from inside a laid
//! snapshot box moves it whole, a drag from its bottom-right grip moves only
//! that corner, and putting the tool down clears the box. Driven through the
//! scripted pointer on a window placed off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_grips.md`.

use super::snapshot_box::{
    ARMED_EVENT, BOX_EVENT, BOX_REGION, FROM, GROUPS, ITEM, Rig, Step, TAB, laid_box, launch,
};
use crate::checks::driving::declared;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::report::CheckReport;

const CLEARED_EVENT: &str = "snapshot-cleared"; // ui-text-exempt: a trace event name, never displayed
/// The box's corners as fractions of the page box (y up): the base check's.
const TO: (f64, f64) = super::snapshot_box::TO;
/// How far the box is moved, and the corner pulled, as fractions of the page.
const SHIFT: (f64, f64) = (0.10, -0.08);
/// How far a corner may land from the expected one, in points: both ends of
/// the drag land on whole logical points.
const TOLERANCE_PT: f64 = 2.5;

/// Page-space corners, `(llx, lly, urx, ury)`.
type Corners = [f64; 4];

/// See the module documentation.
pub struct ASnapshotBoxMovesResizesAndClears;

impl Check for ASnapshotBoxMovesResizesAndClears {
    fn name(&self) -> &'static str {
        "a_snapshot_box_moves_resizes_and_clears"
    }

    fn defect(&self) -> &'static str {
        "a drag inside a snapshot box draws a new box instead of moving it, a drag on a \
         grip does not resize it from that corner, or the box outlives the tool"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (rig, page) = launch(ctx, report, "snapshot-grips")?;
    rig.click(TAB)?;
    rig.click_item(ITEM, GROUPS)?;
    if rig
        .session
        .trace()?
        .last(ARMED_EVENT)
        .and_then(|l| l.get("armed"))
        != Some("true")
    {
        return Ok(Some(
            "★ the Snapshot button did not arm the tool.".to_owned(),
        ));
    }
    let (w, h) = (page.width_pt, page.height_pt);
    let laid = [FROM.0 * w, FROM.1 * h, TO.0 * w, TO.1 * h];
    drag(&rig, ctx, page, (laid[0], laid[1]), (laid[2], laid[3]))?;
    if let Err(why) = laid_box(&rig, ((laid[0], laid[1]), (laid[2], laid[3])))? {
        return Ok(Some(why));
    }

    let (dx, dy) = (SHIFT.0 * w, SHIFT.1 * h);
    let centre = ((laid[0] + laid[2]) / 2.0, (laid[1] + laid[3]) / 2.0);
    drag(&rig, ctx, page, centre, (centre.0 + dx, centre.1 + dy))?;
    let moved = [laid[0] + dx, laid[1] + dy, laid[2] + dx, laid[3] + dy];
    if let Err(why) = landed(&rig, "move", moved, "★ a drag from inside the box")? {
        return Ok(Some(why));
    }
    report.note("★ a drag from inside the box moved it whole");

    // Bottom-right on screen is (urx, lly) on an upright page.
    let grip = (moved[2], moved[1]);
    drag(&rig, ctx, page, grip, (grip.0 + dx, grip.1 + dy))?;
    let resized = [moved[0], moved[1] + dy, moved[2] + dx, moved[3]];
    if let Err(why) = landed(
        &rig,
        "resize",
        resized,
        "★★ a drag from the bottom-right grip",
    )? {
        return Ok(Some(why));
    }
    report.note("★★ a drag from the bottom-right grip moved that corner and no other");

    rig.click_item(ITEM, GROUPS)?;
    rig.session.settle(20);
    let trace = rig.session.trace()?;
    if trace.last(CLEARED_EVENT).and_then(|l| l.get("reason")) != Some("tool") {
        return Ok(Some(format!(
            "★★★ putting the tool down traced no `{CLEARED_EVENT} reason=tool`."
        )));
    }
    if declared(&trace, rig.ui_rect, BOX_REGION).is_some() {
        return Ok(Some(format!(
            "★★★ the box was cleared and `{BOX_REGION}` is still drawn."
        )));
    }
    report.note("★★★ putting the tool down cleared the box from the page");
    Ok(None)
}

/// Drag between two page points through the current mapping.
fn drag(
    rig: &Rig,
    ctx: &CheckContext,
    page: PageGeometry,
    from: (f64, f64),
    to: (f64, f64),
) -> Result<()> {
    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, from.0, from.1))?;
    let b = mapping.doc_to_window(DocPoint::new(0, to.0, to.1))?;
    rig.pointer.drag(&rig.session, a, b, 12)?;
    rig.session.settle(20);
    Ok(())
}

/// The last `snapshot-box` line, required to carry `part` and `want`.
fn landed(rig: &Rig, part: &str, want: Corners, what: &str) -> Result<Step<()>> {
    let trace = rig.session.trace()?;
    let Some(line) = trace.last(BOX_EVENT) else {
        return Ok(Err(format!("{what} traced no `{BOX_EVENT}` line.")));
    };
    let got: Vec<Option<f64>> = ["llx", "lly", "urx", "ury"]
        .iter()
        .map(|k| line.get(k).and_then(|v| v.parse().ok()))
        .collect();
    let off = got
        .iter()
        .zip(want)
        .any(|(g, w)| g.is_none_or(|g| (g - w).abs() > TOLERANCE_PT));
    if line.get("part") != Some(part) || off {
        return Ok(Err(format!(
            "{what} should {part} the box to ({:.1}, {:.1})-({:.1}, {:.1}) and traced `{}`.",
            want[0], want[1], want[2], want[3], line.raw
        )));
    }
    Ok(Ok(()))
}
