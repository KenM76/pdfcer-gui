//! `checks::model_shadow_sun` — **a shadow keeps its place on the model as
//! the eye moves**
//!
//! Places `fixtures/floor-box.prc`, a cube standing on a floor, opens it with
//! *View…* and drags across the picture to turn the model. Then, in the
//! options menu, sets *Shadows cast by* to *The lighting* and turns it again.
//!
//! Oracles, from the `model-view-rendered` line (the viewer is an immediate
//! viewport no screenshot reaches). `shadow-map` fingerprints the matrix
//! that takes the model into the shadow map, so an equal value at two eyes
//! means the shadow lies in the same model place.
//!
//! - Opened: `shadows=1 caster=sun`.
//! - After the first turn, the yaw differs and `shadow-map` does not; after
//!   a wheel zoom, the zoom differs and `shadow-map` still does not.
//! - After choosing *The lighting*, `caster=scheme`, and a second turn
//!   changes `shadow-map`. This is the control: the camera-frame light
//!   turns with the view, so a fingerprint that stayed put there would mean
//!   the oracle cannot see a moving shadow.

use crate::checks::driving::{declared_in, repo_fixture};
use crate::checks::model_view_poster::click;
use crate::checks::model_view_window::{
    VIEW_REGION, launch_with_model_file, press_insert, ui_rect,
};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "floor-box.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe>`.";
/// `dialogs::model3d::REGION_IMAGE`.
const IMAGE: &str = "model3d.image";
/// `dialogs::model3d::effects::REGION_SHADOW_LIGHT`.
const CASTER: &str = "model3d.shadow_light";
/// Its *The lighting* item.
const SCHEME: &str = "model3d.shadow_light.scheme";

/// See the module documentation.
pub struct ShadowsStayOnTheModel;

impl Check for ShadowsStayOnTheModel {
    fn name(&self) -> &'static str {
        "shadows_stay_on_the_model_as_the_eye_moves"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's shadows move round the model as the eye moves, instead of staying \
         where a fixed sun casts them"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture(MODEL, METHOD)
            .and_then(|model| {
                launch_with_model_file(ctx, &mut report, "shadow-sun", &model.to_string_lossy())
            })
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last `model-view-rendered` line's `shadows`, `caster`, `shadow-map`,
/// `yaw` and `zoom`; an absent field reads as `-`.
fn drawn(session: &Session) -> Result<Option<[String; 5]>> {
    let trace = session.trace()?;
    Ok(trace.events("model-view-rendered").last().map(|l| {
        ["shadows", "caster", "shadow-map", "yaw", "zoom"]
            .map(|k| l.get(k).unwrap_or("-").to_owned())
    }))
}

/// Drag across the picture to turn the model, and the picture line after.
fn turn(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<[String; 5]>> {
    let Some((image, vp)) = declared_in(&session.trace()?, ui_rect, IMAGE) else {
        return Ok(None);
    };
    let centre = WindowPoint::centre_of(image);
    let aside = Pt {
        x: centre.x() + 90.0,
        y: centre.y() + 20.0,
    };
    let to = WindowPoint::centre_of(LRect {
        min: aside,
        max: aside,
    });
    pointer.drag_in(session, vp.as_deref(), centre, to, 8, "l")?;
    session.settle(30);
    drawn(session)
}

/// Open the viewer on the placed model; a failure sentence when it cannot.
fn open(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    pointer.gone(session)?;
    session.settle(10);
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some("no 3D model button on the Edit tab.".to_owned()));
    }
    pointer.gone(session)?;
    session.settle(30);
    if !click(session, pointer, ui_rect(ctx)?, VIEW_REGION)? {
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region beside the placed model (look for `add-3d`)."
        )));
    }
    pointer.gone(session)?;
    Ok(None)
}

