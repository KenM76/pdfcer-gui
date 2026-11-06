//! `the_3d_viewers_settings_are_saved_only_as_defaults` — a setting
//! changed in the 3D viewer stays in that viewer until *Save as default*
//! writes every one as the operator's defaults; *Reset to defaults* takes up
//! the built-in ones and leaves the saved defaults alone (O398).
//!
//! Planted: the gimbal on. Oracles, from the trace and the preferences file:
//!
//! - Opened: the picture drawn by the card, `gpu=1` with no `gpu-declined`.
//! - The card's Shadows part switched off: `gpu-declined=turned-off`.
//! - The gimbal switched off too, the menu closed and the merge's second
//!   passed: the file holds `model3d_gimbal = 1` and no card key.
//! - *Save as default*: `model-view-defaults-saved`, and the file holds
//!   `model3d_card_part_0 = 0`, `model3d_gimbal = 0` and the keys never
//!   planted (`model3d_ghost`).
//! - *Reset to defaults*: `model-view-defaults-reset`, the card draws again
//!   with no `gpu-declined`, and the file still holds the saved defaults.

use crate::checks::model_axes::{launch, opened_from_page};
use crate::checks::model_eye::value_of;
use crate::checks::model_shadow_sun::rendered;
use crate::checks::viewer_options::press;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::Session;
use crate::report::CheckReport;

const PLANTED: &str = "model3d_gimbal = 1\n";
/// `dialogs::model3d::card::REGION_CARD_PART` for `CardPart::Shadows`.
const SHADOWS_ON_CARD: &str = "model3d.card.part.0";
/// `dialogs::model3d::nav::REGION_GIMBAL`.
const GIMBAL: &str = "model3d.gimbal";
/// `dialogs::model3d::options::REGION_SAVE_DEFAULTS`.
const SAVE: &str = "model3d.options.save_default";
/// `dialogs::model3d::options::REGION_RESET_DEFAULTS`.
const RESET: &str = "model3d.options.reset_default";
const FIELDS: [&str; 3] = ["gpu", "gpu-declined", "hash"];
/// The keys read from the file at each step.
const KEYS: [&str; 3] = ["model3d_gimbal", "model3d_card_part_0", "model3d_ghost"];

/// See the module documentation.
pub struct TheViewersSettingsAreSavedOnlyAsDefaults;

impl Check for TheViewersSettingsAreSavedOnlyAsDefaults {
    fn name(&self) -> &'static str {
        "the_3d_viewers_settings_are_saved_only_as_defaults"
    }

    fn defect(&self) -> &'static str {
        "A 3D viewer setting changed is written to the preferences file before Save as \
         default, or Save as default does not write it, or Reset to defaults overwrites the \
         saved defaults or leaves the viewer's own settings; or switching a graphics-card part \
         off leaves the picture on the card"
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

/// [`KEYS`]' values in the preferences file.
fn on_disk(userdata: &std::path::Path) -> [Option<String>; 3] {
    let text = std::fs::read_to_string(userdata.join("preferences.txt")).unwrap_or_default();
    KEYS.map(|k| value_of(&text, k))
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
    crate::sandbox::write_prefs(&userdata, PLANTED)
        .map_err(|e| Error::new(format!("could not plant the preferences file: {e}")))?;
    let (session, pointer) = launch(ctx, "model-defaults")?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome = (|| -> Result<Option<String>> {
        if let Some(failure) = opened_from_page(ctx, report, &session, &pointer, ui_rect)? {
            return Ok(Some(failure));
        }
        let steps = Steps {
            session: &session,
            press: &|name| press(&session, &pointer, ui_rect, name),
            userdata: &userdata,
        };
        steps.run(report)
    })();
    pointer.gone(&session)?;
    outcome
}

struct Steps<'a> {
    session: &'a Session,
    press: &'a dyn Fn(&str) -> Result<bool>,
    userdata: &'a std::path::Path,
}

impl Steps<'_> {
    fn pressed(&self, name: &str) -> Result<Option<String>> {
        Ok((!(self.press)(name)?).then(|| format!("no `{name}` region in the options menu.")))
    }

    fn picture(&self) -> Result<[String; 3]> {
        Ok(rendered(self.session, FIELDS)?.unwrap_or_else(|| FIELDS.map(|_| "none".to_owned())))
    }

    fn run(&self, report: &mut CheckReport) -> Result<Option<String>> {
        let first = self.picture()?;
        report.note(format!("opened {FIELDS:?}: {first:?}"));
        if first[..2] != ["1", "-"] {
            return Ok(Some(format!("the model opened off the card: {first:?}.")));
        }
        if let Some(failure) = self.pressed(SHADOWS_ON_CARD)? {
            return Ok(Some(failure));
        }
        let off = self.picture()?;
        report.note(format!("Shadows off the card: {off:?}"));
        if off[1] != "turned-off" {
            return Ok(Some(format!(
                "switching the card's Shadows part off drew {off:?}; wanted \
                 gpu-declined=turned-off."
            )));
        }
        if let Some(failure) = self.pressed(GIMBAL)? {
            return Ok(Some(failure));
        }
        self.session.settle(90);
        let before = on_disk(self.userdata);
        report.note(format!("before Save, file {KEYS:?}: {before:?}"));
        if before != [Some("1".to_owned()), None, None] {
            return Ok(Some(format!(
                "changing settings wrote them before Save as default: file {KEYS:?} {before:?}."
            )));
        }
        self.saved(report)
    }

    /// *Save as default*, then *Reset to defaults*.
    fn saved(&self, report: &mut CheckReport) -> Result<Option<String>> {
        if let Some(failure) = self.pressed(SAVE)? {
            return Ok(Some(failure));
        }
        self.session.settle(20);
        let trace = self.session.trace()?;
        let saved = trace
            .events("model-view-defaults-saved")
            .last()
            .and_then(|l| l.get("keys").map(str::to_owned));
        let after = on_disk(self.userdata);
        report.note(format!("Save: keys {saved:?}, file {KEYS:?}: {after:?}"));
        let want = [Some("0".to_owned()), Some("0".to_owned())];
        if saved.is_none() || after[..2] != want || after[2].is_none() {
            return Ok(Some(format!(
                "Save as default traced keys {saved:?} and left file {KEYS:?} {after:?}; \
                 wanted the gimbal off, card part 0 off and every key written."
            )));
        }
        if let Some(failure) = self.pressed(RESET)? {
            return Ok(Some(failure));
        }
        self.session.settle(30);
        let reset = self
            .session
            .trace()?
            .events("model-view-defaults-reset")
            .count();
        let picture = self.picture()?;
        let kept = on_disk(self.userdata);
        report.note(format!(
            "Reset: {reset} traced, picture {picture:?}, file {kept:?}"
        ));
        if reset == 0 || picture[..2] != ["1", "-"] {
            return Ok(Some(format!(
                "Reset to defaults traced {reset} resets and drew {picture:?}; wanted the \
                 card's picture again."
            )));
        }
        if kept != after {
            return Ok(Some(format!(
                "Reset to defaults changed the saved defaults: file {after:?} -> {kept:?}."
            )));
        }
        Ok(None)
    }
}
