//! The arrow keys on page content, driven on `overlapping-boxes.pdf`: a
//! selected box moves one point a press through the drag's own commit, and a
//! quarter point with Ctrl, with Properties reading the new origin each time.
//! A part inside a placed drawing, on `half-scale-form.pdf`, moves through
//! the same commit into the drawing.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/nudgecontent.md`.

use super::dimdrive::{Fixture, click_on, run_on};
use super::form_node_move::{enter_leaf_at, run_body};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const MOVED: &str = "canvas-move"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "canvas-move-declined"; // ui-text-exempt: a trace event name, never displayed
const SHELL_DECLINED: &str = "canvas-decline"; // ui-text-exempt: a trace event name, never displayed
const LEAVES_MOVED: &str = "move-leaves-in-form"; // ui-text-exempt: a trace event name, never displayed
const FORM_FIXTURE: &str = "../../fixtures/half-scale-form.pdf";
/// On the line inside the half-scale drawing, page space.
const ON_THE_LINE: (f64, f64) = (120.0, 95.0);
const GEOMETRY: &str = "geometry-draft"; // ui-text-exempt: a trace event name, never displayed
const BOXES: Fixture = Fixture {
    file: "overlapping-boxes.pdf",
    method: "Rebuild it with `python fixtures/overlapping-boxes.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 300.0,
    },
};
/// Inside the red box only, page space.
const RED_ONLY: (f64, f64) = (80.0, 80.0);
/// The red box's origin as Properties shows it, before any press.
const RED_ORIGIN: &str = "x=60.00 y=60.00";

/// One press and what Properties must read after it.
struct Step {
    key: &'static str,
    modifiers: Option<&'static str>,
    origin: &'static str,
}

/// Right, Up, then Ctrl+Left: a full step on each axis with Up proving the
/// screen-to-PDF sign, then the fine step.
const STEPS: [Step; 3] = [
    Step {
        key: "ArrowRight",
        modifiers: None,
        origin: "x=61.00 y=60.00",
    },
    Step {
        key: "ArrowUp",
        modifiers: None,
        origin: "x=61.00 y=61.00",
    },
    Step {
        key: "ArrowLeft",
        modifiers: Some("ctrl"),
        origin: "x=60.75 y=61.00",
    },
];

/// See the module documentation.
pub struct ArrowKeysMoveASelectedObjectOnThePage;

impl Check for ArrowKeysMoveASelectedObjectOnThePage {
    fn name(&self) -> &'static str {
        "arrow_keys_move_a_selected_object_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "an arrow key with a box selected on the page does nothing, moves it the wrong way \
         or the wrong distance, or Properties does not show where it went"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &BOXES, INVOKE, "nudgecontent", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &BOXES, 0, RED_ONLY)?;
    session.settle(10);
    let before = last_geometry(session, mark)?;
    if !before.contains(RED_ORIGIN) {
        return Ok(Some(fail(
            session,
            &format!("the click must select the red box at `{RED_ORIGIN}`"),
            &before,
        )));
    }
    for step in &STEPS {
        let mark = session.trace()?.mark();
        pointer.key(session, None, step.key, step.modifiers)?;
        session.settle(40);
        let trace = session.trace()?;
        if let Some(line) = trace.last_after(DECLINED, mark) {
            return Ok(Some(fail(
                session,
                &format!("{} must move the box", step.key),
                &line.raw,
            )));
        }
        let Some(moved) = trace.last_after(MOVED, mark).map(|l| l.raw.clone()) else {
            return Ok(Some(fail(
                session,
                &format!("{} must raise a `{MOVED}`", step.key),
                "nothing",
            )));
        };
        let after = last_geometry(session, mark)?;
        report.note(format!("{}: `{moved}`, then `{after}`", step.key));
        if !after.contains(step.origin) {
            return Ok(Some(fail(
                session,
                &format!("after {} Properties must read `{}`", step.key, step.origin),
                &after,
            )));
        }
    }
    Ok(None)
}

fn last_geometry(session: &Session, mark: usize) -> Result<String> {
    Ok(session
        .trace()?
        .last_after(GEOMETRY, mark)
        .map_or_else(|| format!("<no {GEOMETRY} line>"), |l| l.raw.clone()))
}

fn fail(session: &Session, want: &str, got: &str) -> String {
    format!(
        "{want}; it read `{got}`. Trace: {}.",
        session.trace_path().display()
    )
}

/// See the module documentation.
pub struct AnArrowKeyMovesAPartOfAPlacedDrawing;

impl Check for AnArrowKeyMovesAPartOfAPlacedDrawing {
    fn name(&self) -> &'static str {
        "an_arrow_key_moves_a_part_of_a_placed_drawing"
    }

    fn defect(&self) -> &'static str {
        "an arrow key with a part of a placed drawing selected is refused or does nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(
            ctx,
            &mut report,
            FORM_FIXTURE,
            "nudgecontent-leaf",
            nudge_leaf,
        ) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn nudge_leaf(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    enter_leaf_at(ctx, session, pointer, page, ON_THE_LINE)?;
    session.settle(20);
    let mark = session.trace()?.mark();
    pointer.key(session, None, "ArrowRight", None)?;
    session.settle(40);
    let trace = session.trace()?;
    for refusal in [DECLINED, SHELL_DECLINED] {
        if let Some(line) = trace.last_after(refusal, mark) {
            return Ok(Some(fail(
                session,
                "ArrowRight must move the part",
                &line.raw,
            )));
        }
    }
    let Some(moved) = trace.last_after(LEAVES_MOVED, mark) else {
        return Ok(Some(fail(
            session,
            &format!("ArrowRight must reach the engine as `{LEAVES_MOVED}`"),
            "nothing",
        )));
    };
    report.note(format!("ArrowRight: `{}`", moved.raw));
    Ok((moved.get_usize("n") != Some(1)).then(|| {
        fail(
            session,
            "one part was selected and one must be moved",
            &moved.raw,
        )
    }))
}
