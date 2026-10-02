//! `checks::model_view_window` — **the 3D viewer zooms about the pointer, and
//! minimises, fills the screen and comes back**
//!
//! Drives the window off the desktop through the scripted pointer. The engine
//! corpus's `assembly.prc` is placed from the ribbon and opened with *View…*.
//!
//! Oracles are the viewer's `model-view-rendered` lines. A wheel turn to the
//! right of and above the picture's centre must zoom and shift the view
//! right and up (`pan` both positive), which is what keeps the point under the
//! pointer still; a zoom about the centre pans nothing. The Full screen button
//! must render a larger picture; Minimize must trace the window minimised, and
//! restoring it through the OS without activation must trace it back with the
//! camera it had; Fit pressed there must restore zoom 1 and no
//! pan; Escape must bring the picture back to the size it had while the
//! viewer keeps rendering and traces no `model-view-closed`; Close must then
//! trace one. Escape closing the window loses the scripted key's
//! acknowledgement, so a failed key with a close line is that defect, not a
//! harness fault.

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
const FULL_SCREEN_REGION: &str = "model3d.full_screen";
const FIT_REGION: &str = "model3d.fit";
const CLOSE_REGION: &str = "model3d.close";
const CLOSED: &str = "model-view-closed";
const MINIMIZE_REGION: &str = "model3d.minimize";
const WINDOW: &str = "model-view-window";
/// The camera's fields, as both the rendered and the window lines carry them.
const CAMERA: [&str; 4] = ["yaw", "pitch", "zoom", "pan"];
const MODEL_ENV: &str = "PDFCER_DIAG_MODEL_PATH";
const DOC: &str = "D:/Dev/pdfcer/fixtures/synthetic/pageops/four-pages.pdf";
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc";
const RENDERED: &str = "model-view-rendered";

/// See the module documentation.
pub struct TheModelViewerZoomsAtThePointerAndFillsTheScreen;

impl Check for TheModelViewerZoomsAtThePointerAndFillsTheScreen {
    fn name(&self) -> &'static str {
        "the_model_viewer_zooms_at_the_pointer_and_fills_the_screen"
    }

    fn defect(&self) -> &'static str {
        "scrolling in the 3D viewer zooms about the picture's centre instead of the point under \
         the pointer, or the viewer has no way to fill the screen, or Escape closes it instead \
         of leaving full screen"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_with_model(ctx, &mut report, "model-window")
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Launch on a copy of the four-page document with `assembly.prc` as the
/// model picker's answer, and place it from the ribbon.
pub(crate) fn launch_with_model(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    for needed in [DOC, MODEL] {
        if !std::path::Path::new(needed).is_file() {
            return Err(Error::new(format!(
                "the engine corpus's fixture is missing at {needed}."
            )));
        }
    }
    // Driven on copies: the sources belong to the engine repository.
    let doc = ctx.out(&format!("{stem}-source.pdf"));
    std::fs::copy(DOC, &doc).map_err(|e| Error::new(format!("copying {DOC}: {e}")))?;
    let model = ctx.out(&format!("{stem}-input.prc"));
    std::fs::copy(MODEL, &model).map_err(|e| Error::new(format!("copying {MODEL}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.env
        .push((MODEL_ENV.to_owned(), model.to_string_lossy().into_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((session, pointer))
}

/// Press the ribbon's 3D model button, raising the Edit tab first if needed.
pub(crate) fn press_insert(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<bool> {
    let ui_rect = ui_rect(ctx)?;
    let mut trace = session.trace()?;
    if declared_in(&trace, ui_rect, RIBBON_ITEM).is_none()
        && let Some((tab, vp)) = declared_in(&trace, ui_rect, RIBBON_TAB)
    {
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(tab))?;
        session.settle(15);
        trace = session.trace()?;
    }
    let Some((item, vp)) = declared_in(&trace, ui_rect, RIBBON_ITEM) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(item))?;
    session.settle(30);
    Ok(true)
}

fn ui_rect(ctx: &CheckContext) -> Result<&'static str> {
    ctx.profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))
}

/// The last `model-view-rendered` line's `key`, as a number.
fn last(session: &Session, key: &str) -> Result<Option<f64>> {
    Ok(session
        .trace()?
        .events(RENDERED)
        .last()
        .and_then(|l| l.get(key).and_then(|v| v.parse().ok())))
}

/// The last rendered line's `pan` as (right, up).
fn pan(session: &Session) -> Result<Option<(f64, f64)>> {
    Ok(session.trace()?.events(RENDERED).last().and_then(|l| {
        let (x, y) = l.get("pan")?.split_once(',')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    }))
}

fn renders(session: &Session) -> Result<usize> {
    Ok(session.trace()?.events(RENDERED).count())
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some(format!("no `{RIBBON_ITEM}` region on the Edit tab.")));
    }
    let Some((view, vp)) = declared_in(&session.trace()?, ui_rect, VIEW_REGION) else {
        pointer.gone(session)?;
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region beside the placed model (look for `add-3d`)."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(view))?;
    session.settle(30);
    let Some((image, ivp)) = declared_in(&session.trace()?, ui_rect, IMAGE_REGION) else {
        pointer.gone(session)?;
        return Ok(Some(format!(
            "no `{IMAGE_REGION}` region: View… opened no viewer."
        )));
    };
    let failure = match zoom_at_pointer(report, session, pointer, image, ivp.as_deref())? {
        Some(failure) => Some(failure),
        None => match minimize_and_back(report, ctx, session, pointer)? {
            Some(failure) => Some(failure),
            None => full_screen(report, ctx, session, pointer)?,
        },
    };
    park(session, pointer, failure)
}

