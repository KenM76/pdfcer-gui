//! `the_preflight_panel_runs_jumps_and_fixes` — the Preflight panel runs the
//! digital-printing profile on a worker while the window keeps drawing,
//! reports findings, picking a page-object finding selects that object on the
//! canvas, and Fix leaves a fix trace.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/preflight_panel.md`.

use super::dimdrive::{Fixture, launch_with, press, ui_rect_event};
use super::driving::declared_names;
use super::multiselect::{after, step};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: Fixture = Fixture {
    file: "preflight-low-ppi.pdf",
    method: "Rebuild it with `python fixtures/preflight-low-ppi.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,view.preflight";
const RUN: &str = "preflight.run"; // ui-text-exempt: a trace region name
const FIX: &str = "preflight.fix."; // ui-text-exempt: a trace region name prefix
const FIX_ALL: &str = "preflight.fix_all"; // ui-text-exempt: a trace region name
const STARTED: &str = "preflight-started"; // ui-text-exempt: a trace event name
const FINISHED: &str = "preflight-finished"; // ui-text-exempt: a trace event name
const ROWS: &str = "preflight-rows"; // ui-text-exempt: a trace event name
const PICKED: &str = "preflight-picked"; // ui-text-exempt: a trace event name
const FIXED: &str = "preflight-fixed"; // ui-text-exempt: a trace event name
const SELECTED: &str = "selection-set"; // ui-text-exempt: a trace event name
/// The fixture's page-2 image, drawn about 1 ppi, is page object 1 there.
const IMAGE_RULE: &str = "image-low-ppi"; // ui-text-exempt: an engine rule id
const IMAGE_PAGE: usize = 2;
const IMAGE_OBJECT: usize = 1;

/// `the_preflight_panel_runs_jumps_and_fixes`.
pub struct ThePreflightPanelRunsJumpsAndFixes;

impl Check for ThePreflightPanelRunsJumpsAndFixes {
    fn name(&self) -> &'static str {
        "the_preflight_panel_runs_jumps_and_fixes"
    }

    fn defect(&self) -> &'static str {
        "The Preflight panel finds nothing in a file with a known violation, holds the window \
         during the run, picking a page-object finding does not select that object, or Fix \
         does nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_with(ctx, &mut report, &FIXTURE, INVOKE, "preflight-panel", &[])
            .and_then(|(session, pointer)| {
                let outcome = body(ctx, &mut report, &session, &pointer);
                let parked = pointer.gone(&session);
                match outcome? {
                    Some(failure) => Ok(Some(failure)),
                    None => parked.map(|_| None),
                }
            });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let finished = step!(run(ctx, report, session, pointer));
    let findings = finished.get_usize("findings").unwrap_or(0);
    if findings == 0 || finished.get("cancelled") != Some("0") {
        return Ok(Some(format!(
            "`{}` on a file with a 1 ppi image; expected findings>0 and cancelled=0.",
            finished.raw
        )));
    }
    if finished.get_usize("frames").unwrap_or(0) == 0 {
        return Ok(Some(format!(
            "`{}`: no frame was drawn while the run was going, so the window waited for it.",
            finished.raw
        )));
    }
    if let Some(failure) = pick_the_image(ctx, report, session, pointer)? {
        return Ok(Some(failure));
    }
    fix(ctx, report, session, pointer)
}

/// Press Run and wait for the finished line.
fn run(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<TraceLine, String>> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, RUN)?;
    let started = after(session, STARTED, mark, &format!("`{RUN}` was pressed"))?;
    if let Ok(line) = &started {
        report.note(format!("`{}`", line.raw));
    }
    if let Err(why) = started {
        return Ok(Err(why));
    }
    for _ in 0..120 {
        if let Some(line) = session.trace()?.last_after(FINISHED, mark).cloned() {
            report.note(format!("`{}`", line.raw));
            return Ok(Ok(line));
        }
        session.settle(10);
    }
    after(session, FINISHED, mark, "the run started")
}

/// Find the image's row in the listing, press it, and read the selection.
fn pick_the_image(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let rows = step!(after(session, ROWS, 0, "the run finished"));
    let list = rows.get("list").unwrap_or_default();
    let want = format!("{IMAGE_RULE}@{IMAGE_PAGE}:object");
    let Some(k) = list
        .split(',')
        .find_map(|r| r.split_once(':').filter(|(_, rest)| *rest == want))
        .map(|(k, _)| k.to_owned())
    else {
        return Ok(Some(format!(
            "no `{want}` row among `{list}`: the 1 ppi image was not reported as a page object."
        )));
    };
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, &format!("preflight.row.{k}"))?;
    session.settle(20);
    let picked = step!(after(session, PICKED, mark, "the image's row was pressed"));
    report.note(format!("`{}`", picked.raw));
    let object = IMAGE_OBJECT.to_string();
    if picked.get("selected") != Some(object.as_str()) {
        return Ok(Some(format!(
            "`{}`; expected selected={object}.",
            picked.raw
        )));
    }
    let selected = step!(after(session, SELECTED, mark, "the image's row was picked"));
    report.note(format!("`{}`", selected.raw));
    let page = (IMAGE_PAGE - 1).to_string();
    if selected.get("object") != Some(object.as_str()) || selected.get("page") != Some(&page) {
        return Ok(Some(format!(
            "`{}`: the canvas selection is not page {page} object {object}.",
            selected.raw
        )));
    }
    Ok(None)
}

/// Press the first rule's Fix, or Fix all when none is offered alone.
fn fix(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let fixes = declared_names(&session.trace()?, ui_rect_event(ctx)?, FIX);
    report.note(format!("fix buttons {fixes:?}"));
    let region = fixes.first().cloned().unwrap_or_else(|| FIX_ALL.to_owned());
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, &region)?;
    session.settle(30);
    let fixed = step!(after(
        session,
        FIXED,
        mark,
        &format!("`{region}` was pressed")
    ));
    report.note(format!("`{}`", fixed.raw));
    if fixed.get_usize("steps").unwrap_or(0) == 0 {
        return Ok(Some(format!("`{}`: the fix ran no step.", fixed.raw)));
    }
    Ok(None)
}
