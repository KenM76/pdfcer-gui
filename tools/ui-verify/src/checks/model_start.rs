//! `the_3d_viewer_opens_walking_with_ghost_on_and_walks_by_keys` — with
//! `model_view_start = walk`, the preference's default, the 3D viewer opened
//! from its page is walking with ghost on before anything is pressed; and
//! the keys stand in for the drone sticks: D turns the body, the up arrow
//! flies forward, W rises, L turns the camera and U centres it, K tilts it
//! and O levels it.

use crate::checks::driving::declared_in;
use crate::checks::model_axes::{launch, opened_from_page};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const IMAGE: &str = "model3d.image";
/// A traced angle or coordinate that moved: past its print precision.
const MOVED: f64 = 0.002;

/// See the module documentation.
pub struct TheViewerOpensWalking;

impl Check for TheViewerOpensWalking {
    fn name(&self) -> &'static str {
        "the_3d_viewer_opens_walking_with_ghost_on_and_walks_by_keys"
    }

    fn defect(&self) -> &'static str {
        "The 3D viewer opens orbiting, or walking with ghost off, when the preferences ask it to \
         open walking"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive."))?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, "model_view_start = walk")
        .map_err(|e| Error::new(format!("could not plant the preferences file: {e}")))?;
    let (session, pointer) = launch(ctx, "model-start")?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome = opened_from_page(ctx, report, &session, &pointer, ui_rect).and_then(|f| {
        if f.is_some() {
            return Ok(f);
        }
        session.settle(20);
        let trace = session.trace()?;
        let walk = trace.events("model-walk").last().map(|l| l.raw.clone());
        let prefs = trace
            .events("model-nav-prefs")
            .last()
            .map(|l| l.raw.clone());
        report.note(format!("walk: {walk:?}; prefs: {prefs:?}"));
        Ok(if !walk.as_deref().is_some_and(|w| w.contains("on=true")) {
            Some(format!("the viewer opened without walking: {walk:?}."))
        } else if !prefs.as_deref().is_some_and(|p| p.contains("ghost=true")) {
            Some(format!(
                "the viewer opened walking with ghost off: {prefs:?}."
            ))
        } else {
            return keys(report, ui_rect, &session, &pointer);
        })
    });
    pointer.gone(&session)?;
    outcome
}

/// One key and what it must change on the walker's trace: the field, and
/// whether it must end at zero rather than merely move.
const KEYS: [(&str, &str, bool); 6] = [
    ("D", "yaw", false),
    ("ArrowUp", "eye", false),
    ("W", "eye", false),
    ("L", "look-yaw", false),
    ("U", "look-yaw", true),
    ("K", "pitch", false),
];

fn keys(
    report: &mut CheckReport,
    ui_rect: &str,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let Some((_, vp)) = declared_in(&session.trace()?, ui_rect, IMAGE) else {
        return Ok(Some(format!(
            "no `{IMAGE}` region: the viewer drew no picture."
        )));
    };
    let vp = vp.as_deref();
    let mut steps: Vec<(&str, &str, bool)> = KEYS.to_vec();
    steps.push(("O", "pitch", true));
    for (key, field, to_zero) in steps {
        let before = walker(session)?;
        pointer.key(session, vp, key, None)?;
        session.settle(15);
        let after = walker(session)?;
        let (was, now) = (before.get(field).cloned(), after.get(field).cloned());
        report.note(format!("{key}: {field} {was:?} -> {now:?}"));
        let Some(now) = now else {
            return Ok(Some(format!("{key} left no walker trace with `{field}`.")));
        };
        let ok = if to_zero {
            now.iter().all(|v| v.abs() < MOVED)
        } else {
            was.as_ref()
                .is_none_or(|was| was.iter().zip(&now).any(|(a, b)| (a - b).abs() > MOVED))
        };
        if !ok {
            return Ok(Some(format!(
                "{key} did not {} `{field}`: {was:?} -> {now:?}.",
                if to_zero { "zero" } else { "change" }
            )));
        }
    }
    Ok(None)
}

/// The walker's last traced state, from whichever of the move and button
/// traces came last: each field as its numbers.
fn walker(session: &Session) -> Result<std::collections::HashMap<String, Vec<f64>>> {
    let trace = session.trace()?;
    let mut state = std::collections::HashMap::new();
    if let Some(line) = trace
        .events("model-walk-moved")
        .chain(trace.events("model-drone-button"))
        .max_by_key(|l| l.lineno)
    {
        for field in ["eye", "yaw", "look-yaw", "pitch"] {
            if let Some(v) = line.get(field) {
                let nums = v.split(',').filter_map(|n| n.parse().ok()).collect();
                state.insert(field.to_owned(), nums);
            }
        }
    }
    Ok(state)
}
