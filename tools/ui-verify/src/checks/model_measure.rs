//! `the_3d_viewer_measures_from_points_on_the_model` — points picked on the
//! model make a measurement the engine values, shown as it is measured, and
//! one kept is listed and drawn when the viewer next opens the file.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/model_measure.md`.

use crate::checks::driving::count;
use crate::checks::model_views_saved::{fixture, launch, press_all};
use crate::checks::viewer_options::{close, reach};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const VIEW: &str = "models.view";
const CLOSE: &str = "model3d.close";
const IMAGE: &str = "model3d.image";
const GLYPH: &str = "model3d.measure";
const KEEP: &str = "model3d.measure.keep";
const DELETE_FIRST: &str = "model3d.measure.delete.0";
/// How far a value may stray from the picks' distance: the trace's rounding.
const TOLERANCE: f64 = 1e-3;

/// See the module documentation.
pub struct TheViewerMeasuresFromPointsOnTheModel;

impl Check for TheViewerMeasuresFromPointsOnTheModel {
    fn name(&self) -> &'static str {
        "the_3d_viewer_measures_from_points_on_the_model"
    }

    fn defect(&self) -> &'static str {
        "The 3D viewer's measure tool gives a value other than the distance between the points \
         picked, shows nothing before the last pick, keeps only one measurement from a window, \
         or a kept measurement is not there when the model is opened again"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = fixture(ctx, "model-measure")?;
    let (session, pointer) = launch(ctx, "model-measure", pdf, &[("PDFCER_DIAG_EMBED", "1")])?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let failure = steps(ctx, &session, &pointer, ui_rect, report);
    let parked = pointer.gone(&session);
    match failure? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// The spots tried, off the picture's centre: the fixture's parts are flat and
/// leave gaps.
fn spots(rect: LRect) -> Vec<WindowPoint> {
    let mut found = Vec::new();
    for fx in [-0.1, 0.1, -0.2, 0.2, -0.3, 0.3, -0.05, 0.05, 0.0] {
        for fy in [0.0, 0.05, -0.05, 0.15, -0.15, 0.25, -0.25] {
            let (dx, dy) = (rect.width() * fx, rect.height() * fy);
            let mut r = rect;
            (r.min.x, r.max.x, r.min.y, r.max.y) =
                (r.min.x + dx, r.max.x + dx, r.min.y + dy, r.max.y + dy);
            found.push(WindowPoint::centre_of(r));
        }
    }
    found
}

/// Hover and click spots until a measurement is finished; whether its value
/// showed under the pointer before the last click. `None` when none finished.
fn measure_once(
    session: &Session,
    pointer: &ScriptedPointer,
    at: &[WindowPoint],
) -> Result<Option<bool>> {
    let results = count(session, "model-measure-result")?;
    for spot in at {
        let previews = count(session, "model-measure-preview")?;
        pointer.hover(session, *spot)?;
        session.settle(10);
        let shown = count(session, "model-measure-preview")? > previews;
        pointer.click(session, *spot)?;
        session.settle(20);
        if count(session, "model-measure-result")? > results {
            return Ok(Some(shown));
        }
    }
    Ok(None)
}

/// The newest result's value and the distance between its two picks.
fn newest(session: &Session) -> Result<Option<(f64, f64)>> {
    let trace = session.trace()?;
    let Some(value) = trace
        .last("model-measure-result")
        .and_then(|l| l.get("value"))
        .and_then(|v| v.parse::<f64>().ok())
    else {
        return Ok(None);
    };
    let points: Vec<Vec<f64>> = trace
        .events("model-measure-pick")
        .filter_map(|l| l.get("at"))
        .map(|a| a.split(',').filter_map(|c| c.parse().ok()).collect())
        .collect();
    let [.., a, b] = points.as_slice() else {
        return Ok(None);
    };
    let span: f64 = a.iter().zip(b).map(|(p, q)| (p - q) * (p - q)).sum();
    Ok(Some((value, span.sqrt())))
}

/// Press Keep; the sentence unless the view then lists `want` measurements.
fn keep(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    want: usize,
) -> Result<Option<String>> {
    if let Some(missing) = press_all(session, pointer, ui_rect, &[KEEP])? {
        return Ok(Some(missing));
    }
    session.settle(20);
    let trace = session.trace()?;
    let saved = trace
        .last("model-measure-saved")
        .and_then(|l| l.get_usize("count"));
    if saved != Some(want) {
        return Ok(Some(format!(
            "Keep left {saved:?} measurements on the view, wanted {want}; declined {:?}.",
            trace.last("model-views-declined").map(|l| l.raw.clone())
        )));
    }
    Ok(None)
}

