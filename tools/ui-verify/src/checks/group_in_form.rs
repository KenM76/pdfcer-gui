//! `parts_of_a_placed_drawing_group_inside_it_and_ungroup`, on
//! `fixtures/form-parts.pdf`: two parts inside the placed drawing become a
//! group inside that drawing, and Ungroup on one of them inlines that group
//! back into the drawing, not onto the page.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/group_in_form.md`.

use super::form_node_move::{enter_leaf_at, run_body};
use super::multiselect::{after, step};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "../../fixtures/form-parts.pdf";
/// On the upper bar, leaf 0, page space.
const ON_THE_BAR: (f64, f64) = (100.0, 220.0);
/// On the polyline's first leg, leaf 1, page space.
const ON_THE_POLYLINE: (f64, f64) = (240.0, 120.0);
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const GROUPED: &str = "group-in-form-applied"; // ui-text-exempt: a trace event name, never displayed
const UNGROUPED: &str = "ungroup-in-form-applied"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct PartsOfAPlacedDrawingGroupInsideIt;

impl Check for PartsOfAPlacedDrawingGroupInsideIt {
    fn name(&self) -> &'static str {
        "parts_of_a_placed_drawing_group_inside_it_and_ungroup"
    }

    fn defect(&self) -> &'static str {
        "two parts selected inside a placed drawing cannot be grouped, or Ungroup on a part of \
         that group does not put its parts back inside the drawing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "group-in-form", drive) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    enter_leaf_at(ctx, session, pointer, page, ON_THE_BAR)?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let line = mapping.doc_to_window(DocPoint::new(0, ON_THE_POLYLINE.0, ON_THE_POLYLINE.1))?;
    pointer.send(
        session,
        &format!("click {:.1} {:.1} mods=shift", line.x(), line.y()),
    )?;
    session.settle(25);
    let selected = session
        .trace()?
        .last(SELECTION)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("after Shift+click on the polyline: `{selected}`"));
    if !(selected.contains(" sel=2 ") && selected.contains("first=leaf:")) {
        return Ok(Some(format!(
            "Shift+click on the polyline must add it to the bar, two parts selected; the \
             selection traced `{selected}`."
        )));
    }

    let mark = session.trace()?.mark();
    pointer.key(session, None, "G", Some("ctrl"))?;
    session.settle(40);
    let grouped = step!(after(session, GROUPED, mark, "Ctrl+G on two parts"));
    report.note(format!("Group: `{}`", grouped.raw));
    let leaves = grouped.get("leaves").unwrap_or_default();
    if grouped.get("members") != Some("2") || leaves.split(',').count() != 2 {
        return Ok(Some(format!(
            "Ctrl+G on two parts traced `{}`: two members, found again as two parts inside the \
             new group, was wanted.",
            grouped.raw
        )));
    }

    let bar = mapping.doc_to_window(DocPoint::new(0, ON_THE_BAR.0, ON_THE_BAR.1))?;
    pointer.click(session, bar)?;
    session.settle(25);
    let one = session
        .trace()?
        .last(SELECTION)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("after a click on the bar: `{one}`"));
    let mark = session.trace()?.mark();
    pointer.key(session, None, "G", Some("ctrl+shift"))?;
    session.settle(40);
    let peeled = step!(after(session, UNGROUPED, mark, "Ctrl+Shift+G on one part"));
    report.note(format!("Ungroup: `{}`", peeled.raw));
    let right = peeled.get("form") == grouped.get("group")
        && peeled.get("container") == grouped.get("container")
        && peeled.get("container") != Some("0");
    Ok((!right).then(|| {
        format!(
            "Ungroup on a part of the new group traced `{}` after Group traced `{}`: the \
             group taken apart into the drawing that held it was wanted.",
            peeled.raw, grouped.raw
        )
    }))
}
