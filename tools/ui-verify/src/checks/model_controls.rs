//! `checks::model_controls` — **the 3D viewer's tool row slides away and
//! back on its arrow, and its camera and sticks glyphs work**
//!
//! Places `fixtures/floor-box.prc` and opens it with *View…*. Presses the
//! row's sticks glyph twice with the camera glyph between, taps the picture
//! once, then presses the corner arrow twice.
//!
//! Oracles, from the region log and the trace (the viewer is an immediate
//! viewport no screenshot reaches):
//!
//! - Opened: the arrow, the camera glyph, the sticks glyph and the gimbal's
//!   `+X` tip are all drawn — the row starts shown.
//! - Sticks: `model-nav-controls shown=false`, the tip gone, the camera
//!   glyph still drawn.
//! - Camera: `model-view-shot`, then `model-picture-saved` and a file at
//!   the picker's answer.
//! - Sticks again: `shown=true`, the tip back.
//! - One tap on the picture: no `model-nav-controls` line — a tap no longer
//!   toggles the controls.
//! - Arrow: `model-nav-bar shown=false`, the camera and sticks glyphs gone,
//!   the arrow still drawn; again: `shown=true` and the glyphs back.

use crate::checks::driving::{declared_in, repo_fixture};
use crate::checks::model_shadow_sun::open;
use crate::checks::model_view_window::{launch_with_model_env, picture_saved, ui_rect};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "floor-box.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe>`.";
/// `dialogs::model3d::controls::REGION_BAR`.
const BAR: &str = "model3d.bar";
/// `dialogs::model3d::controls::REGION_SHOT`.
const SHOT: &str = "model3d.shot";
/// `dialogs::model3d::controls::REGION_STICKS`.
const STICKS: &str = "model3d.sticks";
/// The gimbal's `+X` tip.
const TIP: &str = "model3d.gimbal.tip.0";
const IMAGE: &str = "model3d.image";
/// The picture picker's diagnostic answer, read by `app::files::pick_picture_target`.
const PICTURE_ENV: &str = "PDFCER_DIAG_PICTURE_SAVE_PATH";

/// See the module documentation.
pub struct TheViewerControlsHide;

impl Check for TheViewerControlsHide {
    fn name(&self) -> &'static str {
        "the_3d_viewers_tool_row_slides_away_on_its_arrow_and_its_camera_and_sticks_work"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's tool row does not start shown, its arrow does not hide and bring \
         it back, its sticks glyph does not hide and show the navigation controls, its \
         camera glyph saves nothing, or a tap on the picture still toggles the controls"
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

/// What is drawn: the arrow, the camera glyph, the sticks glyph, the tip.
type Drawn = (bool, bool, bool, bool);

impl Driver<'_> {
    fn drawn(&self) -> Result<Drawn> {
        let trace = self.session.trace()?;
        let at = |name| declared_in(&trace, self.ui_rect, name).is_some();
        Ok((at(BAR), at(SHOT), at(STICKS), at(TIP)))
    }

    /// The `shown` of each `event` line.
    fn said(&self, event: &str) -> Result<Vec<String>> {
        Ok(self
            .session
            .trace()?
            .events(event)
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

    /// Press `name` and compare what is drawn and what `event` said.
    fn step(
        &self,
        report: &mut CheckReport,
        name: &str,
        (event, said): (&str, &[&str]),
        want: Drawn,
    ) -> Result<Option<String>> {
        if !self.click(name, 1)? {
            return Ok(Some(format!("no `{name}` drawn to press.")));
        }
        let got = self.drawn()?;
        let lines = self.said(event)?;
        report.note(format!("{name}: drawn {got:?}, {event} {lines:?}"));
        if got != want || lines != said {
            return Ok(Some(format!(
                "pressing `{name}` drew (arrow, camera, sticks, tip) {got:?} and traced \
                 {event} {lines:?}; expected {want:?} and {said:?}."
            )));
        }
        Ok(None)
    }

    fn drive(&self, report: &mut CheckReport, target: &std::path::Path) -> Result<Option<String>> {
        let opened = self.drawn()?;
        report.note(format!("opened: {opened:?}"));
        if opened != (true, true, true, true) {
            return Ok(Some(format!(
                "the viewer opened with (arrow, camera, sticks, tip) drawn {opened:?}; the \
                 row must start shown with the controls."
            )));
        }
        let off = ("model-nav-controls", &["false"][..]);
        if let Some(f) = self.step(report, STICKS, off, (true, true, true, false))? {
            return Ok(Some(f));
        }
        let mark = self.session.trace()?.mark();
        self.click(SHOT, 1)?;
        let saved = picture_saved(self.session, mark)?;
        let shot = self
            .session
            .trace()?
            .last_after("model-view-shot", mark)
            .is_some();
        report.note(format!(
            "camera: shot {shot}, saved {saved}, file {}",
            target.is_file()
        ));
        if !(shot && saved && target.is_file()) {
            return Ok(Some(format!(
                "the camera glyph traced shot {shot}, saved {saved}, wrote a file {}; \
                 expected a saved picture.",
                target.is_file()
            )));
        }
        let on = ("model-nav-controls", &["false", "true"][..]);
        if let Some(f) = self.step(report, STICKS, on, (true, true, true, true))? {
            return Ok(Some(f));
        }
        if let Some(f) = self.step(report, IMAGE, on, (true, true, true, true))? {
            return Ok(Some(f));
        }
        let away = ("model-nav-bar", &["false"][..]);
        if let Some(f) = self.step(report, BAR, away, (true, false, false, true))? {
            return Ok(Some(f));
        }
        let back = ("model-nav-bar", &["false", "true"][..]);
        self.step(report, BAR, back, (true, true, true, true))
    }
}
