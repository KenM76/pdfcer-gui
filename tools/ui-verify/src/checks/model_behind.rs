//! `the_3d_viewer_walks_seen_from_behind` — while walking, the Behind switch
//! draws the picture from behind and above the walker's eye, looking the
//! walker's way, and stands a figure in the picture where the walker is;
//! off again, the picture is drawn from the eye and the figure is gone.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/model_behind.md`.

use crate::checks::driving::declared_in;
use crate::checks::model_axes::{launch, opened_from_page};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const IMAGE: &str = "model3d.image";
const WALK: &str = "model3d.walk";
/// `dialogs::model3d::nav::REGION_BEHIND`.
const BEHIND: &str = "model3d.behind";
/// `dialogs::model3d::behind::REGION_FIGURE`.
const FIGURE: &str = "model3d.figure";
const RENDERED: &str = "model-view-rendered"; // ui-text-exempt: a trace event name
const DRAWN: &str = "model-figure"; // ui-text-exempt: a trace event name
/// The nearest the camera may hang behind the eye, eye heights:
/// `behind::NEAREST` less rounding.
const NEAREST: f64 = 0.3;

/// See the module documentation.
pub struct TheViewerWalksSeenFromBehind;

impl Check for TheViewerWalksSeenFromBehind {
    fn name(&self) -> &'static str {
        "the_3d_viewer_walks_seen_from_behind"
    }

    fn defect(&self) -> &'static str {
        "The 3D viewer's Behind switch is missing while walking, does not move the picture \
         behind and above the walker looking its way, draws no figure where the walker \
         stands, or leaves the camera or the figure behind when switched off"
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

/// Where the last picture was drawn from, relative to the walker.
#[derive(Debug)]
struct View {
    /// The camera less the walker's eye.
    offset: [f64; 3],
    /// The unit look direction.
    dir: [f64; 3],
    /// The camera's unit up.
    up: [f64; 3],
    /// The walker's eye height.
    body: f64,
}

impl View {
    fn back(&self) -> f64 {
        -dot(self.offset, self.dir) / self.body
    }

    fn above(&self) -> f64 {
        dot(self.offset, self.up) / self.body
    }

