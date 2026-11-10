//! The marquee region's commands, driven off the desktop through Tool Options
//! with the snapshot tool armed.
//!
//! - `delete_in_region_removes_what_the_box_holds`: a box over the fixture's
//!   drawing fills the selection; the block's Delete in region deletes that
//!   many objects, and the same box drawn again then holds none.
//! - `redact_outside_hatches_then_marks`: with Outside chosen, Redact opens the
//!   confirmation window and the canvas hatches the paper outside the box but
//!   not inside it; Mark outside then marks one page.
//! - `crop_to_region_crops_to_the_box`: Crop to region opens the crop window
//!   seeded from the box, and Crop commits a crop box at the box's corners.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/region_commands.md`.

use super::snapshot_ants::shot;
use super::snapshot_box::{BOX_REGION, Rig, laid_box, launch_with};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::Result;
use crate::geom::PixRect;
use crate::image::Image;
use crate::report::CheckReport;

/// Edit mode, then the snapshot tool: the region's content commands are Edit's.
const INVOKE: &str = "mode.edit,view.tool_snapshot"; // ui-text-exempt: command ids, never displayed
const SELECTION: &str = "region-selection"; // ui-text-exempt: a trace event name, never displayed
const DELETE_REQUESTED: &str = "region-delete-requested"; // ui-text-exempt: a trace event name, never displayed
const OUTSIDE_OPENED: &str = "redact-outside-opened"; // ui-text-exempt: a trace event name, never displayed
const OUTSIDE_MARKED: &str = "redact-mark-outside-marked"; // ui-text-exempt: a trace event name, never displayed
const DELETE_BUTTON: &str = "properties.tool.region.button.edit.delete_in_region"; // ui-text-exempt: a trace region name, never displayed
const REDACT_OUTSIDE_BUTTON: &str = "properties.tool.region.button.edit.redact_outside_region"; // ui-text-exempt: a trace region name, never displayed
const OUTSIDE_CHOICE: &str = "properties.tool.region.redact_side.outside"; // ui-text-exempt: a trace region name, never displayed
const HATCH: &str = "canvas.region.outside_hatch"; // ui-text-exempt: a trace region name, never displayed
const CROP_BUTTON: &str = "properties.tool.region.button.pages.crop_to_region"; // ui-text-exempt: a trace region name, never displayed
const CROP_APPLY: &str = "page-crop.apply"; // ui-text-exempt: a trace region name, never displayed
const CROP_COMMIT: &str = "page-crop-commit"; // ui-text-exempt: a trace event name, never displayed
/// The crop window works in whole millimetres, so its rect may sit up to half
/// a millimetre (1.42 pt) from the box on each edge.
const CROP_TOLERANCE_PT: f64 = 1.5;
const MARK: &str = "redact_outside.mark"; // ui-text-exempt: a trace region name, never displayed

/// Over the fixture's drawing, which lies inside x 6–50 %, y 50–92.5 %.
const OVER_DRAWING: ((f64, f64), (f64, f64)) = ((0.04, 0.48), (0.52, 0.94));
/// The drawing again, from a corner outside the first box, so the drag draws a
/// new box rather than moving or resizing the one already laid.
const OVER_DRAWING_AGAIN: ((f64, f64), (f64, f64)) = ((0.02, 0.46), (0.54, 0.96));
/// Blank paper, clear of the drawing.
const OVER_BLANK: ((f64, f64), (f64, f64)) = ((0.70, 0.10), (0.90, 0.30));
/// How far inside the box the interior starts, clear of the outline and grips.
const INSET_PX: u32 = 16;
/// The least share of the strip left of the box that must change for hatching.
const HATCHED_SHARE: f64 = 0.02;

/// Arm the snapshot tool and drag a box at `fractions` of page 0.
fn lay(
    ctx: &CheckContext,
    rig: &Rig,
    page: crate::coords::PageGeometry,
    fractions: ((f64, f64), (f64, f64)),
) -> Result<std::result::Result<(), String>> {
    let (w, h) = (page.width_pt, page.height_pt);
    let corners = (
        (fractions.0.0 * w, fractions.0.1 * h),
        (fractions.1.0 * w, fractions.1.1 * h),
    );
    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, corners.0.0, corners.0.1))?;
    let b = mapping.doc_to_window(DocPoint::new(0, corners.1.0, corners.1.1))?;
    rig.pointer.drag(&rig.session, a, b, 12)?;
    rig.session.settle(25);
    laid_box(rig, corners).map(|step| step.map(|_| ()))
}

