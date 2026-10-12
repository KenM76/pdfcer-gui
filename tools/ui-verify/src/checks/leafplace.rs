//! Typed X and Y for a part of a placed drawing, driven on
//! `half-scale-form.pdf`: the rectangle inside the drawing is entered, Properties
//! shows its geometry, a new X is typed and applied, and the part moves through
//! `move_objects_in_form` by exactly the difference. Then the line is added
//! with Shift and Align right (Ctrl+Alt+6) moves the parts through
//! `move_objects_each_in_form`; two boxes on `overlapping-boxes.pdf` align
//! through the page route as the control.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/leafplace.md`.

use super::dimdrive::{click_on, press, run_on};
use super::form_node_move::{enter_leaf_at, run_body};
use super::nudgecontent::{BOXES, INVOKE, RED_ONLY, last_geometry};
use super::stroke_style::{into_view, type_into};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "../../fixtures/half-scale-form.pdf";
/// On the rectangle inside the half-scale drawing, page space.
const ON_THE_RECT: (f64, f64) = (70.0, 65.0);
const GEOMETRY: &str = "geometry-draft"; // ui-text-exempt: a trace event name, never displayed
const MOVED: &str = "move-leaves-in-form"; // ui-text-exempt: a trace event name, never displayed
const X_FIELD: &str = "properties.geometry.x"; // ui-text-exempt: a trace region name, never displayed
const APPLY: &str = "properties.geometry.apply"; // ui-text-exempt: a trace region name, never displayed
/// How far the typed X moves the part, in points.
const SHIFT_PT: f64 = 10.0;

/// See the module documentation.
pub struct TypedXMovesAPartOfAPlacedDrawing;

impl Check for TypedXMovesAPartOfAPlacedDrawing {
    fn name(&self) -> &'static str {
        "typed_x_moves_a_part_of_a_placed_drawing"
    }

    fn defect(&self) -> &'static str {
        "a part of a placed drawing shows no X and Y in Properties, or a typed X does not move it \
         by the difference"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "leafplace", drive) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    enter_leaf_at(ctx, session, pointer, page, ON_THE_RECT)?;
    session.settle(20);
    let Some(x0) = draft_x(session, mark)? else {
        return Ok(Some(fail(
            session,
            "the part's X, Y, W and H must show in Properties",
            &format!("no `{GEOMETRY}` line"),
        )));
    };
    report.note(format!("the part's X before: {x0:.2}"));
    let want = x0 + SHIFT_PT;
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, X_FIELD, &format!("{want:.2}"))?;
    into_view(ctx, session, pointer, APPLY)?;
    press(ctx, session, pointer, APPLY)?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(moved) = trace.last_after(MOVED, mark) else {
        return Ok(Some(fail(
            session,
            &format!("Apply must reach the engine as `{MOVED}`"),
            "nothing",
        )));
    };
    report.note(format!("applied: `{}`", moved.raw));
    if moved.get_usize("n") != Some(1) {
        return Ok(Some(fail(session, "one part must be moved", &moved.raw)));
    }
    // After the move, so a typed draft that never reseeded cannot answer.
    let after = draft_x(session, moved.lineno)?;
    report.note(format!("the part's X after: {after:?}"));
    Ok(after.is_none_or(|x| (x - want).abs() > 0.01).then(|| {
        fail(
            session,
            &format!("Properties must re-read X={want:.2}"),
            &format!("{after:?}"),
        )
    }))
}

/// X from the last `geometry-draft` line after trace line `mark`.
fn draft_x(session: &Session, mark: usize) -> Result<Option<f64>> {
    Ok(session
        .trace()?
        .last_after(GEOMETRY, mark)
        .and_then(|l| l.get("x").and_then(|x| x.parse().ok())))
}

fn fail(session: &Session, want: &str, got: &str) -> String {
    format!(
        "{want}; it read `{got}`. Trace: {}.",
        session.trace_path().display()
    )
}

/// On the line inside the half-scale drawing, page space.
const ON_THE_LINE: (f64, f64) = (120.0, 95.0);
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const ALIGNED: &str = "move-each-in-form-applied"; // ui-text-exempt: a trace event name, never displayed
const ALIGN_DECLINED: &str = "align-declined"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct PartsOfAPlacedDrawingAlignToEachOther;

