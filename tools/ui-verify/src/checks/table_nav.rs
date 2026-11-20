//! `arrow_keys_walk_a_table_by_its_cells` — Up from a table cell lands in the
//! cell above it, not in whichever run of the row above sits under the caret.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/table_nav.md`.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The table whose bottom-left word sits right-aligned under the middle column.
const FIXTURE: &str = "cell-walk-table.pdf";
/// Off the desktop, so no OS input reaches it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
// ui-text-exempt: trace event names and region ids, never displayed
const CARET: &str = "text-edit-caret";
const STEP: &str = "text-caret-step";
const NOWHERE: &str = "text-caret-nowhere";
const MODE: &str = "ribbon.mode.edit";
const TAB: &str = "ribbon.tab.edit";
const TOOL: &str = "ribbon.item.edit.text";
/// Inside "Nut", bottom row, first column, drawn at (86, 90).
const ON_NUT: (f64, f64) = (92.0, 93.0);
/// The runs of `cell-walk-table.pdf` in page-text order: "Nut", the cell
/// above it ("Bolt"), and the run of the row above under the caret's x ("12").
const NUT_RUN: &str = "12";
const BOLT_RUN: &str = "6";
const QTY_RUN: &str = "8";

/// See the module documentation.
pub struct ArrowKeysWalkATableByItsCells;

impl Check for ArrowKeysWalkATableByItsCells {
    fn name(&self) -> &'static str {
        "arrow_keys_walk_a_table_by_its_cells"
    }

    fn defect(&self) -> &'static str {
        "Up or Down in a ruled table ignores its cells: the caret leaves a cell for whichever \
         word of the next row lies under it, so a column cannot be walked with the arrow keys"
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

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer, std::path::PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let pdf = crate::fixture::workspace_root().join("fixtures").join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let mut spec = LaunchSpec::new(&exe, ctx.out("table-nav.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("table-nav.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, pdf))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let (session, pointer, pdf) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    pointer.click_region(&session, ui_rect, MODE)?;
    pointer.click_region(&session, ui_rect, TAB)?;
    pointer.click_region(&session, ui_rect, TOOL)?;

    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        &session,
        mapping.doc_to_window(DocPoint::new(0, ON_NUT.0, ON_NUT.1))?,
    )?;
    session.settle(25);
    let trace = session.trace()?;
    let caret = trace.events(CARET).filter(|l| l.get("run").is_some()).last();
    let Some(caret) = caret else {
        return Ok(Some(format!(
            "a click inside \"Nut\" placed no caret in a run. The text tool is armed and the \
             point is on the word, so the caret placement is the suspect. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if caret.get("run") != Some(NUT_RUN) {
        return Err(Error::new(format!(
            "the caret landed in run {} where this check expects \"Nut\" at run {NUT_RUN}: the \
             shell's page text numbers the fixture's runs differently from the constants \
             here, so neither outcome below could be read. `{}`.",
            caret.get("run").unwrap_or("?"),
            caret.raw
        )));
    }
    report.note(format!("the caret is in \"Nut\": `{}`", caret.raw));

    pointer.key(&session, None, "ArrowUp", None)?;
    session.settle(25);
    pointer.gone(&session)?;
    verdict(&session, report)
}

fn verdict(session: &Session, report: &mut CheckReport) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(step) = trace.events(STEP).last() else {
        let why = trace
            .events(NOWHERE)
            .last()
            .map_or_else(|| "no line at all".to_owned(), |l| format!("`{}`", l.raw));
        return Ok(Some(format!(
            "Up from \"Nut\" moved nowhere ({why}); the cell above holds \"Bolt\". Trace: {}.",
            session.trace_path().display()
        )));
    };
    match step.get("to_run") {
        Some(BOLT_RUN) => {
            report.note(format!("Up walked the column to \"Bolt\": `{}`", step.raw));
            Ok(None)
        }
        Some(QTY_RUN) => Ok(Some(format!(
            "Up from \"Nut\" landed in \"12\", the word of the row above under the caret, not \
             in \"Bolt\", the cell above: the block model the arrow keys read was built without \
             the page's ruled cells (`detect_cell_regions`). `{}`.",
            step.raw
        ))),
        _ => Ok(Some(format!(
            "Up from \"Nut\" landed in neither \"Bolt\" (run {BOLT_RUN}) nor \"12\" (run \
             {QTY_RUN}): `{}`. Trace: {}.",
            step.raw,
            session.trace_path().display()
        ))),
    }
}
