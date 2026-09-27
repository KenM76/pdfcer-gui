//! `layers_can_be_merged_and_flattened` — the Layers panel merges one layer
//! into another, flattens every layer into the page, and Ctrl+Z brings the
//! layers back.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/layer_combine.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::forms_spotlight::open_from_tab;
use crate::checks::layer_authoring::{click, has_row, right_click_row};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const MODE: &str = "edit";
const PANEL_ITEM: &str = "ribbon.item.view.panel_layers";
const NEW_NAME: &str = "panel.layers.new.name";
const NEW: &str = "panel.layers.new";
const MENU_MERGE: &str = "panel.layers.menu.merge";
const MERGE_TARGET: &str = "panel.layers.merge.target";
const MERGE_GO: &str = "panel.layers.merge.go";
const FLATTEN: &str = "panel.layers.flatten";
const FLATTEN_GO: &str = "panel.layers.flatten.go";

/// The layer merged away, and the one it is merged into. No spaces, so each
/// is its own row-region key.
const MERGED: &str = "Welds";
const TARGET: &str = "Frame";

const VK_CONTROL: u16 = 0x11;
const VK_Z: u16 = 0x5A;

/// See the module documentation.
pub struct LayersCanBeMergedAndFlattened;

impl Check for LayersCanBeMergedAndFlattened {
    fn name(&self) -> &'static str {
        "layers_can_be_merged_and_flattened"
    }

    fn defect(&self) -> &'static str {
        "the Layers panel cannot merge a layer into another or flatten the layers into the page, \
         or does so without the list changing, or without Ctrl+Z undoing it"
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

/// Stop the drive with the message if a step failed.
macro_rules! step {
    ($e:expr) => {
        if let Err(why) = $e? {
            return Ok(Some(why));
        }
    };
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. A document with no layers is best; the check adds its own two.")
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check types and clicks in the Layers panel.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("layer_combine.trace.txt"));
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
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);

    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, NEW).is_none() {
        open_from_tab(&session, &driver, ui_rect, "view", PANEL_ITEM)?;
        session.settle(24);
    }

    // --- Two layers -----------------------------------------------------------
    for name in [MERGED, TARGET] {
        step!(click(&session, &driver, ui_rect, NEW_NAME, "no name field"));
        driver.type_ascii(name)?;
        session.settle(10);
        step!(click(
            &session,
            &driver,
            ui_rect,
            NEW,
            "no New layer button"
        ));
        session.settle(20);
        if !has_row(&session.trace()?, ui_rect, name) {
            return Ok(Some(format!(
                "New layer did not add a {name:?} row; merge and flatten were not reached. \
                 Rows: {}.",
                list(&declared_names(
                    &session.trace()?,
                    ui_rect,
                    "panel.layers.row."
                ))
            )));
        }
    }

    // --- Merge ----------------------------------------------------------------
    step!(right_click_row(&session, &driver, ui_rect, MERGED));
    step!(click(
        &session,
        &driver,
        ui_rect,
        MENU_MERGE,
        "the row menu"
    ));
    step!(click(
        &session,
        &driver,
        ui_rect,
        MERGE_TARGET,
        "the Merge window"
    ));
    let option = format!("panel.layers.merge.option.{TARGET}");
    step!(click(
        &session,
        &driver,
        ui_rect,
        &option,
        "the target list"
    ));
    let mark = session.trace()?.mark();
    step!(click(
        &session,
        &driver,
        ui_rect,
        MERGE_GO,
        "the Merge window"
    ));
    session.settle(20);
    let trace = session.trace()?;
    let Some(merged) = trace.last_after("layer-merged", mark) else {
        return Ok(Some(format!(
            "Merge was clicked and no `layer-merged` line followed: `merge_layers` never ran. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("merged: `{}`", merged.raw));
    if merged.get("changed") != Some("true")
        || merged.get("layers") != Some("1")
        || has_row(&trace, ui_rect, MERGED)
        || !has_row(&trace, ui_rect, TARGET)
    {
        return Ok(Some(format!(
            "the merge did not reach the list: `{}`; rows now {}.",
            merged.raw,
            list(&declared_names(&trace, ui_rect, "panel.layers.row."))
        )));
    }

    // --- Flatten --------------------------------------------------------------
    step!(click(
        &session,
        &driver,
        ui_rect,
        FLATTEN,
        "no Flatten button"
    ));
    let mark = session.trace()?.mark();
    step!(click(
        &session,
        &driver,
        ui_rect,
        FLATTEN_GO,
        "the Flatten dialog (a hidden layer would offer two choices instead)"
    ));
    session.settle(20);
    let trace = session.trace()?;
    let Some(flat) = trace.last_after("layer-flattened", mark) else {
        return Ok(Some(format!(
            "Flatten was clicked and no `layer-flattened` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("flattened: `{}`", flat.raw));
    if flat.get("changed") != Some("true") || has_row(&trace, ui_rect, TARGET) {
        return Ok(Some(format!(
            "the flatten did not reach the list: `{}`; rows now {}.",
            flat.raw,
            list(&declared_names(&trace, ui_rect, "panel.layers.row."))
        )));
    }

    // --- Undo -----------------------------------------------------------------
    driver.press_chord(&[VK_CONTROL], VK_Z)?;
    session.settle(30);
    let trace = session.trace()?;
    if !has_row(&trace, ui_rect, TARGET) {
        return Ok(Some(format!(
            "one Ctrl+Z after the flatten did not bring {TARGET:?} back. Rows: {}.",
            list(&declared_names(&trace, ui_rect, "panel.layers.row."))
        )));
    }
    report.note("one Ctrl+Z restored the flattened layer");
    Ok(None)
}
