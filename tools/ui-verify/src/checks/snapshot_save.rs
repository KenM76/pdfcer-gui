//! `a_snapshot_saves_as_a_one_page_pdf` — the snapshot box's right-click menu
//! offers Save as PDF…, and the file it writes is one page the box's size.
//! Driven through the scripted pointer on a window placed off the desktop,
//! with the save dialog answered by `PDFCER_DIAG_SAVE_PATH`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_save.md`.

use super::snapshot_box::{BOX_REGION, GROUPS, ITEM, Rig, TAB, laid_box, launch_with};
use super::snapshot_copy::box_size;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::Result;
use crate::report::CheckReport;

const SAVED: &str = "snapshot-save-pdf"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "snapshot-save-refused"; // ui-text-exempt: a trace event name, never displayed
const MENU: &str = "canvas-menu"; // ui-text-exempt: a trace event name, never displayed
const CONTEXT: &str = "canvas.snapshot"; // ui-text-exempt: a menu context id, never displayed
const MENU_SAVE: &str = "menu.item.canvas.snapshot.view.snapshot_save_pdf"; // ui-text-exempt: a trace region name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// How far the saved page may sit from the box: the trace rounds the box's
/// corners to 0.1 pt.
const PT_TOLERANCE: f64 = 0.5;
/// The box's corners as fractions of the page, over part of the fixture's
/// drawing (x 6–50 %, y 50–92.5 %), so the cut has something to cut.
const FROM: (f64, f64) = (0.20, 0.55);
const TO: (f64, f64) = (0.60, 0.80);

/// See the module documentation.
pub struct ASnapshotSavesAsAOnePagePdf;

impl Check for ASnapshotSavesAsAOnePagePdf {
    fn name(&self) -> &'static str {
        "a_snapshot_saves_as_a_one_page_pdf"
    }

    fn defect(&self) -> &'static str {
        "The snapshot box's menu offers no Save as PDF…, or it writes nothing, or it writes \
         the whole page rather than the box"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let target = ctx.out("snapshot-save.pdf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(crate::error::Error::new(format!(
            "cannot clear {} before the save.",
            target.display()
        )));
    }
    let env = [(SAVE_PATH_ENV.to_owned(), target.display().to_string())];
    let (rig, page) = launch_with(ctx, report, "snapshot-save", &env)?;
    rig.click(TAB)?;
    rig.click_item(ITEM, GROUPS)?;
    let (w, h) = (page.width_pt, page.height_pt);
    let corners = ((FROM.0 * w, FROM.1 * h), (TO.0 * w, TO.1 * h));
    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, corners.0.0, corners.0.1))?;
    let b = mapping.doc_to_window(DocPoint::new(0, corners.1.0, corners.1.1))?;
    rig.pointer.drag(&rig.session, a, b, 12)?;
    rig.session.settle(20);
    if let Err(why) = laid_box(&rig, corners)? {
        return Ok(Some(why));
    }
    let (box_w, box_h) = box_size(&rig)?;

    let inside = WindowPoint::centre_of(rig.region(BOX_REGION)?);
    rig.pointer.right_click(&rig.session, inside)?;
    rig.session.settle(15);
    let context = rig
        .session
        .trace()?
        .last(MENU)
        .and_then(|l| l.get("context").map(str::to_owned));
    if context.as_deref() != Some(CONTEXT) {
        return Ok(Some(format!(
            "★ a right-click inside the box opened `{}`, not `{CONTEXT}`.",
            context.as_deref().unwrap_or("no menu")
        )));
    }
    rig.click(MENU_SAVE)?;
    rig.session.settle(40);
    judge(&rig, report, &target, (box_w, box_h), (w, h))
}

/// Require the saved line and a file whose one page is the box's size.
fn judge(
    rig: &Rig,
    report: &mut CheckReport,
    target: &std::path::Path,
    (box_w, box_h): (f64, f64),
    (page_w, page_h): (f64, f64),
) -> Result<Option<String>> {
    let trace = rig.session.trace()?;
    if let Some(refused) = trace.last(REFUSED) {
        return Ok(Some(format!(
            "★★ Save as PDF… was refused: `{}`.",
            refused.raw
        )));
    }
    let Some(line) = trace.last(SAVED) else {
        return Ok(Some(format!(
            "★★ Save as PDF… wrote no `{SAVED}` line. Trace: {}.",
            rig.session.trace_path().display()
        )));
    };
    report.note(format!("saved: `{}`", line.raw));
    let bytes = std::fs::read(target).unwrap_or_default();
    if !bytes.starts_with(b"%PDF-") {
        return Ok(Some(format!(
            "★★ the save traced success and {} is not a PDF ({} bytes).",
            target.display(),
            bytes.len()
        )));
    }
    report.artifact(target.to_path_buf());
    let Some(saved) = crate::fixture::page_geometry(target) else {
        return Ok(Some(format!(
            "{} has no direct /MediaBox to read the page size from.",
            target.display()
        )));
    };
    report.note(format!(
        "the saved page is {:.1} x {:.1} pt; the box {box_w:.1} x {box_h:.1}; the source page \
         {page_w:.1} x {page_h:.1}",
        saved.width_pt, saved.height_pt
    ));
    let off = |got: f64, want: f64| (got - want).abs() > PT_TOLERANCE;
    if off(saved.width_pt, box_w) || off(saved.height_pt, box_h) {
        return Ok(Some(format!(
            "★★★ the saved page is {:.1} x {:.1} pt; the box is {box_w:.1} x {box_h:.1}.",
            saved.width_pt, saved.height_pt
        )));
    }
    Ok(None)
}
