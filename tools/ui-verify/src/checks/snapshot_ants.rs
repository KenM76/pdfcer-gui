//! `a_snapshot_box_outline_marches` — a laid snapshot box's outline moves
//! between frames while the page inside it does not. Three screenshots of the
//! window, taken a fraction of a second apart, must differ on the box's edge
//! and nowhere inside it. Driven through the scripted pointer on a window
//! placed off the desktop; the shots are egui's own, never the desktop's.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_ants.md`.

use std::time::Duration;

use super::snapshot_box::{BOX_REGION, FROM, GROUPS, ITEM, Rig, TAB, TO, laid_box, launch};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::Result;
use crate::geom::PixRect;
use crate::image::Image;
use crate::report::CheckReport;

/// The time between shots: not a multiple of the dash pattern's period, so
/// consecutive shots catch the dashes at different offsets.
const BETWEEN_SHOTS: Duration = Duration::from_millis(170);
/// How many pixels each side of the box's edge count as the edge.
const EDGE_PX: u32 = 2;
/// How far inside the box the interior starts, clear of the grips.
const INSET_PX: u32 = 16;
/// The fewest changed edge pixels that count as a march.
const MARCHED_PX: usize = 20;

/// See the module documentation.
pub struct ASnapshotBoxOutlineMarches;

impl Check for ASnapshotBoxOutlineMarches {
    fn name(&self) -> &'static str {
        "a_snapshot_box_outline_marches"
    }

    fn defect(&self) -> &'static str {
        "a snapshot box's outline stands still, so nothing tells a laid box from any other \
         rectangle on the page, or the page inside the box flickers with it"
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
    let (rig, page) = launch(ctx, report, "snapshot-ants")?;
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
    rig.pointer.gone(&rig.session)?;
    rig.session.settle(10);
    let shots = (0..3)
        .map(|i| {
            if i > 0 {
                std::thread::sleep(BETWEEN_SHOTS);
            }
            shot(ctx, &rig, report, &format!("snapshot-ants-{i}.png"))
        })
        .collect::<Result<Vec<Image>>>()?;
    let px = rig
        .session
        .frame()?
        .logical_to_capture_pixels(rig.region(BOX_REGION)?);
    Ok(judge(report, &shots, px))
}

/// Shoot the window to `name` and read it back.
fn shot(ctx: &CheckContext, rig: &Rig, report: &mut CheckReport, name: &str) -> Result<Image> {
    let png = ctx.out(name);
    rig.pointer.screenshot(&rig.session, &png)?;
    let image = Image::load_png(&png)?;
    report.artifact(png);
    Ok(image)
}

/// Fail unless some pair of shots differs on the edge and no pair inside.
fn judge(report: &mut CheckReport, shots: &[Image], px: PixRect) -> Option<String> {
    let pairs = [(0, 1), (1, 2), (0, 2)];
    let edge: Vec<usize> = pairs
        .iter()
        .map(|&(i, j)| changed(&shots[i], &shots[j], px, on_edge))
        .collect();
    let interior: Vec<usize> = pairs
        .iter()
        .map(|&(i, j)| changed(&shots[i], &shots[j], px, inside))
        .collect();
    report.note(format!(
        "changed pixels between shot pairs 0-1, 1-2, 0-2: edge {edge:?}, interior {interior:?}, \
         box {px:?}"
    ));
    if interior.iter().any(|&n| n > 0) {
        return Some(format!(
            "★★ the page inside the box changed between shots ({interior:?} pixels), so the \
             animation repaints more than the outline."
        ));
    }
    if edge.iter().all(|&n| n < MARCHED_PX) {
        return Some(format!(
            "★ the box's outline did not move between three shots {BETWEEN_SHOTS:?} apart \
             ({edge:?} edge pixels changed)."
        ));
    }
    None
}

/// How many pixels `keep` selects that differ between `a` and `b`.
fn changed(a: &Image, b: &Image, px: PixRect, keep: fn(PixRect, u32, u32) -> bool) -> usize {
    let (x0, y0) = (px.x.saturating_sub(EDGE_PX), px.y.saturating_sub(EDGE_PX));
    let (x1, y1) = (px.x + px.w + EDGE_PX, px.y + px.h + EDGE_PX);
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .filter(|&(x, y)| keep(px, x, y) && a.pixel(x, y) != b.pixel(x, y))
        .count()
}

/// Within [`EDGE_PX`] of the box's border.
fn on_edge(px: PixRect, x: u32, y: u32) -> bool {
    let near = |v: u32, lo: u32, hi: u32| v.abs_diff(lo) <= EDGE_PX || v.abs_diff(hi) <= EDGE_PX;
    let (x1, y1) = (px.x + px.w, px.y + px.h);
    let in_x = x + EDGE_PX >= px.x && x <= x1 + EDGE_PX;
    let in_y = y + EDGE_PX >= px.y && y <= y1 + EDGE_PX;
    (in_x && near(y, px.y, y1)) || (in_y && near(x, px.x, x1))
}

/// More than [`INSET_PX`] inside the box.
fn inside(px: PixRect, x: u32, y: u32) -> bool {
    x > px.x + INSET_PX
        && y > px.y + INSET_PX
        && x + INSET_PX < px.x + px.w
        && y + INSET_PX < px.y + px.h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edge_and_the_interior_do_not_overlap() {
        let px = PixRect {
            x: 100,
            y: 100,
            w: 200,
            h: 100,
        };
        assert!(on_edge(px, 100, 150));
        assert!(on_edge(px, 301, 200));
        assert!(!on_edge(px, 200, 150));
        assert!(inside(px, 200, 150));
        assert!(!inside(px, 110, 150));
        for (x, y) in [(100, 150), (300, 120), (150, 100), (150, 200)] {
            assert!(!inside(px, x, y));
        }
    }
}
