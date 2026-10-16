//! `tab_walks_the_objects_on_a_clicked_page` — after a click on the page,
//! Tab selects the page's objects one after another and Shift+Tab steps back,
//! instead of walking egui's focus into the ribbon — `OPERATOR_REQUESTS.md`
//! O204, its canvas half. Driven off-screen with the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/tab_object_ring.md`.

use super::dimdrive::{Fixture, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// Three separate squares on an upright 300 × 200 pt sheet.
const FIXTURE: Fixture = Fixture {
    file: "three-squares.pdf",
    method: "It is hand-written PDF syntax from `fixtures/three-squares.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 300.0,
        height_pt: 200.0,
    },
};

const INVOKE: &str = "mode.edit,view.page_single";

/// Blank paper above the squares, in PDF user space: the click gives the page
/// the keyboard and selects nothing.
const PAPER: (f64, f64) = (150.0, 170.0);

/// The `app::keyclaim` plugin took a Tab press for a canvas ring.
const CLAIM: &str = "tab-claim";
/// The object ring spent one.
const MOVED: &str = "tab-object";

/// See the module documentation.
pub struct TabWalksTheObjectsOnAClickedPage;

impl Check for TabWalksTheObjectsOnAClickedPage {
    fn name(&self) -> &'static str {
        "tab_walks_the_objects_on_a_clicked_page"
    }

    fn defect(&self) -> &'static str {
        "Tab pressed after clicking a page walks egui's focus ring into the ribbon instead of \
         selecting the next object on the page, because clicking a page focused nothing and \
         egui hands an unowned Tab to the first focusable widget of the frame"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "tab_object_ring", drive)
    }
}

/// Presses Tab (Shift+Tab when `back`) and returns the ring's new line.
fn press(session: &Session, pointer: &ScriptedPointer, back: bool) -> Result<Option<TraceLine>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Tab", back.then_some("shift"))?;
    session.settle(25);
    Ok(session.trace()?.last_after(MOVED, mark).cloned())
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, PAPER.0, PAPER.1))?;
    pointer.click(session, at)?;
    session.settle(25);
    let path = session.trace_path().display().to_string();

    let Some(first) = press(session, pointer, false)? else {
        let claimed = session
            .trace()?
            .events(CLAIM)
            .any(|l| l.get("scope") == Some("object"));
        return Ok(Some(if claimed {
            format!(
                "Tab was claimed for the object ring and the ring spent nothing: no `{MOVED}` \
                 line. Look for `tab-object-unmodelled` or `tab-object-empty`. Trace: {path}."
            )
        } else {
            format!(
                "Tab pressed after a click on the page was never claimed for the object ring: \
                 no `{CLAIM} scope=object` line. The press went to egui's own focus walk, which \
                 is the ribbon — the operator's report. Does a page response still ask for \
                 focus on click? Trace: {path}."
            )
        }));
    };
    let stops = first.get_usize("stops").unwrap_or(0);
    if stops < 3 {
        return Err(Error::new(format!(
            "the ring has {stops} stop(s) on a sheet of three squares; a ring under three \
             cannot tell a step from a bounce. SKIPPED. `{}`",
            first.raw
        )));
    }
    let Some(second) = press(session, pointer, false)? else {
        return Ok(Some(format!(
            "the first Tab selected an object and the second spent nothing. Trace: {path}."
        )));
    };
    let (a, b) = (first.get("to"), second.get("to"));
    if a == b || second.get("backwards") != Some("false") {
        return Ok(Some(format!(
            "a second Tab did not move forward to another object: `{}` then `{}`.",
            first.raw, second.raw
        )));
    }
    report.note(format!("Tab selected {a:?}, then {b:?}, of {stops} stops"));
    let Some(back) = press(session, pointer, true)? else {
        return Ok(Some(format!(
            "Shift+Tab spent nothing on the object ring. Trace: {path}."
        )));
    };
    if back.get("backwards") != Some("true") || back.get("to") != a {
        return Ok(Some(format!(
            "Shift+Tab did not step back to {a:?}: `{}`. Trace: {path}.",
            back.raw
        )));
    }
    let selected = session
        .trace()?
        .events("selection-set")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if !selected.contains("via=tab-object") {
        return Ok(Some(format!(
            "the ring stepped and the selection was not set by it: last `selection-set` is \
             `{selected}`. Trace: {path}."
        )));
    }
    report.note(format!("Shift+Tab returned to {a:?}: `{selected}`"));
    Ok(None)
}
