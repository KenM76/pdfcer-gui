//! `settings_sun_disc` — **Settings ▸ 3D's default sun is aimed on the sun
//! disc, and Save writes the bearing and height the press chose.** Driven
//! off the desktop with the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/settings_sun_disc.md`.

use super::dimdrive::{Fixture, launch_with, press, ui_rect_event};
use crate::checks::driving::{declared, declared_in, declared_names, list};
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::report::CheckReport;

const PLAIN: Fixture = Fixture {
    file: "checker-image.pdf",
    method: "Rebuild it with `python fixtures/checker-image.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const BEARING: &str = "model3d_look_sun_bearing"; // ui-text-exempt: a settings-file key
const HEIGHT: &str = "model3d_look_sun_height"; // ui-text-exempt: a settings-file key
const SETTINGS_FILE: &str = "preferences.txt";
const OPEN_SETTINGS: &str = "file.settings";
const DIALOG: &str = "dialog:settings";
const HEADING: &str = "settings.heading.model3d";
const PAGE_BODY: &str = "settings.page"; // ui-text-exempt: a trace region name
/// `dialogs::model3d::sunlight::REGION_SUN_DISC`.
const DISC: &str = "model3d.sun.disc";
/// Wheel rolls allowed to bring the disc into view.
const MAX_SCROLLS: usize = 40;
const SAVE: &str = "dialog:settings.save";
const SAVED: &str = "settings-saved"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct SettingsSunDisc;

impl Check for SettingsSunDisc {
    fn name(&self) -> &'static str {
        "settings_sun_disc"
    }

    fn defect(&self) -> &'static str {
        "Settings ▸ 3D draws no sun disc for the default sun, a press on it does not aim \
         the sun, or Save does not write the bearing and height"
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

/// `key`'s value in the sandbox's settings file, as a number.
fn written(userdata: &std::path::Path, key: &str) -> Option<f64> {
    let text = std::fs::read_to_string(userdata.join(SETTINGS_FILE)).ok()?;
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key).then(|| v.trim().parse().ok())?
        })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let dir = userdata(ctx)?;
    let before = (written(&dir, BEARING), written(&dir, HEIGHT));
    report.note(format!(
        "before: bearing {:?}, height {:?}",
        before.0, before.1
    ));
    if before.0.is_some_and(|b| (b - 270.0).abs() < 1.0) {
        return Ok(Some(format!(
            "the sandbox already says {BEARING} = 270, the bearing a left-rim press \
             chooses, so the oracle could not tell."
        )));
    }
    if let Some(failure) = aim_and_save(ctx, report)? {
        return Ok(Some(failure));
    }
    let (bearing, height) = (written(&dir, BEARING), written(&dir, HEIGHT));
    report.note(format!(
        "after pressing the disc's left rim and Save: bearing {bearing:?}, height {height:?}"
    ));
    if bearing.is_none_or(|b| (b - 270.0).abs() > 1.0) || height.is_none_or(|h| h > 1.0) {
        return Ok(Some(format!(
            "the disc's left rim was pressed and saved, and the settings file says \
             {BEARING} = {bearing:?}, {HEIGHT} = {height:?}; expected 270 and 0 or 1: the \
             front at the bottom puts the left rim at 270° on the horizon."
        )));
    }
    Ok(None)
}

/// Open Settings ▸ 3D, roll to the disc, press its left rim, Save. The
/// window is closed before this returns, so the file is final.
fn aim_and_save(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    let (session, pointer) =
        launch_with(ctx, report, &PLAIN, OPEN_SETTINGS, "settings_sun_disc", &[])?;
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, DIALOG).is_none() {
        return Err(Error::new(format!(
            "`{OPEN_SETTINGS}` was invoked and no `{DIALOG}` appeared."
        )));
    }
    if declared_in(&trace, ui_rect, HEADING).is_none() {
        return Ok(Some(format!(
            "the Settings window declares no `{HEADING}`. Headings: {}.",
            list(&declared_names(&trace, ui_rect, "settings.heading."))
        )));
    }
    press(ctx, &session, &pointer, HEADING)?;
    for _ in 0..MAX_SCROLLS {
        let trace = session.trace()?;
        if declared_in(&trace, ui_rect, DISC).is_some() {
            break;
        }
        let Some((body, vp)) = declared_in(&trace, ui_rect, PAGE_BODY) else {
            break;
        };
        pointer.wheel_in(&session, vp.as_deref(), WindowPoint::centre_of(body), -3.0)?;
        session.settle(10);
    }
    let trace = session.trace()?;
    let Some((disc, vp)) = declared_in(&trace, ui_rect, DISC) else {
        return Ok(Some(format!(
            "the 3D page declares no `{DISC}`. Regions: {}.",
            list(&declared_names(&trace, ui_rect, "settings."))
        )));
    };
    // Mid-height: the page scrolls only until most of the disc shows.
    let at = Pt {
        x: disc.min.x + (disc.max.x - disc.min.x) * 0.04,
        y: (disc.min.y + disc.max.y) / 2.0,
    };
    pointer.click_in(
        &session,
        vp.as_deref(),
        WindowPoint::centre_of(LRect { min: at, max: at }),
    )?;
    session.settle(30);
    let trace = session.trace()?;
    if declared(&trace, ui_rect, SAVE).is_none() {
        return Ok(Some(format!("the Settings window declares no `{SAVE}`.")));
    }
    let saves = trace.events(SAVED).count();
    press(ctx, &session, &pointer, SAVE)?;
    if session.trace()?.events(SAVED).count() == saves {
        return Ok(Some(format!(
            "Save was pressed after the disc and no `{SAVED}` line followed (look for \
             `settings-save-failed`)."
        )));
    }
    let _ = pointer.gone(&session);
    drop(session);
    Ok(None)
}
