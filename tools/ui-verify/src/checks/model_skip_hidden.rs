//! `checks::model_skip_hidden` — **the depth prepass hides nothing visible**
//!
//! Places `fixtures/floor-lamp.prc` (a box on a floor, lit by its own lamp,
//! so the floor runs behind the box), opens it with *View…*, and clears
//! *Skip lighting hidden surfaces* in the options menu.
//!
//! Oracles, from the `model-view-rendered` line:
//!
//! - Opened: `gpu-prepass=1`, drawn by the card with no `gpu-declined`.
//! - Cleared: `gpu-prepass=0`, still on the card, and `covered` within
//!   0.5% of the prepass picture's. A prepass whose depth texture is unbound,
//!   misindexed or never cleared reads as "an opaque surface in front" at
//!   every pixel and drops the faces, so the prepass picture covers far
//!   less. Exact equality is not asked: on a multisampled buffer the
//!   prepass may drop edge samples at silhouettes (`pdfcer_3d::gpu_prepass`).
//!
//! What this cannot see: a prepass that draws nothing and so skips nothing
//! gives the same picture as a working one; only frame time tells them apart.

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
/// `dialogs::model3d::looks::REGION_SKIP_HIDDEN`.
const SKIP_HIDDEN: &str = "model3d.skip_hidden";

/// See the module documentation.
pub struct TheDepthPrepassHidesNothingVisible;

impl Check for TheDepthPrepassHidesNothingVisible {
    fn name(&self) -> &'static str {
        "the_depth_prepass_hides_nothing_visible"
    }

    fn defect(&self) -> &'static str {
        "skipping hidden surfaces on the graphics card drops visible parts of the 3D \
         picture, or the switch does not reach the card"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture(MODEL, METHOD)
            .and_then(|model| {
                launch_with_model_file(ctx, &mut report, "skip-hidden", &model.to_string_lossy())
            })
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

const FIELDS: [&str; 4] = ["gpu-prepass", "gpu", "gpu-declined", "covered"];

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
    let Some(on) = rendered(session, FIELDS)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened {FIELDS:?}: {on:?}"));
    if on[..3] != ["1", "1", "-"] {
        return Ok(Some(format!(
            "the viewer opened with gpu-prepass={} gpu={} declined={}; expected the prepass \
             on, drawn on the card.",
            on[0], on[1], on[2]
        )));
    }
    if !viewer_options::press(session, pointer, ui_rect, SKIP_HIDDEN)? {
        return Ok(Some(format!(
            "no `{SKIP_HIDDEN}` box in the options menu of a model the card draws."
        )));
    }
    session.settle(20);
    let Some(off) = rendered(session, FIELDS)? else {
        return Ok(Some("no picture line after clearing the box.".to_owned()));
    };
    report.note(format!("prepass off: {off:?}"));
    if off[..3] != ["0", "1", "-"] {
        return Ok(Some(format!(
            "clearing the box drew gpu-prepass={} gpu={} declined={}; expected the card \
             drawing without the prepass.",
            off[0], off[1], off[2]
        )));
    }
    let count = |s: &str| s.parse::<i64>().unwrap_or(-1);
    let (with, without) = (count(&on[3]), count(&off[3]));
    if without <= 0 || (with - without).abs() * 200 > without {
        return Ok(Some(format!(
            "the prepass picture covers {with} pixels and the picture without it {without}: \
             the prepass hid surfaces that show."
        )));
    }
    Ok(None)
}
