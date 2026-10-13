//! `ocr_colour_setting` — **the recognised text is drawn in the colour the
//! preferences file names, and Settings ▸ Display's Reset writes the default
//! back.** Driven with the scripted pointer in windows placed off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_colour_setting.md`.

use std::path::Path;

use super::dimdrive::{launch_with, press, ui_rect_event};
use crate::checks::driving::{declared, declared_in, declared_names, list};
use crate::checks::ocr_layer_view::{FIXTURE, OCR_BAND, Scene};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::report::CheckReport;

/// Read mode with the OCR layer painted.
const SHOW_LAYER: &str = "mode.read,view.ocr_layer";
const OPEN_SETTINGS: &str = "file.settings";
const DIALOG: &str = "dialog:settings";
const DISPLAY_HEADING: &str = "settings.heading.display";
const RESET: &str = "settings.display.ocr_colour.reset";
/// The page body the Display controls scroll in.
const PAGE_BODY: &str = "settings.page"; // ui-text-exempt: a trace region name
/// Wheel rolls allowed to bring Reset into view.
const MAX_SCROLLS: usize = 8;
const SAVE: &str = "dialog:settings.save";
/// The dialog host's line when fitting the window to its content never settles.
const RUNAWAY: &str = "dialog-fit-runaway"; // ui-text-exempt: a trace event name
const PREFS_SAVED: &str = "prefs-saved"; // ui-text-exempt: a trace event name
const PREFS_FILE: &str = "preferences.txt";
const KEY: &str = "ocr_layer_colour";
/// The planted colour: green, which the default is nowhere near.
const PLANTED: &str = "#00B000";
/// `ocrlayerpref::DEFAULT_COLOUR`, as the file writes it.
const DEFAULT_WRITTEN: &str = "#CC0099";

/// See the module documentation.
pub struct OcrColourIsReadAndReset;

impl Check for OcrColourIsReadAndReset {
    fn name(&self) -> &'static str {
        "ocr_colour_setting"
    }

    fn defect(&self) -> &'static str {
        "the recognised text ignores the colour in the preferences file, or Settings ▸ Display's \
         Reset does not reach the file"
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

/// Pixels in the recognised line's band leaning green, and leaning magenta.
#[derive(Debug, Default)]
struct Tint {
    green: usize,
    magenta: usize,
}

/// Launch with the layer painted and classify the recognised line's pixels.
fn tint(ctx: &CheckContext, report: &mut CheckReport, tag: &str) -> Result<Tint> {
    let stem = format!("ocr_colour_setting.{tag}");
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, SHOW_LAYER, &stem, &[])?;
    let scene = Scene::now(ctx, &session, &pointer)?;
    let img = scene.shot(ctx, report, "ocr_colour_setting", tag)?;
    let mut tint = Tint::default();
    for p in img.pixels_in(scene.band(OCR_BAND)?) {
        let (r, g, b) = (i32::from(p.r), i32::from(p.g), i32::from(p.b));
        if g >= r + 40 && g >= b + 40 {
            tint.green += 1;
        } else if r >= g + 40 && b >= g + 30 {
            tint.magenta += 1;
        }
    }
    let _ = pointer.gone(&session);
    report.note(format!(
        "{tag}: the recognised line holds {} green-leaning and {} magenta-leaning pixels",
        tint.green, tint.magenta
    ));
    Ok(tint)
}

/// The colour line of the sandbox's preferences file, if any.
fn written(userdata: &Path) -> Result<Option<String>> {
    let text = std::fs::read_to_string(userdata.join(PREFS_FILE)).map_err(|e| {
        Error::new(format!(
            "cannot read {}: {e}",
            userdata.join(PREFS_FILE).display()
        ))
    })?;
    Ok(text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == KEY).then(|| v.trim().to_owned())
        }))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, &format!("{KEY} = {PLANTED}\n"))
        .map_err(|e| Error::new(format!("could not plant the preference: {e}")))?;

    // --- 1: the planted colour is the one drawn ------------------------------
    let planted = tint(ctx, report, "planted")?;
    if planted.green + planted.magenta == 0 {
        return Err(Error::new(
            "the recognised line shows no coloured pixels at all, so the layer did not paint. \
             SKIPPED: `ocr_layer_view` owns that property.",
        ));
    }
    if planted.green == 0 || planted.magenta != 0 {
        return Ok(Some(format!(
            "{KEY} = {PLANTED} was planted and the recognised text drew {planted:?}: the colour \
             in the preferences file is not the one on the canvas."
        )));
    }

    // --- 2: Reset, Save, and the file ---------------------------------------
    if let Some(failure) = reset_and_save(ctx, report)? {
        return Ok(Some(failure));
    }
    let line = written(&userdata)?;
    report.note(format!(
        "after Reset and Save the file reads {KEY} = {line:?}"
    ));
    if !line
        .as_deref()
        .is_some_and(|v| v.eq_ignore_ascii_case(DEFAULT_WRITTEN))
    {
        return Ok(Some(format!(
            "Reset and Save left {KEY} = {line:?} in the preferences file, not {DEFAULT_WRITTEN}."
        )));
    }

    // --- 3: the next run reads the default back ------------------------------
    let reset = tint(ctx, report, "reset")?;
    if reset.magenta == 0 || reset.green != 0 {
        return Ok(Some(format!(
            "the file says {DEFAULT_WRITTEN} and the next run drew {reset:?}: the saved colour \
             is not the one read back."
        )));
    }
    report.note("the planted colour is drawn, Reset writes the default, and the next run draws it");
    Ok(None)
}

