//! `split_writes_the_files_the_window_listed` — Pages ▸ Split… divides the
//! document by the rule the operator chose, under the name pattern and into
//! the folder he chose, and writes exactly the files its preview listed.
//!
//! Three launches, off the desktop through the scripted pointer:
//! every 2 pages of `labelled-pages.pdf` with the labels box cleared; after
//! pages 1 and 3 of `four-pages.pdf` under `part-{start}-{end}.pdf`; and at
//! each top-level bookmark of `four-pages.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/split_pages.md`.

use std::path::Path;

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OPENED: &str = "split-pages-open"; // ui-text-exempt: a trace event name, never displayed
const PREVIEW: &str = "split-pages-preview"; // ui-text-exempt: a trace event name, never displayed
const WRITTEN: &str = "split-written"; // ui-text-exempt: a trace event name, never displayed
const PART: &str = "split-part"; // ui-text-exempt: a trace event name, never displayed
const MODE: &str = "ribbon.mode.review"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.pages"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.pages.split"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.pages.organise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const SPLIT: &str = "split-pages.split"; // ui-text-exempt: a trace region name, never displayed
const EVERY_N: &str = "split-pages.rule.every.n"; // ui-text-exempt: a trace region name, never displayed
const AFTER_PAGES: &str = "split-pages.rule.after.pages"; // ui-text-exempt: a trace region name, never displayed
const BOOKMARKS: &str = "split-pages.rule.bookmarks"; // ui-text-exempt: a trace region name, never displayed
const TEMPLATE: &str = "split-pages.template"; // ui-text-exempt: a trace region name, never displayed
const BROWSE: &str = "split-pages.browse"; // ui-text-exempt: a trace region name, never displayed
const LABELS: &str = "split-pages.labels"; // ui-text-exempt: a trace region name, never displayed
const FOLDER_ENV: &str = "PDFCER_DIAG_SPLIT_FOLDER"; // ui-text-exempt: an environment variable name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// See the module documentation.
pub struct SplitWritesTheFilesTheWindowListed;

impl Check for SplitWritesTheFilesTheWindowListed {
    fn name(&self) -> &'static str {
        "split_writes_the_files_the_window_listed"
    }

    fn defect(&self) -> &'static str {
        "Pages ▸ Split… divides the document somewhere other than the operator chose, names \
         or places the files differently from its preview, or ignores the page-labels box"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        for run in RUNS {
            match drive(ctx, &mut report, run) {
                Ok(None) => {}
                Ok(Some(failure)) => return report.fail(failure),
                Err(why) => return report.from_error(&why),
            }
        }
        report.pass()
    }
}

/// How a run picks its rule after the window opens.
enum Pick {
    /// Type into the every-N box.
    Every(&'static str),
    /// Type into the after-pages box.
    After(&'static str),
    /// Click the bookmarks radio.
    Bookmarks,
}

/// One launch: the fixture, the rule, an optional name pattern, whether the
/// labels box is cleared, and what must be written.
struct Run {
    fixture: &'static str,
    stem: &'static str,
    pick: Pick,
    template: Option<&'static str>,
    drop_labels: bool,
    /// Whether the window should offer the bookmarks rule.
    bookmarks: bool,
    /// `(name, first, last)` for each file, 0-based pages.
    files: &'static [(&'static str, usize, usize)],
    /// Fields every `split-part` line must carry.
    part_fields: &'static [(&'static str, &'static str)],
}

const RUNS: &[Run] = &[
    Run {
        fixture: "labelled-pages.pdf",
        stem: "split-every",
        pick: Pick::Every("2"),
        template: None,
        drop_labels: true,
        bookmarks: false,
        files: &[
            ("labelled-pages_1.pdf", 0, 1),
            ("labelled-pages_2.pdf", 2, 3),
        ],
        part_fields: &[("labels_dropped", "1"), ("label_ranges", "0")],
    },
    Run {
        fixture: "four-pages.pdf",
        stem: "split-after",
        pick: Pick::After("1, 3"),
        template: Some("part-{start}-{end}.pdf"),
        drop_labels: false,
        bookmarks: true,
        files: &[
            ("part-1-1.pdf", 0, 0),
            ("part-2-3.pdf", 1, 2),
            ("part-4-4.pdf", 3, 3),
        ],
        part_fields: &[],
    },
    Run {
        fixture: "four-pages.pdf",
        stem: "split-bookmarks",
        pick: Pick::Bookmarks,
        template: None,
        drop_labels: false,
        bookmarks: true,
        files: &[
            ("four-pages_1.pdf", 0, 0),
            ("four-pages_2.pdf", 1, 1),
            ("four-pages_3.pdf", 2, 2),
            ("four-pages_4.pdf", 3, 3),
        ],
        part_fields: &[],
    },
];

