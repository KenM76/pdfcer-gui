//! The side panels follow what the operator does: Properties comes in front
//! of the Objects tree when a tool with options is armed and gives the front
//! back when it is put down, and the Objects tree follows an object picked
//! on the page without Properties taking its place.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/panels_follow.md`.

use super::dimdrive::{Fixture, click_on, press_on_tab, reveal, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,view.reset_layout";
const OBJECTS_TAB: &str = "dock.tab.view.panel_objects";
const MEASURE_TAB: &str = "ribbon.tab.measure";
const LINEAR: &str = "ribbon.item.measure.linear";
const ROW: &str = "objects.row.0";
const FORWARD: &str = "properties-forward"; // ui-text-exempt: a trace event name, never displayed
const BACK: &str = "properties-back"; // ui-text-exempt: a trace event name, never displayed
const FOLLOW: &str = "objects-follow"; // ui-text-exempt: a trace event name, never displayed
const PHOTOS: Fixture = Fixture {
    file: "two-photos.pdf",
    method: "Rebuild it with `python fixtures/two-photos.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 200.0,
    },
};
/// The second picture's centre, page space.
const PICTURE_B: (f64, f64) = (300.0, 100.0);

/// Arming the linear tool brings Properties forward; Escape gives it back.
pub struct PropertiesComesForwardForAnArmedTool;
/// A pick on the page is followed in the Objects tree.
pub struct ObjectsTreeFollowsAPagePick;

impl Check for PropertiesComesForwardForAnArmedTool {
    fn name(&self) -> &'static str {
        "properties_comes_forward_for_an_armed_tool_and_gives_the_front_back"
    }

    fn defect(&self) -> &'static str {
        "arming a tool with options leaves its options behind another tab, so the operator \
         has to hunt for them, or putting the tool down leaves Properties covering the tab \
         the operator had in front"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &PHOTOS, INVOKE, "propforward", forward)
    }
}

impl Check for ObjectsTreeFollowsAPagePick {
    fn name(&self) -> &'static str {
        "objects_tree_follows_an_object_picked_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "picking an object on the page leaves the Objects tree where it was, so the picked \
         row may be folded away or scrolled out of sight, or Properties covers the tree"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &PHOTOS, INVOKE, "objectsfollow", follow)
    }
}

/// Objects to the front of its stack, the mark after it.
fn objects_in_front(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<usize> {
    reveal(ctx, session, pointer, ROW, OBJECTS_TAB)?;
    session.settle(20);
    Ok(session.trace()?.mark())
}

/// `event` after `mark` with `key=want`, noted; the failure otherwise.
fn expect(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    (event, key, want): (&str, &str, &str),
    after: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.last_after(event, mark);
    report.note(format!(
        "{after}: `{}`",
        line.map_or("nothing", |l| l.raw.as_str())
    ));
    if line.and_then(|l| l.get(key)) == Some(want) {
        return Ok(None);
    }
    Ok(Some(format!(
        "{after}, `{event}` must read `{key}={want}`; it read `{}`. Trace: {}.",
        line.map_or("nothing", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}

fn forward(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = objects_in_front(ctx, session, pointer)?;
    press_on_tab(ctx, session, pointer, LINEAR, MEASURE_TAB)?;
    session.settle(20);
    let armed = (FORWARD, "tool", "true");
    if let Some(f) = expect(session, report, mark, armed, "arming the linear tool")? {
        return Ok(Some(f));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Escape", None)?;
    session.settle(30);
    let back = (BACK, "to", "view.panel_objects");
    expect(session, report, mark, back, "putting the tool down")
}

fn follow(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = objects_in_front(ctx, session, pointer)?;
    click_on(ctx, session, pointer, &PHOTOS, 0, PICTURE_B)?;
    session.settle(30);
    if let Some(f) = expect(
        session,
        report,
        mark,
        (FOLLOW, "shows", "true"),
        "a page pick",
    )? {
        return Ok(Some(f));
    }
    let trace = session.trace()?;
    if let Some(line) = trace.last_after(FORWARD, mark) {
        return Ok(Some(format!(
            "a page pick with the Objects tree in front brought Properties over it (`{}`); \
             the tree shows the pick. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    Ok(None)
}
