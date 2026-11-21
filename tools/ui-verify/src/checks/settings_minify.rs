//! `every_minify_filter_is_offered` — **Settings ▸ Images offers all three
//! ways of shrinking an image, area averaging among them, and Save writes
//! the one pressed.** Driven off the desktop with the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/settings_minify.md`.

use super::dimdrive::{Fixture, launch_with, press, ui_rect_event};
use crate::checks::driving::{declared, declared_in, declared_names, list};
use crate::checks::scripteddrive::{setting_written, userdata};
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
const KEY: &str = "image_minify"; // ui-text-exempt: a settings-file key
const OPEN_SETTINGS: &str = "file.settings";
const DIALOG: &str = "dialog:settings";
const HEADING: &str = "settings.heading.images";
const PAGE_BODY: &str = "settings.page"; // ui-text-exempt: a trace region name
/// `settingspages::images::MINIFY_REGION`; each option is `<this>.<word>`,
/// the word being the settings file's.
const OPTION: &str = "settings.images.minify";
const WORDS: [&str; 3] = ["point_sample", "smooth", "area_average"];
/// Wheel rolls allowed to bring an option into view.
const MAX_SCROLLS: usize = 40;
const SAVE: &str = "dialog:settings.save";
const SAVED: &str = "settings-saved"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct EveryMinifyFilterIsOffered;

impl Check for EveryMinifyFilterIsOffered {
    fn name(&self) -> &'static str {
        "every_minify_filter_is_offered"
    }

    fn defect(&self) -> &'static str {
        "Settings ▸ Images does not offer every way of shrinking an image (area averaging, \
         the default, missing), or Save does not write the one pressed"
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

/// Smooth first, so the area-average press has something to change.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    for word in ["smooth", "area_average"] {
        if let Some(failure) = choose(ctx, report, word)? {
            return Ok(Some(failure));
        }
        let after = setting_written(&userdata(ctx)?, KEY);
        report.note(format!(
            "after pressing `{word}` and Save: {KEY} = {after:?}"
        ));
        if after.as_deref() != Some(word) {
            return Ok(Some(format!(
                "`{OPTION}.{word}` was pressed and saved, and the settings file says \
                 {KEY} = {after:?}."
            )));
        }
    }
    Ok(None)
}

/// Open Settings ▸ Images, roll until every option is declared, press
/// `word`'s, Save. The window is closed before this returns.
fn choose(ctx: &CheckContext, report: &mut CheckReport, word: &str) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    let (session, pointer) = launch_with(ctx, report, &PLAIN, OPEN_SETTINGS, "minify", &[])?;
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
    let all = |trace: &crate::trace::Trace| {
        WORDS
            .iter()
            .all(|w| declared_in(trace, ui_rect, &format!("{OPTION}.{w}")).is_some())
    };
    for _ in 0..MAX_SCROLLS {
        let trace = session.trace()?;
        if all(&trace) {
            break;
        }
        let Some((body, vp)) = declared_in(&trace, ui_rect, PAGE_BODY) else {
            break;
        };
        pointer.wheel_in(&session, vp.as_deref(), WindowPoint::centre_of(body), -3.0)?;
        session.settle(10);
    }
    let trace = session.trace()?;
    if !all(&trace) {
        return Ok(Some(format!(
            "the Images page does not declare all of `{OPTION}.{{{}}}`: a way of shrinking \
             images is not offered. Declared: {}.",
            WORDS.join(","),
            list(&declared_names(&trace, ui_rect, OPTION))
        )));
    }
    press(ctx, &session, &pointer, &format!("{OPTION}.{word}"))?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, SAVE).is_none() {
        return Ok(Some(format!("the Settings window declares no `{SAVE}`.")));
    }
    let saves = trace.events(SAVED).count();
    press(ctx, &session, &pointer, SAVE)?;
    if session.trace()?.events(SAVED).count() == saves {
        return Ok(Some(format!(
            "Save was pressed after `{word}` and no `{SAVED}` line followed."
        )));
    }
    let _ = pointer.gone(&session);
    drop(session);
    Ok(None)
}