/// A launched app and the clicks a run needs.
struct Driven {
    session: Session,
    pointer: ScriptedPointer,
    ui_rect: &'static str,
}

impl Driven {
    fn click(&self, region: &str) -> Result<()> {
        let trace = self.session.trace()?;
        let (rect, viewport) = declared_in(&trace, self.ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, self.ui_rect, prefix))
            ))
        })?;
        self.pointer.click_in(
            &self.session,
            viewport.as_deref(),
            WindowPoint::centre_of(rect),
        )?;
        self.session.settle(15);
        Ok(())
    }

    /// Click `region`, optionally select what it holds, and type `text`.
    fn type_into(&self, region: &str, text: &str, replace: bool) -> Result<()> {
        self.click(region)?;
        let viewport =
            declared_in(&self.session.trace()?, self.ui_rect, region).and_then(|(_, vp)| vp);
        if replace {
            self.pointer
                .key(&self.session, viewport.as_deref(), "A", Some("ctrl"))?;
        }
        self.pointer
            .type_text(&self.session, viewport.as_deref(), text)?;
        self.session.settle(10);
        Ok(())
    }

    fn has(&self, region: &str) -> Result<bool> {
        Ok(declared(&self.session.trace()?, self.ui_rect, region).is_some())
    }

    fn path(&self) -> String {
        self.session.trace_path().display().to_string()
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    run: &Run,
    folder: &Path,
) -> Result<Driven> {
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
        return Err(Error::new(format!(
            "the fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{}.trace.txt", run.stem)));
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
        .push((FOLDER_ENV.to_owned(), folder.display().to_string()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out(&format!("{}.pointer.txt", run.stem)))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok(Driven {
        session,
        pointer,
        ui_rect,
    })
}

/// An empty folder for one run's files.
fn fresh_folder(ctx: &CheckContext, run: &Run) -> Result<std::path::PathBuf> {
    let folder = std::path::absolute(ctx.out(run.stem))
        .map_err(|e| Error::new(format!("cannot resolve the output folder: {e}")))?;
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder)
        .map_err(|e| Error::new(format!("cannot make {}: {e}", folder.display())))?;
    Ok(folder)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport, run: &Run) -> Result<Option<String>> {
    let folder = fresh_folder(ctx, run)?;
    let d = launch(ctx, report, run, &folder)?;
    d.click(MODE)?;
    d.click(TAB)?;
    if !d.has(ITEM)? && d.has(COLLAPSED)? {
        d.click(COLLAPSED)?;
    }
    d.click(ITEM)?;
    d.session.settle(20);
    let Some(opened) = d.session.trace()?.last(OPENED).cloned() else {
        return Ok(Some(format!(
            "{}: Split… opened no window (no `{OPENED}`). Trace: {}.",
            run.fixture,
            d.path()
        )));
    };
    if opened.get("bookmarks") != Some(if run.bookmarks { "1" } else { "0" }) {
        return Ok(Some(format!(
            "{}: the window should {} the bookmarks rule; it opened as `{}`.",
            run.fixture,
            if run.bookmarks { "offer" } else { "not offer" },
            opened.raw
        )));
    }
    match run.pick {
        Pick::Every(n) => d.type_into(EVERY_N, n, false)?,
        Pick::After(pages) => d.type_into(AFTER_PAGES, pages, false)?,
        Pick::Bookmarks => d.click(BOOKMARKS)?,
    }
    if let Some(template) = run.template {
        d.type_into(TEMPLATE, template, true)?;
    }
    if run.drop_labels {
        d.click(LABELS)?;
    }
    d.click(BROWSE)?;
    d.session.settle(15);
    if let Some(failure) = judge_preview(&d, run, &folder)? {
        return Ok(Some(failure));
    }
    d.click(SPLIT)?;
    d.session.settle(30);
    d.pointer.gone(&d.session)?;
    judge_written(&d, report, run, &folder)
}