    fn apart(&self) -> f64 {
        dot(self.offset, self.offset).sqrt() / self.body
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn triple(text: &str) -> Option<[f64; 3]> {
    let mut it = text.split(',').map(|c| c.parse::<f64>().ok());
    Some([it.next()??, it.next()??, it.next()??])
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let (session, pointer) = launch(ctx, "model-behind")?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome = match opened_from_page(ctx, report, &session, &pointer, ui_rect)? {
        Some(failure) => Ok(Some(failure)),
        None => Stage {
            session: &session,
            pointer: &pointer,
            ui_rect,
        }
        .run(report),
    };
    pointer.gone(&session)?;
    outcome
}

struct Stage<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    ui_rect: &'static str,
}

impl Stage<'_> {
    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        if !self.press(WALK)? || self.last("model-walk", "on").as_deref() != Some("true") {
            return Ok(Some("Walk traced no `model-walk on=true`.".to_owned()));
        }
        let eye = self.view()?;
        report.note(format!("walking: {eye:?}"));
        if eye.apart() > 1e-3 {
            return Ok(Some(format!(
                "walking with Behind off draws from {:.2} eye heights off the walker's eye; \
                 expected the eye itself.",
                eye.apart()
            )));
        }
        let drawn_before = self.figures();
        if !self.press(BEHIND)? {
            return Ok(Some(format!("no `{BEHIND}` switch while walking.")));
        }
        self.session.settle(30);
        if self.last("model-nav-prefs", "behind").as_deref() != Some("true") {
            return Ok(Some(
                "Behind traced no `model-nav-prefs behind=true`.".to_owned(),
            ));
        }
        if let Some(failure) = self.seen_from_behind(report, drawn_before)? {
            return Ok(Some(failure));
        }
        self.press(BEHIND)?;
        self.session.settle(30);
        let off = self.view()?;
        let drawn_after = self.figures();
        report.note(format!("Behind off: {off:?}, figures drawn {drawn_after}"));
        if off.apart() > 1e-3 {
            return Ok(Some(format!(
                "with Behind off again the picture is drawn {:.2} eye heights off the eye.",
                off.apart()
            )));
        }
        self.session.settle(10);
        if self.figures() != drawn_after {
            return Ok(Some(
                "with Behind off the figure is still being drawn.".to_owned(),
            ));
        }
        Ok(None)
    }

    /// The camera behind and above, looking the walker's way, and the
    /// figure drawn inside the picture, low and across its middle.
    fn seen_from_behind(&self, report: &mut CheckReport, before: usize) -> Result<Option<String>> {
        let on = self.view()?;
        let arm = self.last("model-behind-arm", "eye-heights");
        report.note(format!(
            "Behind on: back {:.2}, above {:.2} eye heights, arm {arm:?}",
            on.back(),
            on.above()
        ));
        if on.back() < NEAREST || on.above() <= 0.0 {
            return Ok(Some(format!(
                "with Behind on the picture is drawn {:.2} eye heights behind the eye and {:.2} \
                 above it; expected behind by at least {NEAREST} and above.",
                on.back(),
                on.above()
            )));
        }
        let trace = self.session.trace()?;
        let triangles = self.last(DRAWN, "triangles");
        let figure = declared_in(&trace, self.ui_rect, FIGURE).map(|(r, _)| r);
        let image = declared_in(&trace, self.ui_rect, IMAGE).map(|(r, _)| r);
        report.note(format!(
            "figure {figure:?} in picture {image:?}, {triangles:?} triangles, {} drawn before",
            before
        ));
        let (Some(figure), Some(image)) = (figure, image) else {
            return Ok(Some(format!(
                "with Behind on no figure is drawn: `{FIGURE}` {figure:?}, `{IMAGE}` {image:?}."
            )));
        };
        if before != 0 || !placed(figure, image) {
            return Ok(Some(format!(
                "the figure {figure:?} is not standing across the middle of the lower picture \
                 {image:?}, or one was drawn before Behind ({before})."
            )));
        }
        Ok(None)
    }

    /// The last picture's camera, from `model-view-rendered`.
    fn view(&self) -> Result<View> {
        let trace = self.session.trace()?;
        let line = trace
            .events(RENDERED)
            .last()
            .ok_or_else(|| Error::new("no `model-view-rendered` line."))?;
        let field = |k: &str| {
            line.get(k)
                .and_then(triple)
                .ok_or_else(|| Error::new(format!("`{RENDERED}` has no `{k}`: `{}`", line.raw)))
        };
        let (eye, from) = (field("eye")?, field("from")?);
        let body = line
            .get("body")
            .and_then(|b| b.parse::<f64>().ok())
            .filter(|b| *b > 0.0)
            .ok_or_else(|| Error::new(format!("`{RENDERED}` has no `body`: `{}`", line.raw)))?;
        Ok(View {
            offset: std::array::from_fn(|i| from[i] - eye[i]),
            dir: field("dir")?,
            up: field("up")?,
            body,
        })
    }

    /// How many `model-figure` lines the trace holds.
    fn figures(&self) -> usize {
        self.session.trace().map_or(0, |t| t.events(DRAWN).count())
    }

    fn press(&self, name: &str) -> Result<bool> {
        crate::checks::viewer_options::press(self.session, self.pointer, self.ui_rect, name)
    }

    fn last(&self, event: &str, key: &str) -> Option<String> {
        let trace = self.session.trace().ok()?;
        let line = trace.events(event).last()?;
        line.get(key).map(str::to_owned)
    }
}

/// Whether `figure` lies inside `image`, its centre within a sixth of the
/// width of the middle and its top below a third of the way down.
fn placed(figure: LRect, image: LRect) -> bool {
    let inside = figure.min.x >= image.min.x - 1.0
        && figure.max.x <= image.max.x + 1.0
        && figure.max.y <= image.max.y + 1.0;
    let middle = ((figure.min.x + figure.max.x) / 2.0 - (image.min.x + image.max.x) / 2.0).abs();
    inside
        && middle <= (image.max.x - image.min.x) / 6.0
        && figure.min.y >= image.min.y + (image.max.y - image.min.y) / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Pt;

    fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> LRect {
        LRect {
            min: Pt { x: x0, y: y0 },
            max: Pt { x: x1, y: y1 },
        }
    }

    #[test]
    fn a_figure_low_in_the_middle_is_placed_and_one_off_to_the_side_is_not() {
        let image = rect(0.0, 0.0, 600.0, 400.0);
        assert!(placed(rect(280.0, 210.0, 320.0, 390.0), image));
        assert!(!placed(rect(20.0, 210.0, 60.0, 390.0), image));
        assert!(!placed(rect(280.0, 20.0, 320.0, 390.0), image));
        assert!(!placed(rect(280.0, 210.0, 320.0, 450.0), image));
    }

    #[test]
    fn a_camera_behind_and_above_reads_so() {
        let v = View {
            offset: [-3.0, 0.0, 0.25],
            dir: [1.0, 0.0, 0.0],
            up: [0.0, 0.0, 1.0],
            body: 1.0,
        };
        assert!((v.back() - 3.0).abs() < 1e-9 && v.above() > 0.0);
        assert_eq!(triple("1,2.5,-3"), Some([1.0, 2.5, -3.0]));
        assert_eq!(triple("1,2"), None);
    }
}
