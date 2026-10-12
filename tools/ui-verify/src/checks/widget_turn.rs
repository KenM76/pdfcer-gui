//! A form field's box tilts by its rotate grip and stands back up from
//! Properties, and its Opacity row writes `/CA`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/widget_turn.md`.

use super::dimdrive::{Fixture, click_on, press, run_on, ui_rect_event};
use super::stroke_style::{into_view, type_into};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: Fixture = Fixture {
    file: "widget-turn.pdf",
    method: "Rebuild it with `python fixtures/widget-turn.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const SELECTED: &str = "form-field-selected"; // ui-text-exempt: a trace event name, never displayed
const TURNED: &str = "turn-widget-applied"; // ui-text-exempt: a trace event name, never displayed
const EDITED: &str = "edit-widget-applied"; // ui-text-exempt: a trace event name, never displayed
/// Mirror `canvas::overlay::ROTATE_HANDLE_REGION` and
/// `panels::properties::widgetturn`.
const HANDLE: &str = "canvas.rotate-handle";
const UPRIGHT: &str = "properties.widget_edit.upright";
const OPACITY: &str = "properties.widget_edit.opacity";
/// The centre of the field's `/Rect [100 180 300 220]`, page space: the
/// rotate drag's pivot.
const CENTRE: (f64, f64) = (200.0, 200.0);
/// How far the drag turns the handle about the centre, anticlockwise.
const TILT: f32 = 30.0;
/// How far the committed angle may sit from `TILT`: the drag lands on whole
/// pixels, and at the fit zoom one pixel at the handle's radius is under a
/// degree.
const TOLERANCE: f64 = 3.0;

/// The rotate grip tilts a field's box, and Stand upright undoes the tilt.
pub struct AFieldsBoxTiltsByItsRotateGrip;
/// A field's Opacity row writes its `/CA`.
pub struct AFieldsOpacityRowWritesIt;

impl Check for AFieldsBoxTiltsByItsRotateGrip {
    fn name(&self) -> &'static str {
        "a_fields_box_tilts_by_its_rotate_grip"
    }

    fn defect(&self) -> &'static str {
        "a selected form field offers no rotate grip, or dragging it does not tilt the box by the \
         angle dragged, or Stand upright does not bring it back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "widget-turn", tilt)
    }
}

impl Check for AFieldsOpacityRowWritesIt {
    fn name(&self) -> &'static str {
        "a_fields_opacity_row_writes_it"
    }

    fn defect(&self) -> &'static str {
        "a selected form field has no Opacity row, or typing a percentage there does not write \
         the box's /CA"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "widget-opacity", opacity)
    }
}

fn tilt(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = select(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let Some((handle, _)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, HANDLE) else {
        return Ok(Some(format!(
            "the selected field shows no `{HANDLE}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let pivot = mapping.doc_to_window(DocPoint::new(0, CENTRE.0, CENTRE.1))?;
    let from = WindowPoint::centre_of(handle);
    let to = turned(pivot, from, TILT);
    let mark = session.trace()?.mark();
    pointer.drag(session, from, to, 12)?;
    session.settle(40);
    let shot = ctx.out("widget-turn-after.png");
    pointer.screenshot(session, &shot)?;
    report.artifact(shot);

    let trace = session.trace()?;
    let Some(applied) = trace.last_after(TURNED, mark) else {
        return Ok(Some(format!(
            "the rotate grip was dragged {TILT} degrees anticlockwise and no `{TURNED}` \
             followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("tilted: `{}`", applied.raw));
    let now = applied.get("now").and_then(|v| v.parse::<f64>().ok());
    if applied.get("changed") != Some("true")
        || now.is_none_or(|n| (n - f64::from(TILT)).abs() > TOLERANCE)
    {
        return Ok(Some(format!(
            "`{}` must report changed=true and now within {TOLERANCE} of {TILT} (anticlockwise \
             is positive).",
            applied.raw
        )));
    }

    into_view(ctx, session, pointer, UPRIGHT)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, UPRIGHT)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(upright) = trace.last_after(TURNED, mark) else {
        return Ok(Some(format!(
            "Stand upright was pressed and no `{TURNED}` followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("stood up: `{}`", upright.raw));
    Ok((upright.get("now") != Some("0.00"))
        .then(|| format!("`{}` must stand the box upright (now=0.00).", upright.raw)))
}

fn opacity(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = select(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, OPACITY, "40")?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(edited) = trace.last_after(EDITED, mark) else {
        return Ok(Some(format!(
            "40 was typed into `{OPACITY}` and no `{EDITED}` followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("edited: `{}`", edited.raw));
    Ok((edited.get("opacity") != Some("0.4"))
        .then(|| format!("`{}` must write the box's opacity as 0.4.", edited.raw)))
}

/// `from` turned about `pivot` by `degrees` anticlockwise as the operator
/// sees it, in window space where y points down.
fn turned(pivot: WindowPoint, from: WindowPoint, degrees: f32) -> WindowPoint {
    let (s, c) = degrees.to_radians().sin_cos();
    let (vx, vy) = (from.x() - pivot.x(), from.y() - pivot.y());
    let p = Pt {
        x: pivot.x() + vx * c + vy * s,
        y: pivot.y() - vx * s + vy * c,
    };
    WindowPoint::centre_of(LRect { min: p, max: p })
}

/// Click the field and require a `form-field-selected` naming it.
fn select(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, CENTRE)?;
    session.settle(20);
    let trace = session.trace()?;
    let picked = trace.last_after(SELECTED, mark);
    Ok((!picked
        .as_ref()
        .is_some_and(|l| l.raw.contains("field=Name")))
    .then(|| {
        format!(
            "a click on the field traced no `{SELECTED}` naming `Name`. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_handle_above_the_pivot_turns_left_and_up() {
        let at = |x: f32, y: f32| {
            let p = Pt { x, y };
            WindowPoint::centre_of(LRect { min: p, max: p })
        };
        let p = turned(at(0.0, 0.0), at(0.0, -100.0), 30.0);
        assert!(
            (p.x() + 50.0).abs() < 1e-3 && (p.y() + 86.602_54).abs() < 1e-3,
            "{p:?}"
        );
    }
}
