//! Edit ▸ Group and Ungroup on page content, driven on three boxes: two that
//! are not neighbours in paint order become one object at the front, a click
//! on either then selects that one object, and Ungroup gives both back.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/group_objects.md`.

use super::dimdrive::{Fixture, click_on, run_on};
use super::restack::{fail, press_ribbon};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const GROUPED: &str = "group-applied"; // ui-text-exempt: a trace event name, never displayed
const UNGROUPED: &str = "ungroup-applied"; // ui-text-exempt: a trace event name, never displayed
const UNGROUP: &str = "ribbon.item.edit.ungroup";
const BOXES: Fixture = Fixture {
    file: "overlapping-boxes.pdf",
    method: "Rebuild it with `python fixtures/overlapping-boxes.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// Inside the red box (object 0) only.
const RED_ONLY: (f64, f64) = (80.0, 80.0);
/// Inside the green box (object 2), clear of the others.
const GREEN: (f64, f64) = (50.0, 250.0);
/// Inside red and blue; blue (object 1) is on top until red is grouped.
const OVERLAP: (f64, f64) = (190.0, 140.0);
const PROPERTIES: &str = "properties-panel"; // ui-text-exempt: a trace event name, never displayed
const GEOMETRY: &str = "geometry-draft"; // ui-text-exempt: a trace event name, never displayed
const BLUE_ORIGIN: &str = "x=160.00 y=100.00";
const GREEN_ORIGIN: &str = "x=20.00 y=230.00";

/// See the module documentation.
pub struct PageObjectsGroupIntoOneAndUngroup;

impl Check for PageObjectsGroupIntoOneAndUngroup {
    fn name(&self) -> &'static str {
        "page_objects_group_into_one_and_ungroup"
    }

    fn defect(&self) -> &'static str {
        "two selected page objects cannot be grouped, or the group does not select as one \
         object, or Ungroup does not give the separate objects back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &BOXES, INVOKE, "group-objects", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let before = selected_at(ctx, session, pointer, OVERLAP)?;
    report.note(format!("before: the overlap selects `{before}`"));
    if !before.contains(BLUE_ORIGIN) {
        return Ok(Some(fail(
            session,
            "before grouping the overlap must select blue",
            &before,
        )));
    }
    click_on(ctx, session, pointer, &BOXES, 0, RED_ONLY)?;
    shift_click(ctx, session, pointer, GREEN)?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "G", Some("ctrl"))?;
    session.settle(40);
    if let Some(failure) = line_has(
        session,
        report,
        mark,
        GROUPED,
        &[("members", "2"), ("index", "1")],
    )? {
        return Ok(Some(failure));
    }
    // Red and green are one object now, in front of blue: a click on green
    // and a click on the overlap select the same object, and it is neither
    // green alone nor blue.
    for (at, name) in [(GREEN, "green"), (OVERLAP, "the overlap")] {
        let mark = session.trace()?.mark();
        let got = selected_at(ctx, session, pointer, at)?;
        let panel = session
            .trace()?
            .last_after(PROPERTIES, mark)
            .map(|l| l.raw.clone())
            .unwrap_or_default();
        report.note(format!("grouped: {name} selects `{got}` / `{panel}`"));
        if !panel.contains("object=1 ") || got.contains(BLUE_ORIGIN) || got.contains(GREEN_ORIGIN) {
            return Ok(Some(fail(
                session,
                &format!("after Group a click on {name} must select the group, object 1"),
                &format!("{got} / {panel}"),
            )));
        }
    }
    let mark = session.trace()?.mark();
    press_ribbon(ctx, session, pointer, UNGROUP)?;
    if let Some(failure) = line_has(
        session,
        report,
        mark,
        UNGROUPED,
        &[("first", "1"), ("count", "2")],
    )? {
        return Ok(Some(failure));
    }
    let after = selected_at(ctx, session, pointer, GREEN)?;
    report.note(format!("ungrouped: green selects `{after}`"));
    if !after.contains(GREEN_ORIGIN) {
        return Ok(Some(fail(
            session,
            "after Ungroup green must select alone again",
            &after,
        )));
    }
    Ok(None)
}

/// The last `event` line after `mark` must carry every `key=value`.
fn line_has(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    event: &str,
    want: &[(&str, &str)],
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(event, mark) else {
        return Ok(Some(format!(
            "the command reached no `{event}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("applied: `{}`", line.raw));
    let right = want.iter().all(|(k, v)| line.get(k) == Some(*v));
    Ok((!right).then(|| fail(session, &format!("`{event}` must read {want:?}"), &line.raw)))
}

/// Click `at` and read the geometry Properties shows for what it selected.
fn selected_at(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<String> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &BOXES, 0, at)?;
    session.settle(10);
    Ok(session
        .trace()?
        .last_after(GEOMETRY, mark)
        .map_or_else(|| format!("<no {GEOMETRY} line>"), |l| l.raw.clone()))
}

fn shift_click(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<()> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, BOXES.page, 0)?;
    let point = mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?;
    pointer.hover(session, point)?;
    session.settle(10);
    pointer.click_with(session, point, "shift")?;
    session.settle(20);
    Ok(())
}
