//! `checks::model_picture_size` — **the 3D viewer's Picture tab sets the
//! size and resolution *Save picture…* draws at**
//!
//! Places the engine corpus's `assembly.prc` as `checks::model_view_window`
//! does, with this run's output folder as the picture picker's answer, and
//! opens it with *View…*. On the options menu's Picture tab it clicks the
//! scale slider's far end, types `WIDTH` into the width entry and `DPI`
//! into the resolution entry, then presses *Save picture…*.
//!
//! Oracles:
//! - after the slider, the `model-view-picture-size` line's scale is above
//!   `SLID`: the slider moved the size, not only its own knob;
//! - the saved PNG is `WIDTH` wide and as tall as the picture on screen's
//!   shape makes it (`base=` on that line), within one pixel of rounding —
//!   a height left at the screen's or the slider's value is neither;
//! - its `pHYs` chunk records `DPI` in pixels per metre, the unit PNG
//!   writes; a picture saved without the resolution has no such chunk.

use crate::checks::model_view_poster::click;
use crate::checks::model_view_window::{
    MODEL, VIEW_REGION, launch_with_model_env, press_insert, ui_rect,
};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// `dialogs::model3d::shot::REGION_SHOT_SCALE`: the slider's rail.
const SCALE: &str = "model3d.shot.scale";
/// `dialogs::model3d::shot::REGION_SHOT_WIDTH`.
const WIDTH_ENTRY: &str = "model3d.shot.width";
/// `dialogs::model3d::shot::REGION_SHOT_DPI`.
const DPI_ENTRY: &str = "model3d.shot.dpi";
/// The viewer's *Save picture…* button.
const SAVE_REGION: &str = "model3d.save_picture";
/// The picture picker's diagnostic answer, read by `app::files::pick_picture_target`.
const PICTURE_ENV: &str = "PDFCER_DIAG_PICTURE_SAVE_PATH";
/// The Picture tab's trace line.
const SIZE_LINE: &str = "model-view-picture-size";
/// The width typed, pixels: neither 1× nor 8× of any window this runs in.
const WIDTH: u32 = 1234;
/// The resolution typed, pixels per inch.
const DPI: u32 = 300;
/// A scale the slider's far end must pass: the slider starts at 1×.
const SLID: f64 = 2.0;
/// Frames waited for the picture, drawn off the screen's thread, in steps.
const WAITS: usize = 40;

/// See the module documentation.
pub struct TheViewerSavesAPictureAtTheSizeSet;

