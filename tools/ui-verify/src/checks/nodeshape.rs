//! Format ▸ Nodes driven on one open path, a line then a curve: a node
//! selected at the Node rung has the line after it made a curve, then a node
//! inserted after it, and the new node is what is selected.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/nodeshape.md`.

use super::dimdrive::{Fixture, click_on, press_on_tab, run_on, ui_rect_event};
use crate::checks::driving::declared;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit";
const APPLIED: &str = "node-shape-applied"; // ui-text-exempt: a trace event name, never displayed
const ANCHORS: &str = "canvas-anchors"; // ui-text-exempt: a trace event name, never displayed
const HANDLES: &str = "canvas-handles"; // ui-text-exempt: a trace event name, never displayed
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const SELECTION_SET: &str = "selection-set"; // ui-text-exempt: a trace event name, never displayed
/// The first drawn anchor mark. Mirrors `anchormarks::anchor_region`.
const FIRST_ANCHOR: &str = "canvas.anchor.0";
const FORMAT_TAB: &str = "ribbon.tab.format";
const TO_CURVE: &str = "ribbon.item.format.segment_curve";
const INSERT: &str = "ribbon.item.format.node_insert";
const PATH: Fixture = Fixture {
    file: "line-and-curve.pdf",
    method: "Rebuild it with `python fixtures/line-and-curve.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// The path's first node, which `canvas.anchor.0` marks at the Part rung
/// (anchors are drawn in node order); the line leaves it.
const NODE: &str = "0";
/// What the insert's selection names: the node after it.
const INSERTED: &str = "node=1 ";
/// On the straight segment, page space.
const ON_THE_LINE: (f64, f64) = (175.0, 100.0);

/// See the module documentation.
pub struct ANodesSegmentCanBeCurvedAndANodeInserted;

impl Check for ANodesSegmentCanBeCurvedAndANodeInserted {
    fn name(&self) -> &'static str {
        "a_nodes_segment_can_be_curved_and_a_node_inserted"
    }

    fn defect(&self) -> &'static str {
        "with a node of a page path selected, Format ▸ Nodes does nothing, changes the wrong \
         node, or leaves the selection on a node the insert renumbered"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &PATH, INVOKE, "nodeshape", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    enter_node(ctx, report, session, pointer)?;
    let handles_before = handles_after(session, 0)?;
    report.note(format!(
        "node {NODE} selected with {handles_before} handle(s)"
    ));

    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, TO_CURVE, FORMAT_TAB)?;
    if let Some(failure) = applied(session, report, mark, "curve")? {
        return Ok(Some(failure));
    }
    let handles = handles_after(session, mark)?;
    if handles <= handles_before {
        return Ok(Some(format!(
            "Segment to curve was applied and node {NODE} shows {handles} handle(s), as before; \
             a curve leaving it has one. Trace: {}.",
            session.trace_path().display()
        )));
    }

    let total_before = last_usize(session, ANCHORS, "total", 0)?;
    let mark = session.trace()?.mark();
    press_on_tab(ctx, session, pointer, INSERT, FORMAT_TAB)?;
    if let Some(failure) = applied(session, report, mark, "insert")? {
        return Ok(Some(failure));
    }
    let total = last_usize(session, ANCHORS, "total", mark)?;
    if total != total_before + 1 {
        return Ok(Some(format!(
            "Insert node was applied and the path shows {total} anchors; it had {total_before}. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    let selected = session
        .trace()?
        .last_after(SELECTION_SET, mark)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if !(selected.contains(INSERTED) && selected.contains("via=node-inserted")) {
        return Ok(Some(format!(
            "after the insert the new node must be selected (`{INSERTED}`, via=node-inserted); \
             the selection read `{selected}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("the new node is selected: `{selected}`"));
    Ok(None)
}

/// Click the line, double-click into its Part rung, double-click its first
/// anchor into the Node rung with one anchor selected.
fn enter_node(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<()> {
    let ui_rect = ui_rect_event(ctx)?;
    click_on(ctx, session, pointer, &PATH, 0, ON_THE_LINE)?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PATH.page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, ON_THE_LINE.0, ON_THE_LINE.1))?;
    pointer.double_click(session, at)?;
    session.settle(30);
    let trace = session.trace()?;
    let anchor = declared(&trace, ui_rect, FIRST_ANCHOR).ok_or_else(|| {
        Error::new(format!(
            "no `{FIRST_ANCHOR}` after descending into the path (last `{ANCHORS}`: {}). Trace: \
             {}.",
            trace.last(ANCHORS).map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        ))
    })?;
    report.note(format!(
        "Part rung: `{}`",
        trace.last(ANCHORS).map_or("none", |l| l.raw.as_str())
    ));
    let mark = trace.mark();
    pointer.double_click(session, WindowPoint::centre_of(anchor))?;
    session.settle(30);
    let trace = session.trace()?;
    let selection = trace.last_after(SELECTION, mark);
    let at_node = selection.is_some_and(|l| l.get("level") == Some("Node"));
    let level = selection.map(|l| l.raw.clone());
    let picked = trace
        .last_after(ANCHORS, mark)
        .and_then(|l| l.get_usize("selected"));
    if !at_node || picked != Some(1) {
        return Err(Error::new(format!(
            "the double-click on the first anchor did not stand at the Node rung with one anchor \
             selected (`{}`, selected={picked:?}). Trace: {}.",
            level.unwrap_or_default(),
            session.trace_path().display()
        )));
    }
    Ok(())
}

/// The `node-shape-applied` line after `mark` must name `shape`, have been
/// asked for [`NODE`], and have done it.
fn applied(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    shape: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "the {shape} command reached no node verb (no `{APPLIED}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("applied: `{}`", line.raw));
    let right = line.get("shape") == Some(shape)
        && line.get("asked") == Some(NODE)
        && line.get("done") == Some(NODE);
    Ok((!right).then(|| {
        format!(
            "the press must read `shape={shape} asked={NODE} done={NODE}`; it read `{}`. \
             Trace: {}.",
            line.raw,
            session.trace_path().display()
        )
    }))
}

/// Handles drawn by the last frame after `mark`. `canvas-handles` is written
/// only when a handle is drawn, so no line means none: a line leaving the
/// node, before the press or after a press that failed to curve it.
fn handles_after(session: &Session, mark: usize) -> Result<usize> {
    Ok(session
        .trace()?
        .last_after(HANDLES, mark)
        .and_then(|l| l.get_usize("n"))
        .unwrap_or(0))
}

/// The `key` of the last `event` line after `mark`.
fn last_usize(session: &Session, event: &str, key: &str, mark: usize) -> Result<usize> {
    session
        .trace()?
        .last_after(event, mark)
        .and_then(|l| l.get_usize(key))
        .ok_or_else(|| {
            Error::new(format!(
                "no `{event} {key}=` line to read. Trace: {}.",
                session.trace_path().display()
            ))
        })
}
