//! `blank_pages_are_inserted_as_one_step` — Pages ▸ Insert blank…, driven:
//! three A3 landscape sheets land at the end as three separate pages, and
//! one undo takes all three away.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/insert_blank_pages.md`.

use super::dimdrive::{Fixture, press, press_on_tab, run_on, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "four-pages.pdf",
    method: "It is checked in.",
    page: PageGeometry {
        width_pt: 2383.937,
        height_pt: 1683.78,
    },
};
const PAGES_TAB: &str = "ribbon.tab.pages";
const ITEM: &str = "ribbon.item.pages.insert_blank";
const BODY: &str = "insert-blank.body";
const SIZE: &str = "insert-blank.size";
/// A3: `PaperSize::ALL` index 3, after the "Same as page N" entry at 0.
const A3_ENTRY: &str = "insert-blank.size.item.4";
const LANDSCAPE: &str = "insert-blank.landscape";
const AT_END: &str = "insert-blank.where.3";
const COUNT: &str = "insert-blank.count";
const INSERT: &str = "insert-blank.insert";
const DIALOG: &str = "insert-blank-dialog"; // ui-text-exempt: a trace event name, never displayed
const LANDED: &str = "insert-pages-landed"; // ui-text-exempt: a trace event name, never displayed
const RESYNC_FAILED: &str = "pages-resync-failed"; // ui-text-exempt: a trace event name, never displayed
const CANVAS: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed
/// A3 landscape in points, the crop box each new sheet must carry.
const A3_LANDSCAPE: (f64, f64) = (1190.55, 841.89);

/// See the module documentation.
pub struct BlankPagesAreInsertedAsOneStep;

impl Check for BlankPagesAreInsertedAsOneStep {
    fn name(&self) -> &'static str {
        "blank_pages_are_inserted_as_one_step"
    }

    fn defect(&self) -> &'static str {
        "Insert blank pages adds sheets of the wrong size, or several sheets that are one page \
         object listed twice (a page tree pdfcer's own reader rejects), or needs one undo per \
         sheet"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.edit", "insert-blank", drive)
    }
}

/// Open the size list (a combo popup may take a second press) and pick A3.
fn pick_a3(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    for _ in 0..3 {
        press(ctx, session, pointer, SIZE)?;
        if declared_in(&session.trace()?, ui_rect_event(ctx)?, A3_ENTRY).is_some() {
            return press(ctx, session, pointer, A3_ENTRY);
        }
    }
    Err(Error::new(format!(
        "the size list never declared `{A3_ENTRY}`. Trace: {}.",
        session.trace_path().display()
    )))
}

/// Replace the count with `n`, typed into its own viewport.
fn type_count(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    n: &str,
) -> Result<()> {
    press(ctx, session, pointer, COUNT)?;
    let (_, vp) = declared_in(&session.trace()?, ui_rect_event(ctx)?, COUNT)
        .ok_or_else(|| Error::new(format!("`{COUNT}` vanished after it was pressed.")))?;
    pointer.key(session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(session, vp.as_deref(), n)?;
    pointer.key(session, vp.as_deref(), "Enter", None)?;
    session.settle(10);
    Ok(())
}

/// Whether `crop` (`llx,lly,urx,ury`) is a sheet of `size` at the origin.
fn crop_is(crop: &str, size: (f64, f64)) -> bool {
    let v: Vec<f64> = crop.split(',').filter_map(|s| s.parse().ok()).collect();
    v.len() == 4
        && v[0].abs() < 0.05
        && v[1].abs() < 0.05
        && (v[2] - size.0).abs() < 0.05
        && (v[3] - size.1).abs() < 0.05
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    press_on_tab(ctx, session, pointer, ITEM, PAGES_TAB)?;
    if declared_in(&session.trace()?, ui_rect_event(ctx)?, BODY).is_none() {
        return Ok(Some(format!(
            "Pages ▸ Insert blank… opened no dialog (no `{BODY}`). Trace: {path}."
        )));
    }
    pick_a3(ctx, session, pointer)?;
    press(ctx, session, pointer, LANDSCAPE)?;
    press(ctx, session, pointer, AT_END)?;
    type_count(ctx, session, pointer, "3")?;

    let mark = session.trace()?.mark();
    press(ctx, session, pointer, INSERT)?;
    session.settle(40);
    let trace = session.trace()?;

    // The dialog's own record of what the harness set. A mismatch is the
    // harness not reaching a control, not the feature failing.
    let dialog = trace
        .last_after(DIALOG, mark)
        .ok_or_else(|| Error::new(format!("Insert raised no `{DIALOG}` line. Trace: {path}.")))?;
    report.note(format!("dialog: `{}`", dialog.raw));
    if dialog.get("n") != Some("3")
        || dialog.get("landscape") != Some("true")
        || dialog.get("position") != Some("End")
    {
        return Err(Error::new(format!(
            "the dialog did not take the harness's choices (3, landscape, at the end): `{}`.",
            dialog.raw
        )));
    }

    if let Some(failed) = trace.last_after(RESYNC_FAILED, mark) {
        return Ok(Some(format!(
            "the inserted sheets left a page tree the reader rejects: `{}`. Three copies of one \
             blank sheet went in as ONE page object listed three times. Trace: {path}.",
            failed.raw
        )));
    }
    let Some(landed) = trace.last_after(LANDED, mark) else {
        return Ok(Some(format!(
            "Insert landed no pages (no `{LANDED}`). Trace: {path}."
        )));
    };
    report.note(format!("landed: `{}`", landed.raw));
    if landed.get("at") != Some("4")
        || landed.get("pages") != Some("7")
        || landed.get("was") != Some("4")
    {
        return Ok(Some(format!(
            "three sheets at the end of four pages must read `at=4 pages=7 was=4`; it read `{}`. \
             Trace: {path}.",
            landed.raw
        )));
    }
    let Some(canvas) = trace.last_after(CANVAS, mark) else {
        return Ok(Some(format!(
            "no `{CANVAS}` line after the insert. Trace: {path}."
        )));
    };
    report.note(format!("canvas: `{}`", canvas.raw));
    if canvas.get("page") != Some("4") {
        return Ok(Some(format!(
            "after the insert the view must be on the first new sheet, page index 4; the canvas \
             read `{}`. Under a continuous display the landing is only a navigation when \
             `tracked_page` is left behind for `canvas::strip::page_scroll_offset`. Trace: {path}.",
            canvas.raw
        )));
    }
    if !canvas.get("crop").is_some_and(|c| crop_is(c, A3_LANDSCAPE)) {
        return Ok(Some(format!(
            "the first new sheet must be A3 landscape, crop `0,0,{},{}`; the canvas read `{}`. \
             Trace: {path}.",
            A3_LANDSCAPE.0, A3_LANDSCAPE.1, canvas.raw
        )));
    }

    // One undo takes all three.
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    let trace = session.trace()?;
    let pages = trace
        .last_after(CANVAS, mark)
        .and_then(|c| c.get("pages").map(str::to_owned));
    report.note(format!("after one undo: pages={pages:?}"));
    if pages.as_deref() != Some("4") {
        return Ok(Some(format!(
            "one Ctrl+Z after inserting three sheets must leave four pages; the canvas read \
             pages={pages:?}. The insert was not folded into one undo step. Trace: {path}."
        )));
    }
    Ok(None)
}
