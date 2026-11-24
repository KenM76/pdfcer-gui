//! `a_scaled_width_sets_the_drawn_size` — a 40 pt box on a page drawn at
//! 5 mm/pt is selected, `500 mm` is typed into the scaled Width row, and Apply
//! must widen the box to 100 pt: a factor of exactly 2.5.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scaled_size.md`.

use super::dimdrive::{Fixture, click_on, press, run_on, ui_rect_event};
use super::print_shop::await_line;
use super::stroke_style::into_view;
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "scaled-box.pdf",
    method: "Rebuild it with `python fixtures/scaled-box.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
const INVOKE: &str = "mode.edit,file.properties";
/// Inside the fixture's 40 x 20 pt box at (100, 200).
const ON_BOX: (f64, f64) = (120.0, 210.0);
/// Mirrors `panels::properties::geometry::scaled::WIDTH_REGION`.
const SCALED_WIDTH: &str = "properties.geometry.scaledwidth"; // ui-text-exempt: a trace region name
/// Mirrors `panels::properties::geometry::APPLY_REGION`.
const APPLY: &str = "properties.geometry.apply"; // ui-text-exempt: a trace region name
const SHOWN: &str = "geometry-scaled"; // ui-text-exempt: a trace event name
const TYPED_EVENT: &str = "geometry-scaled-typed"; // ui-text-exempt: a trace event name
const REFUSED: &str = "geometry-scaled-refused"; // ui-text-exempt: a trace event name
const COMMIT: &str = "resize-scale"; // ui-text-exempt: a trace event name
const APPLIED: &str = "transform-objects-applied"; // ui-text-exempt: a trace event name
/// 100 pt at 5 mm/pt: two and a half times the box's 40 pt.
const TYPED: &str = "500 mm";
const WANT_PTS: f64 = 100.0;
const WANT_SX: f64 = 2.5;

/// See the module documentation.
pub struct AScaledWidthSetsTheDrawnSize;

impl Check for AScaledWidthSetsTheDrawnSize {
    fn name(&self) -> &'static str {
        "a_scaled_width_sets_the_drawn_size"
    }

    fn defect(&self) -> &'static str {
        "a length typed at the drawing's scale is read as points, or as the wrong scale, so the \
         object is drawn at a size other than the one typed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "scaled_size", steps)
    }
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_BOX)?;
    let Some(shown) = await_line(session, SHOWN, mark, |_| true)? else {
        return Ok(Some(format!(
            "the box was clicked and no `{SHOWN}` line followed: the scaled rows did not draw \
             on a page whose default group has a scale. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("scaled rows drew: `{}`", shown.raw));

    into_view(ctx, session, pointer, SCALED_WIDTH)?;
    let vp =
        declared_in(&session.trace()?, ui_rect_event(ctx)?, SCALED_WIDTH).and_then(|(_, vp)| vp);
    press(ctx, session, pointer, SCALED_WIDTH)?;
    pointer.key(session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(session, vp.as_deref(), TYPED)?;
    let mark = session.trace()?.mark();
    pointer.key(session, vp.as_deref(), "Enter", None)?;
    session.settle(20);
    let Some(typed) = await_line(session, TYPED_EVENT, mark, |_| true)? else {
        let refused = session.trace()?.last(REFUSED).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "typed `{TYPED}` into the scaled Width and pressed Enter; no `{TYPED_EVENT}` \
             followed. Refusal: {}. Trace: {}.",
            refused.unwrap_or_else(|| "none".to_owned()),
            session.trace_path().display()
        )));
    };
    let pts: f64 = typed
        .get("pts")
        .and_then(|v| v.parse().ok())
        .unwrap_or(f64::NAN);
    if (pts - WANT_PTS).abs() > 0.01 {
        return Ok(Some(format!(
            "`{TYPED}` at 5 mm/pt read as {pts} pt, not {WANT_PTS}: `{}`.",
            typed.raw
        )));
    }
    report.note(format!("read: `{}`", typed.raw));

    into_view(ctx, session, pointer, APPLY)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, APPLY)?;
    let Some(commit) = await_line(session, COMMIT, mark, |_| true)? else {
        return Ok(Some(format!(
            "Apply was pressed after the scaled width was read and no `{COMMIT}` followed. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let sx: f64 = commit
        .get("sx")
        .and_then(|v| v.parse().ok())
        .unwrap_or(f64::NAN);
    if (sx - WANT_SX).abs() > 0.005 {
        return Ok(Some(format!(
            "Apply committed sx={sx}, not {WANT_SX}: `{}`.",
            commit.raw
        )));
    }
    report.note(format!("committed: `{}`", commit.raw));
    if await_line(session, APPLIED, mark, |_| true)?.is_none() {
        return Ok(Some(format!(
            "`{}` was raised and no `{APPLIED}` followed: nothing reached the document.",
            commit.raw
        )));
    }
    Ok(None)
}
