//! `checks::model_light_map` — **the 3D viewer's *Colour by light level*
//! measures the light over the model and draws the picture from it, and
//! switching it off draws the model as before**
//!
//! Opens `fixtures/glow-room.prc` in the viewer off the desktop, ticks
//! *Colour by light level* on the View tab, waits for the map, then
//! unticks it.
//!
//! Oracles, from the trace (the viewer is an immediate viewport no
//! screenshot reaches):
//!
//! - Opened: the last `model-view-rendered` line, once two reads agree,
//!   has `map=0`.
//! - On: a `model-light-map-ready` line with `triangles` above zero and
//!   `hi` above `lo` (a lit room is not lit evenly), then a picture line
//!   with `map` other than 0, `from=map` and a `hash` other than the
//!   opened one. The hash alone cannot tell: with the switch on the
//!   picture moves to the processor, whose lit picture differs from the
//!   card's, so a map measured but never drawn still changes it; `from`
//!   names the branch the frame took.
//! - Off: a picture line with `map=0` and the opened `hash`, the camera
//!   being where it was.

use crate::checks::driving::repo_fixture;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options::{close, reach};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const METHOD: &str = "python fixtures/glow-room.PROVENANCE.py <pdfcer.exe>";
/// `dialogs::model3d::lightmap::REGION_MAP`.
const SWITCH: &str = "model3d.light_map";
const READY: &str = "model-light-map-ready";
const FAILED: &str = "model-light-map-failed";
const RENDERED: &str = "model-view-rendered";
/// Settles waited for the map, which is measured off the frame loop.
const WAITS: usize = 120;

/// See the module documentation.
pub struct TheViewerColoursByLightLevel;

impl Check for TheViewerColoursByLightLevel {
    fn name(&self) -> &'static str {
        "the_3d_viewer_colours_the_model_by_light_level"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer has no Colour by light level, the map is never measured, the picture \
         is not drawn from it, or switching it off does not draw the model as before"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture("glow-room.prc", METHOD)
            .and_then(|m| {
                launch_with_model_file(ctx, &mut report, "model-light-map", &m.to_string_lossy())
            })
            .and_then(|(session, pointer)| {
                if let Some(failure) = open_viewer(ctx, &session, &pointer)? {
                    return Ok(Some(failure));
                }
                drive(ctx, &mut report, &session, &pointer)
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last picture line's `map`, `hash` and `from`; an absent field
/// reads as `-`.
fn drawn(session: &Session) -> Result<Option<[String; 3]>> {
    let trace = session.trace()?;
    Ok(trace
        .events(RENDERED)
        .last()
        .map(|l| ["map", "hash", "from"].map(|k| l.get(k).unwrap_or("-").to_owned())))
}

/// Waits for the last picture line to satisfy `wanted`.
fn picture_until(
    session: &Session,
    wanted: impl Fn(&[String; 3]) -> bool,
) -> Result<Option<[String; 3]>> {
    let mut last = None;
    for _ in 0..WAITS {
        last = drawn(session)?;
        if last.as_ref().is_some_and(&wanted) {
            break;
        }
        session.settle(15);
    }
    Ok(last)
}

/// The opened picture once two reads a settle apart agree: the viewport
/// changes height while its panels lay out, and each height is a new
/// picture.
fn steady(session: &Session) -> Result<Option<[String; 3]>> {
    let mut last = drawn(session)?;
    for _ in 0..WAITS {
        session.settle(15);
        let now = drawn(session)?;
        if now.is_some() && now == last {
            break;
        }
        last = now;
    }
    Ok(last)
}

/// Tick the switch; `false` when it is drawn nowhere.
fn toggle(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<bool> {
    let Some((rect, vp)) = reach(session, pointer, ui_rect, SWITCH)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(15);
    close(session, pointer, ui_rect)?;
    Ok(true)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    let Some(first) = steady(session)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened [map, hash, from]: {first:?}"));
    if first[0] != "0" {
        return Ok(Some(format!(
            "the viewer opened with map={}; expected 0, the model drawn as usual.",
            first[0]
        )));
    }
    if !toggle(session, pointer, ui_rect)? {
        return Ok(Some(format!(
            "★★★ no *Colour by light level* switch (`{SWITCH}`) in the options menu."
        )));
    }
    let mut ready = None;
    for _ in 0..WAITS {
        let trace = session.trace()?;
        if let Some(line) = trace.last(READY) {
            ready = Some(line.clone());
            break;
        }
        if let Some(line) = trace.last(FAILED) {
            return Ok(Some(format!("★★★ the light map failed: {}", line.raw)));
        }
        session.settle(15);
    }
    let Some(ready) = ready else {
        return Ok(Some(
            "★★★ the switch is on but no light map arrived.".to_owned(),
        ));
    };
    report.note(format!("map: {}", ready.raw));
    let number = |k: &str| ready.get(k).and_then(|v| v.parse::<f64>().ok());
    match (number("triangles"), number("lo"), number("hi")) {
        (Some(n), Some(lo), Some(hi)) if n > 0.0 && hi > lo => {}
        _ => {
            return Ok(Some(format!(
                "★★★ the map is empty or flat: triangles, lo and hi in `{}`.",
                ready.raw
            )));
        }
    }
    let Some(on) = picture_until(session, |d| d[0] != "0" && d[0] != "-")? else {
        return Ok(Some("no picture line after the map arrived.".to_owned()));
    };
    report.note(format!("on [map, hash, from]: {on:?}"));
    if on[0] == "0" || on[0] == "-" || on[2] != "map" || on[1] == first[1] {
        return Ok(Some(format!(
            "★★★ the map arrived but the picture drew map={} from={} with hash {} against \
             {}; expected it drawn from the map into a new picture.",
            on[0], on[2], on[1], first[1]
        )));
    }
    if !toggle(session, pointer, ui_rect)? {
        return Ok(Some(format!("the `{SWITCH}` switch went away once on.")));
    }
    let Some(off) = picture_until(session, |d| d[0] == "0")? else {
        return Ok(Some("no picture line after switching off.".to_owned()));
    };
    report.note(format!("off [map, hash, from]: {off:?}"));
    if off[..2] != first[..2] {
        return Ok(Some(format!(
            "switched off, the picture drew map={} with hash {} against the opened {}; \
             expected the model drawn as before.",
            off[0], off[1], first[1]
        )));
    }
    Ok(None)
}
