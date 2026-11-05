//! The lens steps of `the_3d_viewer_walks_and_its_gimbal_turns_the_view`.
//!
//! From the opening orbit view, where the model fills the picture, *Lens:
//! Fisheye* is drawn by the card (`gpu=1`, nothing declined) and still
//! shows the model, within a factor of the flat picture's covered pixels;
//! *Lens: Flat* puts it back.
//!
//! Walking, zoomed out to the flat limit, *Lens: Fisheye* draws a fisheye
//! on the card and Ctrl+wheel out reaches 180 degrees. The walk's eye sees
//! the fixture edge-on, so nothing there measures what is drawn; *Curve: Classic* redraws it equidistant at the same size, so a
//! lens change reaches the picture by itself and not only through a resize
//! or a zoom; *Lens: Flat* again brings the field back to the flat limit and
//! the picture back to a perspective. Flat is chosen last so the kept lens
//! preference is left at its default for the checks after this one.

use crate::checks::driving::declared_in;
use crate::checks::viewer_options;
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::{ScriptedPointer, xy};
use crate::launch::Session;
use crate::report::CheckReport;

const IMAGE: &str = "model3d.image";
/// `dialogs::model3d::lensmenu::REGION_LENS`.
const LENS: &str = "model3d.lens";
/// The aspect past which the walk narrows its vertical field.
const WIDEST: f64 = 16.0 / 9.0;

/// `dialogs::model3d::lensmenu::REGION_CURVE`.
const CURVE: &str = "model3d.lens.curve";

/// Pick `item` from the `combo` drop-down (the lens's `flat`, `fisheye`,
/// `auto`; the curve's `0`, `1`); `false` when either is drawn nowhere.
fn choose(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    combo: &str,
    item: &str,
) -> Result<bool> {
    let Some((rect, vp)) = viewer_options::reach(session, pointer, ui_rect, combo)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect, &format!("{combo}.{item}"))
    else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    viewer_options::close(session, pointer, ui_rect)?;
    session.settle(20);
    Ok(true)
}

/// The fields of the last picture line, `-` for an absent one.
fn last(session: &Session) -> Result<[String; 8]> {
    Ok(crate::checks::model_shadow_sun::rendered(
        session,
        [
            "w",
            "h",
            "lens",
            "fov",
            "fov-y",
            "gpu-declined",
            "gpu",
            "covered",
        ],
    )?
    .unwrap_or_else(|| std::array::from_fn(|_| "-".to_owned())))
}

fn zoom_out(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<bool> {
    let Some((image, vp)) = declared_in(&session.trace()?, ui_rect, IMAGE) else {
        return Ok(false);
    };
    let vp = vp.map_or(String::new(), |v| format!(" vp={v}"));
    for _ in 0..12 {
        let at = xy(WindowPoint::centre_of(image));
        pointer.send(session, &format!("wheel {at} -20 mods=ctrl{vp}"))?;
        session.settle(4);
    }
    session.settle(20);
    Ok(true)
}

/// The orbit step; see the module documentation.
pub(super) fn orbit(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let [.., flat] = last(session)?;
    if !choose(session, pointer, ui_rect, LENS, "fisheye")? {
        return Ok(Some(format!(
            "no `{LENS}.fisheye` choice in the viewer's options."
        )));
    }
    let [w, h, lens, fov, _, declined, gpu, covered] = last(session)?;
    report.note(format!(
        "orbit fisheye {w}x{h}: lens={lens} fov={fov} gpu={gpu} gpu-declined={declined} \
         covered={covered}; flat covered={flat}"
    ));
    // The same model through a lens of the same height covers about as
    // much; a face never drawn or never resampled leaves background only.
    let [c, f] = [&covered, &flat].map(|v| v.parse::<f64>().unwrap_or(0.0));
    let shown = f > 0.0 && c > f / 3.0 && c < f * 3.0;
    if !lens.starts_with("fisheye") || gpu != "1" || declined != "-" || !shown {
        return Ok(Some(format!(
            "from the orbit view, Lens: Fisheye drew lens={lens} gpu={gpu} \
             gpu-declined={declined} covered={covered}; expected a fisheye drawn by the card \
             (gpu=1, nothing declined) covering within a factor of 3 of the flat picture's \
             {flat} pixels."
        )));
    }
    if !choose(session, pointer, ui_rect, LENS, "flat")? {
        return Ok(Some(format!(
            "no `{LENS}.flat` choice in the viewer's options."
        )));
    }
    Ok(None)
}

/// The walk step; see the module documentation. `flat_widest` is the field
/// the flat walk was just measured zoomed out at.
pub(super) fn lens(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
    flat_widest: f64,
) -> Result<Option<String>> {
    if !choose(session, pointer, ui_rect, LENS, "fisheye")? {
        return Ok(Some(format!(
            "no `{LENS}.fisheye` choice in the viewer's options."
        )));
    }
    if !zoom_out(session, pointer, ui_rect)? {
        return Ok(Some(format!("no `{IMAGE}` region.")));
    }
    let [w, h, lens, fov, _, declined, gpu, covered] = last(session)?;
    let aspect = w.parse::<f64>().unwrap_or(0.0) / h.parse::<f64>().unwrap_or(1.0).max(1.0);
    let wanted = if aspect <= WIDEST {
        180.0
    } else {
        180.0 * WIDEST / aspect
    };
    report.note(format!(
        "fisheye zoomed out {w}x{h}: lens={lens} fov={fov} gpu={gpu} gpu-declined={declined} \
         covered={covered}, wanted {wanted:.3}"
    ));
    let reached = fov.parse::<f64>().is_ok_and(|f| (f - wanted).abs() < 0.01);
    if lens != "fisheye-stereographic" || !reached || gpu != "1" || declined != "-" {
        return Ok(Some(format!(
            "walking with Lens: Fisheye and zoomed out, the picture drew lens={lens} fov={fov} \
             gpu={gpu} gpu-declined={declined}; expected lens=fisheye-stereographic \
             fov={wanted:.3} drawn by the card (gpu=1, nothing declined)."
        )));
    }
    if !choose(session, pointer, ui_rect, CURVE, "1")? {
        return Ok(Some(format!("no `{CURVE}.1` choice with Lens: Fisheye.")));
    }
    let [w2, h2, lens, fov2, _, _, _, _] = last(session)?;
    report.note(format!("curve Classic {w2}x{h2}: lens={lens} fov={fov2}"));
    if lens != "fisheye-equidistant" || [&w2, &h2, &fov2] != [&w, &h, &fov] {
        return Ok(Some(format!(
            "Curve: Classic drew lens={lens} at {w2}x{h2} fov={fov2}; expected \
             lens=fisheye-equidistant at the same {w}x{h} fov={fov}."
        )));
    }
    if !choose(session, pointer, ui_rect, LENS, "flat")? {
        return Ok(Some(format!(
            "no `{LENS}.flat` choice in the viewer's options."
        )));
    }
    let [_, _, lens, _, fov_y, declined, _, _] = last(session)?;
    report.note(format!(
        "back to flat: lens={lens} fov-y={fov_y} gpu-declined={declined}"
    ));
    let back = fov_y
        .parse::<f64>()
        .is_ok_and(|f| (f - flat_widest).abs() < 0.01);
    if lens != "-" || !back || declined != "-" {
        return Ok(Some(format!(
            "Lens: Flat after a fisheye zoomed out drew lens={lens} fov-y={fov_y} \
             gpu-declined={declined}; expected a perspective at the flat widest, \
             {flat_widest:.3}, back on the card."
        )));
    }
    Ok(None)
}
