//! `settings_text_size` — **the Settings window draws its text larger than
//! the program by default, Ctrl+wheel over it and the slider at the far
//! right of its search row change that size alone, and the size is kept the
//! moment it changes.** Driven off the desktop with the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/settings_text_size.md`.

use super::dimdrive::{Fixture, launch_with, press, ui_rect_event};
use crate::checks::driving::{declared, declared_in, declared_names, list};
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::{ScriptedPointer, xy};
use crate::launch::Session;
use crate::report::CheckReport;

const PLAIN: Fixture = Fixture {
    file: "checker-image.pdf",
    method: "Rebuild it with `python fixtures/checker-image.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const KEY: &str = "settings_text_scale"; // ui-text-exempt: a preferences-file key
const PREFS_FILE: &str = "preferences.txt";
const OPEN_SETTINGS: &str = "file.settings";
const DIALOG: &str = "dialog:settings";
const CANCEL: &str = "dialog:settings.cancel";
/// `dialogs::settings::textsize::REGION_TEXT_SIZE`.
const SLIDER: &str = "settings.text_size";
/// `dialogs::settings::textsize::REGION_TEXT_SIZE_LABEL`.
const LABEL: &str = "settings.text_size.label";
/// `dialogs::settings::nav::REGION_SEARCH`.
const SEARCH: &str = "settings.search";
const SCALE: &str = "settings-text-scale"; // ui-text-exempt: a trace event name
const SMALL: &str = "settings-small-text"; // ui-text-exempt: a trace event name
/// The shipped scale, `prefs::settingstext::DEFAULT`.
const DEFAULT: f64 = 1.25;
/// The largest, `prefs::settingstext::MAX`, which the slider's right end is.
const MAX: f64 = 2.5;

/// See the module documentation.
pub struct SettingsTextSize;

impl Check for SettingsTextSize {
    fn name(&self) -> &'static str {
        "settings_text_size"
    }

    fn defect(&self) -> &'static str {
        "the Settings window's text is not larger than the program's by default, \
         Ctrl+wheel or its text-size slider does not change it, the change reaches the \
         main window, or the size is not kept the moment it changes"
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

/// The key's value in the sandbox's preferences file.
fn written(ctx: &CheckContext) -> Result<Option<f64>> {
    let path = userdata(ctx)?.join(PREFS_FILE);
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(None);
    };
    Ok(text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == KEY).then(|| v.trim().parse().ok())?
        }))
}

/// The last `scale=` and the last small-text `pts=` the window traced.
fn sizes(session: &Session) -> Result<(Option<f64>, Option<f64>)> {
    let trace = session.trace()?;
    let num = |event: &str, key: &str| {
        trace
            .last(event)
            .and_then(|l| l.get(key))
            .and_then(|v| v.parse::<f64>().ok())
    };
    Ok((num(SCALE, "scale"), num(SMALL, "pts")))
}

/// Every main-window region a prefix names, with its rect, in a stable order.
fn main_window(session: &Session, ui_rect: &str) -> Result<Vec<(String, LRect)>> {
    let trace = session.trace()?;
    let mut out = Vec::new();
    for name in declared_names(&trace, ui_rect, "ribbon.") {
        if let Some((rect, None)) = declared_in(&trace, ui_rect, &name) {
            out.push((name, rect));
        }
    }
    Ok(out)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    if written(ctx)?.is_some() {
        return Ok(Some(format!(
            "the sandbox's {PREFS_FILE} already holds {KEY}, so the shipped default could \
             not be measured."
        )));
    }
    let (session, pointer) = launch_with(
        ctx,
        report,
        &PLAIN,
        OPEN_SETTINGS,
        "settings_text_size",
        &[],
    )?;
    if declared_in(&session.trace()?, ui_rect, DIALOG).is_none() {
        return Err(Error::new(format!(
            "`{OPEN_SETTINGS}` was invoked and no `{DIALOG}` appeared."
        )));
    }
    let main_before = main_window(&session, ui_rect)?;
    let (scale0, pts0) = sizes(&session)?;
    report.note(format!("opened: scale {scale0:?}, small text {pts0:?} pt"));
    if scale0.is_none_or(|s| (s - DEFAULT).abs() > 1e-3) {
        return Ok(Some(format!(
            "the Settings window opened at scale {scale0:?}; expected the shipped {DEFAULT}."
        )));
    }
    if let Some(failure) = wheel_step(ctx, report, (&session, &pointer), pts0)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = slider_end(ctx, report, (&session, &pointer), pts0)? {
        return Ok(Some(failure));
    }
    let main_after = main_window(&session, ui_rect)?;
    report.note(format!(
        "main window: {} ribbon regions before, {} after",
        main_before.len(),
        main_after.len()
    ));
    if main_before.is_empty() || main_before != main_after {
        return Ok(Some(format!(
            "the main window's ribbon moved or vanished while the Settings window's text \
             grew: {} regions before, {} after; expected the same regions in the same \
             places. Before: {}.",
            main_before.len(),
            main_after.len(),
            list(
                &main_before
                    .iter()
                    .map(|(n, _)| n.clone())
                    .collect::<Vec<_>>()
            )
        )));
    }
    press(ctx, &session, &pointer, CANCEL)?;
    let kept = written(ctx)?;
    report.note(format!("after Cancel: {KEY} = {kept:?}"));
    if kept.is_none_or(|k| (k - MAX).abs() > 1e-3) {
        return Ok(Some(format!(
            "after Cancel the preferences file says {KEY} = {kept:?}; expected {MAX}, the \
             size the slider set, kept the moment it changed."
        )));
    }
    let _ = pointer.gone(&session);
    Ok(None)
}