/// Move the pointer away. A failure already found outranks a parking error:
/// a step queued behind one a closed window never acknowledged can't land.
fn park(
    session: &Session,
    pointer: &ScriptedPointer,
    failure: Option<String>,
) -> Result<Option<String>> {
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// A wheel turn up and to the right of the centre zooms and pans toward it.
fn zoom_at_pointer(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    image: LRect,
    vp: Option<&str>,
) -> Result<Option<String>> {
    let centre = WindowPoint::centre_of(image);
    let aside = Pt {
        x: centre.x() + image.width() * 0.3,
        y: centre.y() - image.height() * 0.2,
    };
    let at = WindowPoint::centre_of(LRect {
        min: aside,
        max: aside,
    });
    let zoom_before = last(session, "zoom")?;
    let pan_before = pan(session)?;
    pointer.wheel_in(session, vp, at, 3.0)?;
    session.settle(20);
    let zoom_after = last(session, "zoom")?;
    let pan_after = pan(session)?;
    report.note(format!(
        "wheel right of and above centre: zoom {zoom_before:?} → {zoom_after:?}; pan \
         {pan_before:?} → {pan_after:?}"
    ));
    let Some((right, up)) = pan_after else {
        return Ok(Some(
            "the viewer's rendered line carries no `pan`, so where the zoom anchored cannot be \
             read: this is a viewer that zooms about the centre."
                .to_owned(),
        ));
    };
    if zoom_after.zip(zoom_before).is_none_or(|(a, b)| a <= b) {
        return Ok(Some(
            "scrolling over the picture did not zoom in.".to_owned(),
        ));
    }
    if right <= 0.0 || up <= 0.0 {
        return Ok(Some(format!(
            "★★★ a zoom up and to the right of the centre left pan at ({right}, {up}); both \
             must grow, or the point under the pointer slides away."
        )));
    }
    Ok(None)
}

/// The camera on the last `event` line whose `minimized` is `state`, or on
/// the last rendered line when `state` is `None`.
fn camera(session: &Session, event: &str, state: Option<&str>) -> Result<Option<Vec<String>>> {
    Ok(session
        .trace()?
        .events(event)
        .filter(|l| state.is_none_or(|s| l.get("minimized") == Some(s)))
        .last()
        .map(|l| {
            CAMERA
                .iter()
                .map(|k| l.get(k).unwrap_or_default().to_owned())
                .collect()
        }))
}

/// Wait up to five seconds for a window line with `minimized=state`.
fn await_window(session: &Session, state: &str) -> Result<bool> {
    for _ in 0..25 {
        if camera(session, WINDOW, Some(state))?.is_some() {
            return Ok(true);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    Ok(false)
}

/// Minimize shrinks the viewer to the taskbar; restoring it, as the taskbar
/// does but without taking the operator's focus, brings back the same camera.
fn minimize_and_back(
    report: &mut CheckReport,
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let Some((button, vp)) = declared_in(&session.trace()?, ui_rect(ctx)?, MINIMIZE_REGION) else {
        return Ok(Some(format!(
            "★★★ no `{MINIMIZE_REGION}` button in the viewer's toolbar."
        )));
    };
    let before = camera(session, RENDERED, None)?;
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(button))?;
    if !await_window(session, "true")? {
        return Ok(Some("★★★ Minimize did not minimise the viewer.".to_owned()));
    }
    let iconic: Vec<_> = crate::sys::windows_for_pid(session.pid())
        .into_iter()
        .filter(|w| crate::sys::is_minimized(*w))
        .collect();
    report.note(format!(
        "minimised windows of the process: {}",
        iconic.len()
    ));
    if iconic.is_empty() {
        return Ok(Some(
            "★★★ the viewer traced minimised but the OS shows no minimised window.".to_owned(),
        ));
    }
    iconic.into_iter().for_each(crate::sys::restore_quietly);
    if !await_window(session, "false")? {
        return Ok(Some(
            "★★★ the viewer did not come back from the taskbar.".to_owned(),
        ));
    }
    let after = camera(session, WINDOW, Some("false"))?;
    report.note(format!("minimise and back: camera {before:?} → {after:?}"));
    Ok((after != before || before.is_none()).then(|| {
        format!("★★★ the viewer came back with a different camera: {before:?} → {after:?}.")
    }))
}

/// The picture's last rendered size.
fn size(session: &Session) -> Result<(Option<f64>, Option<f64>)> {
    Ok((last(session, "w")?, last(session, "h")?))
}

fn closed(session: &Session) -> Result<usize> {
    Ok(session.trace()?.events(CLOSED).count())
}

/// Full screen grows the picture, Fit inside it frames the model again,
/// Escape restores the size and keeps the viewer open, and Close shuts it.
fn full_screen(
    report: &mut CheckReport,
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    let Some((button, bvp)) = declared_in(&session.trace()?, ui_rect, FULL_SCREEN_REGION) else {
        return Ok(Some(format!(
            "★★★ no `{FULL_SCREEN_REGION}` button in the viewer's toolbar."
        )));
    };
    let before = size(session)?;
    pointer.click_in(session, bvp.as_deref(), WindowPoint::centre_of(button))?;
    session.settle(40);
    let full = size(session)?;
    let asked = session
        .trace()?
        .events("model-view-full-screen")
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("full screen: {before:?} → {full:?}; {asked:?}"));
    let area = |(w, h): (Option<f64>, Option<f64>)| w.zip(h).map_or(0.0, |(w, h)| w * h);
    if area(full) <= area(before) {
        return Ok(Some(format!(
            "★★★ Full screen did not render a larger picture: {before:?} → {full:?}."
        )));
    }
    if let Some(failure) = fit(report, ui_rect, session, pointer)? {
        return Ok(Some(failure));
    }
    leave_and_close(
        report,
        ui_rect,
        session,
        pointer,
        bvp.as_deref(),
        (before, full),
    )
}

/// Fit, pressed while full screen, undoes the zoom and the pan.
fn fit(
    report: &mut CheckReport,
    ui_rect: &str,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let Some((button, vp)) = declared_in(&session.trace()?, ui_rect, FIT_REGION) else {
        return Ok(Some(format!(
            "★★★ no `{FIT_REGION}` button reachable while full screen."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(button))?;
    session.settle(30);
    let (zoom, panned) = (last(session, "zoom")?, pan(session)?);
    report.note(format!("fit in full screen: zoom {zoom:?}, pan {panned:?}"));
    let fitted = zoom.is_some_and(|z| (z - 1.0).abs() < 1e-3)
        && panned.is_some_and(|(x, y)| x == 0.0 && y == 0.0);
    Ok((!fitted).then(|| {
        format!(
            "★★★ Fit in full screen left zoom {zoom:?} and pan {panned:?}; it must frame the whole model again (zoom 1, pan 0,0)."
        )
    }))
}

type Sizes = ((Option<f64>, Option<f64>), (Option<f64>, Option<f64>));

/// Escape restores the windowed size without closing; Close then closes.
fn leave_and_close(
    report: &mut CheckReport,
    ui_rect: &str,
    session: &Session,
    pointer: &ScriptedPointer,
    vp: Option<&str>,
    (before, full): Sizes,
) -> Result<Option<String>> {
    let count = renders(session)?;
    if let Err(why) = pointer.key(session, vp, "Escape", None) {
        if closed(session)? > 0 {
            return Ok(Some(
                "★★★ Escape in full screen closed the viewer instead of leaving full screen."
                    .to_owned(),
            ));
        }
        return Err(why);
    }
    session.settle(40);
    let back = size(session)?;
    let shut = closed(session)?;
    report.note(format!("escape: {full:?} → {back:?}; closed lines {shut}"));
    if shut > 0 || renders(session)? <= count {
        return Ok(Some(
            "★★★ after Escape the viewer closed or rendered nothing more: Escape must leave full screen, not close the window."
                .to_owned(),
        ));
    }
    if back != before {
        return Ok(Some(format!(
            "★★★ Escape did not bring the picture back to its size: {before:?} → {full:?} → {back:?}."
        )));
    }
    let Some((close, cvp)) = declared_in(&session.trace()?, ui_rect, CLOSE_REGION) else {
        return Ok(Some(format!("no `{CLOSE_REGION}` button in the viewer.")));
    };
    pointer.click_in(session, cvp.as_deref(), WindowPoint::centre_of(close))?;
    session.settle(30);
    let how = session
        .trace()?
        .events(CLOSED)
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("close: {how:?}"));
    Ok(how
        .is_none()
        .then(|| "★★★ the viewer's Close button did not close it.".to_owned()))
}
