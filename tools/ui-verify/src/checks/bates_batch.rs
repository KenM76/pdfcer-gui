//! `bates_numbering_across_files` — Tools ▸ Batch ▸ Bates number files… on
//! four PDFs, one of them signed. The signed file's row says why it cannot be
//! numbered and Number files is absent until it is removed; the first file is
//! then moved down, and the run numbers the three remaining files in the new
//! order, carrying the count from each to the next, as copies beside the
//! originals, which stay byte-identical. Each copy begins with its original's
//! bytes: the numbering is an appended update, not a rewrite.
//!
//! Driven off the desktop through the scripted pointer, with nothing open.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/bates_batch.md`.

use std::path::PathBuf;

use super::savedannots::read;
use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OPENED: &str = "bates-batch-opened"; // ui-text-exempt: a trace event name, never displayed
const PROBED: &str = "bates-batch-probed"; // ui-text-exempt: a trace event name, never displayed
const WROTE: &str = "bates-batch-wrote"; // ui-text-exempt: a trace event name, never displayed
const DONE: &str = "bates-batch-done"; // ui-text-exempt: a trace event name, never displayed
/// Tools is an Edit-mode tab.
const MODE: &str = "ribbon.mode.edit"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.tools"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.tools.bates_files"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.tools.batch.collapsed"; // ui-text-exempt: a trace region name, never displayed
const RUN: &str = "bates_batch.run"; // ui-text-exempt: a trace region name, never displayed
const BODY: &str = "bates_batch.body"; // ui-text-exempt: a trace region name, never displayed
const SOURCES_ENV: &str = "PDFCER_DIAG_BATES_SOURCES"; // ui-text-exempt: an environment variable name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// The listed files, in the order the picker returns them: fixture, pages.
const LISTED: [(&str, usize); 4] = [
    ("labelled-pages.pdf", 4),
    ("signed-two-pages.pdf", 2),
    ("small-page-first.pdf", 2),
    ("four-pages-unrotated.pdf", 4),
];
/// The signed file's row.
const SIGNED_ROW: usize = 1;
/// After removing the signed row and moving row 0 down: the run's order,
/// with each file's first and last label.
const EXPECTED: [(&str, &str, &str); 3] = [
    ("small-page-first.pdf", "000001", "000002"),
    ("labelled-pages.pdf", "000003", "000006"),
    ("four-pages-unrotated.pdf", "000007", "000010"),
];

pub struct BatesNumberingAcrossFiles;

impl Check for BatesNumberingAcrossFiles {
    fn name(&self) -> &'static str {
        "bates_numbering_across_files"
    }

    fn defect(&self) -> &'static str {
        "Tools ▸ Batch ▸ Bates number files… does not number the listed files in the order shown, \
         does not carry the count from file to file, offers a run over a file it cannot number, \
         or changes an original"
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

