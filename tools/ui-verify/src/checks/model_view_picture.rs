//! `checks::model_view_picture` — **the 3D viewer saves the view on screen
//! as a PNG picture**
//!
//! Places the engine corpus's `assembly.prc` as `checks::model_view_window`
//! does, with a path in this run's output folder as the picture picker's
//! answer (`PDFCER_DIAG_PICTURE_SAVE_PATH`). Opens it with *View…*, turns to
//! the Top view, presses *Save picture…* and closes the viewer.
//!
//! Oracles: a `model-picture-saved` line; the file decodes as PNG with its
//! long side `LONG_SIDE` pixels; and at least `MIN_DRAWN` of its pixels are
//! not the white background, so the model is in it.

use crate::checks::model_view_poster::click;
use crate::checks::model_view_window::{
    MODEL, VIEW_REGION, launch_with_model_env, press_insert, ui_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The viewer's Top view button.
const TOP_REGION: &str = "model3d.view.3";
/// The viewer's *Save picture…* button.
const SAVE_REGION: &str = "model3d.save_picture";
/// The viewer's Close button.
const CLOSE_REGION: &str = "model3d.close";
/// The picture picker's diagnostic answer, read by `app::files::pick_picture_target`.
const PICTURE_ENV: &str = "PDFCER_DIAG_PICTURE_SAVE_PATH";
/// The saved picture's long side: `dialogs::model3d::POSTER_SIDE`.
const LONG_SIDE: u32 = 1200;
/// A channel this far below 255 is not the white background.
const BELOW_WHITE: u8 = 40;
/// Fewer drawn pixels than this is no model in the picture.
const MIN_DRAWN: usize = 5_000;

/// See the module documentation.
pub struct TheViewerSavesItsViewAsAPicture;

impl Check for TheViewerSavesItsViewAsAPicture {
    fn name(&self) -> &'static str {
        "the_3d_viewer_saves_its_view_as_a_picture"
    }

    fn defect(&self) -> &'static str {
        "Save picture… in the 3D viewer writes no file, or a file that is not a PNG, or a picture \
         with no model in it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let target = ctx.out("model-view-picture-saved.png");
        // A file left by an earlier run must not satisfy this one.
        let _ = std::fs::remove_file(&target);
        let extra = [(PICTURE_ENV, target.to_string_lossy().into_owned())];
        let driven = launch_with_model_env(ctx, &mut report, "model-view-picture", MODEL, &extra)
            .and_then(|(session, pointer)| drive(ctx, &session, &pointer))
            .and_then(|failure| match failure {
                Some(failure) => Ok(Some(failure)),
                None => judge(&mut report, &target),
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let ui_rect = ui_rect(ctx)?;
    for (region, what) in [
        (VIEW_REGION, "View… beside the placed model"),
        (TOP_REGION, "the viewer's Top button"),
        (SAVE_REGION, "the viewer's Save picture… button"),
        (CLOSE_REGION, "the viewer's Close button"),
    ] {
        if !click(session, pointer, ui_rect, region)? {
            return Ok(Some(format!("no `{region}` region: {what} was not drawn.")));
        }
    }
    session.settle(30);
    let trace = session.trace()?;
    if trace.events("model-picture-saved").count() == 0 {
        let failed = trace
            .events("model-picture-failed")
            .last()
            .map(|l| l.raw.clone());
        return Ok(Some(format!(
            "the press saved no picture (`model-picture-saved` absent; failure line {failed:?})."
        )));
    }
    Ok(None)
}

/// Read the saved file back and hold it to the oracles.
fn judge(report: &mut CheckReport, target: &std::path::Path) -> Result<Option<String>> {
    if !target.is_file() {
        return Ok(Some(format!("no file at {}.", target.display())));
    }
    report.artifact(target.to_path_buf());
    let picture = match Image::load_png(target) {
        Ok(picture) => picture,
        Err(why) => return Ok(Some(format!("the saved file is not a PNG: {why}"))),
    };
    let (w, h) = (picture.width(), picture.height());
    let drawn = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            picture.pixel(x, y).is_some_and(|p| {
                p.r < 255 - BELOW_WHITE || p.g < 255 - BELOW_WHITE || p.b < 255 - BELOW_WHITE
            })
        })
        .count();
    report.note(format!("saved picture {w} x {h}, {drawn} pixels drawn"));
    if w.max(h) != LONG_SIDE {
        return Ok(Some(format!(
            "the picture is {w} x {h}; its long side should be {LONG_SIDE}."
        )));
    }
    Ok((drawn < MIN_DRAWN).then(|| {
        format!("the picture has {drawn} drawn pixels, under {MIN_DRAWN}: no model in it.")
    }))
}
