//! `export_tables_honours_its_choices` — the Export tables window's page
//! range, sheet grouping and number reading each reach the workbook writer,
//! and a workbook choice is remembered into the next window. Three exports of
//! `fixtures/two-ruled-tables.pdf`, on a window placed off the desktop and
//! driven through the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_tables_choices.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The line a successful export writes, and the one the window writes on opening.
const WROTE: &str = "export-tables"; // ui-text-exempt: a trace event name, never displayed
const OPENED: &str = "export-tables-open"; // ui-text-exempt: a trace event name, never displayed
const EXPORT: &str = "export-tables.export"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.export_tables"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.export.collapsed"; // ui-text-exempt: a trace region name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "two-ruled-tables.pdf";
/// The second worksheet's part name, stored uncompressed in the zip directory.
const SHEET2: &[u8] = b"xl/worksheets/sheet2.xml";

/// One gesture in the window before Export is pressed.
#[derive(Clone, Copy)]
enum Choice<'a> {
    Click(&'a str),
    /// Click the field `into`, then type `text` there.
    Type {
        into: &'a str,
        text: &'a str,
    },
}

/// One export: the gestures, and what its lines and file must show.
struct Run<'a> {
    stem: &'a str,
    choices: &'a [Choice<'a>],
    /// Fields the window's opening line must carry.
    opened: &'a [(&'a str, &'a str)],
    /// Fields the `export-tables` line must carry.
    expect: &'a [(&'a str, &'a str)],
    /// For an Excel file: whether a second worksheet must exist.
    sheet2: Option<bool>,
}

pub struct ExportTablesHonoursItsChoices;

impl Check for ExportTablesHonoursItsChoices {
    fn name(&self) -> &'static str {
        "export_tables_honours_its_choices"
    }

    fn defect(&self) -> &'static str {
        "Export tables writes every page whatever range is typed, or a sheet per table \
         whatever grouping is chosen, or keeps 1.234 as text when told how to read it, \
         or forgets the workbook choices"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        // The fixture's default workbook: two tables on two sheets, 4 numbers
        // and 4 ambiguous ones. Each run differs from it in one choice. The
        // runs share one profile, so each states every choice it relies on.
        let runs = [
            Run {
                stem: "export-tables-range",
                choices: &[
                    Choice::Click("export-tables.format.xlsx"),
                    Choice::Type {
                        into: "export-tables.pages.range",
                        text: "2",
                    },
                    Choice::Click("export-tables.sheets.table"),
                    Choice::Click("export-tables.numbers.auto"),
                ],
                opened: &[],
                expect: &[("tables", "1"), ("sheets", "1")],
                sheet2: Some(false),
            },
            Run {
                stem: "export-tables-single",
                choices: &[
                    Choice::Click("export-tables.format.xlsx"),
                    Choice::Click("export-tables.pages.all"),
                    Choice::Click("export-tables.sheets.single"),
                    Choice::Click("export-tables.numbers.auto"),
                ],
                opened: &[],
                expect: &[("tables", "2"), ("sheets", "1"), ("sheet_option", "single")],
                sheet2: Some(false),
            },
            Run {
                stem: "export-tables-european",
                choices: &[
                    Choice::Click("export-tables.format.ods"),
                    Choice::Click("export-tables.pages.all"),
                    Choice::Click("export-tables.sheets.table"),
                    Choice::Click("export-tables.numbers.european"),
                ],
                // The previous run's grouping, remembered.
                opened: &[("sheets", "single"), ("format", "xlsx")],
                expect: &[
                    ("tables", "2"),
                    ("sheets", "2"),
                    ("numbers", "8"),
                    ("ambiguous", "0"),
                ],
                sheet2: None,
            },
        ];
        let mut failures = Vec::new();
        for run in &runs {
            match drive(ctx, &mut report, run) {
                Ok(Some(failure)) => failures.push(format!("{}: {failure}", run.stem)),
                Ok(None) => {}
                Err(why) => failures.push(format!("{}: {why}", run.stem)),
            }
        }
        if failures.is_empty() {
            report.pass()
        } else {
            report.fail(failures.join(" | "))
        }
    }
}

/// The first field of `expect` that `line` does not carry, as a sentence.
fn missing(what: &str, line: &crate::trace::TraceLine, expect: &[(&str, &str)]) -> Option<String> {
    expect
        .iter()
        .find(|(k, v)| line.get(k) != Some(v))
        .map(|(key, want)| {
            format!(
                "the {what} line should carry `{key}={want}`: `{}`.",
                line.raw
            )
        })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport, run: &Run) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let target = ctx.out(&format!("{}.out", run.stem));
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the export.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{}-scripted.trace.txt", run.stem)));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("{}-scripted.pointer.txt", run.stem)),
    )?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let fresh = |region: &str| -> Result<Option<crate::geom::LRect>> {
        Ok(declared(&session.trace()?, ui_rect, region))
    };
    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };

    click(TAB)?;
    if fresh(ITEM)?.is_none() && fresh(COLLAPSED)?.is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(20);
    if fresh(EXPORT)?.is_none() {
        return Ok(Some(format!(
            "Tables… opened no window with an Export button. Trace: {}.",
            session.trace_path().display()
        )));
    }
    for choice in run.choices {
        match *choice {
            Choice::Click(region) => click(region)?,
            Choice::Type { into, text } => {
                click(into)?;
                let trace = session.trace()?;
                let viewport = declared_in(&trace, ui_rect, into).and_then(|(_, vp)| vp);
                pointer.type_text(&session, viewport.as_deref(), text)?;
                session.settle(10);
            }
        }
    }
    click(EXPORT)?;
    session.settle(20);
    pointer.gone(&session)?;

    let trace = session.trace()?;
    if !run.opened.is_empty() {
        let Some(opened) = trace.last(OPENED) else {
            return Ok(Some(format!("the window wrote no `{OPENED}` line.")));
        };
        if let Some(failure) = missing("window's opening", opened, run.opened) {
            return Ok(Some(failure));
        }
    }
    let Some(line) = trace.last(WROTE) else {
        return Ok(Some(format!(
            "Export was pressed and no `{WROTE}` line followed. Requested: {}. Refused: {}. \
             Failed: {}. Trace: {}.",
            trace
                .last("export-tables-requested")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            trace
                .last("export-tables-refused")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            trace
                .last("export-tables-failed")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    if let Some(failure) = missing("export", line, run.expect) {
        return Ok(Some(failure));
    }
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the export traced success and {} cannot be read: {e}",
            target.display()
        ))
    })?;
    if !bytes.starts_with(b"PK\x03\x04") {
        return Ok(Some(format!(
            "{} is not a zip package, and both workbook formats are.",
            target.display()
        )));
    }
    if let Some(want) = run.sheet2
        && bytes.windows(SHEET2.len()).any(|w| w == SHEET2) != want
    {
        return Ok(Some(format!(
            "{} {} a second worksheet, and the trace said `{}`.",
            target.display(),
            if want { "lacks" } else { "holds" },
            line.raw
        )));
    }
    report.artifact(target);
    report.note(format!("{}: `{}`", run.stem, line.raw));
    Ok(None)
}
