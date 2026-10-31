//! `the_3d_viewer_walks_and_its_gimbal_turns_the_view` — in the 3D viewer, a
//! tap on the gimbal's +X end views the model from +X, Walk starts a walk
//! from the orbit's eye, the wheel walks it forward, a drag turns its look,
//! and Ghost and Orbit switch as they say.

use crate::checks::driving::declared_in;
use crate::checks::model_axes::{launch, opened_from_page};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const IMAGE: &str = "model3d.image";
/// The gimbal's +X end.
const TIP_X: &str = "model3d.gimbal.tip.0";
const WALK: &str = "model3d.walk";
const GHOST: &str = "model3d.ghost";
/// How close a traced unit direction must come to the expected one.
const TOLERANCE: f64 = 0.02;

/// See the module documentation.
pub struct TheViewerWalksAndItsGimbalTurnsTheView;

impl Check for TheViewerWalksAndItsGimbalTurnsTheView {
    fn name(&self) -> &'static str {
        "the_3d_viewer_walks_and_its_gimbal_turns_the_view"
    }

    fn defect(&self) -> &'static str {
        "The 3D viewer's gimbal does not turn the view to the axis tapped, or Walk does not walk: \
         the wheel does not move the eye, a drag does not turn the look, or Ghost and Orbit do \
         not switch"
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
    let (session, pointer) = launch(ctx, "model-walk")?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let walker = Walker {
        session: &session,
        pointer: &pointer,
        ui_rect,
    };
    let outcome =
        opened_from_page(ctx, report, &session, &pointer, ui_rect).and_then(|f| match f {
            Some(failure) => Ok(Some(failure)),
            None => walker.run(report),
        });
    pointer.gone(&session)?;
    outcome
}

struct Walker<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    ui_rect: &'a str,
}

impl Walker<'_> {
    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        if let Some(failure) = self.gimbal(report)? {
            return Ok(Some(failure));
        }
        if !self.press(WALK)? {
            return Ok(Some(format!("no `{WALK}` switch in the viewer.")));
        }
        if self.last("model-walk", "on").as_deref() != Some("true") {
            return Ok(Some("Walk traced no `model-walk on=true`.".to_owned()));
        }
        let start = self.last("model-view-rendered", "eye");
        report.note(format!("walk starts at eye {start:?}"));
        if start.is_none() {
            return Ok(Some(
                "the picture drawn while walking traced no eye.".to_owned(),
            ));
        }
        let Some((image, vp)) = self.region(IMAGE)? else {
            return Ok(Some(format!("no `{IMAGE}` region.")));
        };
        let centre = WindowPoint::centre_of(image);
        self.pointer
            .wheel_in(self.session, vp.as_deref(), centre, 3.0)?;
        self.session.settle(40);
        let walked = self.last("model-walk-moved", "eye");
        let drawn = self.last("model-view-rendered", "eye");
        report.note(format!(
            "after the wheel: moved {walked:?}, drawn {drawn:?}"
        ));
        if walked.is_none() || drawn == start {
            return Ok(Some(format!(
                "the wheel over the picture did not walk: `model-walk-moved` eye {walked:?}, \
                 picture eye {drawn:?} from {start:?}."
            )));
        }
        self.turned(report, image, vp.as_deref())
    }

    /// A drag across the picture turns the look; Ghost and Orbit switch.
    fn turned(
        &self,
        report: &mut CheckReport,
        image: crate::geom::LRect,
        vp: Option<&str>,
    ) -> Result<Option<String>> {
        let yaw = self.last("model-walk-moved", "yaw");
        let centre = WindowPoint::centre_of(image);
        let mut right = image;
        right.min.x += image.width() * 0.4;
        let to = WindowPoint::centre_of(right);
        self.pointer.drag_in(self.session, vp, centre, to, 8, "l")?;
        self.session.settle(30);
        let turned = self.last("model-walk-moved", "yaw");
        report.note(format!("yaw {yaw:?} -> {turned:?}"));
        if turned.is_none() || turned == yaw {
            return Ok(Some(format!(
                "a drag across the picture did not turn the walker: yaw {yaw:?} then {turned:?}."
            )));
        }
        if !self.press(GHOST)? {
            return Ok(Some(format!("no `{GHOST}` switch while walking.")));
        }
        if self.last("model-nav-prefs", "ghost").as_deref() != Some("true") {
            return Ok(Some(
                "Ghost traced no `model-nav-prefs ghost=true`.".to_owned(),
            ));
        }
        self.press(GHOST)?;
        if !self.press(WALK)? || self.last("model-walk", "on").as_deref() != Some("false") {
            return Ok(Some(
                "Orbit did not end the walk (`model-walk on=false`).".to_owned(),
            ));
        }
        Ok(None)
    }

    /// The gimbal's +X end views the model from +X: it looks along -X.
    fn gimbal(&self, report: &mut CheckReport) -> Result<Option<String>> {
        if !self.press(TIP_X)? {
            return Ok(Some(format!(
                "no `{TIP_X}` region: the gimbal is not drawn."
            )));
        }
        let tapped = self
            .session
            .trace()?
            .events("model-gimbal")
            .last()
            .map(|l| l.raw.clone());
        let dir = self.last("model-view-rendered", "dir");
        report.note(format!("gimbal: {tapped:?}, dir {dir:?}"));
        let parsed: Vec<f64> = dir
            .as_deref()
            .map(|d| d.split(',').filter_map(|c| c.parse().ok()).collect())
            .unwrap_or_default();
        let along = parsed.len() == 3
            && parsed
                .iter()
                .zip([-1.0, 0.0, 0.0])
                .all(|(a, b)| (a - b).abs() < TOLERANCE);
        if tapped.is_none() || !along {
            return Ok(Some(format!(
                "a tap on the gimbal's +X end traced {tapped:?} and looks along {dir:?}; it \
                 should look along -X."
            )));
        }
        Ok(None)
    }

    fn region(&self, name: &str) -> Result<Option<(crate::geom::LRect, Option<String>)>> {
        Ok(declared_in(&self.session.trace()?, self.ui_rect, name))
    }

    /// Click `name`'s centre; `false` when it is not drawn.
    fn press(&self, name: &str) -> Result<bool> {
        let Some((rect, vp)) = self.region(name)? else {
            return Ok(false);
        };
        self.pointer
            .click_in(self.session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        self.session.settle(20);
        Ok(true)
    }

    /// `key` on the last `event` line.
    fn last(&self, event: &str, key: &str) -> Option<String> {
        let trace = self.session.trace().ok()?;
        let line = trace.events(event).last()?;
        line.get(key).map(str::to_owned)
    }
}
