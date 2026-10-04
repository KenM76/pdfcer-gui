//! `layer_folders_show_and_reorganise` — the Layers panel draws the document's
//! `/D /Order` as a tree, and its folders and moves rearrange it: a move from
//! the row menu, a new folder, a drag onto a folder, a rename, a removal that
//! keeps what the folder held, and Ctrl+Z.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/layer_folders.md`.

use std::collections::BTreeMap;

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list, repo_fixture,
};
use crate::checks::forms_spotlight::open_from_tab;
use crate::checks::layer_authoring::{click, right_click_row};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::Click;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

const FIXTURE: &str = "layer-folders.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-folders.PROVENANCE.py`.";
const MODE: &str = "edit";
const PANEL_ITEM: &str = "ribbon.item.view.panel_layers";
const OFFSCREEN: &str = "-4200,-4200,1400,900";

const MOVE_INTO: &str = "panel.layers.menu.move_into";
const NEW_FOLDER_NAME: &str = "panel.layers.new_folder.name";
const NEW_FOLDER: &str = "panel.layers.new_folder";
const RENAME: &str = "panel.layers.menu.rename_folder";
const REMOVE: &str = "panel.layers.menu.remove_folder";
const RENAME_NAME: &str = "panel.layers.rename.name";
const RENAME_APPLY: &str = "panel.layers.rename.apply";

/// The tree the fixture declares, as `path kind name`.
const DECLARED: [(&str, &str, &str); 8] = [
    ("0", "folder", "Sheet"),
    ("0.0", "layer", "Title Block"),
    ("0.1", "layer", "Grid Lines"),
    ("1", "layer", "Dimensions"),
    ("1.0", "layer", "Dimensions Reference"),
    ("2", "folder", "Services"),
    ("2.0", "layer", "Electrical"),
    ("3", "layer", "Notes"),
];

/// See the module documentation.
pub struct LayerFoldersShowAndReorganise;

impl Check for LayerFoldersShowAndReorganise {
    fn name(&self) -> &'static str {
        "layer_folders_show_and_reorganise"
    }

    fn defect(&self) -> &'static str {
        "the Layers panel lists layers flat and ignores the folders and sublayers the document \
         declares, or cannot add, rename, remove or move into a folder, or a move does not \
         reach the list, or Ctrl+Z does not undo it"
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

/// The tree as the newest frame drew it: path to `(kind, name)`.
fn tree_now(trace: &Trace) -> BTreeMap<String, (String, String)> {
    let mut out = BTreeMap::new();
    for line in trace.lines.iter().rev().filter(|l| l.event == "layer-node") {
        let path = line.get("path").unwrap_or_default().to_owned();
        if out.contains_key(&path) {
            break;
        }
        let kind = line.get("kind").unwrap_or_default().to_owned();
        let name = line
            .get("name")
            .unwrap_or_default()
            .trim_matches('"')
            .to_owned();
        out.insert(path, (kind, name));
    }
    out
}

fn show(tree: &BTreeMap<String, (String, String)>) -> String {
    let rows: Vec<String> = tree
        .iter()
        .map(|(p, (k, n))| format!("{p} {k} {n:?}"))
        .collect();
    list(&rows)
}

/// `None` when every `(path, kind, name)` is in the newest frame's tree.
fn expect(session: &Session, want: &[(&str, &str, &str)], when: &str) -> Result<Option<String>> {
    let tree = tree_now(&session.trace()?);
    let missing: Vec<String> = want
        .iter()
        .filter(|(p, k, n)| tree.get(*p) != Some(&((*k).to_owned(), (*n).to_owned())))
        .map(|(p, k, n)| format!("{p} {k} {n:?}"))
        .collect();
    Ok((!missing.is_empty()).then(|| {
        format!(
            "{when}, the list does not show {}. It shows {}.",
            list(&missing),
            show(&tree)
        )
    }))
}

