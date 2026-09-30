//! `ocr_colour_setting` — **the recognised text is drawn in the colour the
//! preferences file names, and Settings ▸ Display's Reset writes the default
//! back.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_colour_setting.md`.

use std::path::Path;

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, frame_of, list};
use crate::checks::ocr_layer_view::{OCR_BAND, Scene};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "ocr-layers.pdf";
/// Read mode with the OCR layer painted.
const SHOW_LAYER: &str = "mode.read,view.ocr_layer";
const OPEN_SETTINGS: &str = "file.settings";
const VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
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

fn launch(ctx: &CheckContext, exe: &Path, tag: &str, invoke: &str, pdf: bool) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out(&format!("ocr_colour_setting.{tag}.trace.txt")));
    if pdf {
        spec.pdf = Some(
            crate::fixture::workspace_root()
                .join("fixtures")
                .join(FIXTURE),
        );
    }
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), invoke.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    Session::launch(&spec, ctx.profile.trace_prefix)
}

/// Launch with the layer painted and classify the recognised line's pixels.
fn tint(ctx: &CheckContext, report: &mut CheckReport, exe: &Path, tag: &str) -> Result<Tint> {
    let ui_rect = ui_rect(ctx)?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    let page: PageGeometry = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let session = launch(ctx, exe, tag, SHOW_LAYER, true)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let trace = session.trace()?;
    let frame = session.frame()?;
    let canvas = frame.logical_to_capture_pixels(
        declared(&trace, ui_rect, VIEWPORT)
            .ok_or_else(|| Error::new(format!("no `{VIEWPORT}` region. SKIPPED.")))?,
    );
    let scene = Scene {
        session: &session,
        frame,
        mapping: CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?,
        canvas,
    };
    let path = ctx.out(&format!("ocr_colour_setting.{tag}.png"));
    let img = crate::capture::window_to_png(&session, &path)?;
    report.artifact(path);
    let mut tint = Tint::default();
    for p in img.pixels_in(scene.band(OCR_BAND)?) {
        let (r, g, b) = (i32::from(p.r), i32::from(p.g), i32::from(p.b));
        if g >= r + 40 && g >= b + 40 {
            tint.green += 1;
        } else if r >= g + 40 && b >= g + 30 {
            tint.magenta += 1;
        }
    }
    report.note(format!(
        "{tag}: the recognised line holds {} green-leaning and {} magenta-leaning pixels",
        tint.green, tint.magenta
    ));
    Ok(tint)
}

fn ui_rect(ctx: &CheckContext) -> Result<&'static str> {
    ctx.profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the Settings window.",
        ));
    }
    let ui_rect = ui_rect(ctx)?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, &format!("{KEY} = {PLANTED}\n"))
        .map_err(|e| Error::new(format!("could not plant the preference: {e}")))?;

    // --- 1: the planted colour is the one drawn ------------------------------
    let planted = tint(ctx, report, &exe, "planted")?;
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
    let session = launch(ctx, &exe, "settings", OPEN_SETTINGS, false)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());
    let trace = session.trace()?;
    if declared(&trace, ui_rect, DIALOG).is_none() {
        return Err(Error::new(format!(
            "`{OPEN_SETTINGS}` was invoked and no `{DIALOG}` appeared."
        )));
    }
    let Some(heading) = declared(&trace, ui_rect, DISPLAY_HEADING) else {
        return Ok(Some(format!(
            "the Settings window declares no `{DISPLAY_HEADING}`. Headings: {}.",
            list(&declared_names(&trace, ui_rect, "settings.heading."))
        )));
    };
    driver
        .click_at(frame_of(&session, &trace, ui_rect, DISPLAY_HEADING)?.declared_center(heading))?;
    session.settle(20);
    // The colour sits low on a long page: roll the page body until Reset is
    // in view, as the operator would.
    for _ in 0..MAX_SCROLLS {
        let trace = session.trace()?;
        if declared(&trace, ui_rect, RESET).is_some() {
            break;
        }
        let Some(body) = declared(&trace, ui_rect, PAGE_BODY) else {
            break;
        };
        driver.scroll_at(
            frame_of(&session, &trace, ui_rect, PAGE_BODY)?.declared_center(body),
            -3,
        )?;
        session.settle(10);
    }
    let trace = session.trace()?;
    let Some(reset) = declared(&trace, ui_rect, RESET) else {
        return Ok(Some(format!(
            "the Display page is open with a non-default colour and declares no `{RESET}`: \
             Reset is not offered, or is scrolled out of view. Regions: {}.",
            list(&declared_names(&trace, ui_rect, "settings.display."))
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, RESET)?.declared_center(reset))?;
    session.settle(20);
    let trace = session.trace()?;
    if declared(&trace, ui_rect, RESET).is_some() {
        return Ok(Some(
            "Reset was pressed and is still drawn: the colour did not return to the default."
                .to_owned(),
        ));
    }
    let shot = ctx.out("ocr_colour_setting.settings.png");
    crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot);
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
    driver.click_at(frame_of(&session, &trace, ui_rect, SAVE)?.declared_center(save))?;
    session.settle(30);
    if session.trace()?.events(PREFS_SAVED).count() == saves {
        return Ok(Some(format!(
            "Save was pressed after Reset and no `{PREFS_SAVED}` line followed."
        )));
    }
    drop(session);
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
    let reset = tint(ctx, report, &exe, "reset")?;
    if reset.magenta == 0 || reset.green != 0 {
        return Ok(Some(format!(
            "the file says {DEFAULT_WRITTEN} and the next run drew {reset:?}: the saved colour \
             is not the one read back."
        )));
    }
    report.note("the planted colour is drawn, Reset writes the default, and the next run draws it");
    Ok(None)
}
