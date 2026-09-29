//! `merge_runs` — **select a line written in two pieces, choose Merge text runs
//! on its right-click menu, and the file changes.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/merge_runs.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas selects page content.
const MODE: &str = "edit";
/// Shared with `move_line_of_text`: its first line is written in two pieces.
const FIXTURE: &str = "inherited-runs.pdf";
/// That line, PDF user space (y up).
const AIM: (f64, f64) = (87.0, 704.0);
/// The ladder line, and the rung a Points-tool click on a line lands on.
const SELECTION_EVENT: &str = "canvas-selection";
const PART_LEVEL: &str = "Part";
/// The menu row's region.
const ROW_REGION: &str = "menu.item.canvas.object.format.merge_text_runs";
/// `merge-text-runs-applied page=… object=… merged=… scale=…`, after the engine
/// call succeeded.
const APPLIED_EVENT: &str = "merge-text-runs-applied";
/// Written when the status bar records any canvas decline, the merge's
/// refusals among them.
const RECORDED_EVENT: &str = "canvas-decline-recorded";

/// See the module documentation.
pub struct MergingTextRunsReachesTheDocument;

impl Check for MergingTextRunsReachesTheDocument {
    fn name(&self) -> &'static str {
        "merge_runs"
    }

    fn defect(&self) -> &'static str {
        "Merge text runs is offered on a line written in several pieces, and choosing it \
         commits nothing to the document"
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

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
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
            "input is disabled (--no-input). This check clicks, right-clicks and picks a row.",
        ));
    }
    let ui_rect = vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}.", pdf.display())))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("merge_runs.trace.txt"));
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

    // --- 1: Edit mode; the Points tool; click the line ---------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, 0)?;
    let frame = session.frame()?;
    let at = frame.to_screen(mapping.doc_to_window(DocPoint::new(0, AIM.0, AIM.1))?);
    driver.press(vk::A)?;
    session.settle(10);
    driver.click_at(at)?;
    session.settle(20);
    let trace = session.trace()?;
    let level = trace
        .events(SELECTION_EVENT)
        .last()
        .and_then(|l| l.get("level").map(str::to_owned));
    if level.as_deref() != Some(PART_LEVEL) {
        return Err(Error::new(format!(
            "the click on the line left the ladder at {level:?}, not `{PART_LEVEL}`. \
             `move_line_of_text` owns this link. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 2: right-click it; find the row -----------------------------------
    driver.right_click_at(at)?;
    session.settle(35);
    let trace = session.trace()?;
    let Some(row) = driving::declared(&trace, ui_rect, ROW_REGION) else {
        let shot = ctx.out("merge_runs.no-row.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "a line written in two pieces is selected and its right-click menu has no \
             `{ROW_REGION}` row. Rows present: {}. Trace: {}.",
            driving::list(&driving::declared_names(
                &trace,
                ui_rect,
                "menu.item.canvas.object."
            )),
            session.trace_path().display()
        )));
    };

    // --- 3: choose it ------------------------------------------------------
    let mark = session.trace()?.events(RECORDED_EVENT).count();
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(30);

    // --- 4: the verdict ----------------------------------------------------
    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        let why = trace
            .events(RECORDED_EVENT)
            .nth(mark)
            .map_or_else(|| "none".to_owned(), |l| l.raw.clone());
        return Ok(Some(format!(
            "Merge text runs was chosen and no `{APPLIED_EVENT}` line followed. A decline \
             recorded after the pick: {why}. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the merge committed: `{}`", applied.raw));
    let merged: usize = applied
        .get("merged")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if merged < 2 {
        return Ok(Some(format!(
            "the engine reported `merged={merged}` for a line written in two pieces: `{}`.",
            applied.raw
        )));
    }
    report.note("the line's pieces are one run now, through the engine");
    Ok(None)
}
