//! The Objects tree gathers rows the way a file list does: Ctrl+click adds
//! a row, Shift+click selects a run, and a right-click on a selected row
//! keeps the whole selection, so its menu acts on every gathered picture.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/objects_multiselect.md`.

use super::dimdrive::{Fixture, press, reveal, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,view.reset_layout";
const PANEL_TAB: &str = "dock.tab.view.panel_objects";
const ROW_A: &str = "objects.row.0";
const ROW_B: &str = "objects.row.1";
const PICKED: &str = "objects-panel-selection"; // ui-text-exempt: a trace event name, never displayed
const OPENED: &str = "shrink-images-opened"; // ui-text-exempt: a trace event name, never displayed
const SHRUNK: &str = "images-shrunk"; // ui-text-exempt: a trace event name, never displayed
const MENU_ROW: &str = "menu.item.objects.row.format.shrink_images";
const COMMIT: &str = "shrink-images.commit";
const PHOTOS: Fixture = Fixture {
    file: "two-photos.pdf",
    method: "Rebuild it with `python fixtures/two-photos.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 200.0,
    },
};

/// See the module documentation.
pub struct ObjectsTreeGathersRowsForOneMenu;

impl Check for ObjectsTreeGathersRowsForOneMenu {
    fn name(&self) -> &'static str {
        "objects_tree_gathers_rows_for_one_menu"
    }

    fn defect(&self) -> &'static str {
        "in the Objects tree, Ctrl+click or Shift+click does not gather a second row, or a \
         right-click on a gathered row drops the others so its menu acts on one picture"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &PHOTOS, INVOKE, "objectsmulti", drive)
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    reveal(ctx, session, pointer, ROW_A, PANEL_TAB)?;
    press(ctx, session, pointer, ROW_A)?;
    let mark = session.trace()?.mark();
    click_row(ctx, session, pointer, ROW_B, Some("ctrl"))?;
    if let Some(failure) = picked(session, report, mark, "Ctrl+click", "add")? {
        return Ok(Some(failure));
    }

    press(ctx, session, pointer, ROW_A)?;
    let mark = session.trace()?.mark();
    click_row(ctx, session, pointer, ROW_B, Some("shift"))?;
    if let Some(failure) = picked(session, report, mark, "Shift+click", "set")? {
        return Ok(Some(failure));
    }

    let mark = session.trace()?.mark();
    click_row(ctx, session, pointer, ROW_A, None)?;
    if let Some(line) = session.trace()?.last_after(PICKED, mark) {
        return Ok(Some(format!(
            "a right-click on a gathered row changed the selection (`{}`); it must keep \
             both rows. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    press(ctx, session, pointer, MENU_ROW)?;
    if let Some(failure) = traced(session, report, mark, OPENED, "pictures")? {
        return Ok(Some(failure));
    }
    press(ctx, session, pointer, COMMIT)?;
    session.settle(60);
    traced(session, report, mark, SHRUNK, "asked")
}

/// Click a row's centre with `mods`, or right-click it when `mods` is `None`.
fn click_row(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    row: &str,
    mods: Option<&str>,
) -> Result<()> {
    let trace = session.trace()?;
    let Some((rect, vp)) = declared_in(&trace, ui_rect_event(ctx)?, row) else {
        return Err(Error::new(format!(
            "`{row}` was never declared. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if vp.is_some() {
        return Err(Error::new(format!(
            "`{row}` was drawn in a separate window; modified clicks are driven only in the \
             main one. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let at = WindowPoint::centre_of(rect);
    pointer.hover(session, at)?;
    session.settle(10);
    match mods {
        Some(m) => pointer.click_with(session, at, m)?,
        None => pointer.right_click(session, at)?,
    };
    session.settle(30);
    Ok(())
}

/// The tree's selection after `mark` came from `op` and holds both rows.
fn picked(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    route: &str,
    op: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.last_after(PICKED, mark);
    if let Some(l) = line {
        report.note(format!("{route}: `{}`", l.raw));
        if l.get("op") == Some(op) && l.get("sel") == Some("2") && l.get("level") == Some("object")
        {
            return Ok(None);
        }
    }
    Ok(Some(format!(
        "{route} on the second row must read `op={op} sel=2 level=object`; it read `{}`. \
         Trace: {}.",
        line.map_or("nothing", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}

/// `event` after `mark` carries `field=2`: both gathered pictures.
fn traced(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    event: &str,
    field: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.last_after(event, mark);
    if let Some(l) = line {
        report.note(format!("`{}`", l.raw));
        if l.get(field) == Some("2") {
            return Ok(None);
        }
    }
    Ok(Some(format!(
        "`{event}` must read `{field}=2` after the row menu; it read `{}`. Trace: {}.",
        line.map_or("nothing", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}
