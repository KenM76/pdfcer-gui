//! `checks::model_controls` — **the 3D viewer's navigation controls hide
//! and show, and while hidden a camera button saves the view**
//!
//! Places `fixtures/floor-box.prc` and opens it with *View…*. Clears *Show
//! navigation controls* in the options menu, presses the camera button
//! that takes their place, then taps the picture once and twice.
//!
//! Oracles, from the region log and the trace (the viewer is an immediate
//! viewport no screenshot reaches):
//!
//! - Opened: the gimbal's `+X` tip is drawn and the camera button is not.
//! - Cleared: `model-nav-controls shown=false`; the tip is gone and the
//!   camera button drawn.
//! - The camera button: `model-view-shot`, then `model-picture-saved` and a
//!   file at the picker's answer; the controls stay hidden.
//! - One tap on the picture: `shown=true`, the tip back, the button gone.
//! - Two taps: no `model-nav-controls` line, so a double tap keeps its
//!   own job.

use crate::checks::driving::{declared_in, repo_fixture};
use crate::checks::model_shadow_sun::open;
use crate::checks::model_view_window::{launch_with_model_env, picture_saved, ui_rect};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "floor-box.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe>`.";
/// `dialogs::model3d::controls::REGION_CONTROLS`.
const CONTROLS: &str = "model3d.controls";
/// `dialogs::model3d::controls::REGION_SHOT`.
const SHOT: &str = "model3d.shot";
/// The gimbal's `+X` tip.
const TIP: &str = "model3d.gimbal.tip.0";
const IMAGE: &str = "model3d.image";
/// The picture picker's diagnostic answer, read by `app::files::pick_picture_target`.
const PICTURE_ENV: &str = "PDFCER_DIAG_PICTURE_SAVE_PATH";
const SWITCHED: &str = "model-nav-controls";

/// See the module documentation.
pub struct TheViewerControlsHide;

impl Check for TheViewerControlsHide {
    fn name(&self) -> &'static str {
        "the_3d_viewers_controls_hide_on_a_tap_and_a_camera_button_saves_the_view"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's navigation controls cannot be hidden, a single tap does not show or \
         hide them, a double tap toggles them, or the camera button shown while they are \
         hidden is missing or saves nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let target = ctx.out("model-controls-shot.png");
        let _ = std::fs::remove_file(&target);
        let env = [(PICTURE_ENV, target.display().to_string())];
        let driven = repo_fixture(MODEL, METHOD)
            .and_then(|model| {
                launch_with_model_env(
                    ctx,
                    &mut report,
                    "model-controls",
                    &model.to_string_lossy(),
                    &env,
                )
            })
            .and_then(|(session, pointer)| {
                let driver = Driver {
                    session: &session,
                    pointer: &pointer,
                    ui_rect: ui_rect(ctx)?,
                };
                if let Some(failure) = open(ctx, &session, &pointer)? {
                    return Ok(Some(failure));
                }
                driver.drive(&mut report, &target)
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

struct Driver<'a> {
    session: &'a Session,
    pointer: &'a ScriptedPointer,
    ui_rect: &'static str,
}

impl Driver<'_> {
    /// Whether the tip and the camera button are drawn.
    fn drawn(&self) -> Result<(bool, bool)> {
        let trace = self.session.trace()?;
        Ok((
            declared_in(&trace, self.ui_rect, TIP).is_some(),
            declared_in(&trace, self.ui_rect, SHOT).is_some(),
        ))
    }

    /// The `shown` of each `model-nav-controls` line.
    fn switches(&self) -> Result<Vec<String>> {
        Ok(self
            .session
            .trace()?
            .events(SWITCHED)
            .map(|l| l.get("shown").unwrap_or("-").to_owned())
            .collect())
    }

    /// Click `name`'s centre `times` times; `false` when it is drawn nowhere.
    fn click(&self, name: &str, times: usize) -> Result<bool> {
        let Some((rect, vp)) = declared_in(&self.session.trace()?, self.ui_rect, name) else {
            return Ok(false);
        };
        for _ in 0..times {
            self.pointer
                .click_in(self.session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        }
        self.pointer.gone(self.session)?;
        self.session.settle(60);
        Ok(true)
    }

    fn drive(&self, report: &mut CheckReport, target: &std::path::Path) -> Result<Option<String>> {
        let opened = self.drawn()?;
        report.note(format!("opened (tip, shot): {opened:?}"));
        if opened != (true, false) {
            return Ok(Some(format!(
                "the viewer opened with (gimbal tip, camera button) drawn {opened:?}; expected \
                 the controls shown and no camera button."
            )));
        }
        if !viewer_options::press(self.session, self.pointer, self.ui_rect, CONTROLS)? {
            return Ok(Some(format!("no `{CONTROLS}` switch in the options menu.")));
        }
        self.session.settle(20);
        let hidden = self.drawn()?;
        let said = self.switches()?;
        report.note(format!("cleared: {hidden:?}, switches {said:?}"));
        if hidden != (false, true) || said != ["false"] {
            return Ok(Some(format!(
                "clearing Show navigation controls drew {hidden:?} and traced {said:?}; \
                 expected the gimbal gone, the camera button drawn, and `shown=false`."
            )));
        }
        let mark = self.session.trace()?.mark();
        if !self.click(SHOT, 1)? {
            return Ok(Some(format!(
                "the `{SHOT}` button vanished before its press."
            )));
        }
        let saved = picture_saved(self.session, mark)?;
        let shot = self
            .session
            .trace()?
            .last_after("model-view-shot", mark)
            .is_some();
        let kept = self.drawn()?;
        report.note(format!(
            "camera button: shot {shot}, saved {saved}, file {}, then {kept:?}",
            target.is_file()
        ));
        if !(shot && saved && target.is_file()) || kept != (false, true) {
            return Ok(Some(format!(
                "pressing the camera button traced shot {shot}, saved {saved}, wrote a file \
                 {}, and left {kept:?}; expected a saved picture with the controls still \
                 hidden.",
                target.is_file()
            )));
        }
        self.taps(report)
    }

    /// One tap on the picture shows the controls; two leave them.
    fn taps(&self, report: &mut CheckReport) -> Result<Option<String>> {
        if !self.click(IMAGE, 1)? {
            return Ok(Some(format!("no `{IMAGE}` region.")));
        }
        let shown = self.drawn()?;
        let said = self.switches()?;
        report.note(format!("one tap: {shown:?}, switches {said:?}"));
        if shown != (true, false) || said != ["false", "true"] {
            return Ok(Some(format!(
                "one tap on the picture drew {shown:?} and traced {said:?}; expected the \
                 controls back and the camera button gone."
            )));
        }
        self.click(IMAGE, 2)?;
        let twice = self.drawn()?;
        let after = self.switches()?;
        report.note(format!("two taps: {twice:?}, switches {after:?}"));
        if twice != (true, false) || after != said {
            return Ok(Some(format!(
                "a double tap on the picture drew {twice:?} and traced {after:?}; a double \
                 tap must leave the controls as they were."
            )));
        }
        Ok(None)
    }
}
