//! `the_hide_arrow_sits_clear_of_the_tabs` — each open dock's hide arrow is on
//! the dock's inner edge, no tab or "more" button is drawn under it, and
//! pressing it hides the dock. See `docs/modules/ui-verify/checks/hide_arrow.md`.

use super::dimdrive::{Fixture, press, run_on, ui_rect_event};
use super::driving::{declared, declared_since, list, live_names};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::Trace;

const FIXTURE: Fixture = Fixture {
    file: "a1-titleblock.pdf",
    method: "It is checked in under fixtures/.",
    page: PageGeometry {
        width_pt: 2383.94,
        height_pt: 1683.78,
    },
};

/// How far from the dock's inner edge the arrow may start, in points: the
/// splitter's width plus slack.
const EDGE_SLACK: f32 = 12.0;

/// See the module documentation.
pub struct TheHideArrowSitsClearOfTheTabs;

impl Check for TheHideArrowSitsClearOfTheTabs {
    fn name(&self) -> &'static str {
        "the_hide_arrow_sits_clear_of_the_tabs"
    }

    fn defect(&self) -> &'static str {
        "A dock's hide arrow is drawn under a tab or the tab bar's \"more\" button, or away \
         from the dock's inner edge, or pressing it does not hide the dock"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.edit,view.reset_layout",
            "hide-arrow",
            drive,
        )
    }
}

/// The failure for one side, or `None` when its arrow is clear and placed.
fn side_failure(trace: &Trace, ui_rect: &str, side: &str) -> Option<String> {
    let area = declared(trace, ui_rect, &format!("dock.{side}"))?;
    let Some(arrow) = declared(trace, ui_rect, &format!("dock.{side}.collapse")) else {
        return Some(format!("★★ the open {side} dock drew no hide arrow."));
    };
    let gap = if side == "right" {
        arrow.min.x - area.min.x
    } else {
        area.max.x - arrow.max.x
    };
    if !(0.0..=EDGE_SLACK).contains(&gap) {
        return Some(format!(
            "★★ the {side} dock's hide arrow {arrow:?} is {gap:.1} pt from its inner edge \
             (dock {area:?}); it belongs at that edge."
        ));
    }
    let under: Vec<String> = live_names(trace, ui_rect, "dock.")
        .into_iter()
        .filter(|n| n.starts_with("dock.tab.") || n.ends_with(".overflow"))
        .filter(|n| declared(trace, ui_rect, n).is_some_and(|r| r.overlaps(arrow)))
        .collect();
    (!under.is_empty()).then(|| {
        format!(
            "★★★ the {side} dock's hide arrow {arrow:?} is drawn over {}.",
            list(&under)
        )
    })
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    let trace = session.trace()?;
    let ui_rect = ui_rect_event(ctx)?;
    let overflows: Vec<String> = live_names(&trace, ui_rect, "dock.")
        .into_iter()
        .filter(|n| n.ends_with(".overflow"))
        .collect();
    report.note(format!("\"more\" buttons drawn: {}", list(&overflows)));
    for side in ["left", "right"] {
        if let Some(failure) = side_failure(&trace, ui_rect, side) {
            return Ok(Some(failure));
        }
    }
    let mark = trace.mark();
    press(ctx, session, pointer, "dock.right.collapse")?;
    if declared_since(&session.trace()?, ui_rect, "dock.right.rail", mark).is_none() {
        return Ok(Some(
            "★★ pressing the right dock's hide arrow did not leave the right dock's rail."
                .to_owned(),
        ));
    }
    Ok(None)
}
