//! `the_3d_viewer_turns_about_the_point_pressed` — a mouse drag on the 3D
//! picture turns the model about the model point it pressed on, as
//! SolidWorks does: the point keeps its place on the picture for the whole
//! drag, and the next drag picks its own point. Left alone, the viewer draws
//! nothing.
//!
//! The fixture's parts are flat and leave gaps, so drags run sideways (a
//! pitch would turn the parts edge-on) and start at a grid of spots; a press
//! on a gap falls back to the point under the centre, or picks none
//! (`at=none`), and only a drag whose pivot showed where it pressed is
//! judged. The spot found is pressed again: the pivot still shows there.
//! Fingers turn about the centre's point instead; this check drives the
//! mouse only.
//!
//! Oracles: no `model-view-rendered` line while the viewer sits untouched;
//! for each judged drag, its first picture shows the pivot where the press
//! was, every later picture until the drag ends has the same `pivot=` and
//! `pivot-shown=`, and the drag turned the view (`yaw=` changed). At least
//! two drags are judged.

use crate::checks::model_axes::{launch_with, opened_from_page};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

/// The viewer's picture: `dialogs::model3d::REGION_IMAGE`.
const IMAGE: &str = "model3d.image";
const PIVOT: &str = "model-orbit-pivot";
const RENDERED: &str = "model-view-rendered";
/// How far a traced pivot may stray, model units.
const TOLERANCE: f64 = 1e-3;
/// How far the pivot may show from the press, picture heights: the trace
/// rounds, and a half-scale picture moves the press by up to a pixel.
const PLACE: f64 = 0.02;
/// How far it may drift during the drag, picture heights.
const DRIFT: f64 = 1e-3;
/// Drags to judge.
const WANTED: usize = 2;

/// See the module documentation.
pub struct TheViewerTurnsAboutThePointPressed;

impl Check for TheViewerTurnsAboutThePointPressed {
    fn name(&self) -> &'static str {
        "the_3d_viewer_turns_about_the_point_pressed"
    }

    fn defect(&self) -> &'static str {
        "Turning a 3D model with the mouse spins it about its bounding box's centre instead of \
         the model point pressed on, lets that point wander during the turn, or the idle viewer \
         keeps redrawing as its size changes untouched"
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
    let (session, pointer) = launch_with(ctx, "model-pivot", &[("PDFCER_DIAG_EMBED", "1")])?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome = opened_from_page(ctx, report, &session, &pointer, ui_rect).and_then(|f| {
        if f.is_some() {
            return Ok(f);
        }
        if let Some(failure) = idle(&session, report) {
            return Ok(Some(failure));
        }
        let Some((rect, vp)) =
            crate::checks::driving::declared_in(&session.trace()?, ui_rect, IMAGE)
        else {
            return Ok(Some(format!(
                "no `{IMAGE}` region: the viewer drew no picture."
            )));
        };
        let mut judged = 0;
        let mut spots: std::collections::VecDeque<(f32, f32)> =
            [-0.1, 0.1, -0.2, 0.2, -0.3, 0.3, -0.4, 0.4]
                .into_iter()
                .flat_map(|fx| [0.05, -0.05, 0.15, -0.15].map(|fy| (fx, fy)))
                .collect();
        while let Some((fx, fy)) = spots.pop_front() {
            if judged == WANTED {
                break;
            }
            let from = at(rect, rect.width() * fx, rect.height() * fy);
            let to = at(rect, rect.width() * fx + 80.0, rect.height() * fy);
            let pivots = count(&session, PIVOT);
            pointer.drag_in(&session, vp.as_deref(), from, to, 30, "l")?;
            session.settle(30);
            let pressed = [f64::from(fx * rect.width() / rect.height()), -f64::from(fy)];
            match held(&session, report, pivots, pressed)? {
                // The pivot still shows at the press, so the same spot is
                // on a part for the next drag.
                Drag::Judged => {
                    judged += 1;
                    spots.push_front((fx, fy));
                }
                Drag::Elsewhere => {}
                Drag::Failed(why) => return Ok(Some(why)),
            }
        }
        if judged < WANTED {
            return Ok(Some(format!(
                "only {judged} of the drags pressed on a part and turned about it."
            )));
        }
        Ok(None)
    });
    pointer.gone(&session)?;
    outcome
}

