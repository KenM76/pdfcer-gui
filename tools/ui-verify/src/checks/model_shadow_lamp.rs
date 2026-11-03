//! `checks::model_shadow_lamp` — **a model's own lamp casts its shadow on
//! the graphics card**
//!
//! Places `fixtures/floor-lamp.prc` (the floor-box scene with one point
//! light of its own) and opens it with *View…*, where its lamp casts by
//! default. Then clears *Lamps cast shadows* in the options menu.
//!
//! Oracles, from the `model-view-rendered` line (the viewer is an immediate
//! viewport no screenshot reaches):
//!
//! - Opened: `lamp-shadows=1`, `gpu=1` with no `gpu-declined`, and
//!   `gpu-lamps=1` — the lamp's cube map drawn by the card.
//! - After clearing the box, the control: `lamp-shadows=0 gpu-lamps=0`, and
//!   a different picture `hash`, so the lamp's shadow was in the picture.

use crate::checks::driving::repo_fixture;
use crate::checks::model_shadow_sun::{open, rendered};
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "floor-lamp.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe> lamp`.";
/// `dialogs::model3d::looks::REGION_LAMP_SHADOWS`.
const LAMP_SHADOWS: &str = "model3d.lamp_shadows";

/// See the module documentation.
pub struct ALampCastsOnTheCard;

impl Check for ALampCastsOnTheCard {
    fn name(&self) -> &'static str {
        "a_models_lamp_casts_on_the_card"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer draws a model's own lamp without its shadow, or draws it off the \
         graphics card"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture(MODEL, METHOD)
            .and_then(|model| {
                launch_with_model_file(ctx, &mut report, "shadow-lamp", &model.to_string_lossy())
            })
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

const FIELDS: [&str; 5] = ["lamp-shadows", "gpu", "gpu-lamps", "gpu-declined", "hash"];

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
    let Some(first) = rendered(session, FIELDS)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened {FIELDS:?}: {first:?}"));
    if first[..4] != ["1", "1", "1", "-"] {
        return Ok(Some(format!(
            "a model with one lamp opened with lamp-shadows={} gpu={} gpu-lamps={} declined={}; \
             expected its lamp casting on the card.",
            first[0], first[1], first[2], first[3]
        )));
    }
    if !viewer_options::press(session, pointer, ui_rect, LAMP_SHADOWS)? {
        return Ok(Some(format!(
            "no `{LAMP_SHADOWS}` box in the options menu of a model with a lamp."
        )));
    }
    session.settle(20);
    let Some(off) = rendered(session, FIELDS)? else {
        return Ok(Some("no picture line after clearing the box.".to_owned()));
    };
    report.note(format!("lamps cast nothing: {off:?}"));
    if off[..3] != ["0", "1", "0"] || off[4] == first[4] {
        return Ok(Some(format!(
            "clearing Lamps cast shadows drew lamp-shadows={} gpu={} gpu-lamps={} with hash {} \
             against {}: the lamp's shadow is not in the picture.",
            off[0], off[1], off[2], off[4], first[4]
        )));
    }
    Ok(None)
}