/// Choose *The lighting* as the caster; `false` when either the drop-down
/// or its item is drawn nowhere.
fn choose_scheme(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<bool> {
    let Some((combo, vp)) = viewer_options::reach(session, pointer, ui_rect, CASTER)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(combo))?;
    session.settle(20);
    let Some((item, vp)) = declared_in(&session.trace()?, ui_rect, SCHEME) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(item))?;
    session.settle(20);
    viewer_options::close(session, pointer, ui_rect)?;
    Ok(true)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = open(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    let ui_rect = ui_rect(ctx)?;
    let Some(first) = drawn(session)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!(
        "opened [shadows, caster, map, yaw, zoom]: {first:?}"
    ));
    if first[..2] != ["1", "sun"] || first[2] == "-" {
        return Ok(Some(format!(
            "the viewer opened with shadows={} caster={} shadow-map={}; expected shadows cast \
             by the sun.",
            first[0], first[1], first[2]
        )));
    }
    let Some(turned) = turn(session, pointer, ui_rect)? else {
        return Ok(Some(format!("no `{IMAGE}` picture to drag across.")));
    };
    report.note(format!("turned: {turned:?}"));
    if turned[3] == first[3] {
        return Ok(Some(
            "dragging across the picture did not turn the model.".to_owned(),
        ));
    }
    if turned[2] != first[2] {
        return Ok(Some(format!(
            "turning the model moved the sun's shadow map ({} to {}): the shadow does not stay \
             in one place on the model.",
            first[2], turned[2]
        )));
    }
    if let Some(failure) = zoom(report, session, pointer, ui_rect, &turned)? {
        return Ok(Some(failure));
    }
    control(report, session, pointer, ui_rect)
}

/// A wheel zoom on the picture leaves the sun's map where `before` had it.
fn zoom(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    before: &[String; 5],
) -> Result<Option<String>> {
    let Some((image, vp)) = declared_in(&session.trace()?, ui_rect, IMAGE) else {
        return Ok(Some(format!("no `{IMAGE}` picture to zoom.")));
    };
    pointer.wheel_in(session, vp.as_deref(), WindowPoint::centre_of(image), 3.0)?;
    session.settle(30);
    let Some(zoomed) = drawn(session)? else {
        return Ok(Some("no picture line after the zoom.".to_owned()));
    };
    report.note(format!("zoomed: {zoomed:?}"));
    if zoomed[4] == before[4] {
        return Ok(Some("the wheel on the picture did not zoom.".to_owned()));
    }
    if zoomed[2] != before[2] {
        return Ok(Some(format!(
            "zooming moved the sun's shadow map ({} to {}): the shadow does not stay in one \
             place on the model.",
            before[2], zoomed[2]
        )));
    }
    Ok(None)
}

/// The control: with the lighting casting, a turn moves the map.
fn control(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    if !choose_scheme(session, pointer, ui_rect)? {
        return Ok(Some(format!(
            "no `{CASTER}` drop-down with a `{SCHEME}` item in the options menu."
        )));
    }
    let Some(scheme) = drawn(session)? else {
        return Ok(Some(
            "no picture line after choosing The lighting.".to_owned(),
        ));
    };
    report.note(format!("the lighting casts: {scheme:?}"));
    if scheme[1] != "scheme" {
        return Ok(Some(format!(
            "choosing The lighting drew caster={}; the drop-down did not reach the picture.",
            scheme[1]
        )));
    }
    let Some(again) = turn(session, pointer, ui_rect)? else {
        return Ok(Some(format!("no `{IMAGE}` picture to drag across.")));
    };
    report.note(format!("turned again: {again:?}"));
    if again[3] == scheme[3] || again[2] == scheme[2] {
        return Ok(Some(format!(
            "with the lighting casting, a turn left yaw {} and shadow-map {} unchanged: the \
             camera-frame light should move the map, so the fingerprint cannot see a moving \
             shadow.",
            again[3], again[2]
        )));
    }
    Ok(None)
}