/// `rect`'s centre moved by `dx`, `dy` points.
fn at(rect: LRect, dx: f32, dy: f32) -> WindowPoint {
    let mut r = rect;
    (r.min.x, r.max.x, r.min.y, r.max.y) = (r.min.x + dx, r.max.x + dx, r.min.y + dy, r.max.y + dy);
    WindowPoint::centre_of(r)
}

/// Left alone, the viewer draws nothing more: a window that resized itself
/// would redraw the model at every new size.
fn idle(session: &Session, report: &mut CheckReport) -> Option<String> {
    session.settle(20);
    let before = count(session, RENDERED);
    session.settle(30);
    let redrawn = count(session, RENDERED) - before;
    report.note(format!("pictures drawn while idle: {redrawn}"));
    (redrawn > 0).then(|| {
        format!("the idle viewer drew {redrawn} pictures: its size or camera changes untouched.")
    })
}

fn count(session: &Session, event: &str) -> usize {
    session
        .trace()
        .map(|t| t.events(event).count())
        .unwrap_or(0)
}

enum Drag {
    /// It turned about the pressed point, held.
    Judged,
    /// It pressed on a gap: no pivot, or the centre's.
    Elsewhere,
    Failed(String),
}

/// The drag after the first `pivots` pivot lines, pressed at `pressed`
/// (right and up of the centre, picture heights).
fn held(
    session: &Session,
    report: &mut CheckReport,
    pivots: usize,
    pressed: [f64; 2],
) -> Result<Drag> {
    let trace = session.trace()?;
    let Some(pivot) = trace.events(PIVOT).nth(pivots) else {
        return Ok(Drag::Failed(
            "a drag on the picture traced no `model-orbit-pivot`.".to_owned(),
        ));
    };
    let Some(at) = pivot.get_f64_list("at") else {
        return Ok(Drag::Elsewhere);
    };
    let after: Vec<&TraceLine> = trace
        .events(RENDERED)
        .filter(|l| l.lineno > pivot.lineno)
        .collect();
    let before = trace
        .events(RENDERED)
        .filter(|l| l.lineno < pivot.lineno)
        .last();
    let yaw = |l: &TraceLine| l.get("yaw").and_then(|v| v.parse::<f64>().ok());
    let (Some(first), Some(last)) = (after.first(), after.last()) else {
        return Ok(Drag::Failed(
            "a drag drew no picture after its pivot.".to_owned(),
        ));
    };
    let shown = first.get_f64_list("pivot-shown").unwrap_or_default();
    report.note(format!(
        "pressed {pressed:?}: pivot {at:?} shown {shown:?}, {} pictures",
        after.len()
    ));
    if shown.len() != 2 || (0..2).any(|i| (shown[i] - pressed[i]).abs() > PLACE) {
        return Ok(Drag::Elsewhere);
    }
    if before.and_then(yaw) == yaw(last) {
        return Ok(Drag::Failed(format!(
            "a drag did not turn the view: yaw {:?}.",
            yaw(last)
        )));
    }
    for line in &after {
        let p = line.get_f64_list("pivot").unwrap_or_default();
        let s = line.get_f64_list("pivot-shown").unwrap_or_default();
        let kept = p.len() == 3 && (0..3).all(|i| (p[i] - at[i]).abs() <= TOLERANCE);
        let still = s.len() == 2 && (0..2).all(|i| (s[i] - shown[i]).abs() <= DRIFT);
        if !kept || !still {
            return Ok(Drag::Failed(format!(
                "during a drag the pivot {at:?} moved to {p:?}, or on the picture from \
                 {shown:?} to {s:?}."
            )));
        }
    }
    Ok(Drag::Judged)
}
