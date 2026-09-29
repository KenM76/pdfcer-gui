//! `export_tables_without_the_mouse` — File ▸ Export ▸ Tables… writes the
//! ruled table in `fixtures/ruled-table.pdf` as CSV, as an Excel workbook and
//! as an OpenDocument spreadsheet, each chosen by clicking its radio, on a
//! window placed off the desktop and driven through the scripted pointer.
//! `export_tables_follows_the_tags` exports the unruled table of
//! `fixtures/tagged-report.pdf` as CSV and Excel, which only its tags yield.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_tables_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The line a successful export writes.
const WROTE: &str = "export-tables"; // ui-text-exempt: a trace event name, never displayed
/// The window's Export button.
const EXPORT: &str = "export-tables.export"; // ui-text-exempt: a trace region name, never displayed
/// The File tab and the ribbon item.
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.export_tables"; // ui-text-exempt: a trace region name, never displayed
/// The Export group when the band is too narrow to show it open.
const COLLAPSED: &str = "ribbon.group.file.export.collapsed"; // ui-text-exempt: a trace region name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// Each format: its radio key, and bytes the file must contain.
const FORMATS: &[(&str, &[u8])] = &[
    ("csv", b"Item,Qty,Mass"),
    ("xlsx", b"xl/worksheets/"),
    ("ods", b"application/vnd.oasis.opendocument.spreadsheet"),
];

pub struct ExportTablesWithoutTheMouse;

impl Check for ExportTablesWithoutTheMouse {
    fn name(&self) -> &'static str {
        "export_tables_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Export ▸ Tables… does not write a detected table in the format whose radio \
         was clicked — CSV, Excel or OpenDocument"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let run = Run {
            fixture: "ruled-table.pdf",
            stem: "export-tables",
            formats: FORMATS,
            expect: &[("structure", "untagged")],
        };
        match drive(ctx, &mut report, &run) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

pub struct ExportTablesFollowsTheTags;

impl Check for ExportTablesFollowsTheTags {
    fn name(&self) -> &'static str {
        "export_tables_follows_the_tags"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Export ▸ Tables… on a tagged PDF ignores the tables its tags state"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        // No rules are drawn, so table detection alone finds nothing.
        let run = Run {
            fixture: "tagged-report.pdf",
            stem: "export-tables-tagged",
            formats: &[("csv", b"Name,Qty"), ("xlsx", b"xl/worksheets/")],
            expect: &[("structure", "tree"), ("structure_fallback", "none")],
        };
        match drive(ctx, &mut report, &run) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// One file exported in several formats.
struct Run {
    fixture: &'static str,
    /// Names this run's artifacts.
    stem: &'static str,
    formats: &'static [(&'static str, &'static [u8])],
    /// Fields every `export-tables` line must carry.
    expect: &'static [(&'static str, &'static str)],
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
        .join(run.fixture);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    // One path for every export; it is removed before each, so a file left
    // by the previous format cannot pass for this one's.
    let target = ctx.out(&format!("{}.out", run.stem));

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

    // `declared` reads the ui-rect change log with its retirements, so a
    // popup item or a closed window's button is not returned once gone.
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

    for &(format, needle) in run.formats {
        let _ = std::fs::remove_file(&target);
        if target.exists() {
            return Err(Error::new(format!(
                "cannot clear {} before the {format} export.",
                target.display()
            )));
        }
        let before = session.trace()?.events(WROTE).count();
        click(TAB)?;
        // The 1,400-point window collapses the Export group; its popup holds
        // the item. The check follows whichever the band shows.
        if fresh(ITEM)?.is_none() && fresh(COLLAPSED)?.is_some() {
            click(COLLAPSED)?;
        }
        click(ITEM)?;
        click(&format!("export-tables.format.{format}"))?;
        click(EXPORT)?;
        session.settle(20);

        let trace = session.trace()?;
        let wrote: Vec<_> = trace.events(WROTE).collect();
        let Some(line) = wrote.get(before) else {
            return Ok(Some(format!(
                "{format}: Export was pressed and no `{WROTE}` line followed. Refused: {}. \
                 Failed: {}. Trace: {}.",
                trace
                    .last("export-tables-refused")
                    .map_or("none".to_owned(), |l| l.raw.clone()),
                trace
                    .last("export-tables-failed")
                    .map_or("none".to_owned(), |l| l.raw.clone()),
                session.trace_path().display()
            )));
        };
        if line.get("format") != Some(format) {
            return Ok(Some(format!(
                "the `{format}` radio was clicked and the export wrote `{}` — the click did \
                 not choose the format.",
                line.raw
            )));
        }
        if line.get("tables") != Some("1") {
            return Ok(Some(format!(
                "{} holds one table and the export reports `{}`.",
                run.fixture, line.raw
            )));
        }
        if let Some((key, want)) = run.expect.iter().find(|(k, v)| line.get(k) != Some(v)) {
            return Ok(Some(format!(
                "{format}: {} should export with `{key}={want}`; it traced `{}`.",
                run.fixture, line.raw
            )));
        }
        let bytes = std::fs::read(&target).map_err(|e| {
            Error::new(format!(
                "{format}: the export traced success and {} cannot be read: {e}",
                target.display()
            ))
        })?;
        if format != "csv" && !bytes.starts_with(b"PK\x03\x04") {
            return Ok(Some(format!(
                "{format}: {} is not a zip package, and both workbook formats are.",
                target.display()
            )));
        }
        if !bytes.windows(needle.len()).any(|w| w == needle) {
            return Ok(Some(format!(
                "{format}: {} does not contain `{}`.",
                target.display(),
                String::from_utf8_lossy(needle)
            )));
        }
        if format != "csv" && line.get("numbers").is_none_or(|n| n == "0") {
            return Ok(Some(format!(
                "{format}: the table holds unambiguous numbers and the export wrote \
                 none as a number: `{}`.",
                line.raw
            )));
        }
        report.note(format!("{format}: `{}`", line.raw));
    }
    pointer.gone(&session)?;
    Ok(None)
}