/// One Ctrl+wheel notch away from the operator over the search box: one
/// step larger, written to the file at once, the text drawn larger.
fn wheel_step(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    pts0: Option<f64>,
) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    let Some((search, vp)) = declared_in(&session.trace()?, ui_rect, SEARCH) else {
        return Ok(Some(format!("the Settings window declares no `{SEARCH}`.")));
    };
    let at = WindowPoint::centre_of(search);
    let vp = vp.map_or(String::new(), |v| format!(" vp={v}"));
    pointer.send(session, &format!("wheel {} 1 mods=ctrl{vp}", xy(at)))?;
    session.settle(20);
    let (scale, pts) = sizes(session)?;
    let file = written(ctx)?;
    report.note(format!(
        "Ctrl+wheel: scale {scale:?}, small text {pts:?} pt, file {file:?}"
    ));
    let grew = matches!((pts, pts0), (Some(a), Some(b)) if a > b);
    if scale.is_none_or(|s| (s - 1.3).abs() > 1e-3)
        || file.is_none_or(|f| (f - 1.3).abs() > 1e-3)
        || !grew
    {
        return Ok(Some(format!(
            "one Ctrl+wheel notch over the Settings window traced scale {scale:?} and small \
             text {pts:?} pt against {pts0:?}, and the file says {file:?}; expected 1.3 in \
             both, written without Save, and larger text."
        )));
    }
    Ok(None)
}

/// Press the slider's right end: the largest size, written at once.
fn slider_end(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    pts0: Option<f64>,
) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    let trace = session.trace()?;
    let Some((slider, vp)) = declared_in(&trace, ui_rect, SLIDER) else {
        return Ok(Some(format!(
            "the Settings window declares no `{SLIDER}`. Regions: {}.",
            list(&declared_names(&trace, ui_rect, "settings."))
        )));
    };
    let Some((body, _)) = declared_in(&trace, ui_rect, DIALOG) else {
        return Ok(Some(format!("no `{DIALOG}` rect to place the slider in.")));
    };
    let right = body.max.x - slider.max.x;
    report.note(format!(
        "slider {slider:?}, {right:.0} pt from the window's right"
    ));
    let search = declared(&trace, ui_rect, SEARCH);
    let label = declared(&trace, ui_rect, LABEL);
    let ordered = matches!((search, label), (Some(s), Some(l))
        if s.max.x <= l.min.x && l.max.x <= slider.min.x);
    if !ordered || right > 40.0 {
        return Ok(Some(format!(
            "the search row is not search box, label, slider at the far right: slider \
             {right:.0} pt from the window's edge, search box {search:?}, label {label:?}."
        )));
    }
    let at = Pt {
        x: slider.max.x - 1.0,
        y: (slider.min.y + slider.max.y) / 2.0,
    };
    pointer.click_in(
        session,
        vp.as_deref(),
        WindowPoint::centre_of(LRect { min: at, max: at }),
    )?;
    session.settle(20);
    let (scale, pts) = sizes(session)?;
    let file = written(ctx)?;
    report.note(format!(
        "slider end: scale {scale:?}, small text {pts:?} pt, file {file:?}"
    ));
    let doubled = matches!((pts, pts0), (Some(a), Some(b)) if a > b * 1.8);
    if scale.is_none_or(|s| (s - MAX).abs() > 1e-3)
        || file.is_none_or(|f| (f - MAX).abs() > 1e-3)
        || !doubled
    {
        return Ok(Some(format!(
            "pressing the slider's right end traced scale {scale:?} and small text {pts:?} \
             pt against {pts0:?}, and the file says {file:?}; expected {MAX} in both and \
             text 1.8 times the opening size."
        )));
    }
    Ok(None)
}