impl Check for TheViewerSavesAPictureAtTheSizeSet {
    fn name(&self) -> &'static str {
        "the_3d_viewer_saves_a_picture_at_the_size_and_resolution_set"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Picture tab does not set what Save picture… writes: the slider leaves \
         the size, the height does not follow the width at the screen's shape, or the PNG \
         carries no resolution"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let target = ctx.out("model-picture-size.png");
        let _ = std::fs::remove_file(&target);
        let extra = [(PICTURE_ENV, target.to_string_lossy().into_owned())];
        let driven = launch_with_model_env(ctx, &mut report, "model-picture-size", MODEL, &extra)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer))
            .and_then(|outcome| match outcome {
                Err(failure) => Ok(Some(failure)),
                Ok(base) => judge(&mut report, &target, base),
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Drive the tab and save; the picture on screen's `[w, h]`, or why not.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<[u32; 2], String>> {
    if !press_insert(ctx, session, pointer)? {
        return Ok(Err("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let ui_rect = ui_rect(ctx)?;
    if !click(session, pointer, ui_rect, VIEW_REGION)? {
        return Ok(Err(format!("no `{VIEW_REGION}`: View… was not drawn.")));
    }
    let Some((rail, vp)) = viewer_options::reach(session, pointer, ui_rect, SCALE)? else {
        return Ok(Err(format!("★★★ no `{SCALE}` slider on any options tab.")));
    };
    let mut end = rail;
    end.min.x = end.max.x - 2.0;
    let end = WindowPoint::centre_of(end);
    pointer.click_in(session, vp.as_deref(), end)?;
    session.settle(20);
    let slid = size_line(session)?;
    report.note(format!("after the slider: {slid:?}"));
    let scale = slid.as_ref().and_then(|l| field(l, "scale")).unwrap_or(0.0);
    if scale <= SLID {
        return Ok(Err(format!(
            "★★★ clicking the scale slider's far end left the scale at {scale} (line {slid:?}); \
             expected above {SLID}."
        )));
    }
    for (entry, value) in [(WIDTH_ENTRY, WIDTH), (DPI_ENTRY, DPI)] {
        if !type_into(session, pointer, ui_rect, entry, value)? {
            return Ok(Err(format!("no `{entry}` entry on the Picture tab.")));
        }
    }
    let typed = size_line(session)?;
    report.note(format!("after typing: {typed:?}"));
    let base = typed
        .as_ref()
        .and_then(|l| l.split(" base=").nth(1))
        .and_then(|b| {
            let (w, h) = b.split_whitespace().next()?.split_once('x')?;
            Some([w.parse().ok()?, h.parse().ok()?])
        });
    let Some(base) = base else {
        return Ok(Err(format!(
            "no `base=` on the `{SIZE_LINE}` line: {typed:?}"
        )));
    };
    viewer_options::close(session, pointer, ui_rect)?;
    if !click(session, pointer, ui_rect, SAVE_REGION)? {
        return Ok(Err(format!("no `{SAVE_REGION}` button.")));
    }
    if !saved(session)? {
        let failed = session
            .trace()?
            .events("model-picture-failed")
            .last()
            .map(|l| l.raw.clone());
        return Ok(Err(format!(
            "the press saved no picture (`model-picture-saved` absent; failure line {failed:?})."
        )));
    }
    Ok(Ok(base))
}

/// Scroll the open menu to `entry`, click it, select what it holds, type `value` and press Enter.
fn type_into(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    entry: &str,
    value: u32,
) -> Result<bool> {
    let Some((rect, vp)) = viewer_options::reach(session, pointer, ui_rect, entry)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(10);
    pointer.key(session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(session, vp.as_deref(), &value.to_string())?;
    pointer.key(session, vp.as_deref(), "Enter", None)?;
    session.settle(15);
    Ok(true)
}

/// The newest `model-view-picture-size` line, raw.
fn size_line(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(SIZE_LINE)
        .last()
        .map(|l| l.raw.clone()))
}

/// `key=`'s number on `line`.
fn field(line: &str, key: &str) -> Option<f64> {
    line.split_whitespace()
        .find_map(|w| w.strip_prefix(key)?.strip_prefix('='))?
        .parse()
        .ok()
}

/// Wait for `model-picture-saved`, drawn off the screen's thread.
fn saved(session: &Session) -> Result<bool> {
    for _ in 0..WAITS {
        if session.trace()?.events("model-picture-saved").count() > 0 {
            return Ok(true);
        }
        session.settle(10);
    }
    Ok(session.trace()?.events("model-picture-saved").count() > 0)
}

/// Read the saved file back and hold it to the size and resolution.
fn judge(
    report: &mut CheckReport,
    target: &std::path::Path,
    base: [u32; 2],
) -> Result<Option<String>> {
    if !target.is_file() {
        return Ok(Some(format!("no file at {}.", target.display())));
    }
    report.artifact(target.to_path_buf());
    let picture = match Image::load_png(target) {
        Ok(picture) => picture,
        Err(why) => return Ok(Some(format!("the saved file is not a PNG: {why}"))),
    };
    let (w, h) = (picture.width(), picture.height());
    let tall = f64::from(WIDTH) * f64::from(base[1]) / f64::from(base[0].max(1));
    let ppm = physical(&std::fs::read(target)?);
    report.note(format!("saved {w} x {h}, screen {base:?}, pHYs {ppm:?}"));
    if w != WIDTH || (f64::from(h) - tall).abs() > 1.0 {
        return Ok(Some(format!(
            "★★★ the picture is {w} x {h}; typing {WIDTH} into the width on a {} x {} screen \
             picture should give {WIDTH} x {tall:.0}.",
            base[0], base[1]
        )));
    }
    let want = (f64::from(DPI) / 0.0254).round() as u32;
    Ok((ppm != Some((want, want, 1))).then(|| {
        format!(
            "★★★ the PNG records {ppm:?} pixels per unit; {DPI} dpi is ({want}, {want}, 1 = metre)."
        )
    }))
}

/// The `pHYs` chunk's x and y pixels per unit and its unit, when present.
fn physical(png: &[u8]) -> Option<(u32, u32, u8)> {
    let at = png.windows(4).position(|w| w == b"pHYs")? + 4;
    let word = |i: usize| -> Option<u32> {
        Some(u32::from_be_bytes(
            png.get(at + i..at + i + 4)?.try_into().ok()?,
        ))
    };
    Some((word(0)?, word(4)?, *png.get(at + 8)?))
}
