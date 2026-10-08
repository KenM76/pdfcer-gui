//! `remove_ocr` — **File ▸ Recognise ▸ Remove OCR text takes off every layer
//! pdfcer's OCR wrote, leaves a look-alike from other software, and a second
//! press says there is nothing left.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/remove_ocr.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose ribbon carries the File tab's Recognise group.
const MODE: &str = "read";
/// Two pages, one pdfcer layer each, and a decoy with another `/Producer`.
const FIXTURE: &str = "ocr-layers.pdf";
const TAB: &str = "ribbon.tab.file";
const ITEM: &str = "ribbon.item.file.remove_ocr";
/// Remove, in the window the item opens.
const COMMIT: &str = "remove-ocr.commit"; // ui-text-exempt: a trace region name, never displayed
/// `remove-ocr-layers-applied removed=N pages=M`, after the engine removed N.
const APPLIED_EVENT: &str = "remove-ocr-layers-applied";
/// The funnel's line for a press that wrote nothing.
const REFUSED_EVENT: &str = "remove-ocr-layers-refused";
/// The detail when no pdfcer layer is left.
const NOTHING_DETAIL: &str = "no pdfcer OCR layer in the document";

/// See the module documentation.
pub struct RemovingOcrTextReachesTheDocument;

impl Check for RemovingOcrTextReachesTheDocument {
    fn name(&self) -> &'static str {
        "remove_ocr"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Recognise ▸ Remove OCR text is offered and pressing it removes no layer, \
         removes text pdfcer did not write, or removes again on a second press"
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

/// Find the item on the File tab, following the overflow, and press it.
fn press(session: &Session, driver: &Driver, ui_rect: &str) -> Result<bool> {
    let Some(item) = driving::declared_or_in_overflow(session, driver, ui_rect, ITEM)? else {
        return Ok(false);
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(30);
    // A document holding pdfcer layers opens the window; one holding none
    // goes straight to the refusal.
    if let Some(commit) = driving::declared(&session.trace()?, ui_rect, COMMIT) {
        driver.click_at(session.frame()?.declared_center(commit))?;
        session.settle(30);
    }
    Ok(true)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the committed fixture is not at {}: a broken checkout.",
            pdf.display()
        )));
    }
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the ribbon.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("remove_ocr.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Read mode, File tab --------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    let trace = session.trace()?;
    let tab = driving::declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{TAB}` region in Read mode. Tabs declared: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    // --- 2: the first press removes pdfcer's two layers, not the decoy -----
    if !press(&session, &driver, ui_rect)? {
        return Ok(Some(format!(
            "no `{ITEM}` on the File tab or in its overflow: Remove OCR text is missing. Items \
             declared: {}. Trace: {}.",
            driving::list(&driving::declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.file."
            )),
            session.trace_path().display()
        )));
    }
    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        let why = trace
            .events(REFUSED_EVENT)
            .last()
            .map_or_else(|| "none".to_owned(), |l| l.raw.clone());
        return Ok(Some(format!(
            "Remove OCR text was pressed on a document with two pdfcer layers and no \
             `{APPLIED_EVENT}` line followed. Refusal: {why}. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("first press: `{}`", applied.raw));
    if applied.get("removed") != Some("2") || applied.get("pages") != Some("2") {
        return Ok(Some(format!(
            "the fixture holds two pdfcer layers on two pages and one decoy; the removal \
             reported another count: `{}`. Three means the decoy went too.",
            applied.raw
        )));
    }

    // --- 3: the second press finds nothing left ----------------------------
    let applied_before = trace.events(APPLIED_EVENT).count();
    if !press(&session, &driver, ui_rect)? {
        return Err(Error::new(format!(
            "`{ITEM}` was pressed once and could not be found for the second press. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let trace = session.trace()?;
    if trace.events(APPLIED_EVENT).count() != applied_before {
        return Ok(Some(format!(
            "a second press removed layers again, so the first left some behind. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some(refused) = trace.events(REFUSED_EVENT).last() else {
        return Ok(Some(format!(
            "a second press wrote no `{REFUSED_EVENT}` line: nothing tells the operator the \
             document has no OCR text left. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("second press: `{}`", refused.raw));
    if !refused.raw.contains(NOTHING_DETAIL) {
        return Ok(Some(format!(
            "the second press was refused for another reason than an empty document: `{}`.",
            refused.raw
        )));
    }
    report
        .note("both pdfcer layers came off in one press, through the engine, and the decoy stayed");
    Ok(None)
}
