//! `a_layer_can_be_made_renamed_and_deleted` — the Layers panel creates a
//! layer, renames it through its Properties window, deletes it keeping its
//! drawing, and Ctrl+Z brings it back.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/layer_authoring.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::forms_spotlight::open_from_tab;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode layer authoring is offered in.
const MODE: &str = "edit";
/// The ribbon item that opens the Layers panel.
const PANEL_ITEM: &str = "ribbon.item.view.panel_layers";

const NEW_NAME: &str = "panel.layers.new.name";
const NEW: &str = "panel.layers.new";
const MENU_PROPS: &str = "panel.layers.menu.properties";
const MENU_DELETE: &str = "panel.layers.menu.delete";
const PROPS_NAME: &str = "panel.layers.props.name";
const PROPS_APPLY: &str = "panel.layers.props.apply";
const DELETE_KEEP: &str = "panel.layers.delete.keep";

/// The names typed. No spaces, so each is its own row-region key.
const FIRST: &str = "Welds";
const SECOND: &str = "Welds2";

const VK_CONTROL: u16 = 0x11;
const VK_A: u16 = 0x41;
const VK_Z: u16 = 0x5A;

/// See the module documentation.
pub struct ALayerCanBeMadeRenamedAndDeleted;

impl Check for ALayerCanBeMadeRenamedAndDeleted {
    fn name(&self) -> &'static str {
        "a_layer_can_be_made_renamed_and_deleted"
    }

    fn defect(&self) -> &'static str {
        "the Layers panel can show and hide layers but cannot create, rename or delete one, or \
         does so without the change reaching the list, or without Ctrl+Z undoing it"
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

pub(crate) fn row(name: &str) -> String {
    format!("panel.layers.row.{name}")
}

/// Click a declared region, or say which one was missing.
pub(crate) fn click(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    region: &str,
    what: &str,
) -> Result<std::result::Result<(), String>> {
    let trace = session.trace()?;
    let Some(r) = declared(&trace, ui_rect, region) else {
        return Ok(Err(format!(
            "{what}: no `{region}` region. Regions beginning `panel.layers.`: {}.",
            list(&declared_names(&trace, ui_rect, "panel.layers."))
        )));
    };
    driver.click_at(session.frame()?.declared_center(r))?;
    session.settle(20);
    Ok(Ok(()))
}

/// Open a row's right-click menu.
pub(crate) fn right_click_row(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    name: &str,
) -> Result<std::result::Result<(), String>> {
    let trace = session.trace()?;
    let Some(r) = declared(&trace, ui_rect, &row(name)) else {
        return Ok(Err(format!(
            "the list has no row for {name:?}. Regions beginning `panel.layers.row.`: {}.",
            list(&declared_names(&trace, ui_rect, "panel.layers.row."))
        )));
    };
    driver.right_click_at(session.frame()?.declared_center(r))?;
    session.settle(20);
    Ok(Ok(()))
}

