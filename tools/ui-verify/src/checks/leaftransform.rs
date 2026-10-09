//! `a_part_of_a_placed_drawing_can_be_resized_and_rotated` — go inside a
//! placed drawing, drag the selection's corner grip and then its rotate
//! handle, and the engine's form-scoped transform applies both.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/leaftransform.md`.

use crate::checks::driving;
use crate::checks::form_node_move::{enter_leaf_at, run_body};
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: &str = "../../fixtures/form-parts.pdf";
const APPLIED: &str = "transform-leaves-in-form-applied"; // ui-text-exempt: a trace event name, never displayed
/// Mirrors `canvas::overlay::SELECTION_OUTLINE_REGION`.
const OUTLINE: &str = "canvas.selection-outline";
/// Mirrors `canvas::overlay::ROTATE_HANDLE_REGION`.
const ROTATE_HANDLE: &str = "canvas.rotate-handle";
/// Mid-segment of the polyline's first leg, leaf 1 of `form-parts.pdf`.
const ON_THE_POLYLINE: (f64, f64) = (240.0, 120.0);
/// How far the south-east grip is dragged outward, window points per axis.
const GROW_PX: f32 = 40.0;

/// See the module documentation.
pub struct APartOfAPlacedDrawingCanBeResizedAndRotated;

impl Check for APartOfAPlacedDrawingCanBeResizedAndRotated {
    fn name(&self) -> &'static str {
        "a_part_of_a_placed_drawing_can_be_resized_and_rotated"
    }

    fn defect(&self) -> &'static str {
        "an object inside a placed drawing shows resize grips and a rotate handle, and dragging \
         them refuses or changes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "leaf-transform", drive) {
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
    let (ui_rect, _) = enter_leaf_at(ctx, session, pointer, page, ON_THE_POLYLINE)?;
    let before = region(session, ui_rect, OUTLINE)?;

    let mark = session.trace()?.mark();
    let grip = point(before.max.x, before.max.y);
    pointer.drag(
        session,
        grip,
        point(before.max.x + GROW_PX, before.max.y + GROW_PX),
        8,
    )?;
    session.settle(40);
    let line = match applied(session, mark, "the corner grip")? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!("resize: `{}`", line.raw));
    let after = region(session, ui_rect, OUTLINE)?;
    let grew = after.width() > before.width() + GROW_PX / 2.0
        && after.height() > before.height() + GROW_PX / 2.0;
    if !grew || !scales_up(&line) {
        return Ok(Some(format!(
            "the corner drag must scale the leaf up (`m=` with a and d above 1) and the outline \
             must grow by about {GROW_PX} points; it read `{}`, outline {:.1} x {:.1} -> {:.1} x \
             {:.1}. Trace: {}.",
            line.raw,
            before.width(),
            before.height(),
            after.width(),
            after.height(),
            session.trace_path().display()
        )));
    }

    let handle = region(session, ui_rect, ROTATE_HANDLE)?;
    let from = WindowPoint::centre_of(handle);
    let centre = WindowPoint::centre_of(after);
    // Due north of the centre to due east of it: a quarter turn.
    let to = point(centre.x() + (centre.y() - from.y()), centre.y());
    let mark = session.trace()?.mark();
    pointer.drag(session, from, to, 12)?;
    session.settle(40);
    let line = match applied(session, mark, "the rotate handle")? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    report.note(format!("rotate: `{}`", line.raw));
    if !quarter_turn(&line) {
        return Ok(Some(format!(
            "the rotate drag must turn the leaf a quarter turn (`m=` with |b| near 1); it read \
             `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    Ok(None)
}

fn point(x: f32, y: f32) -> WindowPoint {
    WindowPoint::centre_of(LRect::new(Pt::new(x, y), Pt::new(x, y)))
}

fn region(session: &Session, ui_rect: &str, name: &str) -> Result<LRect> {
    driving::declared(&session.trace()?, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region with a placed drawing's part selected. Trace: {}.",
            session.trace_path().display()
        ))
    })
}

/// The applied line after `mark`, naming leaf 1 of page 0. The inner `Err` is
/// the failure.
fn applied(
    session: &Session,
    mark: usize,
    route: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(APPLIED, mark) else {
        let refused = trace.last_after("transform-leaves-in-form-refused", mark);
        return Ok(Err(format!(
            "dragging {route} reached no `{APPLIED}` (engine refusal: {}). Trace: {}.",
            refused.map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    if line.get("page") == Some("0") && line.get("leaves") == Some("1") {
        return Ok(Ok(line.clone()));
    }
    Ok(Err(format!(
        "dragging {route} transformed `{}`, not leaf 1 of page 0. Trace: {}.",
        line.raw,
        session.trace_path().display()
    )))
}

/// The `m=[a b c d e f]` entries.
fn matrix(line: &TraceLine) -> Option<[f64; 6]> {
    let inner = line.raw.split("m=[").nth(1)?.split(']').next()?;
    let v: Vec<f64> = inner
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    v.try_into().ok()
}

fn scales_up(line: &TraceLine) -> bool {
    matrix(line).is_some_and(|m| m[0] > 1.05 && m[3] > 1.05 && m[1].abs() < 1e-3)
}

fn quarter_turn(line: &TraceLine) -> bool {
    matrix(line).is_some_and(|m| m[0].abs() < 0.1 && m[1].abs() > 0.9)
}
