//! `clicked_text_takes_an_opacity` — a text object of `fixtures/paragraph.pdf`
//! clicked in Edit shows its opacity in Properties; 40 typed into that field
//! is read back from the page as 0.400 and saved as a `/ca 0.4`. Scripted
//! pointer, window off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/text_opacity_scripted.md`.

use crate::checks::add_text_ink_scripted::fill_alphas;
use crate::checks::word_styles::{Driven, IN_WORD, launch_invoking};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit"; // ui-text-exempt: a command id, never displayed
const FIXTURE: &str = "paragraph.pdf";
const PROPERTIES_BODY: &str = "dock.body.file.properties"; // ui-text-exempt: a trace region name
const PROPERTIES_TAB: &str = "dock.tab.file.properties"; // ui-text-exempt: a trace region name
const FIELD: &str = "properties.textobject.opacity"; // ui-text-exempt: a trace region name
const SHOWN: &str = "text-opacity-shown"; // ui-text-exempt: a trace event name
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name
/// What is typed, and the value it must read back as.
const TYPED: &str = "40";
const READ_BACK: &str = "0.400";

/// See the module documentation.
pub struct ClickedTextTakesAnOpacity;

impl Check for ClickedTextTakesAnOpacity {
    fn name(&self) -> &'static str {
        "clicked_text_takes_an_opacity"
    }

    fn defect(&self) -> &'static str {
        "clicked page text offers no opacity, or the opacity typed is not what the page and the \
         saved file then carry"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Ok(note)) => {
                report.note(&note);
                report.pass()
            }
            Ok(Err(failure)) => report.fail(failure),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<std::result::Result<String, String>> {
    let (d, copy) = launch_invoking(ctx, report, "text-opacity", FIXTURE, INVOKE)?;
    let outcome = steps(&d, report, &copy);
    let parked = d.pointer.gone(&d.session);
    match outcome? {
        Err(failure) => Ok(Err(failure)),
        Ok(note) => parked.map(|_| Ok(note)),
    }
}

fn steps(
    d: &Driven,
    report: &mut CheckReport,
    copy: &std::path::Path,
) -> Result<std::result::Result<String, String>> {
    if !d.declares(PROPERTIES_BODY)? {
        d.click(PROPERTIES_TAB)?;
    }
    d.click_page(IN_WORD)?;
    let trace = d.session.trace()?;
    let Some(before) = trace.last(SHOWN) else {
        return Ok(Err(format!(
            "★★★ clicking the paragraph's text traced no `{SHOWN}`: the clicked text offers no \
             opacity. Trace: {}.",
            d.path()
        )));
    };
    report.note(&before.raw);
    // The fixture writes no `gs`, so its text is opaque and readable.
    if before.get("alpha") != Some("1.000") {
        return Ok(Err(format!(
            "★★ the fixture's text has no graphics state, so its opacity reads 1.000; the panel \
             read `{}`. Trace: {}.",
            before.raw,
            d.path()
        )));
    }
    if !d.declares(FIELD)? {
        return Ok(Err(format!(
            "★★★ the opacity was read but no `{FIELD}` field drew. Trace: {}.",
            d.path()
        )));
    }
    let mark = trace.mark();
    // A press on a drag field opens it with its text selected.
    d.click(FIELD)?;
    d.pointer.type_text(&d.session, None, TYPED)?;
    d.pointer.key(&d.session, None, "Enter", None)?;
    d.session.settle(40);
    let trace = d.session.trace()?;
    let after = trace.last_after(SHOWN, mark);
    report.note(format!("after: {:?}", after.map(|l| l.raw.clone())));
    if after.and_then(|l| l.get("alpha")) != Some(READ_BACK) {
        return Ok(Err(format!(
            "★★★ typing {TYPED} into the opacity field and Enter must re-read the text at \
             {READ_BACK}; the next `{SHOWN}` line is {:?}. Trace: {}.",
            after.map(|l| l.raw.clone()),
            d.path()
        )));
    }
    d.pointer.key(&d.session, None, "S", Some("ctrl"))?;
    d.session.settle(40);
    if !d
        .session
        .trace()?
        .events(SAVED)
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Ok(Err(format!(
            "Ctrl+S traced no `{SAVED} outcome=ok`. Trace: {}.",
            d.path()
        )));
    }
    let bytes = std::fs::read(copy)
        .map_err(|e| crate::error::Error::new(format!("reading the save: {e}")))?;
    let alphas = fill_alphas(&bytes);
    report.note(format!("/ca values in the file: {alphas:?}"));
    if !alphas.iter().any(|a| (a - 0.4).abs() < 1e-6) {
        return Ok(Err(format!(
            "★★★ the page re-read the text at {READ_BACK} but the saved file holds no `/ca 0.4` \
             (found {alphas:?})."
        )));
    }
    Ok(Ok(format!(
        "the clicked text read 1.000, took {TYPED} %, re-read as {READ_BACK} and saved `/ca 0.4`"
    )))
}