pub(crate) fn has_row(trace: &Trace, ui_rect: &str, name: &str) -> bool {
    declared(trace, ui_rect, &row(name)).is_some()
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
        Error::new("no --pdf. Any document will do; the check adds its own layer.")
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("layer_authoring.trace.txt"));
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

    // --- Create --------------------------------------------------------------
    if let Err(why) = click(&session, &driver, ui_rect, NEW_NAME, "no name field")? {
        return Ok(Some(why));
    }
    driver.type_ascii(FIRST)?;
    session.settle(10);
    if let Err(why) = click(&session, &driver, ui_rect, NEW, "no New layer button")? {
        return Ok(Some(why));
    }
    session.settle(20);
    let trace = session.trace()?;
    let Some(added) = trace.last("layer-added") else {
        return Ok(Some(format!(
            "New layer was clicked and no `layer-added` line followed: `add_layer` never ran. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("created: `{}`", added.raw));
    if added.get("name").map(|n| n.trim_matches('"')) != Some(FIRST) {
        return Ok(Some(format!(
            "the layer was created under the wrong name: `{}`, typed {FIRST:?}.",
            added.raw
        )));
    }
    if !has_row(&trace, ui_rect, FIRST) {
        return Ok(Some(format!(
            "`add_layer` ran and the list shows no {FIRST:?} row. Rows: {}.",
            list(&declared_names(&trace, ui_rect, "panel.layers.row."))
        )));
    }

    // --- Rename --------------------------------------------------------------
    if let Err(why) = right_click_row(&session, &driver, ui_rect, FIRST)? {
        return Ok(Some(why));
    }
    let mark = session.trace()?.mark();
    if let Err(why) = click(&session, &driver, ui_rect, MENU_PROPS, "the row menu")? {
        return Ok(Some(why));
    }
    // A new layer prints and exports when shown and is for viewing; the
    // window must open on those values, read from the file.
    let props_trace = session.trace()?;
    let Some(opened) = props_trace.last_after("layer-props-opened", mark) else {
        return Ok(Some(
            "Layer properties was clicked and no `layer-props-opened` line followed.".to_owned(),
        ));
    };
    report.note(format!("opened: `{}`", opened.raw));
    if opened.get("print") != Some("when_visible")
        || opened.get("export") != Some("when_visible")
        || opened.get("intent") != Some("view")
    {
        return Ok(Some(format!(
            "the Properties window did not open on the new layer's own settings \
             (printing and exporting when shown, for viewing): `{}`.",
            opened.raw
        )));
    }
    if let Err(why) = click(
        &session,
        &driver,
        ui_rect,
        PROPS_NAME,
        "the Properties window",
    )? {
        return Ok(Some(why));
    }
    driver.press_chord(&[VK_CONTROL], VK_A)?;
    driver.type_ascii(SECOND)?;
    session.settle(10);
    let mark = session.trace()?.mark();
    if let Err(why) = click(
        &session,
        &driver,
        ui_rect,
        PROPS_APPLY,
        "the Properties window",
    )? {
        return Ok(Some(why));
    }
    session.settle(20);
    let trace = session.trace()?;
    let Some(edited) = trace.last_after("layer-edited", mark) else {
        return Ok(Some(format!(
            "Apply was clicked and no `layer-edited` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("renamed: `{}`", edited.raw));
    if edited.get("changed") != Some("true")
        || !has_row(&trace, ui_rect, SECOND)
        || has_row(&trace, ui_rect, FIRST)
    {
        return Ok(Some(format!(
            "the rename did not reach the list: `{}`; rows now {}.",
            edited.raw,
            list(&declared_names(&trace, ui_rect, "panel.layers.row."))
        )));
    }

    // --- Delete, keeping the drawing ----------------------------------------
    if let Err(why) = right_click_row(&session, &driver, ui_rect, SECOND)? {
        return Ok(Some(why));
    }
    if let Err(why) = click(&session, &driver, ui_rect, MENU_DELETE, "the row menu")? {
        return Ok(Some(why));
    }
    let mark = session.trace()?.mark();
    if let Err(why) = click(&session, &driver, ui_rect, DELETE_KEEP, "the Delete dialog")? {
        return Ok(Some(why));
    }
    session.settle(20);
    let trace = session.trace()?;
    let Some(deleted) = trace.last_after("layer-deleted", mark) else {
        return Ok(Some(format!(
            "Keep the drawing was clicked and no `layer-deleted` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("deleted: `{}`", deleted.raw));
    if deleted.get("changed") != Some("true") || has_row(&trace, ui_rect, SECOND) {
        return Ok(Some(format!(
            "the delete did not reach the list: `{}`; rows now {}.",
            deleted.raw,
            list(&declared_names(&trace, ui_rect, "panel.layers.row."))
        )));
    }

    // --- Undo ----------------------------------------------------------------
    driver.press_chord(&[VK_CONTROL], VK_Z)?;
    session.settle(30);
    let trace = session.trace()?;
    if !has_row(&trace, ui_rect, SECOND) {
        return Ok(Some(format!(
            "Ctrl+Z after the delete did not bring {SECOND:?} back. Rows: {}.",
            list(&declared_names(&trace, ui_rect, "panel.layers.row."))
        )));
    }
    report.note("Ctrl+Z restored the deleted layer");
    Ok(None)
}