/// The `objects=` of the `nth` (1-based) `region-selection` line.
fn selected(rig: &Rig, nth: usize) -> Result<Option<usize>> {
    let trace = rig.session.trace()?;
    Ok(trace
        .events(SELECTION)
        .nth(nth - 1)
        .and_then(|l| l.get("objects")?.parse().ok()))
}

/// See the module documentation.
pub struct DeleteInRegion;

impl Check for DeleteInRegion {
    fn name(&self) -> &'static str {
        "delete_in_region_removes_what_the_box_holds"
    }

    fn defect(&self) -> &'static str {
        "a marquee box over a drawing selects nothing, or Delete in region deletes a different \
         number of objects than the box holds, or leaves them on the page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_delete(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive_delete(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let invoke = [("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned())];
    let (rig, page) = launch_with(ctx, report, "region-delete", &invoke)?;
    rig.session.settle(30);
    if let Err(why) = lay(ctx, &rig, page, OVER_DRAWING)? {
        return Ok(Some(why));
    }
    let held = selected(&rig, 1)?;
    report.note(format!(
        "the box over the drawing selected {held:?} objects"
    ));
    let Some(held) = held.filter(|&n| n > 0) else {
        return Ok(Some(format!(
            "★★ a box over the drawing filled the selection with {held:?} objects \
             (no `{SELECTION}` line, or zero)."
        )));
    };
    rig.click(DELETE_BUTTON)?;
    rig.session.settle(60);
    let trace = rig.session.trace()?;
    let Some(asked) = trace.last(DELETE_REQUESTED) else {
        return Ok(Some(format!(
            "★★ Delete in region was pressed and no `{DELETE_REQUESTED}` line followed."
        )));
    };
    report.note(asked.raw.clone());
    if asked.get("objects") != Some(held.to_string().as_str()) {
        return Ok(Some(format!(
            "★★★ the box held {held} objects and Delete in region asked for `{}`.",
            asked.raw
        )));
    }
    if let Err(why) = lay(ctx, &rig, page, OVER_DRAWING_AGAIN)? {
        return Ok(Some(why));
    }
    let after = selected(&rig, 2)?;
    report.note(format!(
        "a box over the same drawing selected {after:?} objects"
    ));
    rig.pointer.gone(&rig.session)?;
    if after != Some(0) {
        return Ok(Some(format!(
            "★★★ after deleting {held} objects a box over the same drawing still holds {after:?}."
        )));
    }
    Ok(None)
}

/// See the module documentation.
pub struct RedactOutsideRegion;

impl Check for RedactOutsideRegion {
    fn name(&self) -> &'static str {
        "redact_outside_hatches_then_marks"
    }

    fn defect(&self) -> &'static str {
        "Redact outside the region opens no confirmation, shows nothing of what will go, hatches \
         inside the box it keeps, or marks nothing once confirmed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_outside(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// How many pixels of `area` differ between `a` and `b`.
fn changed(a: &Image, b: &Image, area: PixRect) -> usize {
    (area.y..area.y + area.h)
        .flat_map(|y| (area.x..area.x + area.w).map(move |x| (x, y)))
        .filter(|&(x, y)| a.pixel(x, y) != b.pixel(x, y))
        .count()
}

