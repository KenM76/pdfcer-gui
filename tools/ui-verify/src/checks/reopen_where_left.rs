//! `a_document_reopens_where_it_was_left` — under Settings > Display >
//! *Reopen each document at the page, position and zoom it was left at*,
//! closing a document and opening it again returns to the place it was left;
//! with the setting off, it opens where any document opens. See
//! `docs/modules/ui-verify/checks/reopen_where_left.md`.

use crate::checks::scripteddrive::{launch_with, userdata};
use crate::checks::tab_keeps_position::{position, scroll};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::sandbox::{reset_prefs, write_prefs};

const DOCUMENT: &str = "four-pages.pdf";
const OPEN_PATH_ENV: &str = "PDFCER_DIAG_OPEN_PATH"; // ui-text-exempt: an environment variable name
const KEY: &str = "reopen_where_left"; // ui-text-exempt: a preferences file key
/// The store the setting writes, beside the preferences.
const STORE: &str = "last-view.txt"; // ui-text-exempt: a file name
const CLOSED: &str = "close"; // ui-text-exempt: a trace event name
const RESTORED: &str = "last-view-restored"; // ui-text-exempt: a trace event name
/// Wheel lines before the close: far enough to leave the opening position.
const LINES: f32 = -9.0;
/// Screen pixels the reopened view may differ from the one left.
const TOLERANCE: f64 = 2.0;

pub struct ADocumentReopensWhereItWasLeft;

impl Check for ADocumentReopensWhereItWasLeft {
    fn name(&self) -> &'static str {
        "a_document_reopens_where_it_was_left"
    }

    fn defect(&self) -> &'static str {
        "with the setting on, a document opened again starts at its first page rather than \
         where it was closed; or, with it off, it does not start where any document starts"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = assess(ctx, &mut report);
        if let Ok(dir) = userdata(ctx) {
            let _ = reset_prefs(&dir);
            let _ = std::fs::remove_file(dir.join(STORE));
        }
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Where a view was, in screen pixels from its acting page's corner.
type At = (f64, f64);

fn distance(a: At, b: At) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// One launch with the setting at `on`: where the document opened, where it
/// was left, and where it came back.
fn drive(ctx: &CheckContext, report: &mut CheckReport, on: bool) -> Result<(At, At, At)> {
    let dir = userdata(ctx)?;
    let _ = std::fs::remove_file(dir.join(STORE));
    write_prefs(&dir, &format!("{KEY} = {on}\n"))
        .map_err(|e| Error::new(format!("could not plant the preferences: {e}")))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = crate::checks::driving::repo_fixture(DOCUMENT, "It is a committed fixture.")?;
    let trace_name = format!("reopen_where_left.{on}.trace.txt");
    let open_path = pdf.clone();
    let (session, pointer) = launch_with(
        ctx,
        report,
        &pdf,
        &trace_name,
        &[(OPEN_PATH_ENV, open_path)],
    )?;
    let opened = position(&session.trace()?)?;
    let left = scroll(&session, &pointer, ui_rect, LINES)?;
    if distance(opened, left) <= TOLERANCE {
        return Err(Error::new(
            "the wheel did not move the document, so there is no position to return to.",
        ));
    }
    pointer.key(&session, None, "W", Some("ctrl"))?;
    session.settle(30);
    if session.trace()?.last(CLOSED).and_then(|l| l.get("slot")) != Some("0") {
        return Err(Error::new(format!(
            "Ctrl+W traced no `{CLOSED} slot=0`, so the document was never closed."
        )));
    }
    pointer.key(&session, None, "O", Some("ctrl"))?;
    session.settle(60);
    let trace = session.trace()?;
    let back = position(&trace)?;
    let restored = trace.last(RESTORED).map(|l| l.raw.clone());
    report.note(format!(
        "setting {on}: opened at {opened:?}, left at {left:?}, came back at {back:?}; \
         restore line {restored:?}"
    ));
    Ok((opened, left, back))
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (opened, _, back) = drive(ctx, report, false)?;
    if distance(back, opened) > TOLERANCE {
        return Ok(Some(format!(
            "★ with the setting off, the document opened at {opened:?} the first time and at \
             {back:?} the second."
        )));
    }
    let (_, left, back) = drive(ctx, report, true)?;
    let missed = distance(back, left);
    if missed > TOLERANCE {
        return Ok(Some(format!(
            "★ with the setting on, the document was left at {left:?} and came back at \
             {back:?}, {missed:.1} px away."
        )));
    }
    Ok(None)
}