/// A fresh folder holding copies of the listed fixtures.
fn stage(ctx: &CheckContext) -> Result<PathBuf> {
    let folder = std::path::absolute(ctx.out("bates-batch-files"))
        .map_err(|e| Error::new(format!("cannot resolve the staging folder: {e}")))?;
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder)
        .map_err(|e| Error::new(format!("cannot make {}: {e}", folder.display())))?;
    let fixtures = crate::fixture::workspace_root().join("fixtures");
    for (name, _) in LISTED {
        std::fs::copy(fixtures.join(name), folder.join(name))
            .map_err(|e| Error::new(format!("cannot stage {name}: {e}")))?;
    }
    Ok(folder)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
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
    let folder = stage(ctx)?;
    let sources: Vec<String> = LISTED
        .iter()
        .map(|(name, _)| folder.join(name).display().to_string())
        .collect();

    let mut spec = LaunchSpec::new(&exe, ctx.out("bates-batch.trace.txt"));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env.push((SOURCES_ENV.to_owned(), sources.join(";")));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("bates-batch.pointer.txt"))?;

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

    if fresh(TAB)?.is_none() {
        click(MODE)?;
    }
    click(TAB)?;
    if fresh(ITEM)?.is_none() && fresh(COLLAPSED)?.is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    // The window reads each file on a worker; wait for every verdict.
    for _ in 0..40 {
        if session.trace()?.events(PROBED).count() >= LISTED.len() {
            break;
        }
        session.settle(10);
    }
    let trace = session.trace()?;
    let Some(opened) = trace.last(OPENED).cloned() else {
        return Ok(Some(format!(
            "`{ITEM}` was clicked and no `{OPENED}` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if fresh(BODY)?.is_none() {
        return Ok(Some(format!(
            "`{}` was traced and the window published no `{BODY}`.",
            opened.raw
        )));
    }
    let probes: Vec<_> = trace.events(PROBED).cloned().collect();
    for (i, (name, pages)) in LISTED.iter().enumerate() {
        let Some(line) = probes
            .iter()
            .find(|l| l.get("path").is_some_and(|p| p.contains(name)))
        else {
            return Ok(Some(format!(
                "{name} was listed and never read: {} `{PROBED}` line(s).",
                probes.len()
            )));
        };
        let refused = line.get("refused") == Some("1");
        if refused != (i == SIGNED_ROW) {
            return Ok(Some(format!(
                "{name} should {} be refused; its probe traced `{}`.",
                if i == SIGNED_ROW { "" } else { "not" },
                line.raw
            )));
        }
        if !refused && line.get("pages") != Some(&pages.to_string()) {
            return Ok(Some(format!(
                "{name} has {pages} pages; its probe traced `{}`.",
                line.raw
            )));
        }
    }
    // A run over a list holding a file it cannot number must not be offered.
    if fresh(RUN)?.is_some() {
        return Ok(Some(format!(
            "`{RUN}` was drawn while {} was listed and refused.",
            LISTED[SIGNED_ROW].0
        )));
    }

    click(&format!("bates_batch.row.{SIGNED_ROW}.remove"))?;
    click("bates_batch.row.0.down")?;
    click(RUN)?;
    for _ in 0..60 {
        if session.trace()?.last(DONE).is_some() {
            break;
        }
        session.settle(10);
    }
    pointer.gone(&session)?;

    let trace = session.trace()?;
    let Some(done) = trace.last(DONE).cloned() else {
        return Ok(Some(format!(
            "Number files was clicked and no `{DONE}` line followed. Last refusal: {}. Trace: {}.",
            trace
                .last("bates-batch-refused")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    if done.get("files") != Some("3")
        || done.get("pages") != Some("10")
        || done.get("next") != Some("11")
    {
        return Ok(Some(format!(
            "three files of 2, 4 and 4 pages from 1 should end with 11 next; the run traced `{}`.",
            done.raw
        )));
    }
    let wrote: Vec<_> = trace.events(WROTE).cloned().collect();
    if wrote.len() != EXPECTED.len() {
        return Ok(Some(format!(
            "the run traced `{}` and {} `{WROTE}` line(s), not {}.",
            done.raw,
            wrote.len(),
            EXPECTED.len()
        )));
    }
    for (line, (name, first, last)) in wrote.iter().zip(EXPECTED) {
        // The parser unquotes a `{:?}` value.
        if line.get("source") != Some(name)
            || line.get("first_label") != Some(first)
            || line.get("last_label") != Some(last)
        {
            return Ok(Some(format!(
                "after removing the signed file and moving the first down, the run should number \
                 {name} {first} to {last}; it traced `{}`.",
                line.raw
            )));
        }
        let original = read(&folder.join(name))?;
        let fixture = read(&crate::fixture::workspace_root().join("fixtures").join(name))?;
        if original != fixture {
            return Ok(Some(format!("the run changed the original {name}.")));
        }
        let stem = name.trim_end_matches(".pdf");
        let copy_path = folder.join(format!("{stem}_Bates.pdf"));
        let copy = read(&copy_path)?;
        if copy.len() <= original.len() || !copy.starts_with(&original) {
            return Ok(Some(format!(
                "{} should be {name} with an appended update; it is {} bytes against {}.",
                copy_path.display(),
                copy.len(),
                original.len()
            )));
        }
    }
    let leftovers: Vec<_> = std::fs::read_dir(&folder)
        .map_err(|e| Error::new(format!("cannot list {}: {e}", folder.display())))?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".pdfcer-part"))
        .collect();
    if !leftovers.is_empty() {
        return Ok(Some(format!(
            "the run left part files behind: {}.",
            leftovers.join(", ")
        )));
    }
    report.note(format!("`{}`", opened.raw));
    for line in &wrote {
        report.note(format!("`{}`", line.raw));
    }
    report.note(format!("`{}`", done.raw));
    Ok(None)
}
