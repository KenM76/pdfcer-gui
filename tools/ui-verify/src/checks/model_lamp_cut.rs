//! `checks::model_lamp_cut` — **the lamp reach choice reaches the picture**
//!
//! Places `fixtures/floor-lamp.prc` (the floor-box scene with one point
//! light of its own), opens it with *View…*, and picks the shortest
//! *Lamps reach* choice in the options menu.
//!
//! Oracles, from the `model-view-rendered` line (the viewer is an immediate
//! viewport no screenshot reaches):
//!
//! - Opened: `lamp-cut=auto`, the engine's default, drawn by the card.
//! - After the pick: `lamp-cut=1600`, still `gpu=1` with no `gpu-declined`,
//!   and a different picture `hash`, so the cut reached the shader rather
//!   than only the trace.

use crate::checks::driving::{declared_in, repo_fixture};
use crate::checks::model_shadow_sun::{open, rendered};
use crate::checks::model_view_window::{launch_with_model_file, ui_rect};
use crate::checks::viewer_options;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const MODEL: &str = "floor-lamp.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe> lamp`.";
/// `dialogs::model3d::looks::REGION_LAMP_CUT`.
const LAMP_CUT: &str = "model3d.lamp_cut";
/// Its shortest choice, sixteen 8-bit levels.
const SHORTEST: &str = "model3d.lamp_cut.1600";

/// See the module documentation.
pub struct TheLampReachChoiceReachesThePicture;

impl Check for TheLampReachChoiceReachesThePicture {
    fn name(&self) -> &'static str {
        "the_lamp_reach_choice_reaches_the_picture"
    }

    fn defect(&self) -> &'static str {
        "picking how far a model's lamps reach leaves the 3D picture unchanged, or sends it \
         off the graphics card"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = repo_fixture(MODEL, METHOD)
            .and_then(|model| {
                launch_with_model_file(ctx, &mut report, "lamp-cut", &model.to_string_lossy())
            })
            .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer));
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

const FIELDS: [&str; 4] = ["lamp-cut", "gpu", "gpu-declined", "hash"];

/// Pick `item` from the *Lamps reach* drop-down; `false` when the drop-down
/// or the item is drawn nowhere.
fn choose(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, item: &str) -> Result<bool> {
    let Some((combo, vp)) = viewer_options::reach(session, pointer, ui_rect, LAMP_CUT)? else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(combo))?;
    session.settle(20);
    let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect, item) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
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
    let Some(first) = rendered(session, FIELDS)? else {
        return Ok(Some("the viewer drew no picture line.".to_owned()));
    };
    report.note(format!("opened {FIELDS:?}: {first:?}"));
    if first[..3] != ["auto", "1", "-"] {
        return Ok(Some(format!(
            "a model with one lamp opened with lamp-cut={} gpu={} declined={}; expected the \
             engine's own cut, drawn on the card.",
            first[0], first[1], first[2]
        )));
    }
    if !choose(session, pointer, ui_rect, SHORTEST)? {
        return Ok(Some(format!(
            "no `{SHORTEST}` choice under `{LAMP_CUT}` in the options menu of a model with a lamp."
        )));
    }
    let Some(cut) = rendered(session, FIELDS)? else {
        return Ok(Some("no picture line after the pick.".to_owned()));
    };
    report.note(format!("shortest reach: {cut:?}"));
    if cut[..3] != ["1600", "1", "-"] || cut[3] == first[3] {
        return Ok(Some(format!(
            "picking the shortest reach drew lamp-cut={} gpu={} declined={} with hash {} \
             against {}: the cut did not reach the card's picture.",
            cut[0], cut[1], cut[2], cut[3], first[3]
        )));
    }
    Ok(None)
}
