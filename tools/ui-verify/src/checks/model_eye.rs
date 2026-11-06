//! `the_3d_viewer_keeps_its_eye_height_and_saves_a_scene_on_a_double_tap` —
//! walking with ghost mode off, a drag on the feet field sets the walker's
//! eye height (O356) in that viewer only, written to the preferences file
//! by *Save as default* (O398) and back in a second launch; and two quick taps on the gimbal's centre save
//! a picture of the view as it was before the taps (O357), both facing
//! along X, whose tip is then drawn over the centre (O358), and from a
//! tilted view.

use std::path::Path;

use crate::checks::driving::declared_in;
use crate::checks::model_axes::{launch_with, opened_from_page};
use crate::checks::model_view_window::picture_saved;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const IMAGE: &str = "model3d.image";
const WALK: &str = "model3d.walk";
const GHOST: &str = "model3d.ghost";
/// `dialogs::model3d::eye::REGION_EYE_HEIGHT`, in the options menu: the
/// feet field, then inches.
const EYE: &str = "model3d.eye-height";
/// The gimbal's centre: the zone after its six ends.
const CENTRE: &str = "model3d.gimbal.tip.6";
/// The gimbal's +X tip.
const TIP_X: &str = "model3d.gimbal.tip.0";
/// `dialogs::model3d::options::REGION_SAVE_DEFAULTS`.
const SAVE_DEFAULTS: &str = "model3d.options.save_default";
/// `prefs::walkeye`'s file key.
const KEY: &str = "walk_eye_height_mm";
/// The picture picker's diagnostic answer, read by `app::files::pick_picture_target`.
const PICTURE_ENV: &str = "PDFCER_DIAG_PICTURE_SAVE_PATH";

/// See the module documentation.
pub struct TheViewerKeepsItsEyeHeight;

impl Check for TheViewerKeepsItsEyeHeight {
    fn name(&self) -> &'static str {
        "the_3d_viewer_keeps_its_eye_height_and_saves_a_scene_on_a_double_tap"
    }

    fn defect(&self) -> &'static str {
        "The 3D walker's feet-and-inches eye height is missing, does not change the height, is \
         written to the preferences file before Save as default or not by it, or is \
         forgotten on the next launch; or two taps \
         on the gimbal's centre save no picture, or change the view"
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
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive."))?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, "")
        .map_err(|e| Error::new(format!("could not plant the preferences file: {e}")))?;
    let target = ctx.out("model-eye-scene.png");
    let _ = std::fs::remove_file(&target);
    let path = target.display().to_string();
    let one = run_stage(
        ctx,
        report,
        ui_rect,
        "model-eye.1",
        &[(PICTURE_ENV, &path)],
        |s, r| s.first(r, &userdata, &target),
    )?;
    let Ok(mm) = one else {
        return Ok(one.err());
    };
    run_stage(ctx, report, ui_rect, "model-eye.2", &[], |s, r| {
        s.second(r, mm)
    })
    .map(|outcome| outcome.err())
}

