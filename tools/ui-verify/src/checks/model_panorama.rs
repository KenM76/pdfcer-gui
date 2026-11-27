//! `checks::model_panorama` — **the 3D viewer saves a partial panorama,
//! tilted with the view, tagged with its place on the sphere**
//!
//! Places `assembly.prc` as `checks::model_view_window` does, opens it with
//! *View…*, switches the Picture tab's panorama on, picks the smallest longer
//! side, types `ACROSS`° across and `UP_DOWN`° up and down, switches *Tilt
//! with the view* on and presses *Save picture…*.
//!
//! Oracles:
//! - the `model-view-poster` line names the panorama lens at those angles,
//!   with a pitch off the horizon: the viewer opens on a pitched orbit, so a
//!   build that ignored the tilt draws pitch 0;
//! - the file is a JPEG `WIDE` × `HIGH`, the angles' 3:1, where the whole
//!   sphere's 2:1 or the screen's shape is neither;
//! - its photo-sphere tag places the strip: the full width is `WIDE` × 360 /
//!   `ACROSS`, and the crop's top row is where the poster line's pitch puts
//!   the band's upper edge, so a tag written for the whole sphere or for an
//!   untilted band fails.

use crate::checks::model_picture_size::type_into;
use crate::checks::model_sphere::{jpeg_size, press};
use crate::checks::model_view_poster::click;
use crate::checks::model_view_window::{
    MODEL, VIEW_REGION, launch_with_model_env, press_insert, ui_rect,
};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// `dialogs::model3d::shot::REGION_SHOT_SPHERE`.
const SPHERE: &str = "model3d.shot.sphere";
/// `dialogs::model3d::shot::REGION_SHOT_SPHERE_SIZE`.
const SIZE: &str = "model3d.shot.sphere_size";
/// `dialogs::model3d::shot::REGION_SHOT_PANO_ACROSS`.
const ACROSS_ENTRY: &str = "model3d.shot.pano_across";
/// `dialogs::model3d::shot::REGION_SHOT_PANO_UP_DOWN`.
const UP_DOWN_ENTRY: &str = "model3d.shot.pano_up_down";
/// `dialogs::model3d::shot::REGION_SHOT_PANO_TILT`.
const TILT: &str = "model3d.shot.pano_tilt";
/// The viewer's *Save picture…* button.
const SAVE_REGION: &str = "model3d.save_picture";
/// The picture picker's diagnostic answer.
const PICTURE_ENV: &str = "PDFCER_DIAG_PICTURE_SAVE_PATH";
/// The angles typed, degrees: neither is a default.
const ACROSS: u32 = 120;
const UP_DOWN: u32 = 40;
/// The picture they give at the smallest longer side, pixels.
const WIDE: usize = 2048;
const HIGH: usize = 683;
/// The least pitch counted as tilted, degrees.
const TILTED: f64 = 1.0;
/// Steps waited for the picture, drawn off the screen's thread.
const WAITS: usize = 60;

/// See the module documentation.
pub struct TheViewerSavesAPartialPanorama;