fn steps(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    if let Some(missing) = press_all(session, pointer, ui_rect, &[VIEW, GLYPH])? {
        return Ok(Some(missing));
    }
    let trace = session.trace()?;
    let Some((rect, _)) = crate::checks::driving::declared_in(&trace, ui_rect, IMAGE) else {
        return Ok(Some(format!("no `{IMAGE}` region: no picture was drawn.")));
    };
    let at = spots(rect);
    let Some(previewed) = measure_once(session, pointer, &at)? else {
        return Ok(Some(format!(
            "no measurement after picking across the picture: {} picks, {} misses.",
            count(session, "model-measure-pick")?,
            count(session, "model-measure-missed")?
        )));
    };
    if !previewed {
        return Ok(Some(
            "no value showed with the pointer over the model before the last pick.".to_owned(),
        ));
    }
    let Some((value, span)) = newest(session)? else {
        return Ok(Some("the result traced no value or picks.".to_owned()));
    };
    report.note(format!("measured {value}; the picks are {span} apart"));
    if (value - span).abs() > TOLERANCE {
        return Ok(Some(format!(
            "the value {value} is not the distance between the points picked, {span}."
        )));
    }
    pointer.screenshot(session, &ctx.out("model-measure.measured.png"))?;
    if let Some(failure) = keep(session, pointer, ui_rect, 1)? {
        return Ok(Some(failure));
    }
    if measure_once(session, pointer, &at)?.is_none() {
        return Ok(Some(
            "no second measurement after the first was kept.".to_owned(),
        ));
    }
    if let Some(failure) = keep(session, pointer, ui_rect, 2)? {
        return Ok(Some(format!("the second keep: {failure}")));
    }
    if let Some(failure) = other_kinds(ctx, session, pointer, ui_rect, &at)? {
        return Ok(Some(failure));
    }
    reopened(ctx, session, pointer, ui_rect, report)
}

/// The fixture lies flat, so a perpendicular between two of its points has no
/// length and the engine refuses it; an angle and a radius finish.
fn other_kinds(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    at: &[WindowPoint],
) -> Result<Option<String>> {
    for (kind, name) in [(1, "perpendicular"), (2, "angle"), (3, "radius")] {
        let region = format!("model3d.measure.kind.{kind}");
        if let Some(missing) = press_all(session, pointer, ui_rect, &[region.as_str()])? {
            return Ok(Some(missing));
        }
        close(session, pointer, ui_rect)?;
        let refused = count(session, "model-measure-refused")?;
        let finished = measure_once(session, pointer, at)?.is_some();
        let said_no = count(session, "model-measure-refused")? > refused;
        pointer.screenshot(session, &ctx.out(&format!("model-measure.{name}.png")))?;
        if kind == 1 && (finished || !said_no) {
            return Ok(Some(format!(
                "a perpendicular on a flat model: finished {finished}, refused {said_no}."
            )));
        }
        if kind != 1 && !finished {
            return Ok(Some(format!(
                "no {name} measurement from points on the model."
            )));
        }
    }
    Ok(None)
}

/// Close and open again: both are drawn, and one can be deleted.
fn reopened(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    if let Some(missing) = press_all(session, pointer, ui_rect, &[CLOSE, VIEW])? {
        return Ok(Some(missing));
    }
    session.settle(30);
    let trace = session.trace()?;
    let drawn = trace.last("model-measure-drawn").map(|l| l.raw.clone());
    report.note(format!("reopened: {drawn:?}"));
    let count_drawn = trace
        .last("model-measure-drawn")
        .and_then(|l| l.get_usize("count"));
    if count_drawn != Some(2) {
        return Ok(Some(format!(
            "the two measurements kept are not drawn on opening the model again: {drawn:?}."
        )));
    }
    pointer.screenshot(session, &ctx.out("model-measure.reopened.png"))?;
    let Some((rect, vp)) = reach(session, pointer, ui_rect, DELETE_FIRST)? else {
        return Ok(Some(format!("no `{DELETE_FIRST}` region in the menu.")));
    };
    pointer.screenshot(session, &ctx.out("model-measure.tab.png"))?;
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    let left = session
        .trace()?
        .last("model-measure-saved")
        .and_then(|l| l.get_usize("count"));
    if left != Some(1) {
        return Ok(Some(format!(
            "Delete left {left:?} measurements, wanted 1."
        )));
    }
    close(session, pointer, ui_rect)?;
    Ok(None)
}
