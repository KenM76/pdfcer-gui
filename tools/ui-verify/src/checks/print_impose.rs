//! `print_several_pages_per_sheet_and_booklets` — the Print dialog's
//! Multiple and Booklet layouts reach the plan sent to the printer.
//!
//! Driven off the desktop with the scripted pointer: `file.print` is rung on
//! `print-impose.pdf`, five Letter pages. The check clicks Multiple (two
//! pages per sheet by default), then Booklet. Oracle, on the `print-plan`
//! line after each click:
//!
//! - Multiple: `impose=nup`, a two-cell grid, `imposed_sheets=3` and
//!   `sheets=Some(3)` — the plan is one entry per sheet, not per page.
//! - Booklet: `impose=booklet`, `booklet_blanks=3`, `booklet_sheets=2` and
//!   `imposed_sheets=4` — five pages pad to eight, folded onto two sheets of
//!   two sides each. Neither count is reachable without the layout: one page
//!   per sheet gives five.
//!   `impose_duplex=short-edge` is the duplex sent: a booklet printed both
//!   sides any other way comes out of order.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "print-impose.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/print-impose.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const OPEN: &str = "print-open";
const PLAN: &str = "print-plan";
const PREFIX: &str = "print.impose.";

/// See the module documentation.
pub struct PrintSeveralPagesPerSheetAndBooklets;

impl Check for PrintSeveralPagesPerSheetAndBooklets {
    fn name(&self) -> &'static str {
        "print_several_pages_per_sheet_and_booklets"
    }

    fn defect(&self) -> &'static str {
        "the Print dialog's Multiple or Booklet choice does not change what is sent to the \
         printer: it still prints one page per sheet"
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_impose.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_impose.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Click `print.impose.<choice>` and return the last `print-plan` line.
fn choose(session: &Session, pointer: &ScriptedPointer, choice: &str) -> Result<String> {
    let name = format!("{PREFIX}{choice}");
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, &name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region. Declared under `{PREFIX}`: {}.",
            list(&declared_names(&trace, UI_RECT, PREFIX))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(60);
    let trace = session.trace()?;
    trace
        .last(PLAN)
        .map(|l| l.raw.clone())
        .ok_or_else(|| Error::new(format!("no `{PLAN}` line after clicking `{name}`.")))
}

/// The fields `line` must carry, as `key=value`, or the failure naming them.
fn expect(choice: &str, line: &str, fields: &[&str]) -> Option<String> {
    let missing: Vec<&str> = fields
        .iter()
        .copied()
        .filter(|f| !line.split_whitespace().any(|w| w == *f))
        .collect();
    (!missing.is_empty()).then(|| {
        format!(
            "★★★ after clicking {choice}, the plan lacks {}: `{line}`.",
            missing.join(", ")
        )
    })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    let nup = choose(&session, &pointer, "multiple")?;
    report.note(nup.clone());
    let grid_ok = nup.contains("nup_grid=2x1") || nup.contains("nup_grid=1x2");
    if let Some(failure) = expect(
        "Multiple",
        &nup,
        &["impose=nup", "imposed_sheets=3", "sheets=Some(3)"],
    ) {
        return Ok(Some(failure));
    }
    if !grid_ok {
        return Ok(Some(format!(
            "★★★ Multiple at two pages per sheet laid out a grid other than two cells: `{nup}`."
        )));
    }
    let booklet = choose(&session, &pointer, "booklet")?;
    report.note(booklet.clone());
    if let Some(failure) = expect(
        "Booklet",
        &booklet,
        &[
            "impose=booklet",
            "booklet_blanks=3",
            "booklet_sheets=2",
            "imposed_sheets=4",
            "sheets=Some(4)",
            "impose_duplex=short-edge",
        ],
    ) {
        return Ok(Some(failure));
    }
    pointer.gone(&session)?;
    Ok(None)
}