impl Check for TheViewerSavesAPartialPanorama {
    fn name(&self) -> &'static str {
        "the_3d_viewer_saves_a_partial_panorama_tilted_with_the_view"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's panorama is not drawn at the angles typed, is not tilted with the view \
         when asked, or its JPEG's photo-sphere tag does not place the strip on the sphere"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let target = ctx.out("model-panorama.jpg");
        let _ = std::fs::remove_file(&target);
        let extra = [(PICTURE_ENV, target.to_string_lossy().into_owned())];
        let driven = launch_with_model_env(ctx, &mut report, "model-panorama", MODEL, &extra)
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer))
            .and_then(|outcome| match outcome {
                Err(failure) => Ok(Some(failure)),
                Ok(pitch) => judge(&mut report, &target, pitch),
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Set the panorama up, save it, and read the band's pitch off the poster
/// line; why not, if not.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<std::result::Result<f64, String>> {
    if !press_insert(ctx, session, pointer)? {
        return Ok(Err("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    let ui_rect = ui_rect(ctx)?;
    if !click(session, pointer, ui_rect, VIEW_REGION)? {
        return Ok(Err(format!("no `{VIEW_REGION}`: View… was not drawn.")));
    }
    let item = format!("{SIZE}.small");
    for region in [SPHERE, SIZE, item.as_str()] {
        if !press(session, pointer, ui_rect, region)? {
            return Ok(Err(format!("no `{region}` on the Picture tab.")));
        }
    }
    for (entry, value) in [(ACROSS_ENTRY, ACROSS), (UP_DOWN_ENTRY, UP_DOWN)] {
        if !type_into(session, pointer, ui_rect, entry, value)? {
            return Ok(Err(format!("★★★ no `{entry}` entry on the Picture tab.")));
        }
    }
    if !press(session, pointer, ui_rect, TILT)? {
        return Ok(Err(format!("★★★ no `{TILT}` switch on the Picture tab.")));
    }
    viewer_options::close(session, pointer, ui_rect)?;
    if !click(session, pointer, ui_rect, SAVE_REGION)? {
        return Ok(Err(format!("no `{SAVE_REGION}` button.")));
    }
    let mut saved = None;
    for _ in 0..WAITS {
        session.settle(15);
        saved = session
            .trace()?
            .events("model-picture-saved")
            .last()
            .map(|l| l.raw.clone());
        if saved.is_some() {
            break;
        }
    }
    report.note(format!("saved: {saved:?}"));
    if saved.is_none() {
        return Ok(Err("the press saved no picture.".to_owned()));
    }
    let poster = session
        .trace()?
        .events("model-view-poster")
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("poster: {poster:?}"));
    pointer.gone(session)?;
    Ok(poster.map_or_else(
        || Err("no `model-view-poster` line.".to_owned()),
        |line| lens_pitch(&line),
    ))
}

/// The band's pitch on `line`, when its lens is the panorama at the angles
/// typed and tilted off the horizon.
fn lens_pitch(line: &str) -> std::result::Result<f64, String> {
    let lens = format!("lens=panorama-equirectangular h={ACROSS}.000 v={UP_DOWN}.000 pitch=");
    let Some((_, rest)) = line.split_once(&lens) else {
        return Err(format!(
            "★★★ the picture was not drawn through a {ACROSS}° × {UP_DOWN}° panorama: {line}"
        ));
    };
    let pitch: f64 = rest
        .split_whitespace()
        .next()
        .and_then(|p| p.parse().ok())
        .ok_or_else(|| format!("no pitch after the lens: {line}"))?;
    if pitch.abs() < TILTED {
        return Err(format!(
            "★★★ the band's middle is {pitch}° off the horizon with Tilt with the view on, \
             from a pitched view: {line}"
        ));
    }
    Ok(pitch)
}

/// The number in `<GPano:key>…<` in `text`.
fn gpano(text: &str, key: &str) -> Option<i64> {
    let (_, rest) = text.split_once(&format!("<GPano:{key}>"))?;
    rest.split('<').next()?.trim().parse().ok()
}

/// The crop's top row the spec gives for a band centred `pitch`° above the
/// horizon in a full sphere `full_high` rows tall.
fn top_for(pitch: f64, full_high: i64) -> i64 {
    let upper = pitch + f64::from(UP_DOWN) / 2.0;
    ((90.0 - upper) / 180.0 * full_high as f64).round() as i64
}

/// Read the saved file back against the band drawn.
fn judge(report: &mut CheckReport, target: &std::path::Path, pitch: f64) -> Result<Option<String>> {
    let bytes = std::fs::read(target)
        .map_err(|e| Error::new(format!("could not read {}: {e}", target.display())))?;
    report.artifact(target.to_path_buf());
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return Ok(Some("the saved file is not a JPEG.".to_owned()));
    }
    let size = jpeg_size(&bytes);
    report.note(format!("{} bytes, frame {size:?}", bytes.len()));
    if size != Some([WIDE, HIGH]) {
        return Ok(Some(format!(
            "★★★ the JPEG's frame is {size:?}; {ACROSS}° × {UP_DOWN}° at the smallest size is \
             {WIDE} × {HIGH}."
        )));
    }
    let text = String::from_utf8_lossy(&bytes);
    let [full_w, full_h, crop_w, top] = [
        "FullPanoWidthPixels",
        "FullPanoHeightPixels",
        "CroppedAreaImageWidthPixels",
        "CroppedAreaTopPixels",
    ]
    .map(|k| gpano(&text, k));
    report.note(format!(
        "tag: full {full_w:?} x {full_h:?}, crop width {crop_w:?}, top {top:?}; pitch {pitch}"
    ));
    let wide = (WIDE as f64 * 360.0 / f64::from(ACROSS)).round() as i64;
    if full_w != Some(wide) || crop_w != Some(WIDE as i64) {
        return Ok(Some(format!(
            "★★★ the tag's full width is {full_w:?} and crop width {crop_w:?}; a {ACROSS}° strip \
             {WIDE} wide sits in a sphere {wide} wide."
        )));
    }
    let (Some(full_h), Some(top)) = (full_h, top) else {
        return Ok(Some("the tag has no full height or crop top.".to_owned()));
    };
    let want = top_for(pitch, full_h);
    Ok(((top - want).abs() > 1).then(|| {
        format!(
            "★★★ the tag's crop top is {top}; a band centred {pitch}° above the horizon starts \
             at row {want} of {full_h}."
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untilted_or_other_lens_band_is_refused() {
        let good = "x lens=panorama-equirectangular h=120.000 v=40.000 pitch=-21.500 dir=1,0,0";
        assert_eq!(lens_pitch(good), Ok(-21.5));
        assert!(lens_pitch(&good.replace("pitch=-21.500", "pitch=0.000")).is_err());
        assert!(lens_pitch(&good.replace("h=120.000", "h=360.000")).is_err());
        assert!(lens_pitch("x lens=fisheye-equirectangular fov=360.000").is_err());
    }

    #[test]
    fn the_crop_top_follows_the_pitch() {
        // A level band's upper edge is 20° up: 70/180 of the way down.
        assert_eq!(top_for(0.0, 1800), 700);
        assert_eq!(top_for(-30.0, 1800), 1000);
        assert_eq!(
            gpano(
                "<GPano:CroppedAreaTopPixels>12</GPano:",
                "CroppedAreaTopPixels"
            ),
            Some(12)
        );
    }
}
