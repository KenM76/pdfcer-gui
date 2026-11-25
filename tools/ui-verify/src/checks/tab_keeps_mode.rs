//! `each_tab_remembers_its_mode` — under Settings > Display > *Remember the
//! mode of each tab*, bringing a tab back restores the Read/Review/Edit mode
//! it was left in; with the setting off, the mode stays the window's. See
//! `docs/modules/ui-verify/checks/tab_keeps_mode.md`.

use crate::checks::driving;
use crate::checks::scripteddrive::{click, launch_with, userdata};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::sandbox::{reset_prefs, write_prefs};
use crate::trace::Trace;

const FIRST: &str = "four-pages.pdf";
const SECOND: &str = "drawing-sheets.pdf";
const OPEN_PATH_ENV: &str = "PDFCER_DIAG_OPEN_PATH"; // ui-text-exempt: an environment variable name
const KEY: &str = "remember_tab_mode"; // ui-text-exempt: a preferences file key
/// `mode-changed from=… to=<id> …`: the dock following the ribbon's mode.
const CHANGED: &str = "mode-changed"; // ui-text-exempt: a trace event name
const ACTIVATE: &str = "document-activate"; // ui-text-exempt: a trace event name
const FIRST_TAB: &str = "doc-tab.0"; // ui-text-exempt: a trace region name
/// The first document is left in Edit, the second in Review: neither is the
/// mode the program starts in, so a restore cannot be mistaken for a default.
const FIRST_MODE: &str = "edit";
const SECOND_MODE: &str = "review";

pub struct EachTabRemembersItsMode;

impl Check for EachTabRemembersItsMode {
    fn name(&self) -> &'static str {
        "each_tab_remembers_its_mode"
    }

    fn defect(&self) -> &'static str {
        "with the setting on, a tab brought back is shown in the mode of the tab left rather \
         than its own; or, with it off, switching tabs changes the mode"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = assess(ctx, &mut report);
        if let Ok(dir) = userdata(ctx) {
            let _ = reset_prefs(&dir);
        }
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The mode the last `mode-changed` line moved to.
fn mode(trace: &Trace) -> Option<String> {
    trace
        .last(CHANGED)
        .and_then(|l| l.get("to").map(str::to_owned))
}

/// Put the window in `want` through its mode segment and require the dock to
/// have followed.
fn switch(
    session: &crate::launch::Session,
    pointer: &crate::input::scripted::ScriptedPointer,
    want: &str,
) -> Result<()> {
    click(session, pointer, &format!("ribbon.mode.{want}"))?;
    session.settle(20);
    match mode(&session.trace()?) {
        Some(got) if got == want => Ok(()),
        got => Err(Error::new(format!(
            "clicking the {want} segment left the mode at {got:?}, so there is no mode to remember."
        ))),
    }
}

/// One launch with the setting at `on`: the mode the first tab comes back in.
fn drive(ctx: &CheckContext, report: &mut CheckReport, on: bool) -> Result<Option<String>> {
    write_prefs(&userdata(ctx)?, &format!("{KEY} = {on}\n"))
        .map_err(|e| Error::new(format!("could not plant the preferences: {e}")))?;
    let first = driving::repo_fixture(FIRST, "It is a committed fixture.")?;
    let second = driving::repo_fixture(
        SECOND,
        "Run `python fixtures/drawing-sheets.PROVENANCE.py`.",
    )?;
    let trace_name = format!("tab_keeps_mode.{on}.trace.txt");
    let (session, pointer) =
        launch_with(ctx, report, &first, &trace_name, &[(OPEN_PATH_ENV, second)])?;
    switch(&session, &pointer, FIRST_MODE)?;
    pointer.key(&session, None, "O", Some("ctrl"))?;
    session.settle(60);
    switch(&session, &pointer, SECOND_MODE)?;
    click(&session, &pointer, FIRST_TAB)?;
    session.settle(30);
    let trace = session.trace()?;
    match trace.last(ACTIVATE) {
        Some(line) if line.get("slot") == Some("0") => {}
        _ => {
            return Err(Error::new(format!(
                "clicking `{FIRST_TAB}` traced no `{ACTIVATE} slot=0`, so the first document \
                 was never brought back."
            )));
        }
    }
    let back = mode(&trace);
    report.note(format!("setting {on}: the first tab came back in {back:?}"));
    Ok(back)
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let off = drive(ctx, report, false)?;
    if off.as_deref() != Some(SECOND_MODE) {
        return Ok(Some(format!(
            "★ with the setting off, bringing the first tab back changed the mode from \
             {SECOND_MODE} to {off:?}."
        )));
    }
    let on = drive(ctx, report, true)?;
    if on.as_deref() != Some(FIRST_MODE) {
        return Ok(Some(format!(
            "★ with the setting on, the first tab was left in {FIRST_MODE} and came back in \
             {on:?}."
        )));
    }
    Ok(None)
}