/// Launch, open the model from its page, start walking, and run `body`.
fn run_stage<T>(
    ctx: &CheckContext,
    report: &mut CheckReport,
    ui_rect: &'static str,
    stem: &str,
    extra: &[(&str, &str)],
    body: impl FnOnce(&Stage, &mut CheckReport) -> Result<std::result::Result<T, String>>,
) -> Result<std::result::Result<T, String>> {
    let (session, pointer) = launch_with(ctx, stem, extra)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let stage = Stage {
        session: &session,
        pointer: &pointer,
        ui_rect,
    };
    let outcome = match opened_from_page(ctx, report, &session, &pointer, ui_rect) {
        Ok(Some(failure)) => Ok(Err(failure)),
        Ok(None) => stage.walking().and_then(|walking| match walking {
            Some(failure) => Ok(Err(failure)),
            None => body(&stage, report),
        }),
        Err(why) => Err(why),
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
    /// Start a walk; `Some` names why it did not.
    fn walking(&self) -> Result<Option<String>> {
        if !self.press(WALK)? || self.last("model-walk", "on").as_deref() != Some("true") {
            return Ok(Some("Walk traced no `model-walk on=true`.".to_owned()));
        }
        Ok(None)
    }

    /// Set the height by dragging the feet field and typing inches; the
    /// viewer must trace it and write nothing until *Save as default*,
    /// which must put it on disk. Then the double tap. `Ok` carries the
    /// millimetres written.
    fn first(
        &self,
        report: &mut CheckReport,
        userdata: &Path,
        target: &Path,
    ) -> Result<std::result::Result<u32, String>> {
        let reached =
            crate::checks::viewer_options::reach(self.session, self.pointer, self.ui_rect, EYE)?;
        let Some((fields, vp)) = reached else {
            return Ok(Err(format!(
                "no `{EYE}` region while walking with ghost mode off."
            )));
        };
        let mut feet = fields;
        feet.max.x = fields.min.x + fields.width() * 0.25;
        let from = WindowPoint::centre_of(feet);
        let mut moved = feet;
        moved.min.x += 80.0;
        moved.max.x += 80.0;
        let to = WindowPoint::centre_of(moved);
        self.pointer
            .drag_in(self.session, vp.as_deref(), from, to, 10, "l")?;
        self.session.settle(30);
        let dragged = self.last("model-nav-prefs", "eye_mm");
        if let Some(failure) = self.typed(report, vp.as_deref(), dragged.as_deref())? {
            return Ok(Err(failure));
        }
        let early = self.last("walk-eye-persisted", "mm");
        if early.is_some() {
            return Ok(Err(format!(
                "editing the height wrote it before Save as default: `walk-eye-persisted mm` \
                 {early:?}."
            )));
        }
        if !self.press(SAVE_DEFAULTS)? {
            return Ok(Err(format!(
                "no `{SAVE_DEFAULTS}` region in the options menu."
            )));
        }
        self.session.settle(20);
        let traced = self.last("model-nav-prefs", "eye_mm");
        let persisted = self.last("walk-eye-persisted", "mm");
        report.note(format!(
            "after Save as default: nav eye_mm {traced:?}, persisted {persisted:?}"
        ));
        let Some(mm) = persisted.as_deref().and_then(|m| m.parse::<u32>().ok()) else {
            return Ok(Err(format!(
                "Save as default persisted no height: `walk-eye-persisted mm` \
                 {persisted:?}, nav eye_mm {traced:?}."
            )));
        };
        let held = traced.as_deref().and_then(|t| t.parse::<f64>().ok());
        if held.is_none_or(|h| (h - f64::from(mm)).abs() > 0.5) {
            return Ok(Err(format!(
                "Save as default wrote {mm} mm while the viewer walks with eye_mm {traced:?}."
            )));
        }
        let on_disk = std::fs::read_to_string(userdata.join("preferences.txt"))
            .ok()
            .and_then(|t| value_of(&t, KEY));
        report.note(format!("preferences file: {KEY} = {on_disk:?}"));
        if on_disk != Some(mm.to_string()) {
            return Ok(Err(format!(
                "the trace persisted {mm} mm but the preferences file holds {on_disk:?}."
            )));
        }
        self.double_tap(report, target).map(|r| r.map(|()| mm))
    }

    /// Click the inches field, type 7 over what it holds and press Enter:
    /// the viewer's height must change from `before` and end on 7 inches.
    /// `Some` names the failure.
    fn typed(
        &self,
        report: &mut CheckReport,
        vp: Option<&str>,
        before: Option<&str>,
    ) -> Result<Option<String>> {
        let Some((now, _)) = declared_in(&self.session.trace()?, self.ui_rect, EYE) else {
            return Ok(Some(format!("`{EYE}` went away after the drag.")));
        };
        let mut inch = now;
        inch.min.x = now.max.x - now.width() * 0.25;
        self.pointer
            .click_in(self.session, vp, WindowPoint::centre_of(inch))?;
        self.session.settle(10);
        self.pointer.key(self.session, vp, "A", Some("ctrl"))?;
        self.pointer.type_text(self.session, vp, "7")?;
        self.pointer.key(self.session, vp, "Enter", None)?;
        self.session.settle(30);
        let after = self.last("model-nav-prefs", "eye_mm");
        report.note(format!("typed 7 in: eye_mm {before:?} -> {after:?}"));
        let inches = after
            .as_deref()
            .and_then(|m| m.parse::<f64>().ok())
            .map(|mm| (mm / 25.4) % 12.0);
        if after.as_deref() == before || inches.is_none_or(|i| (i - 7.0).abs() > 0.06) {
            return Ok(Some(format!(
                "typing 7 into the inches field did not set the height: eye_mm \
                 {before:?} -> {after:?}."
            )));
        }
        Ok(None)
    }

    /// Two quick taps on the gimbal's centre; `false` when it is not drawn.
    fn tap_centre_twice(&self) -> Result<bool> {
        let Some((rect, vp)) = declared_in(&self.session.trace()?, self.ui_rect, CENTRE) else {
            return Ok(false);
        };
        let at = WindowPoint::centre_of(rect);
        self.pointer.click_in(self.session, vp.as_deref(), at)?;
        self.pointer.click_in(self.session, vp.as_deref(), at)?;
        self.session.settle(60);
        Ok(true)
    }

    /// How many `model-gimbal-snapshot` lines the trace holds.
    fn snapshots(&self) -> Result<usize> {
        Ok(self
            .session
            .trace()?
            .events("model-gimbal-snapshot")
            .count())
    }

    /// Two taps on the gimbal's centre save a picture of the view before them.
    fn double_tap(
        &self,
        report: &mut CheckReport,
        target: &Path,
    ) -> Result<std::result::Result<(), String>> {
        // Facing along X puts that axis's tip over the centre, as the level
        // view walking opens on does with its forward axis.
        let Some((tip, vp)) = declared_in(&self.session.trace()?, self.ui_rect, TIP_X) else {
            return Ok(Err(format!(
                "no `{TIP_X}` region: the gimbal is not drawn."
            )));
        };
        self.pointer
            .click_in(self.session, vp.as_deref(), WindowPoint::centre_of(tip))?;
        self.session.settle(30);
        let level = self.snapshots()?;
        self.tap_centre_twice()?;
        if self.snapshots()? <= level {
            return Ok(Err(
                "two taps on the gimbal's centre while facing along X traced no \
                 `model-gimbal-snapshot`: the axis the view looks along took them."
                    .to_owned(),
            ));
        }
        // Tilt the look first, so a centre tap's levelling would show.
        let Some((image, vp)) = declared_in(&self.session.trace()?, self.ui_rect, IMAGE) else {
            return Ok(Err(format!("no `{IMAGE}` region.")));
        };
        let mut up = image;
        up.max.y -= image.height() * 0.3;
        let (from, to) = (WindowPoint::centre_of(image), WindowPoint::centre_of(up));
        self.pointer
            .drag_in(self.session, vp.as_deref(), from, to, 8, "l")?;
        self.session.settle(30);
        let before = self.last("model-view-rendered", "dir");
        let mark = self.session.trace()?.mark();
        if !self.tap_centre_twice()? {
            return Ok(Err(format!(
                "no `{CENTRE}` region: the gimbal is not drawn."
            )));
        }
        picture_saved(self.session, mark)?;
        let trace = self.session.trace()?;
        let snapshot = trace.events("model-gimbal-snapshot").last().is_some();
        let saved = trace
            .last_after("model-picture-saved", mark)
            .map(|l| l.raw.clone());
        let after = self.last("model-view-rendered", "dir");
        report.note(format!(
            "double tap: snapshot {snapshot}, saved {saved:?}, dir {before:?} -> {after:?}"
        ));
        if !snapshot || saved.is_none() {
            return Ok(Err(format!(
                "two taps on the gimbal's centre traced snapshot={snapshot} and saved {saved:?}."
            )));
        }
        if after != before {
            return Ok(Err(format!(
                "the double tap changed the view: dir {before:?} -> {after:?}."
            )));
        }
        let Ok(picture) = Image::load_png(target) else {
            return Ok(Err(format!("no PNG at {}.", target.display())));
        };
        report.artifact(target.to_path_buf());
        report.note(format!("scene {} x {}", picture.width(), picture.height()));
        Ok(Ok(()))
    }

    /// The second launch starts with the height written by the first.
    fn second(&self, report: &mut CheckReport, mm: u32) -> Result<std::result::Result<(), String>> {
        self.press(GHOST)?;
        self.press(GHOST)?;
        let traced = self.last("model-nav-prefs", "eye_mm");
        report.note(format!("second launch: eye_mm {traced:?}, written {mm}"));
        let back = traced.as_deref().and_then(|t| t.parse::<f64>().ok());
        if back.is_none_or(|b| (b - f64::from(mm)).abs() > 0.5) {
            return Ok(Err(format!(
                "the second launch walks with eye_mm {traced:?}; the first wrote {mm}."
            )));
        }
        Ok(Ok(()))
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

/// The value of `key = value` in a preferences file, ignoring comments.
pub(super) fn value_of(text: &str, key: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim().to_owned())
}
