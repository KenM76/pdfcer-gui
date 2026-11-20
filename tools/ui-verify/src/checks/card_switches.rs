//! `card_switches` — **Settings ▸ Display opens one window holding every
//! graphics-card switch beside what it did on this machine, and its
//! switches write the same settings their first homes do.** Driven off the
//! desktop with the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/card_switches.md`.

use super::dimdrive::{Fixture, launch_with, press, ui_rect_event};
use crate::checks::driving::{declared_in, declared_names, list};
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "a1-titleblock.pdf",
    method: "It is checked in under fixtures/.",
    page: PageGeometry {
        width_pt: 2383.94,
        height_pt: 1683.78,
    },
};
const PREFS_FILE: &str = "preferences.txt";
const PAGES_KEY: &str = "draw_pages_on_graphics_card"; // ui-text-exempt: a preferences-file key
const PICTURE_KEY: &str = "model3d_card_picture"; // ui-text-exempt: a preferences-file key
const OPEN_SETTINGS: &str = "file.settings";
const HEADING: &str = "settings.heading.display";
const PAGE_BODY: &str = "settings.page"; // ui-text-exempt: a trace region name
/// `dialogs::settings::display::CARD_SWITCHES_REGION`.
const OPEN_BUTTON: &str = "settings.display.card_switches";
const WINDOW: &str = "dialog:card-switches";
const SWITCH: &str = "cards.switch.";
const STATUS: &str = "cards.status.";
const SHOWN: &str = "card-switches-shown"; // ui-text-exempt: a trace event name
const CHANGED: &str = "card-switch"; // ui-text-exempt: a trace event name
const CARD_STATUS: &str = "card-status"; // ui-text-exempt: a trace event name
const SAVE: &str = "dialog:settings.save";
const SAVED: &str = "settings-saved"; // ui-text-exempt: a trace event name
const MAX_SCROLLS: usize = 40;
/// A Display choice other than the default and the pages switch, pressed so
/// the draft is dirty and Save is enabled.
const OTHER: &str = "settings.display.overprint.pdfx";

/// See the module documentation.
pub struct CardSwitches;

impl Check for CardSwitches {
    fn name(&self) -> &'static str {
        "every_card_switch_is_in_one_window_beside_its_status"
    }

    fn defect(&self) -> &'static str {
        "Settings ▸ Display does not open the graphics card switches window, a row has no \
         status, the page row shows no outcome after a page is drawn, or a switch there does \
         not write the setting its first home writes (or Settings' Save puts it back)"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = drive(ctx, &mut report);
        if let Ok(dir) = userdata(ctx) {
            let _ = crate::sandbox::reset_prefs(&dir);
        }
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// `key`'s value in the sandbox's preferences file; `None` when absent.
fn written(ctx: &CheckContext, key: &str) -> Result<Option<String>> {
    let path = userdata(ctx)?.join(PREFS_FILE);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    Ok(text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key).then(|| v.trim().to_owned())
        }))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    crate::sandbox::write_prefs(&userdata(ctx)?, &format!("{PAGES_KEY} = true\n"))
        .map_err(|e| Error::new(format!("could not plant the preferences: {e}")))?;
    let ui_rect = ui_rect_event(ctx)?;
    let (session, pointer) =
        launch_with(ctx, report, &FIXTURE, OPEN_SETTINGS, "card_switches", &[])?;
    session.settle(60);
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, HEADING).is_none() {
        return Ok(Some(format!(
            "the Settings window declares no `{HEADING}`. Headings: {}.",
            list(&declared_names(&trace, ui_rect, "settings.heading."))
        )));
    }
    press(ctx, &session, &pointer, HEADING)?;
    if !reach(&session, &pointer, ui_rect, OPEN_BUTTON)? {
        return Ok(Some(format!(
            "★★★ Settings ▸ Display declares no `{OPEN_BUTTON}`: the card switches window \
             cannot be reached from Settings."
        )));
    }
    press(ctx, &session, &pointer, OPEN_BUTTON)?;
    session.settle(20);
    if let Some(failure) = judge_window(&session.trace()?, ui_rect, report) {
        return Ok(Some(failure));
    }
    if let Some(failure) = flip(ctx, &session, &pointer, report, "pages", PAGES_KEY, "false")? {
        return Ok(Some(failure));
    }
    if let Some(failure) = save_keeps(ctx, &session, &pointer)? {
        return Ok(Some(failure));
    }
    let trace = session.trace()?;
    if declared_in(&trace, ui_rect, &format!("{SWITCH}picture")).is_some() {
        if let Some(failure) = flip(ctx, &session, &pointer, report, "picture", PICTURE_KEY, "0")? {
            return Ok(Some(failure));
        }
    } else {
        report.note("this build has no 3D viewer: the 3D rows are not drawn");
    }
    let _ = pointer.gone(&session);
    drop(session);
    Ok(None)
}

