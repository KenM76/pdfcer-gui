//! `calibrated_cmyk_setting` — **Settings ▸ Colour's *Keep the CMYK numbers
//! as written* switch is offered, and Save writes it to the settings file.**
//! Driven off the desktop with the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/calibrated_cmyk_setting.md`.

use super::dimdrive::{Fixture, launch_with, press, ui_rect_event};
use crate::checks::driving::{declared, declared_in, declared_names, list};
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const PLAIN: Fixture = Fixture {
    file: "checker-image.pdf",
    method: "Rebuild it with `python fixtures/checker-image.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const KEY: &str = "preserve_calibrated_cmyk"; // ui-text-exempt: a settings-file key
const SETTINGS_FILE: &str = "settings.txt";
const OPEN_SETTINGS: &str = "file.settings";
const DIALOG: &str = "dialog:settings";
const COLOUR_HEADING: &str = "settings.heading.colour";
const PAGE_BODY: &str = "settings.page"; // ui-text-exempt: a trace region name
/// `settingspages::colour::CALIBRATED_CMYK_REGION`.
const SWITCH: &str = "settings.colour.calibrated_cmyk";
/// Wheel rolls allowed to bring the switch into view; it is the page's last.
const MAX_SCROLLS: usize = 40;
const SAVE: &str = "dialog:settings.save";
const SAVED: &str = "settings-saved"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct CalibratedCmykIsSaved;

impl Check for CalibratedCmykIsSaved {
    fn name(&self) -> &'static str {
        "calibrated_cmyk_setting"
    }

    fn defect(&self) -> &'static str {
        "Settings ▸ Colour offers no way to keep calibrated CMYK numbers as the ink, or Save \
         does not write the choice"
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

/// The key's value in the sandbox's settings file; `None` when absent.
fn written(userdata: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(userdata.join(SETTINGS_FILE)).ok()?;
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == KEY).then(|| v.trim().to_owned())
        })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let before = written(&userdata(ctx)?);
    report.note(format!("before: {KEY} = {before:?}"));
    if before.as_deref() == Some("true") {
        return Ok(Some(format!(
            "the sandbox's settings file already says {KEY} = true, so pressing the switch \
             would turn it off and the oracle could not tell."
        )));
    }
    if let Some(failure) = switch_on(ctx, report)? {
        return Ok(Some(failure));
    }
    let after = written(&userdata(ctx)?);
    report.note(format!(
        "after pressing the switch and Save: {KEY} = {after:?}"
    ));
    if after.as_deref() != Some("true") {
        return Ok(Some(format!(
            "the switch was pressed and saved, and the settings file says {KEY} = {after:?}."
        )));
    }
    Ok(None)
}

/// Open Settings ▸ Colour, roll to the switch, press it, Save. The window is
/// closed before this returns, so the file is final.
fn switch_on(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    let (session, pointer) =
        launch_with(ctx, report, &PLAIN, OPEN_SETTINGS, "calibrated_cmyk", &[])?;
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, DIALOG).is_none() {
        return Err(Error::new(format!(
            "`{OPEN_SETTINGS}` was invoked and no `{DIALOG}` appeared."
        )));
    }
    if declared_in(&trace, ui_rect, COLOUR_HEADING).is_none() {
        return Ok(Some(format!(
            "the Settings window declares no `{COLOUR_HEADING}`. Headings: {}.",
            list(&declared_names(&trace, ui_rect, "settings.heading."))
        )));
    }
    press(ctx, &session, &pointer, COLOUR_HEADING)?;
    for _ in 0..MAX_SCROLLS {
        let trace = session.trace()?;
        if declared_in(&trace, ui_rect, SWITCH).is_some() {
            break;
        }
        let Some((body, vp)) = declared_in(&trace, ui_rect, PAGE_BODY) else {
            break;
        };
        pointer.wheel_in(&session, vp.as_deref(), WindowPoint::centre_of(body), -3.0)?;
        session.settle(10);
    }
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, SWITCH).is_none() {
        return Ok(Some(format!(
            "the Colour page declares no `{SWITCH}`: the switch is not offered, or is out of \
             reach. Regions: {}.",
            list(&declared_names(&trace, ui_rect, "settings."))
        )));
    }
    press(ctx, &session, &pointer, SWITCH)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, SAVE).is_none() {
        return Ok(Some(format!("the Settings window declares no `{SAVE}`.")));
    }
    let saves = trace.events(SAVED).count();
    press(ctx, &session, &pointer, SAVE)?;
    if session.trace()?.events(SAVED).count() == saves {
        return Ok(Some(format!(
            "Save was pressed after the switch and no `{SAVED}` line followed (look for \
             `settings-save-failed`)."
        )));
    }
    let _ = pointer.gone(&session);
    drop(session);
    Ok(None)
}
