//! `export_dxf_writes_one_file_per_page` — Export to DXF with *Every page*
//! chosen writes one DXF per page, each holding its own page's geometry.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_dxf_pages.md`.

use super::export_dxf_options::{click_region, header_value};
use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The width ratio of `cropped-sheets.pdf`'s two drawings: 180 pt on page 1, 120 pt on page 2.
const WIDTH_RATIO: f64 = 1.5;
/// Slack for the stroke's half-width, which the extents may include.
const RATIO_TOLERANCE: f64 = 0.01;
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const FILE_TAB: &str = "file"; // ui-text-exempt: a ribbon tab id
const COMMAND: &str = "ribbon.item.file.export_dxf"; // ui-text-exempt: a trace region name
const ALL_PAGES: &str = "export-dxf.pages.all"; // ui-text-exempt: a trace region name
const EXPORT: &str = "export-dxf.export"; // ui-text-exempt: a trace region name
const OPENED: &str = "export-dxf-open"; // ui-text-exempt: a trace event name
const PAGES: &str = "export-dxf-pages"; // ui-text-exempt: a trace event name
const WROTE: &str = "export-dxf"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct ExportDxfWritesOneFilePerPage;

impl Check for ExportDxfWritesOneFilePerPage {
    fn name(&self) -> &'static str {
        "export_dxf_writes_one_file_per_page"
    }

    fn defect(&self) -> &'static str {
        "Export to DXF offers every page and writes one file, or writes the page on screen \
         under every page's name"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The chosen name and the two per-page files it must become, all cleared.
fn targets(ctx: &CheckContext) -> Result<[std::path::PathBuf; 3]> {
    let all = [
        ctx.out("dxf-pages.dxf"),
        ctx.out("dxf-pages_p1.dxf"),
        ctx.out("dxf-pages_p2.dxf"),
    ];
    for path in &all {
        let _ = std::fs::remove_file(path);
        if path.exists() {
            return Err(Error::new(format!(
                "cannot clear {} before the run.",
                path.display()
            )));
        }
    }
    Ok(all)
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    chosen: &std::path::Path,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let pdf = driving::repo_fixture(
        "cropped-sheets.pdf",
        "The oracle is that fixture's two rectangle widths, so --pdf is ignored.",
    )?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("export_dxf_pages.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), chosen.display().to_string()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("export_dxf_pages.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched on {} as pid {}",
        pdf.display(),
        session.pid()
    ));
    session.settle(30);
    Ok((session, pointer))
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let [chosen, first, second] = targets(ctx)?;
    let (session, pointer) = launch(ctx, report, &chosen)?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    crate::checks::ocr::click_tab(&session, &pointer, ui_rect, FILE_TAB)?;
    let Some(item) = driving::declared_or_in_overflow(&session, &pointer, ui_rect, COMMAND)? else {
        return Err(Error::new(format!("the File tab declares no `{COMMAND}`.")));
    };
    crate::input::Click::click_rect(&pointer, &session, item)?;
    session.settle(20);
    if session.trace()?.last(OPENED).is_none() {
        return Ok(Some(format!(
            "`{COMMAND}` was clicked and no `{OPENED}` followed."
        )));
    }
    click_region(&session, &pointer, ui_rect, ALL_PAGES)?;
    session.settle(10);
    let trace = session.trace()?;
    let chose = trace.last(PAGES);
    report.note(format!(
        "pages: `{}`",
        chose.map_or("none", |l| l.raw.as_str())
    ));
    if chose.and_then(|l| l.get("pages")) != Some("0,1") {
        return Ok(Some(format!(
            "★ *Every page* was clicked and the window did not move to pages 0,1: `{}`.",
            chose.map_or("no export-dxf-pages line", |l| l.raw.as_str())
        )));
    }
    click_region(&session, &pointer, ui_rect, EXPORT)?;
    session.settle(40);
    let wrote = session.trace()?.events(WROTE).count();
    if wrote != 2 {
        return Ok(Some(format!(
            "★★ Export wrote {wrote} `{WROTE}` lines for two pages. Trace: {}.",
            session.trace_path().display()
        )));
    }
    judge(report, &chosen, &first, &second)
}

/// Both files exist, the bare chosen name does not, and each holds its own page.
fn judge(
    report: &mut CheckReport,
    chosen: &std::path::Path,
    first: &std::path::Path,
    second: &std::path::Path,
) -> Result<Option<String>> {
    if chosen.exists() {
        return Ok(Some(format!(
            "★★ a two-page export wrote the chosen name {} itself; each page should carry _p<n>.",
            chosen.display()
        )));
    }
    let mut widths = Vec::new();
    for path in [first, second] {
        let Ok(dxf) = std::fs::read_to_string(path) else {
            return Ok(Some(format!("★★ {} was not written.", path.display())));
        };
        report.artifact(path.to_path_buf());
        let min = header_value(&dxf, "$EXTMIN").and_then(|v| v.parse::<f64>().ok());
        let max = header_value(&dxf, "$EXTMAX").and_then(|v| v.parse::<f64>().ok());
        let (Some(min), Some(max)) = (min, max) else {
            return Ok(Some(format!(
                "{} carries no readable extents.",
                path.display()
            )));
        };
        widths.push(max - min);
    }
    let ratio = widths[0] / widths[1];
    report.note(format!(
        "widths {:.3} and {:.3}, ratio {ratio:.4}; expected {WIDTH_RATIO}",
        widths[0], widths[1]
    ));
    if (ratio - WIDTH_RATIO).abs() > RATIO_TOLERANCE {
        return Ok(Some(format!(
            "★★★ the two files' drawings are {:.3} and {:.3} wide (ratio {ratio:.4}); page 1's \
             is half as wide again as page 2's, so each file does not hold its own page.",
            widths[0], widths[1]
        )));
    }
    Ok(None)
}
