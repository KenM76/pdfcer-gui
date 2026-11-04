//! `the_3d_viewers_drone_sticks_fly_and_turn_the_camera` — walking in the
//! full-screen 3D viewer, drawn inside the application as on a phone: the
//! right stick flies the body without turning it, the camera stick turns the
//! camera without turning the body, the left stick turns the body, the
//! centre-left/right button brings the camera back to the heading, and the
//! Stairs button switches stair climbing. Ghost turned off, the left stick pushed
//! up leaves the eye where it was, and the sticks are photographed showing
//! its sideways track.
//!
//! Each stick is dragged with the mouse, which the viewer takes as one more
//! finger: the drag holds the stick for its frames, then lets it spring back.

use crate::checks::driving::declared_in;
use crate::checks::model_axes::{launch_with, opened_from_page};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const WALK: &str = "model3d.walk";
const IMMERSIVE: &str = "model3d.immersive";
const STICK: &str = "model3d.drone.stick.";
const BUTTON: &str = "model3d.drone.button.";
/// `dialogs::model3d::nav::REGION_GHOST`.
const GHOST: &str = "model3d.ghost";
/// The sticks and buttons, as `nav.rs` numbers them.
const LEFT: u8 = 0;
const RIGHT: u8 = 1;
const CAMERA: u8 = 2;
const CENTRE_YAW: u8 = 0;
const STAIRS: u8 = 2;
/// The smallest traced angle change that counts as a turn, radians.
const TURNED: f64 = 0.005;

/// See the module documentation.
pub struct TheDroneSticksFlyAndTurnTheCamera;

impl Check for TheDroneSticksFlyAndTurnTheCamera {
    fn name(&self) -> &'static str {
        "the_3d_viewers_drone_sticks_fly_and_turn_the_camera"
    }

    fn defect(&self) -> &'static str {
        "The full-screen 3D walk's drone sticks do not do what a DJI remote's do: the right \
         stick does not fly, the camera stick turns the body, the left stick does not turn, or \
         the centre and Stairs buttons do nothing"
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
    let (session, pointer) = launch_with(ctx, "model-drone", &[("PDFCER_DIAG_EMBED", "1")])?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let pilot = Pilot {
        session: &session,
        pointer: &pointer,
        ui_rect,
        shot: Some(ctx.out("model-drone-solid.png")),
    };
    let outcome =
        opened_from_page(ctx, report, &session, &pointer, ui_rect).and_then(|f| match f {
            Some(failure) => Ok(Some(failure)),
            None => pilot.run(report),
        });
    pointer.gone(&session)?;
    outcome
}

struct Pilot<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    ui_rect: &'a str,
    /// Where the solid-walking picture is written.
    shot: Option<std::path::PathBuf>,
}

/// One `model-walk-moved` line's pose.
#[derive(Debug, Clone, PartialEq)]
struct Pose {
    eye: String,
    yaw: f64,
    look_yaw: f64,
}

