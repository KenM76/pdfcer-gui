//! `header_footer_and_printer_marks` — Pages ▸ Stamp ▸ Header & footer… and
//! File ▸ Print ▸ Printer's marks… on `fixtures/four-pages.pdf`, each with
//! nothing picked in the rail, reach all four pages as one edit each. The
//! window is placed off the desktop and driven through the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/pagestamps_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The header-and-footer engine call's outcome line, and its funnel line.
const RUNNING: &str = "running-applied"; // ui-text-exempt: a trace event name, never displayed
const RUNNING_EDIT: &str = "running-stamped"; // ui-text-exempt: a trace event name, never displayed
/// The printer's-marks engine call's outcome line, and its funnel line.
const MARKS: &str = "printmarks-applied"; // ui-text-exempt: a trace event name, never displayed
const MARKS_EDIT: &str = "printer-marks"; // ui-text-exempt: a trace event name, never displayed
/// Pages is not a Read-mode tab.
const MODE: &str = "ribbon.mode.review"; // ui-text-exempt: a trace region name, never displayed
const PAGES: &str = "ribbon.tab.pages"; // ui-text-exempt: a trace region name, never displayed
const FILE: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const RUNNING_ITEM: &str = "ribbon.item.pages.header_footer"; // ui-text-exempt: a trace region name, never displayed
const MARKS_ITEM: &str = "ribbon.item.file.printer_marks"; // ui-text-exempt: a trace region name, never displayed
/// The groups when the band is too narrow to show them open.
const STAMP_COLLAPSED: &str = "ribbon.group.pages.stamp.collapsed"; // ui-text-exempt: a trace region name, never displayed
const PRINT_COLLAPSED: &str = "ribbon.group.file.print.collapsed"; // ui-text-exempt: a trace region name, never displayed
const RUNNING_ADD: &str = "running.add"; // ui-text-exempt: a trace region name, never displayed
const MARKS_ADD: &str = "printmarks.add"; // ui-text-exempt: a trace region name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

pub struct HeaderFooterAndPrinterMarks;

impl Check for HeaderFooterAndPrinterMarks {
    fn name(&self) -> &'static str {
        "header_footer_and_printer_marks"
    }

    fn defect(&self) -> &'static str {
        "Header & footer or Printer's marks does not reach every page of the document as one edit"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join("four-pages.pdf");
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the four-page fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let (session, pointer) = launch(ctx, report, pdf)?;
    let press = |item: &str, collapsed: &str| -> Result<()> {
        let drawn = |region: &str| -> Result<bool> {
            Ok(declared(&session.trace()?, ui_rect, region).is_some())
        };
        if !drawn(item)? && drawn(collapsed)? {
            pointer.click_region(&session, ui_rect, collapsed)?;
        }
        pointer.click_region(&session, ui_rect, item)
    };
    pointer.click_region(&session, ui_rect, MODE)?;
    pointer.click_region(&session, ui_rect, PAGES)?;
    press(RUNNING_ITEM, STAMP_COLLAPSED)?;
    pointer.click_region(&session, ui_rect, RUNNING_ADD)?;
    session.settle(20);
    pointer.click_region(&session, ui_rect, FILE)?;
    press(MARKS_ITEM, PRINT_COLLAPSED)?;
    pointer.click_region(&session, ui_rect, MARKS_ADD)?;
    session.settle(20);
    pointer.gone(&session)?;
    verdict(&session, report)
}

/// Launch off the desktop with the scripted pointer attached.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: std::path::PathBuf,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("pagestamps-scripted.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("pagestamps-scripted.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((session, pointer))
}

/// Both stamps reached all four pages, each through the edit funnel.
fn verdict(session: &Session, report: &mut CheckReport) -> Result<Option<String>> {
    let trace = session.trace()?;
    let funnel = |tag: &str| {
        trace
            .last(&format!("{tag}-refused"))
            .or_else(|| trace.last(tag))
            .map_or("none".to_owned(), |l| l.raw.clone())
    };
    let Some(running) = trace.last(RUNNING) else {
        return Ok(Some(format!(
            "Add was clicked in Header & footer and no `{RUNNING}` line followed. Funnel: {}. Trace: {}.",
            funnel(RUNNING_EDIT),
            session.trace_path().display()
        )));
    };
    // Nothing picked means the whole document; the default line is a footer
    // reading "Page 1 of 4" on the first page.
    if running.get("n") != Some("4")
        || running.get("kind") != Some("Footer")
        || running.get("first_label") != Some("Page_1_of_4")
    {
        return Ok(Some(format!(
            "four pages with nothing picked should get the default footer, the first reading `Page 1 of 4`; the stamp traced `{}`.",
            running.raw
        )));
    }
    if trace.last(RUNNING_EDIT).is_none() {
        return Ok(Some(format!(
            "the engine traced `{}` and the edit funnel wrote no `{RUNNING_EDIT}` line.",
            running.raw
        )));
    }
    let Some(marks) = trace.last(MARKS) else {
        return Ok(Some(format!(
            "Add marks was clicked and no `{MARKS}` line followed. Funnel: {}. Trace: {}.",
            funnel(MARKS_EDIT),
            session.trace_path().display()
        )));
    };
    let grown = marks
        .get("grown_pt")
        .and_then(|g| g.parse::<f64>().ok())
        .unwrap_or(0.0);
    // The default marks are 6 pt from the trim and 18 pt long: the sheet must
    // grow by at least their sum on each side.
    if marks.get("n") != Some("4") || grown < 24.0 {
        return Ok(Some(format!(
            "four pages should get every default mark and grow by at least 24 pt; the marks traced `{}`.",
            marks.raw
        )));
    }
    if trace.last(MARKS_EDIT).is_none() {
        return Ok(Some(format!(
            "the engine traced `{}` and the edit funnel wrote no `{MARKS_EDIT}` line.",
            marks.raw
        )));
    }
    report.note(format!("`{}`", running.raw));
    report.note(format!("`{}`", marks.raw));
    Ok(None)
}
