//! `checks::model_sphere` — **the 3D viewer saves a 360° picture a photo
//! viewer can look around in**
//!
//! Places `assembly.prc` as `checks::model_view_window` does, opens it with
//! *View…*, switches the Picture tab's 360° picture on and picks the smallest
//! size, then presses *Save picture…*.
//!
//! Oracles:
//! - the file is a JPEG whose frame header says `WIDE` × `HIGH`: a picture at
//!   the screen's size, or a PNG, is neither;
//! - it carries the photo sphere tag, naming the equirectangular projection
//!   and the same full width;
//! - the `model-view-poster` line says the 360° lens, and its camera looks
//!   square to its up: the viewer opens on a pitched orbit, which a camera
//!   left unlevelled would show as a tilted horizon.

use crate::checks::model_view_poster::click;
use crate::checks::model_view_window::{
    MODEL, VIEW_REGION, launch_with_model_env, press_insert, ui_rect,
};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// `dialogs::model3d::shot::REGION_SHOT_SPHERE`.
const SPHERE: &str = "model3d.shot.sphere";
/// `dialogs::model3d::shot::REGION_SHOT_SPHERE_SIZE`.
const SIZE: &str = "model3d.shot.sphere_size";
/// The viewer's *Save picture…* button.
const SAVE_REGION: &str = "model3d.save_picture";
/// The picture picker's diagnostic answer.
const PICTURE_ENV: &str = "PDFCER_DIAG_PICTURE_SAVE_PATH";
/// The smallest size, pixels: not the default, so the pick is what sets it.
const WIDE: usize = 2048;
const HIGH: usize = 1024;
/// How far from square the look and up may be: the trace's four decimals.
const LEVEL: f64 = 1e-3;
/// Steps waited for the picture, drawn off the screen's thread.
const WAITS: usize = 60;

/// See the module documentation.
pub struct TheViewerSavesA360Picture;

impl Check for TheViewerSavesA360Picture {
    fn name(&self) -> &'static str {
        "the_3d_viewer_saves_a_360_picture_for_photo_viewers"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's 360° picture is not a tagged 2:1 JPEG at the size picked, or is drawn \
         through another lens, or from a camera not levelled onto the model's up"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let target = ctx.out("model-sphere.jpg");
        let _ = std::fs::remove_file(&target);
        let extra = [(PICTURE_ENV, target.to_string_lossy().into_owned())];
        let driven = launch_with_model_env(ctx, &mut report, "model-sphere", MODEL, &extra)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer))
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

/// Click `region` on the open options menu; `false` when it is not drawn.
fn press(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
) -> Result<bool> {
    let Some((rect, vp)) = viewer_options::reach(session, pointer, ui_rect, region)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(15);
    Ok(true)
}

/// Switch 360° on, pick the size, save, and read the camera; why not, if not.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let ui_rect = ui_rect(ctx)?;
    if !click(session, pointer, ui_rect, VIEW_REGION)? {
        return Ok(Some(format!("no `{VIEW_REGION}`: View… was not drawn.")));
    }
    let item = format!("{SIZE}.small");
    for region in [SPHERE, SIZE, item.as_str()] {
        if !press(session, pointer, ui_rect, region)? {
            return Ok(Some(format!("no `{region}` on the Picture tab.")));
        }
    }
    viewer_options::close(session, pointer, ui_rect)?;
    if !click(session, pointer, ui_rect, SAVE_REGION)? {
        return Ok(Some(format!("no `{SAVE_REGION}` button.")));
    }
    let mut saved = None;
    for _ in 0..WAITS {
        session.settle(15);
        let trace = session.trace()?;
        saved = trace
            .events("model-picture-saved")
            .last()
            .map(|l| l.raw.clone());
        if saved.is_some() {
            break;
        }
    }
    report.note(format!("saved: {saved:?}"));
    let Some(saved) = saved else {
        return Ok(Some("the press saved no picture.".to_owned()));
    };
    if !saved.contains("sphere=1") {
        return Ok(Some(format!("the save was not a 360° one: {saved}")));
    }
    let trace = session.trace()?;
    let poster = trace
        .events("model-view-poster")
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("poster: {poster:?}"));
    pointer.gone(session)?;
    Ok(poster.map_or_else(
        || Some("no `model-view-poster` line.".to_owned()),
        |line| camera_failure(&line),
    ))
}

