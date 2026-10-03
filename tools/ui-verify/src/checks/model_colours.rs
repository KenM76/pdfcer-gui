//! `checks::model_colours` — **a 3D model whose parts carry colours draws in
//! them, in the viewer and on the page**
//!
//! Places the engine corpus's `coloured.prc` from the ribbon, as
//! `checks::model_view_window` does, then opens it with *View…*.
//!
//! Oracles: on the page, the pixels the insert changed must fall in at least
//! two hue sectors of the window's own frame; in the viewer, which is an
//! immediate viewport no screenshot reaches, the `model-view-rendered` line
//! must count at least two hues (`hues>=2`) and `model-view-opened` must count
//! fewer parts `uncoloured` than it has.

use crate::checks::driving::{declared, declared_in};
use crate::checks::model_view_window::{
    VIEW_REGION, launch_with_model_file, press_insert, ui_rect,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::PixRect;
use crate::image::{Image, Rgb};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// A part-coloured PRC model from the engine corpus.
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/coloured.prc";
/// The first page's region on the canvas.
const PAGE_REGION: &str = "page";
/// A channel spread at or above this is a colour, not a shade of grey.
const CHROMA: u8 = 48;
/// A hue sector holding fewer than this share of the coloured pixels is
/// antialiasing between two others.
const SECTOR_SHARE: usize = 50;

/// See the module documentation.
pub struct AColouredModelDrawsInItsColours;

impl Check for AColouredModelDrawsInItsColours {
    fn name(&self) -> &'static str {
        "a_coloured_3d_model_draws_in_its_own_colours"
    }

    fn defect(&self) -> &'static str {
        "a 3D model whose parts carry colours is drawn in one flat colour, in the viewer or in \
         its picture on the page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model_file(ctx, &mut report, "model-colours", MODEL)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn shot(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    name: &str,
) -> Result<Image> {
    let path = ctx.out(name);
    pointer.screenshot(session, &path)?;
    report.artifact(path.clone());
    Image::load_png(&path)
}

/// The hue sector, of twelve, of a coloured pixel; `None` for a grey one.
fn sector(px: Rgb) -> Option<usize> {
    let (r, g, b) = (i32::from(px.r), i32::from(px.g), i32::from(px.b));
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let spread = max - min;
    if spread < i32::from(CHROMA) {
        return None;
    }
    // Hue in twelfths of a turn: 2 per 60-degree segment.
    let (base, num) = if max == r {
        (0, g - b)
    } else if max == g {
        (4, b - r)
    } else {
        (8, r - g)
    };
    let twelfths = (base + (2 * num).div_euclid(spread)).rem_euclid(12);
    usize::try_from(twelfths).ok()
}

/// How many hue sectors the pixels changed between `before` and `after`
/// inside `page` fill.
fn page_hues(before: &Image, after: &Image, page: PixRect) -> (usize, usize) {
    let mut sectors = [0_usize; 12];
    let w = before.width().min(after.width()).min(page.x + page.w);
    let h = before.height().min(after.height()).min(page.y + page.h);
    for y in page.y..h {
        for x in page.x..w {
            let Some((a, b)) = before.pixel(x, y).zip(after.pixel(x, y)) else {
                continue;
            };
            if a != b
                && let Some(s) = sector(b)
            {
                sectors[s] += 1;
            }
        }
    }
    let coloured: usize = sectors.iter().sum();
    let floor = (coloured / SECTOR_SHARE).max(1);
    (coloured, sectors.iter().filter(|n| **n >= floor).count())
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    pointer.gone(session)?;
    session.settle(10);
    let before = shot(ctx, report, session, pointer, "model-colours-before.png")?;
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let after = shot(ctx, report, session, pointer, "model-colours-after.png")?;
    let ui_rect = ui_rect(ctx)?;
    let trace = session.trace()?;
    let poster = trace
        .events("model-insert-poster")
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("poster line: {poster:?}"));
    let Some(page) = declared(&trace, ui_rect, PAGE_REGION) else {
        return Ok(Some(format!("no `{PAGE_REGION}` region on the canvas.")));
    };
    let page = session.frame()?.logical_to_capture_pixels(page);
    let (coloured, hues) = page_hues(&before, &after, page);
    report.note(format!(
        "page: {coloured} coloured pixels changed, in {hues} hues"
    ));
    if hues < 2 {
        return Ok(Some(format!(
            "the model's picture on the page shows {hues} hue(s) over {coloured} coloured \
             pixels; its parts carry at least two colours."
        )));
    }

    let Some((view, vp)) = declared_in(&trace, ui_rect, VIEW_REGION) else {
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region beside the placed model (look for `add-3d`)."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(view))?;
    session.settle(30);
    pointer.gone(session)?;
    let trace = session.trace()?;
    let opened = trace
        .events("model-view-opened")
        .last()
        .map(|l| l.raw.clone());
    let rendered = trace.events("model-view-rendered").last();
    let viewer_hues = rendered.and_then(|l| l.get("hues")?.parse::<usize>().ok());
    report.note(format!(
        "viewer: {opened:?}; rendered {:?}",
        rendered.map(|l| l.raw.clone())
    ));
    let line = trace.events("model-view-opened").last();
    let part_counts = line.and_then(|l| {
        Some((
            l.get("parts")?.parse::<usize>().ok()?,
            l.get("uncoloured")?.parse::<usize>().ok()?,
        ))
    });
    match (part_counts, viewer_hues) {
        (None, _) => Ok(Some(
            "View… opened no viewer, or its line counts no uncoloured parts.".to_owned(),
        )),
        (Some((parts, uncoloured)), _) if uncoloured >= parts => Ok(Some(format!(
            "the viewer counts all {parts} parts uncoloured; this model colours them."
        ))),
        (_, Some(n)) if n >= 2 => Ok(None),
        (_, n) => Ok(Some(format!(
            "the viewer drew the model in {n:?} hue(s); its parts carry at least two colours."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grey_has_no_sector_and_primaries_land_apart() {
        let px = |r, g, b| Rgb { r, g, b };
        assert_eq!(sector(px(128, 128, 128)), None);
        assert_eq!(sector(px(190, 192, 200)), None);
        assert_eq!(sector(px(220, 20, 20)), Some(0));
        assert_eq!(sector(px(20, 220, 20)), Some(4));
        assert_eq!(sector(px(20, 20, 220)), Some(8));
    }
}
