//! `checks::model_view` — **a placed 3D model opens in the viewer and turns,
//! zooms and jumps to a named view under the scripted pointer**
//!
//! Drives the window off the desktop through the scripted pointer (no OS mouse
//! or keyboard). The model is the engine corpus's `assembly.prc`, placed from
//! the ribbon as `checks::models` does, then opened with *View…* on its
//! Attachments row.
//!
//! Oracles are the viewer's `model-view-rendered` lines: the first must cover
//! pixels (`covered>0`, the model is in frame); a primary drag must change
//! `yaw` and the picture's `hash`; a wheel must change `zoom`; the Top button
//! must set `pitch` near a right angle.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Edit mode with the Attachments panel showing.
const INVOKE: &str = "mode.edit,edit.attachments";
const RIBBON_TAB: &str = "ribbon.tab.edit";
const RIBBON_ITEM: &str = "ribbon.item.edit.insert_3d";
const VIEW_REGION: &str = "models.view";
const IMAGE_REGION: &str = "model3d.image";
/// The Top button; its index in the viewer's named views.
const TOP_REGION: &str = "model3d.view.3";
const MODEL_ENV: &str = "PDFCER_DIAG_MODEL_PATH";
const DOC: &str = "D:/Dev/pdfcer/fixtures/synthetic/pageops/four-pages.pdf";
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc";

/// See the module documentation.
pub struct AModelTurnsUnderThePointer;

impl Check for AModelTurnsUnderThePointer {
    fn name(&self) -> &'static str {
        "a_3d_model_turns_under_the_pointer"
    }

    fn defect(&self) -> &'static str {
        "View… on a PRC model opens no viewer, or the viewer draws nothing, or dragging does not \
         turn the model, or scrolling does not zoom, or the Top view does not look down"
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

/// The last `model-view-rendered` line's `key`, and how many there are.
fn rendered(session: &Session, key: &str) -> Result<(usize, Option<String>)> {
    let trace = session.trace()?;
    let lines: Vec<_> = trace.events("model-view-rendered").collect();
    let last = lines.last().and_then(|l| l.get(key).map(str::to_owned));
    Ok((lines.len(), last))
}

fn number(value: Option<&String>) -> Option<f64> {
    value.and_then(|v| v.parse().ok())
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    for needed in [DOC, MODEL] {
        if !std::path::Path::new(needed).is_file() {
            return Err(Error::new(format!(
                "the engine corpus's fixture is missing at {needed}."
            )));
        }
    }
    // Driven on copies: the sources belong to the engine repository.
    let doc = ctx.out("model-view-source.pdf");
    std::fs::copy(DOC, &doc).map_err(|e| Error::new(format!("copying {DOC}: {e}")))?;
    let model = ctx.out("model-view-input.prc");
    std::fs::copy(MODEL, &model).map_err(|e| Error::new(format!("copying {MODEL}: {e}")))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("model-view.trace.txt"));
    spec.pdf = Some(doc);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((MODEL_ENV.to_owned(), model.to_string_lossy().into_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("model-view.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let mut trace = session.trace()?;
    if declared_in(&trace, ui_rect, RIBBON_ITEM).is_none()
        && let Some((tab, vp)) = declared_in(&trace, ui_rect, RIBBON_TAB)
    {
        pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(tab))?;
        session.settle(15);
        trace = session.trace()?;
    }
    let Some((item, vp)) = declared_in(&trace, ui_rect, RIBBON_ITEM) else {
        return Ok(Some(format!(
            "no `{RIBBON_ITEM}` region: the 3D model button is not on the Edit tab."
        )));
    };
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(item))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some((view, vp)) = declared_in(&trace, ui_rect, VIEW_REGION) else {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region beside the placed model (was it placed? look for \
             `add-3d`)."
        )));
    };
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(view))?;
    session.settle(30);

    let trace = session.trace()?;
    let opened = trace
        .events("model-view-opened")
        .last()
        .map(|l| l.raw.clone());
    let (first_count, covered) = rendered(&session, "covered")?;
    report.note(format!(
        "opened: {opened:?}; renders={first_count}; covered={covered:?}"
    ));
    if opened.is_none() {
        pointer.gone(&session)?;
        return Ok(Some(
            "View… opened no viewer (look for `model-view-declined`).".to_owned(),
        ));
    }
    if number(covered.as_ref()).is_none_or(|c| c <= 0.0) {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "the viewer's picture covers no pixel ({covered:?}): the model is out of frame or \
             nothing was drawn (look for `model-view-render-failed`)."
        )));
    }
    let Some((image, ivp)) = declared_in(&trace, ui_rect, IMAGE_REGION) else {
        pointer.gone(&session)?;
        return Ok(Some(format!("no `{IMAGE_REGION}` region in the viewer.")));
    };
    let centre = WindowPoint::centre_of(image);
    let (_, yaw_before) = rendered(&session, "yaw")?;
    let (_, hash_before) = rendered(&session, "hash")?;
    // A point right of and below the centre, still on the picture.
    let aside = Pt {
        x: centre.x() + 80.0,
        y: centre.y() + 30.0,
    };
    let to = WindowPoint::centre_of(LRect {
        min: aside,
        max: aside,
    });
    pointer.drag_in(&session, ivp.as_deref(), centre, to, 8, "l")?;
    session.settle(20);
    let (_, yaw_after) = rendered(&session, "yaw")?;
    let (_, hash_after) = rendered(&session, "hash")?;
    report.note(format!(
        "drag: yaw {yaw_before:?} → {yaw_after:?}; hash {hash_before:?} → {hash_after:?}"
    ));
    if yaw_after == yaw_before || hash_after == hash_before {
        pointer.gone(&session)?;
        return Ok(Some(
            "dragging across the picture did not turn the model: the yaw or the picture stayed \
             the same."
                .to_owned(),
        ));
    }

    let (_, zoom_before) = rendered(&session, "zoom")?;
    pointer.wheel_in(&session, ivp.as_deref(), centre, 3.0)?;
    session.settle(20);
    let (_, zoom_after) = rendered(&session, "zoom")?;
    report.note(format!("wheel: zoom {zoom_before:?} → {zoom_after:?}"));
    if zoom_after == zoom_before {
        pointer.gone(&session)?;
        return Ok(Some("scrolling over the picture did not zoom.".to_owned()));
    }

    let trace = session.trace()?;
    let Some((top, tvp)) = declared_in(&trace, ui_rect, TOP_REGION) else {
        pointer.gone(&session)?;
        return Ok(Some(format!("no `{TOP_REGION}` (Top view) button.")));
    };
    pointer.click_in(&session, tvp.as_deref(), WindowPoint::centre_of(top))?;
    session.settle(20);
    // No picture: the viewer is an immediate viewport, which eframe 0.35
    // never screenshots.
    pointer.gone(&session)?;
    let (_, pitch) = rendered(&session, "pitch")?;
    drop(session);
    report.note(format!("top: pitch {pitch:?}"));
    match number(pitch.as_ref()) {
        Some(p) if p > 1.5 => Ok(None),
        _ => Ok(Some(format!(
            "the Top view left the pitch at {pitch:?}; looking down is about 1.55 radians."
        ))),
    }
}
