//! Align and drag with a selection that mixes parts of placed drawings with
//! page objects, driven on `parts-beside-objects.pdf`: a part and a page object
//! move together as one step; a part whose own drawing is selected rides with
//! it (and Align says so); Align at the Part rung says why it does nothing.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/alignmixed.md`.

use super::form_node_move::{enter_leaf_at, run_body};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: &str = "../../fixtures/parts-beside-objects.pdf";
/// On the red rectangle inside drawing Fm0, page space.
const ON_RED: (f64, f64) = (90.0, 80.0);
/// On the blue rectangle inside drawing Fm0, page space.
const ON_BLUE: (f64, f64) = (80.0, 155.0);
/// On the green page rectangle, page space.
const ON_GREEN: (f64, f64) = (330.0, 240.0);
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const MIXED: &str = "move-mixed-applied"; // ui-text-exempt: a trace event name, never displayed
const FUNNEL: &str = "move-mixed"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "align-declined"; // ui-text-exempt: a trace event name, never displayed
const MOVED: &str = "canvas-move"; // ui-text-exempt: a trace event name, never displayed
/// How far the drag checks move the green rectangle, page space.
const DRAG_BY: (f64, f64) = (20.0, -15.0);

/// See the module documentation.
pub struct APartAndAPageObjectAlignTogether;

impl Check for APartAndAPageObjectAlignTogether {
    fn name(&self) -> &'static str {
        "a_part_and_a_page_object_align_together"
    }

    fn defect(&self) -> &'static str {
        "Align centres with a part of a placed drawing and a page object selected moves only one \
         kind, or takes two undo steps"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "alignmixed", together) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn together(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    if let Some(failure) = red_and_green(ctx, report, session, pointer, page)? {
        return Ok(Some(failure));
    }
    let applied = match align_centres(report, session, pointer)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    let wanted = [
        "objects=1/1",
        "parts=1/1",
        "drawings=1/1",
        "one_step=true",
        "notes=0",
    ];
    Ok(missing(&applied.raw, &wanted).map(|w| {
        fail(
            session,
            &format!("the part and the page object must both move as one step (`{w}`)"),
            &applied.raw,
        )
    }))
}

/// See the module documentation.
pub struct APartInsideASelectedDrawingRidesWithIt;

impl Check for APartInsideASelectedDrawingRidesWithIt {
    fn name(&self) -> &'static str {
        "a_part_inside_a_selected_drawing_rides_with_it"
    }

    fn defect(&self) -> &'static str {
        "Align centres with a part and its own drawing both selected moves the part twice, or drops \
         it without saying so"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "aligncarried", carried) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn carried(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    if let Some(failure) = red_and_green(ctx, report, session, pointer, page)? {
        return Ok(Some(failure));
    }
    // Out of the drawing now, so this click resolves to the whole of Fm0.
    let selected = shift_click(ctx, session, pointer, page, ON_BLUE)?;
    report.note(format!("after Shift+click on the blue part: `{selected}`"));
    if !selected.contains(" sel=3 ") {
        return Ok(Some(fail(
            session,
            "Shift+click on the drawing must add it whole, three entries selected",
            &selected,
        )));
    }
    let applied = match align_centres(report, session, pointer)? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    let wanted = ["objects=2/2", "parts=0/0", "drawings=0/0", "notes=1"];
    if let Some(w) = missing(&applied.raw, &wanted) {
        return Ok(Some(fail(
            session,
            &format!("the drawing must carry its part, and say so (`{w}`)"),
            &applied.raw,
        )));
    }
    // The funnel's own line: what reached the status line, not what was asked.
    let said = session
        .trace()?
        .last(FUNNEL)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("committed: `{said}`"));
    Ok(
        (said.is_empty() || said.contains("disclosures=none")).then(|| {
            fail(
                session,
                "the status line must be handed the carried part's sentence",
                &said,
            )
        }),
    )
}

/// See the module documentation.
pub struct AlignAtThePartRungSaysWhy;

impl Check for AlignAtThePartRungSaysWhy {
    fn name(&self) -> &'static str {
        "align_at_the_part_rung_says_why"
    }

    fn defect(&self) -> &'static str {
        "Align with a subpath selected inside an object does nothing and says nothing selected"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "alignpartrung", part_rung) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn part_rung(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    let (_, at) = enter_leaf_at(ctx, session, pointer, page, ON_RED)?;
    pointer.double_click(session, at)?;
    session.settle(30);
    let selected = last_selection(session)?;
    report.note(format!("after a second double-click: `{selected}`"));
    if !selected.contains("level=Part") {
        return Ok(Some(fail(
            session,
            "a second double-click must stand at the Part rung",
            &selected,
        )));
    }
    let mark = session.trace()?.mark();
    pointer.key(session, None, "6", Some("ctrl+alt"))?;
    session.settle(40);
    let trace = session.trace()?;
    if let Some(moved) = trace.last_after(MIXED, mark) {
        return Ok(Some(fail(session, "nothing must move", &moved.raw)));
    }
    let declined = trace
        .last_after(DECLINED, mark)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("Align right: `{declined}`"));
    Ok((!declined.contains("reason=part-rung")).then(|| {
        fail(
            session,
            "Align must decline with `reason=part-rung`",
            &declined,
        )
    }))
}

/// See the module documentation.
pub struct APartAndAPageObjectDragTogether;

