//! Edit ▸ Arrange and the Ctrl+bracket chords on page content, driven on two
//! overlapping boxes: the selected box changes place in paint order, stays
//! selected, and a click where the boxes overlap then lands on whichever is on
//! top in the file.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/restack.md`.

use super::dimdrive::{Fixture, click_on, press, run_on, ui_rect_event};
use crate::checks::driving::declared_or_in_overflow;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::Click;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const APPLIED: &str = "restack-applied"; // ui-text-exempt: a trace event name, never displayed
const EDIT_TAB: &str = "ribbon.tab.edit";
const SEND_TO_BACK: &str = "ribbon.item.edit.send_to_back";
const BOXES: Fixture = Fixture {
    file: "overlapping-boxes.pdf",
    method: "Rebuild it with `python fixtures/overlapping-boxes.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// Inside both boxes, page space.
const OVERLAP: (f64, f64) = (190.0, 140.0);
/// Inside the red box only.
const RED_ONLY: (f64, f64) = (80.0, 80.0);
const PROPERTIES: &str = "properties-panel"; // ui-text-exempt: a trace event name, never displayed
const GEOMETRY: &str = "geometry-draft"; // ui-text-exempt: a trace event name, never displayed
/// The red box's lower-left corner as Properties shows it.
const RED_ORIGIN: &str = "x=60.00 y=60.00";
/// The blue box's.
const BLUE_ORIGIN: &str = "x=160.00 y=100.00";

/// See the module documentation.
pub struct AnObjectCanBeBroughtForwardAndSentToBack;

impl Check for AnObjectCanBeBroughtForwardAndSentToBack {
    fn name(&self) -> &'static str {
        "an_object_can_be_brought_forward_and_sent_to_back"
    }

    fn defect(&self) -> &'static str {
        "a selected box on the page cannot be brought forward or sent back, or the command \
         reports a move the file does not show, or the selection is left on the object that \
         now occupies the old place"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &BOXES, INVOKE, "restack", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let on_top = top_at_overlap(ctx, session, pointer)?;
    report.note(format!("before: the overlap selects `{on_top}`"));
    if !on_top.contains(BLUE_ORIGIN) {
        return Ok(Some(fail(
            session,
            "before any command the overlap must select the blue box",
            &on_top,
        )));
    }
    click_on(ctx, session, pointer, &BOXES, 0, RED_ONLY)?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "CloseBracket", Some("ctrl"))?;
    session.settle(40);
    if let Some(failure) = applied(session, report, mark, "0", "1")? {
        return Ok(Some(failure));
    }
    if let Some(failure) = red_selected_as(session, mark, "1")? {
        return Ok(Some(failure));
    }
    let on_top = top_at_overlap(ctx, session, pointer)?;
    report.note(format!("after Ctrl+]: the overlap selects `{on_top}`"));
    if !on_top.contains(RED_ORIGIN) {
        return Ok(Some(fail(
            session,
            "after Ctrl+] the red box must be on top",
            &on_top,
        )));
    }
    let mark = session.trace()?.mark();
    press_ribbon(ctx, session, pointer, SEND_TO_BACK)?;
    if let Some(failure) = applied(session, report, mark, "1", "0")? {
        return Ok(Some(failure));
    }
    if let Some(failure) = red_selected_as(session, mark, "0")? {
        return Ok(Some(failure));
    }
    let on_top = top_at_overlap(ctx, session, pointer)?;
    report.note(format!(
        "after Send to back: the overlap selects `{on_top}`"
    ));
    if !on_top.contains(BLUE_ORIGIN) {
        return Ok(Some(fail(
            session,
            "after Send to back the blue box must be on top",
            &on_top,
        )));
    }
    Ok(None)
}

/// The `restack-applied` line after `mark` must carry both tokens.
fn applied(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    moved: &str,
    indices: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "the command reached no restack (no `{APPLIED}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("applied: `{}`", line.raw));
    let right = line.get("moved") == Some(moved) && line.get("indices") == Some(indices);
    Ok((!right).then(|| {
        format!(
            "the restack must read `moved={moved} indices={indices}`; it read `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )
    }))
}

/// After a restack the red box must still be the selection, under its new
/// index: Properties shows object `index` with the red box's geometry. A
/// selection left on the old index would show the blue box at (160,100).
fn red_selected_as(session: &Session, mark: usize, index: &str) -> Result<Option<String>> {
    let trace = session.trace()?;
    let panel = trace.last_after(PROPERTIES, mark).map(|l| l.raw.clone());
    let geometry = trace.last_after(GEOMETRY, mark).map(|l| l.raw.clone());
    let right = panel
        .as_deref()
        .is_some_and(|l| l.contains(&format!("object={index} ")))
        && geometry.as_deref().is_some_and(|l| l.contains(RED_ORIGIN));
    Ok((!right).then(|| {
        format!(
            "after the restack the red box must stay selected as object {index} (`{RED_ORIGIN}`); Properties read `{}` and `{}`. Trace: {}.",
            panel.unwrap_or_default(),
            geometry.unwrap_or_default(),
            session.trace_path().display()
        )
    }))
}

/// Click where the boxes overlap and read the geometry of whichever box the
/// click selected: the one on top in paint order.
fn top_at_overlap(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<String> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &BOXES, 0, OVERLAP)?;
    session.settle(10);
    Ok(session
        .trace()?
        .last_after(GEOMETRY, mark)
        .map_or_else(|| format!("<no {GEOMETRY} line>"), |l| l.raw.clone()))
}

fn press_ribbon(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    item: &str,
) -> Result<()> {
    let ui_rect = ui_rect_event(ctx)?;
    press(ctx, session, pointer, EDIT_TAB)?;
    let found = declared_or_in_overflow(session, pointer, ui_rect, item)?.ok_or_else(|| {
        crate::error::Error::new(format!(
            "no `{item}` on the Edit tab or in its overflow. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    pointer.click_rect(session, found)?;
    session.settle(40);
    Ok(())
}

fn fail(session: &Session, want: &str, got: &str) -> String {
    format!(
        "{want}; it read `{got}`. Trace: {}.",
        session.trace_path().display()
    )
}