/// The newest `layer-order` line after `mark`, or the failure.
fn order_line(
    session: &Session,
    mark: usize,
    op: &str,
    path: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after("layer-order", mark) else {
        return Ok(Some(format!(
            "no `layer-order` line followed the {op}: the engine was never asked. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("{op}: `{}`", line.raw));
    let ok = line.get("op") == Some(op)
        && line.get("path") == Some(path)
        && line.get("changed") == Some("true");
    Ok((!ok).then(|| {
        format!(
            "expected op={op} path={path} changed=true, got `{}`.",
            line.raw
        )
    }))
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<(Session, ScriptedPointer, &'static str)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("layer_folders.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
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
    let driver = ScriptedPointer::attach(&mut spec, ctx.out("layer_folders.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(driver.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, "panel.layers.row.Notes").is_none() {
        open_from_tab(&session, &driver, ui_rect, "view", PANEL_ITEM)?;
        session.settle(24);
    }
    Ok((session, driver, ui_rect))
}

/// A declared region, or the failure naming what the panel did declare.
fn region(
    session: &Session,
    ui_rect: &str,
    name: &str,
) -> Result<std::result::Result<crate::geom::LRect, String>> {
    let trace = session.trace()?;
    Ok(declared(&trace, ui_rect, name).ok_or_else(|| {
        format!(
            "no `{name}` region. Regions beginning `panel.layers.`: {}.",
            list(&declared_names(&trace, ui_rect, "panel.layers."))
        )
    }))
}

macro_rules! step {
    ($e:expr) => {
        match $e {
            Ok(()) => {}
            Err(why) => return Ok(Some(why)),
        }
    };
}

macro_rules! check {
    ($e:expr) => {
        if let Some(why) = $e {
            return Ok(Some(why));
        }
    };
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, driver, ui_rect) = launch(ctx, report)?;
    check!(expect(&session, &DECLARED, "on opening")?);
    report.note("the tree is drawn as the document's /Order declares it");

    // --- Move into a folder, from the row menu ------------------------------
    step!(right_click_row(&session, &driver, ui_rect, "Notes")?);
    let into = match region(&session, ui_rect, MOVE_INTO)? {
        Ok(r) => r,
        Err(why) => return Ok(Some(format!("the Notes row menu: {why}"))),
    };
    driver.hover(&session, WindowPoint::centre_of(into))?;
    session.settle(20);
    let mark = session.trace()?.mark();
    step!(click(
        &session,
        &driver,
        ui_rect,
        "panel.layers.menu.into.Services",
        "Move into folder"
    )?);
    session.settle(20);
    check!(order_line(&session, mark, "move", "2.1", report)?);
    check!(expect(
        &session,
        &[("2.1", "layer", "Notes")],
        "after Move into folder ▸ Services"
    )?);

    // --- New folder -----------------------------------------------------------
    step!(click(
        &session,
        &driver,
        ui_rect,
        NEW_FOLDER_NAME,
        "no folder name field"
    )?);
    driver.type_text(&session, None, "Civil")?;
    session.settle(10);
    let mark = session.trace()?.mark();
    step!(click(
        &session,
        &driver,
        ui_rect,
        NEW_FOLDER,
        "no New folder button"
    )?);
    session.settle(20);
    check!(order_line(&session, mark, "add-folder", "3", report)?);
    check!(expect(
        &session,
        &[("3", "folder", "Civil")],
        "after New folder"
    )?);

    check!(drag_into_civil(&session, &driver, ui_rect, report)?);
    check!(rename_remove_undo(&session, &driver, ui_rect, report)?);
    Ok(None)
}

/// Drag Electrical onto the Civil folder's middle band.
fn drag_into_civil(
    session: &Session,
    driver: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let from = match region(session, ui_rect, "panel.layers.row.Electrical")? {
        Ok(r) => r,
        Err(why) => return Ok(Some(why)),
    };
    let to = match region(session, ui_rect, "panel.layers.folder.Civil")? {
        Ok(r) => r,
        Err(why) => return Ok(Some(why)),
    };
    let mark = session.trace()?.mark();
    driver.drag(
        session,
        WindowPoint::centre_of(from),
        WindowPoint::centre_of(to),
        12,
    )?;
    session.settle(24);
    let trace = session.trace()?;
    if trace.last_after("layer-drag-begin", mark).is_none() {
        return Ok(Some(
            "the Electrical row was dragged and no `layer-drag-begin` followed: the row does \
             not sense a drag."
                .to_owned(),
        ));
    }
    if let Some(drop) = trace.last_after("layer-drop", mark) {
        report.note(format!("drop: `{}`", drop.raw));
    }
    check!(order_line(session, mark, "move", "3.0", report)?);
    expect(
        session,
        &[("3.0", "layer", "Electrical"), ("2.0", "layer", "Notes")],
        "after the drag",
    )
}

/// Rename Civil, remove Services keeping Notes, then undo the removal.
fn rename_remove_undo(
    session: &Session,
    driver: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    step!(open_folder_menu(session, driver, ui_rect, "Civil")?);
    step!(click(session, driver, ui_rect, RENAME, "the folder menu")?);
    step!(click(
        session,
        driver,
        ui_rect,
        RENAME_NAME,
        "the Rename folder window"
    )?);
    driver.key(session, None, "A", Some("ctrl"))?;
    driver.type_text(session, None, "Civil2")?;
    session.settle(10);
    let mark = session.trace()?.mark();
    step!(click(
        session,
        driver,
        ui_rect,
        RENAME_APPLY,
        "the Rename folder window"
    )?);
    session.settle(20);
    check!(order_line(session, mark, "rename-folder", "3", report)?);
    check!(expect(
        session,
        &[("3", "folder", "Civil2")],
        "after Rename folder"
    )?);

    step!(open_folder_menu(session, driver, ui_rect, "Services")?);
    let mark = session.trace()?.mark();
    step!(click(session, driver, ui_rect, REMOVE, "the folder menu")?);
    session.settle(20);
    check!(order_line(session, mark, "remove-folder", "2", report)?);
    let after = [("2", "layer", "Notes"), ("3", "folder", "Civil2")];
    check!(expect(session, &after, "after Remove folder")?);

    driver.key(session, None, "Z", Some("ctrl"))?;
    session.settle(30);
    let back = [("2", "folder", "Services"), ("2.0", "layer", "Notes")];
    check!(expect(session, &back, "after Ctrl+Z")?);
    report.note("Ctrl+Z put the removed folder back with what it held");
    Ok(None)
}

fn open_folder_menu(
    session: &Session,
    driver: &ScriptedPointer,
    ui_rect: &str,
    label: &str,
) -> Result<std::result::Result<(), String>> {
    let r = match region(session, ui_rect, &format!("panel.layers.folder.{label}"))? {
        Ok(r) => r,
        Err(why) => return Ok(Err(why)),
    };
    driver.right_click_rect(session, r)?;
    session.settle(20);
    Ok(Ok(()))
}