impl Check for PartsOfAPlacedDrawingAlignToEachOther {
    fn name(&self) -> &'static str {
        "parts_of_a_placed_drawing_align_to_each_other"
    }

    fn defect(&self) -> &'static str {
        "Align right with two parts of a placed drawing selected is refused or moves nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "leafalign", align) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn align(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    enter_leaf_at(ctx, session, pointer, page, ON_THE_RECT)?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let line = mapping.doc_to_window(DocPoint::new(0, ON_THE_LINE.0, ON_THE_LINE.1))?;
    pointer.send(
        session,
        &format!("click {:.1} {:.1} mods=shift", line.x(), line.y()),
    )?;
    session.settle(25);
    let selected = session
        .trace()?
        .last(SELECTION)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("after Shift+click on the line: `{selected}`"));
    if !(selected.contains(" sel=2 ") && selected.contains("first=leaf:")) {
        return Ok(Some(fail(
            session,
            "Shift+click must add the line to the rectangle, two parts selected",
            &selected,
        )));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "6", Some("ctrl+alt"))?;
    session.settle(40);
    let trace = session.trace()?;
    if let Some(line) = trace.last_after(ALIGN_DECLINED, mark) {
        return Ok(Some(fail(
            session,
            "Align right must move a part",
            &line.raw,
        )));
    }
    let Some(applied) = trace.last_after(ALIGNED, mark) else {
        return Ok(Some(fail(
            session,
            &format!("Align right must commit `{ALIGNED}`"),
            "nothing",
        )));
    };
    report.note(format!("Align right: `{}`", applied.raw));
    let moved = applied.get_usize("moved");
    Ok(
        (moved.is_none_or(|n| n == 0) || moved != applied.get_usize("of"))
            .then(|| fail(session, "every planned part must move", &applied.raw)),
    )
}

const PAGE_ALIGNED: &str = "move-each-applied"; // ui-text-exempt: a trace event name, never displayed
/// Inside the blue box only, page space.
const BLUE_ONLY: (f64, f64) = (300.0, 200.0);
/// Inside the red box only once it is aligned right onto the blue one.
const RED_AFTER: (f64, f64) = (300.0, 70.0);
/// The red box's origin once its right edge meets the blue box's.
const RED_ALIGNED: &str = "x=160.00 y=60.00";

/// See the module documentation.
pub struct PageObjectsAlignToEachOther;

impl Check for PageObjectsAlignToEachOther {
    fn name(&self) -> &'static str {
        "page_objects_align_to_each_other"
    }

    fn defect(&self) -> &'static str {
        "Align right with two page objects selected is refused, takes the parts route, or puts \
         the moved box in the wrong place"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &BOXES, INVOKE, "pagealign", page_align)
    }
}

fn page_align(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click_on(ctx, session, pointer, &BOXES, 0, RED_ONLY)?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, BOXES.page, 0)?;
    let blue = mapping.doc_to_window(DocPoint::new(0, BLUE_ONLY.0, BLUE_ONLY.1))?;
    pointer.send(
        session,
        &format!("click {:.1} {:.1} mods=shift", blue.x(), blue.y()),
    )?;
    session.settle(25);
    let selected = session
        .trace()?
        .last(SELECTION)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("after Shift+click on blue: `{selected}`"));
    if !selected.contains(" sel=2 ") || selected.contains("first=leaf:") {
        return Ok(Some(fail(
            session,
            "Shift+click must add blue to red, two page objects selected",
            &selected,
        )));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "6", Some("ctrl+alt"))?;
    session.settle(40);
    let trace = session.trace()?;
    if let Some(line) = trace.last_after(ALIGN_DECLINED, mark) {
        return Ok(Some(fail(session, "Align right must move red", &line.raw)));
    }
    let Some(applied) = trace.last_after(PAGE_ALIGNED, mark) else {
        return Ok(Some(fail(
            session,
            &format!("Align right must commit `{PAGE_ALIGNED}`"),
            "nothing",
        )));
    };
    report.note(format!("Align right: `{}`", applied.raw));
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &BOXES, 0, RED_AFTER)?;
    session.settle(10);
    let after = last_geometry(session, mark)?;
    report.note(format!("red alone afterwards: `{after}`"));
    Ok((!after.contains(RED_ALIGNED))
        .then(|| fail(session, &format!("red must sit at `{RED_ALIGNED}`"), &after)))
}