/// The window opened, every row it counts has a switch and a status cell,
/// and the page row's outcome was recorded once a page was drawn.
fn judge_window(
    trace: &crate::trace::Trace,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Option<String> {
    let Some(shown) = trace.last(SHOWN) else {
        return Some(format!(
            "★★★ `{OPEN_BUTTON}` was pressed and no `{SHOWN}` line followed."
        ));
    };
    if declared_in(trace, ui_rect, WINDOW).is_none() {
        return Some(format!(
            "`{SHOWN}` was traced and no `{WINDOW}` region was declared."
        ));
    }
    let rows = shown.get_usize("rows").unwrap_or(0);
    let switches = declared_names(trace, ui_rect, SWITCH);
    let cells = declared_names(trace, ui_rect, STATUS);
    report.note(format!(
        "rows={rows}; switches {}; status {}",
        list(&switches),
        list(&cells)
    ));
    let has_3d = switches.iter().any(|s| s.ends_with("picture"));
    let drawn = if has_3d { rows } else { 1 };
    if switches.len() != drawn || cells.len() != drawn {
        return Some(format!(
            "★★★ the window counts {rows} paths and declares {} switch(es) and {} status \
             cell(s); every row is a switch beside its status.",
            switches.len(),
            cells.len()
        ));
    }
    let pages = trace
        .events(CARD_STATUS)
        .filter(|l| l.get("path") == Some("pages"))
        .last()
        .and_then(|l| l.get("outcome").map(str::to_owned));
    report.note(format!("page row's last outcome: {pages:?}"));
    match pages.as_deref() {
        Some("in-use" | "processor" | "limit") => None,
        other => Some(format!(
            "★★★ a page was drawn with the switch on and the page row's outcome is {other:?}: \
             its status beside the switch is not measured."
        )),
    }
}

/// Press `path`'s switch; it must trace the change and write `key = want`.
fn flip(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    path: &str,
    key: &str,
    want: &str,
) -> Result<Option<String>> {
    let before = session.trace()?.events(CHANGED).count();
    press(ctx, session, pointer, &format!("{SWITCH}{path}"))?;
    session.settle(10);
    let trace = session.trace()?;
    let change = trace.events(CHANGED).nth(before).map(|l| l.raw.clone());
    let after = written(ctx, key)?;
    report.note(format!("{path}: {change:?}; {key} = {after:?}"));
    if !change
        .as_deref()
        .is_some_and(|c| c.contains(&format!("path={path} on=false")))
    {
        return Ok(Some(format!(
            "★★★ `{SWITCH}{path}` was pressed and the change traced was {change:?}."
        )));
    }
    if after.as_deref() != Some(want) {
        return Ok(Some(format!(
            "★★★ `{SWITCH}{path}` was cleared and the preferences file says {key} = {after:?}; \
             `{want}` is what its first home writes."
        )));
    }
    Ok(None)
}

/// Roll the Settings page until `region` is declared in view.
fn reach(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
) -> Result<bool> {
    for _ in 0..MAX_SCROLLS {
        let trace = session.trace()?;
        if declared_in(&trace, ui_rect, region).is_some() {
            return Ok(true);
        }
        let Some((body, vp)) = declared_in(&trace, ui_rect, PAGE_BODY) else {
            return Ok(false);
        };
        pointer.wheel_in(session, vp.as_deref(), WindowPoint::centre_of(body), -3.0)?;
        session.settle(10);
    }
    Ok(declared_in(&session.trace()?, ui_rect, region).is_some())
}

/// Settings' Save, pressed after the window changed `draw_pages` and with
/// another Display setting changed so the draft has something to save,
/// keeps the window's value.
fn save_keeps(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    if !reach(session, pointer, ui_rect, OTHER)? {
        return Ok(Some(format!(
            "Settings ▸ Display declares no `{OTHER}` in view."
        )));
    }
    press(ctx, session, pointer, OTHER)?;
    session.settle(5);
    let saves = session.trace()?.events(SAVED).count();
    press(ctx, session, pointer, SAVE)?;
    session.settle(10);
    if session.trace()?.events(SAVED).count() == saves {
        return Ok(Some(format!(
            "Settings' `{SAVE}` was pressed and no `{SAVED}` followed."
        )));
    }
    let after = written(ctx, PAGES_KEY)?;
    if after.as_deref() != Some("false") {
        return Ok(Some(format!(
            "★★★ the card switches window cleared the pages switch, Settings was saved, and \
             the preferences file says {PAGES_KEY} = {after:?}: Settings' draft put the old \
             value back."
        )));
    }
    Ok(None)
}
