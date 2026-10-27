//! `ctrl_alt_click_deletes_a_markup_point` and
//! `ctrl_alt_click_deletes_a_page_point` — with the Points tool armed, one
//! Ctrl+Alt+click on a shown point deletes it (O332). Scripted pointer, window
//! off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/point_click_delete.md`.

use super::dimdrive::{Fixture, press_on_tab, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const LETTER: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
const MARKUP_FIXTURE: Fixture = Fixture {
    file: "square-no-appearance.pdf",
    method: "Committed with the repository; restore it from git.",
    page: LETTER,
};
const PATH_FIXTURE: Fixture = Fixture {
    file: "polyline-nodes.pdf",
    method: "Committed with the repository; restore it from git.",
    page: LETTER,
};
const MARKUP_TAB: &str = "ribbon.tab.markup";
const POLYGON: &str = "ribbon.item.markup.polygon";
/// `canvas.markup-node.N` — a selected markup shape's anchor N.
const NODE_REGION: &str = "canvas.markup-node.1";
/// Mirrors `canvas::pointdelete::TRACE`.
const POINT_DELETE: &str = "point-delete"; // ui-text-exempt: a trace event name, never displayed
const MARKUP_APPLIED: &str = "remove-annotation-vertex"; // ui-text-exempt: a trace event name
const PATH_APPLIED: &str = "delete-node-applied"; // ui-text-exempt: a trace event name
const COMMIT: &str = "add-markup"; // ui-text-exempt: a trace event name, never displayed
/// The polygon's corners, page points.
/// The middle of the polygon's bottom edge.
const EDGE: (f64, f64) = (250.0, 500.0);
const CORNERS: [(f64, f64); 4] = [
    (150.0, 500.0),
    (350.0, 500.0),
    (350.0, 650.0),
    (150.0, 650.0),
];
/// A point on the fixture polyline's first segment (enters the object), and
/// its second anchor (the one deleted).
const ON_PATH: (f64, f64) = (150.0, 260.0);
const ANCHOR: (f64, f64) = (200.0, 320.0);

/// See the module documentation.
pub struct CtrlAltClickDeletesAMarkupPoint;

impl Check for CtrlAltClickDeletesAMarkupPoint {
    fn name(&self) -> &'static str {
        "ctrl_alt_click_deletes_a_markup_point"
    }

    fn defect(&self) -> &'static str {
        "with the Points tool armed, a Ctrl+Alt+click on a markup shape's point does not delete \
         it, so deleting a point still takes a right-click menu or a modified drag"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &MARKUP_FIXTURE,
            "mode.review",
            "point-click-delete.markup",
            markup_steps,
        )
    }
}

/// See the module documentation.
pub struct CtrlAltClickDeletesAPagePoint;

impl Check for CtrlAltClickDeletesAPagePoint {
    fn name(&self) -> &'static str {
        "ctrl_alt_click_deletes_a_page_point"
    }

    fn defect(&self) -> &'static str {
        "with the Points tool armed, a Ctrl+Alt+click on a path's shown point does not delete \
         it, so deleting a point still takes a click and then the Delete key"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &PATH_FIXTURE,
            "mode.edit",
            "point-click-delete.path",
            path_steps,
        )
    }
}

fn at(
    ctx: &CheckContext,
    session: &Session,
    fixture: &Fixture,
    p: (f64, f64),
) -> Result<WindowPoint> {
    CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, fixture.page, 0)?
        .doc_to_window(DocPoint::new(0, p.0, p.1))
}

fn markup_steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    press_on_tab(ctx, session, pointer, POLYGON, MARKUP_TAB)?;
    for (i, corner) in CORNERS.iter().enumerate() {
        let p = at(ctx, session, &MARKUP_FIXTURE, *corner)?;
        if i + 1 == CORNERS.len() {
            pointer.double_click(session, p)?;
        } else {
            pointer.click(session, p)?;
        }
        session.settle(12);
    }
    session.settle(30);
    if session.trace()?.last(COMMIT).is_none() {
        return Ok(Some(format!(
            "the polygon never committed (no `{COMMIT}`), so there was no point to delete. \
             Trace: {path}."
        )));
    }
    pointer.key(session, None, "A", None)?;
    session.settle(20);
    // Click the shape to show its points.
    pointer.click(session, at(ctx, session, &MARKUP_FIXTURE, EDGE)?)?;
    session.settle(30);
    let Some((rect, _)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, NODE_REGION) else {
        return Ok(Some(format!(
            "with the Points tool armed the drawn polygon showed no `{NODE_REGION}`. Trace: {path}."
        )));
    };
    let mark = session.trace()?.mark();
    pointer.click_with(session, WindowPoint::centre_of(rect), "ctrl+alt")?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(POINT_DELETE, mark) else {
        return Ok(Some(format!(
            "★★★ Ctrl+Alt+click on the polygon's point 1 traced no `{POINT_DELETE}`: the click \
             did not reach `canvas::pointdelete::markup`. Trace: {path}."
        )));
    };
    report.note(format!("`{}`", line.raw));
    if line.get("via") != Some("markup") || line.get("outcome") != Some("raised") {
        return Ok(Some(format!(
            "★★★ the click reached the point delete and it did not raise a markup removal: `{}`. \
             Trace: {path}.",
            line.raw
        )));
    }
    Ok(trace.last_after(MARKUP_APPLIED, mark).is_none().then(|| {
        format!(
            "★★★ the removal was raised and no `{MARKUP_APPLIED}` followed: the document did not \
             change. Trace: {path}."
        )
    }))
}

fn path_steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    pointer.key(session, None, "A", None)?;
    session.settle(20);
    pointer.click(session, at(ctx, session, &PATH_FIXTURE, ON_PATH)?)?;
    session.settle(30);
    let mark = session.trace()?.mark();
    pointer.click_with(
        session,
        at(ctx, session, &PATH_FIXTURE, ANCHOR)?,
        "ctrl+alt",
    )?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(POINT_DELETE, mark) else {
        return Ok(Some(format!(
            "★★★ Ctrl+Alt+click on the polyline's second point traced no `{POINT_DELETE}`: \
             either the first click did not show the points or the click did not reach \
             `canvas::pointdelete::anchor`. Trace: {path}."
        )));
    };
    report.note(format!("`{}`", line.raw));
    if line.get("via") != Some("anchor") || line.get("outcome") != Some("raised") {
        return Ok(Some(format!(
            "★★★ the point delete did not raise a node delete: `{}`. Trace: {path}.",
            line.raw
        )));
    }
    Ok(trace.last_after(PATH_APPLIED, mark).is_none().then(|| {
        format!(
            "★★★ the delete was raised and no `{PATH_APPLIED}` followed: the document did not \
             change. Trace: {path}."
        )
    }))
}