/// The preview must list exactly the run's files, in the run's folder,
/// before Split is pressed.
fn judge_preview(d: &Driven, run: &Run, folder: &Path) -> Result<Option<String>> {
    let trace = d.session.trace()?;
    let Some(preview) = trace.last(PREVIEW) else {
        return Ok(Some(format!(
            "{}: no `{PREVIEW}` line after the rule was chosen. Trace: {}.",
            run.fixture,
            d.path()
        )));
    };
    let names: Vec<String> = run.files.iter().map(|(n, ..)| format!("{n:?}")).collect();
    let want_names = format!("[{}]", names.join(", "));
    let want_ranges = run
        .files
        .iter()
        .map(|(_, a, b)| format!("{}-{}", a + 1, b + 1))
        .collect::<Vec<_>>()
        .join(",");
    let want_folder = format!("{:?}", folder.display().to_string());
    let files = run.files.len().to_string();
    if preview.get("files") != Some(files.as_str())
        || preview.get("ranges") != Some(want_ranges.as_str())
        || !preview.raw.contains(&format!("names={want_names}"))
        || !preview.raw.contains(&format!("folder={want_folder}"))
    {
        return Ok(Some(format!(
            "{}: before Split the window previewed `{}`; wanted files={files} \
             ranges={want_ranges} names={want_names} folder={want_folder}.",
            run.fixture, preview.raw
        )));
    }
    Ok(None)
}

/// Every listed file is on disk as a PDF, each `split-part` line carries its
/// pages, and nothing else was written to the folder.
fn judge_written(
    d: &Driven,
    report: &mut CheckReport,
    run: &Run,
    folder: &Path,
) -> Result<Option<String>> {
    let trace = d.session.trace()?;
    let Some(written) = trace.last(WRITTEN) else {
        return Ok(Some(format!(
            "{}: Split was pressed and no `{WRITTEN}` line followed. Failed: {}. Trace: {}.",
            run.fixture,
            trace
                .last("split-failed")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            d.path()
        )));
    };
    report.note(format!("`{}`", written.raw));
    let parts: Vec<_> = trace.events(PART).collect();
    if parts.len() != run.files.len() {
        return Ok(Some(format!(
            "{}: {} `{PART}` lines for the {} files the window listed. Trace: {}.",
            run.fixture,
            parts.len(),
            run.files.len(),
            d.path()
        )));
    }
    for (line, (name, first, last)) in parts.iter().zip(run.files) {
        let pages = (last - first + 1).to_string();
        let (first, last) = (first.to_string(), last.to_string());
        let mut want = vec![
            ("first", first.as_str()),
            ("last", last.as_str()),
            ("pages", pages.as_str()),
        ];
        want.extend_from_slice(run.part_fields);
        if let Some((key, value)) = want.iter().find(|(k, v)| line.get(k) != Some(v)) {
            return Ok(Some(format!(
                "{}: {name} should be written with `{key}={value}`; it traced `{}`.",
                run.fixture, line.raw
            )));
        }
        let target = folder.join(name);
        let bytes = std::fs::read(&target).unwrap_or_default();
        if !bytes.starts_with(b"%PDF-") {
            return Ok(Some(format!(
                "{}: {} is missing or is not a PDF after the split traced it.",
                run.fixture,
                target.display()
            )));
        }
        report.artifact(target);
    }
    let on_disk = std::fs::read_dir(folder).map_or(0, Iterator::count);
    if on_disk != run.files.len() {
        return Ok(Some(format!(
            "{}: {on_disk} files are in {} after the split; the window listed {}.",
            run.fixture,
            folder.display(),
            run.files.len()
        )));
    }
    Ok(None)
}