impl Check for APartAndAPageObjectDragTogether {
    fn name(&self) -> &'static str {
        "a_part_and_a_page_object_drag_together"
    }

    fn defect(&self) -> &'static str {
        "Dragging a selection of a part of a placed drawing and a page object moves only the page \
         object, or takes two undo steps"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "dragmixed", drag_together) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drag_together(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    if let Some(failure) = red_and_green(ctx, report, session, pointer, page)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    drag_green(ctx, session, pointer, page)?;
    let trace = session.trace()?;
    let Some(applied) = trace.last_after(MIXED, mark) else {
        let moved = trace
            .last_after(MOVED, mark)
            .map_or("nothing", |l| l.raw.as_str());
        return Ok(Some(fail(
            session,
            &format!("the drag must commit `{MIXED}`"),
            moved,
        )));
    };
    report.note(format!("drag: `{}`", applied.raw));
    let wanted = [
        "gesture=drag",
        "objects=1/1",
        "parts=1/1",
        "drawings=1/1",
        "one_step=true",
    ];
    Ok(missing(&applied.raw, &wanted).map(|w| {
        fail(
            session,
            &format!("the part and the page object must both move as one step (`{w}`)"),
            &applied.raw,
        )
    }))
}

/// See the module documentation.
pub struct APartInsideADraggedDrawingRidesWithIt;

impl Check for APartInsideADraggedDrawingRidesWithIt {
    fn name(&self) -> &'static str {
        "a_part_inside_a_dragged_drawing_rides_with_it"
    }

    fn defect(&self) -> &'static str {
        "Dragging a part together with its own drawing moves the part twice"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "dragcarried", drag_carried) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drag_carried(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    if let Some(failure) = red_and_green(ctx, report, session, pointer, page)? {
        return Ok(Some(failure));
    }
    let selected = shift_click(ctx, session, pointer, page, ON_BLUE)?;
    report.note(format!("after Shift+click on the blue part: `{selected}`"));
    if !selected.contains(" sel=3 ") {
        return Ok(Some(fail(
            session,
            "Shift+click on the drawing must add it whole, three entries selected",
            &selected,
        )));
    }
    let mark = session.trace()?.mark();
    drag_green(ctx, session, pointer, page)?;
    let trace = session.trace()?;
    if let Some(mixed) = trace.last_after(MIXED, mark) {
        return Ok(Some(fail(
            session,
            "the drawing carries its part, so the drag is a plain page move",
            &mixed.raw,
        )));
    }
    let moved = trace
        .last_after(MOVED, mark)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    report.note(format!("drag: `{moved}`"));
    // The drawing has no coordinate operands, so the pair moves through
    // `TransformObjects`; either page verb is right, so the operands are asserted.
    Ok((!moved.contains("objects: [0, 1]")).then(|| {
        fail(
            session,
            "the drag must move the two page objects, and only them, in one page action",
            &moved,
        )
    }))
}

/// Drag the green rectangle by `DRAG_BY` page units.
fn drag_green(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<()> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let from = mapping.doc_to_window(DocPoint::new(0, ON_GREEN.0, ON_GREEN.1))?;
    let to = mapping.doc_to_window(DocPoint::new(
        0,
        ON_GREEN.0 + DRAG_BY.0,
        ON_GREEN.1 + DRAG_BY.1,
    ))?;
    pointer.drag(session, from, to, 8)?;
    session.settle(40);
    Ok(())
}

/// Enter the red part, then Shift+click the green page rectangle: one leaf
/// and one page object selected. `Some` is a FAIL.
fn red_and_green(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    enter_leaf_at(ctx, session, pointer, page, ON_RED)?;
    let selected = shift_click(ctx, session, pointer, page, ON_GREEN)?;
    report.note(format!("after Shift+click on green: `{selected}`"));
    Ok(
        // Entries are kept sorted, page objects first, so `first=` names the
        // rectangle; the part's survival is read off the move's `parts=`.
        (!(selected.contains(" sel=2 ") && selected.contains("level=Object"))).then(|| {
            fail(
                session,
                "Shift+click must add the page rectangle to the part, two entries selected",
                &selected,
            )
        }),
    )
}

/// Shift+click at page point `on`; the last `canvas-selection` line after it.
fn shift_click(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
    on: (f64, f64),
) -> Result<String> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, on.0, on.1))?;
    pointer.send(
        session,
        &format!("click {:.1} {:.1} mods=shift", at.x(), at.y()),
    )?;
    session.settle(25);
    last_selection(session)
}

fn last_selection(session: &Session) -> Result<String> {
    Ok(session
        .trace()?
        .last(SELECTION)
        .map(|l| l.raw.clone())
        .unwrap_or_default())
}

/// Ctrl+Alt+7, Align centres left to right: every entry not already on the
/// selection's centre line moves, so neither kind can sit out as the anchor.
/// The `move-mixed-applied` line, or the FAIL sentence.
fn align_centres(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<TraceLine, String>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "7", Some("ctrl+alt"))?;
    session.settle(40);
    let trace = session.trace()?;
    if let Some(line) = trace.last_after(DECLINED, mark) {
        return Ok(Err(fail(session, "Align centres must move", &line.raw)));
    }
    let Some(applied) = trace.last_after(MIXED, mark) else {
        return Ok(Err(fail(
            session,
            &format!("Align centres must commit `{MIXED}`"),
            "nothing",
        )));
    };
    report.note(format!("Align centres: `{}`", applied.raw));
    Ok(Ok(applied.clone()))
}

/// The first of `wanted` that `raw` does not hold as a whole field.
fn missing<'a>(raw: &str, wanted: &[&'a str]) -> Option<&'a str> {
    let fields: Vec<&str> = raw.split_whitespace().collect();
    wanted.iter().copied().find(|w| !fields.contains(w))
}

fn fail(session: &Session, want: &str, got: &str) -> String {
    format!(
        "{want}; it read `{got}`. Trace: {}.",
        session.trace_path().display()
    )
}
