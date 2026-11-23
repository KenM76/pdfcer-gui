//! A measure tool snaps to a markup: with Linear armed, hovering beside a
//! square markup's corner resolves a snap that names the markup, and hovering
//! over blank paper names none.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snap_markup.md`.

use super::annot_multiselect::FIXTURE;
use super::dimdrive::{Fixture, run_on};
use super::multiselect::{after, step};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

pub(super) const HOVER: &str = "measure-hover"; // ui-text-exempt: a trace event name
/// Just inside the square markup's top-left corner, `/Rect [40 200 100 240]`.
const BESIDE_CORNER: (f64, f64) = (41.0, 239.0);
/// Paper with no markup and no page content near it.
const BLANK: (f64, f64) = (350.0, 120.0);

/// See the module documentation.
pub struct MeasureToolSnapsToAMarkup;

impl Check for MeasureToolSnapsToAMarkup {
    fn name(&self) -> &'static str {
        "measure_tool_snaps_to_a_markup"
    }

    fn defect(&self) -> &'static str {
        "a measure tool hovering at a markup's corner finds nothing to snap to, so a markup \
         shape or text box cannot be dimensioned"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.review,measure.linear",
            "snap-markup",
            drive,
        )
    }
}

/// Hover `at` (page points) on `fixture`'s page and answer the
/// `measure-hover` line it caused.
pub(super) fn hover(
    ctx: &CheckContext,
    fixture: &Fixture,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
    what: &str,
) -> Result<std::result::Result<crate::trace::TraceLine, String>> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, fixture.page, 0)?;
    let point = mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?;
    let mark = session.trace()?.mark();
    pointer.hover(session, point)?;
    session.settle(12);
    after(session, HOVER, mark, what)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let blank = step!(hover(
        ctx,
        &FIXTURE,
        session,
        pointer,
        BLANK,
        "the pointer rested on blank paper with Linear armed"
    ));
    report.note(format!("blank: `{}`", blank.raw));
    if blank.get("markup") != Some("0") {
        return Ok(Some(format!(
            "over blank paper the hover traced `{}`: `markup=0` was wanted, so a snap that \
             names a markup means something.",
            blank.raw
        )));
    }
    let corner = step!(hover(
        ctx,
        &FIXTURE,
        session,
        pointer,
        BESIDE_CORNER,
        "the pointer rested beside the square markup's corner"
    ));
    report.note(format!("corner: `{}`", corner.raw));
    let named = corner.get("markup").is_some_and(|id| id != "0");
    if corner.get("snap") != Some("1") || !named {
        return Ok(Some(format!(
            "beside the square markup's corner the hover traced `{}`: a snap naming the \
             markup was wanted.",
            corner.raw
        )));
    }
    Ok(None)
}
