//! `checks::model_poster` — **a 3D model placed from the ribbon shows a
//! picture of the model on the page, not a stand-in box**
//!
//! Drives the window off the desktop through the scripted pointer and places
//! the engine corpus's `assembly.prc` as `checks::model_view_window` does.
//!
//! The oracle is the window's own frame before and after the insert. The
//! engine's stand-in is a frame and a wireframe cube: thin lines that ink a
//! few percent of the box they span. A rendered model is shaded solid, so most
//! pixels the insert changed on the page must have all eight neighbours
//! changed too; a sparse model's bounding box says nothing either way. Only the page is compared: the insert also adds a row to the
//! Attachments panel and a status line. The `model-insert-poster` line is noted, never judged.

use crate::checks::driving::declared;
use crate::checks::model_view_window::{launch_with_model, press_insert};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::PixRect;
use crate::image::{Image, Rgb};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// A channel difference below this is antialiasing or rounding, not a change.
const TOLERANCE: u8 = 24;
/// The share of changed pixels whose eight neighbours also changed. A shaded
/// surface is mostly interior; a one- or two-pixel line has almost none.
const MIN_SOLID: f64 = 0.4;
/// Fewer changed pixels than this is no visible poster at all.
const MIN_CHANGED: usize = 2_000;
/// The first page's region on the canvas.
const PAGE_REGION: &str = "page";

/// See the module documentation.
pub struct AnInsertedModelShowsItsPicture;

impl Check for AnInsertedModelShowsItsPicture {
    fn name(&self) -> &'static str {
        "an_inserted_3d_model_shows_its_picture_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "a 3D model placed on the page shows the engine's stand-in, a frame with a wireframe \
         cube, instead of a picture of the model"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model(ctx, &mut report, "model-poster")
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

fn differs(a: Rgb, b: Rgb) -> bool {
    a.r.abs_diff(b.r) > TOLERANCE || a.g.abs_diff(b.g) > TOLERANCE || a.b.abs_diff(b.b) > TOLERANCE
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    pointer.gone(session)?;
    session.settle(10);
    let before = shot(ctx, report, session, pointer, "model-poster-before.png")?;
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let after = shot(ctx, report, session, pointer, "model-poster-after.png")?;
    let trace = session.trace()?;
    let poster = trace
        .events("model-insert-poster")
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("poster line: {poster:?}"));
    if trace.events("add-3d").count() == 0 {
        return Ok(Some(
            "the model was not placed (no `add-3d` line).".to_owned(),
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let Some(page) = declared(&trace, ui_rect, PAGE_REGION) else {
        return Ok(Some(format!("no `{PAGE_REGION}` region on the canvas.")));
    };
    let page = session.frame()?.logical_to_capture_pixels(page);
    Ok(judge(report, &before, &after, page))
}

/// Enough of the changed pixels are interior to be a shaded picture.
fn judge(report: &mut CheckReport, before: &Image, after: &Image, page: PixRect) -> Option<String> {
    let w = before.width().min(after.width()).min(page.x + page.w);
    let h = before.height().min(after.height()).min(page.y + page.h);
    let changed = |x: u32, y: u32| {
        x >= page.x
            && y >= page.y
            && x < w
            && y < h
            && before
                .pixel(x, y)
                .zip(after.pixel(x, y))
                .is_some_and(|(a, b)| differs(a, b))
    };
    let (mut count, mut interior) = (0_usize, 0_usize);
    for y in page.y..h {
        for x in page.x..w {
            if !changed(x, y) {
                continue;
            }
            count += 1;
            let surrounded = (x > 0 && y > 0)
                && (y - 1..=y + 1).all(|ny| (x - 1..=x + 1).all(|nx| changed(nx, ny)));
            interior += usize::from(surrounded);
        }
    }
    if count < MIN_CHANGED {
        return Some(format!(
            "★★★ placing the model changed only {count} pixels: nothing visible was drawn."
        ));
    }
    #[allow(clippy::cast_precision_loss)]
    let solid = interior as f64 / count as f64;
    report.note(format!(
        "changed {count} pixels, {interior} interior; solid {solid:.3}"
    ));
    (solid < MIN_SOLID).then(|| {
        format!(
            "★★★ only {solid:.3} of the placed model's pixels are interior, under {MIN_SOLID}: the page shows thin lines, the stand-in frame and cube, not a shaded picture of the model."
        )
    })
}
