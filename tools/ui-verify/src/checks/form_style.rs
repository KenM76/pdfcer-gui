//! Properties' colour and "Line and opacity" sections on a part of a placed
//! drawing, driven on a form placed at half scale: a width typed in points
//! reaches the form's stream doubled, and a colour picked reaches the part.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/form_style.md`.

use super::dimdrive::{press, ui_rect_event};
use super::form_node_move::{enter_leaf_at, run_body};
use super::stroke_style::{into_view, shown_since, type_into};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "../../fixtures/half-scale-form.pdf";
const CALLS: &str = "stroke-style-calls"; // ui-text-exempt: a trace event name, never displayed
const PAINT: &str = "paint-shown"; // ui-text-exempt: a trace event name, never displayed
/// The label `app::actions::apply::vector_edit` writes when the recolour
/// reached the engine.
const APPLIED: &str = "set-object-paint"; // ui-text-exempt: a trace event name, never displayed
/// Mirrors `panels::properties::{strokestyle, paint}`'s regions.
const WIDTH: &str = "properties.stroke.width";
const LINE_ALPHA: &str = "properties.stroke.line-opacity";
const FILL_SWATCH: &str = "properties.paint.fill";
const FILL_PICKER: &str = "properties.paint.fill.picker";
/// The blue line, leaf 0: page y 95, 2 pt wide.
const ON_THE_LINE: (f64, f64) = (120.0, 95.0);
/// The red rectangle, leaf 1: page (50,50)..(90,80).
const ON_THE_RECT: (f64, f64) = (70.0, 65.0);
/// The form's placement scale.
const SCALE: f64 = 0.5;

/// A width typed in points on a part of a placed drawing lands in the form's
/// own units, and opacity reaches it.
pub struct APartOfAPlacedDrawingTakesAWidthInPoints;
/// A colour picked for a part of a placed drawing reaches it.
pub struct APartOfAPlacedDrawingCanBeRecoloured;

impl Check for APartOfAPlacedDrawingTakesAWidthInPoints {
    fn name(&self) -> &'static str {
        "a_part_of_a_placed_drawing_takes_a_width_in_points"
    }

    fn defect(&self) -> &'static str {
        "a line inside a placed drawing offers no width or opacity, or a width typed in points \
         is written in the drawing's own units, so a drawing placed at half scale ends at half \
         the width typed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "form-style-width", width) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

impl Check for APartOfAPlacedDrawingCanBeRecoloured {
    fn name(&self) -> &'static str {
        "a_part_of_a_placed_drawing_can_be_recoloured"
    }

    fn defect(&self) -> &'static str {
        "a shape inside a placed drawing offers no colour, or the colour picked never reaches it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match run_body(ctx, &mut report, FIXTURE, "form-style-colour", colour) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn width(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    enter_leaf_at(ctx, session, pointer, page, ON_THE_LINE)?;
    session.settle(20);
    let shown = shown_since(session, 0)?;
    report.note(format!("selected: `{shown}`"));
    if !(shown.contains("leaves=true") && shown.contains("width_pt=2.000")) {
        return Ok(Some(format!(
            "the line inside the half-scale drawing (4 units, 2 pt on the page) reads `{shown}`; \
             it must read as a part (`leaves=true`) at `width_pt=2.000`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, WIDTH, "5")?;
    let trace = session.trace()?;
    let Some(calls) = trace.last_after(CALLS, mark) else {
        return Ok(Some(format!(
            "5 was typed into the width field and nothing was applied (no `{CALLS}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("width applied: `{}`", calls.raw));
    let want_user = 5.0 / SCALE;
    if calls.get("user_widths") != Some(&format!("{want_user:.3}")[..]) {
        return Ok(Some(format!(
            "5 pt on a drawing placed at scale {SCALE} must reach the engine as {want_user:.3} \
             of the drawing's units; the call was `{}`. Trace: {}.",
            calls.raw,
            session.trace_path().display()
        )));
    }
    let shown = shown_since(session, mark)?;
    if !shown.contains("width_pt=5.000") {
        return Ok(Some(format!(
            "the width was applied and the part re-reads `{shown}`; it must read 5 pt. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, LINE_ALPHA, "40")?;
    let shown = shown_since(session, mark)?;
    report.note(format!("after line opacity 40 %: `{shown}`"));
    Ok((!shown.contains("line_alpha=0.400")).then(|| {
        format!(
            "line opacity 40 % was typed and the part re-reads `{shown}`; the opacity must be \
             bound in the drawing's own resources and read back 0.400. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

fn colour(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    enter_leaf_at(ctx, session, pointer, page, ON_THE_RECT)?;
    session.settle(20);
    let before = paint_since(session, 0)?;
    report.note(format!("selected: `{before}`"));
    if !(before.contains("leaves=true") && before.contains("fill=255,0,0")) {
        return Ok(Some(format!(
            "the red rectangle inside the drawing shows `{before}`; it must show one part \
             (`leaves=true`) filled `255,0,0`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let mark = session.trace()?.mark();
    into_view(ctx, session, pointer, FILL_SWATCH)?;
    press(ctx, session, pointer, FILL_SWATCH)?;
    let ui_rect = ui_rect_event(ctx)?;
    let Some((picker, vp)) = declared_in(&session.trace()?, ui_rect, FILL_PICKER) else {
        return Ok(Some(format!(
            "the fill swatch was pressed and no picker opened (`{FILL_PICKER}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    // The lower part of the picker is its saturation/value square; the top
    // holds the preview button, which opens a nested popup.
    pointer.click_in(session, vp.as_deref(), at_fraction(picker, 0.35, 0.80))?;
    session.settle(16);
    // The close is the commit: `swatch::show` answers on the frame it closes.
    pointer.key(session, vp.as_deref(), "Escape", None)?;
    session.settle(40);
    let trace = session.trace()?;
    if trace.last_after(APPLIED, mark).is_none() {
        return Ok(Some(format!(
            "a colour was picked and the picker closed, and no recolour reached the engine (no \
             `{APPLIED}`). Trace: {}.",
            session.trace_path().display()
        )));
    }
    let after = paint_since(session, mark)?;
    report.note(format!("after the pick: `{after}`"));
    Ok(
        (!after.contains("leaves=true") || after.contains("fill=255,0,0")).then(|| {
            format!(
                "the recolour was applied and the part re-reads `{after}`; it must read a fill \
             other than the red it started with. Trace: {}.",
                session.trace_path().display()
            )
        }),
    )
}

/// The point `(fx, fy)` of the way across `r`.
fn at_fraction(r: LRect, fx: f32, fy: f32) -> WindowPoint {
    let p = Pt {
        x: r.min.x + (r.max.x - r.min.x) * fx,
        y: r.min.y + (r.max.y - r.min.y) * fy,
    };
    WindowPoint::centre_of(LRect { min: p, max: p })
}

/// The last `paint-shown` line after trace line `mark`, or a token every
/// caller's assertion rejects.
fn paint_since(session: &Session, mark: usize) -> Result<String> {
    Ok(session
        .trace()?
        .last_after(PAINT, mark)
        .map_or_else(|| format!("<no {PAINT} line>"), |l| l.raw.clone()))
}
