//! `extract_pages_keeps_or_drops_the_labels` — Pages ▸ Extract… opens a
//! window whose page range, *Keep the page labels* and *Delete these pages
//! afterwards* each reach the written file. Driven on a window placed off the
//! desktop through the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/extract_pages_labels.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The line a written extraction leaves.
const WROTE: &str = "extract"; // ui-text-exempt: a trace event name, never displayed
/// The line a page delete leaves once the engine ran.
const DELETED: &str = "delete-pages"; // ui-text-exempt: a trace event name, never displayed
const TAB: &str = "ribbon.tab.pages"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.pages.extract"; // ui-text-exempt: a trace region name, never displayed
const MODE: &str = "ribbon.mode.review"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.pages.organise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const EXTRACT: &str = "extract-pages.extract"; // ui-text-exempt: a trace region name, never displayed
const RANGE: &str = "extract-pages.pages.range"; // ui-text-exempt: a trace region name, never displayed
const LABELS: &str = "extract-pages.labels"; // ui-text-exempt: a trace region name, never displayed
const DELETE_AFTER: &str = "extract-pages.delete_after"; // ui-text-exempt: a trace region name, never displayed

pub struct ExtractPagesKeepsOrDropsTheLabels;

impl Check for ExtractPagesKeepsOrDropsTheLabels {
    fn name(&self) -> &'static str {
        "extract_pages_keeps_or_drops_the_labels"
    }

    fn defect(&self) -> &'static str {
        "Pages ▸ Extract… writes pages without the labels they show, or keeps the labels \
         when told to drop them, or ignores the typed range, or does not delete the \
         extracted pages when asked to"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let runs: [Run; 3] = [
            Run {
                fixture: "labelled-pages.pdf",
                stem: "extract-keep",
                untick: &[],
                expect: &[
                    ("pages", "2"),
                    ("asked", "2"),
                    ("labels", "keep"),
                    ("labels_dropped", "0"),
                    ("label_ranges", "1"),
                ],
                labelled: true,
                deletes: false,
            },
            Run {
                fixture: "labelled-pages.pdf",
                stem: "extract-drop-delete",
                untick: &[LABELS, DELETE_AFTER],
                expect: &[
                    ("pages", "2"),
                    ("labels", "drop"),
                    ("labels_dropped", "1"),
                    ("label_ranges", "0"),
                ],
                labelled: true,
                deletes: true,
            },
            Run {
                fixture: "pure-k-square.pdf",
                stem: "extract-unlabelled",
                untick: &[],
                expect: &[
                    ("pages", "1"),
                    ("labels_dropped", "0"),
                    ("label_ranges", "0"),
                ],
                labelled: false,
                deletes: false,
            },
        ];
        let mut failures = Vec::new();
        for run in &runs {
            match drive(ctx, &mut report, run) {
                Ok(Some(failure)) => failures.push(failure),
                Ok(None) => {}
                Err(why) => failures.push(why.to_string()),
            }
        }
        if failures.is_empty() {
            report.pass()
        } else {
            report.fail(failures.join(" | "))
        }
    }
}

/// One launch: the fixture, its artifact stem, the boxes clicked, the fields
/// required on the `extract` line, whether the labels box should be drawn and
/// whether a delete should follow.
struct Run {
    fixture: &'static str,
    stem: &'static str,
    untick: &'static [&'static str],
    expect: &'static [(&'static str, &'static str)],
    labelled: bool,
    deletes: bool,
}

/// Extracts the fixture's first two pages (its only page when it has one)
/// and judges the run. `Err` is a harness failure.
#[allow(clippy::too_many_lines)]
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
    let target = ctx.out(&format!("{}.pdf", run.stem));
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the extraction.",
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

    click(MODE)?;
    click(TAB)?;
    if fresh(ITEM)?.is_none() && fresh(COLLAPSED)?.is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(20);
    if fresh(EXTRACT)?.is_none() {
        return Ok(Some(format!(
            "Extract… opened no window with an Extract button. Trace: {}.",
            session.trace_path().display()
        )));
    }
    if fresh(LABELS)?.is_some() != run.labelled {
        return Ok(Some(format!(
            "{}: the Keep-the-page-labels box is {} and the file {} page labels. Trace: {}.",
            run.fixture,
            if run.labelled { "missing" } else { "drawn" },
            if run.labelled { "has" } else { "has no" },
            session.trace_path().display()
        )));
    }
    if run.labelled {
        click(RANGE)?;
        let trace = session.trace()?;
        let viewport = declared_in(&trace, ui_rect, RANGE).and_then(|(_, vp)| vp);
        pointer.type_text(&session, viewport.as_deref(), "1-2")?;
        session.settle(10);
    }
    for region in run.untick {
        click(region)?;
    }
    click(EXTRACT)?;
    session.settle(25);
    pointer.gone(&session)?;

    let trace = session.trace()?;
    let Some(line) = trace.last(WROTE) else {
        return Ok(Some(format!(
            "{}: Extract was pressed and no `{WROTE}` line followed. Failed: {}. Trace: {}.",
            run.fixture,
            trace
                .last("extract-failed")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    if let Some((key, want)) = run.expect.iter().find(|(k, v)| line.get(k) != Some(v)) {
        return Ok(Some(format!(
            "{} should extract with `{key}={want}`; the extraction traced `{}`.",
            run.fixture, line.raw
        )));
    }
    let deleted = trace.last(DELETED);
    if deleted.is_some() != run.deletes {
        return Ok(Some(format!(
            "{}: delete-afterwards was {} and {} page delete followed (`{}`).",
            run.fixture,
            if run.deletes { "ticked" } else { "not ticked" },
            if deleted.is_some() { "a" } else { "no" },
            deleted.map_or("none", |l| l.raw.as_str())
        )));
    }
    if let Some(deleted) = deleted
        && (deleted.get("page") != Some("0") || deleted.get("n") != Some("2"))
    {
        return Ok(Some(format!(
            "the delete afterwards should take pages 1-2 (`page=0 n=2`); it traced `{}`.",
            deleted.raw
        )));
    }
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the extraction traced success and {} cannot be read: {e}",
            target.display()
        ))
    })?;
    if !bytes.starts_with(b"%PDF-") {
        return Ok(Some(format!("{} is not a PDF.", target.display())));
    }
    report.artifact(target);
    report.note(format!("`{}`", line.raw));
    Ok(None)
}