/// Why `line`'s camera is not a levelled 360° one, if it is not.
fn camera_failure(line: &str) -> Option<String> {
    if !line.contains("lens=fisheye-equirectangular fov=360.000") {
        return Some(format!(
            "the picture was not drawn through the 360° lens: {line}"
        ));
    }
    if !line.contains(" w=2048 h=1024 ") || !line.contains(" sphere=1") {
        return Some(format!(
            "the picture was not drawn at the size picked: {line}"
        ));
    }
    let vector = |key: &str| -> Option<Vec<f64>> {
        let (_, rest) = line.split_once(key)?;
        rest.split_whitespace()
            .next()?
            .split(',')
            .map(|c| c.parse().ok())
            .collect()
    };
    let (Some(dir), Some(up)) = (vector(" dir="), vector(" up=")) else {
        return Some(format!("no camera on the poster line: {line}"));
    };
    let square: f64 = dir.iter().zip(&up).map(|(a, b)| a * b).sum();
    (square.abs() > LEVEL)
        .then(|| format!("the 360° camera is not level: dir·up = {square:.4} in {line}"))
}

/// The width and height in the JPEG's first frame header.
fn jpeg_size(bytes: &[u8]) -> Option<[usize; 2]> {
    let mut at = 2;
    loop {
        let marker = *bytes.get(at + 1)?;
        let length = usize::from(*bytes.get(at + 2)?) << 8 | usize::from(*bytes.get(at + 3)?);
        if matches!(marker, 0xC0..=0xC2) {
            let side = |i: usize| -> Option<usize> {
                Some(usize::from(*bytes.get(at + i)?) << 8 | usize::from(*bytes.get(at + i + 1)?))
            };
            return Some([side(7)?, side(5)?]);
        }
        at += 2 + length;
    }
}

/// Read the saved file back.
fn judge(report: &mut CheckReport, target: &std::path::Path) -> Result<Option<String>> {
    let bytes = std::fs::read(target)
        .map_err(|e| Error::new(format!("could not read {}: {e}", target.display())))?;
    report.artifact(target.to_path_buf());
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return Ok(Some(format!(
            "the saved file is not a JPEG: it starts {:02X?}.",
            &bytes[..bytes.len().min(4)]
        )));
    }
    let size = jpeg_size(&bytes);
    report.note(format!("{} bytes, frame {size:?}", bytes.len()));
    if size != Some([WIDE, HIGH]) {
        return Ok(Some(format!(
            "the JPEG's frame is {size:?}; wanted {WIDE} × {HIGH}."
        )));
    }
    let text = String::from_utf8_lossy(&bytes);
    let projection = text.contains("GPano:ProjectionType>equirectangular<");
    let full = text.contains(&format!("GPano:FullPanoWidthPixels>{WIDE}<"));
    report.note(format!("tag: projection {projection}, full width {full}"));
    if !projection || !full {
        return Ok(Some(format!(
            "the JPEG lacks the photo sphere tag: projection {projection}, full width {full}."
        )));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_header_is_found_past_other_segments() {
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1, 0, 4, 9, 9];
        jpeg.extend([0xFF, 0xC0, 0, 17, 8, 0x04, 0x00, 0x08, 0x00]);
        assert_eq!(jpeg_size(&jpeg), Some([2048, 1024]));
        assert_eq!(jpeg_size(&[0xFF, 0xD8]), None);
    }

    #[test]
    fn a_pitched_or_other_lens_camera_is_refused() {
        let good =
            "x w=2048 h=1024 sphere=1 dir=1,0,0 up=0,0,1 lens=fisheye-equirectangular fov=360.000";
        assert_eq!(camera_failure(good), None);
        assert!(camera_failure(&good.replace("dir=1,0,0", "dir=0.8,0,-0.6")).is_some());
        assert!(camera_failure(&good.replace("fov=360.000", "fov=180.000")).is_some());
        assert!(camera_failure(&good.replace("w=2048", "w=4096")).is_some());
    }
}