fn drive_outside(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let invoke = [("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned())];
    let (rig, page) = launch_with(ctx, report, "region-redact-outside", &invoke)?;
    rig.session.settle(30);
    if let Err(why) = lay(ctx, &rig, page, OVER_BLANK)? {
        return Ok(Some(why));
    }
    rig.click(OUTSIDE_CHOICE)?;
    rig.pointer.gone(&rig.session)?;
    rig.session.settle(10);
    let before = shot(ctx, &rig, report, "region-outside-before.png")?;
    rig.click(REDACT_OUTSIDE_BUTTON)?;
    rig.pointer.gone(&rig.session)?;
    rig.session.settle(30);
    let trace = rig.session.trace()?;
    if trace.last(OUTSIDE_OPENED).is_none() {
        return Ok(Some(format!(
            "★★ Redact outside was pressed and no `{OUTSIDE_OPENED}` line followed."
        )));
    }
    let during = shot(ctx, &rig, report, "region-outside-during.png")?;
    let frame = rig.session.frame()?;
    let boxed = frame.logical_to_capture_pixels(rig.region(BOX_REGION)?);
    let paper = frame.logical_to_capture_pixels(rig.region(HATCH)?);
    let strip = PixRect {
        x: paper.x,
        y: boxed.y,
        w: boxed.x.saturating_sub(paper.x),
        h: boxed.h,
    };
    let interior = PixRect {
        x: boxed.x + INSET_PX,
        y: boxed.y + INSET_PX,
        w: boxed.w.saturating_sub(2 * INSET_PX),
        h: boxed.h.saturating_sub(2 * INSET_PX),
    };
    let (out, inn) = (
        changed(&before, &during, strip),
        changed(&before, &during, interior),
    );
    let area = f64::from(strip.w) * f64::from(strip.h);
    report.note(format!(
        "changed pixels with the window open: {out} of {area} left of the box, {inn} inside it"
    ));
    if inn > 0 {
        return Ok(Some(format!(
            "★★★ the box it keeps changed by {inn} pixels when the window opened."
        )));
    }
    if area <= 0.0 || (out as f64) < area * HATCHED_SHARE {
        return Ok(Some(format!(
            "★★ the paper outside the box did not change when the window opened ({out} of \
             {area} pixels), so nothing shows what will go."
        )));
    }
    crate::checks::region_press::press_region(&rig.session, &rig.pointer, MARK, "redact_outside.")?;
    rig.session.settle(60);
    let trace = rig.session.trace()?;
    let Some(marked) = trace.last(OUTSIDE_MARKED) else {
        return Ok(Some(format!(
            "★★ Mark outside was pressed and no `{OUTSIDE_MARKED}` line followed."
        )));
    };
    report.note(marked.raw.clone());
    rig.pointer.gone(&rig.session)?;
    if marked.get("marked") != Some("1") {
        return Ok(Some(format!(
            "★★★ Mark outside on the one-page fixture should mark one page: `{}`.",
            marked.raw
        )));
    }
    Ok(None)
}

/// See the module documentation.
pub struct CropToRegion;

impl Check for CropToRegion {
    fn name(&self) -> &'static str {
        "crop_to_region_crops_to_the_box"
    }

    fn defect(&self) -> &'static str {
        "Crop to region opens no crop window, or commits a crop box somewhere other than the          marquee box"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_crop(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive_crop(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let invoke = [("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned())];
    let (rig, page) = launch_with(ctx, report, "region-crop", &invoke)?;
    rig.session.settle(30);
    if let Err(why) = lay(ctx, &rig, page, OVER_DRAWING)? {
        return Ok(Some(why));
    }
    let trace = rig.session.trace()?;
    let laid = trace.last(super::snapshot_box::BOX_EVENT).map(|l| {
        ["llx", "lly", "urx", "ury"].map(|k| l.get(k).and_then(|v| v.parse::<f64>().ok()))
    });
    rig.click(CROP_BUTTON)?;
    rig.session.settle(30);
    crate::checks::region_press::press_region(
        &rig.session,
        &rig.pointer,
        CROP_APPLY,
        "page-crop.",
    )?;
    rig.session.settle(60);
    let trace = rig.session.trace()?;
    let Some(commit) = trace.last(CROP_COMMIT) else {
        return Ok(Some(format!(
            "★★ Crop was pressed in the window Crop to region opened and no `{CROP_COMMIT}`              line followed."
        )));
    };
    report.note(commit.raw.clone());
    rig.pointer.gone(&rig.session)?;
    let got =
        ["llx", "lly", "urx", "ury"].map(|k| commit.get(k).and_then(|v| v.parse::<f64>().ok()));
    let off = laid.is_none_or(|want| {
        want.iter().zip(got).any(|(w, g)| match (w, g) {
            (Some(w), Some(g)) => (w - g).abs() > CROP_TOLERANCE_PT,
            _ => true,
        })
    });
    if off || commit.get("n") != Some("1") {
        return Ok(Some(format!(
            "★★★ the box was {laid:?} and the crop committed `{}`.",
            commit.raw
        )));
    }
    Ok(None)
}