/// Open Settings, roll Display down to the colour's Reset, press it, Save.
/// The window is closed (dropped) before this returns, so the file is final.
fn reset_and_save(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    let (session, pointer) = launch_with(
        ctx,
        report,
        &FIXTURE,
        OPEN_SETTINGS,
        "ocr_colour_setting.settings",
        &[],
    )?;
    let trace = session.trace()?;
    let Some((_, dialog_vp)) = declared_in(&trace, ui_rect, DIALOG) else {
        return Err(Error::new(format!(
            "`{OPEN_SETTINGS}` was invoked and no `{DIALOG}` appeared."
        )));
    };
    if declared_in(&trace, ui_rect, DISPLAY_HEADING).is_none() {
        return Ok(Some(format!(
            "the Settings window declares no `{DISPLAY_HEADING}`. Headings: {}.",
            list(&declared_names(&trace, ui_rect, "settings.heading."))
        )));
    }
    press(ctx, &session, &pointer, DISPLAY_HEADING)?;
    // The colour sits low on a long page: roll the page body until Reset is
    // in view, as the operator would.
    for _ in 0..MAX_SCROLLS {
        let trace = session.trace()?;
        if declared_in(&trace, ui_rect, RESET).is_some() {
            break;
        }
        let Some((body, vp)) = declared_in(&trace, ui_rect, PAGE_BODY) else {
            break;
        };
        pointer.wheel_in(&session, vp.as_deref(), WindowPoint::centre_of(body), -3.0)?;
        session.settle(10);
    }
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, RESET).is_none() {
        return Ok(Some(format!(
            "the Display page is open with a non-default colour and declares no `{RESET}`: \
             Reset is not offered, or is scrolled out of view. Regions: {}.",
            list(&declared_names(&trace, ui_rect, "settings.display."))
        )));
    }
    press(ctx, &session, &pointer, RESET)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, RESET).is_some() {
        return Ok(Some(
            "Reset was pressed and is still drawn: the colour did not return to the default."
                .to_owned(),
        ));
    }
    // The picture is evidence for a reader, not an oracle: an immediate
    // viewport never answers a screenshot, so a refusal is noted and passed.
    let shot = ctx.out("ocr_colour_setting.settings.png");
    match pointer.screenshot_in(&session, dialog_vp.as_deref(), &shot) {
        Ok(()) => {
            report.artifact(shot);
        }
        Err(why) => {
            report.note(format!("no picture of the Settings window: {why}"));
        }
    }
    let Some(save) = declared(&trace, ui_rect, SAVE) else {
        return Ok(Some(format!("the Settings window declares no `{SAVE}`.")));
    };
    if let Some(runaway) = trace.last(RUNAWAY) {
        return Ok(Some(format!(
            "the Settings window grew to fit its content until the host gave up: `{}`.",
            runaway.raw
        )));
    }
    let body = declared(&trace, ui_rect, DIALOG)
        .ok_or_else(|| Error::new(format!("no `{DIALOG}` region after Reset.")))?;
    if save.max.y > body.max.y {
        return Ok(Some(format!(
            "Save is laid out at y={:.0}..{:.0}, below the window's body, which ends at y={:.0}: the operator cannot press it.",
            save.min.y, save.max.y, body.max.y
        )));
    }
    let saves = trace.events(PREFS_SAVED).count();
    press(ctx, &session, &pointer, SAVE)?;
    if session.trace()?.events(PREFS_SAVED).count() == saves {
        return Ok(Some(format!(
            "Save was pressed after Reset and no `{PREFS_SAVED}` line followed."
        )));
    }
    let _ = pointer.gone(&session);
    drop(session);
    Ok(None)
}