impl Pilot<'_> {
    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        if !self.press(WALK)? || !self.press(IMMERSIVE)? {
            return Ok(Some(format!(
                "no `{WALK}` or `{IMMERSIVE}` switch in the embedded viewer."
            )));
        }
        if self.region(&format!("{STICK}{RIGHT}"))?.is_none() {
            return Ok(Some(format!(
                "no `{STICK}{RIGHT}` region: the full-screen walk drew no drone sticks."
            )));
        }
        // A button pressed holds a finger for a frame, so it traces the pose
        // the sticks start from.
        self.press(&format!("{BUTTON}{CENTRE_YAW}"))?;
        let flown = self.stick(RIGHT, [0.0, -0.8])?;
        report.note(format!("right stick up: {flown:?}"));
        let Some((before, after)) = flown else {
            return Ok(Some(
                "the right stick traced no `model-walk-moved`.".to_owned(),
            ));
        };
        if after.eye == before.eye || (after.yaw - before.yaw).abs() > TURNED {
            return Ok(Some(format!(
                "the right stick pushed up should fly without turning: {before:?} then {after:?}."
            )));
        }
        let looked = self.stick(CAMERA, [0.8, 0.0])?;
        report.note(format!("camera stick right: {looked:?}"));
        match looked {
            Some((b, a))
                if (a.look_yaw - b.look_yaw).abs() > TURNED && (a.yaw - b.yaw).abs() < TURNED => {}
            other => {
                return Ok(Some(format!(
                    "the camera stick should turn the camera alone: {other:?}."
                )));
            }
        }
        let turned = self.stick(LEFT, [0.8, 0.0])?;
        report.note(format!("left stick right: {turned:?}"));
        match turned {
            Some((b, a)) if (a.yaw - b.yaw).abs() > TURNED => {}
            other => {
                return Ok(Some(format!(
                    "the left stick should turn the body: {other:?}."
                )));
            }
        }
        if let Some(failure) = self.solid(report)? {
            return Ok(Some(failure));
        }
        self.buttons(report)
    }

    /// Ghost off, the left stick pushed up must not lift the eye.
    fn solid(&self, report: &mut CheckReport) -> Result<Option<String>> {
        // Walking entered from orbit starts with ghost off and traces no
        // preference until one changes: on, then off again, proves the state.
        if !self.press(GHOST)? || !self.press(GHOST)? {
            return Ok(Some(format!(
                "no `{GHOST}` switch in the full-screen walk."
            )));
        }
        let ghost = self.last("model-nav-prefs", "ghost");
        if ghost.as_deref() != Some("false") {
            return Ok(Some(format!("Ghost did not turn off: ghost={ghost:?}.")));
        }
        let before = self.pose();
        self.stick(LEFT, [0.0, -0.8])?;
        let after = self.pose();
        report.note(format!(
            "left stick up, ghost off: eye {:?} -> {:?}",
            before.as_ref().map(|p| &p.eye),
            after.as_ref().map(|p| &p.eye)
        ));
        if let Some(png) = &self.shot {
            self.pointer.screenshot(self.session, png)?;
            report.artifact(png.clone());
        }
        let rose = matches!((&before, &after), (Some(b), Some(a)) if a.eye != b.eye);
        Ok(rose.then(|| {
            format!(
                "with ghost off, the left stick pushed up moved the eye: {before:?} then {after:?}."
            )
        }))
    }

    /// Centre the camera; switch Stairs.
    fn buttons(&self, report: &mut CheckReport) -> Result<Option<String>> {
        self.press(&format!("{BUTTON}{CENTRE_YAW}"))?;
        let centred = self.last("model-drone-button", "look-yaw");
        report.note(format!("centred look-yaw {centred:?}"));
        if centred.as_deref().and_then(|v| v.parse::<f64>().ok()) != Some(0.0) {
            return Ok(Some(format!(
                "the centre-left/right button left the camera at look-yaw {centred:?}."
            )));
        }
        self.press(&format!("{BUTTON}{STAIRS}"))?;
        let stairs = self.last("model-nav-prefs", "stairs");
        report.note(format!("stairs {stairs:?}"));
        if stairs.as_deref() != Some("true") {
            return Ok(Some(format!(
                "the Stairs button traced stairs={stairs:?}, not true."
            )));
        }
        Ok(None)
    }

    /// Drag stick `n` from its centre by `tilt` of its radius, held for the
    /// drag's frames: the poses before and after, from the last
    /// `model-walk-moved` lines.
    fn stick(&self, n: u8, tilt: [f32; 2]) -> Result<Option<(Pose, Pose)>> {
        let Some((base, vp)) = self.region(&format!("{STICK}{n}"))? else {
            return Ok(None);
        };
        let Some(before) = self.pose() else {
            return Ok(None);
        };
        let from = WindowPoint::centre_of(base);
        let r = base.width() / 2.0;
        let (dx, dy) = (tilt[0] * r, tilt[1] * r);
        let mut aim = base;
        (aim.min.x, aim.max.x, aim.min.y, aim.max.y) = (
            aim.min.x + dx,
            aim.max.x + dx,
            aim.min.y + dy,
            aim.max.y + dy,
        );
        let to = WindowPoint::centre_of(aim);
        let lines = self.moved_count();
        self.pointer
            .drag_in(self.session, vp.as_deref(), from, to, 40, "l")?;
        self.session.settle(30);
        if self.moved_count() == lines {
            return Ok(None);
        }
        Ok(self.pose().map(|after| (before, after)))
    }

    fn moved_count(&self) -> usize {
        self.session
            .trace()
            .map(|t| t.events("model-walk-moved").count())
            .unwrap_or(0)
    }

    fn pose(&self) -> Option<Pose> {
        let trace = self.session.trace().ok()?;
        let line = trace.events("model-walk-moved").last()?;
        Some(Pose {
            eye: line.get("eye")?.to_owned(),
            yaw: line.get("yaw")?.parse().ok()?,
            look_yaw: line.get("look-yaw")?.parse().ok()?,
        })
    }

    fn region(&self, name: &str) -> Result<Option<(crate::geom::LRect, Option<String>)>> {
        Ok(declared_in(&self.session.trace()?, self.ui_rect, name))
    }

    /// Click `name`'s centre; `false` when it is not drawn.
    fn press(&self, name: &str) -> Result<bool> {
        crate::checks::viewer_options::press(self.session, self.pointer, self.ui_rect, name)
    }

    /// `key` on the last `event` line.
    fn last(&self, event: &str, key: &str) -> Option<String> {
        let trace = self.session.trace().ok()?;
        let line = trace.events(event).last()?;
        line.get(key).map(str::to_owned)
    }
}
